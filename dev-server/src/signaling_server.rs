use axum::{
    routing::get,
    Router,
    response::Json,
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    extract::State,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, warn};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    connected_clients: Arc<RwLock<std::collections::HashMap<String, ()>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    info!("Starting GenXlink Development Signaling Server");
    
    let state = AppState {
        connected_clients: Arc::new(RwLock::new(std::collections::HashMap::new())),
    };
    
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/ws", get(websocket_handler))
        .with_state(state);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 GenXlink Development Signaling Server running on http://127.0.0.1:8081");
    info!("📊 Health check: http://127.0.0.1:8081/health");
    info!("🔌 WebSocket endpoint: ws://127.0.0.1:8081/ws");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "genxlink-signaling-dev",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "websocket": "available",
        "connected_clients": 0
    }))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = uuid::Uuid::new_v4().to_string();
    
    info!("New WebSocket client connected: {}", client_id);
    
    // Add client to connected list
    state.connected_clients.write().await.insert(client_id.clone(), ());
    
    // Send welcome message
    let welcome = json!({
        "type": "welcome",
        "client_id": client_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "message": "Connected to GenXlink Development Signaling Server"
    });
    
    if let Ok(msg) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(msg)).await;
    }
    
    // Handle messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received message from {}: {}", client_id, text);
                
                // Parse and respond to signaling messages
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                    
                    let response = match msg_type {
                        "offer" => json!({
                            "type": "offer-response",
                            "original": parsed,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "status": "received"
                        }),
                        "answer" => json!({
                            "type": "answer-response", 
                            "original": parsed,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "status": "received"
                        }),
                        "ice-candidate" => json!({
                            "type": "ice-candidate-response",
                            "original": parsed,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "status": "received"
                        }),
                        _ => json!({
                            "type": "echo",
                            "original": text,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        })
                    };
                    
                    if let Ok(resp) = serde_json::to_string(&response) {
                        let _ = sender.send(Message::Text(resp)).await;
                    }
                } else {
                    // Echo back for invalid JSON
                    let response = json!({
                        "type": "echo",
                        "original": text,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    
                    if let Ok(resp) = serde_json::to_string(&response) {
                        let _ = sender.send(Message::Text(resp)).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} disconnected", client_id);
                break;
            }
            Err(e) => {
                warn!("WebSocket error for client {}: {}", client_id, e);
                break;
            }
            _ => {}
        }
    }
    
    // Remove client from connected list
    state.connected_clients.write().await.remove(&client_id);
    info!("Client {} removed from active connections", client_id);
}
