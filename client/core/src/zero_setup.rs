use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Zero-setup access method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessMethod {
    BrowserLink,
    QrCode,
    TemporaryPin,
}

impl AccessMethod {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BrowserLink => "Browser Link",
            Self::QrCode => "QR Code",
            Self::TemporaryPin => "Temporary PIN",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::BrowserLink => "🔗",
            Self::QrCode => "📱",
            Self::TemporaryPin => "🔢",
        }
    }
}

/// Temporary access session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporarySession {
    pub session_id: String,
    pub access_code: String,
    pub pin: Option<String>,
    pub browser_link: String,
    pub qr_code_data: String,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub max_connections: u32,
    pub current_connections: u32,
    pub is_active: bool,
}

impl TemporarySession {
    /// Create a new temporary session
    pub fn new(duration_minutes: u32) -> Self {
        let session_id = Self::generate_session_id();
        let access_code = Self::generate_access_code();
        let pin = Some(Self::generate_pin());
        let created_at = SystemTime::now();
        let expires_at = created_at + Duration::from_secs(duration_minutes as u64 * 60);
        
        let browser_link = format!("https://genxlink.com/connect/{}", access_code);
        let qr_code_data = format!("genxlink://connect/{}", access_code);
        
        Self {
            session_id,
            access_code,
            pin,
            browser_link,
            qr_code_data,
            created_at,
            expires_at,
            max_connections: 1,
            current_connections: 0,
            is_active: true,
        }
    }
    
    /// Generate a unique session ID
    fn generate_session_id() -> String {
        use std::time::UNIX_EPOCH;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("sess_{}", timestamp)
    }
    
    /// Generate a random access code (9 digits)
    fn generate_access_code() -> String {
        use std::time::UNIX_EPOCH;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("{:09}", timestamp % 1_000_000_000)
    }
    
    /// Generate a random PIN (6 digits)
    fn generate_pin() -> String {
        use std::time::UNIX_EPOCH;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();
        format!("{:06}", timestamp % 1_000_000)
    }
    
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
    
    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> u64 {
        if self.is_expired() {
            return 0;
        }
        
        self.expires_at
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    }
    
    /// Format remaining time as string
    pub fn remaining_time_string(&self) -> String {
        let seconds = self.remaining_seconds();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        
        if hours > 0 {
            format!("{}h {}m", hours, minutes % 60)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds % 60)
        } else {
            format!("{}s", seconds)
        }
    }
    
    /// Verify PIN
    pub fn verify_pin(&self, pin: &str) -> bool {
        if let Some(ref session_pin) = self.pin {
            session_pin == pin
        } else {
            true // No PIN required
        }
    }
    
    /// Can accept new connection
    pub fn can_accept_connection(&self) -> bool {
        self.is_active 
            && !self.is_expired() 
            && self.current_connections < self.max_connections
    }
}

/// Zero-setup manager
pub struct ZeroSetupManager {
    active_sessions: Vec<TemporarySession>,
}

impl Default for ZeroSetupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroSetupManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Vec::new(),
        }
    }
    
    /// Create a new temporary session
    pub fn create_session(&mut self, duration_minutes: u32) -> TemporarySession {
        let session = TemporarySession::new(duration_minutes);
        self.active_sessions.push(session.clone());
        session
    }
    
    /// Get session by access code
    pub fn get_session(&self, access_code: &str) -> Option<&TemporarySession> {
        self.active_sessions
            .iter()
            .find(|s| s.access_code == access_code && !s.is_expired())
    }
    
    /// Get session by access code (mutable)
    pub fn get_session_mut(&mut self, access_code: &str) -> Option<&mut TemporarySession> {
        self.active_sessions
            .iter_mut()
            .find(|s| s.access_code == access_code && !s.is_expired())
    }
    
    /// Verify access code and PIN
    pub fn verify_access(&self, access_code: &str, pin: Option<&str>) -> bool {
        if let Some(session) = self.get_session(access_code) {
            if let Some(pin_str) = pin {
                session.verify_pin(pin_str)
            } else {
                session.pin.is_none()
            }
        } else {
            false
        }
    }
    
    /// Accept connection
    pub fn accept_connection(&mut self, access_code: &str) -> Result<(), String> {
        if let Some(session) = self.get_session_mut(access_code) {
            if session.can_accept_connection() {
                session.current_connections += 1;
                Ok(())
            } else {
                Err("Session cannot accept more connections".to_string())
            }
        } else {
            Err("Session not found or expired".to_string())
        }
    }
    
    /// End connection
    pub fn end_connection(&mut self, access_code: &str) {
        if let Some(session) = self.get_session_mut(access_code) {
            if session.current_connections > 0 {
                session.current_connections -= 1;
            }
        }
    }
    
    /// Revoke session
    pub fn revoke_session(&mut self, access_code: &str) {
        if let Some(session) = self.get_session_mut(access_code) {
            session.is_active = false;
        }
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) {
        self.active_sessions.retain(|s| !s.is_expired());
    }
    
    /// Get all active sessions
    pub fn active_sessions(&self) -> Vec<&TemporarySession> {
        self.active_sessions
            .iter()
            .filter(|s| !s.is_expired() && s.is_active)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_temporary_session() {
        let session = TemporarySession::new(30);
        assert!(!session.is_expired());
        assert!(session.remaining_seconds() > 0);
        assert!(session.can_accept_connection());
    }
    
    #[test]
    fn test_pin_verification() {
        let session = TemporarySession::new(30);
        let pin = session.pin.clone().unwrap();
        assert!(session.verify_pin(&pin));
        assert!(!session.verify_pin("000000"));
    }
    
    #[test]
    fn test_zero_setup_manager() {
        let mut manager = ZeroSetupManager::new();
        let session = manager.create_session(30);
        
        assert!(manager.verify_access(&session.access_code, session.pin.as_deref()));
        assert!(manager.accept_connection(&session.access_code).is_ok());
    }
}
