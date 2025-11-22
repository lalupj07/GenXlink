use anyhow::Result;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tracing::info;

mod peer_manager;

use peer_manager::PeerManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("GenXLink Signaling Server starting...");
    
    // Create peer manager
    let peer_manager = PeerManager::new();
    
    // Build router
    let app = Router::new()
        .route("/ws", get(websocket_handler));
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("Signaling server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

/// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket) {
    info!("New WebSocket connection");
    
    // TODO: Implement WebRTC signaling
    // 1. Register peer with device ID
    // 2. Forward SDP offers/answers between peers
    // 3. Forward ICE candidates
    // 4. Handle disconnections
}
