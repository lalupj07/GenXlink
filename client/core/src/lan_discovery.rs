use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::SystemTime;

/// LAN device for offline P2P connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanDevice {
    pub device_id: String,
    pub device_name: String,
    pub ip_address: IpAddr,
    pub port: u16,
    pub is_online: bool,
    pub last_seen: SystemTime,
}

/// LAN discovery manager
pub struct LanDiscoveryManager {
    discovered_devices: Vec<LanDevice>,
    is_discovering: bool,
}

impl Default for LanDiscoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LanDiscoveryManager {
    pub fn new() -> Self {
        Self {
            discovered_devices: Vec::new(),
            is_discovering: false,
        }
    }
    
    pub fn start_discovery(&mut self) -> Result<(), String> {
        self.is_discovering = true;
        Ok(())
    }
    
    pub fn stop_discovery(&mut self) {
        self.is_discovering = false;
    }
    
    pub fn get_devices(&self) -> Vec<&LanDevice> {
        self.discovered_devices.iter().filter(|d| d.is_online).collect()
    }
}
