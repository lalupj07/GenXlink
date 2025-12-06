use eframe::egui;
use genxlink_protocol::DeviceId;
use super::{NotificationManager};
use super::connection_dialog::ConnectionDialog;
use super::remote_control_panel::RemoteControlPanel;
use super::premium_features::PremiumFeaturesPanel;
use super::permission_panel::PermissionPanel;
use super::screen_preview::ScreenPreviewPanel;
use super::streaming_panel::StreamingPanel;
use super::devices::DevicesPanel;
use super::settings::SettingsPanel;
use super::auth_panel::{AuthPanel, AuthPanelState};
use genxlink_client_core::audio_streaming::AudioStreamManager;
use genxlink_client_core::localization::LocalizationManager;
use genxlink_client_core::theme::ThemeManager;
use genxlink_client_core::{
    config::ServerConfig,
    auth_service::AuthService,
};
use genxlink_client_core::session_manager::{SessionManager, SessionConfig};
use genxlink_client_core::installation_id::get_installation_id;
use genxlink_client_core::connection_id::get_connection_id;

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
    
    /// Devices panel
    devices_panel: DevicesPanel,
    
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
    
    /// Authentication panel
    auth_panel: AuthPanel,
    
    /// Session manager
    session_manager: SessionManager,
    
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
    Authentication,
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
            devices_panel: DevicesPanel::new(),
            remote_control_panel: RemoteControlPanel::new(),
            premium_panel: PremiumFeaturesPanel::new(),
            permission_panel: PermissionPanel::new(),
            screen_preview: ScreenPreviewPanel::new(),
            streaming_panel: StreamingPanel::new(),
            settings_panel: SettingsPanel::new(),
            auth_panel: AuthPanel::new(),
            session_manager: SessionManager::new(
                AuthService::new(
                    ServerConfig::default().api_server_url,
                    "your-anon-key".to_string(),
                ),
                SessionConfig::default(),
            ),
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
        // Update async authentication operations
        let server_config = ServerConfig::default();
        let mut auth_service = AuthService::new(
            server_config.api_server_url,
            "your-anon-key".to_string(),
        );
        self.auth_panel.update_async_operations(&mut auth_service);
        
        // Top panel with YOUR ID prominently displayed
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("GenXLink");
                
                ui.separator();
                
                // Show YOUR CONNECTION ID prominently
                let conn_id = get_connection_id();
                ui.label("Your ID:");
                ui.add(egui::Label::new(
                    egui::RichText::new(&conn_id.display_id)
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::from_rgb(0, 212, 255))
                ));
                if ui.small_button("📋 Copy").clicked() {
                    ui.output_mut(|o| o.copied_text = conn_id.display_id.clone());
                }
                
                ui.separator();
                
                ui.selectable_value(&mut self.current_tab, Tab::Devices, "📱 Devices");
                ui.selectable_value(&mut self.current_tab, Tab::ScreenCapture, "📺 Screen Capture");
                ui.selectable_value(&mut self.current_tab, Tab::Streaming, "🌐 WebRTC Streaming");
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
                
                // Show installation ID (unique per installation)
                let install_id = get_installation_id();
                ui.label(format!("📋 {}", install_id.display_id()));
                
                ui.separator();
                
                ui.label(format!("Device ID: {}", self.device_id.0.chars().take(8).collect::<String>()));
            });
        });
        
        // Central panel with content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Authentication => self.show_authentication_tab(ui),
                Tab::Devices => self.show_devices_tab(ui),
                Tab::ScreenCapture => self.show_screen_capture_tab(ui),
                Tab::Streaming => self.show_streaming_tab(ui),
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
    fn show_authentication_tab(&mut self, ui: &mut egui::Ui) {
        // Update auth panel with async operations
        let server_config = ServerConfig::default();
        let mut auth_service = AuthService::new(
            server_config.api_server_url,
            "your-anon-key".to_string(),
        );
        
        // Show authentication panel
        let auth_action = self.auth_panel.ui(ui, &mut auth_service);
        
        // Handle authentication actions
        match auth_action {
            super::auth_panel::AuthAction::LoginSuccess => {
                self.notification_manager.success("Login Successful", "Welcome back to GenXLink");
                // Switch to devices tab after successful login
                self.current_tab = Tab::Devices;
            }
            super::auth_panel::AuthAction::RegisterSuccess => {
                self.notification_manager.success("Registration Successful", "Please check your email to verify your account");
            }
            super::auth_panel::AuthAction::Logout => {
                self.notification_manager.info("Logged Out", "You have been successfully logged out");
                // TODO: Terminate session asynchronously
                // let _ = self.session_manager.terminate_session().await;
                // Switch to authentication tab
                self.current_tab = Tab::Authentication;
            }
            super::auth_panel::AuthAction::None => {}
        }
    }
    
    fn show_devices_tab(&mut self, ui: &mut egui::Ui) {
        // Update devices panel
        self.devices_panel.update();
        
        // Show devices panel UI
        self.devices_panel.ui(ui);
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
            ui.label("Your connection history will appear here");
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
            super::settings::SettingsAction::ResetToDefaults => {
                self.settings_panel = super::settings::SettingsPanel::new();
                self.notification_manager.success("Settings", "Reset to default settings");
            }
            super::settings::SettingsAction::ExportSettings => {
                // TODO: Implement settings export functionality
                self.notification_manager.info("Settings", "Export functionality coming soon");
            }
            super::settings::SettingsAction::ImportSettings => {
                // TODO: Implement settings import functionality
                self.notification_manager.info("Settings", "Import functionality coming soon");
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
            super::premium_features::PremiumAction::UpgradeToEnterprise => {
                self.notification_manager.info("Upgrade to Enterprise", "Redirecting to enterprise sales page...");
                tracing::info!("User clicked upgrade to Enterprise");
            }
            super::premium_features::PremiumAction::ContactSales => {
                self.notification_manager.info("Contact Sales", "Opening email client...");
                tracing::info!("User clicked contact sales");
            }
            super::premium_features::PremiumAction::None => {}
        }
    }
}
