// Copyright (c) 2025 GenXis Innovations
// Discovery Service - LAN device discovery and network scanning

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Device type
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Server,
    Mobile,
    Unknown,
}

/// Device status
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Busy,
    Away,
}

/// Discovered device info
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub ip_address: String,
    pub port: u16,
    pub device_type: DeviceType,
    pub os_info: String,
    pub status: DeviceStatus,
    pub last_seen: Instant,
    pub signal_strength: i32, // -100 to 0 dBm
    pub capabilities: DeviceCapabilities,
}

/// Device capabilities
#[derive(Debug, Clone, Default)]
pub struct DeviceCapabilities {
    pub screen_share: bool,
    pub remote_control: bool,
    pub file_transfer: bool,
    pub audio: bool,
    pub clipboard: bool,
}

impl DiscoveredDevice {
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }

    pub fn signal_quality(&self) -> &str {
        match self.signal_strength {
            -50..=0 => "Excellent",
            -70..=-51 => "Good",
            -80..=-71 => "Fair",
            _ => "Poor",
        }
    }
}

/// Discovery Service
pub struct DiscoveryService {
    devices: HashMap<String, DiscoveredDevice>,
    discovery_port: u16,
    broadcast_interval: Duration,
    device_timeout: Duration,
    is_running: bool,
    local_device_name: String,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            discovery_port: 45678,
            broadcast_interval: Duration::from_secs(5),
            device_timeout: Duration::from_secs(30),
            is_running: false,
            local_device_name: get_hostname(),
        }
    }

    /// Start device discovery (synchronous version)
    pub fn start_discovery_sync(&mut self) {
        if self.is_running {
            return;
        }

        self.is_running = true;
        println!("🔍 Starting device discovery on port {}", self.discovery_port);

        // Simulate discovering some devices on the network
        self.simulate_device_discovery();
    }

    /// Start device discovery (async version)
    pub async fn start_discovery(&mut self) {
        self.start_discovery_sync();
    }

    /// Stop device discovery
    pub fn stop_discovery(&mut self) {
        self.is_running = false;
        println!("🛑 Device discovery stopped");
    }

    /// Simulate discovering devices (for demo purposes)
    fn simulate_device_discovery(&mut self) {
        // Add some simulated devices
        let devices = vec![
            DiscoveredDevice {
                id: "GENX-001".to_string(),
                name: "DESKTOP-DEMO".to_string(),
                ip_address: "192.168.1.100".to_string(),
                port: 45678,
                device_type: DeviceType::Desktop,
                os_info: "Windows 11 Pro".to_string(),
                status: DeviceStatus::Online,
                last_seen: Instant::now(),
                signal_strength: -45,
                capabilities: DeviceCapabilities {
                    screen_share: true,
                    remote_control: true,
                    file_transfer: true,
                    audio: true,
                    clipboard: true,
                },
            },
            DiscoveredDevice {
                id: "GENX-002".to_string(),
                name: "LAPTOP-WORK".to_string(),
                ip_address: "192.168.1.101".to_string(),
                port: 45678,
                device_type: DeviceType::Laptop,
                os_info: "Windows 11 Home".to_string(),
                status: DeviceStatus::Online,
                last_seen: Instant::now(),
                signal_strength: -55,
                capabilities: DeviceCapabilities {
                    screen_share: true,
                    remote_control: true,
                    file_transfer: true,
                    audio: true,
                    clipboard: true,
                },
            },
            DiscoveredDevice {
                id: "GENX-003".to_string(),
                name: "SERVER-MAIN".to_string(),
                ip_address: "192.168.1.10".to_string(),
                port: 45678,
                device_type: DeviceType::Server,
                os_info: "Ubuntu Server 22.04".to_string(),
                status: DeviceStatus::Online,
                last_seen: Instant::now(),
                signal_strength: -40,
                capabilities: DeviceCapabilities {
                    screen_share: true,
                    remote_control: true,
                    file_transfer: true,
                    audio: false,
                    clipboard: true,
                },
            },
        ];

        for device in devices {
            self.devices.insert(device.id.clone(), device);
        }

        println!("✅ Discovered {} devices", self.devices.len());
    }

    /// Scan network for devices
    pub async fn scan_network(&mut self) -> Vec<DiscoveredDevice> {
        println!("🔍 Scanning network...");
        
        // Remove stale devices
        self.cleanup_stale_devices();

        // In production, this would:
        // 1. Send UDP broadcast packets
        // 2. Listen for responses
        // 3. Parse device info from responses

        // For now, refresh simulated devices
        for device in self.devices.values_mut() {
            device.last_seen = Instant::now();
        }

        self.devices.values().cloned().collect()
    }

    /// Get all discovered devices
    pub fn get_devices(&self) -> Vec<&DiscoveredDevice> {
        self.devices.values().collect()
    }

    /// Get online devices
    pub fn get_online_devices(&self) -> Vec<&DiscoveredDevice> {
        self.devices.values()
            .filter(|d| d.status == DeviceStatus::Online)
            .collect()
    }

    /// Get device by ID
    pub fn get_device(&self, device_id: &str) -> Option<&DiscoveredDevice> {
        self.devices.get(device_id)
    }

    /// Get device by IP
    pub fn get_device_by_ip(&self, ip: &str) -> Option<&DiscoveredDevice> {
        self.devices.values().find(|d| d.ip_address == ip)
    }

    /// Update device status
    pub fn update_device_status(&mut self, device_id: &str, status: DeviceStatus) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.status = status;
            device.last_seen = Instant::now();
        }
    }

    /// Remove stale devices
    pub fn cleanup_stale_devices(&mut self) {
        let timeout = self.device_timeout;
        self.devices.retain(|_, d| !d.is_stale(timeout));
    }

    /// Get device count
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Get online device count
    pub fn online_device_count(&self) -> usize {
        self.devices.values()
            .filter(|d| d.status == DeviceStatus::Online)
            .count()
    }

    /// Check if discovery is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Add a device manually (for testing or manual entry)
    pub fn add_device(&mut self, device: DiscoveredDevice) {
        self.devices.insert(device.id.clone(), device);
    }

    /// Remove a device
    pub fn remove_device(&mut self, device_id: &str) {
        self.devices.remove(device_id);
    }

    /// Send discovery broadcast
    pub fn send_broadcast(&self) -> Result<(), String> {
        // In production, this would send a UDP broadcast
        println!("📡 Sending discovery broadcast...");
        Ok(())
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Get local hostname
fn get_hostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}
