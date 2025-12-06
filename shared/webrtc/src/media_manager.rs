use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use webrtc::{
    api::{
        media_engine::{MediaEngine, MIME_TYPE_VP8, MIME_TYPE_OPUS},
        peer_connection::{
            configuration::RTCConfiguration,
            peer_connection_state::RTCPeerConnectionState,
            sdp::session_description::RTCSessionDescriptionInit,
        },
        track::track_local::{track_local_static_rtp::TrackLocalStaticRTP, track_local_static_sample::TrackLocalStaticSample},
    },
    data_channel::{data_channel_init::RTCDataChannelInit, data_channel::RTCDataChannel},
    ice_transport::{
        ice_candidate::RTCIceCandidate,
        ice_server::RTCIceServer,
    },
    peer_connection::{RTCPeerConnection, peer_connection_factory::RTCPeerConnectionFactory},
    rtp::{packetizer::Packetizer, sequence_number_strategy::SequenceNumberStrategy},
    track::{
        track_remote::{track_remote_static_rtp::TrackRemoteStaticRTP, TrackRemote},
        track_local::TrackLocal,
    },
};

use crate::crypto::encryption::EncryptionManager;
use crate::protocol::{MediaMessage, MediaType, FileInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenShareConfig {
    pub width: u32,
    pub height: u32,
    pub frame_rate: u32,
    pub quality: u8, // 0-100
    pub cursor: bool,
    pub audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: u32,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferConfig {
    pub chunk_size: usize,
    pub max_file_size: u64,
    pub resume_support: bool,
    pub compression: bool,
}

#[derive(Debug)]
pub struct MediaSession {
    pub id: Uuid,
    pub peer_connection: Arc<RTCPeerConnection>,
    pub screen_track: Option<Arc<TrackLocalStaticRTP>>,
    pub audio_track: Option<Arc<TrackLocalStaticRTP>>,
    pub data_channel: Option<Arc<RTCDataChannel>>,
    pub encryption: Arc<EncryptionManager>,
    pub config: MediaSessionConfig,
}

#[derive(Debug, Clone)]
pub struct MediaSessionConfig {
    pub screen_share: Option<ScreenShareConfig>,
    pub audio: Option<AudioConfig>,
    pub file_transfer: Option<FileTransferConfig>,
}

pub struct MediaManager {
    peer_connections: Arc<RwLock<HashMap<Uuid, Arc<MediaSession>>>>,
    media_engine: Arc<MediaEngine>,
    factory: Arc<RTCPeerConnectionFactory>,
    encryption: Arc<EncryptionManager>,
    screen_capture: Arc<dyn ScreenCapture + Send + Sync>,
    audio_capture: Arc<dyn AudioCapture + Send + Sync>,
    file_manager: Arc<dyn FileManager + Send + Sync>,
}

#[async_trait::async_trait]
pub trait ScreenCapture {
    async fn start_capture(&self, config: ScreenShareConfig) -> Result<mpsc::Receiver<Vec<u8>>>;
    async fn stop_capture(&self) -> Result<()>;
    async fn get_screen_list(&self) -> Result<Vec<ScreenInfo>>;
}

#[async_trait::async_trait]
pub trait AudioCapture {
    async fn start_capture(&self, config: AudioConfig) -> Result<mpsc::Receiver<Vec<u8>>>;
    async fn stop_capture(&self) -> Result<()>;
    async fn get_device_list(&self) -> Result<Vec<AudioDeviceInfo>>;
}

#[async_trait::async_trait]
pub trait FileManager {
    async fn send_file(&self, file_info: FileInfo, data: Vec<u8>) -> Result<()>;
    async fn receive_file(&self, file_id: Uuid) -> Result<FileInfo>;
    async fn cancel_transfer(&self, file_id: Uuid) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: String, // "input" or "output"
    pub channels: u8,
    pub sample_rate: u32,
}

impl MediaManager {
    pub fn new(
        encryption: Arc<EncryptionManager>,
        screen_capture: Arc<dyn ScreenCapture + Send + Sync>,
        audio_capture: Arc<dyn AudioCapture + Send + Sync>,
        file_manager: Arc<dyn FileManager + Send + Sync>,
    ) -> Result<Self> {
        let mut media_engine = MediaEngine::default();
        
        // Register video codecs
        media_engine.register_default_codecs()?;
        
        // Create factory
        let factory = Arc::new(RTCPeerConnectionFactory::new(Arc::new(media_engine.clone()))?);

        Ok(Self {
            peer_connections: Arc::new(RwLock::new(HashMap::new())),
            media_engine: Arc::new(media_engine),
            factory,
            encryption,
            screen_capture,
            audio_capture,
            file_manager,
        })
    }

    pub async fn create_session(&self, config: MediaSessionConfig) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        
        // Configure ICE servers
        let ice_servers = vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
        ];

        let rtc_config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        // Create peer connection
        let peer_connection = Arc::new(self.factory.create_peer_connection(rtc_config).await?);

        // Create tracks based on config
        let mut screen_track = None;
        let mut audio_track = None;

        if let Some(screen_config) = &config.screen_share {
            screen_track = Some(self.create_screen_track(screen_config).await?);
        }

        if let Some(audio_config) = &config.audio {
            audio_track = Some(self.create_audio_track(audio_config).await?);
        }

        // Create data channel for file transfer
        let data_channel = if config.file_transfer.is_some() {
            Some(self.create_data_channel(&peer_connection).await?)
        } else {
            None
        };

        // Create media session
        let session = Arc::new(MediaSession {
            id: session_id,
            peer_connection,
            screen_track,
            audio_track,
            data_channel,
            encryption: self.encryption.clone(),
            config,
        });

        // Store session
        let mut sessions = self.peer_connections.write().await;
        sessions.insert(session_id, session.clone());

        info!("Created media session: {}", session_id);
        Ok(session_id)
    }

    async fn create_screen_track(&self, config: &ScreenShareConfig) -> Result<Arc<TrackLocalStaticRTP>> {
        let track = Arc::new(TrackLocalStaticRTP::new(
            "screen".to_string(),
            MIME_TYPE_VP8.to_string(),
            90000, // Video clock rate
        ));

        // Start screen capture
        let capture_rx = self.screen_capture.start_capture(config.clone()).await?;
        
        // Start encoding and sending loop
        let track_clone = track.clone();
        let encryption = self.encryption.clone();
        
        tokio::spawn(async move {
            Self::screen_capture_loop(capture_rx, track_clone, encryption).await;
        });

        Ok(track)
    }

    async fn create_audio_track(&self, config: &AudioConfig) -> Result<Arc<TrackLocalStaticRTP>> {
        let track = Arc::new(TrackLocalStaticRTP::new(
            "audio".to_string(),
            MIME_TYPE_OPUS.to_string(),
            48000, // Audio clock rate
        ));

        // Start audio capture
        let capture_rx = self.audio_capture.start_capture(config.clone()).await?;
        
        // Start encoding and sending loop
        let track_clone = track.clone();
        let encryption = self.encryption.clone();
        
        tokio::spawn(async move {
            Self::audio_capture_loop(capture_rx, track_clone, encryption).await;
        });

        Ok(track)
    }

    async fn create_data_channel(&self, peer_connection: &Arc<RTCPeerConnection>) -> Result<Arc<RTCDataChannel>> {
        let config = RTCDataChannelInit {
            ordered: Some(true),
            max_retransmits: Some(16),
            ..Default::default()
        };

        let data_channel = peer_connection.create_data_channel("file-transfer", Some(config)).await?;
        
        // Set up data channel handlers
        let dc_clone = data_channel.clone();
        let file_manager = self.file_manager.clone();
        let encryption = self.encryption.clone();
        
        data_channel.on_open(Box::new(move || {
            let dc = dc_clone.clone();
            let fm = file_manager.clone();
            let enc = encryption.clone();
            Box::pin(async move {
                Self::handle_data_channel(dc, fm, enc).await;
            })
        }));

        Ok(data_channel)
    }

    pub async fn start_screen_share(&self, session_id: Uuid, config: ScreenShareConfig) -> Result<()> {
        let sessions = self.peer_connections.read().await;
        let session = sessions.get(&session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        if let Some(track) = &session.screen_track {
            // Add track to peer connection
            session.peer_connection.add_track(track.clone()).await?;
            
            // Start capture if not already running
            let capture_rx = self.screen_capture.start_capture(config).await?;
            let track_clone = track.clone();
            let encryption = session.encryption.clone();
            
            tokio::spawn(async move {
                Self::screen_capture_loop(capture_rx, track_clone, encryption).await;
            });

            info!("Started screen sharing for session: {}", session_id);
            Ok(())
        } else {
            Err(anyhow!("Screen track not initialized for session: {}", session_id))
        }
    }

    pub async fn start_audio_stream(&self, session_id: Uuid, config: AudioConfig) -> Result<()> {
        let sessions = self.peer_connections.read().await;
        let session = sessions.get(&session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        if let Some(track) = &session.audio_track {
            // Add track to peer connection
            session.peer_connection.add_track(track.clone()).await?;
            
            // Start capture if not already running
            let capture_rx = self.audio_capture.start_capture(config).await?;
            let track_clone = track.clone();
            let encryption = session.encryption.clone();
            
            tokio::spawn(async move {
                Self::audio_capture_loop(capture_rx, track_clone, encryption).await;
            });

            info!("Started audio streaming for session: {}", session_id);
            Ok(())
        } else {
            Err(anyhow!("Audio track not initialized for session: {}", session_id))
        }
    }

    pub async fn send_file(&self, session_id: Uuid, file_info: FileInfo, data: Vec<u8>) -> Result<()> {
        let sessions = self.peer_connections.read().await;
        let session = sessions.get(&session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        if let Some(data_channel) = &session.data_channel {
            // Encrypt file data
            let encrypted_data = session.encryption.encrypt_data(&data).await?;
            
            // Create file message
            let message = MediaMessage {
                id: Uuid::new_v4(),
                session_id,
                media_type: MediaType::File,
                timestamp: chrono::Utc::now(),
                data: encrypted_data,
                metadata: serde_json::to_value(file_info)?,
            };

            // Send through data channel
            let message_bytes = serde_json::to_vec(&message)?;
            data_channel.send(&message_bytes).await?;
            
            info!("Sent file {} for session: {}", file_info.name, session_id);
            Ok(())
        } else {
            Err(anyhow!("Data channel not available for session: {}", session_id))
        }
    }

    pub async fn stop_session(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.peer_connections.write().await;
        if let Some(session) = sessions.remove(&session_id) {
            // Stop captures
            let _ = self.screen_capture.stop_capture().await;
            let _ = self.audio_capture.stop_capture().await;
            
            // Close peer connection
            session.peer_connection.close().await?;
            
            info!("Stopped media session: {}", session_id);
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    async fn screen_capture_loop(
        mut capture_rx: mpsc::Receiver<Vec<u8>>,
        track: Arc<TrackLocalStaticRTP>,
        encryption: Arc<EncryptionManager>,
    ) {
        info!("Starting screen capture loop");
        
        while let Some(frame_data) = capture_rx.recv().await {
            // Encrypt frame data
            match encryption.encrypt_data(&frame_data).await {
                Ok(encrypted_data) => {
                    // Create RTP packet with encrypted data
                    // This is simplified - in production, you'd use proper RTP packetization
                    debug!("Sending encrypted video frame: {} bytes", encrypted_data.len());
                    
                    // For now, we'll just log the data
                    // In a real implementation, you'd send this through the WebRTC track
                }
                Err(e) => {
                    error!("Failed to encrypt screen frame: {}", e);
                }
            }
        }
        
        info!("Screen capture loop ended");
    }

    async fn audio_capture_loop(
        mut capture_rx: mpsc::Receiver<Vec<u8>>,
        track: Arc<TrackLocalStaticRTP>,
        encryption: Arc<EncryptionManager>,
    ) {
        info!("Starting audio capture loop");
        
        while let Some(audio_data) = capture_rx.recv().await {
            // Encrypt audio data
            match encryption.encrypt_data(&audio_data).await {
                Ok(encrypted_data) => {
                    // Create RTP packet with encrypted data
                    debug!("Sending encrypted audio frame: {} bytes", encrypted_data.len());
                    
                    // For now, we'll just log the data
                    // In a real implementation, you'd send this through the WebRTC track
                }
                Err(e) => {
                    error!("Failed to encrypt audio frame: {}", e);
                }
            }
        }
        
        info!("Audio capture loop ended");
    }

    async fn handle_data_channel(
        data_channel: Arc<RTCDataChannel>,
        file_manager: Arc<dyn FileManager + Send + Sync>,
        encryption: Arc<EncryptionManager>,
    ) {
        info!("Handling data channel messages");
        
        let dc_clone = data_channel.clone();
        let fm = file_manager.clone();
        let enc = encryption.clone();
        
        data_channel.on_message(Box::new(move |msg| {
            let dc = dc_clone.clone();
            let fm = fm.clone();
            let enc = enc.clone();
            Box::pin(async move {
                match serde_json::from_slice::<MediaMessage>(&msg.data) {
                    Ok(message) => {
                        match message.media_type {
                            MediaType::File => {
                                // Decrypt file data
                                match enc.decrypt_data(&message.data).await {
                                    Ok(decrypted_data) => {
                                        // Extract file info from metadata
                                        if let Ok(file_info) = serde_json::from_value::<FileInfo>(message.metadata) {
                                            if let Err(e) = fm.send_file(file_info, decrypted_data).await {
                                                error!("Failed to handle received file: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to decrypt file data: {}", e);
                                    }
                                }
                            }
                            _ => {
                                warn!("Received unsupported media type: {:?}", message.media_type);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse media message: {}", e);
                    }
                }
            })
        }));
    }

    pub async fn get_session_list(&self) -> Vec<Uuid> {
        let sessions = self.peer_connections.read().await;
        sessions.keys().copied().collect()
    }

    pub async fn get_session_status(&self, session_id: Uuid) -> Result<Option<MediaSessionConfig>> {
        let sessions = self.peer_connections.read().await;
        Ok(sessions.get(&session_id).map(|s| s.config.clone()))
    }
}
