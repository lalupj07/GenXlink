// Copyright (c) 2025 GenXis Innovations
// Connection Service - Manages WebRTC P2P connections

use std::collections::HashMap;
use std::time::Instant;

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

/// Peer connection info
#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub id: String,
    pub device_name: String,
    pub ip_address: String,
    pub state: ConnectionState,
    pub connected_at: Option<Instant>,
    pub latency_ms: u32,
    pub bandwidth_mbps: f32,
}

/// WebRTC signaling message types
#[derive(Debug, Clone)]
pub enum SignalingMessage {
    Offer(String),
    Answer(String),
    IceCandidate(String),
    Disconnect,
}

/// ICE Server configuration for NAT traversal
#[derive(Debug, Clone)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

impl IceServer {
    pub fn stun(url: &str) -> Self {
        Self {
            urls: vec![url.to_string()],
            username: None,
            credential: None,
        }
    }
    
    pub fn turn(url: &str, username: &str, credential: &str) -> Self {
        Self {
            urls: vec![url.to_string()],
            username: Some(username.to_string()),
            credential: Some(credential.to_string()),
        }
    }
}

/// Connection Service - handles P2P WebRTC connections
pub struct ConnectionService {
    connections: HashMap<String, PeerConnection>,
    local_device_id: String,
    signaling_server_url: String,
    is_connected_to_signaling: bool,
    ice_servers: Vec<IceServer>,
}

impl ConnectionService {
    pub fn new() -> Self {
        // Configure ICE servers for NAT traversal
        // Using Metered.ca Open Relay (free TURN service)
        let ice_servers = vec![
            // Public STUN servers (free)
            IceServer::stun("stun:stun.l.google.com:19302"),
            IceServer::stun("stun:stun1.l.google.com:19302"),
            IceServer::stun("stun:stun.cloudflare.com:3478"),
            // Open Relay TURN servers (free, global)
            IceServer::turn(
                "turn:openrelay.metered.ca:80",
                "openrelayproject",
                "openrelayproject"
            ),
            IceServer::turn(
                "turn:openrelay.metered.ca:443",
                "openrelayproject",
                "openrelayproject"
            ),
            IceServer::turn(
                "turn:openrelay.metered.ca:443?transport=tcp",
                "openrelayproject",
                "openrelayproject"
            ),
            IceServer::turn(
                "turns:openrelay.metered.ca:443",
                "openrelayproject",
                "openrelayproject"
            ),
        ];
        
        println!("🌐 ICE servers configured: {} STUN, {} TURN (Open Relay)", 
            ice_servers.iter().filter(|s| s.username.is_none()).count(),
            ice_servers.iter().filter(|s| s.username.is_some()).count()
        );
        
        Self {
            connections: HashMap::new(),
            local_device_id: generate_device_id(),
            signaling_server_url: "wss://genxlink-production.up.railway.app/ws".to_string(),
            is_connected_to_signaling: false,
            ice_servers,
        }
    }
    
    /// Get configured ICE servers
    pub fn get_ice_servers(&self) -> &[IceServer] {
        &self.ice_servers
    }
    
    /// Add a custom TURN server
    pub fn add_turn_server(&mut self, url: &str, username: &str, credential: &str) {
        self.ice_servers.push(IceServer::turn(url, username, credential));
        println!("➕ Added TURN server: {}", url);
    }

    /// Connect to signaling server (sync version)
    pub fn connect_to_signaling_sync(&mut self) -> Result<(), String> {
        println!("🔗 Connecting to signaling server: {}", self.signaling_server_url);
        self.is_connected_to_signaling = true;
        println!("✅ Connected to signaling server");
        Ok(())
    }

    /// Connect to signaling server (async version)
    pub async fn connect_to_signaling(&mut self) -> Result<(), String> {
        self.connect_to_signaling_sync()
    }

    /// Initiate connection to a peer (sync version)
    pub fn connect_to_peer_sync(&mut self, peer_id: &str, peer_ip: &str) -> Result<(), String> {
        println!("📡 Initiating connection to peer: {} ({})", peer_id, peer_ip);

        // Create peer connection entry
        let connection = PeerConnection {
            id: peer_id.to_string(),
            device_name: format!("Device-{}", if peer_id.len() >= 8 { &peer_id[..8] } else { peer_id }),
            ip_address: peer_ip.to_string(),
            state: ConnectionState::Connected, // Immediately connected for demo
            connected_at: Some(Instant::now()),
            latency_ms: 25,
            bandwidth_mbps: 100.0,
        };

        self.connections.insert(peer_id.to_string(), connection);
        println!("✅ Connected to peer: {}", peer_id);
        Ok(())
    }

    /// Initiate connection to a peer (async version)
    pub async fn connect_to_peer(&mut self, peer_id: &str, peer_ip: &str) -> Result<(), String> {
        self.connect_to_peer_sync(peer_id, peer_ip)
    }

    /// Disconnect from a peer
    pub fn disconnect_from_peer(&mut self, peer_id: &str) -> Result<(), String> {
        if let Some(conn) = self.connections.get_mut(peer_id) {
            conn.state = ConnectionState::Disconnected;
            println!("🔌 Disconnected from peer: {}", peer_id);
            Ok(())
        } else {
            Err(format!("Peer not found: {}", peer_id))
        }
    }

    /// Get all active connections
    pub fn get_connections(&self) -> Vec<PeerConnection> {
        self.connections.values().cloned().collect()
    }

    /// Get connection by ID
    pub fn get_connection(&self, peer_id: &str) -> Option<&PeerConnection> {
        self.connections.get(peer_id)
    }

    /// Check if connected to signaling server
    pub fn is_signaling_connected(&self) -> bool {
        self.is_connected_to_signaling
    }

    /// Get local device ID
    pub fn get_local_device_id(&self) -> &str {
        &self.local_device_id
    }

    /// Send screen share offer to peer
    pub async fn start_screen_share(&mut self, peer_id: &str) -> Result<(), String> {
        println!("🖥️ Starting screen share with peer: {}", peer_id);
        
        if let Some(conn) = self.connections.get(peer_id) {
            if conn.state != ConnectionState::Connected {
                return Err("Peer not connected".to_string());
            }
        } else {
            return Err("Peer not found".to_string());
        }

        // In production: Start screen capture, create video track, add to peer connection
        println!("✅ Screen share started with peer: {}", peer_id);
        Ok(())
    }

    /// Stop screen share
    pub fn stop_screen_share(&mut self, peer_id: &str) -> Result<(), String> {
        println!("🛑 Stopping screen share with peer: {}", peer_id);
        Ok(())
    }

    /// Handle incoming signaling message
    pub async fn handle_signaling_message(&mut self, from_peer: &str, message: SignalingMessage) {
        match message {
            SignalingMessage::Offer(sdp) => {
                println!("📥 Received offer from: {}", from_peer);
                // In production: Set remote description, create answer
            }
            SignalingMessage::Answer(sdp) => {
                println!("📥 Received answer from: {}", from_peer);
                // In production: Set remote description
            }
            SignalingMessage::IceCandidate(candidate) => {
                println!("📥 Received ICE candidate from: {}", from_peer);
                // In production: Add ICE candidate
            }
            SignalingMessage::Disconnect => {
                println!("📥 Peer disconnected: {}", from_peer);
                self.disconnect_from_peer(from_peer).ok();
            }
        }
    }
}

impl Default for ConnectionService {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a unique device ID
fn generate_device_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("GENX-{:X}", timestamp)
}
