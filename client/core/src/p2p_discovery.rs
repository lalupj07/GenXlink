use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, error, warn, debug};
use uuid::Uuid;

use genxlink_protocol::{DeviceId, SignalingMessage};

/// Peer discovery mechanism without central server
/// Uses multicast DNS and local network discovery
pub struct P2PDiscovery {
    device_id: DeviceId,
    device_name: String,
    peers: Arc<RwLock<HashMap<DeviceId, PeerInfo>>>,
    event_tx: mpsc::UnboundedSender<DiscoveryEvent>,
    multicast_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<DiscoveryPacket>>>,
    multicast_tx: mpsc::UnboundedSender<DiscoveryPacket>,
}

impl Clone for P2PDiscovery {
    fn clone(&self) -> Self {
        // Create new channels for the clone - each clone gets its own receiver
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            device_id: self.device_id.clone(),
            device_name: self.device_name.clone(),
            peers: self.peers.clone(),
            event_tx: self.event_tx.clone(),
            multicast_rx: Arc::new(tokio::sync::Mutex::new(rx)),
            multicast_tx: tx,
        }
    }
}

/// Information about a discovered peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub device_id: DeviceId,
    pub device_name: String,
    pub ip_address: String,
    pub port: u16,
    pub last_seen: std::time::SystemTime,
    pub capabilities: Vec<String>,
}

/// Discovery events
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    PeerDiscovered(PeerInfo),
    PeerLost(DeviceId),
    PeerUpdated(PeerInfo),
}

/// Discovery packet for multicast communication
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryPacket {
    packet_type: PacketType,
    device_id: DeviceId,
    device_name: String,
    ip_address: String,
    port: u16,
    capabilities: Vec<String>,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PacketType {
    Announcement,
    Response,
    Goodbye,
}

impl P2PDiscovery {
    /// Create a new P2P discovery service
    pub fn new(device_id: DeviceId, device_name: String) -> (Self, mpsc::UnboundedReceiver<DiscoveryEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (multicast_tx, multicast_rx) = mpsc::unbounded_channel();
        
        let discovery = Self {
            device_id,
            device_name,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            multicast_rx: Arc::new(tokio::sync::Mutex::new(multicast_rx)),
            multicast_tx,
        };
        
        (discovery, event_rx)
    }

    /// Start the discovery service
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting P2P discovery service for device {}", self.device_id);
        
        // Start announcement broadcaster
        self.start_announcement_broadcaster().await;
        
        // Start peer listener
        self.start_peer_listener().await;
        
        // Start cleanup task
        self.start_cleanup_task().await;
        
        Ok(())
    }

    /// Start broadcasting our presence on the local network
    async fn start_announcement_broadcaster(&mut self) {
        let device_id = self.device_id.clone();
        let device_name = self.device_name.clone();
        let multicast_tx = self.multicast_tx.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            // Get local IP address
            let local_ip = Self::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
            let port = 8080; // Default WebRTC signaling port
            
            loop {
                interval.tick().await;
                
                let packet = DiscoveryPacket {
                    packet_type: PacketType::Announcement,
                    device_id: device_id.clone(),
                    device_name: device_name.clone(),
                    ip_address: local_ip.clone(),
                    port,
                    capabilities: vec!["screen-share".to_string(), "audio".to_string(), "file-transfer".to_string()],
                    timestamp: std::time::SystemTime::now(),
                };
                
                if let Err(e) = multicast_tx.send(packet) {
                    error!("Failed to send announcement: {}", e);
                }
                
                debug!("Broadcasted peer announcement");
            }
        });
    }

    /// Start listening for peer announcements
    async fn start_peer_listener(&mut self) {
        let multicast_rx = self.multicast_rx.clone();
        let peers = self.peers.clone();
        let event_tx = self.event_tx.clone();
        let my_device_id = self.device_id.clone();
        
        tokio::spawn(async move {
            loop {
                let packet = {
                    let mut rx = multicast_rx.lock().await;
                    rx.recv().await
                };
                let Some(packet) = packet else { break };
                // Ignore our own packets
                if packet.device_id == my_device_id {
                    continue;
                }
                
                match packet.packet_type {
                    PacketType::Announcement => {
                        let peer_info = PeerInfo {
                            device_id: packet.device_id.clone(),
                            device_name: packet.device_name,
                            ip_address: packet.ip_address,
                            port: packet.port,
                            last_seen: packet.timestamp,
                            capabilities: packet.capabilities,
                        };
                        
                        let mut peers_guard = peers.write().await;
                        let is_new = !peers_guard.contains_key(&packet.device_id);
                        peers_guard.insert(packet.device_id.clone(), peer_info.clone());
                        drop(peers_guard);
                        
                        if is_new {
                            info!("Discovered new peer: {} ({})", peer_info.device_name, peer_info.device_id);
                            let _ = event_tx.send(DiscoveryEvent::PeerDiscovered(peer_info));
                        } else {
                            let _ = event_tx.send(DiscoveryEvent::PeerUpdated(peer_info));
                        }
                    }
                    PacketType::Response => {
                        // Handle peer responses to our announcements
                        debug!("Received response from peer: {}", packet.device_id);
                    }
                    PacketType::Goodbye => {
                        let mut peers_guard = peers.write().await;
                        if peers_guard.remove(&packet.device_id).is_some() {
                            info!("Peer disconnected: {}", packet.device_id);
                            let _ = event_tx.send(DiscoveryEvent::PeerLost(packet.device_id));
                        }
                    }
                }
            }
        });
    }

    /// Start cleanup task to remove inactive peers
    async fn start_cleanup_task(&self) {
        let peers = self.peers.clone();
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let now = std::time::SystemTime::now();
                let mut peers_guard = peers.write().await;
                let mut to_remove = Vec::new();
                
                for (device_id, peer_info) in peers_guard.iter() {
                    if let Ok(duration) = now.duration_since(peer_info.last_seen) {
                        if duration > Duration::from_secs(120) { // 2 minutes timeout
                            to_remove.push(device_id.clone());
                        }
                    }
                }
                
                for device_id in to_remove {
                    if peers_guard.remove(&device_id).is_some() {
                        info!("Removed inactive peer: {}", device_id);
                        let _ = event_tx.send(DiscoveryEvent::PeerLost(device_id));
                    }
                }
            }
        });
    }

    /// Get all discovered peers
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Get a specific peer by ID
    pub async fn get_peer(&self, device_id: &DeviceId) -> Option<PeerInfo> {
        self.peers.read().await.get(device_id).cloned()
    }

    /// Get local IP address
    fn get_local_ip() -> Option<String> {
        // Try to get local IP by connecting to a remote address
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if let Ok(_) = socket.connect("8.8.8.8:80") {
                if let Ok(local_addr) = socket.local_addr() {
                    return Some(local_addr.ip().to_string());
                }
            }
        }
        
        // Fallback to localhost
        Some("127.0.0.1".to_string())
    }

    /// Stop the discovery service
    pub async fn stop(&self) {
        // Send goodbye packet
        let packet = DiscoveryPacket {
            packet_type: PacketType::Goodbye,
            device_id: self.device_id.clone(),
            device_name: self.device_name.clone(),
            ip_address: Self::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string()),
            port: 8080,
            capabilities: vec![],
            timestamp: std::time::SystemTime::now(),
        };
        
        let _ = self.multicast_tx.send(packet);
        info!("P2P discovery service stopped");
    }
}

/// Direct connection manager for P2P connections
pub struct P2PConnectionManager {
    device_id: DeviceId,
    discovery: P2PDiscovery,
    active_connections: Arc<RwLock<HashMap<DeviceId, P2PConnection>>>,
}

/// Information about a P2P connection
#[derive(Debug, Clone)]
pub struct P2PConnection {
    pub peer_device_id: DeviceId,
    pub peer_address: String,
    pub connection_state: ConnectionState,
    pub established_at: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

impl P2PConnectionManager {
    /// Create a new P2P connection manager
    pub fn new(device_id: DeviceId, device_name: String) -> Self {
        let (discovery, _) = P2PDiscovery::new(device_id.clone(), device_name);
        
        Self {
            device_id,
            discovery,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to a discovered peer
    pub async fn connect_to_peer(&self, peer_device_id: DeviceId) -> Result<()> {
        let peer_info = self.discovery.get_peer(&peer_device_id).await
            .ok_or_else(|| anyhow!("Peer not found: {}", peer_device_id))?;
        
        info!("Connecting to peer: {} at {}", peer_info.device_name, peer_info.ip_address);
        
        // Create P2P connection record
        let connection = P2PConnection {
            peer_device_id: peer_device_id.clone(),
            peer_address: format!("{}:{}", peer_info.ip_address, peer_info.port),
            connection_state: ConnectionState::Connecting,
            established_at: std::time::SystemTime::now(),
        };
        
        let mut connections = self.active_connections.write().await;
        connections.insert(peer_device_id, connection);
        drop(connections);
        
        // TODO: Implement actual WebRTC direct connection
        // This would involve:
        // 1. Create WebSocket connection to peer's signaling endpoint
        // 2. Exchange SDP offers/answers
        // 3. Establish WebRTC peer connection
        
        Ok(())
    }

    /// Get all active connections
    pub async fn get_active_connections(&self) -> Vec<P2PConnection> {
        self.active_connections.read().await.values().cloned().collect()
    }

    /// Disconnect from a peer
    pub async fn disconnect_from_peer(&self, peer_device_id: &DeviceId) -> Result<()> {
        let mut connections = self.active_connections.write().await;
        if connections.remove(peer_device_id).is_some() {
            info!("Disconnected from peer: {}", peer_device_id);
        }
        Ok(())
    }
}
