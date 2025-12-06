use egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use genxlink_client_core::lan_discovery::{LanDiscoveryManager, LanDevice};
use tracing::{info, error};

/// Device management panel
pub struct DevicesPanel {
    discovery_manager: Arc<Mutex<LanDiscoveryManager>>,
    discovered_devices: Vec<LanDevice>,
    is_scanning: bool,
    selected_device: Option<String>,
    error_message: Option<String>,
    last_scan_time: Option<std::time::Instant>,
}

impl DevicesPanel {
    pub fn new() -> Self {
        Self {
            discovery_manager: Arc::new(Mutex::new(LanDiscoveryManager::new())),
            discovered_devices: Vec::new(),
            is_scanning: false,
            selected_device: None,
            error_message: None,
            last_scan_time: None,
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📱 Devices");
        ui.add_space(10.0);
        
        // Scan controls
        self.show_scan_controls(ui);
        ui.add_space(10.0);
        
        // Device list
        self.show_device_list(ui);
        ui.add_space(10.0);
        
        // Connection controls
        self.show_connection_controls(ui);
        
        // Error message
        if let Some(error) = &self.error_message {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
        }
    }
    
    fn show_scan_controls(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("🔍 Device Discovery");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                if !self.is_scanning {
                    if ui.button("🚀 Start Scanning").clicked() {
                        self.start_scanning();
                    }
                } else {
                    if ui.button("⏹ Stop Scanning").clicked() {
                        self.stop_scanning();
                    }
                    ui.spinner();
                    ui.label("Scanning...");
                }
                
                ui.add_enabled(self.is_scanning, egui::Button::new("🔄 Refresh"))
                    .clicked()
                    .then(|| self.refresh_devices());
            });
            
            if let Some(last_scan) = self.last_scan_time {
                let elapsed = last_scan.elapsed();
                ui.label(format!("Last scan: {:.1}s ago", elapsed.as_secs_f32()));
            }
        });
    }
    
    fn show_device_list(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(format!("📋 Discovered Devices ({})", self.discovered_devices.len()));
            ui.add_space(5.0);
            
            if self.discovered_devices.is_empty() {
                ui.colored_label(egui::Color32::GRAY, "No devices found. Start scanning to discover devices on your network.");
                return;
            }
            
            // Device list
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for device in &self.discovered_devices {
                        let device_id = device.device_id.clone();
                        let is_selected = self.selected_device.as_ref() == Some(&device_id);
                        
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, "📱").clicked() {
                                self.selected_device = Some(device_id.clone());
                            }
                            
                            ui.vertical(|ui| {
                                ui.colored_label(
                                    if device.is_online { egui::Color32::GREEN } else { egui::Color32::GRAY },
                                    device.device_name.clone()
                                );
                                ui.label(format!("ID: {}...{}", &device.device_id[..8], &device.device_id[device.device_id.len()-8..]));
                                ui.label(format!("IP: {}:{}", device.ip_address, device.port));
                            });
                        });
                        
                        ui.separator();
                    }
                });
        });
    }
    
    fn show_connection_controls(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("🔗 Connection");
            ui.add_space(5.0);
            
            if let Some(selected_id) = &self.selected_device {
                ui.label(format!("Selected: {}...{}", &selected_id[..8], &selected_id[selected_id.len()-8..]));
                
                let selected_id_clone = selected_id.clone();
                ui.horizontal(|ui| {
                    if ui.button("🌐 Connect via WebRTC").clicked() {
                        self.connect_webrtc(&selected_id_clone);
                    }
                    
                    if ui.button("📡 Connect Direct (LAN)").clicked() {
                        self.connect_direct(&selected_id_clone);
                    }
                });
            } else {
                ui.colored_label(egui::Color32::GRAY, "Select a device to connect");
            }
        });
    }
    
    fn start_scanning(&mut self) {
        self.is_scanning = true;
        self.error_message = None;
        
        let discovery_manager = Arc::clone(&self.discovery_manager);
        
        tokio::spawn(async move {
            let mut manager = discovery_manager.lock().await;
            if let Err(e) = manager.start_discovery().await {
                error!("Failed to start device discovery: {}", e);
            }
        });
        
        info!("🔍 Started device discovery scanning");
    }
    
    fn stop_scanning(&mut self) {
        self.is_scanning = false;
        
        let discovery_manager = Arc::clone(&self.discovery_manager);
        
        tokio::spawn(async move {
            let mut manager = discovery_manager.lock().await;
            manager.stop_discovery();
        });
        
        info!("⏹ Stopped device discovery scanning");
    }
    
    fn refresh_devices(&mut self) {
        let discovery_manager = Arc::clone(&self.discovery_manager);
        
        tokio::spawn(async move {
            let manager = discovery_manager.lock().await;
            let devices = manager.get_devices();
            info!("🔄 Refreshed device list: {} devices found", devices.len());
        });
        
        self.last_scan_time = Some(std::time::Instant::now());
    }
    
    fn connect_webrtc(&mut self, device_id: &str) {
        info!("🌐 Initiating WebRTC connection to device: {}", device_id);
        // TODO: Integrate with streaming panel
        self.error_message = Some("WebRTC connection not yet integrated".to_string());
    }
    
    fn connect_direct(&mut self, device_id: &str) {
        info!("📡 Initiating direct LAN connection to device: {}", device_id);
        // TODO: Implement direct LAN connection
        self.error_message = Some("Direct LAN connection not yet implemented".to_string());
    }
    
    /// Update device list (called from main loop)
    pub fn update(&mut self) {
        if self.is_scanning {
            // In a real implementation, we'd update the device list from the discovery manager
            // For now, this is a placeholder
        }
    }
}
