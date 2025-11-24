use eframe::egui;
use genxlink_protocol::DeviceId;
use super::{NotificationManager, ConnectionDialog};
use super::remote_control_panel::RemoteControlPanel;
use super::premium_features::PremiumFeaturesPanel;
use super::permission_panel::PermissionPanel;
use super::screen_preview::ScreenPreviewPanel;
use super::streaming_panel::StreamingPanel;
use super::settings::SettingsPanel;
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
    
    /// Streaming panel
    streaming_panel: StreamingPanel,
    
    /// Settings panel
    settings_panel: SettingsPanel,
    
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
    Streaming,
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
            streaming_panel: StreamingPanel::new(),
            settings_panel: SettingsPanel::new(),
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
        // Material Design top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(255, 255, 255))
                .shadow(egui::epaint::Shadow {
                    offset: egui::vec2(0.0, 2.0),
                    blur: 8.0,
                    spread: 0.0,
                    color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 12),
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        
                        // Material Design app title
                        ui.label(egui::RichText::new("🚀 GenXLink")
                            .size(20.0)
                            .strong()
                            .color(egui::Color32::from_rgb(63, 81, 181)));
                        
                        ui.add_space(24.0);
                        
                        // Material Design tab buttons
                        let tab_style = |active: bool| {
                            if active {
                                egui::Button::new(
                                    egui::RichText::new("")
                                        .color(egui::Color32::from_rgb(63, 81, 181))
                                        .size(13.0)
                                        .strong()
                                )
                                    .fill(egui::Color32::from_rgb(232, 240, 254))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(120.0, 36.0))
                            } else {
                                egui::Button::new(
                                    egui::RichText::new("")
                                        .color(egui::Color32::from_rgb(97, 97, 97))
                                        .size(13.0)
                                )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(120.0, 36.0))
                            }
                        };
                        
                        // Devices tab
                        let devices_tab = if self.current_tab == Tab::Devices {
                            egui::Button::new(
                                egui::RichText::new("📱 Devices")
                                    .color(egui::Color32::from_rgb(63, 81, 181))
                                    .size(13.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(232, 240, 254))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        } else {
                            egui::Button::new(
                                egui::RichText::new("📱 Devices")
                                    .color(egui::Color32::from_rgb(97, 97, 97))
                                    .size(13.0)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        };
                        
                        if ui.add(devices_tab).clicked() {
                            self.current_tab = Tab::Devices;
                        }
                        
                        ui.add_space(4.0);
                        
                        // Screen Capture tab
                        let screen_capture_tab = if self.current_tab == Tab::ScreenCapture {
                            egui::Button::new(
                                egui::RichText::new("📺 Screen Capture")
                                    .color(egui::Color32::from_rgb(63, 81, 181))
                                    .size(13.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(232, 240, 254))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        } else {
                            egui::Button::new(
                                egui::RichText::new("📺 Screen Capture")
                                    .color(egui::Color32::from_rgb(97, 97, 97))
                                    .size(13.0)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        };
                        
                        if ui.add(screen_capture_tab).clicked() {
                            self.current_tab = Tab::ScreenCapture;
                        }
                        
                        ui.add_space(4.0);
                        
                        // Streaming tab
                        let streaming_tab = if self.current_tab == Tab::Streaming {
                            egui::Button::new(
                                egui::RichText::new("🌐 Streaming")
                                    .color(egui::Color32::from_rgb(63, 81, 181))
                                    .size(13.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(232, 240, 254))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        } else {
                            egui::Button::new(
                                egui::RichText::new("🌐 Streaming")
                                    .color(egui::Color32::from_rgb(97, 97, 97))
                                    .size(13.0)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        };
                        
                        if ui.add(streaming_tab).clicked() {
                            self.current_tab = Tab::Streaming;
                        }
                        
                        ui.add_space(4.0);
                        
                        // History tab
                        let history_tab = if self.current_tab == Tab::History {
                            egui::Button::new(
                                egui::RichText::new("📜 History")
                                    .color(egui::Color32::from_rgb(63, 81, 181))
                                    .size(13.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(232, 240, 254))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        } else {
                            egui::Button::new(
                                egui::RichText::new("📜 History")
                                    .color(egui::Color32::from_rgb(97, 97, 97))
                                    .size(13.0)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        };
                        
                        if ui.add(history_tab).clicked() {
                            self.current_tab = Tab::History;
                        }
                        
                        ui.add_space(4.0);
                        
                        // Settings tab
                        let settings_tab = if self.current_tab == Tab::Settings {
                            egui::Button::new(
                                egui::RichText::new("⚙ Settings")
                                    .color(egui::Color32::from_rgb(63, 81, 181))
                                    .size(13.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(232, 240, 254))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        } else {
                            egui::Button::new(
                                egui::RichText::new("⚙ Settings")
                                    .color(egui::Color32::from_rgb(97, 97, 97))
                                    .size(13.0)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        };
                        
                        if ui.add(settings_tab).clicked() {
                            self.current_tab = Tab::Settings;
                        }
                        
                        ui.add_space(4.0);
                        
                        // Premium tab
                        let premium_tab = if self.current_tab == Tab::Premium {
                            egui::Button::new(
                                egui::RichText::new("🌟 Premium")
                                    .color(egui::Color32::from_rgb(63, 81, 181))
                                    .size(13.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(232, 240, 254))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        } else {
                            egui::Button::new(
                                egui::RichText::new("🌟 Premium")
                                    .color(egui::Color32::from_rgb(97, 97, 97))
                                    .size(13.0)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        };
                        
                        if ui.add(premium_tab).clicked() {
                            self.current_tab = Tab::Premium;
                        }
                        
                        ui.add_space(16.0);
                    });
                    ui.add_space(12.0);
                });
        });
        
        // Material Design bottom panel with status
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(248, 249, 250))
                .shadow(egui::epaint::Shadow {
                    offset: egui::vec2(0.0, -1.0),
                    blur: 4.0,
                    spread: 0.0,
                    color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 8),
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        
                        // Material Design status display
                        match &self.state {
                            AppState::Ready => {
                                ui.label(egui::RichText::new("● Ready")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(76, 175, 80)));
                            }
                            AppState::Connecting(device) => {
                                ui.spinner();
                                ui.label(egui::RichText::new(format!("◐ Connecting to {}...", device))
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(255, 152, 0)));
                            }
                            AppState::Connected(device) => {
                                ui.label(egui::RichText::new(format!("● Connected to {}", device))
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(76, 175, 80)));
                            }
                            AppState::Error(msg) => {
                                ui.label(egui::RichText::new(format!("● {}", msg))
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(244, 67, 54)));
                            }
                        }
                        
                        ui.add_space(24.0);
                        
                        // Material Design separator
                        ui.add(egui::Separator::default().horizontal());
                        
                        ui.add_space(24.0);
                        
                        // Material Design device ID
                        ui.label(egui::RichText::new("🔗 Device ID:")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(117, 117, 117)));
                        ui.label(egui::RichText::new(self.device_id.0.chars().take(8).collect::<String>())
                            .size(12.0)
                            .strong()
                            .color(egui::Color32::from_rgb(33, 33, 33)));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(16.0);
                        });
                    });
                    ui.add_space(8.0);
                });
        });
        
        // Material Design central panel with content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(250, 250, 250))
                .show(ui, |ui| {
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        
                        // Material Design content area
                        match self.current_tab {
                            Tab::Devices => self.show_devices_tab(ui),
                            Tab::ScreenCapture => self.show_screen_capture_tab(ui),
                            Tab::Streaming => self.show_streaming_tab(ui),
                            Tab::History => self.show_history_tab(ui),
                            Tab::Settings => self.show_settings_tab(ui),
                            Tab::Premium => self.show_premium_tab(ui),
                        }
                        
                        ui.add_space(16.0);
                    });
                    ui.add_space(16.0);
                });
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
        // Material Design devices header
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 12),
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    
                    ui.label(egui::RichText::new("📱 Available Devices")
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::from_rgb(33, 33, 33)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);
                        
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("➕ Connect to Device")
                                    .color(egui::Color32::WHITE)
                                    .size(13.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(63, 81, 181))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(160.0, 36.0))
                        ).clicked() {
                            // Show connection dialog
                            if let Some(ref mut dialog) = self.connection_dialog {
                                dialog.show_dialog();
                            }
                        }
                    });
                });
                ui.add_space(16.0);
            });
        
        ui.add_space(16.0);
        
        // Material Design devices list
        egui::ScrollArea::vertical().show(ui, |ui| {
            for device in &self.devices {
                self.show_device_card(ui, device);
                ui.add_space(12.0);
            }
            
            if self.devices.is_empty() {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(255, 255, 255))
                    .rounding(egui::Rounding::same(12.0))
                    .shadow(egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 2.0),
                        blur: 8.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 12),
                    })
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            
                            ui.label(egui::RichText::new("📱")
                                .size(48.0)
                                .color(egui::Color32::from_rgb(189, 189, 189)));
                            
                            ui.add_space(16.0);
                            
                            ui.label(egui::RichText::new("No devices found")
                                .size(16.0)
                                .strong()
                                .color(egui::Color32::from_rgb(117, 117, 117)));
                            
                            ui.add_space(8.0);
                            
                            ui.label(egui::RichText::new("Devices will appear here when they come online")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(158, 158, 158)));
                            
                            ui.add_space(24.0);
                            
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("➕ Connect to Device Manually")
                                        .color(egui::Color32::WHITE)
                                        .size(13.0)
                                        .strong()
                                )
                                    .fill(egui::Color32::from_rgb(63, 81, 181))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(200.0, 36.0))
                            ).clicked() {
                                if let Some(ref mut dialog) = self.connection_dialog {
                                    dialog.show_dialog();
                                }
                            }
                            
                            ui.add_space(40.0);
                        });
                    });
            }
        });
    }
    
    fn show_device_card(&self, ui: &mut egui::Ui, device: &DeviceInfo) {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 12),
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    
                    // Device icon
                    ui.label(egui::RichText::new(Self::device_icon(device.device_type))
                        .size(32.0));
                    
                    ui.add_space(16.0);
                    
                    // Device info
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&device.name)
                            .size(16.0)
                            .strong()
                            .color(egui::Color32::from_rgb(33, 33, 33)));
                        
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            let (indicator, color) = Self::status_indicator(device.status);
                            ui.label(egui::RichText::new(indicator)
                                .size(12.0)
                                .color(color));
                            
                            ui.add_space(8.0);
                            
                            ui.label(egui::RichText::new(&device.ip_address)
                                .size(12.0)
                                .color(egui::Color32::from_rgb(117, 117, 117)));
                            
                            if let Some(last_seen) = device.last_seen {
                                let duration = chrono::Utc::now() - last_seen;
                                let time_text = if duration.num_minutes() < 1 {
                                    "Just now".to_string()
                                } else if duration.num_hours() < 1 {
                                    format!("{} min ago", duration.num_minutes())
                                } else {
                                    format!("{} hours ago", duration.num_hours())
                                };
                                
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new(format!("• {}", time_text))
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(158, 158, 158)));
                            }
                        });
                    });
                    
                    // Spacer and connect button
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);
                        
                        if device.status == DeviceStatus::Online {
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("Connect")
                                        .color(egui::Color32::WHITE)
                                        .size(12.0)
                                        .strong()
                                )
                                    .fill(egui::Color32::from_rgb(76, 175, 80))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(80.0, 32.0))
                            ).clicked() {
                                // Show connection dialog
                                tracing::info!("Connect button clicked for: {}", device.name);
                                // Note: This is immutable borrow, so we log for now
                                // In a real implementation, we'd use message passing
                            }
                        } else {
                            ui.add(
                                egui::Button::new(
                                    egui::RichText::new("Unavailable")
                                        .color(egui::Color32::from_rgb(117, 117, 117))
                                        .size(12.0)
                                )
                                    .fill(egui::Color32::from_rgb(245, 245, 245))
                                    .rounding(egui::Rounding::same(8.0))
                                    .min_size(egui::vec2(100.0, 32.0))
                            );
                        }
                    });
                    
                    ui.add_space(16.0);
                });
                ui.add_space(16.0);
            });
    }
    
    fn show_history_tab(&mut self, ui: &mut egui::Ui) {
        // Material Design history header
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 12),
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    
                    ui.label(egui::RichText::new("📜 Connection History")
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::from_rgb(33, 33, 33)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);
                        
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("🗑️ Clear History")
                                    .color(egui::Color32::from_rgb(117, 117, 117))
                                    .size(13.0)
                            )
                                .fill(egui::Color32::TRANSPARENT)
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(120.0, 36.0))
                        ).clicked() {
                            // Clear history logic
                        }
                    });
                });
                ui.add_space(16.0);
            });
        
        ui.add_space(16.0);
        
        // Material Design empty state
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .rounding(egui::Rounding::same(12.0))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 12),
            })
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    
                    ui.label(egui::RichText::new("📜")
                        .size(48.0)
                        .color(egui::Color32::from_rgb(189, 189, 189)));
                    
                    ui.add_space(16.0);
                    
                    ui.label(egui::RichText::new("No connection history yet")
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::from_rgb(117, 117, 117)));
                    
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("Your connection history will appear here")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(158, 158, 158)));
                    
                    ui.add_space(60.0);
                });
            });
    }
    
    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        let action = self.settings_panel.show(ui);
        
        // Handle settings actions
        match action {
            super::settings::SettingsAction::OpenLogFolder => {
                if let Err(e) = std::fs::create_dir_all("logs") {
                    tracing::error!("Failed to create log folder: {}", e);
                }
                if let Err(e) = open::that("logs") {
                    tracing::error!("Failed to open log folder: {}", e);
                    self.notification_manager.error("Error", "Could not open log folder");
                }
            }
            super::settings::SettingsAction::ViewLicense => {
                if let Err(e) = open::that("LICENSE") {
                    tracing::error!("Failed to open license file: {}", e);
                    self.notification_manager.error("Error", "Could not open license file");
                }
            }
            super::settings::SettingsAction::OpenDocumentation => {
                if let Err(e) = open::that("README.md") {
                    tracing::error!("Failed to open documentation: {}", e);
                    self.notification_manager.error("Error", "Could not open documentation");
                }
            }
            super::settings::SettingsAction::None => {}
        }
    }
    
    fn show_screen_capture_tab(&mut self, ui: &mut egui::Ui) {
        self.screen_preview.ui(ui);
    }
    
    fn show_streaming_tab(&mut self, ui: &mut egui::Ui) {
        self.streaming_panel.ui(ui);
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
