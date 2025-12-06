// Static binary GenXLink Signaling Server
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::IntoResponse,
    routing::get,
    Router,
    Json,
};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    timestamp: String,
}

type PeerRegistry = Arc<RwLock<HashMap<String, ()>>>;

#[tokio::main]
async fn main() {
    println!("🚀 GenXLink Signaling Server v1.0.0");
    
    // Create peer registry
    let peers: PeerRegistry = Arc::new(RwLock::new(HashMap::new()));
    
    // Build router
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/health", get(health_check))
        .route("/peers", get(list_peers))
        .with_state(peers);
    
    // Get port from environment
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = port.parse().expect("Invalid PORT");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    println!("📡 Server listening on: {}", addr);
    println!("🔗 WebSocket: ws://{}:{}/ws", addr.ip(), addr.port());
    println!("❤️  Health: http://{}:{}/health", addr.ip(), addr.port());
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(peers): State<PeerRegistry>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, peers))
}

async fn handle_socket(socket: WebSocket, peers: PeerRegistry) {
    let peer_id = format!("peer_{}", uuid::Uuid::new_v4());
    
    println!("🔌 New connection: {}", peer_id);
    
    // Register peer
    peers.write().await.insert(peer_id.clone(), ());
    
    let (mut sender, mut receiver) = socket.split();
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("📨 Message from {}: {}", peer_id, text);
                
                // Simple signaling logic
                if let Ok(signaling) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(target) = signaling.get("target").and_then(|v| v.as_str()) {
                        println!("🎯 Routing message to: {}", target);
                    }
                }
                
                // Echo back for testing
                let response = serde_json::json!({
                    "type": "echo",
                    "from": peer_id,
                    "message": text,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                
                if let Err(e) = sender.send(Message::Text(response.to_string())).await {
                    println!("❌ Send error: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("🔌 {} disconnected", peer_id);
                break;
            }
            Err(e) => {
                println!("❌ Receive error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    // Unregister peer
    peers.write().await.remove(&peer_id);
    println!("👋 {} removed from registry", peer_id);
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "genxlink-signaling".to_string(),
        version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn list_peers(State(peers): State<PeerRegistry>) -> Json<Vec<String>> {
    let peer_list = peers.read().await.keys().cloned().collect();
    Json(peer_list)
}
