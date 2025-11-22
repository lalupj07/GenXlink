use crate::{License, LicenseError, OfflineLicense, LicensePlan};
use genxlink_crypto::LicenseSignature;
use genxlink_protocol::DeviceId;
use chrono::{DateTime, Utc};

/// License validator
pub struct LicenseValidator {
    signature: LicenseSignature,
}

impl LicenseValidator {
    /// Create a new validator with a license signature helper
    pub fn new(signature: LicenseSignature) -> Self {
        Self { signature }
    }
    
    /// Validate a license
    pub fn validate(&self, license: &License) -> Result<(), LicenseError> {
        // Check expiration
        if license.is_expired() {
            return Err(LicenseError::Expired);
        }
        
        // Verify signature
        let data = self.license_data_for_signature(license);
        let valid = self.signature.verify(data.as_bytes(), &license.signature)
            .map_err(|_| LicenseError::InvalidSignature)?;
        
        if !valid {
            return Err(LicenseError::InvalidSignature);
        }
        
        Ok(())
    }
    
    /// Validate an offline license file
    pub fn validate_offline(&self, offline_license: &OfflineLicense) -> Result<License, LicenseError> {
        // Verify signature
        let data = offline_license.signable_data();
        let valid = self.signature.verify(data.as_bytes(), &offline_license.signature)
            .map_err(|_| LicenseError::InvalidSignature)?;
        
        if !valid {
            return Err(LicenseError::InvalidSignature);
        }
        
        // Parse plan
        let plan = match offline_license.plan.as_str() {
            "free" => LicensePlan::Free,
            "pro" => LicensePlan::Pro,
            "enterprise" => LicensePlan::Enterprise,
            _ => return Err(LicenseError::InvalidKey),
        };
        
        // Parse dates
        let issued_at = DateTime::parse_from_rfc3339(&offline_license.issued_at)
            .map_err(|_| LicenseError::InvalidKey)?
            .with_timezone(&Utc);
        
        let expires_at = if let Some(expires) = &offline_license.expires {
            Some(DateTime::parse_from_rfc3339(expires)
                .map_err(|_| LicenseError::InvalidKey)?
                .with_timezone(&Utc))
        } else {
            None
        };
        
        // Create license
        let license = License {
            license_key: offline_license.license_key.clone(),
            plan,
            device_id: DeviceId::from_string(offline_license.device_id.clone()),
            issued_at,
            expires_at,
            max_devices: offline_license.max_devices,
            signature: offline_license.signature.clone(),
        };
        
        // Check if expired
        if license.is_expired() {
            return Err(LicenseError::Expired);
        }
        
        Ok(license)
    }
    
    /// Generate signature data for a license
    fn license_data_for_signature(&self, license: &License) -> String {
        format!(
            "{}|{}|{:?}|{}|{}|{}",
            license.license_key,
            license.device_id,
            license.plan,
            license.issued_at.to_rfc3339(),
            license.expires_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            license.max_devices.map(|d| d.to_string()).unwrap_or_default()
        )
    }
}

/// License key generator
pub struct LicenseKeyGenerator;

impl LicenseKeyGenerator {
    /// Generate a new license key
    pub fn generate() -> String {
        use uuid::Uuid;
        let uuid = Uuid::new_v4();
        let hex = uuid.to_string().replace("-", "").to_uppercase();
        
        // Format as XXXXX-XXXXX-XXXXX-XXXXX-XXXXX
        format!(
            "{}-{}-{}-{}-{}",
            &hex[0..5],
            &hex[5..10],
            &hex[10..15],
            &hex[15..20],
            &hex[20..25]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_key_generation() {
        let key = LicenseKeyGenerator::generate();
        assert_eq!(key.len(), 29); // 5 groups of 5 chars + 4 dashes
        assert_eq!(key.matches('-').count(), 4);
    }
}
