use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// License status in database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseStatus {
    Active,
    Suspended,
    Expired,
    Revoked,
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub status: UserStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
}

/// License record in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseRecord {
    pub id: i64,
    pub user_id: i64,
    pub license_key: String,
    pub plan_type: String,
    pub max_devices: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: LicenseStatus,
    pub created_at: DateTime<Utc>,
}

/// Device link record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLink {
    pub id: i64,
    pub license_id: i64,
    pub device_id: String,
    pub device_name: String,
    pub activated_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
}

/// Offline license file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineLicense {
    pub license_key: String,
    pub device_id: String,
    pub plan: String,
    pub expires: Option<String>,
    pub max_devices: Option<u32>,
    pub issued_at: String,
    pub signature: String,
}

impl OfflineLicense {
    /// Get the data to be signed (everything except signature)
    pub fn signable_data(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.license_key,
            self.device_id,
            self.plan,
            self.expires.as_deref().unwrap_or(""),
            self.max_devices.map(|d| d.to_string()).unwrap_or_default(),
            self.issued_at
        )
    }
}
