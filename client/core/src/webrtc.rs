use crate::ClientError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_sender::RTCRtpSender;
use webrtc::track::track_local::TrackLocal;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use bytes::Bytes;

/// WebRTC connection manager
pub struct WebRTCManager {
    device_id: String,
    state: Arc<RwLock<ConnectionState>>,
    config: WebRTCConfig,
    peer_connection: Option<Arc<RTCPeerConnection>>,
    data_channels: Arc<RwLock<HashMap<String, Arc<RTCDataChannel>>>>,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    SignalingConnected,
    GatheringCandidates,
    Connected,
    Reconnecting,
    Failed(String),
    Closed,
}

/// WebRTC configuration
#[derive(Debug, Clone)]
pub struct WebRTCConfig {
    pub ice_servers: Vec<IceServer>,
    pub ice_transport_policy: IceTransportPolicy,
}

/// ICE server configuration
#[derive(Debug, Clone)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

/// ICE transport policy
#[derive(Debug, Clone)]
pub enum IceTransportPolicy {
    All,
    Relay,  // Force TURN
}

impl Default for WebRTCConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                IceServer {
                    urls: vec![
                        "stun:stun.l.google.com:19302".to_string(),
                        "stun:stun1.l.google.com:19302".to_string(),
                    ],
                    username: None,
                    credential: None,
                },
            ],
            ice_transport_policy: IceTransportPolicy::All,
        }
    }
}

impl WebRTCManager {
    /// Create a new WebRTC manager
    pub fn new(device_id: String, config: WebRTCConfig) -> Self {
        Self {
            device_id,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            config,
            peer_connection: None,
            data_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize peer connection
    pub async fn initialize(&mut self) -> Result<(), ClientError> {
        self.set_state(ConnectionState::Connecting).await;
        
        // Create API with default settings
        let api = APIBuilder::new().build();
        
        // Convert config to WebRTC format
        let ice_servers: Vec<RTCIceServer> = self.config.ice_servers
            .iter()
            .map(|server| RTCIceServer {
                urls: server.urls.clone(),
                username: server.username.clone().unwrap_or_default(),
                credential: server.credential.clone().unwrap_or_default(),
                ..Default::default()
            })
            .collect();
        
        let rtc_config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };
        
        // Create peer connection
        let peer_connection = Arc::new(
            api.new_peer_connection(rtc_config)
                .await
                .map_err(|e| ClientError::TransportError(format!("Failed to create peer connection: {}", e)))?
        );
        
        // Set up event handlers
        self.setup_event_handlers(&peer_connection).await?;
        
        self.peer_connection = Some(peer_connection);
        self.set_state(ConnectionState::SignalingConnected).await;
        
        Ok(())
    }
    
    /// Setup event handlers for peer connection
    async fn setup_event_handlers(&self, pc: &Arc<RTCPeerConnection>) -> Result<(), ClientError> {
        let state = self.state.clone();
        
        // Handle peer connection state changes
        pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let state = state.clone();
            Box::pin(async move {
                tracing::info!("Peer connection state changed: {:?}", s);
                let new_state = match s {
                    RTCPeerConnectionState::New => ConnectionState::Connecting,
                    RTCPeerConnectionState::Connecting => ConnectionState::Connecting,
                    RTCPeerConnectionState::Connected => ConnectionState::Connected,
                    RTCPeerConnectionState::Disconnected => ConnectionState::Reconnecting,
                    RTCPeerConnectionState::Failed => ConnectionState::Failed("Connection failed".to_string()),
                    RTCPeerConnectionState::Closed => ConnectionState::Closed,
                    _ => return,
                };
                
                let mut s = state.write().await;
                *s = new_state;
            })
        }));
        
        Ok(())
    }

    /// Get current connection state
    pub async fn get_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Set connection state
    async fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.write().await;
        *state = new_state;
    }

    /// Add video track to peer connection
    pub async fn add_video_track(&self, track: Arc<dyn TrackLocal + Send + Sync>) -> Result<Arc<RTCRtpSender>, ClientError> {
        let pc = self.peer_connection.as_ref()
            .ok_or_else(|| ClientError::TransportError("Peer connection not initialized".to_string()))?;
        
        let sender = pc.add_track(track)
            .await
            .map_err(|e| ClientError::WebRTCError(format!("Failed to add video track: {}", e)))?;
        
        tracing::info!("Video track added to peer connection");
        
        Ok(sender)
    }

    /// Create an offer (initiator side)
    pub async fn create_offer(&self) -> Result<String, ClientError> {
        let pc = self.peer_connection.as_ref()
            .ok_or_else(|| ClientError::TransportError("Peer connection not initialized".to_string()))?;
        
        self.set_state(ConnectionState::GatheringCandidates).await;
        
        // Create offer
        let offer = pc.create_offer(None)
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to create offer: {}", e)))?;
        
        // Set local description
        pc.set_local_description(offer.clone())
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to set local description: {}", e)))?;
        
        Ok(offer.sdp)
    }

    /// Set remote answer (initiator side)
    pub async fn set_remote_answer(&mut self, sdp: String) -> Result<(), ClientError> {
        let pc = self.peer_connection.as_ref()
            .ok_or_else(|| ClientError::TransportError("Peer connection not initialized".to_string()))?;
        
        let answer = RTCSessionDescription::answer(sdp)
            .map_err(|e| ClientError::TransportError(format!("Invalid SDP: {}", e)))?;
        
        pc.set_remote_description(answer)
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to set remote description: {}", e)))?;
        
        Ok(())
    }

    /// Create an answer (responder side)
    pub async fn create_answer(&mut self, offer_sdp: String) -> Result<String, ClientError> {
        let pc = self.peer_connection.as_ref()
            .ok_or_else(|| ClientError::TransportError("Peer connection not initialized".to_string()))?;
        
        // Set remote description (offer)
        let offer = RTCSessionDescription::offer(offer_sdp)
            .map_err(|e| ClientError::TransportError(format!("Invalid SDP: {}", e)))?;
        
        pc.set_remote_description(offer)
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to set remote description: {}", e)))?;
        
        // Create answer
        let answer = pc.create_answer(None)
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to create answer: {}", e)))?;
        
        // Set local description
        pc.set_local_description(answer.clone())
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to set local description: {}", e)))?;
        
        Ok(answer.sdp)
    }

    /// Add ICE candidate
    pub async fn add_ice_candidate(&mut self, candidate: String) -> Result<(), ClientError> {
        let pc = self.peer_connection.as_ref()
            .ok_or_else(|| ClientError::TransportError("Peer connection not initialized".to_string()))?;
        
        let ice_candidate = RTCIceCandidateInit {
            candidate,
            ..Default::default()
        };
        
        pc.add_ice_candidate(ice_candidate)
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to add ICE candidate: {}", e)))?;
        
        Ok(())
    }
    
    /// Create a data channel
    pub async fn create_data_channel(&mut self, label: &str) -> Result<(), ClientError> {
        let pc = self.peer_connection.as_ref()
            .ok_or_else(|| ClientError::TransportError("Peer connection not initialized".to_string()))?;
        
        let data_channel = pc.create_data_channel(label, None)
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to create data channel: {}", e)))?;
        
        // Store data channel (already wrapped in Arc by webrtc crate)
        let mut channels = self.data_channels.write().await;
        channels.insert(label.to_string(), data_channel);
        
        Ok(())
    }

    /// Send data on a channel
    pub async fn send_data(&self, channel: &str, data: &[u8]) -> Result<(), ClientError> {
        let channels = self.data_channels.read().await;
        let dc = channels.get(channel)
            .ok_or_else(|| ClientError::TransportError(format!("Data channel '{}' not found", channel)))?;
        
        dc.send(&Bytes::copy_from_slice(data))
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to send data: {}", e)))?;
        
        Ok(())
    }
    
    /// Get ICE candidates channel
    pub async fn on_ice_candidate(&self) -> Result<mpsc::UnboundedReceiver<RTCIceCandidate>, ClientError> {
        let pc = self.peer_connection.as_ref()
            .ok_or_else(|| ClientError::TransportError("Peer connection not initialized".to_string()))?;
        
        let (tx, rx) = mpsc::unbounded_channel();
        
        pc.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
            let tx = tx.clone();
            Box::pin(async move {
                if let Some(c) = candidate {
                    let _ = tx.send(c);
                }
            })
        }));
        
        Ok(rx)
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<(), ClientError> {
        if let Some(pc) = &self.peer_connection {
            pc.close()
                .await
                .map_err(|e| ClientError::TransportError(format!("Failed to close connection: {}", e)))?;
        }
        
        self.set_state(ConnectionState::Closed).await;
        self.peer_connection = None;
        
        Ok(())
    }
}

/// Data channel handler trait
pub trait DataChannelHandler: Send + Sync {
    /// Handle incoming data
    fn on_data(&mut self, channel: &str, data: &[u8]);
    
    /// Handle channel open
    fn on_open(&mut self, channel: &str);
    
    /// Handle channel close
    fn on_close(&mut self, channel: &str);
}

/// Default implementation for testing
pub struct DefaultDataChannelHandler;

impl DataChannelHandler for DefaultDataChannelHandler {
    fn on_data(&mut self, channel: &str, data: &[u8]) {
        println!("Received {} bytes on channel '{}'", data.len(), channel);
    }
    
    fn on_open(&mut self, channel: &str) {
        println!("Channel '{}' opened", channel);
    }
    
    fn on_close(&mut self, channel: &str) {
        println!("Channel '{}' closed", channel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webrtc_manager_creation() {
        let config = WebRTCConfig::default();
        let manager = WebRTCManager::new("test-device".to_string(), config);
        
        let state = manager.get_state().await;
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let config = WebRTCConfig::default();
        let manager = WebRTCManager::new("test-device".to_string(), config);
        
        manager.set_state(ConnectionState::Connecting).await;
        let state = manager.get_state().await;
        assert_eq!(state, ConnectionState::Connecting);
    }
}
