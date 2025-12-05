use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("🚀 GenXLink Signaling Server starting...");
    
    // Build router
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/health", get(health_check));
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("🚀 GenXLink Signaling Server listening on {}", addr);
    println!("📡 WebSocket endpoint: ws://{}:{}/ws", addr.ip(), addr.port());
    println!("🔍 Health check: http://{}:{}/health", addr.ip(), addr.port());
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket) {
    info!("🔌 New WebSocket connection");
    
    let (mut sender, mut receiver) = socket.split();
    
    // Simple echo server for now
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                if let Err(e) = sender.send(msg).await {
                    info!("Error sending message: {}", e);
                    break;
                }
            }
            Err(e) => {
                info!("Error receiving message: {}", e);
                break;
            }
        }
    }
    
    info!("WebSocket connection closed");
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "{\"status\":\"healthy\",\"service\":\"genxlink-signaling\"}"
}
