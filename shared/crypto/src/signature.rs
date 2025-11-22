use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Sign};
use sha2::{Sha256, Digest};
use crate::CryptoError;

/// Sign data using RSA private key
pub fn sign_data(private_key: &RsaPrivateKey, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    
    let padding = Pkcs1v15Sign::new_unprefixed();
    let signature = private_key.sign_with_rng(&mut rand::thread_rng(), padding, &hash)
        .map_err(|e| CryptoError::SignatureError(format!("Sign failed: {}", e)))?;
    
    Ok(signature)
}

/// Verify signature using RSA public key
pub fn verify_signature(
    public_key: &RsaPublicKey,
    data: &[u8],
    signature: &[u8],
) -> Result<bool, CryptoError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    
    let padding = Pkcs1v15Sign::new_unprefixed();
    
    match public_key.verify(padding, &hash, signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// License signature helper
pub struct LicenseSignature {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl LicenseSignature {
    /// Create a signature for license data
    /// Sign data and return base64 signature
    pub fn sign(&self, data: &[u8]) -> Result<String, CryptoError> {
        use base64::{Engine as _, engine::general_purpose};
        let signature = sign_data(&self.private_key, data)?;
        Ok(general_purpose::STANDARD.encode(&signature))
    }
    
    /// Verify base64 signature
    pub fn verify(&self, data: &[u8], signature_b64: &str) -> Result<bool, CryptoError> {
        use base64::{Engine as _, engine::general_purpose};
        let signature = general_purpose::STANDARD.decode(signature_b64)
            .map_err(|e| CryptoError::SignatureError(format!("Invalid base64: {}", e)))?;
        verify_signature(&self.public_key, data, &signature)
    }
}
