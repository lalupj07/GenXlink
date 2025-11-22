use std::time::Instant;

/// Connection dialog state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionDialogState {
    Hidden,
    Input,
    Connecting {
        device_name: String,
        started_at: Instant,
    },
    Failed {
        device_name: String,
        error: String,
    },
}

/// Connection progress steps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStep {
    Initializing,
    ConnectingToSignaling,
    ExchangingOffer,
    GatheringCandidates,
    EstablishingConnection,
    Connected,
}

impl ConnectionStep {
    pub fn progress(&self) -> f32 {
        match self {
            ConnectionStep::Initializing => 0.0,
            ConnectionStep::ConnectingToSignaling => 0.2,
            ConnectionStep::ExchangingOffer => 0.4,
            ConnectionStep::GatheringCandidates => 0.6,
            ConnectionStep::EstablishingConnection => 0.8,
            ConnectionStep::Connected => 1.0,
        }
    }
    
    pub fn message(&self) -> &'static str {
        match self {
            ConnectionStep::Initializing => "Initializing connection...",
            ConnectionStep::ConnectingToSignaling => "Connecting to signaling server...",
            ConnectionStep::ExchangingOffer => "Exchanging connection details...",
            ConnectionStep::GatheringCandidates => "Finding best connection path...",
            ConnectionStep::EstablishingConnection => "Establishing peer connection...",
            ConnectionStep::Connected => "Connected successfully!",
        }
    }
}

/// Connection dialog
pub struct ConnectionDialog {
    state: ConnectionDialogState,
    current_step: ConnectionStep,
    device_address: String,
    device_name_input: String,
}

impl Default for ConnectionDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionDialog {
    pub fn new() -> Self {
        Self {
            state: ConnectionDialogState::Hidden,
            current_step: ConnectionStep::Initializing,
            device_address: String::new(),
            device_name_input: String::new(),
        }
    }
    
    pub fn show_dialog(&mut self) {
        self.state = ConnectionDialogState::Input;
        self.device_address.clear();
        self.device_name_input.clear();
    }
    
    pub fn show_connecting(&mut self, device_name: String) {
        self.state = ConnectionDialogState::Connecting {
            device_name,
            started_at: Instant::now(),
        };
        self.current_step = ConnectionStep::Initializing;
    }
    
    pub fn show_failed(&mut self, device_name: String, error: String) {
        self.state = ConnectionDialogState::Failed {
            device_name,
            error,
        };
    }
    
    pub fn hide(&mut self) {
        self.state = ConnectionDialogState::Hidden;
    }
    
    pub fn set_step(&mut self, step: ConnectionStep) {
        self.current_step = step;
    }
    
    pub fn is_visible(&self) -> bool {
        !matches!(self.state, ConnectionDialogState::Hidden)
    }
    
    pub fn show(&mut self, ctx: &egui::Context) -> DialogResult {
        if !self.is_visible() {
            return DialogResult::Continue;
        }
        
        let mut result = DialogResult::Continue;
        
        egui::Window::new("connection_dialog")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([400.0, 200.0])
            .show(ctx, |ui| {
                match &self.state {
                    ConnectionDialogState::Input => {
                        result = self.show_input_ui(ui);
                    }
                    ConnectionDialogState::Connecting { device_name, started_at } => {
                        result = self.show_connecting_ui(ui, device_name, started_at);
                    }
                    ConnectionDialogState::Failed { device_name, error } => {
                        result = self.show_failed_ui(ui, device_name, error);
                    }
                    ConnectionDialogState::Hidden => {}
                }
            });
        
        result
    }
    
    fn show_input_ui(&mut self, ui: &mut egui::Ui) -> DialogResult {
        let mut result = DialogResult::Continue;
        
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // Title
            ui.heading("Connect to Remote Device");
            ui.add_space(20.0);
            
            // Device ID input (privacy-focused, no IP addresses)
            ui.horizontal(|ui| {
                ui.label("Device ID:");
                ui.add_space(5.0);
            });
            ui.add_space(5.0);
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.device_address)
                    .hint_text("e.g., ABC-123-XYZ or device-unique-id")
                    .desired_width(300.0)
            );
            
            // Auto-focus on first show
            if response.gained_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                response.request_focus();
            }
            
            ui.add_space(15.0);
            
            // Optional device name
            ui.horizontal(|ui| {
                ui.label("Device Name (optional):");
                ui.add_space(5.0);
            });
            ui.add_space(5.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.device_name_input)
                    .hint_text("e.g., Work Laptop")
                    .desired_width(300.0)
            );
            
            ui.add_space(25.0);
            
            // Buttons
            ui.horizontal(|ui| {
                let can_connect = !self.device_address.trim().is_empty();
                
                if ui.add_enabled(can_connect, egui::Button::new("Connect")).clicked() {
                    // Start connection
                    let device_name = if self.device_name_input.trim().is_empty() {
                        self.device_address.clone()
                    } else {
                        self.device_name_input.clone()
                    };
                    self.show_connecting(device_name);
                }
                
                if ui.button("Cancel").clicked() {
                    result = DialogResult::Cancel;
                }
            });
            
            ui.add_space(15.0);
            
            // Help text
            ui.label(
                egui::RichText::new("🔒 Privacy: Only Device IDs are used (no IP addresses exposed)")
                    .small()
                    .color(egui::Color32::from_rgb(156, 163, 175))
            );
        });
        
        result
    }
    
    fn show_connecting_ui(&self, ui: &mut egui::Ui, device_name: &str, started_at: &Instant) -> DialogResult {
        let mut result = DialogResult::Continue;
        
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // Title
            ui.heading(format!("Connecting to {}", device_name));
            ui.add_space(20.0);
            
            // Spinner
            ui.spinner();
            ui.add_space(10.0);
            
            // Progress bar
            let progress = self.current_step.progress();
            ui.add(
                egui::ProgressBar::new(progress)
                    .text(format!("{}%", (progress * 100.0) as u32))
                    .animate(true)
            );
            ui.add_space(10.0);
            
            // Status message
            ui.label(self.current_step.message());
            ui.add_space(5.0);
            
            // Elapsed time
            let elapsed = started_at.elapsed().as_secs();
            ui.label(
                egui::RichText::new(format!("Elapsed: {}s", elapsed))
                    .small()
                    .color(egui::Color32::from_rgb(156, 163, 175))
            );
            
            ui.add_space(20.0);
            
            // Cancel button
            if ui.button("Cancel").clicked() {
                result = DialogResult::Cancel;
            }
        });
        
        result
    }
    
    fn show_failed_ui(&self, ui: &mut egui::Ui, device_name: &str, error: &str) -> DialogResult {
        let mut result = DialogResult::Continue;
        
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // Error icon
            ui.label(egui::RichText::new("❌").size(48.0));
            ui.add_space(10.0);
            
            // Title
            ui.heading(format!("Failed to connect to {}", device_name));
            ui.add_space(10.0);
            
            // Error message
            ui.label(
                egui::RichText::new(error)
                    .color(egui::Color32::from_rgb(239, 68, 68))
            );
            
            ui.add_space(30.0);
            
            // Buttons
            ui.horizontal(|ui| {
                if ui.button("Retry").clicked() {
                    result = DialogResult::Retry;
                }
                if ui.button("Close").clicked() {
                    result = DialogResult::Close;
                }
            });
        });
        
        result
    }
}

/// Dialog result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogResult {
    Continue,
    Cancel,
    Retry,
    Close,
}
