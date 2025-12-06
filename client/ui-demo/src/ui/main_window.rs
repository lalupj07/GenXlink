use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTheme {
    pub primary_color: egui::Color32,
    pub secondary_color: egui::Color32,
    pub surface_color: egui::Color32,
    pub background_color: egui::Color32,
    pub text_color: egui::Color32,
    pub border_color: egui::Color32,
    pub success_color: egui::Color32,
    pub warning_color: egui::Color32,
    pub error_color: egui::Color32,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            primary_color: egui::Color32::from_rgb(0, 120, 215),
            secondary_color: egui::Color32::from_rgb(45, 45, 45),
            surface_color: egui::Color32::from_rgb(35, 35, 35),
            background_color: egui::Color32::from_rgb(25, 25, 25),
            text_color: egui::Color32::from_rgb(255, 255, 255),
            border_color: egui::Color32::from_rgb(60, 60, 60),
            success_color: egui::Color32::from_rgb(40, 167, 69),
            warning_color: egui::Color32::from_rgb(255, 193, 7),
            error_color: egui::Color32::from_rgb(220, 53, 69),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    pub is_connected: bool,
    pub remote_user: Option<String>,
    pub connection_quality: ConnectionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl ConnectionState {
    pub fn new() -> Self {
        Self {
            is_connected: false,
            remote_user: None,
            connection_quality: ConnectionQuality::Excellent,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub upload_bps: f64,
    pub download_bps: f64,
    pub latency_ms: f64,
    pub packet_loss: f64,
}

impl Default for BandwidthStats {
    fn default() -> Self {
        Self {
            upload_bps: 0.0,
            download_bps: 0.0,
            latency_ms: 0.0,
            packet_loss: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MainWindow {
    pub connection_state: ConnectionState,
    pub bandwidth_stats: BandwidthStats,
    pub theme: AppTheme,
    pub dark_mode: bool,
    pub show_connection_dialog: bool,
    pub show_settings: bool,
    pub show_notifications: bool,
    pub is_fullscreen: bool,
    pub window_title: String,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::new(),
            bandwidth_stats: BandwidthStats::default(),
            theme: AppTheme::default(),
            dark_mode: true,
            show_connection_dialog: false,
            show_settings: false,
            show_notifications: true,
            is_fullscreen: false,
            window_title: "GenXLink Remote Desktop".to_string(),
        }
    }

    pub fn update_connection_status(&mut self, connected: bool, remote_user: Option<String>) {
        self.connection_state.is_connected = connected;
        self.connection_state.remote_user = remote_user;
    }

    pub fn set_theme(&mut self, theme: AppTheme) {
        self.theme = theme;
    }

    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
    }

    pub fn toggle_fullscreen(&mut self) {
        self.is_fullscreen = !self.is_fullscreen;
    }

    pub fn show_connection_dialog(&mut self) {
        self.show_connection_dialog = true;
    }

    pub fn hide_connection_dialog(&mut self) {
        self.show_connection_dialog = false;
    }

    pub fn show_settings_panel(&mut self) {
        self.show_settings = true;
    }

    pub fn hide_settings_panel(&mut self) {
        self.show_settings = false;
    }

    pub fn toggle_notifications(&mut self) {
        self.show_notifications = !self.show_notifications;
    }
}
