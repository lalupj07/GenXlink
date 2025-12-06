use eframe::egui;
use genxlink_client_core::device_registry::{DeviceRegistryService, DeviceSyncStatus};
use genxlink_client_core::database::{DeviceRegistry, DeviceType};

/// Device management panel UI
#[derive(Default)]
pub struct DeviceManagementPanel {
    state: DevicePanelState,
    registration_form: DeviceRegistrationForm,
    update_form: DeviceUpdateForm,
    devices: Vec<DeviceRegistry>,
    selected_device: Option<String>,
    error_message: String,
    success_message: String,
    loading: bool,
    filter_online_only: bool,
    search_query: String,
    sync_status: Option<DeviceSyncStatus>,
}

/// Device panel state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevicePanelState {
    List,
    Register,
    Details,
    Update,
    Loading,
}

impl Default for DevicePanelState {
    fn default() -> Self {
        Self::List
    }
}

/// Device registration form
#[derive(Default, Clone)]
pub struct DeviceRegistrationForm {
    pub device_name: String,
    pub device_type: DeviceType,
    pub os_version: String,
    pub ip_address: String,
    pub mac_address: String,
    pub screen_sharing: bool,
    pub remote_control: bool,
    pub file_transfer: bool,
    pub audio_streaming: bool,
    pub clipboard_access: bool,
    pub encryption: bool,
    pub multi_monitor: bool,
    pub max_width: u32,
    pub max_height: u32,
}

/// Device update form
#[derive(Default, Clone)]
pub struct DeviceUpdateForm {
    pub device_name: String,
    pub ip_address: String,
    pub screen_sharing: bool,
    pub remote_control: bool,
    pub file_transfer: bool,
    pub audio_streaming: bool,
    pub clipboard_access: bool,
    pub encryption: bool,
    pub multi_monitor: bool,
    pub max_width: u32,
    pub max_height: u32,
}

impl DeviceManagementPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the device management panel
    pub fn ui(&mut self, ui: &mut egui::Ui, device_registry: &DeviceRegistryService) {
        ui.horizontal(|ui| {
            ui.heading("📱 Device Management");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_devices(device_registry);
                }
                
                if ui.button("📝 Register Device").clicked() {
                    self.state = DevicePanelState::Register;
                    self.clear_messages();
                }
            });
        });

        ui.add_space(8.0);

        // Tab selection
        ui.horizontal(|ui| {
            let selected = self.state == DevicePanelState::List;
            if ui.selectable_label(selected, "📋 Device List").clicked() {
                self.state = DevicePanelState::List;
                self.clear_messages();
            }
            
            if ui.button("📝 Register Device").clicked() {
                self.state = DevicePanelState::Register;
                self.clear_messages();
            }
        });

        ui.add_space(8.0);
        ui.separator();

        // Show error/success messages
        if !self.error_message.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error_message);
            ui.add_space(8.0);
        }
        
        if !self.success_message.is_empty() {
            ui.colored_label(egui::Color32::GREEN, &self.success_message);
            ui.add_space(8.0);
        }

        match self.state {
            DevicePanelState::List => self.render_device_list(ui),
            DevicePanelState::Register => self.render_registration_panel(ui, device_registry),
            DevicePanelState::Details => self.render_device_details(ui),
            DevicePanelState::Update => self.render_update_panel(ui, device_registry),
            DevicePanelState::Loading => self.render_loading_panel(ui),
        }
    }

    /// Render device list
    fn render_device_list(&mut self, ui: &mut egui::Ui) {
        // Search and filter
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.text_edit_singleline(&mut self.search_query);
            
            ui.checkbox(&mut self.filter_online_only, "Online only");
        });

        ui.add_space(8.0);

        // Device list
        if self.devices.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("No devices registered");
                ui.label("Click 'Register Device' to add your first device");
                ui.add_space(20.0);
            });
        } else {
            // Filter devices first to avoid borrowing issues
            let filtered_devices: Vec<DeviceRegistry> = self.devices.iter()
                .filter(|device| {
                    let matches_search = self.search_query.is_empty() 
                        || device.device_name.to_lowercase().contains(&self.search_query.to_lowercase())
                        || device.device_id.to_lowercase().contains(&self.search_query.to_lowercase());
                    
                    let matches_filter = !self.filter_online_only || device.is_online;
                    
                    matches_search && matches_filter
                })
                .cloned()
                .collect();

            for device in &filtered_devices {
                self.render_device_item(ui, device);
                ui.add_space(4.0);
            }
        }

        // Sync status
        if let Some(sync_status) = &self.sync_status {
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("🔄 Sync Status:");
                
                if sync_status.sync_success {
                    ui.colored_label(egui::Color32::GREEN, "✓ Sync successful");
                } else {
                    ui.colored_label(egui::Color32::RED, "✗ Sync failed");
                }
            });
        }
    }

    /// Render a single device item
    fn render_device_item(&mut self, ui: &mut egui::Ui, device: &DeviceRegistry) {
        let is_selected = self.selected_device.as_ref() == Some(&device.device_id);
        
        let frame = egui::Frame::none()
            .fill(if is_selected {
                egui::Color32::from_rgb(230, 240, 250)
            } else {
                ui.visuals().faint_bg_color
            })
            .stroke(if is_selected {
                egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE)
            } else {
                egui::Stroke::NONE
            })
            .rounding(6.0)
            .inner_margin(12.0);

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Device status icon
                let status_icon = if device.is_online { "🟢" } else { "🔴" };
                ui.label(egui::RichText::new(status_icon).size(20.0));

                ui.add_space(8.0);

                // Device info
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&device.device_name).strong());
                        ui.label(format!("({})", &device.device_id[..8]));
                        
                        // Device type badge
                        let type_color = match device.device_type {
                            DeviceType::Desktop => egui::Color32::BLUE,
                            DeviceType::Laptop => egui::Color32::GREEN,
                            DeviceType::Server => egui::Color32::RED,
                            DeviceType::Mobile => egui::Color32::YELLOW,
                            DeviceType::Tablet => egui::Color32::from_rgb(255, 165, 0),
                            DeviceType::IoT => egui::Color32::from_rgb(128, 0, 128),
                            DeviceType::Unknown => egui::Color32::GRAY,
                        };
                        ui.colored_label(type_color, format!("{:?}", device.device_type));
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("OS: {}", device.os_version));
                        ui.label("•");
                        ui.label(format!("IP: {}", device.ip_address));
                        
                        if let Some(mac) = &device.mac_address {
                            ui.label("•");
                            ui.label(format!("MAC: {}", mac));
                        }
                    });

                    // Capabilities
                    ui.horizontal(|ui| {
                        if device.capabilities.screen_sharing {
                            ui.colored_label(egui::Color32::GREEN, "🖥️ Screen");
                        }
                        if device.capabilities.remote_control {
                            ui.colored_label(egui::Color32::GREEN, "🎮 Control");
                        }
                        if device.capabilities.file_transfer {
                            ui.colored_label(egui::Color32::GREEN, "📁 Files");
                        }
                        if device.capabilities.audio_streaming {
                            ui.colored_label(egui::Color32::GREEN, "🔊 Audio");
                        }
                    });

                    // Last seen
                    if let Ok(duration) = std::time::SystemTime::now().duration_since(device.last_seen) {
                        ui.label(format!("Last seen: {} seconds ago", duration.as_secs()));
                    }
                });

                // Action buttons
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑️ Delete").clicked() {
                        self.handle_delete_device(&device.device_id);
                    }
                    
                    if ui.button("✏️ Edit").clicked() {
                        self.selected_device = Some(device.device_id.clone());
                        self.load_device_for_update(device);
                        self.state = DevicePanelState::Update;
                    }
                    
                    if ui.button("👁️ Details").clicked() {
                        self.selected_device = Some(device.device_id.clone());
                        self.state = DevicePanelState::Details;
                    }
                });
            });
        });
    }

    /// Render registration panel
    fn render_registration_panel(&mut self, ui: &mut egui::Ui, _device_registry: &DeviceRegistryService) {
        ui.vertical_centered(|ui| {
            ui.heading("📝 Register New Device");
            ui.add_space(16.0);
        });

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Device Name:");
                    ui.text_edit_singleline(&mut self.registration_form.device_name);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Device Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.registration_form.device_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.registration_form.device_type, DeviceType::Desktop, "Desktop");
                            ui.selectable_value(&mut self.registration_form.device_type, DeviceType::Laptop, "Laptop");
                            ui.selectable_value(&mut self.registration_form.device_type, DeviceType::Server, "Server");
                            ui.selectable_value(&mut self.registration_form.device_type, DeviceType::Mobile, "Mobile");
                            ui.selectable_value(&mut self.registration_form.device_type, DeviceType::Tablet, "Tablet");
                            ui.selectable_value(&mut self.registration_form.device_type, DeviceType::IoT, "IoT");
                            ui.selectable_value(&mut self.registration_form.device_type, DeviceType::Unknown, "Unknown");
                        });
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("OS Version:");
                    ui.text_edit_singleline(&mut self.registration_form.os_version);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("IP Address:");
                    ui.text_edit_singleline(&mut self.registration_form.ip_address);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("MAC Address:");
                    ui.text_edit_singleline(&mut self.registration_form.mac_address);
                });
                
                ui.add_space(16.0);
                
                ui.heading("Capabilities");
                
                ui.add_space(8.0);
                
                ui.checkbox(&mut self.registration_form.screen_sharing, "Screen Sharing");
                ui.checkbox(&mut self.registration_form.remote_control, "Remote Control");
                ui.checkbox(&mut self.registration_form.file_transfer, "File Transfer");
                ui.checkbox(&mut self.registration_form.audio_streaming, "Audio Streaming");
                ui.checkbox(&mut self.registration_form.clipboard_access, "Clipboard Access");
                ui.checkbox(&mut self.registration_form.encryption, "Encryption");
                ui.checkbox(&mut self.registration_form.multi_monitor, "Multi-Monitor Support");
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Max Resolution:");
                    ui.add(egui::Slider::new(&mut self.registration_form.max_width, 640..=3840).text("Width"));
                    ui.add(egui::Slider::new(&mut self.registration_form.max_height, 480..=2160).text("Height"));
                });
                
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    if ui.button("📝 Register Device").clicked() {
                        self.handle_device_registration(_device_registry);
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        self.state = DevicePanelState::List;
                        self.clear_forms();
                    }
                });
            });
    }

    /// Render device details
    fn render_device_details(&mut self, ui: &mut egui::Ui) {
        if let Some(_device_id) = &self.selected_device {
            ui.vertical_centered(|ui| {
                ui.heading("📱 Device Details");
                ui.add_space(16.0);
                
                ui.label("Device details would be shown here");
                
                ui.add_space(16.0);
                
                if ui.button("🔙 Back to List").clicked() {
                    self.state = DevicePanelState::List;
                }
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.label("No device selected");
                if ui.button("🔙 Back to List").clicked() {
                    self.state = DevicePanelState::List;
                }
            });
        }
    }

    /// Render update panel
    fn render_update_panel(&mut self, ui: &mut egui::Ui, _device_registry: &DeviceRegistryService) {
        ui.vertical_centered(|ui| {
            ui.heading("✏️ Update Device");
            ui.add_space(16.0);
        });

        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(8.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Device Name:");
                    ui.text_edit_singleline(&mut self.update_form.device_name);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("IP Address:");
                    ui.text_edit_singleline(&mut self.update_form.ip_address);
                });
                
                ui.add_space(16.0);
                
                ui.heading("Capabilities");
                
                ui.add_space(8.0);
                
                ui.checkbox(&mut self.update_form.screen_sharing, "Screen Sharing");
                ui.checkbox(&mut self.update_form.remote_control, "Remote Control");
                ui.checkbox(&mut self.update_form.file_transfer, "File Transfer");
                ui.checkbox(&mut self.update_form.audio_streaming, "Audio Streaming");
                ui.checkbox(&mut self.update_form.clipboard_access, "Clipboard Access");
                ui.checkbox(&mut self.update_form.encryption, "Encryption");
                ui.checkbox(&mut self.update_form.multi_monitor, "Multi-Monitor Support");
                
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    if ui.button("💾 Update Device").clicked() {
                        self.handle_device_update(_device_registry);
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        self.state = DevicePanelState::List;
                        self.clear_forms();
                    }
                });
            });
    }

    /// Render loading panel
    fn render_loading_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.spinner();
            ui.add_space(16.0);
            ui.label("Loading...");
        });
    }

    /// Handle device registration
    fn handle_device_registration(&mut self, _device_registry: &DeviceRegistryService) {
        if self.registration_form.device_name.is_empty() || self.registration_form.os_version.is_empty() {
            self.error_message = "Please fill in all required fields".to_string();
            return;
        }

        self.loading = true;
        self.state = DevicePanelState::Loading;

        // Simulate registration
        self.state = DevicePanelState::List;
        self.clear_messages();
        self.success_message = "Device registered successfully!".to_string();
        self.loading = false;
    }

    /// Handle device update
    fn handle_device_update(&mut self, _device_registry: &DeviceRegistryService) {
        if self.update_form.device_name.is_empty() {
            self.error_message = "Device name is required".to_string();
            return;
        }

        self.loading = true;
        self.state = DevicePanelState::Loading;

        // Simulate update
        self.state = DevicePanelState::List;
        self.clear_messages();
        self.success_message = "Device updated successfully!".to_string();
        self.loading = false;
    }

    /// Handle device deletion
    fn handle_delete_device(&mut self, device_id: &str) {
        // Simulate deletion
        self.devices.retain(|d| d.device_id != device_id);
        self.success_message = "Device deleted successfully!".to_string();
    }

    /// Load device data for update
    fn load_device_for_update(&mut self, device: &DeviceRegistry) {
        self.update_form.device_name = device.device_name.clone();
        self.update_form.ip_address = device.ip_address.clone();
        self.update_form.screen_sharing = device.capabilities.screen_sharing;
        self.update_form.remote_control = device.capabilities.remote_control;
        self.update_form.file_transfer = device.capabilities.file_transfer;
        self.update_form.audio_streaming = device.capabilities.audio_streaming;
        self.update_form.clipboard_access = device.capabilities.clipboard_access;
        self.update_form.encryption = device.capabilities.encryption;
        self.update_form.multi_monitor = device.capabilities.multi_monitor;
        self.update_form.max_width = device.capabilities.max_resolution.0;
        self.update_form.max_height = device.capabilities.max_resolution.1;
    }

    /// Refresh devices list
    fn refresh_devices(&mut self, _device_registry: &DeviceRegistryService) {
        // Simulate refresh
        self.success_message = "Device list refreshed!".to_string();
    }

    /// Clear all forms
    fn clear_forms(&mut self) {
        self.registration_form = DeviceRegistrationForm::default();
        self.update_form = DeviceUpdateForm::default();
    }

    /// Clear messages
    fn clear_messages(&mut self) {
        self.error_message.clear();
        self.success_message.clear();
    }

    /// Set devices list
    pub fn set_devices(&mut self, devices: Vec<DeviceRegistry>) {
        self.devices = devices;
    }

    /// Get current panel state
    pub fn get_state(&self) -> &DevicePanelState {
        &self.state
    }

    /// Set panel state
    pub fn set_state(&mut self, state: DevicePanelState) {
        self.state = state;
        self.clear_messages();
    }
}
