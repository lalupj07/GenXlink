use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Screen preview panel for displaying captured frames
pub struct ScreenPreviewPanel {
    /// Current frame data (BGRA format)
    frame_data: Arc<Mutex<Option<FrameData>>>,
    
    /// Selected monitor index
    selected_monitor: usize,
    
    /// Available monitors
    monitors: Vec<MonitorInfo>,
    
    /// Capture status
    is_capturing: bool,
    
    /// FPS counter
    fps: f32,
    last_frame_time: std::time::Instant,
}

#[derive(Clone)]
struct FrameData {
    width: u32,
    height: u32,
    data: Vec<u8>,
    timestamp: std::time::Instant,
}

#[derive(Clone, Debug)]
struct MonitorInfo {
    index: usize,
    name: String,
    width: u32,
    height: u32,
    is_primary: bool,
}

impl Default for ScreenPreviewPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenPreviewPanel {
    pub fn new() -> Self {
        // Get available monitors
        let monitors = Self::get_monitors();
        
        Self {
            frame_data: Arc::new(Mutex::new(None)),
            selected_monitor: 0,
            monitors,
            is_capturing: false,
            fps: 0.0,
            last_frame_time: std::time::Instant::now(),
        }
    }
    
    fn get_monitors() -> Vec<MonitorInfo> {
        #[cfg(target_os = "windows")]
        {
            use genxlink_client_core::screen_capture::ScreenCapturer;
            
            match ScreenCapturer::get_monitors() {
                Ok(monitors) => monitors.into_iter().map(|m| MonitorInfo {
                    index: m.index,
                    name: m.name,
                    width: m.width,
                    height: m.height,
                    is_primary: m.is_primary,
                }).collect(),
                Err(e) => {
                    tracing::error!("Failed to get monitors: {}", e);
                    vec![MonitorInfo {
                        index: 0,
                        name: "Primary Monitor".to_string(),
                        width: 1920,
                        height: 1080,
                        is_primary: true,
                    }]
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            vec![MonitorInfo {
                index: 0,
                name: "Primary Monitor".to_string(),
                width: 1920,
                height: 1080,
                is_primary: true,
            }]
        }
    }
    
    pub fn start_capture(&mut self) {
        if self.is_capturing {
            return;
        }
        
        self.is_capturing = true;
        
        #[cfg(target_os = "windows")]
        {
            use genxlink_client_core::screen_capture::{ScreenCapturer, CaptureConfig};
            
            let config = CaptureConfig {
                monitor_index: self.selected_monitor,
                capture_cursor: true,
                target_fps: 30,
            };
            
            let frame_data = self.frame_data.clone();
            
            tokio::spawn(async move {
                match ScreenCapturer::new(config) {
                    Ok(capturer) => {
                        tracing::info!("Screen capturer initialized");
                        
                        if let Err(e) = capturer.start_capture(move |frame| {
                            // Update frame data
                            let frame_data_clone = frame_data.clone();
                            tokio::spawn(async move {
                                let mut data = frame_data_clone.lock().await;
                                *data = Some(FrameData {
                                    width: frame.width,
                                    height: frame.height,
                                    data: frame.data,
                                    timestamp: frame.timestamp,
                                });
                            });
                            Ok(())
                        }).await {
                            tracing::error!("Capture error: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create screen capturer: {}", e);
                    }
                }
            });
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            tracing::warn!("Screen capture not supported on this platform");
        }
    }
    
    pub fn stop_capture(&mut self) {
        self.is_capturing = false;
        // TODO: Send stop signal to capture thread
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📺 Screen Preview");
        ui.add_space(10.0);
        
        // Monitor selection
        ui.horizontal(|ui| {
            ui.label("Monitor:");
            egui::ComboBox::from_id_source("monitor_select")
                .selected_text(format!("{} ({}x{})", 
                    self.monitors.get(self.selected_monitor)
                        .map(|m| m.name.as_str())
                        .unwrap_or("Unknown"),
                    self.monitors.get(self.selected_monitor)
                        .map(|m| m.width)
                        .unwrap_or(0),
                    self.monitors.get(self.selected_monitor)
                        .map(|m| m.height)
                        .unwrap_or(0)
                ))
                .show_ui(ui, |ui| {
                    for (idx, monitor) in self.monitors.iter().enumerate() {
                        let label = if monitor.is_primary {
                            format!("⭐ {} ({}x{}) - Primary", monitor.name, monitor.width, monitor.height)
                        } else {
                            format!("{} ({}x{})", monitor.name, monitor.width, monitor.height)
                        };
                        
                        ui.selectable_value(&mut self.selected_monitor, idx, label);
                    }
                });
        });
        
        ui.add_space(10.0);
        
        // Capture controls
        ui.horizontal(|ui| {
            if self.is_capturing {
                if ui.button("⏹ Stop Capture").clicked() {
                    self.stop_capture();
                }
                
                ui.label(format!("🎥 Capturing at {:.1} FPS", self.fps));
            } else {
                if ui.button("▶️ Start Capture").clicked() {
                    self.start_capture();
                }
            }
        });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // Frame preview
        if self.is_capturing {
            // Try to get frame data
            if let Ok(frame_guard) = self.frame_data.try_lock() {
                if let Some(ref frame) = *frame_guard {
                    // Update FPS
                    let now = std::time::Instant::now();
                    let delta = now.duration_since(self.last_frame_time).as_secs_f32();
                    if delta > 0.0 {
                        self.fps = 0.9 * self.fps + 0.1 * (1.0 / delta);
                    }
                    self.last_frame_time = now;
                    
                    // Display frame info
                    ui.label(format!("Frame: {}x{}", frame.width, frame.height));
                    ui.label(format!("Data size: {} bytes", frame.data.len()));
                    
                    // TODO: Convert BGRA to texture and display
                    // For now, just show a placeholder
                    let available_size = ui.available_size();
                    let aspect_ratio = frame.width as f32 / frame.height as f32;
                    let preview_width = available_size.x.min(800.0);
                    let preview_height = preview_width / aspect_ratio;
                    
                    ui.allocate_space(egui::vec2(preview_width, preview_height));
                    
                    ui.colored_label(
                        egui::Color32::GREEN,
                        "✅ Frame captured successfully"
                    );
                    ui.label("(Frame display will be implemented in next iteration)");
                } else {
                    ui.label("Waiting for first frame...");
                }
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("📷");
                ui.add_space(10.0);
                ui.label("Click 'Start Capture' to begin screen capture");
                ui.add_space(50.0);
            });
        }
        
        ui.add_space(10.0);
        
        // Info panel
        ui.group(|ui| {
            ui.label("ℹ️ Screen Capture Info");
            ui.separator();
            ui.label(format!("• Available monitors: {}", self.monitors.len()));
            ui.label(format!("• Selected monitor: {}", self.selected_monitor));
            ui.label(format!("• Capture status: {}", if self.is_capturing { "Active" } else { "Stopped" }));
            ui.label("• Technology: DXGI Desktop Duplication");
            ui.label("• Target FPS: 30");
        });
    }
}
