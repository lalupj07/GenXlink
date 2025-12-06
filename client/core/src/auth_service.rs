use crate::database::{DatabaseClient, UserAccount, UserPreferences, SubscriptionType};
use crate::ClientError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication service for Supabase
pub struct AuthService {
    client: reqwest::Client,
    base_url: String,
    anon_key: String,
    database_client: DatabaseClient,
    current_user: Option<UserAccount>,
    session: Option<AuthSession>,
}

/// Authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub user_id: String,
}

/// Login request
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Register request
#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub username: String,
    pub display_name: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserAccount,
    pub session: AuthSession,
}

/// Password reset request
#[derive(Debug, Clone, Serialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// Update profile request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<UserPreferences>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(base_url: String, anon_key: String) -> Self {
        let database_client = DatabaseClient::new(format!("{}/rest/v1", base_url), anon_key.clone());
        
        Self {
            client: reqwest::Client::new(),
            base_url,
            anon_key,
            database_client,
            current_user: None,
            session: None,
        }
    }

    /// Register a new user
    pub async fn register(&mut self, request: RegisterRequest) -> Result<AuthResponse, ClientError> {
        let url = format!("{}/auth/v1/signup", self.base_url);
        
        let mut body = HashMap::new();
        body.insert("email".to_string(), request.email);
        body.insert("password".to_string(), request.password);
        body.insert("data".to_string(), serde_json::json!({
            "username": request.username,
            "display_name": request.display_name,
        }).to_string());
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Auth request failed: {}", e)))?;

        if response.status().is_success() {
            let auth_response: SupabaseAuthResponse = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse auth response: {}", e)))?;
            
            let user = self.convert_supabase_user(auth_response.user)?;
            let session = AuthSession {
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at: auth_response.expires_at.unwrap_or(0),
                user_id: user.id.clone(),
            };

            // Update database client with auth token
            self.database_client.set_auth_token(session.access_token.clone());
            
            // Store current session
            self.current_user = Some(user.clone());
            self.session = Some(session.clone());

            Ok(AuthResponse { user, session })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(ClientError::IoError(format!("Registration failed: {}", error_text)))
        }
    }

    /// Login user
    pub async fn login(&mut self, request: LoginRequest) -> Result<AuthResponse, ClientError> {
        let url = format!("{}/auth/v1/token?grant_type=password", self.base_url);
        
        let mut body = HashMap::new();
        body.insert("email".to_string(), request.email);
        body.insert("password".to_string(), request.password);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Auth request failed: {}", e)))?;

        if response.status().is_success() {
            let auth_response: SupabaseAuthResponse = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse auth response: {}", e)))?;
            
            let user = self.convert_supabase_user(auth_response.user)?;
            let session = AuthSession {
                access_token: auth_response.access_token,
                refresh_token: auth_response.refresh_token,
                expires_at: auth_response.expires_at.unwrap_or(0),
                user_id: user.id.clone(),
            };

            // Update database client with auth token
            self.database_client.set_auth_token(session.access_token.clone());
            
            // Store current session
            self.current_user = Some(user.clone());
            self.session = Some(session.clone());

            Ok(AuthResponse { user, session })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(ClientError::IoError(format!("Login failed: {}", error_text)))
        }
    }

    /// Logout user
    pub async fn logout(&mut self) -> Result<(), ClientError> {
        if let Some(session) = &self.session {
            let url = format!("{}/auth/v1/logout", self.base_url);
            
            let response = self.client
                .post(&url)
                .header("apikey", &self.anon_key)
                .header("Authorization", format!("Bearer {}", session.access_token))
                .send()
                .await
                .map_err(|e| ClientError::IoError(format!("Logout request failed: {}", e)))?;

            // Clear local session regardless of response
            self.current_user = None;
            self.session = None;
            
            // Reset database client to use anon key
            self.database_client.set_auth_token(self.anon_key.clone());

            if !response.status().is_success() {
                tracing::warn!("Logout request failed: {}", response.status());
            }
        }
        
        Ok(())
    }

    /// Refresh authentication token
    pub async fn refresh_token(&mut self) -> Result<AuthSession, ClientError> {
        if let Some(session) = &self.session {
            let url = format!("{}/auth/v1/token?grant_type=refresh_token", self.base_url);
            
            let mut body = HashMap::new();
            body.insert("refresh_token".to_string(), session.refresh_token.clone());
            
            let response = self.client
                .post(&url)
                .header("apikey", &self.anon_key)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ClientError::IoError(format!("Token refresh failed: {}", e)))?;

            if response.status().is_success() {
                let auth_response: SupabaseAuthResponse = response.json().await
                    .map_err(|e| ClientError::IoError(format!("Failed to parse refresh response: {}", e)))?;
                
                let new_session = AuthSession {
                    access_token: auth_response.access_token,
                    refresh_token: auth_response.refresh_token,
                    expires_at: auth_response.expires_at.unwrap_or(0),
                    user_id: session.user_id.clone(),
                };

                // Update database client with new token
                self.database_client.set_auth_token(new_session.access_token.clone());
                
                // Store new session
                self.session = Some(new_session.clone());

                Ok(new_session)
            } else {
                Err(ClientError::IoError("Token refresh failed".to_string()))
            }
        } else {
            Err(ClientError::IoError("No active session to refresh".to_string()))
        }
    }

    /// Request password reset
    pub async fn reset_password(&self, request: PasswordResetRequest) -> Result<(), ClientError> {
        let url = format!("{}/auth/v1/recover", self.base_url);
        
        let mut body = HashMap::new();
        body.insert("email".to_string(), request.email);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Password reset request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(ClientError::IoError(format!("Password reset failed: {}", error_text)))
        }
    }

    /// Get current user
    pub fn get_current_user(&self) -> Option<&UserAccount> {
        self.current_user.as_ref()
    }

    /// Get current session
    pub fn get_current_session(&self) -> Option<&AuthSession> {
        self.session.as_ref()
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.session.is_some() && self.current_user.is_some()
    }

    /// Update user profile
    pub async fn update_profile(&mut self, request: UpdateProfileRequest) -> Result<UserAccount, ClientError> {
        if let Some(user) = &self.current_user {
            let url = format!("{}/rest/v1/users?id=eq.{}", self.base_url, user.id);
            
            let mut updates = HashMap::new();
            if let Some(display_name) = request.display_name {
                updates.insert("display_name".to_string(), serde_json::Value::String(display_name));
            }
            if let Some(avatar_url) = request.avatar_url {
                updates.insert("avatar_url".to_string(), serde_json::Value::String(avatar_url));
            }
            if let Some(preferences) = request.preferences {
                updates.insert("preferences".to_string(), serde_json::to_value(preferences).unwrap());
            }
            
            let response = self.client
                .patch(&url)
                .header("apikey", &self.anon_key)
                .header("Authorization", format!("Bearer {}", self.session.as_ref().unwrap().access_token))
                .header("Content-Type", "application/json")
                .header("Prefer", "return=representation")
                .json(&updates)
                .send()
                .await
                .map_err(|e| ClientError::IoError(format!("Profile update failed: {}", e)))?;

            if response.status().is_success() {
                let users: Vec<UserAccount> = response.json().await
                    .map_err(|e| ClientError::IoError(format!("Failed to parse profile response: {}", e)))?;
                
                if let Some(updated_user) = users.into_iter().next() {
                    self.current_user = Some(updated_user.clone());
                    Ok(updated_user)
                } else {
                    Err(ClientError::IoError("No user returned from update".to_string()))
                }
            } else {
                Err(ClientError::IoError("Profile update failed".to_string()))
            }
        } else {
            Err(ClientError::IoError("No authenticated user".to_string()))
        }
    }

    /// Get database client
    pub fn get_database_client(&self) -> &DatabaseClient {
        &self.database_client
    }

    /// Convert Supabase user to our UserAccount format
    fn convert_supabase_user(&self, supabase_user: SupabaseUser) -> Result<UserAccount, ClientError> {
        let user_data = supabase_user.user_metadata.unwrap_or_default();
        let email = supabase_user.email.clone();
        
        Ok(UserAccount {
            id: supabase_user.id,
            email: email.clone(),
            username: user_data.get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            display_name: user_data.get("display_name")
                .and_then(|v| v.as_str())
                .unwrap_or(&email)
                .to_string(),
            avatar_url: user_data.get("avatar_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            is_active: true, // Supabase doesn't have this concept directly
            is_verified: supabase_user.email_confirmed_at.is_some(),
            subscription_type: SubscriptionType::Free, // Default to free, can be updated later
            created_at: supabase_user.created_at.unwrap_or(SystemTime::UNIX_EPOCH),
            last_login: supabase_user.last_sign_in_at,
            preferences: UserPreferences::default(),
        })
    }

    /// Validate session token
    pub async fn validate_session(&mut self) -> Result<bool, ClientError> {
        if let Some(session) = &self.session {
            let url = format!("{}/auth/v1/user", self.base_url);
            
            let response = self.client
                .get(&url)
                .header("apikey", &self.anon_key)
                .header("Authorization", format!("Bearer {}", session.access_token))
                .send()
                .await
                .map_err(|e| ClientError::IoError(format!("Session validation failed: {}", e)))?;

            if response.status().is_success() {
                let supabase_user: SupabaseUser = response.json().await
                    .map_err(|e| ClientError::IoError(format!("Failed to parse user response: {}", e)))?;
                
                let user = self.convert_supabase_user(supabase_user)?;
                self.current_user = Some(user);
                Ok(true)
            } else {
                // Session is invalid, clear it
                self.current_user = None;
                self.session = None;
                self.database_client.set_auth_token(self.anon_key.clone());
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}

/// Supabase authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SupabaseAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<u64>,
    pub expires_at: Option<u64>,
    pub user: SupabaseUser,
}

/// Supabase user structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SupabaseUser {
    pub id: String,
    pub email: String,
    pub created_at: Option<SystemTime>,
    pub updated_at: Option<SystemTime>,
    pub last_sign_in_at: Option<SystemTime>,
    pub email_confirmed_at: Option<SystemTime>,
    pub phone: Option<String>,
    pub phone_confirmed_at: Option<SystemTime>,
    pub user_metadata: Option<HashMap<String, serde_json::Value>>,
    pub app_metadata: Option<HashMap<String, serde_json::Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service_creation() {
        let auth_service = AuthService::new(
            "https://test.supabase.co".to_string(),
            "test_key".to_string(),
        );
        
        assert!(!auth_service.is_authenticated());
        assert!(auth_service.get_current_user().is_none());
        assert!(auth_service.get_current_session().is_none());
    }

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("password123"));
    }

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("testuser"));
        assert!(json.contains("Test User"));
    }

    #[test]
    fn test_auth_session_serialization() {
        let session = AuthSession {
            access_token: "test_token".to_string(),
            refresh_token: "refresh_token".to_string(),
            expires_at: 1234567890,
            user_id: "user_123".to_string(),
        };
        
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("test_token"));
        assert!(json.contains("refresh_token"));
        assert!(json.contains("1234567890"));
        assert!(json.contains("user_123"));
    }
}
