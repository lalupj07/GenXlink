use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth_service::{AuthService, AuthSession};
use crate::database::{UserAccount, UserPreferences, SubscriptionType};
use crate::webrtc_integration::{WebRTCIntegration, IntegrationState, IntegrationEvent};
use genxlink_protocol::DeviceId;

/// Secure session manager
/// Handles user sessions, authentication tokens, and secure connections
pub struct SessionManager {
    current_session: Arc<RwLock<Option<UserSession>>>,
    active_connections: Arc<RwLock<HashMap<DeviceId, ActiveConnection>>>,
    auth_service: Arc<Mutex<AuthService>>,
    webrtc_integration: Arc<Mutex<Option<WebRTCIntegration>>>,
    config: SessionConfig,
    event_handlers: Arc<RwLock<Vec<Box<dyn SessionEventHandler>>>>,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub session_timeout: Duration,
    pub max_concurrent_sessions: usize,
    pub require_reauth_after: Duration,
    pub enable_persistence: bool,
    pub encryption_key_rotation_interval: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_timeout: Duration::from_secs(30 * 60), // 30 minutes
            max_concurrent_sessions: 5,
            require_reauth_after: Duration::from_secs(24 * 60 * 60), // 24 hours
            enable_persistence: true,
            encryption_key_rotation_interval: Duration::from_secs(60 * 60), // 1 hour
        }
    }
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: String,
    pub device_id: DeviceId,
    pub auth_session: AuthSession,
    pub created_at: u64,
    pub last_activity: u64,
    pub expires_at: u64,
    pub preferences: UserPreferences,
    pub is_premium: bool,
}

/// Active connection information
#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub device_id: DeviceId,
    pub connection_id: String,
    pub started_at: u64,
    pub last_ping: u64,
    pub state: ConnectionState,
    pub stats: ConnectionStats,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Streaming,
    Disconnected,
    Failed(String),
}

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub frames_sent: u64,
    pub frames_received: u64,
    pub rtt_ms: u32,
    pub packet_loss: f32,
}

/// Session events
#[derive(Debug, Clone)]
pub enum SessionEvent {
    SessionCreated(UserSession),
    SessionExpired(String),
    SessionTerminated(String),
    ConnectionEstablished(DeviceId),
    ConnectionLost(DeviceId),
    AuthenticationRequired,
    SecurityViolation(String),
}

/// Session event handler trait
pub trait SessionEventHandler: Send + Sync {
    fn on_session_event(&self, event: SessionEvent);
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(auth_service: AuthService, config: SessionConfig) -> Self {
        Self {
            current_session: Arc::new(RwLock::new(None)),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            auth_service: Arc::new(Mutex::new(auth_service)),
            webrtc_integration: Arc::new(Mutex::new(None)),
            config,
            event_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize a user session
    pub async fn create_session(&self, auth_session: AuthSession, user: UserAccount, device_id: DeviceId) -> Result<String> {
        info!("Creating session for user: {}", user.id);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = UserSession {
            session_id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            device_id: device_id.clone(),
            auth_session,
            created_at: now,
            last_activity: now,
            expires_at: now + self.config.session_timeout.as_secs(),
            preferences: user.preferences.clone(),
            is_premium: user.subscription_type != crate::database::SubscriptionType::Free,
        };

        // Store session
        let session_id = session.session_id.clone();
        let mut session_guard = self.current_session.write().await;
        *session_guard = Some(session.clone());
        drop(session_guard);

        // Persist session if enabled
        if self.config.enable_persistence {
            self.persist_session(&session).await?;
        }

        // Emit event
        self.emit_event(SessionEvent::SessionCreated(session)).await;

        info!("Session created successfully: {}", session_id);
        Ok(session_id)
    }

    /// Get current session
    pub async fn get_current_session(&self) -> Option<UserSession> {
        self.current_session.read().await.clone()
    }

    /// Validate and refresh session
    pub async fn validate_session(&self) -> Result<bool> {
        let mut session_guard = self.current_session.write().await;
        
        if let Some(session) = session_guard.as_mut() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Check if session expired
            if now >= session.expires_at {
                warn!("Session expired: {}", session.session_id);
                let session_id = session.session_id.clone();
                *session_guard = None;
                drop(session_guard);
                
                self.emit_event(SessionEvent::SessionExpired(session_id)).await;
                return Ok(false);
            }

            // Update last activity
            session.last_activity = now;
            
            // Extend expiry if within threshold
            if now + self.config.session_timeout.as_secs() / 2 > session.expires_at {
                session.expires_at = now + self.config.session_timeout.as_secs();
                debug!("Session extended: {}", session.session_id);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Terminate current session
    pub async fn terminate_session(&self) -> Result<()> {
        let mut session_guard = self.current_session.write().await;
        
        if let Some(session) = session_guard.take() {
            info!("Terminating session: {}", session.session_id);
            
            // Disconnect all active connections
            self.disconnect_all().await?;
            
            // Clear persisted session
            if self.config.enable_persistence {
                self.clear_persisted_session().await?;
            }

            self.emit_event(SessionEvent::SessionTerminated(session.session_id)).await;
        }

        Ok(())
    }

    /// Establish a connection to a remote device
    pub async fn establish_connection(&self, remote_device_id: DeviceId, signaling_server_url: String) -> Result<String> {
        info!("Establishing connection to device: {}", remote_device_id);

        // Validate session first
        if !self.validate_session().await? {
            return Err(anyhow::anyhow!("No valid session"));
        }

        // Check connection limit
        let connections = self.active_connections.read().await;
        if connections.len() >= self.config.max_concurrent_sessions {
            return Err(anyhow::anyhow!("Maximum concurrent connections reached"));
        }
        drop(connections);

        let connection_id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let connection = ActiveConnection {
            device_id: remote_device_id.clone(),
            connection_id: connection_id.clone(),
            started_at: now,
            last_ping: now,
            state: ConnectionState::Connecting,
            stats: ConnectionStats::default(),
        };

        // Store connection
        let mut connections = self.active_connections.write().await;
        connections.insert(remote_device_id.clone(), connection);
        drop(connections);

        // Start WebRTC connection
        self.start_webrtc_connection(remote_device_id, signaling_server_url).await?;

        info!("Connection established: {}", connection_id);
        Ok(connection_id)
    }

    /// Disconnect from a specific device
    pub async fn disconnect(&self, device_id: &DeviceId) -> Result<()> {
        info!("Disconnecting from device: {}", device_id);

        // Remove from active connections
        let mut connections = self.active_connections.write().await;
        if connections.remove(device_id).is_some() {
            drop(connections);
            
            // Stop WebRTC session
            let webrtc_guard = self.webrtc_integration.lock().await;
            if let Some(integration) = webrtc_guard.as_ref() {
                integration.stop_streaming().await?;
            }
            drop(webrtc_guard);

            self.emit_event(SessionEvent::ConnectionLost(device_id.clone())).await;
        }

        Ok(())
    }

    /// Disconnect from all devices
    pub async fn disconnect_all(&self) -> Result<()> {
        info!("Disconnecting from all devices");

        let connections: Vec<DeviceId> = self.active_connections.read().await
            .keys()
            .cloned()
            .collect();

        for device_id in connections {
            self.disconnect(&device_id).await?;
        }

        Ok(())
    }

    /// Get active connections
    pub async fn get_active_connections(&self) -> HashMap<DeviceId, ActiveConnection> {
        self.active_connections.read().await.clone()
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> SessionStats {
        let session = self.get_current_session().await;
        let connections = self.get_active_connections().await;
        
        let total_bytes_sent = connections.values()
            .map(|c| c.stats.bytes_sent)
            .sum();
        
        let total_bytes_received = connections.values()
            .map(|c| c.stats.bytes_received)
            .sum();

        SessionStats {
            session_id: session.as_ref().map(|s| s.session_id.clone()),
            user_id: session.as_ref().map(|s| s.user_id.clone()),
            active_connections: connections.len(),
            total_bytes_sent,
            total_bytes_received,
            session_duration: session.map(|s| {
                Duration::from_secs(SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - s.created_at)
            }),
        }
    }

    /// Add event handler
    pub async fn add_event_handler(&self, handler: Box<dyn SessionEventHandler>) {
        let mut handlers = self.event_handlers.write().await;
        handlers.push(handler);
    }

    /// Start WebRTC connection
    async fn start_webrtc_connection(&self, _remote_device_id: DeviceId, _signaling_server_url: String) -> Result<()> {
        // TODO: Implement WebRTC connection setup
        // This is a placeholder implementation
        info!("Starting WebRTC connection (not yet implemented)");
        Ok(())
    }

    /// Emit session event
    async fn emit_event(&self, event: SessionEvent) {
        let handlers = self.event_handlers.read().await;
        for handler in handlers.iter() {
            handler.on_session_event(event.clone());
        }
    }

    /// Persist session to storage
    async fn persist_session(&self, session: &UserSession) -> Result<()> {
        // TODO: Implement secure session persistence
        debug!("Persisting session: {}", session.session_id);
        Ok(())
    }

    /// Clear persisted session
    async fn clear_persisted_session(&self) -> Result<()> {
        // TODO: Implement session cleanup
        debug!("Clearing persisted session");
        Ok(())
    }

    /// Load persisted session
    pub async fn load_persisted_session(&self) -> Result<Option<UserSession>> {
        // TODO: Implement session loading
        debug!("Loading persisted session");
        Ok(None)
    }

    /// Update session preferences
    pub async fn update_preferences(&self, preferences: UserPreferences) -> Result<()> {
        let mut session_guard = self.current_session.write().await;
        if let Some(session) = session_guard.as_mut() {
            session.preferences = preferences.clone();
            
            // Persist changes
            if self.config.enable_persistence {
                self.persist_session(session).await?;
            }
        }
        Ok(())
    }

    /// Check if user has premium features
    pub async fn is_premium(&self) -> bool {
        self.get_current_session().await
            .map(|s| s.is_premium)
            .unwrap_or(false)
    }

    /// Get user preferences
    pub async fn get_preferences(&self) -> Option<UserPreferences> {
        self.get_current_session().await
            .map(|s| s.preferences)
    }
}

/// Session statistics
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub active_connections: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub session_duration: Option<Duration>,
}

/// Default session event handler
pub struct DefaultSessionEventHandler;

impl SessionEventHandler for DefaultSessionEventHandler {
    fn on_session_event(&self, event: SessionEvent) {
        match event {
            SessionEvent::SessionCreated(session) => {
                info!("Session created: {}", session.session_id);
            }
            SessionEvent::SessionExpired(session_id) => {
                warn!("Session expired: {}", session_id);
            }
            SessionEvent::ConnectionEstablished(device_id) => {
                info!("Connection established: {}", device_id);
            }
            SessionEvent::ConnectionLost(device_id) => {
                warn!("Connection lost: {}", device_id);
            }
            SessionEvent::SecurityViolation(msg) => {
                error!("Security violation: {}", msg);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let auth_service = AuthService::new(
            "http://localhost:8000".to_string(),
            "test-key".to_string(),
        );
        let config = SessionConfig::default();
        let manager = SessionManager::new(auth_service, config);

        let auth_session = AuthSession {
            access_token: "test-token".to_string(),
            refresh_token: "test-refresh".to_string(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600,
            user_id: "test-user".to_string(),
        };

        let user = UserAccount {
            id: "test-user".to_string(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: None,
            subscription_type: SubscriptionType::Free,
            created_at: SystemTime::now(),
            last_login: Some(SystemTime::now()),
            preferences: UserPreferences::default(),
            is_active: true,
            is_verified: true,
        };

        let device_id = "test-device".to_string();
        let session_id = manager.create_session(auth_session, user, genxlink_protocol::DeviceId(device_id)).await.unwrap();

        assert!(!session_id.is_empty());
        
        let session = manager.get_current_session().await.unwrap();
        assert_eq!(session.session_id, session_id);
    }

    #[tokio::test]
    async fn test_session_validation() {
        let auth_service = AuthService::new(
            "http://localhost:8000".to_string(),
            "test-key".to_string(),
        );
        let config = SessionConfig::default();
        let manager = SessionManager::new(auth_service, config);

        // Should be false when no session
        assert!(!manager.validate_session().await.unwrap());
    }
}
