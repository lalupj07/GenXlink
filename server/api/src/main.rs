use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use time::Duration as TimeDuration;

mod handlers;
mod models;
mod db;
mod auth;

use handlers::*;
use db::Database;
use auth::{AuthService, RateLimiter, auth_middleware, rate_limit_middleware};

// Application state
#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    auth_service: Arc<AuthService>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("GenXLink API Server starting...");
    
    // Initialize database connection
    let db = Database::new().await?;
    let db = Arc::new(db);
    
    // Initialize authentication service
    let auth_service = Arc::new(AuthService::new((*db).clone()));
    
    // Initialize application state
    let app_state = AppState {
        db: db.clone(),
        auth_service: auth_service.clone(),
    };
    
    // Initialize rate limiter (100 requests per minute per IP)
    let rate_limiter = Arc::new(RateLimiter::new(100, TimeDuration::minutes(1)));
    
    // Build router with middleware
    let app = Router::new()
        // Public endpoints (no auth required)
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .layer(middleware::from_fn_with_state(rate_limiter.clone(), rate_limit_middleware))
        .nest("/api", Router::new()
            // Protected endpoints (auth required)
            .route("/auth/refresh", post(refresh_token))
            .route("/auth/change-password", post(change_password))
            .route("/profile", get(get_profile))
            .route("/profile", post(update_profile))
            .route("/devices", get(get_devices))
            .route("/devices", post(register_device))
            .route("/devices/:device_id/status", post(update_device_status))
            .route("/sessions", get(get_sessions))
            .route("/sessions", post(create_session))
            .route("/sessions/:session_id/end", post(end_session))
            .route("/license/activate", post(activate_license))
            .route("/license/status", get(license_status))
            // Temporarily comment out problematic handlers
            // .route("/connection/start", post(start_connection))
            // .route("/connection/end", post(end_connection))
            // .route("/stats/system", get(get_system_stats))
            // .route("/stats/usage", get(get_usage_stats))
            .layer(middleware::from_fn_with_state(app_state.auth_service.clone(), auth_middleware))
        )
        .with_state(app_state);
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("API server listening on {}", addr);
    info!("Available endpoints:");
    info!("  Health check: GET /health");
    info!("  Authentication:");
    info!("    Register: POST /auth/register");
    info!("    Login: POST /auth/login");
    info!("    Refresh token: POST /api/auth/refresh");
    info!("    Change password: POST /api/auth/change-password");
    info!("  Profile:");
    info!("    Get profile: GET /api/profile");
    info!("    Update profile: POST /api/profile");
    info!("  Devices:");
    info!("    Get devices: GET /api/devices");
    info!("    Register device: POST /api/devices");
    info!("    Update device status: POST /api/devices/:device_id/status");
    info!("  Sessions:");
    info!("    Get sessions: GET /api/sessions");
    info!("    Create session: POST /api/sessions");
    info!("    End session: POST /api/sessions/:session_id/end");
    info!("  Licenses:");
    info!("    Activate license: POST /api/license/activate");
    info!("    Get license status: GET /api/license/status");
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}
