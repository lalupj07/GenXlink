use crate::ClientError;
use rand::Rng;
use std::time::{Duration, Instant};

/// Session password manager
pub struct SessionPasswordManager {
    current_password: Option<SessionPassword>,
    password_length: usize,
    timeout_duration: Duration,
}

impl SessionPasswordManager {
    /// Create a new session password manager
    pub fn new() -> Self {
        Self {
            current_password: None,
            password_length: 6,
            timeout_duration: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Generate a new session password
    pub fn generate_password(&mut self) -> String {
        let password = Self::generate_random_password(self.password_length);
        self.current_password = Some(SessionPassword {
            password: password.clone(),
            created_at: Instant::now(),
            expires_at: Instant::now() + self.timeout_duration,
            attempts: 0,
        });
        password
    }

    /// Verify a password
    pub fn verify_password(&mut self, password: &str) -> Result<bool, ClientError> {
        if let Some(ref mut session_pwd) = self.current_password {
            // Check if expired
            if Instant::now() > session_pwd.expires_at {
                self.current_password = None;
                return Err(ClientError::AuthenticationError("Password expired".to_string()));
            }

            // Increment attempts
            session_pwd.attempts += 1;

            // Check max attempts
            if session_pwd.attempts > 3 {
                self.current_password = None;
                return Err(ClientError::AuthenticationError("Too many attempts".to_string()));
            }

            // Verify password
            Ok(session_pwd.password == password)
        } else {
            Err(ClientError::AuthenticationError("No active session".to_string()))
        }
    }

    /// Check if password is still valid
    pub fn is_valid(&self) -> bool {
        if let Some(ref session_pwd) = self.current_password {
            Instant::now() <= session_pwd.expires_at && session_pwd.attempts <= 3
        } else {
            false
        }
    }

    /// Get remaining time
    pub fn remaining_time(&self) -> Option<Duration> {
        if let Some(ref session_pwd) = self.current_password {
            let now = Instant::now();
            if now < session_pwd.expires_at {
                Some(session_pwd.expires_at - now)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Clear current password
    pub fn clear(&mut self) {
        self.current_password = None;
    }

    /// Set password length
    pub fn set_password_length(&mut self, length: usize) {
        self.password_length = length.max(4).min(12);
    }

    /// Set timeout duration
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Generate random password
    fn generate_random_password(length: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Exclude similar chars
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect()
    }
}

impl Default for SessionPasswordManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session password information
struct SessionPassword {
    password: String,
    created_at: Instant,
    expires_at: Instant,
    attempts: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let mut manager = SessionPasswordManager::new();
        let password = manager.generate_password();
        assert_eq!(password.len(), 6);
        assert!(password.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_password_verification() {
        let mut manager = SessionPasswordManager::new();
        let password = manager.generate_password();
        
        assert!(manager.verify_password(&password).unwrap());
        assert!(!manager.verify_password("WRONG").unwrap());
    }

    #[test]
    fn test_password_validity() {
        let mut manager = SessionPasswordManager::new();
        assert!(!manager.is_valid());
        
        manager.generate_password();
        assert!(manager.is_valid());
        
        manager.clear();
        assert!(!manager.is_valid());
    }
}
