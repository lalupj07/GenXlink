use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use eframe::egui::{self, Ui, ScrollArea, Grid, Vec2, ProgressBar};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use crate::ui::main_window::{ConnectionState, ConnectionQuality, BandwidthStats};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: NetworkUsage,
    pub gpu_usage: Option<f64>,
    pub temperature: Option<f64>,
    pub uptime: Duration,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub connections_active: u32,
    pub connections_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub screen_frames_sent: u64,
    pub audio_frames_sent: u64,
    pub files_sent: u64,
    pub files_received: u64,
    pub total_bytes_transferred: u64,
    pub average_latency: f64,
    pub connection_quality: ConnectionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub encoding_fps: f64,
    pub decoding_fps: f64,
    pub encoding_latency: f64,
    pub decoding_latency: f64,
    pub network_latency: f64,
    pub queue_size: u32,
    pub dropped_frames: u64,
    pub compression_ratio: f64,
}

pub struct StatusPanel {
    system_status: Arc<RwLock<SystemStatus>>,
    session_stats: Arc<RwLock<HashMap<Uuid, SessionStats>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    connection_state: Arc<RwLock<ConnectionState>>,
    auto_refresh: bool,
    refresh_interval: u32,
}

impl StatusPanel {
    pub fn new() -> Self {
        Self {
            system_status: Arc::new(RwLock::new(SystemStatus {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_usage: NetworkUsage {
                    bytes_sent: 0,
                    bytes_received: 0,
                    packets_sent: 0,
                    packets_received: 0,
                    connections_active: 0,
                    connections_total: 0,
                },
                gpu_usage: None,
                temperature: None,
                uptime: Duration::zero(),
                last_updated: Utc::now(),
            })),
            session_stats: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics {
                encoding_fps: 0.0,
                decoding_fps: 0.0,
                encoding_latency: 0.0,
                decoding_latency: 0.0,
                network_latency: 0.0,
                queue_size: 0,
                dropped_frames: 0,
                compression_ratio: 1.0,
            })),
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
            auto_refresh: true,
            refresh_interval: 1, // seconds
        }
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &crate::ui::main_window::AppTheme) {
        ui.horizontal(|ui| {
            ui.heading("System Status");
            ui.separator();
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(
                    egui::Button::new("🔄 Refresh")
                        .fill(theme.surface_color)
                ).clicked() {
                    self.refresh_status();
                }
                
                if ui.checkbox(&mut self.auto_refresh, "Auto Refresh").changed() {
                    info!("Auto refresh: {}", self.auto_refresh);
                }
            });
        });
        
        ui.separator();
        
        // System Resources
        self.show_system_resources(ui, theme);
        
        ui.separator();
        
        // Connection Status
        self.show_connection_status(ui, theme);
        
        ui.separator();
        
        // Performance Metrics
        self.show_performance_metrics(ui, theme);
        
        ui.separator();
        
        // Session Statistics
        self.show_session_statistics(ui, theme);
    }

    fn show_system_resources(&self, ui: &mut Ui, theme: &crate::ui::main_window::AppTheme) {
        ui.heading("System Resources");
        
        let system_status = self.system_status.try_read().unwrap();
        
        Grid::new("system_resources")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                // CPU Usage
                ui.label("CPU Usage:");
                ui.add(ProgressBar::new(system_status.cpu_usage / 100.0)
                    .show_percentage()
                    .fill(self.get_usage_color(system_status.cpu_usage, theme)));
                ui.end_row();
                
                // Memory Usage
                ui.label("Memory Usage:");
                ui.add(ProgressBar::new(system_status.memory_usage / 100.0)
                    .show_percentage()
                    .fill(self.get_usage_color(system_status.memory_usage, theme)));
                ui.end_row();
                
                // Disk Usage
                ui.label("Disk Usage:");
                ui.add(ProgressBar::new(system_status.disk_usage / 100.0)
                    .show_percentage()
                    .fill(self.get_usage_color(system_status.disk_usage, theme)));
                ui.end_row();
                
                // GPU Usage (if available)
                if let Some(gpu_usage) = system_status.gpu_usage {
                    ui.label("GPU Usage:");
                    ui.add(ProgressBar::new(gpu_usage / 100.0)
                        .show_percentage()
                        .fill(self.get_usage_color(gpu_usage, theme)));
                    ui.end_row();
                }
                
                // Temperature (if available)
                if let Some(temperature) = system_status.temperature {
                    ui.label("Temperature:");
                    let temp_color = if temperature > 80.0 {
                        theme.error_color
                    } else if temperature > 60.0 {
                        theme.warning_color
                    } else {
                        theme.success_color
                    };
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!("{:.1}°C", temperature))
                            .color(temp_color)
                    ));
                    ui.end_row();
                }
                
                // Uptime
                ui.label("System Uptime:");
                ui.label(format_duration(system_status.uptime));
                ui.end_row();
                
                // Network Usage
                ui.label("Network (Sent/Received):");
                ui.label(format!(
                    "{} / {}",
                    format_bytes(system_status.network_usage.bytes_sent),
                    format_bytes(system_status.network_usage.bytes_received)
                ));
                ui.end_row();
                
                // Active Connections
                ui.label("Active Connections:");
                ui.label(format!("{}", system_status.network_usage.connections_active));
                ui.end_row();
            });
    }

    fn show_connection_status(&self, ui: &mut Ui, theme: &crate::ui::main_window::AppTheme) {
        ui.heading("Connection Status");
        
        let connection_state = self.connection_state.try_read().unwrap();
        
        Grid::new("connection_status")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                // Connection Status
                let (status_text, status_color) = if connection_state.is_connected {
                    ("Connected", theme.success_color)
                } else {
                    ("Disconnected", theme.error_color)
                };
                
                ui.label("Status:");
                ui.add(egui::Label::new(
                    egui::RichText::new(status_text)
                        .color(status_color)
                ));
                ui.end_row();
                
                // Connection Quality
                let (quality_text, quality_color) = match connection_state.connection_quality {
                    ConnectionQuality::Excellent => ("Excellent", theme.success_color),
                    ConnectionQuality::Good => ("Good", theme.success_color),
                    ConnectionQuality::Fair => ("Fair", theme.warning_color),
                    ConnectionQuality::Poor => ("Poor", theme.error_color),
                    ConnectionQuality::Disconnected => ("Disconnected", egui::Color32::GRAY),
                };
                
                ui.label("Quality:");
                ui.add(egui::Label::new(
                    egui::RichText::new(quality_text)
                        .color(quality_color)
                ));
                ui.end_row();
                
                // Remote User
                ui.label("Remote User:");
                ui.label(connection_state.remote_user.as_deref().unwrap_or("None"));
                ui.end_row();
                
                // Session ID
                ui.label("Session ID:");
                ui.label(connection_state.session_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "None".to_string()));
                ui.end_row();
                
                // Latency
                ui.label("Latency:");
                let latency_color = if connection_state.latency < 50 {
                    theme.success_color
                } else if connection_state.latency < 100 {
                    theme.warning_color
                } else {
                    theme.error_color
                };
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("{} ms", connection_state.latency))
                        .color(latency_color)
                ));
                ui.end_row();
                
                // Upload Speed
                ui.label("Upload Speed:");
                ui.label(format!("{:.2} Mbps", connection_state.bandwidth.upload_speed));
                ui.end_row();
                
                // Download Speed
                ui.label("Download Speed:");
                ui.label(format!("{:.2} Mbps", connection_state.bandwidth.download_speed));
                ui.end_row();
                
                // Total Uploaded
                ui.label("Total Uploaded:");
                ui.label(format_bytes(connection_state.bandwidth.total_uploaded * 1024 * 1024));
                ui.end_row();
                
                // Total Downloaded
                ui.label("Total Downloaded:");
                ui.label(format_bytes(connection_state.bandwidth.total_downloaded * 1024 * 1024));
                ui.end_row();
                
                // Connection Uptime
                ui.label("Connection Uptime:");
                let uptime = Utc::now() - connection_state.uptime;
                ui.label(format_duration(uptime));
                ui.end_row();
            });
    }

    fn show_performance_metrics(&self, ui: &mut Ui, theme: &crate::ui::main_window::AppTheme) {
        ui.heading("Performance Metrics");
        
        let metrics = self.performance_metrics.try_read().unwrap();
        
        Grid::new("performance_metrics")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                // Encoding FPS
                ui.label("Encoding FPS:");
                ui.label(format!("{:.2}", metrics.encoding_fps));
                ui.end_row();
                
                // Decoding FPS
                ui.label("Decoding FPS:");
                ui.label(format!("{:.2}", metrics.decoding_fps));
                ui.end_row();
                
                // Encoding Latency
                ui.label("Encoding Latency:");
                ui.label(format!("{:.2} ms", metrics.encoding_latency));
                ui.end_row();
                
                // Decoding Latency
                ui.label("Decoding Latency:");
                ui.label(format!("{:.2} ms", metrics.decoding_latency));
                ui.end_row();
                
                // Network Latency
                ui.label("Network Latency:");
                ui.label(format!("{:.2} ms", metrics.network_latency));
                ui.end_row();
                
                // Queue Size
                ui.label("Queue Size:");
                ui.label(format!("{}", metrics.queue_size));
                ui.end_row();
                
                // Dropped Frames
                ui.label("Dropped Frames:");
                ui.label(format!("{}", metrics.dropped_frames));
                ui.end_row();
                
                // Compression Ratio
                ui.label("Compression Ratio:");
                ui.label(format!("{:.2}x", metrics.compression_ratio));
                ui.end_row();
            });
    }

    fn show_session_statistics(&self, ui: &mut Ui, theme: &crate::ui::main_window::AppTheme) {
        ui.heading("Session Statistics");
        
        let session_stats = self.session_stats.try_read().unwrap();
        
        if session_stats.is_empty() {
            ui.add(egui::Label::new(
                egui::RichText::new("No active sessions")
                    .color(egui::Color32::GRAY)
            ));
            return;
        }
        
        ScrollArea::vertical().show(ui, |ui| {
            for (session_id, stats) in session_stats.iter() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(format!("Session: {}", session_id.to_string()[..8].to_uppercase()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (quality_text, quality_color) = match stats.connection_quality {
                                ConnectionQuality::Excellent => ("Excellent", theme.success_color),
                                ConnectionQuality::Good => ("Good", theme.success_color),
                                ConnectionQuality::Fair => ("Fair", theme.warning_color),
                                ConnectionQuality::Poor => ("Poor", theme.error_color),
                                ConnectionQuality::Disconnected => ("Disconnected", egui::Color32::GRAY),
                            };
                            
                            ui.add(egui::Label::new(
                                egui::RichText::new(quality_text)
                                    .color(quality_color)
                            ));
                        });
                    });
                    
                    Grid::new(format!("session_stats_{}", session_id))
                        .num_columns(2)
                        .spacing([10.0, 5.0])
                        .show(ui, |ui| {
                            ui.label("Duration:");
                            ui.label(format_duration(stats.duration));
                            ui.end_row();
                            
                            ui.label("Screen Frames Sent:");
                            ui.label(format!("{}", stats.screen_frames_sent));
                            ui.end_row();
                            
                            ui.label("Audio Frames Sent:");
                            ui.label(format!("{}", stats.audio_frames_sent));
                            ui.end_row();
                            
                            ui.label("Files Sent:");
                            ui.label(format!("{}", stats.files_sent));
                            ui.end_row();
                            
                            ui.label("Files Received:");
                            ui.label(format!("{}", stats.files_received));
                            ui.end_row();
                            
                            ui.label("Total Bytes Transferred:");
                            ui.label(format_bytes(stats.total_bytes_transferred));
                            ui.end_row();
                            
                            ui.label("Average Latency:");
                            ui.label(format!("{:.2} ms", stats.average_latency));
                            ui.end_row();
                            
                            ui.label("Start Time:");
                            ui.label(stats.start_time.format("%Y-%m-%d %H:%M:%S").to_string());
                            ui.end_row();
                        });
                });
                
                ui.add_space(10.0);
            }
        });
    }

    fn get_usage_color(&self, usage: f64, theme: &crate::ui::main_window::AppTheme) -> egui::Color32 {
        if usage > 90.0 {
            theme.error_color
        } else if usage > 70.0 {
            theme.warning_color
        } else {
            theme.success_color
        }
    }

    fn refresh_status(&self) {
        info!("Refreshing system status");
        // Implementation would update all status values
        tokio::spawn(async {
            // This would gather real system information
            info!("Status refresh completed");
        });
    }

    pub async fn update_system_status(&self, status: SystemStatus) {
        let mut system_status = self.system_status.try_write().unwrap();
        *system_status = status;
    }

    pub async fn update_connection_state(&self, state: ConnectionState) {
        let mut connection_state = self.connection_state.try_write().unwrap();
        *connection_state = state;
    }

    pub async fn update_performance_metrics(&self, metrics: PerformanceMetrics) {
        let mut performance_metrics = self.performance_metrics.try_write().unwrap();
        *performance_metrics = metrics;
    }

    pub async fn add_session_stats(&self, session_id: Uuid, stats: SessionStats) {
        let mut session_stats = self.session_stats.try_write().unwrap();
        session_stats.insert(session_id, stats);
    }

    pub async fn remove_session_stats(&self, session_id: Uuid) {
        let mut session_stats = self.session_stats.try_write().unwrap();
        session_stats.remove(&session_id);
    }

    pub async fn get_system_status(&self) -> SystemStatus {
        self.system_status.try_read().unwrap().clone()
    }

    pub async fn get_connection_state(&self) -> ConnectionState {
        self.connection_state.try_read().unwrap().clone()
    }

    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.try_read().unwrap().clone()
    }
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
