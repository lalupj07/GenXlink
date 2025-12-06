use anyhow::{Context, Result};
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::RwLock;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit, OsRng, AeadCore}};
use aes_gcm::aead::generic_array::GenericArray;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

/// Security manager for end-to-end encryption
pub struct SecurityManager {
    encryption_key: Option<Key<Aes256Gcm>>,
    pub device_keys: HashMap<String, DevicePublicKey>,
    pub session_keys: HashMap<String, SessionKey>,
    key_exchange: KeyExchangeManager,
}

/// Device public key for key exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePublicKey {
    pub device_id: String,
    pub public_key: Vec<u8>,
    pub fingerprint: String,
    pub created_at: std::time::SystemTime,
}

/// Session key for encrypted communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    pub device_id: String,
    pub key_bytes: Vec<u8>, // Raw key bytes for serialization
    pub created_at: std::time::SystemTime,
    pub expires_at: std::time::SystemTime,
}

/// Key exchange manager
pub struct KeyExchangeManager {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    fingerprint: String,
}

/// Encrypted message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub auth_tag: Vec<u8>,
    pub sender_id: String,
    pub timestamp: u64,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Result<Self> {
        let key_exchange = KeyExchangeManager::new()
            .context("Failed to initialize key exchange")?;
        
        Ok(Self {
            encryption_key: None,
            device_keys: HashMap::new(),
            session_keys: HashMap::new(),
            key_exchange,
        })
    }
    
    /// Generate a new encryption key
    pub fn generate_encryption_key(&mut self) -> Result<String> {
        let key_bytes = rand::thread_rng().gen::<[u8; 32]>();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        self.encryption_key = Some(key.clone());
        
        // Return key fingerprint
        let fingerprint = self.compute_key_fingerprint(&key_bytes);
        Ok(fingerprint)
    }
    
    /// Get our public key for sharing
    pub fn get_public_key(&self) -> DevicePublicKey {
        DevicePublicKey {
            device_id: "local".to_string(), // Would be actual device ID
            public_key: self.key_exchange.public_key.clone(),
            fingerprint: self.key_exchange.fingerprint.clone(),
            created_at: std::time::SystemTime::now(),
        }
    }
    
    /// Add a device's public key
    pub fn add_device_key(&mut self, device_key: DevicePublicKey) -> Result<()> {
        // Verify the key fingerprint
        let computed_fingerprint = self.compute_key_fingerprint(&device_key.public_key);
        if computed_fingerprint != device_key.fingerprint {
            return Err(anyhow::anyhow!("Key fingerprint verification failed"));
        }
        
        let device_id = device_key.device_id.clone();
        self.device_keys.insert(device_id.clone(), device_key);
        tracing::info!("Added public key for device: {}", device_id);
        Ok(())
    }
    
    /// Establish session key with a device
    pub fn establish_session(&mut self, device_id: &str) -> Result<String> {
        let device_key = self.device_keys.get(device_id)
            .ok_or_else(|| anyhow::anyhow!("No public key found for device: {}", device_id))?;
        
        // Perform ECDH key exchange (simplified)
        let shared_secret = self.perform_key_exchange(&device_key.public_key)?;
        
        // Derive session key from shared secret
        let session_key_bytes = self.derive_session_key(&shared_secret, device_id);
        
        let session = SessionKey {
            device_id: device_id.to_string(),
            key_bytes: session_key_bytes.clone(),
            created_at: std::time::SystemTime::now(),
            expires_at: std::time::SystemTime::now() + std::time::Duration::from_secs(3600), // 1 hour
        };
        
        self.session_keys.insert(device_id.to_string(), session);
        
        let fingerprint = self.compute_key_fingerprint(&session_key_bytes);
        tracing::info!("Established encrypted session with device: {}", device_id);
        Ok(fingerprint)
    }
    
    /// Encrypt a message for a specific device
    pub fn encrypt_message(&self, device_id: &str, plaintext: &[u8]) -> Result<EncryptedMessage> {
        let session_key = self.session_keys.get(device_id)
            .ok_or_else(|| anyhow::anyhow!("No session key found for device: {}", device_id))?;
        
        // Check if session key has expired
        if std::time::SystemTime::now() > session_key.expires_at {
            return Err(anyhow::anyhow!("Session key expired for device: {}", device_id));
        }
        
        // Convert key_bytes back to Key<Aes256Gcm>
        let key = Key::<Aes256Gcm>::from_slice(&session_key.key_bytes);
        
        // Generate random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt using AES-256-GCM
        let cipher = Aes256Gcm::new(&key);
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .context("Failed to encrypt message")?;
        
        // Split ciphertext and auth tag (simplified - in real implementation would handle properly)
        let auth_tag = ciphertext[ciphertext.len() - 16..].to_vec();
        let ciphertext = ciphertext[..ciphertext.len() - 16].to_vec();
        
        Ok(EncryptedMessage {
            ciphertext,
            nonce: nonce.to_vec(),
            auth_tag,
            sender_id: "local".to_string(), // Would be actual device ID
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// Decrypt a message from a specific device
    pub fn decrypt_message(&self, device_id: &str, encrypted: &EncryptedMessage) -> Result<Vec<u8>> {
        let session_key = self.session_keys.get(device_id)
            .ok_or_else(|| anyhow::anyhow!("No session key found for device: {}", device_id))?;
        
        // Check if session key has expired
        if std::time::SystemTime::now() > session_key.expires_at {
            return Err(anyhow::anyhow!("Session key expired for device: {}", device_id));
        }
        
        // Convert key_bytes back to Key<Aes256Gcm>
        let key = Key::<Aes256Gcm>::from_slice(&session_key.key_bytes);
        
        // Reconstruct ciphertext with auth tag
        let mut full_ciphertext = encrypted.ciphertext.clone();
        full_ciphertext.extend_from_slice(&encrypted.auth_tag);
        
        // Decrypt using AES-256-GCM
        let nonce = Nonce::from_slice(&encrypted.nonce);
        let cipher = Aes256Gcm::new(&key);
        let plaintext = cipher.decrypt(nonce, full_ciphertext.as_ref())
            .context("Failed to decrypt message - authentication failed")?;
        
        Ok(plaintext)
    }
    
    /// Encrypt WebRTC media data
    pub fn encrypt_media_data(&self, device_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // For media, we use a more efficient streaming encryption
        let session_key = self.session_keys.get(device_id)
            .ok_or_else(|| anyhow::anyhow!("No session key found for device: {}", device_id))?;
        
        // Convert key_bytes back to Key<Aes256Gcm>
        let key = Key::<Aes256Gcm>::from_slice(&session_key.key_bytes);
        
        // Generate nonce from data hash for consistency
        let nonce_bytes = self.generate_media_nonce_from_data(data);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let cipher = Aes256Gcm::new(&key);
        let encrypted = cipher.encrypt(nonce, data)
            .context("Failed to encrypt media data")?;
        
        // Prepend nonce to encrypted data for decryption
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&encrypted);
        
        Ok(result)
    }
    
    /// Decrypt WebRTC media data
    pub fn decrypt_media_data(&self, device_id: &str, encrypted_data_with_nonce: &[u8]) -> Result<Vec<u8>> {
        let session_key = self.session_keys.get(device_id)
            .ok_or_else(|| anyhow::anyhow!("No session key found for device: {}", device_id))?;
        
        if encrypted_data_with_nonce.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data - too short"));
        }
        
        // Extract nonce and encrypted data
        let nonce_bytes = &encrypted_data_with_nonce[..12];
        let encrypted_data = &encrypted_data_with_nonce[12..];
        
        // Convert key_bytes back to Key<Aes256Gcm>
        let key = Key::<Aes256Gcm>::from_slice(&session_key.key_bytes);
        
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = Aes256Gcm::new(&key);
        let decrypted = cipher.decrypt(nonce, encrypted_data)
            .context("Failed to decrypt media data")?;
        
        Ok(decrypted)
    }
    
    /// Rotate session keys for security
    pub fn rotate_session_keys(&mut self) -> Result<()> {
        let expired_devices: Vec<String> = self.session_keys
            .iter()
            .filter(|(_, session)| std::time::SystemTime::now() > session.expires_at)
            .map(|(device_id, _)| device_id.clone())
            .collect();
        
        for device_id in expired_devices {
            self.session_keys.remove(&device_id);
            tracing::info!("Removed expired session key for device: {}", device_id);
        }
        
        Ok(())
    }
    
    /// Get security status
    pub fn get_security_status(&self) -> SecurityStatus {
        SecurityStatus {
            has_encryption_key: self.encryption_key.is_some(),
            active_sessions: self.session_keys.len(),
            known_devices: self.device_keys.len(),
            key_fingerprint: self.key_exchange.fingerprint.clone(),
        }
    }
    
    // Private helper methods
    
    /// Compute SHA-256 fingerprint of a key
    fn compute_key_fingerprint(&self, key_bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key_bytes);
        let hash = hasher.finalize();
        general_purpose::STANDARD.encode(&hash[..8]) // Short fingerprint
    }
    
    /// Perform ECDH key exchange (simplified implementation)
    fn perform_key_exchange(&self, public_key: &[u8]) -> Result<Vec<u8>> {
        // In a real implementation, this would use proper ECDH
        // For now, simulate with XOR (NOT SECURE - for demonstration only)
        let mut shared_secret = vec![0u8; public_key.len().min(self.key_exchange.private_key.len())];
        for (i, &pk_byte) in public_key.iter().enumerate() {
            if i < self.key_exchange.private_key.len() {
                shared_secret[i] = pk_byte ^ self.key_exchange.private_key[i];
            }
        }
        Ok(shared_secret)
    }
    
    /// Derive session key from shared secret
    fn derive_session_key(&self, shared_secret: &[u8], device_id: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        hasher.update(device_id.as_bytes());
        hasher.update(b"GenXLinkSessionKey");
        let hash = hasher.finalize();
        hash[..32].to_vec() // Use first 32 bytes as AES-256 key
    }
    
    /// Generate nonce for media encryption
    fn generate_media_nonce(&self, data_len: usize) -> [u8; 12] {
        // In a real implementation, this would use sequence numbers
        // For now, use a simple deterministic approach
        let mut nonce = [0u8; 12];
        nonce[0..4].copy_from_slice(&(data_len as u32).to_le_bytes());
        // Fill rest with random data
        for i in 4..12 {
            nonce[i] = rand::thread_rng().gen();
        }
        nonce
    }
    
    /// Generate nonce from data hash for consistent encryption/decryption
    fn generate_media_nonce_from_data(&self, data: &[u8]) -> [u8; 12] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(b"GenXLinkMediaNonce");
        let hash = hasher.finalize();
        
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&hash[..12]);
        nonce
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl KeyExchangeManager {
    /// Create a new key exchange manager
    pub fn new() -> Result<Self> {
        // Generate key pair (simplified - in real implementation would use proper crypto)
        let private_key = rand::thread_rng().gen::<[u8; 32]>();
        let public_key = Self::derive_public_key(&private_key);
        
        let mut hasher = Sha256::new();
        hasher.update(&public_key);
        let hash = hasher.finalize();
        let fingerprint = general_purpose::STANDARD.encode(&hash[..8]);
        
        Ok(Self {
            private_key: private_key.to_vec(),
            public_key,
            fingerprint,
        })
    }
    
    /// Derive public key from private key (simplified)
    fn derive_public_key(private_key: &[u8]) -> Vec<u8> {
        // In a real implementation, this would use proper ECC
        // For now, simulate with a simple transformation
        let mut public_key = vec![0u8; 32];
        for (i, &pk_byte) in private_key.iter().enumerate() {
            public_key[i] = pk_byte.wrapping_mul(2).wrapping_add(1);
        }
        public_key
    }
}

/// Security status information
#[derive(Debug, Clone)]
pub struct SecurityStatus {
    pub has_encryption_key: bool,
    pub active_sessions: usize,
    pub known_devices: usize,
    pub key_fingerprint: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_encryption_key_generation() {
        let mut manager = SecurityManager::new().unwrap();
        let fingerprint = manager.generate_encryption_key().unwrap();
        assert!(!fingerprint.is_empty());
        assert!(manager.encryption_key.is_some());
    }
    
    #[test]
    fn test_device_key_management() {
        let mut manager = SecurityManager::new().unwrap();
        let public_key = manager.get_public_key();
        
        assert!(manager.add_device_key(public_key.clone()).is_ok());
        assert_eq!(manager.device_keys.len(), 1);
    }
    
    #[tokio::test]
    async fn test_message_encryption() {
        let mut manager = SecurityManager::new().unwrap();
        
        // Add device key
        let mut device_key = manager.get_public_key();
        device_key.device_id = "test_device".to_string();
        manager.add_device_key(device_key).unwrap();
        
        // Establish session
        manager.establish_session("test_device").unwrap();
        
        // Encrypt and decrypt message
        let plaintext = b"Hello, secure world!";
        let encrypted = manager.encrypt_message("test_device", plaintext).unwrap();
        let decrypted = manager.decrypt_message("test_device", &encrypted).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_media_encryption() {
        let mut manager = SecurityManager::new().unwrap();
        
        // Add device key
        let mut device_key = manager.get_public_key();
        device_key.device_id = "test_device".to_string();
        manager.add_device_key(device_key).unwrap();
        
        // Establish session
        manager.establish_session("test_device").unwrap();
        
        // Encrypt and decrypt media data
        let media_data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let encrypted = manager.encrypt_media_data("test_device", &media_data).unwrap();
        let decrypted = manager.decrypt_media_data("test_device", &encrypted).unwrap();
        
        assert_eq!(media_data, decrypted);
    }
}
