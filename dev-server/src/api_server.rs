use axum::{
    routing::get,
    Router,
    response::Json,
    extract::State,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, error};
use tower_http::cors::CorsLayer;
use tower::ServiceBuilder;

#[derive(Clone)]
struct AppState {
    // Simple in-memory state for development
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting GenXLink Development API Server");
    
    let state = AppState {};
    
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/health", get(api_health))
        .route("/api/auth/login", get(mock_login))
        .route("/api/devices", get(mock_devices))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(state);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 GenXLink Development API Server running on http://127.0.0.1:8000");
    info!("📊 Health check: http://127.0.0.1:8000/health");
    info!("🔌 API endpoint: http://127.0.0.1:8000/api/health");
    info!("🔐 Mock auth: http://127.0.0.1:8000/api/auth/login");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "genxlink-development-api-server",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0-dev"
    }))
}

async fn api_health(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "genxlink-api",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "features": {
            "authentication": "development_mode",
            "database": "mock",
            "signaling": "ready",
            "relay": "ready"
        }
    }))
}

async fn mock_login() -> Json<serde_json::Value> {
    Json(json!({
        "status": "success",
        "token": "dev_token_12345",
        "user": {
            "id": "dev-user-123",
            "email": "dev@genxlink.com",
            "name": "Development User"
        }
    }))
}

async fn mock_devices() -> Json<serde_json::Value> {
    Json(json!({
        "devices": [
            {
                "id": "device-1",
                "name": "Development Desktop",
                "type": "desktop",
                "status": "online",
                "last_seen": chrono::Utc::now().to_rfc3339()
            },
            {
                "id": "device-2", 
                "name": "Development Laptop",
                "type": "laptop",
                "status": "online",
                "last_seen": chrono::Utc::now().to_rfc3339()
            }
        ]
    }))
}
