use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use eframe::egui::{self, Ui, ScrollArea, Grid, Vec2, Slider, ComboBox};
use serde::{Deserialize, Serialize};

use crate::webrtc::media_manager::{MediaManager, ScreenShareConfig, AudioConfig, FileTransferConfig};
use crate::ui::main_window::AppTheme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub screen_share: ScreenShareSettings,
    pub audio: AudioSettings,
    pub file_transfer: FileTransferSettings,
    pub security: SecuritySettings,
    pub network: NetworkSettings,
    pub ui: UISettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub check_updates: bool,
    pub language: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenShareSettings {
    pub default_width: u32,
    pub default_height: u32,
    pub default_frame_rate: u32,
    pub default_quality: u8,
    pub show_cursor: bool,
    pub capture_audio: bool,
    pub compression_level: u8,
    pub color_depth: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: u32,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
    pub automatic_gain_control: bool,
    pub input_device: String,
    pub output_device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferSettings {
    pub download_directory: String,
    pub max_file_size: u64,
    pub chunk_size: usize,
    pub auto_accept: bool,
    pub compression_enabled: bool,
    pub resume_transfers: bool,
    pub scan_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub require_password: bool,
    pub session_timeout: u32,
    pub encryption_enabled: bool,
    pub key_rotation_interval: u32,
    pub save_passwords: bool,
    pub two_factor_auth: bool,
    pub trusted_devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub proxy_enabled: bool,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_username: String,
    pub proxy_password: String,
    pub bandwidth_limit: u32,
    pub connection_timeout: u32,
    pub keep_alive_interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub theme: String,
    pub font_size: f32,
    pub window_opacity: f32,
    pub show_notifications: bool,
    pub notification_sound: bool,
    pub minimize_animation: bool,
    pub show_status_bar: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                auto_start: false,
                minimize_to_tray: true,
                start_minimized: false,
                check_updates: true,
                language: "en".to_string(),
                log_level: "info".to_string(),
            },
            screen_share: ScreenShareSettings {
                default_width: 1920,
                default_height: 1080,
                default_frame_rate: 30,
                default_quality: 80,
                show_cursor: true,
                capture_audio: false,
                compression_level: 6,
                color_depth: 32,
            },
            audio: AudioSettings {
                sample_rate: 48000,
                channels: 2,
                bitrate: 128000,
                echo_cancellation: true,
                noise_suppression: true,
                automatic_gain_control: false,
                input_device: "default".to_string(),
                output_device: "default".to_string(),
            },
            file_transfer: FileTransferSettings {
                download_directory: std::env::var("USERPROFILE")
                    .unwrap_or_else(|_| ".".to_string())
                    + "/Downloads/GenXLink",
                max_file_size: 1024 * 1024 * 1024, // 1GB
                chunk_size: 64 * 1024, // 64KB
                auto_accept: false,
                compression_enabled: true,
                resume_transfers: true,
                scan_files: true,
            },
            security: SecuritySettings {
                require_password: false,
                session_timeout: 3600, // 1 hour
                encryption_enabled: true,
                key_rotation_interval: 1800, // 30 minutes
                save_passwords: false,
                two_factor_auth: false,
                trusted_devices: Vec::new(),
            },
            network: NetworkSettings {
                proxy_enabled: false,
                proxy_host: String::new(),
                proxy_port: 8080,
                proxy_username: String::new(),
                proxy_password: String::new(),
                bandwidth_limit: 0, // No limit
                connection_timeout: 30, // 30 seconds
                keep_alive_interval: 30, // 30 seconds
            },
            ui: UISettings {
                theme: "dark".to_string(),
                font_size: 14.0,
                window_opacity: 1.0,
                show_notifications: true,
                notification_sound: true,
                minimize_animation: true,
                show_status_bar: true,
            },
        }
    }
}

pub struct SettingsPanel {
    media_manager: Arc<MediaManager>,
    settings: Arc<RwLock<AppSettings>>,
    selected_category: SettingsCategory,
    has_unsaved_changes: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum SettingsCategory {
    General,
    ScreenShare,
    Audio,
    FileTransfer,
    Security,
    Network,
    UI,
}

impl SettingsPanel {
    pub fn new(media_manager: Arc<MediaManager>) -> Result<Self> {
        let settings = AppSettings::default();
        
        Ok(Self {
            media_manager,
            settings: Arc::new(RwLock::new(settings)),
            selected_category: SettingsCategory::General,
            has_unsaved_changes: false,
        })
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.horizontal(|ui| {
            ui.heading("Settings");
            ui.separator();
            
            if self.has_unsaved_changes {
                ui.add(egui::Label::new(
                    egui::RichText::new("● Unsaved changes")
                        .color(theme.warning_color)
                ));
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(
                    egui::Button::new("💾 Save")
                        .fill(theme.success_color)
                ).clicked() {
                    self.save_settings();
                }
                
                if ui.add(
                    egui::Button::new("↺ Reset")
                        .fill(theme.surface_color)
                ).clicked() {
                    self.reset_settings();
                }
            });
        });
        
        ui.separator();
        
        // Category sidebar
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Categories");
                
                let categories = [
                    (SettingsCategory::General, "🏠 General"),
                    (SettingsCategory::ScreenShare, "🖥️ Screen Share"),
                    (SettingsCategory::Audio, "🎤 Audio"),
                    (SettingsCategory::FileTransfer, "📁 File Transfer"),
                    (SettingsCategory::Security, "🔒 Security"),
                    (SettingsCategory::Network, "🌐 Network"),
                    (SettingsCategory::UI, "🎨 Interface"),
                ];
                
                for (category, name) in categories {
                    let is_selected = self.selected_category == category;
                    if ui.add(
                        egui::Button::new(name)
                            .fill(if is_selected { theme.primary_color } else { theme.surface_color })
                    ).clicked() {
                        self.selected_category = category;
                    }
                }
            });
            
            ui.separator();
            
            // Settings content
            ScrollArea::vertical().show(ui, |ui| {
                match self.selected_category {
                    SettingsCategory::General => self.show_general_settings(ui, theme),
                    SettingsCategory::ScreenShare => self.show_screen_share_settings(ui, theme),
                    SettingsCategory::Audio => self.show_audio_settings(ui, theme),
                    SettingsCategory::FileTransfer => self.show_file_transfer_settings(ui, theme),
                    SettingsCategory::Security => self.show_security_settings(ui, theme),
                    SettingsCategory::Network => self.show_network_settings(ui, theme),
                    SettingsCategory::UI => self.show_ui_settings(ui, theme),
                }
            });
        });
    }

    fn show_general_settings(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.heading("General Settings");
        ui.separator();
        
        let mut settings = self.settings.try_write().unwrap();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Auto-start with Windows:"));
            if ui.checkbox(&mut settings.general.auto_start, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Minimize to tray:"));
            if ui.checkbox(&mut settings.general.minimize_to_tray, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Start minimized:"));
            if ui.checkbox(&mut settings.general.start_minimized, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Check for updates:"));
            if ui.checkbox(&mut settings.general.check_updates, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Language:"));
            let mut selected_index = 0;
            let languages = ["English", "Spanish", "French", "German", "Chinese", "Japanese"];
            if ComboBox::from_label("")
                .selected_text(&settings.general.language)
                .show_index(ui, &mut selected_index, languages.len(), |i| languages[i].to_string())
                .changed()
            {
                settings.general.language = languages[selected_index].to_lowercase();
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Log Level:"));
            let mut selected_index = 0;
            let log_levels = ["Error", "Warn", "Info", "Debug", "Trace"];
            if ComboBox::from_label("")
                .selected_text(&settings.general.log_level)
                .show_index(ui, &mut selected_index, log_levels.len(), |i| log_levels[i].to_string())
                .changed()
            {
                settings.general.log_level = log_levels[selected_index].to_lowercase();
                self.has_unsaved_changes = true;
            }
        });
    }

    fn show_screen_share_settings(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.heading("Screen Share Settings");
        ui.separator();
        
        let mut settings = self.settings.try_write().unwrap();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Default Width:"));
            if ui.add(Slider::new(&mut settings.screen_share.default_width, 640..=3840)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("px"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Default Height:"));
            if ui.add(Slider::new(&mut settings.screen_share.default_height, 480..=2160)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("px"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Frame Rate:"));
            if ui.add(Slider::new(&mut settings.screen_share.default_frame_rate, 15..=60)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("fps"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Quality:"));
            if ui.add(Slider::new(&mut settings.screen_share.default_quality, 10..=100)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("%"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Show Cursor:"));
            if ui.checkbox(&mut settings.screen_share.show_cursor, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Capture Audio:"));
            if ui.checkbox(&mut settings.screen_share.capture_audio, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Compression Level:"));
            if ui.add(Slider::new(&mut settings.screen_share.compression_level, 1..=9)).changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Color Depth:"));
            if ui.add(Slider::new(&mut settings.screen_share.color_depth, 16..=32)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("bit"));
        });
    }

    fn show_audio_settings(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.heading("Audio Settings");
        ui.separator();
        
        let mut settings = self.settings.try_write().unwrap();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Sample Rate:"));
            let mut selected_index = 0;
            let sample_rates = [8000, 16000, 22050, 44100, 48000, 96000];
            let current_index = sample_rates.iter().position(|&r| r == settings.audio.sample_rate).unwrap_or(3);
            if ComboBox::from_label("")
                .selected_text(format!("{} Hz", settings.audio.sample_rate))
                .show_index(ui, &mut selected_index, sample_rates.len(), |i| format!("{} Hz", sample_rates[i]))
                .changed()
            {
                settings.audio.sample_rate = sample_rates[selected_index];
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Channels:"));
            if ui.add(Slider::new(&mut settings.audio.channels, 1..=8)).changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Bitrate:"));
            if ui.add(Slider::new(&mut settings.audio.bitrate, 64000..=320000)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("bps"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Echo Cancellation:"));
            if ui.checkbox(&mut settings.audio.echo_cancellation, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Noise Suppression:"));
            if ui.checkbox(&mut settings.audio.noise_suppression, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Automatic Gain Control:"));
            if ui.checkbox(&mut settings.audio.automatic_gain_control, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
    }

    fn show_file_transfer_settings(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.heading("File Transfer Settings");
        ui.separator();
        
        let mut settings = self.settings.try_write().unwrap();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Download Directory:"));
            if ui.text_edit_singleline(&mut settings.file_transfer.download_directory).changed() {
                self.has_unsaved_changes = true;
            }
            if ui.button("Browse...").clicked() {
                // Open file dialog
                info!("Browse for download directory");
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Max File Size:"));
            if ui.add(Slider::new(&mut settings.file_transfer.max_file_size, 1024*1024..=10*1024*1024*1024)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("bytes"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Chunk Size:"));
            if ui.add(Slider::new(&mut settings.file_transfer.chunk_size, 1024..=1024*1024)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("bytes"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Auto Accept Files:"));
            if ui.checkbox(&mut settings.file_transfer.auto_accept, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Compression Enabled:"));
            if ui.checkbox(&mut settings.file_transfer.compression_enabled, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Resume Transfers:"));
            if ui.checkbox(&mut settings.file_transfer.resume_transfers, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Scan Files for Viruses:"));
            if ui.checkbox(&mut settings.file_transfer.scan_files, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
    }

    fn show_security_settings(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.heading("Security Settings");
        ui.separator();
        
        let mut settings = self.settings.try_write().unwrap();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Require Password:"));
            if ui.checkbox(&mut settings.security.require_password, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Session Timeout:"));
            if ui.add(Slider::new(&mut settings.security.session_timeout, 300..=86400)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("seconds"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Encryption Enabled:"));
            if ui.checkbox(&mut settings.security.encryption_enabled, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Key Rotation Interval:"));
            if ui.add(Slider::new(&mut settings.security.key_rotation_interval, 300..=7200)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("seconds"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Save Passwords:"));
            if ui.checkbox(&mut settings.security.save_passwords, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Two-Factor Authentication:"));
            if ui.checkbox(&mut settings.security.two_factor_auth, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
    }

    fn show_network_settings(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.heading("Network Settings");
        ui.separator();
        
        let mut settings = self.settings.try_write().unwrap();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Proxy Enabled:"));
            if ui.checkbox(&mut settings.network.proxy_enabled, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        if settings.network.proxy_enabled {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("Proxy Host:"));
                if ui.text_edit_singleline(&mut settings.network.proxy_host).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("Proxy Port:"));
                if ui.add(Slider::new(&mut settings.network.proxy_port, 1..=65535)).changed() {
                    self.has_unsaved_changes = true;
                }
            });
        }
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Bandwidth Limit:"));
            if ui.add(Slider::new(&mut settings.network.bandwidth_limit, 0..=1000)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("Mbps"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Connection Timeout:"));
            if ui.add(Slider::new(&mut settings.network.connection_timeout, 5..=300)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("seconds"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Keep Alive Interval:"));
            if ui.add(Slider::new(&mut settings.network.keep_alive_interval, 10..=300)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("seconds"));
        });
    }

    fn show_ui_settings(&mut self, ui: &mut Ui, theme: &AppTheme) {
        ui.heading("Interface Settings");
        ui.separator();
        
        let mut settings = self.settings.try_write().unwrap();
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Theme:"));
            let mut selected_index = 0;
            let themes = ["Dark", "Light", "System"];
            if ComboBox::from_label("")
                .selected_text(&settings.ui.theme)
                .show_index(ui, &mut selected_index, themes.len(), |i| themes[i].to_string())
                .changed()
            {
                settings.ui.theme = themes[selected_index].to_lowercase();
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Font Size:"));
            if ui.add(Slider::new(&mut settings.ui.font_size, 10.0..=24.0)).changed() {
                self.has_unsaved_changes = true;
            }
            ui.add(egui::Label::new("px"));
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Window Opacity:"));
            if ui.add(Slider::new(&mut settings.ui.window_opacity, 0.3..=1.0)).changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Show Notifications:"));
            if ui.checkbox(&mut settings.ui.show_notifications, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Notification Sound:"));
            if ui.checkbox(&mut settings.ui.notification_sound, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Minimize Animation:"));
            if ui.checkbox(&mut settings.ui.minimize_animation, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Show Status Bar:"));
            if ui.checkbox(&mut settings.ui.show_status_bar, "").changed() {
                self.has_unsaved_changes = true;
            }
        });
    }

    fn save_settings(&mut self) {
        info!("Saving settings");
        self.has_unsaved_changes = false;
        // Implementation would save to config file
    }

    fn reset_settings(&mut self) {
        info!("Resetting settings to defaults");
        let mut settings = self.settings.try_write().unwrap();
        *settings = AppSettings::default();
        self.has_unsaved_changes = true;
    }

    pub async fn get_settings(&self) -> AppSettings {
        self.settings.try_read().unwrap().clone()
    }

    pub async fn update_settings(&self, new_settings: AppSettings) -> Result<()> {
        let mut settings = self.settings.try_write().unwrap();
        *settings = new_settings;
        info!("Settings updated");
        Ok(())
    }
}
