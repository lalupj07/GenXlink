use crate::{CryptoError, encrypt_aes256, decrypt_aes256};

/// Encrypted message container
pub struct EncryptedMessage {
    pub ciphertext: Vec<u8>,
}

impl EncryptedMessage {
    /// Create a new encrypted message
    pub fn new(key: &[u8; 32], plaintext: &[u8]) -> Result<Self, CryptoError> {
        let ciphertext = encrypt_aes256(key, plaintext)?;
        Ok(Self { ciphertext })
    }
    
    /// Decrypt the message
    pub fn decrypt(&self, key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        decrypt_aes256(key, &self.ciphertext)
    }
    
    /// Get the ciphertext bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.ciphertext
    }
    
    /// Create from ciphertext bytes
    pub fn from_bytes(ciphertext: Vec<u8>) -> Self {
        Self { ciphertext }
    }
}

/// Session key manager for secure communication
pub struct SessionKeyManager {
    local_key: [u8; 32],
    remote_key: Option<[u8; 32]>,
}

impl SessionKeyManager {
    /// Create a new session key manager
    pub fn new() -> Self {
        Self {
            local_key: crate::generate_key(),
            remote_key: None,
        }
    }
    
    /// Get the local public key
    pub fn get_local_key(&self) -> &[u8; 32] {
        &self.local_key
    }
    
    /// Set the remote public key
    pub fn set_remote_key(&mut self, key: [u8; 32]) {
        self.remote_key = Some(key);
    }
    
    /// Encrypt data for sending
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        encrypt_aes256(&self.local_key, data)
    }
    
    /// Decrypt received data
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match &self.remote_key {
            Some(key) => decrypt_aes256(key, data),
            None => Err(CryptoError::InvalidKey("Remote key not set".to_string())),
        }
    }
}

impl Default for SessionKeyManager {
    fn default() -> Self {
        Self::new()
    }
}
