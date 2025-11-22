use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tracing::info;

mod handlers;
mod models;
mod db;
mod auth;

use handlers::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("GenXLink API Server starting...");
    
    // TODO: Initialize database connection pool
    // TODO: Initialize Redis connection
    
    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/license/activate", post(activate_license))
        .route("/license/status", get(license_status))
        .route("/connection/start", post(start_connection))
        .route("/connection/end", post(end_connection));
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}
