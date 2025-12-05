use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::{SystemTime, Duration};
use tokio::net::UdpSocket as TokioUdpSocket;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use genxlink_protocol::DeviceId;

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

/// Device discovery message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryMessage {
    device_id: String,
    device_name: String,
    port: u16,
    timestamp: SystemTime,
}

/// LAN discovery manager
pub struct LanDiscoveryManager {
    discovered_devices: Vec<LanDevice>,
    is_discovering: bool,
    device_id: DeviceId,
    device_name: String,
    broadcast_tx: Option<mpsc::UnboundedSender<()>>,
}

const DISCOVERY_PORT: u16 = 9090;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(5);
const DEVICE_TIMEOUT: Duration = Duration::from_secs(30);

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
            device_id: DeviceId::new(),
            device_name: format!("GenXLink-{}", whoami::username()),
            broadcast_tx: None,
        }
    }
    
    /// Start LAN device discovery
    pub async fn start_discovery(&mut self) -> Result<(), String> {
        if self.is_discovering {
            return Ok(());
        }
        
        info!("🔍 Starting LAN device discovery on port {}", DISCOVERY_PORT);
        self.is_discovering = true;
        
        // Start broadcasting our presence
        let (tx, mut rx) = mpsc::unbounded_channel::<()>();
        self.broadcast_tx = Some(tx);
        
        let device_id = self.device_id.clone();
        let device_name = self.device_name.clone();
        
        // Spawn broadcast task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(BROADCAST_INTERVAL);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::broadcast_presence(&device_id, &device_name).await {
                            error!("Failed to broadcast presence: {}", e);
                        }
                    }
                    _ = rx.recv() => {
                        info!("Stopping broadcast task");
                        break;
                    }
                }
            }
        });
        
        // Start listening for other devices
        self.start_listener().await?;
        
        Ok(())
    }
    
    /// Stop LAN device discovery
    pub fn stop_discovery(&mut self) {
        if !self.is_discovering {
            return;
        }
        
        info!("⏹ Stopping LAN device discovery");
        self.is_discovering = false;
        
        // Stop broadcast task
        if let Some(tx) = self.broadcast_tx.take() {
            let _ = tx.send(());
        }
    }
    
    /// Get list of discovered devices
    pub fn get_devices(&self) -> Vec<&LanDevice> {
        self.cleanup_expired_devices();
        self.discovered_devices.iter().filter(|d| d.is_online).collect()
    }
    
    /// Start listening for discovery messages
    async fn start_listener(&mut self) -> Result<(), String> {
        let socket = TokioUdpSocket::bind(("0.0.0.0", DISCOVERY_PORT))
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", DISCOVERY_PORT, e))?;
        
        info!("📡 Listening for device discovery messages on port {}", DISCOVERY_PORT);
        
        let device_id = self.device_id.clone();
        let devices = std::sync::Arc::new(tokio::sync::Mutex::new(self.discovered_devices.clone()));
        let devices_clone = std::sync::Arc::clone(&devices);
        
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        let data = &buf[..len];
                        
                        // Parse discovery message
                        match serde_json::from_slice::<DiscoveryMessage>(data) {
                            Ok(msg) => {
                                // Ignore our own broadcasts
                                if msg.device_id == device_id.to_string() {
                                    continue;
                                }
                                
                                debug!("📨 Received discovery from {} at {}", msg.device_name, addr.ip());
                                
                                // Update device list
                                let mut devices = devices_clone.lock().await;
                                Self::update_device_list(&mut devices, msg, addr.ip());
                            }
                            Err(e) => {
                                debug!("Invalid discovery message from {}: {}", addr.ip(), e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving discovery message: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Broadcast our presence to the local network
    async fn broadcast_presence(device_id: &DeviceId, device_name: &str) -> Result<(), String> {
        let socket = TokioUdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| format!("Failed to bind broadcast socket: {}", e))?;
        
        let message = DiscoveryMessage {
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            port: DISCOVERY_PORT,
            timestamp: SystemTime::now(),
        };
        
        let data = serde_json::to_vec(&message)
            .map_err(|e| format!("Failed to serialize discovery message: {}", e))?;
        
        // Broadcast to common network ranges
        let broadcast_addresses = [
            "255.255.255.255:9090", // Global broadcast
            "192.168.255.255:9090", // Class C broadcast
            "10.255.255.255:9090",  // Class A broadcast
            "172.16.255.255:9090",  // Class B broadcast
        ];
        
        for addr in &broadcast_addresses {
            if let Err(e) = socket.send_to(&data, addr).await {
                debug!("Failed to send broadcast to {}: {}", addr, e);
            }
        }
        
        debug!("📡 Broadcasted presence to network");
        Ok(())
    }
    
    /// Update device list with new discovery
    fn update_device_list(devices: &mut Vec<LanDevice>, msg: DiscoveryMessage, ip: IpAddr) {
        // Check if device already exists
        if let Some(device) = devices.iter_mut().find(|d| d.device_id == msg.device_id) {
            device.last_seen = msg.timestamp;
            device.is_online = true;
        } else {
            // Add new device
            devices.push(LanDevice {
                device_id: msg.device_id,
                device_name: msg.device_name,
                ip_address: ip,
                port: msg.port,
                is_online: true,
                last_seen: msg.timestamp,
            });
        }
    }
    
    /// Remove expired devices
    fn cleanup_expired_devices(&self) {
        // This would be implemented with mutable self in a real scenario
        // For now, devices expire after timeout when queried
    }
}
