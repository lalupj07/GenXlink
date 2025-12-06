use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};
use tracing::{info, error, warn, debug};

use crate::webrtc_session::{WebRTCSession, SessionState};
use crate::streaming::{VideoStreamer, Frame};
use crate::encoder::{H264Encoder, EncoderConfig, VideoCodec};
use crate::capture::{ScreenCapture, DxgiCapture};
use crate::signaling_client::{SignalingClient, SignalingState};
use crate::auth_service::{AuthService, AuthSession};
use crate::p2p_integration::{P2PIntegration, P2PIntegrationEvent, P2PConfiguration};
use crate::config::ServerConfig;
use genxlink_protocol::{SignalingMessage, DeviceId};

/// Complete WebRTC integration manager
/// Handles the full flow: Authentication → Signaling → WebRTC → Streaming
/// Supports both server-based and P2P modes
pub struct WebRTCIntegration {
    device_id: DeviceId,
    auth_service: Arc<Mutex<AuthService>>,
    session: Arc<Mutex<Option<WebRTCSession>>>,
    video_streamer: Arc<Mutex<Option<VideoStreamer>>>,
    screen_capture: Arc<Mutex<Option<DxgiCapture>>>,
    p2p_integration: Option<Arc<Mutex<P2PIntegration>>>,
    connection_mode: ConnectionMode,
    state: Arc<RwLock<IntegrationState>>,
    event_tx: mpsc::UnboundedSender<IntegrationEvent>,
}

/// Connection mode for WebRTC integration
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionMode {
    ServerBased(String), // Signaling server URL
    P2P(P2PConfiguration),
}

/// Integration state
#[derive(Debug, Clone, PartialEq)]
pub enum IntegrationState {
    Unauthenticated,
    Authenticating,
    Authenticated,
    Connecting,
    SignalingConnected,
    WebRTCConnecting,
    Streaming,
    Disconnected,
    Failed(String),
}

/// Integration events
#[derive(Debug, Clone)]
pub enum IntegrationEvent {
    AuthenticationRequired,
    Authenticated,
    Connected,
    StreamingStarted,
    StreamingStopped,
    Error(String),
    StateChanged(IntegrationState),
}

impl WebRTCIntegration {
    /// Create a new WebRTC integration manager with server-based mode
    pub fn new_server_based(
        device_id: DeviceId,
        auth_service: AuthService,
        server_config: ServerConfig,
    ) -> (Self, mpsc::UnboundedReceiver<IntegrationEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        let integration = Self {
            device_id: device_id.clone(),
            auth_service: Arc::new(Mutex::new(auth_service)),
            session: Arc::new(Mutex::new(None)),
            video_streamer: Arc::new(Mutex::new(None)),
            screen_capture: Arc::new(Mutex::new(None)),
            p2p_integration: None,
            connection_mode: ConnectionMode::ServerBased(server_config.api_server_url),
            state: Arc::new(RwLock::new(IntegrationState::Unauthenticated)),
            event_tx,
        };
        
        (integration, event_rx)
    }

    /// Create a new WebRTC integration manager with P2P mode
    pub fn new_p2p(
        device_id: DeviceId,
        auth_service: AuthService,
        p2p_config: P2PConfiguration,
    ) -> (Self, mpsc::UnboundedReceiver<IntegrationEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        let (p2p_integration, _) = P2PIntegration::new(device_id.clone(), p2p_config.device_name.clone());
        
        let integration = Self {
            device_id: device_id.clone(),
            auth_service: Arc::new(Mutex::new(auth_service)),
            session: Arc::new(Mutex::new(None)),
            video_streamer: Arc::new(Mutex::new(None)),
            screen_capture: Arc::new(Mutex::new(None)),
            p2p_integration: Some(Arc::new(Mutex::new(p2p_integration))),
            connection_mode: ConnectionMode::P2P(p2p_config),
            state: Arc::new(RwLock::new(IntegrationState::Unauthenticated)),
            event_tx,
        };
        
        (integration, event_rx)
    }

    /// Initialize the integration with authentication
    pub async fn initialize(&self) -> Result<()> {
        self.set_state(IntegrationState::Authenticating).await;
        
        // Check if we have a valid session
        let auth = self.auth_service.lock().await;
        if auth.get_current_session().is_none() {
            self.send_event(IntegrationEvent::AuthenticationRequired).await;
            return Err(anyhow::anyhow!("Authentication required"));
        }
        drop(auth);
        
        self.set_state(IntegrationState::Authenticated).await;
        self.send_event(IntegrationEvent::Authenticated).await;
        
        // Initialize P2P integration if in P2P mode
        if let Some(p2p_integration) = &self.p2p_integration {
            info!("Initializing P2P integration");
            let mut p2p = p2p_integration.lock().await;
            p2p.start().await?;
            info!("P2P integration initialized successfully");
        }
        
        info!("WebRTC integration initialized successfully");
        Ok(())
    }

    /// Start a complete streaming session
    pub async fn start_streaming(
        &self,
        remote_device_id: DeviceId,
        monitor_index: usize,
        server_config: Option<ServerConfig>,
    ) -> Result<()> {
        self.set_state(IntegrationState::Connecting).await;
        
        match &self.connection_mode {
            ConnectionMode::ServerBased(_) => {
                let config = server_config.unwrap_or_else(|| ServerConfig::default());
                self.start_server_based_streaming(remote_device_id, monitor_index, config).await
            }
            ConnectionMode::P2P(_) => {
                self.start_p2p_streaming(remote_device_id, monitor_index).await
            }
        }
    }

    /// Start server-based streaming
    async fn start_server_based_streaming(
        &self,
        remote_device_id: DeviceId,
        monitor_index: usize,
        server_config: ServerConfig,
    ) -> Result<()> {
        // Create WebRTC session with server-based signaling
        let session = WebRTCSession::new(self.device_id.clone(), server_config.signaling_server_url);
        
        if let Err(e) = session.start_streaming(monitor_index, remote_device_id.clone()).await {
            error!("Failed to start server-based streaming: {}", e);
            self.set_state(IntegrationState::Failed(e.to_string())).await;
            return Err(e);
        }
        
        // Store session
        let mut session_guard = self.session.lock().await;
        *session_guard = Some(session);
        drop(session_guard);
        
        self.set_state(IntegrationState::Streaming).await;
        self.send_event(IntegrationEvent::StreamingStarted).await;
        
        info!("Server-based streaming started successfully");
        Ok(())
    }

    /// Start P2P streaming
    async fn start_p2p_streaming(
        &self,
        remote_device_id: DeviceId,
        monitor_index: usize,
    ) -> Result<()> {
        if let Some(p2p_integration) = &self.p2p_integration {
            info!("Starting P2P streaming with device: {}", remote_device_id);
            
            let p2p = p2p_integration.lock().await;
            
            // Connect to peer
            p2p.connect_to_peer(remote_device_id.clone()).await?;
            
            // Start streaming
            p2p.start_streaming(remote_device_id.clone(), monitor_index).await?;
            
            self.set_state(IntegrationState::Streaming).await;
            self.send_event(IntegrationEvent::StreamingStarted).await;
            
            info!("P2P streaming started successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!("P2P integration not available"))
        }
    }

    /// Stop the current streaming session
    pub async fn stop_streaming(&self) -> Result<()> {
        info!("Stopping streaming session");
        
        // Stop P2P streaming if in P2P mode
        if let Some(p2p_integration) = &self.p2p_integration {
            let p2p = p2p_integration.lock().await;
            // TODO: Get current peer device ID and stop streaming
            info!("Stopping P2P streaming");
        }
        
        // Stop screen capture
        let mut capture_guard = self.screen_capture.lock().await;
        if let Some(_capture) = capture_guard.as_mut() {
            // TODO: Implement stop method for screen capture
            *capture_guard = None;
        }
        drop(capture_guard);
        
        // Stop video streamer
        let mut streamer_guard = self.video_streamer.lock().await;
        *streamer_guard = None;
        drop(streamer_guard);
        
        // Stop WebRTC session
        let mut session_guard = self.session.lock().await;
        *session_guard = None;
        drop(session_guard);
        
        self.set_state(IntegrationState::Disconnected).await;
        self.send_event(IntegrationEvent::StreamingStopped).await;
        
        info!("Streaming session stopped");
        Ok(())
    }

    /// Set up video streaming components
    async fn setup_video_streaming(&self) -> Result<()> {
        // Create H.264 encoder
        let encoder = H264Encoder::new();
        
        // Create video streamer
        let video_streamer = VideoStreamer::new(Box::new(encoder))
            .context("Failed to create video streamer")?;
        
        // Create screen capture
        let screen_capture = DxgiCapture::new();
        
        // Store components
        let mut streamer_guard = self.video_streamer.lock().await;
        *streamer_guard = Some(video_streamer);
        drop(streamer_guard);
        
        let mut capture_guard = self.screen_capture.lock().await;
        *capture_guard = Some(screen_capture);
        drop(capture_guard);
        
        info!("Video streaming components set up successfully");
        Ok(())
    }

    /// Stream a frame to the remote peer
    pub async fn stream_frame(&self, frame: Frame) -> Result<()> {
        let streamer_guard = self.video_streamer.lock().await;
        if let Some(streamer) = streamer_guard.as_ref() {
            // Encode the frame first
            let _streamer_clone = streamer.clone();
            drop(streamer_guard);
            
            // TODO: Implement proper frame encoding
            // For now, just log the frame
            debug!("Streaming frame: {}x{}", frame.width, frame.height);
        } else {
            warn!("Video streamer not initialized");
        }
        Ok(())
    }

    /// Get current integration state
    pub async fn get_state(&self) -> IntegrationState {
        self.state.read().await.clone()
    }

    /// Check if currently streaming
    pub async fn is_streaming(&self) -> bool {
        matches!(self.get_state().await, IntegrationState::Streaming)
    }

    /// Get current session info
    pub async fn get_session_info(&self) -> Option<SessionInfo> {
        let session_guard = self.session.lock().await;
        if let Some(session) = session_guard.as_ref() {
            Some(SessionInfo {
                device_id: self.device_id.clone(),
                state: session.get_state().await,
            })
        } else {
            None
        }
    }

    /// Set integration state
    async fn set_state(&self, state: IntegrationState) {
        let mut state_guard = self.state.write().await;
        *state_guard = state.clone();
        drop(state_guard);
        
        self.send_event(IntegrationEvent::StateChanged(state)).await;
    }

    /// Send an event
    async fn send_event(&self, event: IntegrationEvent) {
        if let Err(e) = self.event_tx.send(event) {
            error!("Failed to send integration event: {}", e);
        }
    }

    /// Handle authentication success
    pub async fn on_authenticated(&self, _session: AuthSession) -> Result<()> {
        let auth = self.auth_service.lock().await;
        // TODO: Implement session setting in auth service
        drop(auth);
        
        self.set_state(IntegrationState::Authenticated).await;
        self.send_event(IntegrationEvent::Authenticated).await;
        
        info!("Authentication successful");
        Ok(())
    }

    /// Handle connection lost
    pub async fn on_connection_lost(&self) -> Result<()> {
        warn!("Connection lost, attempting cleanup");
        
        // Stop streaming if active
        if self.is_streaming().await {
            self.stop_streaming().await?;
        }
        
        self.set_state(IntegrationState::Disconnected).await;
        Ok(())
    }

    /// Restart streaming with current session
    pub async fn restart_streaming(&self) -> Result<()> {
        info!("Restarting streaming session");
        
        if self.is_streaming().await {
            self.stop_streaming().await?;
        }
        
        // TODO: Implement restart logic with stored remote device ID
        warn!("Restart streaming not fully implemented - need to store remote device ID");
        
        Ok(())
    }
}

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub device_id: DeviceId,
    pub state: SessionState,
}

/// Streaming statistics
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    pub frames_sent: u64,
    pub bytes_sent: u64,
    pub fps: f32,
    pub bitrate: u32,
    pub duration: std::time::Duration,
}

impl WebRTCIntegration {
    /// Get streaming statistics
    pub async fn get_streaming_stats(&self) -> StreamingStats {
        // TODO: Implement stats collection from video streamer
        StreamingStats::default()
    }

    /// Update streaming quality based on network conditions
    pub async fn update_quality(&self, bandwidth_kbps: u32) -> Result<()> {
        info!("Updating streaming quality based on bandwidth: {} kbps", bandwidth_kbps);
        
        // TODO: Implement quality adjustment
        warn!("Quality adjustment not implemented");
        
        Ok(())
    }

    /// Get discovered peers (P2P mode only)
    pub async fn get_discovered_peers(&self) -> Vec<crate::p2p_discovery::PeerInfo> {
        if let Some(p2p_integration) = &self.p2p_integration {
            let p2p = p2p_integration.lock().await;
            p2p.get_discovered_peers().await
        } else {
            Vec::new()
        }
    }

    /// Check if P2P mode is enabled
    pub fn is_p2p_mode(&self) -> bool {
        matches!(self.connection_mode, ConnectionMode::P2P(_))
    }

    /// Get connection mode
    pub fn get_connection_mode(&self) -> &ConnectionMode {
        &self.connection_mode
    }

    /// Connect to a specific peer in P2P mode
    pub async fn connect_to_p2p_peer(&self, peer_device_id: DeviceId) -> Result<()> {
        if let Some(p2p_integration) = &self.p2p_integration {
            let p2p = p2p_integration.lock().await;
            p2p.connect_to_peer(peer_device_id).await
        } else {
            Err(anyhow::anyhow!("P2P integration not available"))
        }
    }

    /// Disconnect from a P2P peer
    pub async fn disconnect_from_p2p_peer(&self, peer_device_id: DeviceId) -> Result<()> {
        if let Some(p2p_integration) = &self.p2p_integration {
            let p2p = p2p_integration.lock().await;
            p2p.disconnect_from_peer(peer_device_id).await
        } else {
            Err(anyhow::anyhow!("P2P integration not available"))
        }
    }

    /// Get P2P connection statistics
    pub async fn get_p2p_stats(&self) -> Option<crate::p2p_integration::P2PConnectionStats> {
        if let Some(p2p_integration) = &self.p2p_integration {
            let p2p = p2p_integration.lock().await;
            Some(p2p.get_connection_stats().await)
        } else {
            None
        }
    }

    /// Enable/disable hardware acceleration
    pub async fn set_hardware_acceleration(&self, enabled: bool) -> Result<()> {
        info!("Setting hardware acceleration: {}", enabled);
        
        // TODO: Implement hardware acceleration toggle
        // This would require recreating the encoder with different settings
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerConfig, Environment};

    #[tokio::test]
    async fn test_webrtc_integration_creation() {
        let device_id = "test-device".to_string();
        let auth_service = AuthService::new(
            "http://localhost:8000".to_string(),
            "test-key".to_string(),
        );
        
        let server_config = ServerConfig::development();
        
        let (integration, _event_rx) = WebRTCIntegration::new_server_based(
            genxlink_protocol::DeviceId(device_id),
            auth_service,
            server_config,
        );
        
        assert_eq!(integration.get_state().await, IntegrationState::Unauthenticated);
        assert!(!integration.is_streaming().await);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let device_id = "test-device".to_string();
        let auth_service = AuthService::new(
            "http://localhost:8000".to_string(),
            "test-key".to_string(),
        );
        
        let server_config = ServerConfig::development();
        
        let (_integration, mut event_rx) = WebRTCIntegration::new_server_based(
            genxlink_protocol::DeviceId(device_id),
            auth_service,
            server_config,
        );
        
        // Should receive authentication required event
        let event = event_rx.recv().await.unwrap();
        assert!(matches!(event, IntegrationEvent::AuthenticationRequired));
    }

    #[tokio::test]
    async fn test_production_config() {
        let config = ServerConfig::production();
        assert!(config.is_production());
        assert!(config.api_server_url.starts_with("https://"));
        assert!(config.signaling_server_url.starts_with("wss://"));
    }

    #[tokio::test]
    async fn test_development_config() {
        let config = ServerConfig::development();
        assert!(config.is_development());
        assert!(config.api_server_url.starts_with("http://"));
        assert!(config.signaling_server_url.starts_with("ws://"));
    }
}
