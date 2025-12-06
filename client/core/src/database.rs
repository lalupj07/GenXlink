use crate::ClientError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Database client for Supabase integration
pub struct DatabaseClient {
    client: reqwest::Client,
    base_url: String,
    anon_key: String,
    auth_token: Option<String>,
}

/// Device registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistry {
    pub id: String,
    pub user_id: String,
    pub device_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub os_version: String,
    pub ip_address: String,
    pub mac_address: Option<String>,
    pub last_seen: SystemTime,
    pub is_online: bool,
    pub capabilities: DeviceCapabilities,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Device type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DeviceType {
    #[default]
    Desktop,
    Laptop,
    Server,
    Mobile,
    Tablet,
    IoT,
    Unknown,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Laptop => "laptop",
            Self::Server => "server",
            Self::Mobile => "mobile",
            Self::Tablet => "tablet",
            Self::IoT => "iot",
            Self::Unknown => "unknown",
        }
    }
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub screen_sharing: bool,
    pub remote_control: bool,
    pub file_transfer: bool,
    pub audio_streaming: bool,
    pub clipboard_access: bool,
    pub encryption: bool,
    pub multi_monitor: bool,
    pub max_resolution: (u32, u32),
    pub supported_codecs: Vec<String>,
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub subscription_type: SubscriptionType,
    pub created_at: SystemTime,
    pub last_login: Option<SystemTime>,
    pub preferences: UserPreferences,
}

/// Subscription type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionType {
    Free,
    Premium,
    Enterprise,
}

impl SubscriptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Premium => "premium",
            Self::Enterprise => "enterprise",
        }
    }
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,
    pub language: String,
    pub notifications_enabled: bool,
    pub auto_accept_connections: bool,
    pub require_confirmation: bool,
    pub max_concurrent_sessions: u32,
    pub default_permissions: Vec<String>,
}

/// Session record for tracking connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub user_id: String,
    pub device_id: String,
    pub remote_device_id: String,
    pub session_type: SessionType,
    pub started_at: SystemTime,
    pub ended_at: Option<SystemTime>,
    pub duration_seconds: Option<u64>,
    pub status: SessionStatus,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Session type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    RemoteControl,
    ScreenSharing,
    FileTransfer,
    AudioStreaming,
    MultiSession,
}

impl SessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RemoteControl => "remote_control",
            Self::ScreenSharing => "screen_sharing",
            Self::FileTransfer => "file_transfer",
            Self::AudioStreaming => "audio_streaming",
            Self::MultiSession => "multi_session",
        }
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
    Terminated,
    Timeout,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Terminated => "terminated",
            Self::Timeout => "timeout",
        }
    }
}

/// Database query options
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order_by: Option<String>,
    pub ascending: bool,
    pub filters: HashMap<String, String>,
}

/// Database response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseResponse<T> {
    pub data: Vec<T>,
    pub count: Option<u64>,
    pub error: Option<String>,
}

impl DatabaseClient {
    /// Create a new database client
    pub fn new(base_url: String, anon_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            anon_key,
            auth_token: None,
        }
    }

    /// Set authentication token
    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    /// Register a new device
    pub async fn register_device(&self, device: DeviceRegistry) -> Result<DeviceRegistry, ClientError> {
        let url = format!("{}/rest/v1/devices", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&device)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            let devices: Vec<DeviceRegistry> = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse response: {}", e)))?;
            Ok(devices.into_iter().next().unwrap_or(device))
        } else {
            Err(ClientError::IoError(format!("Device registration failed: {}", response.status())))
        }
    }

    /// Update device information
    pub async fn update_device(&self, device_id: &str, updates: PartialDeviceUpdate) -> Result<DeviceRegistry, ClientError> {
        let url = format!("{}/rest/v1/devices?id=eq.{}", self.base_url, device_id);
        
        let response = self.client
            .patch(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&updates)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            let devices: Vec<DeviceRegistry> = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse response: {}", e)))?;
            devices.into_iter().next()
                .ok_or_else(|| ClientError::IoError("No device returned".to_string()))
        } else {
            Err(ClientError::IoError(format!("Device update failed: {}", response.status())))
        }
    }

    /// Get device by ID
    pub async fn get_device(&self, device_id: &str) -> Result<Option<DeviceRegistry>, ClientError> {
        let url = format!("{}/rest/v1/devices?id=eq.{}", self.base_url, device_id);
        
        let response = self.client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            let devices: Vec<DeviceRegistry> = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse response: {}", e)))?;
            Ok(devices.into_iter().next())
        } else {
            Err(ClientError::IoError(format!("Failed to get device: {}", response.status())))
        }
    }

    /// Get user's devices
    pub async fn get_user_devices(&self, user_id: &str, options: Option<QueryOptions>) -> Result<Vec<DeviceRegistry>, ClientError> {
        let mut url = format!("{}/rest/v1/devices?user_id=eq.{}", self.base_url, user_id);
        
        if let Some(opts) = &options {
            if let Some(limit) = opts.limit {
                url.push_str(&format!("&limit={}", limit));
            }
            if let Some(offset) = opts.offset {
                url.push_str(&format!("&offset={}", offset));
            }
            if let Some(order_by) = &opts.order_by {
                url.push_str(&format!("&order={}.{}", order_by, if opts.ascending { "asc" } else { "desc" }));
            }
            for (key, value) in &opts.filters {
                url.push_str(&format!("&{}=eq.{}", key, value));
            }
        }
        
        let response = self.client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            let devices: Vec<DeviceRegistry> = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse response: {}", e)))?;
            Ok(devices)
        } else {
            Err(ClientError::IoError(format!("Failed to get user devices: {}", response.status())))
        }
    }

    /// Update device online status
    pub async fn update_device_status(&self, device_id: &str, is_online: bool) -> Result<(), ClientError> {
        let url = format!("{}/rest/v1/devices?id=eq.{}", self.base_url, device_id);
        
        let mut updates = PartialDeviceUpdate::default();
        updates.is_online = Some(is_online);
        updates.last_seen = Some(SystemTime::now());
        
        let response = self.client
            .patch(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .header("Content-Type", "application/json")
            .json(&updates)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::IoError(format!("Failed to update device status: {}", response.status())))
        }
    }

    /// Create session record
    pub async fn create_session(&self, session: SessionRecord) -> Result<SessionRecord, ClientError> {
        let url = format!("{}/rest/v1/sessions", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&session)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            let sessions: Vec<SessionRecord> = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse response: {}", e)))?;
            sessions.into_iter().next()
                .ok_or_else(|| ClientError::IoError("No session returned".to_string()))
        } else {
            Err(ClientError::IoError(format!("Session creation failed: {}", response.status())))
        }
    }

    /// Update session record
    pub async fn update_session(&self, session_id: &str, updates: PartialSessionUpdate) -> Result<SessionRecord, ClientError> {
        let url = format!("{}/rest/v1/sessions?id=eq.{}", self.base_url, session_id);
        
        let response = self.client
            .patch(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&updates)
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            let sessions: Vec<SessionRecord> = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse response: {}", e)))?;
            sessions.into_iter().next()
                .ok_or_else(|| ClientError::IoError("No session returned".to_string()))
        } else {
            Err(ClientError::IoError(format!("Session update failed: {}", response.status())))
        }
    }

    /// Get user sessions
    pub async fn get_user_sessions(&self, user_id: &str, options: Option<QueryOptions>) -> Result<Vec<SessionRecord>, ClientError> {
        let mut url = format!("{}/rest/v1/sessions?user_id=eq.{}", self.base_url, user_id);
        
        if let Some(opts) = &options {
            if let Some(limit) = opts.limit {
                url.push_str(&format!("&limit={}", limit));
            }
            if let Some(order_by) = &opts.order_by {
                url.push_str(&format!("&order={}.{}", order_by, if opts.ascending { "asc" } else { "desc" }));
            }
        }
        
        let response = self.client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            let sessions: Vec<SessionRecord> = response.json().await
                .map_err(|e| ClientError::IoError(format!("Failed to parse response: {}", e)))?;
            Ok(sessions)
        } else {
            Err(ClientError::IoError(format!("Failed to get user sessions: {}", response.status())))
        }
    }

    /// Delete device
    pub async fn delete_device(&self, device_id: &str) -> Result<(), ClientError> {
        let url = format!("{}/rest/v1/devices?id=eq.{}", self.base_url, device_id);
        
        let response = self.client
            .delete(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.auth_token.as_ref().unwrap_or(&self.anon_key)))
            .send()
            .await
            .map_err(|e| ClientError::IoError(format!("Database request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::IoError(format!("Failed to delete device: {}", response.status())))
        }
    }
}

/// Partial device update structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialDeviceUpdate {
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub last_seen: Option<SystemTime>,
    pub is_online: Option<bool>,
    pub capabilities: Option<DeviceCapabilities>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Partial session update structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialSessionUpdate {
    pub ended_at: Option<SystemTime>,
    pub duration_seconds: Option<u64>,
    pub status: Option<SessionStatus>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            screen_sharing: true,
            remote_control: true,
            file_transfer: true,
            audio_streaming: true,
            clipboard_access: true,
            encryption: true,
            multi_monitor: false,
            max_resolution: (1920, 1080),
            supported_codecs: vec!["h264".to_string(), "vp8".to_string()],
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications_enabled: true,
            auto_accept_connections: false,
            require_confirmation: true,
            max_concurrent_sessions: 5,
            default_permissions: vec!["screen_sharing".to_string(), "remote_control".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_serialization() {
        let device_type = DeviceType::Desktop;
        assert_eq!(device_type.as_str(), "desktop");
    }

    #[test]
    fn test_subscription_type_serialization() {
        let subscription = SubscriptionType::Premium;
        assert_eq!(subscription.as_str(), "premium");
    }

    #[test]
    fn test_session_type_serialization() {
        let session_type = SessionType::RemoteControl;
        assert_eq!(session_type.as_str(), "remote_control");
    }

    #[test]
    fn test_device_capabilities_default() {
        let capabilities = DeviceCapabilities::default();
        assert!(capabilities.screen_sharing);
        assert!(capabilities.remote_control);
        assert_eq!(capabilities.max_resolution, (1920, 1080));
    }

    #[test]
    fn test_user_preferences_default() {
        let preferences = UserPreferences::default();
        assert_eq!(preferences.theme, "dark");
        assert_eq!(preferences.language, "en");
        assert!(preferences.notifications_enabled);
        assert!(!preferences.auto_accept_connections);
        assert_eq!(preferences.max_concurrent_sessions, 5);
    }
}
