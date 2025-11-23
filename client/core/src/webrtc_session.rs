use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_VP8};
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;

use crate::screen_streamer::ScreenStreamer;
use crate::signaling_client::{SignalingClient, SignalingState};
use genxlink_protocol::{SignalingMessage, DeviceId};

/// WebRTC streaming session
/// Manages the complete flow: signaling → peer connection → streaming
pub struct WebRTCSession {
    device_id: DeviceId,
    signaling: Arc<Mutex<SignalingClient>>,
    peer_connection: Arc<Mutex<Option<Arc<RTCPeerConnection>>>>,
    screen_streamer: Arc<Mutex<Option<ScreenStreamer>>>,
    state: Arc<RwLock<SessionState>>,
}

/// Session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Idle,
    ConnectingToSignaling,
    SignalingConnected,
    CreatingOffer,
    WaitingForAnswer,
    GatheringCandidates,
    Connected,
    Streaming,
    Disconnecting,
    Disconnected,
    Failed(String),
}

impl WebRTCSession {
    /// Create a new WebRTC session
    pub fn new(device_id: DeviceId, signaling_server_url: String) -> Self {
        let signaling = SignalingClient::new(device_id.clone(), signaling_server_url);
        
        Self {
            device_id,
            signaling: Arc::new(Mutex::new(signaling)),
            peer_connection: Arc::new(Mutex::new(None)),
            screen_streamer: Arc::new(Mutex::new(None)),
            state: Arc::new(RwLock::new(SessionState::Idle)),
        }
    }

    /// Start a streaming session
    pub async fn start_streaming(&self, monitor_index: usize, remote_device_id: DeviceId) -> Result<()> {
        self.set_state(SessionState::ConnectingToSignaling).await;
        
        // Step 1: Connect to signaling server
        tracing::info!("Connecting to signaling server...");
        let mut signaling = self.signaling.lock().await;
        let mut incoming_rx = signaling.connect().await
            .context("Failed to connect to signaling server")?;
        drop(signaling);
        
        self.set_state(SessionState::SignalingConnected).await;
        
        // Step 2: Create peer connection
        tracing::info!("Creating peer connection...");
        let peer_connection = self.create_peer_connection().await?;
        
        // Step 3: Create screen streamer and add track
        tracing::info!("Setting up screen streaming...");
        let streamer = ScreenStreamer::new()?;
        let video_track = streamer.get_track();
        
        // Add track to peer connection
        peer_connection.add_track(Arc::clone(&video_track) as Arc<dyn webrtc::track::track_local::TrackLocal + Send + Sync>).await
            .context("Failed to add video track")?;
        
        // Store streamer
        let mut streamer_guard = self.screen_streamer.lock().await;
        *streamer_guard = Some(streamer);
        drop(streamer_guard);
        
        // Step 4: Create and send offer
        self.set_state(SessionState::CreatingOffer).await;
        tracing::info!("Creating SDP offer...");
        
        let offer = peer_connection.create_offer(None).await
            .context("Failed to create offer")?;
        
        peer_connection.set_local_description(offer.clone()).await
            .context("Failed to set local description")?;
        
        // Send offer via signaling
        let signaling = self.signaling.lock().await;
        signaling.send_message(SignalingMessage::Offer {
            from: self.device_id.clone(),
            to: remote_device_id.clone(),
            sdp: offer.sdp,
        }).await?;
        drop(signaling);
        
        self.set_state(SessionState::WaitingForAnswer).await;
        tracing::info!("Offer sent, waiting for answer...");
        
        // Step 5: Handle incoming signaling messages
        let peer_conn_clone = Arc::clone(&peer_connection);
        let state_clone = Arc::clone(&self.state);
        let streamer_clone = Arc::clone(&self.screen_streamer);
        
        tokio::spawn(async move {
            while let Some(msg) = incoming_rx.recv().await {
                match msg {
                    SignalingMessage::Answer { sdp, .. } => {
                        tracing::info!("Received answer");
                        
                        let answer = RTCSessionDescription::answer(sdp)
                            .expect("Failed to create answer description");
                        
                        if let Err(e) = peer_conn_clone.set_remote_description(answer).await {
                            tracing::error!("Failed to set remote description: {}", e);
                            continue;
                        }
                        
                        // Start streaming
                        let mut state = state_clone.write().await;
                        *state = SessionState::Streaming;
                        drop(state);
                        
                        // Start screen capture
                        if let Some(streamer) = streamer_clone.lock().await.as_ref() {
                            if let Err(e) = streamer.start_streaming(monitor_index).await {
                                tracing::error!("Failed to start streaming: {}", e);
                            } else {
                                tracing::info!("Screen streaming started!");
                            }
                        }
                    }
                    SignalingMessage::IceCandidate { candidate, sdp_mid, sdp_m_line_index, .. } => {
                        tracing::debug!("Received ICE candidate");
                        
                        // Create ICE candidate init
                        let ice_init = RTCIceCandidateInit {
                            candidate,
                            sdp_mid,
                            sdp_mline_index: sdp_m_line_index,
                            username_fragment: None,
                        };
                        
                        if let Err(e) = peer_conn_clone.add_ice_candidate(ice_init).await {
                            tracing::error!("Failed to add ICE candidate: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        });
        
        // Store peer connection
        let mut pc_guard = self.peer_connection.lock().await;
        *pc_guard = Some(peer_connection);
        
        Ok(())
    }

    /// Create a peer connection with proper configuration
    async fn create_peer_connection(&self) -> Result<Arc<RTCPeerConnection>> {
        // Create media engine with VP8 support
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()
            .context("Failed to register codecs")?;
        
        // Create API
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .build();
        
        // Configure ICE servers
        let config = RTCConfiguration {
            ice_servers: vec![
                RTCIceServer {
                    urls: vec![
                        "stun:stun.l.google.com:19302".to_owned(),
                        "stun:stun1.l.google.com:19302".to_owned(),
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        
        // Create peer connection
        let peer_connection = Arc::new(
            api.new_peer_connection(config).await
                .context("Failed to create peer connection")?
        );
        
        // Set up connection state handler
        let state_clone = Arc::clone(&self.state);
        peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let state = state_clone.clone();
            Box::pin(async move {
                tracing::info!("Peer connection state changed: {:?}", s);
                
                let mut state_guard = state.write().await;
                match s {
                    RTCPeerConnectionState::Connected => {
                        *state_guard = SessionState::Connected;
                    }
                    RTCPeerConnectionState::Disconnected => {
                        *state_guard = SessionState::Disconnected;
                    }
                    RTCPeerConnectionState::Failed => {
                        *state_guard = SessionState::Failed("Connection failed".to_string());
                    }
                    _ => {}
                }
            })
        }));
        
        // Set up ICE candidate handler
        let signaling = Arc::clone(&self.signaling);
        let device_id = self.device_id.clone();
        
        peer_connection.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
            let signaling = signaling.clone();
            let device_id = device_id.clone();
            
            Box::pin(async move {
                if let Some(candidate) = candidate {
                    tracing::debug!("New ICE candidate: {:?}", candidate);
                    
                    let signaling = signaling.lock().await;
                    let candidate_json = candidate.to_json().unwrap_or_default();
                    if let Err(e) = signaling.send_message(SignalingMessage::IceCandidate {
                        from: device_id,
                        to: DeviceId::default(), // Will be set by server
                        candidate: candidate_json.candidate,
                        sdp_mid: candidate_json.sdp_mid,
                        sdp_m_line_index: candidate_json.sdp_mline_index,
                    }).await {
                        tracing::error!("Failed to send ICE candidate: {}", e);
                    }
                }
            })
        }));
        
        Ok(peer_connection)
    }

    /// Stop streaming
    pub async fn stop_streaming(&self) -> Result<()> {
        self.set_state(SessionState::Disconnecting).await;
        
        // Stop screen streamer
        if let Some(streamer) = self.screen_streamer.lock().await.as_ref() {
            streamer.stop_streaming().await?;
        }
        
        // Close peer connection
        if let Some(pc) = self.peer_connection.lock().await.as_ref() {
            pc.close().await?;
        }
        
        self.set_state(SessionState::Disconnected).await;
        
        Ok(())
    }

    /// Get current session state
    pub async fn get_state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    /// Set session state
    async fn set_state(&self, state: SessionState) {
        let mut s = self.state.write().await;
        *s = state;
    }
}
