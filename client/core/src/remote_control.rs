use crate::{ClientError, input::InputInjector};
use genxlink_protocol::{MouseEvent, KeyboardEvent, MouseEventType};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

/// Remote control event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteControlEvent {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
}

/// Remote control handler
pub struct RemoteControlHandler {
    injector: Arc<Mutex<Box<dyn InputInjector>>>,
    enabled: Arc<Mutex<bool>>,
    event_count: Arc<Mutex<u64>>,
}

impl RemoteControlHandler {
    /// Create a new remote control handler
    pub fn new(injector: Box<dyn InputInjector>) -> Self {
        Self {
            injector: Arc::new(Mutex::new(injector)),
            enabled: Arc::new(Mutex::new(true)),
            event_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Enable remote control
    pub async fn enable(&self) {
        let mut enabled = self.enabled.lock().await;
        *enabled = true;
        tracing::info!("Remote control enabled");
    }

    /// Disable remote control
    pub async fn disable(&self) {
        let mut enabled = self.enabled.lock().await;
        *enabled = false;
        tracing::info!("Remote control disabled");
    }

    /// Check if remote control is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }

    /// Handle a remote control event
    pub async fn handle_event(&self, event: RemoteControlEvent) -> Result<(), ClientError> {
        // Check if enabled
        if !self.is_enabled().await {
            return Err(ClientError::InputError("Remote control is disabled".to_string()));
        }

        // Increment event counter
        {
            let mut count = self.event_count.lock().await;
            *count += 1;
        }

        // Get injector
        let mut injector = self.injector.lock().await;

        // Handle event
        match event {
            RemoteControlEvent::Mouse(mouse_event) => {
                injector.inject_mouse(&mouse_event)?;
                tracing::trace!("Mouse event: {:?}", mouse_event.event_type);
            }
            RemoteControlEvent::Keyboard(keyboard_event) => {
                injector.inject_keyboard(&keyboard_event)?;
                tracing::debug!("Keyboard event: key_code={}", keyboard_event.key_code);
            }
        }

        Ok(())
    }

    /// Get event statistics
    pub async fn get_event_count(&self) -> u64 {
        *self.event_count.lock().await
    }

    /// Reset event counter
    pub async fn reset_counter(&self) {
        let mut count = self.event_count.lock().await;
        *count = 0;
    }
}

/// Remote control session
pub struct RemoteControlSession {
    handler: Arc<RemoteControlHandler>,
    session_id: String,
    started_at: std::time::Instant,
}

impl RemoteControlSession {
    /// Create a new remote control session
    pub fn new(handler: Arc<RemoteControlHandler>, session_id: String) -> Self {
        Self {
            handler,
            session_id,
            started_at: std::time::Instant::now(),
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get session duration
    pub fn duration(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    /// Handle event in this session
    pub async fn handle_event(&self, event: RemoteControlEvent) -> Result<(), ClientError> {
        self.handler.handle_event(event).await
    }

    /// Get handler
    pub fn handler(&self) -> Arc<RemoteControlHandler> {
        Arc::clone(&self.handler)
    }
}

/// Remote control manager for multiple sessions
pub struct RemoteControlManager {
    sessions: Arc<Mutex<Vec<Arc<RemoteControlSession>>>>,
}

impl RemoteControlManager {
    /// Create a new remote control manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        injector: Box<dyn InputInjector>,
        session_id: String,
    ) -> Arc<RemoteControlSession> {
        let handler = Arc::new(RemoteControlHandler::new(injector));
        let session = Arc::new(RemoteControlSession::new(handler, session_id));
        
        let mut sessions = self.sessions.lock().await;
        sessions.push(Arc::clone(&session));
        
        tracing::info!("Remote control session created: {}", session.session_id());
        
        session
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<RemoteControlSession>> {
        let sessions = self.sessions.lock().await;
        sessions.iter()
            .find(|s| s.session_id() == session_id)
            .map(Arc::clone)
    }

    /// Remove session
    pub async fn remove_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.lock().await;
        let initial_len = sessions.len();
        sessions.retain(|s| s.session_id() != session_id);
        
        let removed = sessions.len() < initial_len;
        if removed {
            tracing::info!("Remote control session removed: {}", session_id);
        }
        
        removed
    }

    /// Get active session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.lock().await;
        sessions.len()
    }

    /// Clear all sessions
    pub async fn clear_sessions(&self) {
        let mut sessions = self.sessions.lock().await;
        let count = sessions.len();
        sessions.clear();
        tracing::info!("Cleared {} remote control sessions", count);
    }
}

impl Default for RemoteControlManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission levels for remote control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// View only, no control
    ViewOnly,
    /// Mouse control only
    MouseOnly,
    /// Keyboard control only
    KeyboardOnly,
    /// Full control (mouse + keyboard)
    FullControl,
}

impl PermissionLevel {
    /// Check if mouse events are allowed
    pub fn allows_mouse(&self) -> bool {
        matches!(self, PermissionLevel::MouseOnly | PermissionLevel::FullControl)
    }

    /// Check if keyboard events are allowed
    pub fn allows_keyboard(&self) -> bool {
        matches!(self, PermissionLevel::KeyboardOnly | PermissionLevel::FullControl)
    }
}

/// Remote control session with permissions
pub struct PermissionedSession {
    session: Arc<RemoteControlSession>,
    permissions: PermissionLevel,
}

impl PermissionedSession {
    /// Create a new permissioned session
    pub fn new(session: Arc<RemoteControlSession>, permissions: PermissionLevel) -> Self {
        Self {
            session,
            permissions,
        }
    }

    /// Handle event with permission check
    pub async fn handle_event(&self, event: RemoteControlEvent) -> Result<(), ClientError> {
        // Check permissions
        match &event {
            RemoteControlEvent::Mouse(_) => {
                if !self.permissions.allows_mouse() {
                    return Err(ClientError::InputError(
                        "Mouse control not permitted".to_string()
                    ));
                }
            }
            RemoteControlEvent::Keyboard(_) => {
                if !self.permissions.allows_keyboard() {
                    return Err(ClientError::InputError(
                        "Keyboard control not permitted".to_string()
                    ));
                }
            }
        }

        // Handle event
        self.session.handle_event(event).await
    }

    /// Get permission level
    pub fn permissions(&self) -> PermissionLevel {
        self.permissions
    }

    /// Update permissions
    pub fn set_permissions(&mut self, permissions: PermissionLevel) {
        self.permissions = permissions;
        tracing::info!(
            "Session {} permissions updated to {:?}",
            self.session.session_id(),
            permissions
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_levels() {
        assert!(PermissionLevel::FullControl.allows_mouse());
        assert!(PermissionLevel::FullControl.allows_keyboard());
        assert!(PermissionLevel::MouseOnly.allows_mouse());
        assert!(!PermissionLevel::MouseOnly.allows_keyboard());
        assert!(!PermissionLevel::ViewOnly.allows_mouse());
        assert!(!PermissionLevel::ViewOnly.allows_keyboard());
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = RemoteControlManager::new();
        assert_eq!(manager.session_count().await, 0);
    }
}
