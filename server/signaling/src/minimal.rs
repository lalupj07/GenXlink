// Minimal GenXLink Signaling Server for Railway
use std::net::SocketAddr;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("🚀 GenXLink Signaling Server starting...");
    
    // Build router with CORS
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/health", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        );
    
    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = port.parse().expect("PORT must be a number");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    println!("🚀 Server listening on {}", addr);
    println!("📡 WebSocket: ws://{}:{}/ws", addr.ip(), addr.port());
    println!("🔍 Health: http://{}:{}/health", addr.ip(), addr.port());
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    println!("🔌 New WebSocket connection");
    
    let (mut sender, mut receiver) = socket.split();
    
    // Simple echo server for now
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("📨 Received: {}", text);
                // Echo back
                if let Err(e) = sender.send(Message::Text(format!("Echo: {}", text))).await {
                    println!("❌ Send error: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("🔌 Client disconnected");
                break;
            }
            Err(e) => {
                println!("❌ Receive error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    println!("🔌 Connection closed");
}

async fn health_check() -> &'static str {
    "{\"status\":\"healthy\",\"service\":\"genxlink-signaling\",\"version\":\"1.0.0\"}"
}
