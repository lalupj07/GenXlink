use crate::security::{SecurityManager, EncryptedMessage, DevicePublicKey};
use crate::ClientError;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use bytes::Bytes;

/// WebRTC security manager for E2E encryption
pub struct WebRTCSecurityManager {
    security: Arc<Mutex<SecurityManager>>,
    enabled: bool,
}

impl WebRTCSecurityManager {
    /// Create a new WebRTC security manager
    pub fn new() -> Result<Self> {
        let security = SecurityManager::new()
            .context("Failed to initialize security manager")?;
        
        Ok(Self {
            security: Arc::new(Mutex::new(security)),
            enabled: true,
        })
    }
    
    /// Enable/disable E2E encryption
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        tracing::info!("E2E encryption {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Check if E2E encryption is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Initialize encryption for a new session
    pub async fn initialize_session(&self) -> Result<String> {
        if !self.enabled {
            return Ok("encryption_disabled".to_string());
        }
        
        let mut security = self.security.lock().await;
        let fingerprint = security.generate_encryption_key()
            .context("Failed to generate encryption key")?;
        
        tracing::info!("E2E encryption initialized with key fingerprint: {}", fingerprint);
        Ok(fingerprint)
    }
    
    /// Get our public key for sharing with remote device
    pub async fn get_public_key(&self) -> Result<DevicePublicKey> {
        if !self.enabled {
            return Err(anyhow::anyhow!("E2E encryption is disabled"));
        }
        
        let security = self.security.lock().await;
        Ok(security.get_public_key())
    }
    
    /// Add remote device's public key
    pub async fn add_remote_key(&self, device_key: DevicePublicKey) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut security = self.security.lock().await;
        security.add_device_key(device_key)
            .context("Failed to add remote device key")?;
        Ok(())
    }
    
    /// Establish secure session with remote device
    pub async fn establish_session(&self, device_id: &str) -> Result<String> {
        if !self.enabled {
            return Ok("encryption_disabled".to_string());
        }
        
        let mut security = self.security.lock().await;
        let fingerprint = security.establish_session(device_id)
            .context("Failed to establish secure session")?;
        
        tracing::info!("Secure session established with device: {}", device_id);
        Ok(fingerprint)
    }
    
    /// Encrypt WebRTC data channel message
    pub async fn encrypt_message(&self, device_id: &str, data: &[u8]) -> Result<Bytes> {
        if !self.enabled {
            return Ok(Bytes::copy_from_slice(data));
        }
        
        let security = self.security.lock().await;
        let encrypted = security.encrypt_message(device_id, data)
            .context("Failed to encrypt message")?;
        
        // Serialize encrypted message
        let serialized = serde_json::to_vec(&encrypted)
            .context("Failed to serialize encrypted message")?;
        
        Ok(Bytes::from(serialized))
    }
    
    /// Decrypt WebRTC data channel message
    pub async fn decrypt_message(&self, device_id: &str, encrypted_data: &[u8]) -> Result<Bytes> {
        if !self.enabled {
            return Ok(Bytes::copy_from_slice(encrypted_data));
        }
        
        // Deserialize encrypted message
        let encrypted: EncryptedMessage = serde_json::from_slice(encrypted_data)
            .context("Failed to deserialize encrypted message")?;
        
        let security = self.security.lock().await;
        let decrypted = security.decrypt_message(device_id, &encrypted)
            .context("Failed to decrypt message")?;
        
        Ok(Bytes::from(decrypted))
    }
    
    /// Encrypt WebRTC media data (video/audio frames)
    pub async fn encrypt_media(&self, device_id: &str, data: &[u8]) -> Result<Bytes> {
        if !self.enabled {
            return Ok(Bytes::copy_from_slice(data));
        }
        
        let security = self.security.lock().await;
        let encrypted = security.encrypt_media_data(device_id, data)
            .context("Failed to encrypt media data")?;
        
        Ok(Bytes::from(encrypted))
    }
    
    /// Decrypt WebRTC media data (video/audio frames)
    pub async fn decrypt_media(&self, device_id: &str, encrypted_data: &[u8]) -> Result<Bytes> {
        if !self.enabled {
            return Ok(Bytes::copy_from_slice(encrypted_data));
        }
        
        let security = self.security.lock().await;
        let decrypted = security.decrypt_media_data(device_id, encrypted_data)
            .context("Failed to decrypt media data")?;
        
        Ok(Bytes::from(decrypted))
    }
    
    /// Rotate session keys for security
    pub async fn rotate_keys(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut security = self.security.lock().await;
        security.rotate_session_keys()
            .context("Failed to rotate session keys")?;
        
        tracing::info!("Session keys rotated for security");
        Ok(())
    }
    
    /// Get security status
    pub async fn get_security_status(&self) -> SecurityStatus {
        let security = self.security.lock().await;
        let status = security.get_security_status();
        
        SecurityStatus {
            e2e_encryption_enabled: self.enabled,
            has_encryption_key: status.has_encryption_key,
            active_sessions: status.active_sessions,
            known_devices: status.known_devices,
            key_fingerprint: status.key_fingerprint,
        }
    }
    
    /// Clear all session keys
    pub async fn clear_sessions(&self) -> Result<()> {
        let mut security = self.security.lock().await;
        security.session_keys.clear();
        tracing::info!("All secure sessions cleared");
        Ok(())
    }
}

impl Default for WebRTCSecurityManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Security status for WebRTC
#[derive(Debug, Clone)]
pub struct SecurityStatus {
    pub e2e_encryption_enabled: bool,
    pub has_encryption_key: bool,
    pub active_sessions: usize,
    pub known_devices: usize,
    pub key_fingerprint: String,
}

impl SecurityStatus {
    /// Check if connection is fully secure
    pub fn is_fully_secure(&self) -> bool {
        self.e2e_encryption_enabled && self.has_encryption_key && self.active_sessions > 0
    }
    
    /// Get security level description
    pub fn security_level(&self) -> &'static str {
        if !self.e2e_encryption_enabled {
            "No Encryption"
        } else if !self.has_encryption_key {
            "Encryption Initializing"
        } else if self.active_sessions == 0 {
            "Encryption Ready"
        } else {
            "End-to-End Encrypted"
        }
    }
    
    /// Get security icon
    pub fn security_icon(&self) -> &'static str {
        if self.is_fully_secure() {
            "🔒"
        } else if self.e2e_encryption_enabled {
            "🔐"
        } else {
            "🔓"
        }
    }
}

/// WebRTC security configuration
#[derive(Debug, Clone)]
pub struct WebRTCSecurityConfig {
    pub e2e_encryption_enabled: bool,
    pub key_rotation_interval_secs: u64,
    pub max_session_duration_secs: u64,
    pub require_device_authentication: bool,
}

impl Default for WebRTCSecurityConfig {
    fn default() -> Self {
        Self {
            e2e_encryption_enabled: true,
            key_rotation_interval_secs: 3600, // 1 hour
            max_session_duration_secs: 86400, // 24 hours
            require_device_authentication: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_webrtc_security_creation() {
        let manager = WebRTCSecurityManager::new();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert!(manager.is_enabled());
    }
    
    #[tokio::test]
    async fn test_encryption_toggle() {
        let mut manager = WebRTCSecurityManager::new().unwrap();
        
        manager.set_enabled(false);
        assert!(!manager.is_enabled());
        
        manager.set_enabled(true);
        assert!(manager.is_enabled());
    }
    
    #[tokio::test]
    async fn test_session_initialization() {
        let manager = WebRTCSecurityManager::new().unwrap();
        let fingerprint = manager.initialize_session().await.unwrap();
        assert!(!fingerprint.is_empty());
        assert_ne!(fingerprint, "encryption_disabled");
    }
    
    #[tokio::test]
    async fn test_message_encryption() {
        let manager = WebRTCSecurityManager::new().unwrap();
        
        // Initialize session
        let fingerprint = manager.initialize_session().await.unwrap();
        assert_ne!(fingerprint, "encryption_disabled");
        
        // Get public key and add as remote (self-test)
        let public_key = manager.get_public_key().await.unwrap();
        manager.add_remote_key(public_key.clone()).await.unwrap();
        
        // Establish session
        manager.establish_session(&public_key.device_id).await.unwrap();
        
        // Test encryption/decryption
        let original_data = b"Hello, secure WebRTC!";
        let encrypted = manager.encrypt_message(&public_key.device_id, original_data).await.unwrap();
        let decrypted = manager.decrypt_message(&public_key.device_id, &encrypted).await.unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted.to_vec());
    }
    
    #[tokio::test]
    async fn test_media_encryption() {
        let manager = WebRTCSecurityManager::new().unwrap();
        
        // Initialize session
        let fingerprint = manager.initialize_session().await.unwrap();
        assert_ne!(fingerprint, "encryption_disabled");
        
        // Get public key and add as remote (self-test)
        let public_key = manager.get_public_key().await.unwrap();
        manager.add_remote_key(public_key.clone()).await.unwrap();
        
        // Establish session
        manager.establish_session(&public_key.device_id).await.unwrap();
        
        // Test media encryption/decryption
        let media_data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let encrypted = manager.encrypt_media(&public_key.device_id, &media_data).await.unwrap();
        let decrypted = manager.decrypt_media(&public_key.device_id, &encrypted).await.unwrap();
        
        assert_eq!(media_data, decrypted.to_vec());
    }
    
    #[tokio::test]
    async fn test_security_status() {
        let manager = WebRTCSecurityManager::new().unwrap();
        let status = manager.get_security_status().await;
        
        assert!(status.e2e_encryption_enabled);
        assert_eq!(status.security_level(), "Encryption Initializing");
        assert_eq!(status.security_icon(), "🔐");
        assert!(!status.is_fully_secure());
    }
    
    #[test]
    fn test_security_status_methods() {
        let status = SecurityStatus {
            e2e_encryption_enabled: true,
            has_encryption_key: true,
            active_sessions: 1,
            known_devices: 1,
            key_fingerprint: "test123".to_string(),
        };
        
        assert!(status.is_fully_secure());
        assert_eq!(status.security_level(), "End-to-End Encrypted");
        assert_eq!(status.security_icon(), "🔒");
    }
}
