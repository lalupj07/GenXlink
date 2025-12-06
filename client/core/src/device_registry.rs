use crate::auth_service::AuthService;
use crate::database::{DatabaseClient, DeviceRegistry, DeviceType, DeviceCapabilities, QueryOptions};
use crate::ClientError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Device registry service for managing device registration and synchronization
pub struct DeviceRegistryService {
    auth_service: AuthService,
    database_client: DatabaseClient,
    local_device: RwLock<Option<DeviceRegistry>>,
    registered_devices: RwLock<HashMap<String, DeviceRegistry>>,
    sync_interval: std::time::Duration,
    last_sync: RwLock<SystemTime>,
}

/// Device registration request
#[derive(Debug, Clone, Serialize)]
pub struct DeviceRegistrationRequest {
    pub device_name: String,
    pub device_type: DeviceType,
    pub os_version: String,
    pub ip_address: String,
    pub mac_address: Option<String>,
    pub capabilities: DeviceCapabilities,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Device sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncStatus {
    pub device_id: String,
    pub last_sync: SystemTime,
    pub is_online: bool,
    pub sync_success: bool,
    pub error_message: Option<String>,
}

/// Device discovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDiscoveryResult {
    pub devices: Vec<DeviceRegistry>,
    pub total_count: u32,
    pub online_count: u32,
    pub last_updated: SystemTime,
}

impl DeviceRegistryService {
    /// Create a new device registry service
    pub fn new(auth_service: AuthService, database_client: DatabaseClient) -> Self {
        Self {
            auth_service,
            database_client,
            local_device: RwLock::new(None),
            registered_devices: RwLock::new(HashMap::new()),
            sync_interval: std::time::Duration::from_secs(60), // Sync every minute
            last_sync: RwLock::new(SystemTime::UNIX_EPOCH),
        }
    }

    /// Register the current device
    pub async fn register_local_device(&self, request: DeviceRegistrationRequest) -> Result<DeviceRegistry, ClientError> {
        if !self.auth_service.is_authenticated() {
            return Err(ClientError::IoError("User not authenticated".to_string()));
        }

        let user = self.auth_service.get_current_user()
            .ok_or_else(|| ClientError::IoError("No current user".to_string()))?;

        let device = DeviceRegistry {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            device_id: self.generate_device_id(&request),
            device_name: request.device_name.clone(),
            device_type: request.device_type,
            os_version: request.os_version,
            ip_address: request.ip_address,
            mac_address: request.mac_address,
            last_seen: SystemTime::now(),
            is_online: true,
            capabilities: request.capabilities,
            metadata: request.metadata,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        let registered_device = self.database_client.register_device(device.clone()).await?;
        
        // Store locally
        {
            let mut local_device = self.local_device.write().await;
            *local_device = Some(registered_device.clone());
        }

        // Add to registered devices
        {
            let mut devices = self.registered_devices.write().await;
            devices.insert(registered_device.device_id.clone(), registered_device.clone());
        }

        tracing::info!("Registered device: {} ({})", registered_device.device_name, registered_device.device_id);
        Ok(registered_device)
    }

    /// Update local device status
    pub async fn update_local_device_status(&self, is_online: bool) -> Result<(), ClientError> {
        let device_id = {
            let local_device = self.local_device.read().await;
            local_device.as_ref()
                .ok_or_else(|| ClientError::IoError("No local device registered".to_string()))?
                .device_id.clone()
        };

        self.database_client.update_device_status(&device_id, is_online).await?;

        // Update local cache
        {
            let mut local_device = self.local_device.write().await;
            if let Some(device) = local_device.as_mut() {
                device.is_online = is_online;
                device.last_seen = SystemTime::now();
            }
        }

        {
            let mut devices = self.registered_devices.write().await;
            if let Some(device) = devices.get_mut(&device_id) {
                device.is_online = is_online;
                device.last_seen = SystemTime::now();
            }
        }

        Ok(())
    }

    /// Get local device information
    pub async fn get_local_device(&self) -> Option<DeviceRegistry> {
        self.local_device.read().await.clone()
    }

    /// Get all registered devices for the current user
    pub async fn get_user_devices(&self, options: Option<QueryOptions>) -> Result<Vec<DeviceRegistry>, ClientError> {
        if !self.auth_service.is_authenticated() {
            return Err(ClientError::IoError("User not authenticated".to_string()));
        }

        let user = self.auth_service.get_current_user()
            .ok_or_else(|| ClientError::IoError("No current user".to_string()))?;

        let devices = self.database_client.get_user_devices(&user.id, options).await?;
        
        // Update local cache
        {
            let mut registered_devices = self.registered_devices.write().await;
            for device in &devices {
                registered_devices.insert(device.device_id.clone(), device.clone());
            }
        }

        Ok(devices)
    }

    /// Get device by ID
    pub async fn get_device(&self, device_id: &str) -> Result<Option<DeviceRegistry>, ClientError> {
        // Check local cache first
        {
            let devices = self.registered_devices.read().await;
            if let Some(device) = devices.get(device_id) {
                return Ok(Some(device.clone()));
            }
        }

        // Fetch from database
        let device = self.database_client.get_device(device_id).await?;
        
        // Update local cache
        if let Some(ref device) = device {
            let mut devices = self.registered_devices.write().await;
            devices.insert(device_id.to_string(), device.clone());
        }

        Ok(device)
    }

    /// Update device information
    pub async fn update_device(&self, device_id: &str, updates: DeviceUpdateRequest) -> Result<DeviceRegistry, ClientError> {
        let partial_update = updates.to_partial_update();
        let updated_device = self.database_client.update_device(device_id, partial_update).await?;
        
        // Update local cache
        {
            let mut devices = self.registered_devices.write().await;
            devices.insert(device_id.to_string(), updated_device.clone());
        }

        // Update local device if it's the one being updated
        {
            let mut local_device = self.local_device.write().await;
            if let Some(device) = local_device.as_mut() {
                if device.device_id == device_id {
                    *device = updated_device.clone();
                }
            }
        }

        Ok(updated_device)
    }

    /// Delete device
    pub async fn delete_device(&self, device_id: &str) -> Result<(), ClientError> {
        self.database_client.delete_device(device_id).await?;
        
        // Remove from local cache
        {
            let mut devices = self.registered_devices.write().await;
            devices.remove(device_id);
        }

        // Clear local device if it's the one being deleted
        {
            let mut local_device = self.local_device.write().await;
            if let Some(device) = local_device.as_ref() {
                if device.device_id == device_id {
                    *local_device = None;
                }
            }
        }

        Ok(())
    }

    /// Sync devices with server
    pub async fn sync_devices(&self) -> Result<DeviceSyncStatus, ClientError> {
        if !self.auth_service.is_authenticated() {
            return Err(ClientError::IoError("User not authenticated".to_string()));
        }

        let local_device_id = {
            let local_device = self.local_device.read().await;
            local_device.as_ref().map(|d| d.device_id.clone())
        };

        let sync_result = if let Some(device_id) = local_device_id {
            // Update local device status
            match self.update_local_device_status(true).await {
                Ok(_) => DeviceSyncStatus {
                    device_id: device_id.clone(),
                    last_sync: SystemTime::now(),
                    is_online: true,
                    sync_success: true,
                    error_message: None,
                },
                Err(e) => DeviceSyncStatus {
                    device_id: device_id.clone(),
                    last_sync: SystemTime::now(),
                    is_online: false,
                    sync_success: false,
                    error_message: Some(e.to_string()),
                },
            }
        } else {
            DeviceSyncStatus {
                device_id: "unknown".to_string(),
                last_sync: SystemTime::now(),
                is_online: false,
                sync_success: false,
                error_message: Some("No local device registered".to_string()),
            }
        };

        // Update last sync time
        {
            let mut last_sync = self.last_sync.write().await;
            *last_sync = SystemTime::now();
        }

        Ok(sync_result)
    }

    /// Discover devices (get user devices with online status)
    pub async fn discover_devices(&self) -> Result<DeviceDiscoveryResult, ClientError> {
        let devices = self.get_user_devices(None).await?;
        let total_count = devices.len() as u32;
        let online_count = devices.iter().filter(|d| d.is_online).count() as u32;

        Ok(DeviceDiscoveryResult {
            devices,
            total_count,
            online_count,
            last_updated: SystemTime::now(),
        })
    }

    /// Check if sync is needed
    pub async fn is_sync_needed(&self) -> bool {
        let last_sync = *self.last_sync.read().await;
        SystemTime::now().duration_since(last_sync).unwrap_or_default() >= self.sync_interval
    }

    /// Get sync status
    pub async fn get_sync_status(&self) -> DeviceSyncStatus {
        let local_device_id = {
            let local_device = self.local_device.read().await;
            local_device.as_ref().map(|d| d.device_id.clone())
        };

        let last_sync = *self.last_sync.read().await;

        if let Some(device_id) = local_device_id {
            DeviceSyncStatus {
                device_id,
                last_sync,
                is_online: true,
                sync_success: true,
                error_message: None,
            }
        } else {
            DeviceSyncStatus {
                device_id: "unknown".to_string(),
                last_sync,
                is_online: false,
                sync_success: false,
                error_message: Some("No local device registered".to_string()),
            }
        }
    }

    /// Generate unique device ID
    fn generate_device_id(&self, request: &DeviceRegistrationRequest) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        request.device_name.hash(&mut hasher);
        request.os_version.hash(&mut hasher);
        if let Some(mac) = &request.mac_address {
            mac.hash(&mut hasher);
        }
        
        format!("device_{:x}", hasher.finish())
    }

    /// Start automatic sync task
    pub fn start_auto_sync(&self) -> tokio::task::JoinHandle<()> {
        let sync_interval = self.sync_interval;
        
        // Create a channel to communicate with the registry service
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SyncCommand>();
        
        // Spawn the sync task
        let sync_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(sync_interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Send sync command
                        let _ = tx.send(SyncCommand::Sync);
                    }
                    command = rx.recv() => {
                        match command {
                            Some(SyncCommand::Sync) => {
                                // Handle sync (would need to be implemented with proper communication)
                                tracing::debug!("Auto sync triggered");
                            }
                            Some(SyncCommand::Stop) => {
                                break;
                            }
                            None => break,
                        }
                    }
                }
            }
        });
        
        sync_handle
    }
}

/// Sync commands for the auto-sync task
#[derive(Debug)]
enum SyncCommand {
    Sync,
    Stop,
}

/// Device update request
#[derive(Debug, Clone, Serialize)]
pub struct DeviceUpdateRequest {
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub capabilities: Option<DeviceCapabilities>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl DeviceUpdateRequest {
    fn to_partial_update(&self) -> crate::database::PartialDeviceUpdate {
        crate::database::PartialDeviceUpdate {
            device_name: self.device_name.clone(),
            ip_address: self.ip_address.clone(),
            last_seen: Some(SystemTime::now()),
            is_online: None,
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{DatabaseClient, DeviceCapabilities};

    #[test]
    fn test_device_registry_creation() {
        let auth_service = AuthService::new(
            "https://test.supabase.co".to_string(),
            "test_key".to_string(),
        );
        let database_client = DatabaseClient::new(
            "https://test.supabase.co/rest/v1".to_string(),
            "test_key".to_string(),
        );
        
        let registry = DeviceRegistryService::new(auth_service, database_client);
        
        // Initially no local device
        let local_device = futures::executor::block_on(registry.get_local_device());
        assert!(local_device.is_none());
    }

    #[test]
    fn test_device_registration_request() {
        let request = DeviceRegistrationRequest {
            device_name: "Test Device".to_string(),
            device_type: DeviceType::Desktop,
            os_version: "Windows 10".to_string(),
            ip_address: "192.168.1.100".to_string(),
            mac_address: Some("00:11:22:33:44:55".to_string()),
            capabilities: DeviceCapabilities::default(),
            metadata: HashMap::new(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Test Device"));
        assert!(json.contains("Windows 10"));
        assert!(json.contains("192.168.1.100"));
    }

    #[test]
    fn test_device_sync_status() {
        let status = DeviceSyncStatus {
            device_id: "device_123".to_string(),
            last_sync: SystemTime::now(),
            is_online: true,
            sync_success: true,
            error_message: None,
        };
        
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("device_123"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_device_discovery_result() {
        let result = DeviceDiscoveryResult {
            devices: vec![],
            total_count: 0,
            online_count: 0,
            last_updated: SystemTime::now(),
        };
        
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("total_count"));
        assert!(json.contains("online_count"));
    }

    #[test]
    fn test_device_update_request() {
        let mut metadata = HashMap::new();
        metadata.insert("test".to_string(), serde_json::Value::String("value".to_string()));
        
        let request = DeviceUpdateRequest {
            device_name: Some("Updated Device".to_string()),
            ip_address: Some("192.168.1.200".to_string()),
            capabilities: Some(DeviceCapabilities::default()),
            metadata: Some(metadata),
        };
        
        let partial_update = request.to_partial_update();
        assert_eq!(partial_update.device_name, Some("Updated Device".to_string()));
        assert_eq!(partial_update.ip_address, Some("192.168.1.200".to_string()));
    }
}
