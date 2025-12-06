use axum::{
    Json,
    http::StatusCode,
    response::IntoResponse,
    extract::{Request, State, Path},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error, warn};
use uuid::Uuid;
use std::time::SystemTime;
use chrono::Utc;

use crate::models::*;
use crate::db::Database;
use crate::auth::{AuthService, extract_user, LoginRequest, RegisterRequest, AuthResponse, PasswordChangeRequest, AuthenticatedUser};

// Application state
use crate::AppState;

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "genxlink-api",
        "timestamp": Utc::now(),
        "version": "0.1.0"
    }))
}

/// Register a new user
pub async fn register(
    State(app_state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    match app_state.auth_service.register(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Registration error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Login user
pub async fn login(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    match app_state.auth_service.login(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Login error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Refresh JWT token
pub async fn refresh_token(
    State(app_state): State<AppState>,
    request: Request,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = extract_user(&request)?;
    match app_state.auth_service.refresh_token(user.id).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Token refresh error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user profile
pub async fn get_profile(
    State(_app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "username": user.username,
        "display_name": user.display_name,
        "avatar_url": user.avatar_url,
        "subscription_type": user.subscription_type,
        "created_at": user.created_at,
        "last_login": user.last_login,
        "is_active": user.is_active,
        "is_verified": user.is_verified
    })))
}

/// Refresh JWT token
pub async fn refresh_token(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<AuthResponse>, StatusCode> {
    match app_state.auth_service.refresh_token(user.id).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Token refresh error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update user profile
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

pub async fn update_profile(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(update): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    
    // TODO: Implement profile update in database
    // For now, return success
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Profile updated successfully"
    })))
}

/// Register a new device
pub async fn register_device(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(device): Json<Device>,
) -> Result<Json<Device>, StatusCode> {
    
    // Ensure device belongs to authenticated user
    if device.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    match app_state.db.register_device(&device).await {
        Ok(device) => Ok(Json(device)),
        Err(e) => {
            error!("Device registration error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user devices
pub async fn get_devices(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Vec<Device>>, StatusCode> {
    match app_state.db.get_user_devices(user.id).await {
        Ok(devices) => Ok(Json(devices)),
        Err(e) => {
            error!("Get devices error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update device online status
#[derive(Debug, Deserialize)]
pub struct UpdateDeviceStatusRequest {
    pub is_online: bool,
}

pub async fn update_device_status(
    State(app_state): State<AppState>,
    Path(device_id): Path<String>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Json(status): Json<UpdateDeviceStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    
    match app_state.db.update_device_online_status(&device_id, status.is_online).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Device status updated"
        }))),
        Err(e) => {
            error!("Update device status error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new session
pub async fn create_session(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(session): Json<Session>,
) -> Result<Json<Session>, StatusCode> {
    
    // Ensure session belongs to authenticated user
    if session.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    match app_state.db.create_session(&session).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => {
            error!("Session creation error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user sessions
pub async fn get_sessions(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Vec<Session>>, StatusCode> {
    match app_state.db.get_user_sessions(user.id, 50).await {
        Ok(sessions) => Ok(Json(sessions)),
        Err(e) => {
            error!("Get sessions error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// End a session
pub async fn end_session(
    State(app_state): State<AppState>,
    Path(session_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    
    // TODO: Verify session belongs to user before ending
    match app_state.db.end_session(session_id).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Session ended"
        }))),
        Err(e) => {
            error!("End session error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// License activation request
#[derive(Debug, Deserialize)]
pub struct LicenseActivationRequest {
    pub license_key: String,
}

#[derive(Debug, Serialize)]
pub struct LicenseActivationResponse {
    pub success: bool,
    pub message: String,
    pub license: Option<License>,
}

pub async fn activate_license(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(license_request): Json<LicenseActivationRequest>,
) -> Result<Json<LicenseActivationResponse>, StatusCode> {
    
    match app_state.db.get_license_by_key(&license_request.license_key).await {
        Ok(Some(license)) => {
            if license.user_id != user.id {
                return Ok(Json(LicenseActivationResponse {
                    success: false,
                    message: "License does not belong to this user".to_string(),
                    license: None,
                }));
            }
            
            Ok(Json(LicenseActivationResponse {
                success: true,
                message: "License activated successfully".to_string(),
                license: Some(license),
            }))
        }
        Ok(None) => Ok(Json(LicenseActivationResponse {
            success: false,
            message: "Invalid license key".to_string(),
            license: None,
        })),
        Err(e) => {
            error!("License activation error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get license status
pub async fn license_status(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Vec<License>>, StatusCode> {
    match app_state.db.get_user_licenses(user.id).await {
        Ok(licenses) => Ok(Json(licenses)),
        Err(e) => {
            error!("Get license status error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Change password
pub async fn change_password(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(request): Json<PasswordChangeRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    match app_state.auth_service.change_password(user.id, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Change password error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Start connection request
#[derive(Debug, Deserialize)]
pub struct StartConnectionRequest {
    pub from_device_id: Uuid,
    pub to_device_id: Uuid,
    pub connection_type: String,
}

#[derive(Debug, Serialize)]
pub struct StartConnectionResponse {
    pub success: bool,
    pub message: String,
    pub session_id: Option<Uuid>,
}

pub async fn start_connection(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(connection_request): Json<StartConnectionRequest>,
) -> Result<Json<StartConnectionResponse>, StatusCode> {
    
    // Create a new session for the connection
    let session = Session {
        id: Uuid::new_v4(),
        user_id: user.id,
        device_id: connection_request.from_device_id,
        remote_device_id: connection_request.to_device_id,
        session_type: connection_request.connection_type,
        started_at: SystemTime::now().into(),
        ended_at: None,
        duration_seconds: None,
        status: "active".to_string(),
        connection_quality: None,
        metadata: serde_json::json!({}),
    };
    
    match app_state.db.create_session(&session).await {
        Ok(created_session) => Ok(Json(StartConnectionResponse {
            success: true,
            message: "Connection started successfully".to_string(),
            session_id: Some(created_session.id),
        })),
        Err(e) => {
            error!("Start connection error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// End connection request
#[derive(Debug, Deserialize)]
pub struct EndConnectionRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct EndConnectionResponse {
    pub success: bool,
    pub message: String,
}

pub async fn end_connection(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(connection_request): Json<EndConnectionRequest>,
) -> Result<Json<EndConnectionResponse>, StatusCode> {
    
    // TODO: Verify session belongs to user before ending
    match app_state.db.end_session(connection_request.session_id).await {
        Ok(_) => Ok(Json(EndConnectionResponse {
            success: true,
            message: "Connection ended successfully".to_string(),
        })),
        Err(e) => {
            error!("End connection error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get system statistics
#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_users: i64,
    pub total_devices: i64,
    pub active_sessions: i64,
    pub total_connections: i64,
}

pub async fn get_system_stats(
    State(_app_state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<SystemStats>, StatusCode> {
    
    // TODO: Implement actual statistics queries
    let stats = SystemStats {
        total_users: 0,
        total_devices: 0,
        active_sessions: 0,
        total_connections: 0,
    };
    
    Ok(Json(stats))
}

/// Get API usage statistics
#[derive(Debug, Serialize)]
pub struct UsageStats {
    pub api_calls: i64,
    pub bandwidth_used: i64,
    pub session_duration: i64,
}

pub async fn get_usage_stats(
    State(_app_state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<UsageStats>, StatusCode> {
    
    // TODO: Implement actual usage statistics for the user
    let stats = UsageStats {
        api_calls: 0,
        bandwidth_used: 0,
        session_duration: 0,
    };
    
    Ok(Json(stats))
}
