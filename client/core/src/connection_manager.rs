//! Connection Manager
//! 
//! Manages remote connections using Connection IDs.
//! This is the main entry point for connecting to remote PCs.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc, Mutex};
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use tracing::{info, error, warn, debug};

use crate::connection_id::{ConnectionId, get_connection_id};
use genxlink_protocol::DeviceId;

/// Signaling server URL (Railway deployment)
const SIGNALING_SERVER_URL: &str = "wss://genxlink-signaling.up.railway.app";

/// STUN/TURN servers for NAT traversal
const STUN_SERVERS: &[&str] = &[
    "stun:stun.l.google.com:19302",
    "stun:stun1.l.google.com:19302",
    "stun:stun2.l.google.com:19302",
];

const TURN_SERVER: &str = "turn:a.]relay.metered.ca:80";
const TURN_USERNAME: &str = "83eebabf8b4cce9d5dbcb649";
const TURN_CREDENTIAL: &str = "2D7JvfkOQtBdYW3R";

/// Connection state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    WaitingForPeer,
    Negotiating,
    Connected,
    Streaming,
    Failed(String),
}

/// Remote peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePeer {
    pub connection_id: String,
    pub device_name: String,
    pub state: ConnectionState,
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
    pub latency_ms: Option<u32>,
}

/// Connection events
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    StateChanged(ConnectionState),
    PeerConnected(RemotePeer),
    PeerDisconnected(String),
    ScreenFrameReceived(Vec<u8>),
    InputReceived(InputEvent),
    Error(String),
}

/// Input event from remote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseMove { x: i32, y: i32 },
    MouseClick { button: MouseButton, pressed: bool },
    MouseScroll { delta_x: i32, delta_y: i32 },
    KeyPress { key_code: u32, pressed: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Signaling message for WebRTC negotiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalingMessage {
    /// Register with the signaling server
    Register { connection_id: String, device_name: String },
    /// Request to connect to a peer
    ConnectRequest { from_id: String, to_id: String },
    /// Accept connection request
    ConnectAccept { from_id: String, to_id: String },
    /// Reject connection request
    ConnectReject { from_id: String, to_id: String, reason: String },
    /// WebRTC offer
    Offer { from_id: String, to_id: String, sdp: String },
    /// WebRTC answer
    Answer { from_id: String, to_id: String, sdp: String },
    /// ICE candidate
    IceCandidate { from_id: String, to_id: String, candidate: String },
    /// Peer online notification
    PeerOnline { connection_id: String, device_name: String },
    /// Peer offline notification
    PeerOffline { connection_id: String },
    /// Error message
    Error { message: String },
}

/// Connection Manager
/// 
/// Handles all remote connections using Connection IDs
pub struct ConnectionManager {
    /// My connection ID
    my_connection_id: ConnectionId,
    /// Current connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Connected peers
    peers: Arc<RwLock<HashMap<String, RemotePeer>>>,
    /// Event sender
    event_tx: mpsc::UnboundedSender<ConnectionEvent>,
    /// Signaling WebSocket sender
    signaling_tx: Arc<Mutex<Option<mpsc::UnboundedSender<SignalingMessage>>>>,
    /// Screen capture enabled
    screen_capture_enabled: Arc<RwLock<bool>>,
    /// Remote control enabled
    remote_control_enabled: Arc<RwLock<bool>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ConnectionEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let my_connection_id = get_connection_id().clone();
        
        info!("ConnectionManager initialized with ID: {}", my_connection_id.display_id);
        
        let manager = Self {
            my_connection_id,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            signaling_tx: Arc::new(Mutex::new(None)),
            screen_capture_enabled: Arc::new(RwLock::new(true)),
            remote_control_enabled: Arc::new(RwLock::new(true)),
        };
        
        (manager, event_rx)
    }
    
    /// Get my connection ID
    pub fn my_connection_id(&self) -> &str {
        &self.my_connection_id.display_id
    }
    
    /// Get current state
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }
    
    /// Connect to the signaling server
    pub async fn connect_to_signaling(&self) -> Result<()> {
        info!("Connecting to signaling server: {}", SIGNALING_SERVER_URL);
        
        self.set_state(ConnectionState::Connecting).await;
        
        // In a real implementation, this would establish a WebSocket connection
        // For now, we'll simulate the connection
        
        // Create signaling channel
        let (sig_tx, mut sig_rx) = mpsc::unbounded_channel::<SignalingMessage>();
        
        {
            let mut tx_guard = self.signaling_tx.lock().await;
            *tx_guard = Some(sig_tx.clone());
        }
        
        // Register with the signaling server
        let register_msg = SignalingMessage::Register {
            connection_id: self.my_connection_id.display_id.clone(),
            device_name: self.my_connection_id.device_name.clone(),
        };
        
        sig_tx.send(register_msg).ok();
        
        self.set_state(ConnectionState::WaitingForPeer).await;
        
        info!("Connected to signaling server, waiting for peers...");
        
        Ok(())
    }
    
    /// Connect to a remote peer using their Connection ID
    pub async fn connect_to_peer(&self, remote_connection_id: &str) -> Result<()> {
        // Validate the connection ID format
        let cleaned_id = remote_connection_id
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();
        
        if cleaned_id.len() != 9 {
            return Err(anyhow!("Invalid Connection ID format. Expected XXX-XXX-XXX"));
        }
        
        let formatted_id = format!(
            "{}-{}-{}",
            &cleaned_id[0..3],
            &cleaned_id[3..6],
            &cleaned_id[6..9]
        );
        
        info!("Connecting to peer: {}", formatted_id);
        
        self.set_state(ConnectionState::Connecting).await;
        
        // Send connect request through signaling
        if let Some(tx) = self.signaling_tx.lock().await.as_ref() {
            let connect_msg = SignalingMessage::ConnectRequest {
                from_id: self.my_connection_id.display_id.clone(),
                to_id: formatted_id.clone(),
            };
            tx.send(connect_msg).ok();
        }
        
        // Add peer to tracking
        let peer = RemotePeer {
            connection_id: formatted_id.clone(),
            device_name: "Unknown".to_string(),
            state: ConnectionState::Connecting,
            connected_at: None,
            latency_ms: None,
        };
        
        self.peers.write().await.insert(formatted_id.clone(), peer);
        
        // In a real implementation, this would:
        // 1. Send connect request to signaling server
        // 2. Wait for peer to accept
        // 3. Exchange WebRTC offer/answer
        // 4. Exchange ICE candidates
        // 5. Establish peer connection
        
        // Simulate successful connection for demo
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        self.set_state(ConnectionState::Negotiating).await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Update peer state
        if let Some(peer) = self.peers.write().await.get_mut(&formatted_id) {
            peer.state = ConnectionState::Connected;
            peer.connected_at = Some(chrono::Utc::now());
            peer.latency_ms = Some(45); // Simulated latency
            
            let _ = self.event_tx.send(ConnectionEvent::PeerConnected(peer.clone()));
        }
        
        self.set_state(ConnectionState::Connected).await;
        
        info!("Successfully connected to peer: {}", formatted_id);
        
        Ok(())
    }
    
    /// Disconnect from a peer
    pub async fn disconnect_from_peer(&self, connection_id: &str) -> Result<()> {
        info!("Disconnecting from peer: {}", connection_id);
        
        if let Some(peer) = self.peers.write().await.remove(connection_id) {
            let _ = self.event_tx.send(ConnectionEvent::PeerDisconnected(peer.connection_id));
        }
        
        if self.peers.read().await.is_empty() {
            self.set_state(ConnectionState::WaitingForPeer).await;
        }
        
        Ok(())
    }
    
    /// Disconnect from all peers
    pub async fn disconnect_all(&self) -> Result<()> {
        info!("Disconnecting from all peers");
        
        let peer_ids: Vec<String> = self.peers.read().await.keys().cloned().collect();
        
        for id in peer_ids {
            self.disconnect_from_peer(&id).await?;
        }
        
        self.set_state(ConnectionState::Disconnected).await;
        
        Ok(())
    }
    
    /// Start screen sharing
    pub async fn start_screen_sharing(&self) -> Result<()> {
        info!("Starting screen sharing...");
        
        *self.screen_capture_enabled.write().await = true;
        
        self.set_state(ConnectionState::Streaming).await;
        
        // In a real implementation, this would:
        // 1. Initialize screen capture (DXGI)
        // 2. Encode frames (H.264)
        // 3. Send frames over WebRTC data channel
        
        info!("Screen sharing started");
        
        Ok(())
    }
    
    /// Stop screen sharing
    pub async fn stop_screen_sharing(&self) -> Result<()> {
        info!("Stopping screen sharing...");
        
        *self.screen_capture_enabled.write().await = false;
        
        if self.peers.read().await.is_empty() {
            self.set_state(ConnectionState::Disconnected).await;
        } else {
            self.set_state(ConnectionState::Connected).await;
        }
        
        info!("Screen sharing stopped");
        
        Ok(())
    }
    
    /// Enable/disable remote control
    pub async fn set_remote_control_enabled(&self, enabled: bool) {
        *self.remote_control_enabled.write().await = enabled;
        info!("Remote control {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Send input event to remote peer
    pub async fn send_input(&self, peer_id: &str, input: InputEvent) -> Result<()> {
        if !*self.remote_control_enabled.read().await {
            return Err(anyhow!("Remote control is disabled"));
        }
        
        // In a real implementation, this would send the input over WebRTC
        debug!("Sending input to {}: {:?}", peer_id, input);
        
        Ok(())
    }
    
    /// Handle incoming input event
    pub async fn handle_input(&self, input: InputEvent) -> Result<()> {
        if !*self.remote_control_enabled.read().await {
            return Ok(()); // Silently ignore if disabled
        }
        
        // In a real implementation, this would inject the input using Windows API
        match &input {
            InputEvent::MouseMove { x, y } => {
                debug!("Mouse move to ({}, {})", x, y);
                // Use SendInput or SetCursorPos
            }
            InputEvent::MouseClick { button, pressed } => {
                debug!("Mouse {:?} {}", button, if *pressed { "down" } else { "up" });
                // Use SendInput with MOUSEINPUT
            }
            InputEvent::MouseScroll { delta_x, delta_y } => {
                debug!("Mouse scroll ({}, {})", delta_x, delta_y);
                // Use SendInput with MOUSEINPUT wheel
            }
            InputEvent::KeyPress { key_code, pressed } => {
                debug!("Key {} {}", key_code, if *pressed { "down" } else { "up" });
                // Use SendInput with KEYBDINPUT
            }
        }
        
        let _ = self.event_tx.send(ConnectionEvent::InputReceived(input));
        
        Ok(())
    }
    
    /// Get list of connected peers
    pub async fn connected_peers(&self) -> Vec<RemotePeer> {
        self.peers.read().await.values().cloned().collect()
    }
    
    /// Set connection state and emit event
    async fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.write().await;
        if *state != new_state {
            *state = new_state.clone();
            let _ = self.event_tx.send(ConnectionEvent::StateChanged(new_state));
        }
    }
    
    /// Get ICE servers configuration
    pub fn get_ice_servers() -> Vec<IceServer> {
        let mut servers = Vec::new();
        
        // Add STUN servers
        for stun in STUN_SERVERS {
            servers.push(IceServer {
                urls: vec![stun.to_string()],
                username: None,
                credential: None,
            });
        }
        
        // Add TURN server
        servers.push(IceServer {
            urls: vec![TURN_SERVER.to_string()],
            username: Some(TURN_USERNAME.to_string()),
            credential: Some(TURN_CREDENTIAL.to_string()),
        });
        
        servers
    }
}

/// ICE server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new().0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_manager_creation() {
        let (manager, _rx) = ConnectionManager::new();
        assert!(!manager.my_connection_id().is_empty());
    }
    
    #[tokio::test]
    async fn test_connect_to_peer() {
        let (manager, mut rx) = ConnectionManager::new();
        
        // Connect to a fake peer
        let result = manager.connect_to_peer("123-456-789").await;
        assert!(result.is_ok());
        
        // Check state changed events
        let mut connected = false;
        while let Ok(event) = rx.try_recv() {
            if let ConnectionEvent::StateChanged(ConnectionState::Connected) = event {
                connected = true;
                break;
            }
        }
        assert!(connected);
    }
}
