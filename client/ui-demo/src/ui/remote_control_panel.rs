use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use genxlink_client_core::remote_control_manager::RemoteControlManager;
use genxlink_protocol::{DeviceId, RemoteControlState};

/// Remote control panel state
pub struct RemoteControlPanel {
    enabled: bool,
    permission_level: PermissionLevel,
    event_count: u64,
    show_settings: bool,
    
    // Remote control manager
    manager: Arc<Mutex<Option<RemoteControlManager>>>,
    remote_device_id: String,
    current_state: RemoteControlState,
    auto_accept: bool,
}

impl Default for RemoteControlPanel {
    fn default() -> Self {
        Self {
            enabled: false,
            permission_level: PermissionLevel::default(),
            event_count: 0,
            show_settings: false,
            manager: Arc::new(Mutex::new(None)),
            remote_device_id: String::new(),
            current_state: RemoteControlState::Idle,
            auto_accept: false,
        }
    }
}

/// Permission level for remote control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    ViewOnly,
    MouseOnly,
    KeyboardOnly,
    FullControl,
}

impl Default for PermissionLevel {
    fn default() -> Self {
        Self::ViewOnly
    }
}

impl PermissionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ViewOnly => "View Only",
            Self::MouseOnly => "Mouse Only",
            Self::KeyboardOnly => "Keyboard Only",
            Self::FullControl => "Full Control",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::ViewOnly => "👁",
            Self::MouseOnly => "🖱",
            Self::KeyboardOnly => "⌨",
            Self::FullControl => "🎮",
        }
    }

    pub fn all() -> [PermissionLevel; 4] {
        [
            Self::ViewOnly,
            Self::MouseOnly,
            Self::KeyboardOnly,
            Self::FullControl,
        ]
    }
}

impl RemoteControlPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update event count
    pub fn set_event_count(&mut self, count: u64) {
        self.event_count = count;
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get permission level
    pub fn permission_level(&self) -> PermissionLevel {
        self.permission_level
    }
    
    /// Initialize remote control manager
    pub fn initialize(&mut self, device_id: DeviceId) {
        let manager = RemoteControlManager::new(device_id);
        let manager_arc = Arc::new(Mutex::new(Some(manager)));
        self.manager = manager_arc;
        tracing::info!("Remote control manager initialized");
    }
    
    /// Request control of remote device
    pub fn request_control(&mut self) {
        if self.remote_device_id.is_empty() {
            tracing::warn!("No remote device ID specified");
            return;
        }
        
        let manager = self.manager.clone();
        let remote_id = DeviceId(self.remote_device_id.clone());
        
        tokio::spawn(async move {
            let mut guard = manager.lock().await;
            if let Some(mgr) = guard.as_mut() {
                match mgr.request_control(remote_id).await {
                    Ok(response) => {
                        if response.granted {
                            tracing::info!("Remote control granted!");
                        } else {
                            tracing::warn!("Remote control denied: {:?}", response.reason);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to request control: {}", e);
                    }
                }
            }
        });
    }
    
    /// End remote control session
    pub fn end_session(&mut self) {
        let manager = self.manager.clone();
        
        tokio::spawn(async move {
            let mut guard = manager.lock().await;
            if let Some(mgr) = guard.as_mut() {
                if let Err(e) = mgr.end_session().await {
                    tracing::error!("Failed to end session: {}", e);
                }
            }
        });
        
        self.current_state = RemoteControlState::Ended;
    }

    /// Render the remote control panel
    pub fn ui(&mut self, ui: &mut egui::Ui) -> RemoteControlAction {
        let mut action = RemoteControlAction::None;

        ui.group(|ui| {
            ui.heading("🎮 Remote Control");
            ui.separator();

            // Enable/Disable toggle
            ui.horizontal(|ui| {
                ui.label("Status:");
                let status_text = if self.enabled { "🟢 Enabled" } else { "🔴 Disabled" };
                let status_color = if self.enabled {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                };
                ui.colored_label(status_color, status_text);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let button_text = if self.enabled { "Disable" } else { "Enable" };
                    if ui.button(button_text).clicked() {
                        self.enabled = !self.enabled;
                        action = if self.enabled {
                            RemoteControlAction::Enable
                        } else {
                            RemoteControlAction::Disable
                        };
                    }
                });
            });

            ui.add_space(8.0);

            // Permission level selector
            ui.horizontal(|ui| {
                ui.label("Permissions:");
                egui::ComboBox::from_id_source("permission_level")
                    .selected_text(format!("{} {}", 
                        self.permission_level.icon(), 
                        self.permission_level.as_str()))
                    .show_ui(ui, |ui| {
                        for level in PermissionLevel::all() {
                            let text = format!("{} {}", level.icon(), level.as_str());
                            if ui.selectable_value(&mut self.permission_level, level, text).clicked() {
                                action = RemoteControlAction::ChangePermission(level);
                            }
                        }
                    });
            });

            ui.add_space(8.0);

            // Statistics
            ui.horizontal(|ui| {
                ui.label("Events processed:");
                ui.colored_label(egui::Color32::LIGHT_BLUE, format!("{}", self.event_count));
            });

            ui.add_space(8.0);

            // Settings toggle
            if ui.button("⚙ Settings").clicked() {
                self.show_settings = !self.show_settings;
            }

            // Settings panel
            if self.show_settings {
                ui.separator();
                ui.collapsing("Advanced Settings", |ui| {
                    ui.label("🔒 Security");
                    ui.checkbox(&mut true, "Require confirmation for file transfers");
                    ui.checkbox(&mut true, "Show notification on control events");
                    
                    ui.add_space(4.0);
                    ui.label("⚡ Performance");
                    ui.checkbox(&mut true, "Enable hardware acceleration");
                    ui.checkbox(&mut false, "Reduce input latency");
                });
            }

            // Quick actions
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("📋 Copy Stats").clicked() {
                    action = RemoteControlAction::CopyStats;
                }
                if ui.button("🔄 Reset Counter").clicked() {
                    self.event_count = 0;
                    action = RemoteControlAction::ResetCounter;
                }
            });
        });

        action
    }
}

/// Actions that can be triggered from the remote control panel
#[derive(Debug, Clone, PartialEq)]
pub enum RemoteControlAction {
    None,
    Enable,
    Disable,
    ChangePermission(PermissionLevel),
    CopyStats,
    ResetCounter,
}
