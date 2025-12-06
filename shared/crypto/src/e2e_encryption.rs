use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKeyPair {
    pub key_id: Uuid,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    pub session_id: Uuid,
    pub key_id: Uuid,
    pub shared_secret: Vec<u8>,
    pub salt: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub message_id: Uuid,
    pub session_id: Uuid,
    pub key_id: Uuid,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

pub struct EndToEndEncryption {
    key_pairs: Arc<RwLock<HashMap<Uuid, EncryptionKeyPair>>>,
    session_keys: Arc<RwLock<HashMap<Uuid, SessionKey>>>,
    current_key_id: Arc<RwLock<Option<Uuid>>>,
    key_rotation_interval: std::time::Duration,
    session_key_ttl: std::time::Duration,
}

impl EndToEndEncryption {
    pub fn new(
        key_rotation_interval: std::time::Duration,
        session_key_ttl: std::time::Duration,
    ) -> Result<Self> {
        Ok(Self {
            key_pairs: Arc::new(RwLock::new(HashMap::new())),
            session_keys: Arc::new(RwLock::new(HashMap::new())),
            current_key_id: Arc::new(RwLock::new(None)),
            key_rotation_interval,
            session_key_ttl,
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        // Generate initial key pair
        let key_pair = self.generate_key_pair().await?;
        
        let mut key_pairs = self.key_pairs.write().await;
        let mut current_key_id = self.current_key_id.write().await;
        
        key_pairs.insert(key_pair.key_id, key_pair.clone());
        *current_key_id = Some(key_pair.key_id);
        
        info!("Initialized E2E encryption with key pair: {}", key_pair.key_id);
        
        // Start key rotation task
        self.start_key_rotation().await;
        
        Ok(())
    }

    async fn generate_key_pair(&self) -> Result<EncryptionKeyPair> {
        let key_id = Uuid::new_v4();
        let created_at = chrono::Utc::now();
        let expires_at = created_at + chrono::Duration::from_std(self.key_rotation_interval)
            .map_err(|e| anyhow!("Invalid duration: {}", e))?;

        // Generate X25519 key pair for key exchange
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);
        
        // Generate Ed25519 key pair for signing
        let ed_key_pair = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
            .map_err(|e| anyhow!("Failed to generate Ed25519 key pair: {}", e))?;
        
        let key_pair = EncryptionKeyPair {
            key_id,
            public_key: public_key.as_bytes().to_vec(),
            private_key: secret.to_bytes().to_vec(),
            created_at,
            expires_at,
        };

        info!("Generated new key pair: {}", key_id);
        Ok(key_pair)
    }

    pub async fn get_public_key(&self, key_id: Option<Uuid>) -> Result<Vec<u8>> {
        let key_id = key_id.or_else(|| {
            // Get current key ID if none specified
            futures::executor::block_on(self.current_key_id.read()).clone()
        });

        if let Some(key_id) = key_id {
            let key_pairs = self.key_pairs.read().await;
            if let Some(key_pair) = key_pairs.get(&key_id) {
                Ok(key_pair.public_key.clone())
            } else {
                Err(anyhow!("Key pair not found: {}", key_id))
            }
        } else {
            Err(anyhow!("No current key pair available"))
        }
    }

    pub async fn create_session_key(&self, session_id: Uuid, peer_public_key: &[u8]) -> Result<SessionKey> {
        // Get current key pair
        let current_key_id = self.current_key_id.read().await
            .ok_or_else(|| anyhow!("No current key pair available"))?;
        
        let key_pairs = self.key_pairs.read().await;
        let key_pair = key_pairs.get(&current_key_id)
            .ok_or_else(|| anyhow!("Key pair not found: {}", current_key_id))?;

        // Perform Diffie-Hellman key exchange
        let secret = EphemeralSecret::from_slice(&key_pair.private_key);
        let peer_public = PublicKey::from_slice(peer_public_key)
            .map_err(|e| anyhow!("Invalid peer public key: {}", e))?;
        
        let shared_secret = secret.diffie_hellman(&peer_public);
        
        // Generate salt for HKDF
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        
        // Derive session key using HKDF
        let hk = Hkdf::<Sha256>::new(Some(&salt), shared_secret.as_bytes());
        let mut session_key_bytes = [0u8; 32];
        hk.expand(b"genxlink-session-key", &mut session_key_bytes)
            .map_err(|e| anyhow!("HKDF expansion failed: {}", e))?;

        let created_at = chrono::Utc::now();
        let expires_at = created_at + chrono::Duration::from_std(self.session_key_ttl)
            .map_err(|e| anyhow!("Invalid duration: {}", e))?;

        let session_key = SessionKey {
            session_id,
            key_id: current_key_id,
            shared_secret: session_key_bytes.to_vec(),
            salt: salt.to_vec(),
            created_at,
            expires_at,
        };

        // Store session key
        let mut session_keys = self.session_keys.write().await;
        session_keys.insert(session_id, session_key.clone());

        info!("Created session key for session: {}", session_id);
        Ok(session_key)
    }

    pub async fn encrypt_message(&self, session_id: Uuid, plaintext: &[u8], metadata: serde_json::Value) -> Result<EncryptedMessage> {
        // Get session key
        let session_keys = self.session_keys.read().await;
        let session_key = session_keys.get(&session_id)
            .ok_or_else(|| anyhow!("Session key not found: {}", session_id))?;

        // Check if session key is expired
        if chrono::Utc::now() > session_key.expires_at {
            return Err(anyhow!("Session key expired: {}", session_id));
        }

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&session_key.shared_secret);
        let cipher = Aes256Gcm::new(key);

        // Encrypt message
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Split ciphertext and tag (AES-GCM integrates the tag)
        let tag_length = 16; // AES-GCM tag length
        let ciphertext_len = ciphertext.len() - tag_length;
        let ciphertext_data = ciphertext[..ciphertext_len].to_vec();
        let tag_data = ciphertext[ciphertext_len..].to_vec();

        let encrypted_message = EncryptedMessage {
            message_id: Uuid::new_v4(),
            session_id,
            key_id: session_key.key_id,
            nonce: nonce_bytes.to_vec(),
            ciphertext: ciphertext_data,
            tag: tag_data,
            timestamp: chrono::Utc::now(),
            metadata,
        };

        debug!("Encrypted message for session: {} ({} bytes -> {} bytes)", 
               session_id, plaintext.len(), ciphertext.len());

        Ok(encrypted_message)
    }

    pub async fn decrypt_message(&self, encrypted_message: EncryptedMessage) -> Result<(Vec<u8>, serde_json::Value)> {
        // Get session key
        let session_keys = self.session_keys.read().await;
        let session_key = session_keys.get(&encrypted_message.session_id)
            .ok_or_else(|| anyhow!("Session key not found: {}", encrypted_message.session_id))?;

        // Check if session key is expired
        if chrono::Utc::now() > session_key.expires_at {
            return Err(anyhow!("Session key expired: {}", encrypted_message.session_id));
        }

        // Recombine ciphertext and tag
        let mut full_ciphertext = encrypted_message.ciphertext;
        full_ciphertext.extend_from_slice(&encrypted_message.tag);

        // Create nonce
        let nonce = Nonce::from_slice(&encrypted_message.nonce);

        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&session_key.shared_secret);
        let cipher = Aes256Gcm::new(key);

        // Decrypt message
        let plaintext = cipher.decrypt(nonce, full_ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        debug!("Decrypted message for session: {} ({} bytes -> {} bytes)", 
               encrypted_message.session_id, full_ciphertext.len(), plaintext.len());

        Ok((plaintext, encrypted_message.metadata))
    }

    pub async fn encrypt_stream_data(&self, session_id: Uuid, data: &[u8], sequence_number: u64) -> Result<Vec<u8>> {
        // For streaming data (video/audio), we use a more efficient approach
        // with sequence-based nonces instead of random nonces
        
        // Get session key
        let session_keys = self.session_keys.read().await;
        let session_key = session_keys.get(&session_id)
            .ok_or_else(|| anyhow!("Session key not found: {}", session_id))?;

        // Check if session key is expired
        if chrono::Utc::now() > session_key.expires_at {
            return Err(anyhow!("Session key expired: {}", session_id));
        }

        // Create nonce from sequence number
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&sequence_number.to_le_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&session_key.shared_secret);
        let cipher = Aes256Gcm::new(key);

        // Encrypt data
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| anyhow!("Stream encryption failed: {}", e))?;

        // Prepend sequence number for decryption
        let mut encrypted_data = Vec::with_capacity(8 + ciphertext.len());
        encrypted_data.extend_from_slice(&sequence_number.to_le_bytes());
        encrypted_data.extend_from_slice(&ciphertext);

        debug!("Encrypted stream data for session: {} (seq: {}, {} bytes -> {} bytes)", 
               session_id, sequence_number, data.len(), ciphertext.len());

        Ok(encrypted_data)
    }

    pub async fn decrypt_stream_data(&self, session_id: Uuid, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 8 {
            return Err(anyhow!("Invalid encrypted stream data: too short"));
        }

        // Extract sequence number
        let sequence_number = u64::from_le_bytes([
            encrypted_data[0], encrypted_data[1], encrypted_data[2], encrypted_data[3],
            encrypted_data[4], encrypted_data[5], encrypted_data[6], encrypted_data[7],
        ]);

        // Get session key
        let session_keys = self.session_keys.read().await;
        let session_key = session_keys.get(&session_id)
            .ok_or_else(|| anyhow!("Session key not found: {}", session_id))?;

        // Check if session key is expired
        if chrono::Utc::now() > session_key.expires_at {
            return Err(anyhow!("Session key expired: {}", session_id));
        }

        // Create nonce from sequence number
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&sequence_number.to_le_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&session_key.shared_secret);
        let cipher = Aes256Gcm::new(key);

        // Decrypt data
        let ciphertext = &encrypted_data[8..];
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Stream decryption failed: {}", e))?;

        debug!("Decrypted stream data for session: {} (seq: {}, {} bytes -> {} bytes)", 
               session_id, sequence_number, ciphertext.len(), plaintext.len());

        Ok(plaintext)
    }

    pub async fn rotate_key(&self) -> Result<()> {
        // Generate new key pair
        let new_key_pair = self.generate_key_pair().await?;
        
        // Add to key pairs
        let mut key_pairs = self.key_pairs.write().await;
        let mut current_key_id = self.current_key_id.write().await;
        
        key_pairs.insert(new_key_pair.key_id, new_key_pair.clone());
        *current_key_id = Some(new_key_pair.key_id);
        
        info!("Rotated to new key pair: {}", new_key_pair.key_id);
        
        // Clean up old expired keys
        self.cleanup_expired_keys().await?;
        
        Ok(())
    }

    async fn cleanup_expired_keys(&self) -> Result<()> {
        let now = chrono::Utc::now();
        let mut key_pairs = self.key_pairs.write().await;
        let mut session_keys = self.session_keys.write().await;
        
        let initial_key_count = key_pairs.len();
        let initial_session_count = session_keys.len();
        
        // Remove expired key pairs (keep current key even if expired)
        let current_key_id = self.current_key_id.read().await.clone();
        key_pairs.retain(|key_id, key_pair| {
            current_key_id.as_ref().map_or(true, |current| key_id == current) || key_pair.expires_at > now
        });
        
        // Remove expired session keys
        session_keys.retain(|_, session_key| session_key.expires_at > now);
        
        let cleaned_keys = initial_key_count - key_pairs.len();
        let cleaned_sessions = initial_session_count - session_keys.len();
        
        if cleaned_keys > 0 || cleaned_sessions > 0 {
            info!("Cleaned up {} expired key pairs and {} expired session keys", 
                  cleaned_keys, cleaned_sessions);
        }
        
        Ok(())
    }

    async fn start_key_rotation(&self) {
        let key_pairs = self.key_pairs.clone();
        let current_key_id = self.current_key_id.clone();
        let rotation_interval = self.key_rotation_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(rotation_interval);
            
            loop {
                interval.tick().await;
                
                info!("Performing scheduled key rotation");
                
                // Generate new key pair
                let new_key_pair = match Self::generate_key_pair_static().await {
                    Ok(kp) => kp,
                    Err(e) => {
                        error!("Failed to generate new key pair during rotation: {}", e);
                        continue;
                    }
                };
                
                // Update key pairs
                {
                    let mut key_pairs_guard = key_pairs.write().await;
                    let mut current_key_id_guard = current_key_id.write().await;
                    
                    key_pairs_guard.insert(new_key_pair.key_id, new_key_pair.clone());
                    *current_key_id_guard = Some(new_key_pair.key_id);
                }
                
                info!("Successfully rotated to new key pair: {}", new_key_pair.key_id);
            }
        });
    }

    async fn generate_key_pair_static() -> Result<EncryptionKeyPair> {
        let key_id = Uuid::new_v4();
        let created_at = chrono::Utc::now();
        let expires_at = created_at + chrono::Duration::hours(24); // 24 hour lifetime

        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret);
        
        let key_pair = EncryptionKeyPair {
            key_id,
            public_key: public_key.as_bytes().to_vec(),
            private_key: secret.to_bytes().to_vec(),
            created_at,
            expires_at,
        };

        Ok(key_pair)
    }

    pub async fn get_session_info(&self, session_id: Uuid) -> Result<Option<SessionKey>> {
        let session_keys = self.session_keys.read().await;
        Ok(session_keys.get(&session_id).cloned())
    }

    pub async fn revoke_session_key(&self, session_id: Uuid) -> Result<()> {
        let mut session_keys = self.session_keys.write().await;
        session_keys.remove(&session_id);
        info!("Revoked session key for session: {}", session_id);
        Ok(())
    }

    pub async fn get_encryption_stats(&self) -> EncryptionStats {
        let key_pairs = self.key_pairs.read().await;
        let session_keys = self.session_keys.read().await;
        let current_key_id = self.current_key_id.read().await;
        
        let now = chrono::Utc::now();
        let expired_keys = key_pairs.values()
            .filter(|kp| kp.expires_at < now)
            .count();
        let expired_sessions = session_keys.values()
            .filter(|sk| sk.expires_at < now)
            .count();
        
        EncryptionStats {
            total_key_pairs: key_pairs.len(),
            active_key_pairs: key_pairs.len() - expired_keys,
            expired_key_pairs: expired_keys,
            current_key_id: current_key_id.clone(),
            total_session_keys: session_keys.len(),
            active_session_keys: session_keys.len() - expired_sessions,
            expired_session_keys: expired_sessions,
            key_rotation_interval: self.key_rotation_interval,
            session_key_ttl: self.session_key_ttl,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStats {
    pub total_key_pairs: usize,
    pub active_key_pairs: usize,
    pub expired_key_pairs: usize,
    pub current_key_id: Option<Uuid>,
    pub total_session_keys: usize,
    pub active_session_keys: usize,
    pub expired_session_keys: usize,
    pub key_rotation_interval: std::time::Duration,
    pub session_key_ttl: std::time::Duration,
}
