use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use eframe::{
    egui::{self, Context, CentralPanel, SidePanel, TopBottomPanel, ScrollArea, Grid, Vec2},
    epi,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::webrtc::media_manager::{MediaManager, MediaSessionConfig, ScreenShareConfig, AudioConfig, FileTransferConfig};
use crate::crypto::e2e_encryption::EndToEndEncryption;
use crate::ui::{
    settings_panel::SettingsPanel,
    status_panel::StatusPanel,
    connection_panel::ConnectionPanel,
    file_transfer_panel::FileTransferPanel,
    chat_panel::ChatPanel,
    notifications::NotificationManager,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    pub is_connected: bool,
    pub session_id: Option<Uuid>,
    pub remote_user: Option<String>,
    pub connection_quality: ConnectionQuality,
    pub bandwidth: BandwidthStats,
    pub latency: u32,
    pub uptime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub upload_speed: f64, // Mbps
    pub download_speed: f64, // Mbps
    pub total_uploaded: u64, // MB
    pub total_downloaded: u64, // MB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub status: UserOnlineStatus,
    pub avatar_url: Option<String>,
    pub capabilities: UserCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserOnlineStatus {
    Online,
    Away,
    Busy,
    Invisible,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCapabilities {
    pub can_screen_share: bool,
    pub can_audio_share: bool,
    pub can_file_transfer: bool,
    pub can_remote_control: bool,
    pub max_file_size: u64,
}

pub struct MainWindow {
    // Core components
    media_manager: Arc<MediaManager>,
    encryption: Arc<EndToEndEncryption>,
    
    // UI Panels
    settings_panel: Arc<SettingsPanel>,
    status_panel: Arc<StatusPanel>,
    connection_panel: Arc<ConnectionPanel>,
    file_transfer_panel: Arc<FileTransferPanel>,
    chat_panel: Arc<ChatPanel>,
    notification_manager: Arc<NotificationManager>,
    
    // State
    connection_state: Arc<RwLock<ConnectionState>>,
    current_user: Arc<RwLock<Option<UserStatus>>>,
    active_sessions: Arc<RwLock<HashMap<Uuid, MediaSessionConfig>>>,
    
    // UI State
    show_settings: bool,
    show_status: bool,
    show_chat: bool,
    show_file_transfers: bool,
    selected_session: Option<Uuid>,
    
    // Theme
    theme: AppTheme,
    dark_mode: bool,
}

#[derive(Debug, Clone)]
pub struct AppTheme {
    pub primary_color: egui::Color32,
    pub secondary_color: egui::Color32,
    pub background_color: egui::Color32,
    pub surface_color: egui::Color32,
    pub text_color: egui::Color32,
    pub error_color: egui::Color32,
    pub success_color: egui::Color32,
    pub warning_color: egui::Color32,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            primary_color: egui::Color32::from_rgb(0x1E, 0x88, 0xE5), // Blue
            secondary_color: egui::Color32::from_rgb(0x43, 0xA0, 0x47), // Green
            background_color: egui::Color32::from_rgb(0x1E, 0x1E, 0x1E), // Dark gray
            surface_color: egui::Color32::from_rgb(0x2D, 0x2D, 0x30), // Slightly lighter gray
            text_color: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF), // White
            error_color: egui::Color32::from_rgb(0xF4, 0x43, 0x36), // Red
            success_color: egui::Color32::from_rgb(0x4C, 0xAF, 0x50), // Green
            warning_color: egui::Color32::from_rgb(0xFF, 0x98, 0x00), // Orange
        }
    }
}

impl MainWindow {
    pub fn new(
        media_manager: Arc<MediaManager>,
        encryption: Arc<EndToEndEncryption>,
    ) -> Result<Self> {
        let settings_panel = Arc::new(SettingsPanel::new(media_manager.clone())?);
        let status_panel = Arc::new(StatusPanel::new());
        let connection_panel = Arc::new(ConnectionPanel::new(media_manager.clone())?);
        let file_transfer_panel = Arc::new(FileTransferPanel::new());
        let chat_panel = Arc::new(ChatPanel::new());
        let notification_manager = Arc::new(NotificationManager::new());

        Ok(Self {
            media_manager,
            encryption,
            settings_panel,
            status_panel,
            connection_panel,
            file_transfer_panel,
            chat_panel,
            notification_manager,
            connection_state: Arc::new(RwLock::new(ConnectionState {
                is_connected: false,
                session_id: None,
                remote_user: None,
                connection_quality: ConnectionQuality::Disconnected,
                bandwidth: BandwidthStats {
                    upload_speed: 0.0,
                    download_speed: 0.0,
                    total_uploaded: 0,
                    total_downloaded: 0,
                },
                latency: 0,
                uptime: Utc::now(),
            })),
            current_user: Arc::new(RwLock::new(None)),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            show_settings: false,
            show_status: false,
            show_chat: false,
            show_file_transfers: false,
            selected_session: None,
            theme: AppTheme::default(),
            dark_mode: true,
        })
    }

    fn render_top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Logo and title
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("🖥️ GenXLink")
                            .size(20.0)
                            .color(self.theme.primary_color)
                    ));
                    
                    ui.separator();
                    
                    // Connection status indicator
                    let connection_state = self.connection_state.try_read().unwrap();
                    let (status_color, status_text) = match connection_state.connection_quality {
                        ConnectionQuality::Excellent => (self.theme.success_color, "Excellent"),
                        ConnectionQuality::Good => (self.theme.success_color, "Good"),
                        ConnectionQuality::Fair => (self.theme.warning_color, "Fair"),
                        ConnectionQuality::Poor => (self.theme.error_color, "Poor"),
                        ConnectionQuality::Disconnected => (egui::Color32::GRAY, "Disconnected"),
                    };
                    
                    ui.add(egui::Label::new(
                        egui::RichText::new("●")
                            .size(16.0)
                            .color(status_color)
                    ));
                    ui.add(egui::Label::new(
                        egui::RichText::new(status_text)
                            .size(14.0)
                            .color(self.theme.text_color)
                    ));
                    
                    if connection_state.is_connected {
                        if let Some(remote_user) = &connection_state.remote_user {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("Connected to {}", remote_user))
                                    .size(14.0)
                                    .color(self.theme.text_color)
                            ));
                        }
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("{} ms", connection_state.latency))
                                .size(14.0)
                                .color(self.theme.text_color)
                        ));
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Settings button
                    if ui.add(
                        egui::Button::new("⚙️ Settings")
                            .fill(if self.show_settings { self.theme.primary_color } else { self.theme.surface_color })
                    ).clicked() {
                        self.show_settings = !self.show_settings;
                        self.show_status = false;
                        self.show_chat = false;
                        self.show_file_transfers = false;
                    }
                    
                    // Status button
                    if ui.add(
                        egui::Button::new("📊 Status")
                            .fill(if self.show_status { self.theme.primary_color } else { self.theme.surface_color })
                    ).clicked() {
                        self.show_status = !self.show_status;
                        self.show_settings = false;
                        self.show_chat = false;
                        self.show_file_transfers = false;
                    }
                    
                    // Chat button
                    if ui.add(
                        egui::Button::new("💬 Chat")
                            .fill(if self.show_chat { self.theme.primary_color } else { self.theme.surface_color })
                    ).clicked() {
                        self.show_chat = !self.show_chat;
                        self.show_settings = false;
                        self.show_status = false;
                        self.show_file_transfers = false;
                    }
                    
                    // File transfers button
                    if ui.add(
                        egui::Button::new("📁 Files")
                            .fill(if self.show_file_transfers { self.theme.primary_color } else { self.theme.surface_color })
                    ).clicked() {
                        self.show_file_transfers = !self.show_file_transfers;
                        self.show_settings = false;
                        self.show_status = false;
                        self.show_chat = false;
                    }
                    
                    // Notifications indicator
                    let notification_count = self.notification_manager.get_unread_count();
                    if notification_count > 0 {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("🔔 {}", notification_count))
                                .size(16.0)
                                .color(self.theme.warning_color)
                        ));
                    }
                });
            });
        });
    }

    fn render_side_panel(&mut self, ctx: &Context) {
        SidePanel::left("side_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Active Sessions");
                ui.separator();
                
                // Session list
                let active_sessions = self.active_sessions.try_read().unwrap();
                
                if active_sessions.is_empty() {
                    ui.add(egui::Label::new(
                        egui::RichText::new("No active sessions")
                            .color(egui::Color32::GRAY)
                    ));
                } else {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (session_id, config) in active_sessions.iter() {
                            let is_selected = self.selected_session == Some(*session_id);
                            
                            let session_text = if config.screen_share.is_some() && config.audio.is_some() {
                                "🖥️🎤 Screen + Audio"
                            } else if config.screen_share.is_some() {
                                "🖥️ Screen Share"
                            } else if config.audio.is_some() {
                                "🎤 Audio Share"
                            } else if config.file_transfer.is_some() {
                                "📁 File Transfer"
                            } else {
                                "📡 Connection"
                            };
                            
                            if ui.add(
                                egui::Button::new(session_text)
                                    .fill(if is_selected { self.theme.primary_color } else { self.theme.surface_color })
                            ).clicked() {
                                self.selected_session = Some(*session_id);
                            }
                        }
                    });
                }
                
                ui.separator();
                
                // Quick actions
                ui.heading("Quick Actions");
                
                if ui.add(
                    egui::Button::new("🖥️ Start Screen Share")
                        .fill(self.theme.secondary_color)
                ).clicked() {
                    self.start_screen_share();
                }
                
                if ui.add(
                    egui::Button::new("🎤 Start Audio Share")
                        .fill(self.theme.secondary_color)
                ).clicked() {
                    self.start_audio_share();
                }
                
                if ui.add(
                    egui::Button::new("📁 Send File")
                        .fill(self.theme.secondary_color)
                ).clicked() {
                    self.send_file();
                }
                
                if ui.add(
                    egui::Button::new("🔌 Connect to User")
                        .fill(self.theme.primary_color)
                ).clicked() {
                    self.show_connection_dialog();
                }
            });
    }

    fn render_main_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.show_settings {
                self.settings_panel.show(ui, &self.theme);
            } else if self.show_status {
                self.status_panel.show(ui, &self.theme);
            } else if self.show_chat {
                self.chat_panel.show(ui, &self.theme);
            } else if self.show_file_transfers {
                self.file_transfer_panel.show(ui, &self.theme);
            } else {
                // Show connection panel or welcome screen
                let connection_state = self.connection_state.try_read().unwrap();
                
                if connection_state.is_connected {
                    self.connection_panel.show_connected(ui, &self.theme, &connection_state);
                } else {
                    self.show_welcome_screen(ui);
                }
            }
        });
    }

    fn show_welcome_screen(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            ui.add(egui::Label::new(
                egui::RichText::new("Welcome to GenXLink")
                    .size(32.0)
                    .color(self.theme.primary_color)
            ));
            
            ui.add_space(20.0);
            
            ui.add(egui::Label::new(
                egui::RichText::new("Secure Remote Desktop Connection")
                    .size(18.0)
                    .color(self.theme.text_color)
            ));
            
            ui.add_space(40.0);
            
            ui.horizontal(|ui| {
                if ui.add(
                    egui::Button::new("🔌 Connect to User")
                        .min_size(Vec2::new(200.0, 50.0))
                        .fill(self.theme.primary_color)
                ).clicked() {
                    self.show_connection_dialog();
                }
                
                if ui.add(
                    egui::Button::new("⚙️ Settings")
                        .min_size(Vec2::new(200.0, 50.0))
                        .fill(self.theme.surface_color)
                ).clicked() {
                    // Note: This would need to be mutable in the actual implementation
                    info!("Settings button clicked");
                }
            });
            
            ui.add_space(30.0);
            
            ui.group(|ui| {
                ui.heading("Features:");
                ui.horizontal(|ui| {
                    ui.label("🖥️ Screen Sharing");
                    ui.label("🎤 Audio Streaming");
                    ui.label("📁 File Transfer");
                });
                ui.horizontal(|ui| {
                    ui.label("🔒 End-to-End Encryption");
                    ui.label("⚡ High Performance");
                    ui.label("🌍 Cross-Platform");
                });
            });
        });
    }

    fn start_screen_share(&self) {
        let config = ScreenShareConfig {
            width: 1920,
            height: 1080,
            frame_rate: 30,
            quality: 80,
            cursor: true,
            audio: false,
        };
        
        info!("Starting screen share with config: {:?}", config);
        // Implementation would call media_manager.start_screen_share()
    }

    fn start_audio_share(&self) {
        let config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            bitrate: 128000,
            echo_cancellation: true,
            noise_suppression: true,
        };
        
        info!("Starting audio share with config: {:?}", config);
        // Implementation would call media_manager.start_audio_stream()
    }

    fn send_file(&self) {
        info!("File send requested");
        // Implementation would open file dialog and call file transfer manager
    }

    fn show_connection_dialog(&self) {
        info!("Connection dialog requested");
        // Implementation would show connection dialog
    }
}

impl epi::App for MainWindow {
    fn name(&self) -> &str {
        "GenXLink Remote Desktop"
    }

    fn update(&mut self, ctx: &Context, _frame: &epi::Frame) {
        // Apply theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Render UI
        self.render_top_panel(ctx);
        self.render_side_panel(ctx);
        self.render_main_panel(ctx);
        
        // Handle notifications
        self.notification_manager.update(ctx, &self.theme);
    }

    fn setup(&mut self, ctx: &Context) {
        // Initialize the app
        info!("Setting up main window");
        
        // Configure fonts
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "custom_font".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.ttf")),
        );
        
        fonts.families.get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "custom_font".to_owned());
        
        ctx.set_fonts(fonts);
    }
}
