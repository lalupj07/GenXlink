use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use axum::extract::ws::{WebSocket, Message};
use futures::{sink::SinkExt, stream::StreamExt};
use genxlink_protocol::{DeviceId, SignalingMessage};
use tracing::{info, error, debug, warn};

/// Manages connected peers
pub struct PeerManager {
    peers: Arc<RwLock<HashMap<DeviceId, PeerInfo>>>,
}

/// Information about a connected peer
pub struct PeerInfo {
    pub device_id: DeviceId,
    pub sender: mpsc::UnboundedSender<Message>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a new peer with WebSocket connection
    pub async fn register_peer(&self, socket: WebSocket) -> DeviceId {
        let device_id = DeviceId::new();
        let (mut sender, mut receiver) = socket.split();
        
        // Create channel for sending messages to this peer
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        
        // Store peer info
        let peer_info = PeerInfo {
            device_id: device_id.clone(),
            sender: tx,
            connected_at: chrono::Utc::now(),
        };
        
        let mut peers = self.peers.write().await;
        peers.insert(device_id.clone(), peer_info);
        drop(peers);
        
        // Spawn task to handle outgoing messages
        let device_id_clone = device_id.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = sender.send(msg).await {
                    error!("Failed to send message to peer {}: {}", device_id_clone, e);
                    break;
                }
            }
        });
        
        // Spawn task to handle incoming messages
        let peers_arc = Arc::clone(&self.peers);
        let device_id_clone = device_id.clone();
        tokio::spawn(async move {
            while let Some(msg_result) = receiver.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        debug!("Received message from {}: {}", device_id_clone, text);
                        
                        match serde_json::from_str::<SignalingMessage>(&text) {
                            Ok(signaling_msg) => {
                                handle_signaling_message(signaling_msg, device_id_clone.clone(), Arc::clone(&peers_arc)).await;
                            }
                            Err(e) => {
                                error!("Failed to parse signaling message from {}: {}", device_id_clone, e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Peer {} disconnected", device_id_clone);
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error for peer {}: {}", device_id_clone, e);
                        break;
                    }
                    _ => {}
                }
            }
            
            // Remove peer on disconnect
            let mut peers = peers_arc.write().await;
            peers.remove(&device_id_clone);
            info!("Peer {} removed from registry", device_id_clone);
        });
        
        device_id
    }
    /// Send message to a specific peer
    pub async fn send_to_peer(&self, target: &DeviceId, message: SignalingMessage) -> Result<(), String> {
        let peers = self.peers.read().await;
        
        if let Some(peer_info) = peers.get(target) {
            let json = serde_json::to_string(&message)
                .map_err(|e| format!("Failed to serialize message: {}", e))?;
            
            peer_info.sender.send(Message::Text(json))
                .map_err(|e| format!("Failed to send message: {}", e))?;
            
            Ok(())
        } else {
            Err(format!("Peer {} not found", target))
        }
    }
    
    /// Get list of all connected peers
    pub async fn get_connected_peers(&self) -> Vec<DeviceId> {
        let peers = self.peers.read().await;
        peers.keys().cloned().collect()
    }
    
    /// Unregister a peer
    pub async fn unregister_peer(&self, device_id: &DeviceId) {
        let mut peers = self.peers.write().await;
        peers.remove(device_id);
    }
    
    /// Check if peer is online
    pub async fn is_peer_online(&self, device_id: &DeviceId) -> bool {
        let peers = self.peers.read().await;
        peers.contains_key(device_id)
    }
}

/// Handle incoming signaling messages and route them appropriately
async fn handle_signaling_message(
    message: SignalingMessage,
    from_device: DeviceId,
    peers: Arc<RwLock<HashMap<DeviceId, PeerInfo>>>,
) {
    match message {
        SignalingMessage::Offer { to, sdp, .. } => {
            info!("📤 Forwarding offer from {} to {}", from_device, to);
            forward_message_to_peer(&to.clone(), SignalingMessage::Offer {
                from: from_device,
                to: to.clone(),
                sdp,
            }, &peers).await;
        }
        SignalingMessage::Answer { to, sdp, .. } => {
            info!("📤 Forwarding answer from {} to {}", from_device, to);
            forward_message_to_peer(&to.clone(), SignalingMessage::Answer {
                from: from_device,
                to: to.clone(),
                sdp,
            }, &peers).await;
        }
        SignalingMessage::IceCandidate { to, candidate, sdp_mid, sdp_m_line_index, .. } => {
            debug!("📤 Forwarding ICE candidate from {} to {}", from_device, to);
            forward_message_to_peer(&to.clone(), SignalingMessage::IceCandidate {
                from: from_device,
                to: to.clone(),
                candidate,
                sdp_mid,
                sdp_m_line_index,
            }, &peers).await;
        }
        SignalingMessage::ConnectionRequest { target, .. } => {
            info!("🔗 Connection request from {} to {}", from_device, target);
            forward_message_to_peer(&target.clone(), SignalingMessage::ConnectionRequest {
                target: target.clone(),
                from: from_device,
            }, &peers).await;
        }
        SignalingMessage::ListPeers => {
            // Send list of connected peers back to requester
            let peer_list = peers.read().await.keys().cloned().collect::<Vec<_>>();
            // Implementation would send this back to the requesting peer
            info!("📋 Peer list requested by {}: {} peers online", from_device, peer_list.len());
        }
        _ => {
            debug!("Received unhandled message type from {}", from_device);
        }
    }
}

/// Forward a message to a specific peer
async fn forward_message_to_peer(
    target: &DeviceId,
    message: SignalingMessage,
    peers: &Arc<RwLock<HashMap<DeviceId, PeerInfo>>>,
) {
    let peers_guard = peers.read().await;
    
    if let Some(peer_info) = peers_guard.get(target) {
        let json = match serde_json::to_string(&message) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to serialize message: {}", e);
                return;
            }
        };
        
        if let Err(e) = peer_info.sender.send(Message::Text(json)) {
            error!("Failed to forward message to {}: {}", target, e);
        } else {
            debug!("✅ Message forwarded to {}", target);
        }
    } else {
        warn!("⚠️ Target peer {} not found", target);
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}
