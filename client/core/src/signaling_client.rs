use crate::ClientError;
use genxlink_protocol::{SignalingMessage, DeviceId};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Signaling client for WebRTC connection setup
pub struct SignalingClient {
    device_id: DeviceId,
    server_url: String,
    state: Arc<RwLock<SignalingState>>,
    message_tx: Option<mpsc::UnboundedSender<SignalingMessage>>,
}

/// Signaling connection state
#[derive(Debug, Clone, PartialEq)]
pub enum SignalingState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

impl SignalingClient {
    /// Create a new signaling client
    pub fn new(device_id: DeviceId, server_url: String) -> Self {
        Self {
            device_id,
            server_url,
            state: Arc::new(RwLock::new(SignalingState::Disconnected)),
            message_tx: None,
        }
    }

    /// Connect to the signaling server
    pub async fn connect(&mut self) -> Result<mpsc::UnboundedReceiver<SignalingMessage>, ClientError> {
        self.set_state(SignalingState::Connecting).await;

        // Connect to WebSocket server
        let (ws_stream, _) = connect_async(&self.server_url)
            .await
            .map_err(|e| ClientError::TransportError(format!("WebSocket connection failed: {}", e)))?;

        tracing::info!("Connected to signaling server: {}", self.server_url);

        let (mut write, mut read) = ws_stream.split();

        // Create channels for bidirectional communication
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<SignalingMessage>();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<SignalingMessage>();

        self.message_tx = Some(outgoing_tx);
        self.set_state(SignalingState::Connected).await;

        let state = self.state.clone();

        // Spawn task to handle outgoing messages
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                let json = match serde_json::to_string(&msg) {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if let Err(e) = write.send(Message::Text(json)).await {
                    tracing::error!("Failed to send message: {}", e);
                    let mut s = state.write().await;
                    *s = SignalingState::Failed(format!("Send failed: {}", e));
                    break;
                }
            }
        });

        let state = self.state.clone();

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<SignalingMessage>(&text) {
                            Ok(msg) => {
                                if let Err(e) = incoming_tx.send(msg) {
                                    tracing::error!("Failed to forward message: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to deserialize message: {}", e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("WebSocket closed by server");
                        let mut s = state.write().await;
                        *s = SignalingState::Disconnected;
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        // Pongs are handled automatically by tungstenite
                        tracing::debug!("Received ping: {} bytes", data.len());
                    }
                    Ok(_) => {
                        // Ignore other message types
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        let mut s = state.write().await;
                        *s = SignalingState::Failed(format!("WebSocket error: {}", e));
                        break;
                    }
                }
            }
        });

        Ok(incoming_rx)
    }

    /// Send a signaling message
    pub async fn send(&self, message: SignalingMessage) -> Result<(), ClientError> {
        if let Some(tx) = &self.message_tx {
            tx.send(message)
                .map_err(|e| ClientError::TransportError(format!("Failed to send message: {}", e)))?;
            Ok(())
        } else {
            Err(ClientError::TransportError("Not connected".to_string()))
        }
    }

    /// Send a signaling message (alias for send)
    pub async fn send_message(&self, message: SignalingMessage) -> Result<(), ClientError> {
        self.send(message).await
    }

    /// Get current connection state
    pub async fn get_state(&self) -> SignalingState {
        self.state.read().await.clone()
    }

    /// Set connection state
    async fn set_state(&self, new_state: SignalingState) {
        let mut state = self.state.write().await;
        *state = new_state;
    }

    /// Request list of available peers
    pub async fn list_peers(&self) -> Result<(), ClientError> {
        self.send(SignalingMessage::ListPeers).await
    }

    /// Request connection to a peer
    pub async fn request_connection(&self, target: DeviceId) -> Result<(), ClientError> {
        self.send(SignalingMessage::ConnectionRequest {
            target,
            from: self.device_id.clone(),
        }).await
    }

    /// Send WebRTC offer
    pub async fn send_offer(&self, sdp: String, to: DeviceId) -> Result<(), ClientError> {
        self.send(SignalingMessage::Offer {
            sdp,
            from: self.device_id.clone(),
            to,
        }).await
    }

    /// Send WebRTC answer
    pub async fn send_answer(&self, sdp: String, to: DeviceId) -> Result<(), ClientError> {
        self.send(SignalingMessage::Answer {
            sdp,
            from: self.device_id.clone(),
            to,
        }).await
    }

    /// Send ICE candidate
    pub async fn send_ice_candidate(
        &self,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
        to: DeviceId,
    ) -> Result<(), ClientError> {
        self.send(SignalingMessage::IceCandidate {
            candidate,
            sdp_mid,
            sdp_m_line_index,
            from: self.device_id.clone(),
            to,
        }).await
    }

    /// Close the connection
    pub async fn close(&mut self) {
        self.message_tx = None;
        self.set_state(SignalingState::Disconnected).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signaling_client_creation() {
        let device_id = DeviceId::new();
        let client = SignalingClient::new(
            device_id,
            "ws://localhost:8081/ws".to_string(),
        );
        
        assert_eq!(client.server_url, "ws://localhost:8081/ws");
    }

    #[tokio::test]
    async fn test_initial_state() {
        let device_id = DeviceId::new();
        let client = SignalingClient::new(
            device_id,
            "ws://localhost:8081/ws".to_string(),
        );
        
        let state = client.get_state().await;
        assert_eq!(state, SignalingState::Disconnected);
    }
}
