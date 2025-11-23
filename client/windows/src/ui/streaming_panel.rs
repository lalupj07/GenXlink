use egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use genxlink_client_core::webrtc_session::{WebRTCSession, SessionState};
use genxlink_protocol::DeviceId;

/// Streaming panel for WebRTC screen sharing
pub struct StreamingPanel {
    session: Arc<Mutex<Option<WebRTCSession>>>,
    signaling_server_url: String,
    remote_device_id: String,
    selected_monitor: usize,
    session_state: SessionState,
    error_message: Option<String>,
}

impl StreamingPanel {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            signaling_server_url: "ws://localhost:8080/signaling".to_string(),
            remote_device_id: String::new(),
            selected_monitor: 0,
            session_state: SessionState::Idle,
            error_message: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌐 WebRTC Streaming");
        ui.add_space(10.0);

        // Connection status
        self.show_status(ui);
        ui.add_space(10.0);

        // Configuration section
        ui.group(|ui| {
            ui.label("⚙️ Configuration");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Signaling Server:");
                ui.text_edit_singleline(&mut self.signaling_server_url);
            });

            ui.horizontal(|ui| {
                ui.label("Remote Device ID:");
                ui.text_edit_singleline(&mut self.remote_device_id);
            });

            ui.horizontal(|ui| {
                ui.label("Monitor:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("Monitor {}", self.selected_monitor))
                    .show_ui(ui, |ui| {
                        for i in 0..4 {
                            ui.selectable_value(&mut self.selected_monitor, i, format!("Monitor {}", i));
                        }
                    });
            });
        });

        ui.add_space(10.0);

        // Control buttons
        ui.horizontal(|ui| {
            match self.session_state {
                SessionState::Idle | SessionState::Disconnected | SessionState::Failed(_) => {
                    if ui.button("🚀 Start Streaming").clicked() {
                        self.start_streaming();
                    }
                }
                SessionState::Streaming | SessionState::Connected => {
                    if ui.button("⏹ Stop Streaming").clicked() {
                        self.stop_streaming();
                    }
                }
                _ => {
                    ui.add_enabled(false, egui::Button::new("⏳ Connecting..."));
                }
            }
        });

        // Error message
        if let Some(error) = &self.error_message {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        }

        // Instructions
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);
        
        ui.collapsing("📖 Instructions", |ui| {
            ui.label("1. Make sure the signaling server is running");
            ui.label("2. Enter the remote device ID you want to connect to");
            ui.label("3. Select which monitor to share");
            ui.label("4. Click 'Start Streaming' to begin");
            ui.add_space(5.0);
            ui.label("💡 The remote device will receive your screen in real-time!");
        });
    }

    fn show_status(&mut self, ui: &mut egui::Ui) {
        let (status_text, status_color) = match &self.session_state {
            SessionState::Idle => ("⚪ Idle", egui::Color32::GRAY),
            SessionState::ConnectingToSignaling => ("🔵 Connecting to signaling server...", egui::Color32::BLUE),
            SessionState::SignalingConnected => ("🟢 Signaling connected", egui::Color32::GREEN),
            SessionState::CreatingOffer => ("🔵 Creating offer...", egui::Color32::BLUE),
            SessionState::WaitingForAnswer => ("🟡 Waiting for answer...", egui::Color32::YELLOW),
            SessionState::GatheringCandidates => ("🟡 Gathering ICE candidates...", egui::Color32::YELLOW),
            SessionState::Connected => ("🟢 Peer connected", egui::Color32::GREEN),
            SessionState::Streaming => ("🟢 Streaming active!", egui::Color32::from_rgb(0, 200, 0)),
            SessionState::Disconnecting => ("🟡 Disconnecting...", egui::Color32::YELLOW),
            SessionState::Disconnected => ("⚪ Disconnected", egui::Color32::GRAY),
            SessionState::Failed(err) => {
                self.error_message = Some(err.clone());
                ("🔴 Failed", egui::Color32::RED)
            }
        };

        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.colored_label(status_color, status_text);
        });
    }

    fn start_streaming(&mut self) {
        // Validate inputs
        if self.remote_device_id.is_empty() {
            self.error_message = Some("Please enter a remote device ID".to_string());
            return;
        }

        self.error_message = None;
        self.session_state = SessionState::ConnectingToSignaling;

        // Create device ID (in real app, this would be from config)
        let device_id = DeviceId::new();
        
        // Create session
        let session = WebRTCSession::new(
            device_id,
            self.signaling_server_url.clone(),
        );

        // Start streaming in background
        let remote_id = DeviceId(self.remote_device_id.clone());
        let monitor_index = self.selected_monitor;
        
        tokio::spawn(async move {
            if let Err(e) = session.start_streaming(monitor_index, remote_id).await {
                tracing::error!("Failed to start streaming: {}", e);
            }
        });

        tracing::info!("Streaming session started");
    }

    fn stop_streaming(&mut self) {
        self.session_state = SessionState::Disconnecting;
        
        // For now, just update state
        // In a real implementation, we'd store the session and call stop on it
        
        self.session_state = SessionState::Disconnected;
        tracing::info!("Streaming session stopped");
    }

    /// Update session state from the WebRTC session
    pub fn update(&mut self) {
        // State updates would happen via callbacks in a real implementation
        // For now, this is a no-op
    }
}
