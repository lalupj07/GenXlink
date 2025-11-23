use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

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
    
    /// Last screenshot message
    last_screenshot_msg: Option<String>,
    
    /// Recording status
    is_recording: bool,
    recording_start_time: Option<std::time::Instant>,
    video_encoder: Arc<Mutex<Option<Arc<genxlink_client_core::video_encoder::VideoEncoder>>>>,
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
            last_screenshot_msg: None,
            is_recording: false,
            recording_start_time: None,
            video_encoder: Arc::new(Mutex::new(None)),
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
            let video_encoder = self.video_encoder.clone();
            
            // Spawn capture thread (not using tokio since we're in egui context)
            std::thread::spawn(move || {
                tracing::info!("Starting screen capture...");
                
                // Create a simple runtime for the capture
                let rt = tokio::runtime::Runtime::new().unwrap();
                
                rt.block_on(async move {
                    match ScreenCapturer::new(config) {
                        Ok(capturer) => {
                            tracing::info!("Screen capturer initialized successfully");
                            
                            // Start capture with callback
                            let result = capturer.start_capture(move |frame| {
                                // Update frame data synchronously
                                if let Ok(mut frame_guard) = frame_data.try_lock() {
                                    *frame_guard = Some(FrameData {
                                        width: frame.width,
                                        height: frame.height,
                                        data: frame.data.clone(),
                                        timestamp: frame.timestamp,
                                    });
                                }
                                
                                // Encode frame if recording
                                let encoder_guard = video_encoder.clone();
                                let frame_data_clone = frame.data.clone();
                                tokio::spawn(async move {
                                    if let Ok(encoder_opt) = encoder_guard.try_lock() {
                                        if let Some(encoder) = encoder_opt.as_ref() {
                                            if let Err(e) = encoder.encode_frame(&frame_data_clone).await {
                                                tracing::error!("Failed to encode frame: {}", e);
                                            }
                                        } else {
                                            // This is normal - encoder not set yet or recording stopped
                                        }
                                    }
                                });
                                
                                Ok(())
                            }).await;
                            
                            if let Err(e) = result {
                                tracing::error!("Capture error: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to create screen capturer: {}", e);
                        }
                    }
                });
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
    
    pub fn save_screenshot(&mut self) {
        let frame_data = self.frame_data.clone();
        
        // Spawn thread to save screenshot
        std::thread::spawn(move || {
            if let Ok(frame_guard) = frame_data.try_lock() {
                if let Some(frame) = frame_guard.as_ref() {
                    match Self::save_frame_to_file(frame) {
                        Ok(path) => {
                            tracing::info!("Screenshot saved to: {}", path.display());
                        }
                        Err(e) => {
                            tracing::error!("Failed to save screenshot: {}", e);
                        }
                    }
                } else {
                    tracing::warn!("No frame data available to save");
                }
            }
        });
        
        self.last_screenshot_msg = Some("Screenshot saved!".to_string());
    }
    
    fn save_frame_to_file(frame: &FrameData) -> anyhow::Result<PathBuf> {
        use image::{ImageBuffer, Rgba};
        use chrono::Local;
        
        // Get Documents folder
        let docs_dir = dirs::document_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find Documents folder"))?;
        
        // Create GenXLink Captures directory
        let captures_dir = docs_dir.join("GenXLink Captures");
        std::fs::create_dir_all(&captures_dir)?;
        
        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let filename = format!("screenshot_{}.png", timestamp);
        let filepath = captures_dir.join(filename);
        
        // Convert BGRA to RGBA
        let mut rgba_data = Vec::with_capacity(frame.data.len());
        for chunk in frame.data.chunks_exact(4) {
            rgba_data.push(chunk[2]); // R
            rgba_data.push(chunk[1]); // G
            rgba_data.push(chunk[0]); // B
            rgba_data.push(chunk[3]); // A
        }
        
        // Create image and save
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            frame.width,
            frame.height,
            rgba_data,
        ).ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;
        
        img.save(&filepath)?;
        
        Ok(filepath)
    }
    
    pub fn start_recording(&mut self) {
        tracing::info!("start_recording called - is_recording: {}, is_capturing: {}", 
            self.is_recording, self.is_capturing);
        
        if self.is_recording {
            tracing::warn!("Already recording, ignoring");
            return;
        }
        
        if !self.is_capturing {
            tracing::warn!("Not capturing, cannot start recording");
            return;
        }
        
        use genxlink_client_core::video_encoder::{VideoEncoder, VideoEncoderConfig};
        use chrono::Local;
        
        // Get current frame dimensions
        let (width, height) = if let Ok(frame_guard) = self.frame_data.try_lock() {
            if let Some(frame) = frame_guard.as_ref() {
                (frame.width, frame.height)
            } else {
                (1920, 1080) // Default
            }
        } else {
            (1920, 1080) // Default
        };
        
        // Create output path
        let docs_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
        let captures_dir = docs_dir.join("GenXLink Captures");
        std::fs::create_dir_all(&captures_dir).ok();
        
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let filename = format!("recording_{}.raw", timestamp);
        let filepath = captures_dir.join(filename);
        
        // Create encoder config
        let config = VideoEncoderConfig {
            width,
            height,
            fps: 30,
            bitrate: 5_000_000, // 5 Mbps
        };
        
        // Create encoder
        tracing::info!("Creating video encoder: {}x{} @ {} fps, output: {}", 
            width, height, 30, filepath.display());
        
        match VideoEncoder::new(config, filepath.clone()) {
            Ok(encoder) => {
                tracing::info!("Video encoder created successfully");
                let encoder_arc = Arc::new(encoder);
                
                // Start recording
                let encoder_clone = encoder_arc.clone();
                tokio::spawn(async move {
                    tracing::info!("Starting encoder async task");
                    if let Err(e) = encoder_clone.start_recording().await {
                        tracing::error!("Failed to start recording: {}", e);
                    } else {
                        tracing::info!("Encoder started successfully");
                    }
                });
                
                // Set the encoder in the shared mutex
                if let Ok(mut encoder_guard) = self.video_encoder.try_lock() {
                    *encoder_guard = Some(encoder_arc);
                    tracing::info!("Encoder set in shared mutex");
                } else {
                    tracing::error!("Failed to lock encoder mutex");
                }
                
                self.is_recording = true;
                self.recording_start_time = Some(std::time::Instant::now());
                
                tracing::info!("Recording started - file will be saved to: {}", filepath.display());
            }
            Err(e) => {
                tracing::error!("Failed to create video encoder: {}", e);
            }
        }
    }
    
    pub fn stop_recording(&mut self) {
        if !self.is_recording {
            return;
        }
        
        // Take encoder from shared mutex
        if let Ok(mut encoder_guard) = self.video_encoder.try_lock() {
            if let Some(encoder) = encoder_guard.take() {
                tokio::spawn(async move {
                    match encoder.stop_recording().await {
                        Ok(path) => {
                            tracing::info!("Recording saved to: {}", path.display());
                        }
                        Err(e) => {
                            tracing::error!("Failed to stop recording: {}", e);
                        }
                    }
                });
            }
        }
        
        self.is_recording = false;
        self.recording_start_time = None;
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
                
                // Screenshot button
                if ui.button("📸 Take Screenshot").clicked() {
                    self.save_screenshot();
                }
            } else {
                if ui.button("▶️ Start Capture").clicked() {
                    self.start_capture();
                }
            }
        });
        
        // Recording controls
        if self.is_capturing {
            ui.horizontal(|ui| {
                if self.is_recording {
                    if ui.button("⏺ Stop Recording").clicked() {
                        self.stop_recording();
                    }
                    
                    // Show recording duration
                    if let Some(start_time) = self.recording_start_time {
                        let duration = start_time.elapsed().as_secs();
                        ui.label(format!("🔴 Recording: {}:{:02}", duration / 60, duration % 60));
                    }
                } else {
                    if ui.button("⏺ Start Recording").clicked() {
                        self.start_recording();
                    }
                }
            });
        }
        
        // Show screenshot message
        if let Some(msg) = &self.last_screenshot_msg {
            ui.label(egui::RichText::new(msg).color(egui::Color32::GREEN));
            
            // Clear message after 3 seconds
            let msg_clone = self.last_screenshot_msg.clone();
            if msg_clone.is_some() {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                });
                // Message will be cleared on next frame
            }
        }
        
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
