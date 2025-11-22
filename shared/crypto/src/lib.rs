use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::{Aead, OsRng}};
use rand::RngCore;
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::{Sha256, Digest};
use thiserror::Error;

pub mod encryption;
pub mod signature;

pub use encryption::*;
pub use signature::*;

/// Crypto error types
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    #[error("Signature error: {0}")]
    SignatureError(String),
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
}

/// Generate a random 256-bit key
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Generate a random nonce for AES-GCM
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Hash data using SHA-256
pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_aes256(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    
    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_aes256(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.len() < 12 {
        return Err(CryptoError::DecryptionFailed("Invalid ciphertext length".to_string()));
    }
    
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let encrypted_data = &ciphertext[12..];
    
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

/// Generate RSA key pair
pub fn generate_rsa_keypair(bits: usize) -> Result<(RsaPrivateKey, RsaPublicKey), CryptoError> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, bits)
        .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?;
    let public_key = RsaPublicKey::from(&private_key);
    
    Ok((private_key, public_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption() {
        let key = generate_key();
        let plaintext = b"Hello, GenXLink!";
        
        let ciphertext = encrypt_aes256(&key, plaintext).unwrap();
        let decrypted = decrypt_aes256(&key, &ciphertext).unwrap();
        
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash = hash_sha256(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_rsa_keypair_generation() {
        let result = generate_rsa_keypair(2048);
        assert!(result.is_ok());
    }
}
