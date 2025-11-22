use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
}

pub struct SignalingServer {
    devices: HashMap<String, DeviceInfo>,
}

impl SignalingServer {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }
    
    pub fn register_device(&mut self, device_id: String, device_name: String) {
        let now = Utc::now();
        let device = DeviceInfo {
            device_id: device_id.clone(),
            device_name,
            connected_at: now,
            last_seen: now,
            is_online: true,
        };
        
        self.devices.insert(device_id.clone(), device);
        tracing::info!("Device registered: {}", device_id);
    }
    
    pub fn unregister_device(&mut self, device_id: &str) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.is_online = false;
            tracing::info!("Device unregistered: {}", device_id);
        }
    }
    
    pub fn update_last_seen(&mut self, device_id: &str) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.last_seen = Utc::now();
        }
    }
    
    pub fn get_devices(&self) -> Vec<DeviceInfo> {
        self.devices.values().cloned().collect()
    }
    
    pub fn get_device(&self, device_id: &str) -> Option<&DeviceInfo> {
        self.devices.get(device_id)
    }
}
