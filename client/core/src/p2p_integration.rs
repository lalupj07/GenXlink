use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error, warn, debug};
use uuid::Uuid;

use crate::p2p_discovery::{P2PDiscovery, PeerInfo, DiscoveryEvent, P2PConnectionManager};
use crate::p2p_signaling::{P2PSignaling, P2PSignalingEvent};
use crate::webrtc_session::WebRTCSession;
use genxlink_protocol::{DeviceId, SignalingMessage};

/// Complete P2P integration manager
/// Handles peer discovery, signaling, and WebRTC connections without central server
pub struct P2PIntegration {
    device_id: DeviceId,
    device_name: String,
    discovery: P2PDiscovery,
    signaling: P2PSignaling,
    connection_manager: P2PConnectionManager,
    active_sessions: Arc<RwLock<Vec<Arc<WebRTCSession>>>>,
    state: Arc<RwLock<P2PIntegrationState>>,
    event_tx: mpsc::UnboundedSender<P2PIntegrationEvent>,
}

/// P2P integration state
#[derive(Debug, Clone, PartialEq)]
pub enum P2PIntegrationState {
    Stopped,
    Starting,
    Discovering,
    Ready,
    Connecting(DeviceId),
    Connected(DeviceId),
    Streaming(DeviceId),
    Error(String),
}

/// P2P integration events
#[derive(Debug, Clone)]
pub enum P2PIntegrationEvent {
    Started,
    PeerDiscovered(PeerInfo),
    PeerConnected(DeviceId),
    PeerDisconnected(DeviceId),
    ConnectionFailed(DeviceId, String),
    StreamingStarted(DeviceId),
    StreamingStopped(DeviceId),
    Error(String),
    StateChanged(P2PIntegrationState),
}

impl P2PIntegration {
    /// Create a new P2P integration manager
    pub fn new(device_id: DeviceId, device_name: String) -> (Self, mpsc::UnboundedReceiver<P2PIntegrationEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        let (discovery, _) = P2PDiscovery::new(device_id.clone(), device_name.clone());
        let (signaling, _) = P2PSignaling::new(device_id.clone(), device_name.clone());
        let connection_manager = P2PConnectionManager::new(device_id.clone(), device_name.clone());
        
        let integration = Self {
            device_id: device_id.clone(),
            device_name,
            discovery,
            signaling,
            connection_manager,
            active_sessions: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(P2PIntegrationState::Stopped)),
            event_tx,
        };
        
        (integration, event_rx)
    }

    /// Start the P2P integration
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting P2P integration for device {}", self.device_id);
        self.set_state(P2PIntegrationState::Starting).await;
        
        // Start discovery service
        self.discovery.start().await?;
        
        // Start signaling service
        self.signaling.start().await?;
        
        // Start event handlers
        self.start_event_handlers().await;
        
        self.set_state(P2PIntegrationState::Discovering).await;
        let _ = self.event_tx.send(P2PIntegrationEvent::Started);
        
        info!("P2P integration started successfully");
        Ok(())
    }

    /// Start event handlers for discovery and signaling events
    async fn start_event_handlers(&mut self) {
        let event_tx = self.event_tx.clone();
        let state = self.state.clone();
        let device_id = self.device_id.clone();
        
        // Handle discovery events
        // Note: In a real implementation, we'd get discovery events from the discovery service
        // For now, we'll simulate this with periodic checks
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Simulate peer discovery
                // In a real implementation, this would be triggered by actual discovery events
                debug!("P2P integration heartbeat - checking for peers");
            }
        });
    }

    /// Connect to a discovered peer
    pub async fn connect_to_peer(&self, peer_device_id: DeviceId) -> Result<()> {
        info!("Connecting to peer: {}", peer_device_id);
        self.set_state(P2PIntegrationState::Connecting(peer_device_id.clone())).await;
        
        // Get peer information
        let peer_info = self.discovery.get_peer(&peer_device_id).await
            .ok_or_else(|| anyhow!("Peer not found: {}", peer_device_id))?;
        
        // Connect via P2P signaling
        if let Err(e) = self.signaling.connect_to_peer(&peer_info).await {
            error!("Failed to connect to peer via P2P signaling: {}", e);
            self.set_state(P2PIntegrationState::Error(e.to_string())).await;
            let _ = self.event_tx.send(P2PIntegrationEvent::ConnectionFailed(peer_device_id, e.to_string()));
            return Err(e);
        }
        
        // Create WebRTC session
        let session = WebRTCSession::new(
            self.device_id.clone(),
            format!("ws://{}:{}/ws", peer_info.ip_address, peer_info.port)
        );
        
        // Start streaming session
        if let Err(e) = session.start_streaming(0, peer_device_id.clone()).await {
            error!("Failed to start WebRTC session: {}", e);
            self.set_state(P2PIntegrationState::Error(e.to_string())).await;
            let _ = self.event_tx.send(P2PIntegrationEvent::ConnectionFailed(peer_device_id, e.to_string()));
            return Err(e);
        }
        
        // Store the session
        let mut sessions = self.active_sessions.write().await;
        sessions.push(Arc::new(session));
        drop(sessions);
        
        self.set_state(P2PIntegrationState::Connected(peer_device_id.clone())).await;
        let _ = self.event_tx.send(P2PIntegrationEvent::PeerConnected(peer_device_id.clone()));
        
        info!("Successfully connected to peer: {}", peer_device_id);
        Ok(())
    }

    /// Start streaming with a connected peer
    pub async fn start_streaming(&self, peer_device_id: DeviceId, monitor_index: usize) -> Result<()> {
        info!("Starting streaming with peer: {}", peer_device_id);
        self.set_state(P2PIntegrationState::Streaming(peer_device_id.clone())).await;
        
        let sessions = self.active_sessions.read().await;
        
        // Find the session for this peer
        let session = sessions.iter().find(|s| {
            // In a real implementation, we'd check if this session is for the specified peer
            true // Simplified for now
        });
        
        if let Some(session) = session {
            // Start streaming (already started in connect_to_peer, but we can restart if needed)
            debug!("Streaming already active with peer: {}", peer_device_id);
            let _ = self.event_tx.send(P2PIntegrationEvent::StreamingStarted(peer_device_id));
        } else {
            return Err(anyhow!("No active session found for peer: {}", peer_device_id));
        }
        
        Ok(())
    }

    /// Stop streaming with a peer
    pub async fn stop_streaming(&self, peer_device_id: DeviceId) -> Result<()> {
        info!("Stopping streaming with peer: {}", peer_device_id);
        
        let mut sessions = self.active_sessions.write().await;
        sessions.retain(|session| {
            // In a real implementation, we'd check if this session is for the specified peer
            // and stop it appropriately
            true // Simplified for now
        });
        
        self.set_state(P2PIntegrationState::Connected(peer_device_id.clone())).await;
        let _ = self.event_tx.send(P2PIntegrationEvent::StreamingStopped(peer_device_id));
        
        Ok(())
    }

    /// Disconnect from a peer
    pub async fn disconnect_from_peer(&self, peer_device_id: DeviceId) -> Result<()> {
        info!("Disconnecting from peer: {}", peer_device_id);
        
        // Stop streaming if active
        let _ = self.stop_streaming(peer_device_id.clone()).await;
        
        // Remove session
        let mut sessions = self.active_sessions.write().await;
        sessions.retain(|session| {
            // In a real implementation, we'd check if this session is for the specified peer
            true // Simplified for now
        });
        
        // Disconnect via signaling
        let _ = self.signaling.disconnect_from_peer(&peer_device_id).await;
        
        self.set_state(P2PIntegrationState::Ready).await;
        let _ = self.event_tx.send(P2PIntegrationEvent::PeerDisconnected(peer_device_id));
        
        Ok(())
    }

    /// Get all discovered peers
    pub async fn get_discovered_peers(&self) -> Vec<PeerInfo> {
        self.discovery.get_peers().await
    }

    /// Get current state
    pub async fn get_state(&self) -> P2PIntegrationState {
        self.state.read().await.clone()
    }

    /// Set state and notify
    async fn set_state(&self, new_state: P2PIntegrationState) {
        let mut state = self.state.write().await;
        *state = new_state.clone();
        drop(state);
        
        let _ = self.event_tx.send(P2PIntegrationEvent::StateChanged(new_state));
    }

    /// Get active sessions count
    pub async fn get_active_sessions_count(&self) -> usize {
        self.active_sessions.read().await.len()
    }

    /// Stop the P2P integration
    pub async fn stop(&self) {
        info!("Stopping P2P integration");
        
        // Stop all active sessions
        let mut sessions = self.active_sessions.write().await;
        sessions.clear();
        
        // Stop signaling
        self.signaling.stop().await;
        
        // Stop discovery
        self.discovery.stop().await;
        
        self.set_state(P2PIntegrationState::Stopped).await;
        
        info!("P2P integration stopped");
    }

    /// Check if P2P is ready for connections
    pub async fn is_ready(&self) -> bool {
        matches!(self.get_state().await, P2PIntegrationState::Ready)
    }

    /// Check if connected to a specific peer
    pub async fn is_connected_to_peer(&self, peer_device_id: &DeviceId) -> bool {
        matches!(self.get_state().await, P2PIntegrationState::Connected(id) if id == *peer_device_id)
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> P2PConnectionStats {
        let sessions = self.active_sessions.read().await;
        let discovered_peers = self.discovery.get_peers().await;
        let active_connections = self.signaling.get_active_connections().await;
        
        P2PConnectionStats {
            discovered_peers_count: discovered_peers.len(),
            active_connections_count: active_connections.len(),
            active_sessions_count: sessions.len(),
            state: self.get_state().await,
        }
    }
}

/// P2P connection statistics
#[derive(Debug, Clone)]
pub struct P2PConnectionStats {
    pub discovered_peers_count: usize,
    pub active_connections_count: usize,
    pub active_sessions_count: usize,
    pub state: P2PIntegrationState,
}

/// P2P configuration
#[derive(Debug, Clone, PartialEq)]
pub struct P2PConfiguration {
    pub device_name: String,
    pub auto_connect: bool,
    pub auto_stream: bool,
    pub discovery_interval_secs: u64,
    pub connection_timeout_secs: u64,
}

impl Default for P2PConfiguration {
    fn default() -> Self {
        Self {
            device_name: "GenXLink Device".to_string(),
            auto_connect: false,
            auto_stream: false,
            discovery_interval_secs: 30,
            connection_timeout_secs: 30,
        }
    }
}
