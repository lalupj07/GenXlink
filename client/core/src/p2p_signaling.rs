use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use futures::{SinkExt, StreamExt};

use crate::p2p_discovery::{P2PDiscovery, PeerInfo, DiscoveryEvent};
use crate::webrtc_session::WebRTCSession;
use genxlink_protocol::{DeviceId, SignalingMessage};

/// Direct P2P signaling without central server
/// Uses WebSocket connections between peers for SDP exchange
pub struct P2PSignaling {
    device_id: DeviceId,
    discovery: P2PDiscovery,
    websocket_server: Option<TcpListener>,
    active_peer_connections: Arc<RwLock<HashMap<DeviceId, PeerWebSocketConnection>>>,
    pending_offers: Arc<RwLock<HashMap<String, PendingOffer>>>,
    event_tx: mpsc::UnboundedSender<P2PSignalingEvent>,
}

/// WebSocket connection to a peer
#[derive(Debug)]
pub struct PeerWebSocketConnection {
    device_id: DeviceId,
    websocket_tx: mpsc::UnboundedSender<Message>,
    message_rx: mpsc::UnboundedReceiver<P2PSignalingMessage>,
    last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

/// P2P signaling events
#[derive(Debug, Clone)]
pub enum P2PSignalingEvent {
    PeerConnected(DeviceId),
    PeerDisconnected(DeviceId),
    OfferReceived { from_device: DeviceId, offer: String },
    AnswerReceived { from_device: DeviceId, answer: String },
    IceCandidateReceived { from_device: DeviceId, candidate: String },
    ConnectionFailed(DeviceId, String),
}

/// Pending offer information
#[derive(Debug, Clone)]
struct PendingOffer {
    offer_id: String,
    from_device: DeviceId,
    offer_sdp: String,
    created_at: std::time::SystemTime,
}

/// Direct signaling message for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
struct P2PSignalingMessage {
    message_type: P2PMessageType,
    from_device: DeviceId,
    to_device: DeviceId,
    offer_id: Option<String>,
    payload: String,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum P2PMessageType {
    Offer,
    Answer,
    IceCandidate,
    Ping,
    Pong,
}

impl P2PSignaling {
    /// Create a new P2P signaling service
    pub fn new(device_id: DeviceId, device_name: String) -> (Self, mpsc::UnboundedReceiver<P2PSignalingEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (discovery, _) = P2PDiscovery::new(device_id.clone(), device_name);
        
        let signaling = Self {
            device_id,
            discovery,
            websocket_server: None,
            active_peer_connections: Arc::new(RwLock::new(HashMap::new())),
            pending_offers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        };
        
        (signaling, event_rx)
    }

    /// Start the P2P signaling service
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting P2P signaling service for device {}", self.device_id);
        
        // Start WebSocket server for incoming peer connections
        self.start_websocket_server().await?;
        
        // Start peer discovery
        self.discovery.start().await?;
        
        // Start peer connection manager
        self.start_peer_connection_manager().await;
        
        Ok(())
    }

    /// Start WebSocket server for peer connections
    async fn start_websocket_server(&mut self) -> Result<()> {
        let addr: std::net::SocketAddr = "0.0.0.0:8080".parse().unwrap();
        let listener = TcpListener::bind(addr).await
            .context("Failed to bind WebSocket server")?;
        
        info!("P2P WebSocket server listening on {}", addr);
        
        let active_connections = self.active_peer_connections.clone();
        let event_tx = self.event_tx.clone();
        let device_id = self.device_id.clone();
        
        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                debug!("New WebSocket connection from {}", addr);
                
                let active_connections_clone = active_connections.clone();
                let event_tx_clone = event_tx.clone();
                let device_id_clone = device_id.clone();
                
                tokio::spawn(async move {
                    match Self::handle_peer_websocket(stream, device_id_clone, active_connections_clone, event_tx_clone).await {
                        Ok(peer_device_id) => {
                            info!("Successfully established WebSocket connection with peer: {}", peer_device_id);
                        }
                        Err(e) => {
                            error!("Failed to handle peer WebSocket connection: {}", e);
                        }
                    }
                });
            }
        });
        
        Ok(())
    }

    /// Handle incoming WebSocket connection from a peer
    async fn handle_peer_websocket(
        stream: TcpStream,
        my_device_id: DeviceId,
        active_connections: Arc<RwLock<HashMap<DeviceId, PeerWebSocketConnection>>>,
        event_tx: mpsc::UnboundedSender<P2PSignalingEvent>,
    ) -> Result<DeviceId> {
        let ws_stream = accept_async(stream).await
            .context("Failed to accept WebSocket connection")?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (message_tx, mut message_rx) = mpsc::unbounded_channel::<Message>();
        let (response_tx, mut response_rx) = mpsc::unbounded_channel::<P2PSignalingMessage>();
        
        // Handle outgoing messages
        let ws_sender_task = tokio::spawn(async move {
            while let Some(msg) = message_rx.recv().await {
                if let Err(e) = ws_sender.send(msg).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });
        
        // Handle incoming messages
        let active_connections_clone = active_connections.clone();
        let event_tx_clone = event_tx.clone();
        let response_tx_clone = response_tx.clone();
        
        let ws_receiver_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<P2PSignalingMessage>(&text) {
                            Ok(signaling_msg) => {
                                let _ = response_tx_clone.send(signaling_msg);
                            }
                            Err(e) => {
                                error!("Failed to parse signaling message: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        debug!("WebSocket connection closed");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        // Process signaling messages
        let mut peer_device_id: Option<DeviceId> = None;
        let mut connection_stored = false;
        
        while let Some(signaling_msg) = response_rx.recv().await {
            if peer_device_id.is_none() {
                peer_device_id = Some(signaling_msg.from_device.clone());
                
                // Create a new channel for the connection
                let (_conn_tx, _conn_rx) = mpsc::unbounded_channel::<Message>();
                
                // Store the connection (without moving response_rx)
                if !connection_stored {
                    let connection = PeerWebSocketConnection {
                        device_id: signaling_msg.from_device.clone(),
                        websocket_tx: message_tx.clone(),
                        message_rx: mpsc::unbounded_channel::<P2PSignalingMessage>().1, // Dummy receiver
                        last_seen: chrono::Utc::now(),
                    };
                    
                    let mut connections = active_connections_clone.write().await;
                    connections.insert(signaling_msg.from_device.clone(), connection);
                    drop(connections);
                    connection_stored = true;
                }
                
                let _ = event_tx_clone.send(P2PSignalingEvent::PeerConnected(signaling_msg.from_device.clone()));
            }
            
            // Process the signaling message
            match signaling_msg.message_type {
                P2PMessageType::Offer => {
                    let _ = event_tx_clone.send(P2PSignalingEvent::OfferReceived {
                        from_device: signaling_msg.from_device,
                        offer: signaling_msg.payload,
                    });
                }
                P2PMessageType::Answer => {
                    let _ = event_tx_clone.send(P2PSignalingEvent::AnswerReceived {
                        from_device: signaling_msg.from_device,
                        answer: signaling_msg.payload,
                    });
                }
                P2PMessageType::IceCandidate => {
                    let _ = event_tx_clone.send(P2PSignalingEvent::IceCandidateReceived {
                        from_device: signaling_msg.from_device,
                        candidate: signaling_msg.payload,
                    });
                }
                P2PMessageType::Ping => {
                    // Send pong response
                    let pong_msg = P2PSignalingMessage {
                        message_type: P2PMessageType::Pong,
                        from_device: my_device_id.clone(),
                        to_device: signaling_msg.from_device,
                        offer_id: None,
                        payload: "pong".to_string(),
                        timestamp: std::time::SystemTime::now(),
                    };
                    
                    if let Ok(text) = serde_json::to_string(&pong_msg) {
                        let _ = message_tx.send(Message::Text(text));
                    }
                }
                P2PMessageType::Pong => {
                    debug!("Received pong from peer: {}", signaling_msg.from_device);
                }
            }
        }
        
        // Cleanup
        ws_sender_task.abort();
        ws_receiver_task.abort();
        
        if let Some(device_id) = peer_device_id {
            let mut connections = active_connections.write().await;
            connections.remove(&device_id);
            drop(connections);
            
            let _ = event_tx.send(P2PSignalingEvent::PeerDisconnected(device_id.clone()));
            return Ok(device_id);
        }
        
        Err(anyhow!("Peer connection established but device ID not determined"))
    }

    /// Start peer connection manager
    async fn start_peer_connection_manager(&self) {
        let discovery = self.discovery.clone();
        let active_connections = self.active_peer_connections.clone();
        let my_device_id = self.device_id.clone();
        
        tokio::spawn(async move {
            // TODO: Implement automatic peer connection based on discovery
            // This would connect to discovered peers automatically
        });
    }

    /// Connect to a peer's WebSocket endpoint
    pub async fn connect_to_peer(&self, peer_info: &PeerInfo) -> Result<()> {
        let peer_url = format!("ws://{}:{}/ws", peer_info.ip_address, peer_info.port);
        info!("Connecting to peer WebSocket: {}", peer_url);
        
        let (ws_stream, _) = connect_async(&peer_url).await
            .context("Failed to connect to peer WebSocket")?;
        
        let device_id = self.device_id.clone();
        let active_connections = self.active_peer_connections.clone();
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            match Self::handle_peer_websocket(ws_stream, device_id, active_connections, event_tx).await {
                Ok(peer_device_id) => {
                    info!("Successfully connected to peer: {}", peer_device_id);
                }
                Err(e) => {
                    error!("Failed to establish peer connection: {}", e);
                }
            }
        });
        
        Ok(())
    }

    /// Send an offer to a peer
    pub async fn send_offer(&self, to_device: DeviceId, offer_sdp: String) -> Result<String> {
        let offer_id = Uuid::new_v4().to_string();
        
        let message = P2PSignalingMessage {
            message_type: P2PMessageType::Offer,
            from_device: self.device_id.clone(),
            to_device,
            offer_id: Some(offer_id.clone()),
            payload: offer_sdp,
            timestamp: std::time::SystemTime::now(),
        };
        
        self.send_signaling_message(message).await?;
        Ok(offer_id)
    }

    /// Send an answer to a peer
    pub async fn send_answer(&self, to_device: DeviceId, answer_sdp: String) -> Result<()> {
        let message = P2PSignalingMessage {
            message_type: P2PMessageType::Answer,
            from_device: self.device_id.clone(),
            to_device,
            offer_id: None,
            payload: answer_sdp,
            timestamp: std::time::SystemTime::now(),
        };
        
        self.send_signaling_message(message).await
    }

    /// Send ICE candidate to a peer
    pub async fn send_ice_candidate(&self, to_device: DeviceId, candidate: String) -> Result<()> {
        let message = P2PSignalingMessage {
            message_type: P2PMessageType::IceCandidate,
            from_device: self.device_id.clone(),
            to_device,
            offer_id: None,
            payload: candidate,
            timestamp: std::time::SystemTime::now(),
        };
        
        self.send_signaling_message(message).await
    }

    /// Send a signaling message to a peer
    async fn send_signaling_message(&self, message: P2PSignalingMessage) -> Result<()> {
        let connections = self.active_peer_connections.read().await;
        
        if let Some(connection) = connections.get(&message.to_device) {
            if let Ok(text) = serde_json::to_string(&message) {
                if let Err(e) = connection.websocket_tx.send(Message::Text(text)) {
                    error!("Failed to send signaling message: {}", e);
                    return Err(anyhow!("Failed to send message to peer: {}", e));
                }
            }
        } else {
            return Err(anyhow!("No active connection to peer: {}", message.to_device));
        }
        
        Ok(())
    }

    /// Get all active peer connections
    pub async fn get_active_connections(&self) -> Vec<DeviceId> {
        self.active_peer_connections.read().await.keys().cloned().collect()
    }

    /// Disconnect from a specific peer
    pub async fn disconnect_from_peer(&self, peer_device_id: &DeviceId) -> Result<()> {
        info!("Disconnecting from peer: {}", peer_device_id);
        
        let mut connections = self.active_peer_connections.write().await;
        if let Some(_connection) = connections.remove(peer_device_id) {
            let _ = self.event_tx.send(P2PSignalingEvent::PeerDisconnected(peer_device_id.clone()));
            info!("Successfully disconnected from peer: {}", peer_device_id);
            Ok(())
        } else {
            Err(anyhow!("No active connection to peer: {}", peer_device_id))
        }
    }

    /// Get discovered peers
    pub async fn get_discovered_peers(&self) -> Vec<PeerInfo> {
        self.discovery.get_peers().await
    }

    /// Stop the P2P signaling service
    pub async fn stop(&self) {
        info!("Stopping P2P signaling service");
        self.discovery.stop().await;
        
        // Close all active connections
        let mut connections = self.active_peer_connections.write().await;
        connections.clear();
    }
}
