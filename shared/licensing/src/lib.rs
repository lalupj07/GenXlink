use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use genxlink_protocol::DeviceId;
use thiserror::Error;

pub mod types;
pub mod validator;

pub use types::*;
pub use validator::*;

/// License error types
#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("License expired")]
    Expired,
    
    #[error("Invalid license key")]
    InvalidKey,
    
    #[error("Device limit reached")]
    DeviceLimitReached,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Feature not available in this license tier")]
    FeatureNotAvailable,
    
    #[error("License not activated")]
    NotActivated,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// License plan types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicensePlan {
    Free,
    Pro,
    Enterprise,
}

impl LicensePlan {
    /// Get session time limit in minutes (None = unlimited)
    pub fn session_time_limit(&self) -> Option<u32> {
        match self {
            LicensePlan::Free => Some(10),
            LicensePlan::Pro => None,
            LicensePlan::Enterprise => None,
        }
    }
    
    /// Get maximum number of devices
    pub fn max_devices(&self) -> Option<u32> {
        match self {
            LicensePlan::Free => Some(1),
            LicensePlan::Pro => Some(5),
            LicensePlan::Enterprise => None,
        }
    }
    
    /// Check if file transfer is available
    pub fn has_file_transfer(&self) -> bool {
        matches!(self, LicensePlan::Pro | LicensePlan::Enterprise)
    }
    
    /// Check if unattended access is available
    pub fn has_unattended_access(&self) -> bool {
        matches!(self, LicensePlan::Pro | LicensePlan::Enterprise)
    }
    
    /// Check if priority relay is available
    pub fn has_priority_relay(&self) -> bool {
        matches!(self, LicensePlan::Pro | LicensePlan::Enterprise)
    }
    
    /// Check if multi-monitor support is available
    pub fn has_multi_monitor(&self) -> bool {
        matches!(self, LicensePlan::Pro | LicensePlan::Enterprise)
    }
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub license_key: String,
    pub plan: LicensePlan,
    pub device_id: DeviceId,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_devices: Option<u32>,
    pub signature: String,
}

impl License {
    /// Check if license is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Check if a feature is available
    pub fn has_feature(&self, feature: LicenseFeature) -> bool {
        if self.is_expired() {
            return false;
        }
        
        match feature {
            LicenseFeature::FileTransfer => self.plan.has_file_transfer(),
            LicenseFeature::UnattendedAccess => self.plan.has_unattended_access(),
            LicenseFeature::PriorityRelay => self.plan.has_priority_relay(),
            LicenseFeature::MultiMonitor => self.plan.has_multi_monitor(),
        }
    }
    
    /// Get session time limit
    pub fn session_time_limit(&self) -> Option<u32> {
        if self.is_expired() {
            Some(0)
        } else {
            self.plan.session_time_limit()
        }
    }
}

/// License features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseFeature {
    FileTransfer,
    UnattendedAccess,
    PriorityRelay,
    MultiMonitor,
}

/// License activation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRequest {
    pub license_key: String,
    pub device_id: DeviceId,
    pub device_name: String,
}

/// License activation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResponse {
    pub success: bool,
    pub license: Option<License>,
    pub error: Option<String>,
    pub jwt_token: Option<String>,
}
