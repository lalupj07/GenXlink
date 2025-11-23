use eframe::egui;
use genxlink_protocol::DeviceId;
use super::{NotificationManager, ConnectionDialog};
use super::remote_control_panel::RemoteControlPanel;
use super::premium_features::PremiumFeaturesPanel;
use super::permission_panel::PermissionPanel;
use super::screen_preview::ScreenPreviewPanel;
use genxlink_client_core::audio_streaming::AudioStreamManager;
use genxlink_client_core::localization::LocalizationManager;
use genxlink_client_core::theme::ThemeManager;

/// Main application state
pub struct GenXLinkApp {
    /// Current tab
    current_tab: Tab,
    
    /// Device list
    devices: Vec<DeviceInfo>,
    
    /// Application state
    state: AppState,
    
    /// Device ID
    device_id: DeviceId,
    
    /// Notification manager
    notification_manager: NotificationManager,
    
    /// Connection dialog
    connection_dialog: Option<ConnectionDialog>,
    
    /// Remote control panel
    remote_control_panel: RemoteControlPanel,
    
    /// Premium features panel
    premium_panel: PremiumFeaturesPanel,
    
    /// Permission panel
    permission_panel: PermissionPanel,
    
    /// Screen preview panel
    screen_preview: ScreenPreviewPanel,
    
    /// Audio manager
    audio_manager: AudioStreamManager,
    
    /// Localization manager
    localization: LocalizationManager,
    
    /// Theme manager
    theme_manager: ThemeManager,
}

/// Application tabs
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Devices,
    ScreenCapture,
    History,
    Settings,
    Premium,
}

/// Application state
#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Ready,
    Connecting(String),
    Connected(String),
    Error(String),
}

/// Device information for UI
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub device_type: DeviceType,
    pub ip_address: String,
    pub status: DeviceStatus,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Tablet,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Connecting,
}

impl Default for GenXLinkApp {
    fn default() -> Self {
        let mut app = Self {
            current_tab: Tab::Devices,
            devices: vec![
                // Sample devices for testing
                DeviceInfo {
                    id: DeviceId::new(),
                    name: "Desktop-PC".to_string(),
                    device_type: DeviceType::Desktop,
                    ip_address: "192.168.1.100".to_string(),
                    status: DeviceStatus::Online,
                    last_seen: Some(chrono::Utc::now()),
                },
                DeviceInfo {
                    id: DeviceId::new(),
                    name: "Laptop-Work".to_string(),
                    device_type: DeviceType::Laptop,
                    ip_address: "192.168.1.101".to_string(),
                    status: DeviceStatus::Online,
                    last_seen: Some(chrono::Utc::now()),
                },
                DeviceInfo {
                    id: DeviceId::new(),
                    name: "Phone-Android".to_string(),
                    device_type: DeviceType::Mobile,
                    ip_address: "192.168.1.102".to_string(),
                    status: DeviceStatus::Offline,
                    last_seen: Some(chrono::Utc::now() - chrono::Duration::hours(2)),
                },
            ],
            state: AppState::Ready,
            device_id: DeviceId::new(),
            notification_manager: NotificationManager::new(),
            connection_dialog: Some(ConnectionDialog::new()),
            remote_control_panel: RemoteControlPanel::new(),
            premium_panel: PremiumFeaturesPanel::new(),
            permission_panel: PermissionPanel::new(),
            screen_preview: ScreenPreviewPanel::new(),
            audio_manager: AudioStreamManager::new(),
            localization: LocalizationManager::new(),
            theme_manager: ThemeManager::new(),
        };
        
        // Show welcome notification
        app.notification_manager.success("Welcome to GenXLink", "Ready to connect to remote devices");
        
        app
    }
}

impl GenXLinkApp {
    /// Create a new application instance
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
    
    /// Get device icon emoji
    fn device_icon(device_type: DeviceType) -> &'static str {
        match device_type {
            DeviceType::Desktop => "🖥️",
            DeviceType::Laptop => "💻",
            DeviceType::Mobile => "📱",
            DeviceType::Tablet => "📱",
        }
    }
    
    /// Get status indicator
    fn status_indicator(status: DeviceStatus) -> (&'static str, egui::Color32) {
        match status {
            DeviceStatus::Online => ("●", egui::Color32::from_rgb(34, 197, 94)),
            DeviceStatus::Offline => ("●", egui::Color32::from_rgb(156, 163, 175)),
            DeviceStatus::Connecting => ("◐", egui::Color32::from_rgb(251, 191, 36)),
        }
    }
}

impl eframe::App for GenXLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("GenXLink");
                
                ui.separator();
                
                ui.selectable_value(&mut self.current_tab, Tab::Devices, "📱 Devices");
                ui.selectable_value(&mut self.current_tab, Tab::ScreenCapture, "📺 Screen Capture");
                ui.selectable_value(&mut self.current_tab, Tab::History, "📜 History");
                ui.selectable_value(&mut self.current_tab, Tab::Settings, "⚙ Settings");
                ui.selectable_value(&mut self.current_tab, Tab::Premium, "🌟 Premium");
            });
        });
        
        // Bottom panel with status
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                match &self.state {
                    AppState::Ready => {
                        ui.label("Status: Ready");
                    }
                    AppState::Connecting(device) => {
                        ui.spinner();
                        ui.label(format!("Connecting to {}...", device));
                    }
                    AppState::Connected(device) => {
                        ui.label(format!("✓ Connected to {}", device));
                    }
                    AppState::Error(msg) => {
                        ui.colored_label(egui::Color32::from_rgb(239, 68, 68), format!("⚠ {}", msg));
                    }
                }
                
                ui.separator();
                
                ui.label(format!("Device ID: {}", self.device_id.0.chars().take(8).collect::<String>()));
            });
        });
        
        // Central panel with content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Devices => self.show_devices_tab(ui),
                Tab::ScreenCapture => self.show_screen_capture_tab(ui),
                Tab::History => self.show_history_tab(ui),
                Tab::Settings => self.show_settings_tab(ui),
                Tab::Premium => self.show_premium_tab(ui),
            }
        });
        
        // Show connection dialog
        if let Some(ref mut dialog) = self.connection_dialog {
            let dialog_result = dialog.show(ctx);
            match dialog_result {
                super::DialogResult::Cancel => {
                    dialog.hide();
                    self.notification_manager.info("Connection Cancelled", "Connection attempt was cancelled");
                }
                super::DialogResult::Retry => {
                    self.notification_manager.info("Retrying", "Attempting to reconnect...");
                }
                super::DialogResult::Close => {
                    dialog.hide();
                }
                super::DialogResult::Continue => {}
            }
        }
        
        // Show notifications
        self.notification_manager.show(ctx);
    }
}

impl GenXLinkApp {
    fn show_devices_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Available Devices");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕ Connect to Device").clicked() {
                    // Show connection dialog
                    if let Some(ref mut dialog) = self.connection_dialog {
                        dialog.show_dialog();
                    }
                }
            });
        });
        ui.add_space(10.0);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for device in &self.devices {
                self.show_device_card(ui, device);
                ui.add_space(5.0);
            }
            
            if self.devices.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("No devices found");
                    ui.label("Devices will appear here when they come online");
                    ui.add_space(10.0);
                    if ui.button("➕ Connect to Device Manually").clicked() {
                        if let Some(ref mut dialog) = self.connection_dialog {
                            dialog.show_dialog();
                        }
                    }
                });
            }
        });
    }
    
    fn show_device_card(&self, ui: &mut egui::Ui, device: &DeviceInfo) {
        egui::Frame::group(ui.style())
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Device icon
                    ui.label(egui::RichText::new(Self::device_icon(device.device_type)).size(32.0));
                    
                    ui.add_space(10.0);
                    
                    // Device info
                    ui.vertical(|ui| {
                        ui.heading(&device.name);
                        
                        ui.horizontal(|ui| {
                            let (indicator, color) = Self::status_indicator(device.status);
                            ui.colored_label(color, indicator);
                            ui.label(&device.ip_address);
                            
                            if let Some(last_seen) = device.last_seen {
                                let duration = chrono::Utc::now() - last_seen;
                                if duration.num_minutes() < 1 {
                                    ui.label("• Just now");
                                } else if duration.num_hours() < 1 {
                                    ui.label(format!("• {} min ago", duration.num_minutes()));
                                } else {
                                    ui.label(format!("• {} hours ago", duration.num_hours()));
                                }
                            }
                        });
                    });
                    
                    // Spacer
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if device.status == DeviceStatus::Online {
                            if ui.button("Connect").clicked() {
                                // Show connection dialog
                                tracing::info!("Connect button clicked for: {}", device.name);
                                // Note: This is immutable borrow, so we log for now
                                // In a real implementation, we'd use message passing
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("Unavailable"));
                        }
                    });
                });
            });
    }
    
    fn show_history_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Connection History");
        ui.add_space(10.0);
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label("No connection history yet");
            ui.label("Your recent connections will appear here");
        });
    }
    
    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙ Settings");
        ui.add_space(10.0);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.collapsing("General", |ui| {
                ui.label("Device name:");
                ui.text_edit_singleline(&mut "My Device".to_string());
                
                ui.add_space(5.0);
                
                ui.checkbox(&mut true, "Start on boot");
                ui.checkbox(&mut true, "Minimize to tray");
                ui.checkbox(&mut true, "Show notifications");
            });
            
            ui.add_space(10.0);
            
            ui.collapsing("Connection", |ui| {
                ui.label("STUN Server:");
                ui.text_edit_singleline(&mut "stun:stun.l.google.com:19302".to_string());
                
                ui.add_space(5.0);
                
                ui.label("Connection timeout (seconds):");
                ui.add(egui::Slider::new(&mut 30, 10..=120));
            });
            
            ui.add_space(10.0);
            
            ui.collapsing("Display", |ui| {
                ui.label("Video quality:");
                egui::ComboBox::from_label("")
                    .selected_text("High")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut 0, 0, "Low");
                        ui.selectable_value(&mut 1, 1, "Medium");
                        ui.selectable_value(&mut 2, 2, "High");
                        ui.selectable_value(&mut 3, 3, "Ultra");
                    });
                
                ui.add_space(5.0);
                
                ui.label("Frame rate:");
                ui.add(egui::Slider::new(&mut 60, 15..=120).suffix(" FPS"));
            });
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    tracing::info!("Settings saved");
                }
                if ui.button("Reset to defaults").clicked() {
                    tracing::info!("Settings reset");
                }
            });
        });
    }
    
    fn show_screen_capture_tab(&mut self, ui: &mut egui::Ui) {
        self.screen_preview.ui(ui);
    }
    
    fn show_premium_tab(&mut self, ui: &mut egui::Ui) {
        let action = self.premium_panel.show(ui);
        
        // Handle premium actions
        match action {
            super::premium_features::PremiumAction::UpgradeToSolo => {
                self.notification_manager.info("Upgrade to Solo", "Redirecting to payment page...");
                tracing::info!("User clicked upgrade to Solo");
            }
            super::premium_features::PremiumAction::UpgradeToTeam => {
                self.notification_manager.info("Upgrade to Team", "Redirecting to payment page...");
                tracing::info!("User clicked upgrade to Team");
            }
            super::premium_features::PremiumAction::ContactSales => {
                self.notification_manager.info("Contact Sales", "Opening email client...");
                tracing::info!("User clicked contact sales");
            }
            super::premium_features::PremiumAction::None => {}
        }
    }
}
