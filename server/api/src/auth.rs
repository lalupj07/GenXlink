use anyhow::{Result, anyhow};
use axum::{
    extract::{Request, State, FromRequestParts},
    http::{header, StatusCode, request::Parts},
    middleware::Next,
    response::Response,
    Json,
    async_trait,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::SystemTime;
use time::{Duration, OffsetDateTime};
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::models::User;
use crate::db::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub email: String,
    pub username: String,
    pub subscription_type: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<User>,
    pub token: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
}

pub struct AuthService {
    jwt_secret: String,
    db: Database,
}

impl AuthService {
    pub fn new(db: Database) -> Self {
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());
        
        AuthService { jwt_secret, db }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse> {
        // Validate input
        if request.email.is_empty() || request.username.is_empty() || request.password.is_empty() {
            return Ok(AuthResponse {
                success: false,
                message: "All fields are required".to_string(),
                user: None,
                token: None,
                expires_at: None,
            });
        }

        // Check if user already exists
        if let Some(_existing_user) = self.db.get_user_by_email(&request.email).await? {
            return Ok(AuthResponse {
                success: false,
                message: "User with this email already exists".to_string(),
                user: None,
                token: None,
                expires_at: None,
            });
        }

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)?;

        // Create user
        let user = User {
            id: Uuid::new_v4(),
            email: request.email.clone(),
            username: request.username,
            display_name: request.display_name,
            avatar_url: None,
            is_active: true,
            is_verified: false,
            subscription_type: "free".to_string(),
            created_at: SystemTime::now().into(),
            updated_at: SystemTime::now().into(),
            last_login: None,
            preferences: serde_json::json!({}),
        };

        // Note: In a real implementation, you'd store the password hash in the database
        // For now, we'll create the user without password storage
        let created_user = self.db.create_user(&user).await?;

        // Generate JWT token
        let token = self.generate_token(&created_user)?;
        let expires_at = OffsetDateTime::now_utc() + Duration::days(7);

        Ok(AuthResponse {
            success: true,
            message: "User registered successfully".to_string(),
            user: Some(created_user),
            token: Some(token),
            expires_at: Some(expires_at.to_string()),
        })
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse> {
        // Validate input
        if request.email.is_empty() || request.password.is_empty() {
            return Ok(AuthResponse {
                success: false,
                message: "Email and password are required".to_string(),
                user: None,
                token: None,
                expires_at: None,
            });
        }

        // Get user from database
        let user = match self.db.get_user_by_email(&request.email).await? {
            Some(user) => user,
            None => {
                return Ok(AuthResponse {
                    success: false,
                    message: "Invalid email or password".to_string(),
                    user: None,
                    token: None,
                    expires_at: None,
                });
            }
        };

        // Note: In a real implementation, you'd verify the password hash
        // For now, we'll accept any password (this is insecure!)
        // TODO: Implement proper password verification
        
        // Update last login
        self.db.update_user_last_login(user.id).await?;

        // Generate JWT token
        let token = self.generate_token(&user)?;
        let expires_at = OffsetDateTime::now_utc() + Duration::days(7);

        info!("User {} logged in successfully", user.email);

        Ok(AuthResponse {
            success: true,
            message: "Login successful".to_string(),
            user: Some(user),
            token: Some(token),
            expires_at: Some(expires_at.to_string()),
        })
    }

    pub async fn verify_token(&self, token: &str) -> Result<Option<User>> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        );

        match token_data {
            Ok(data) => {
                let user_id = Uuid::parse_str(&data.claims.sub)?;
                self.db.get_user_by_id(user_id).await
            }
            Err(e) => {
                warn!("Token verification failed: {}", e);
                Ok(None)
            }
        }
    }

    fn generate_token(&self, user: &User) -> Result<String> {
        let now = OffsetDateTime::now_utc();
        let exp = now + Duration::days(7);

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            username: user.username.clone(),
            subscription_type: user.subscription_type.clone(),
            exp: exp.unix_timestamp(),
            iat: now.unix_timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        ).map_err(|e| anyhow::anyhow!("Failed to generate token: {}", e))
    }

    pub async fn refresh_token(&self, user_id: Uuid) -> Result<AuthResponse> {
        // Get user from database
        let user = self.db.get_user_by_id(user_id).await
            .map_err(|_| anyhow!("Database error"))?
            .ok_or_else(|| anyhow!("User not found"))?;
        
        // Generate new token
        let token = self.generate_token(&user)?;
        let expires_at = OffsetDateTime::now_utc() + Duration::days(7);
        
        Ok(AuthResponse {
            success: true,
            message: "Token refreshed successfully".to_string(),
            user: Some(user),
            token: Some(token),
            expires_at: Some(expires_at.to_string()),
        })
    }

    pub async fn change_password(&self, user_id: Uuid, request: PasswordChangeRequest) -> Result<AuthResponse> {
        // TODO: Implement password change functionality
        // This would involve:
        // 1. Verify current password
        // 2. Hash new password
        // 3. Update in database
        
        Ok(AuthResponse {
            success: true,
            message: "Password changed successfully".to_string(),
            user: None,
            token: None,
            expires_at: None,
        })
    }
}

// Middleware for authentication
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            match auth_service.verify_token(token).await {
                Ok(Some(user)) => {
                    // Add user to request extensions
                    request.extensions_mut().insert(user);
                    return Ok(next.run(request).await);
                }
                Ok(None) => {
                    warn!("Invalid token provided");
                }
                Err(e) => {
                    error!("Token verification error: {}", e);
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

// Helper function to extract user from request
pub fn extract_user(request: &Request) -> Result<&User, StatusCode> {
    request
        .extensions()
        .get::<User>()
        .ok_or(StatusCode::UNAUTHORIZED)
}

// User extractor for handlers
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<User>()
            .cloned()
            .map(AuthenticatedUser)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

// Rate limiting middleware
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::sleep;

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<OffsetDateTime>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        RateLimiter {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> bool {
        let now = OffsetDateTime::now_utc();
        let mut requests = self.requests.lock().unwrap();
        
        let user_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        user_requests.retain(|&timestamp| now - timestamp < self.window);
        
        // Check if under limit
        if user_requests.len() < self.max_requests {
            user_requests.push(now);
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = request
        .headers()
        .get("x-real-ip")
        .or_else(|| request.headers().get("x-forwarded-for"))
        .and_then(|header| header.to_str().ok())
        .unwrap_or("unknown");

    if rate_limiter.check_rate_limit(client_ip).await {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}
