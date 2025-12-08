use eframe::egui;
use genxlink_client_core::access_control::{
    AccessControlManager, AccessSession, AuditEvent,
    SessionStatus, AuditEventType, AuditLevel, AccessControlConfig
};
use genxlink_client_core::permission_profiles::{Permission, PermissionCategory};
use std::time::{Duration, SystemTime};

/// Enhanced access control panel UI
#[derive(Default)]
pub struct AccessControlPanel {
    manager: AccessControlManager,
    selected_tab: AccessControlTab,
    selected_session: Option<String>,
    show_create_policy: bool,
    new_policy: NewPolicyForm,
    permission_request: PermissionRequestForm,
    filter_sessions: SessionFilter,
    filter_audit: AuditFilter,
    search_query: String,
}

/// UI tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessControlTab {
    Sessions,
    Permissions,
    Policies,
    AuditLog,
    Settings,
}

impl Default for AccessControlTab {
    fn default() -> Self {
        Self::Sessions
    }
}

/// Session filter options
#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub status: Option<SessionStatus>,
    pub show_temporary_only: bool,
}

/// Audit log filter options
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub event_type: Option<AuditEventType>,
    pub level: Option<AuditLevel>,
}

/// New policy form
#[derive(Debug, Clone, Default)]
pub struct NewPolicyForm {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub priority: u8,
}

/// Permission request form
#[derive(Debug, Clone, Default)]
pub struct PermissionRequestForm {
    pub session_id: String,
    pub permission: Option<Permission>,
    pub reason: String,
    pub temporary: bool,
    pub duration_minutes: u64,
}

impl AccessControlPanel {
    pub fn new() -> Self {
        let config = AccessControlConfig::default();
        Self {
            manager: AccessControlManager::new(config),
            ..Default::default()
        }
    }

    /// Render the access control panel
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🔐 Access Control");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.manager.cleanup();
                }
            });
        });

        ui.add_space(8.0);

        // Tab selection
        ui.horizontal(|ui| {
            for tab in [
                AccessControlTab::Sessions,
                AccessControlTab::Permissions,
                AccessControlTab::Policies,
                AccessControlTab::AuditLog,
                AccessControlTab::Settings,
            ] {
                let selected = self.selected_tab == tab;
                let button_text = match tab {
                    AccessControlTab::Sessions => "📱 Sessions",
                    AccessControlTab::Permissions => "🔑 Permissions",
                    AccessControlTab::Policies => "📋 Policies",
                    AccessControlTab::AuditLog => "📊 Audit Log",
                    AccessControlTab::Settings => "⚙️ Settings",
                };
                
                if ui.selectable_label(selected, button_text).clicked() {
                    self.selected_tab = tab;
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();

        match self.selected_tab {
            AccessControlTab::Sessions => self.render_sessions_tab(ui),
            AccessControlTab::Permissions => self.render_permissions_tab(ui),
            AccessControlTab::Policies => self.render_policies_tab(ui),
            AccessControlTab::AuditLog => self.render_audit_log_tab(ui),
            AccessControlTab::Settings => self.render_settings_tab(ui),
        }
    }

    /// Render sessions management tab
    fn render_sessions_tab(&mut self, ui: &mut egui::Ui) {
        // Filters
        ui.horizontal(|ui| {
            ui.label("Filter:");
            
            egui::ComboBox::from_label("Status:")
                .selected_text(format!("{:?}", self.filter_sessions.status.unwrap_or(SessionStatus::Active)))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.filter_sessions.status.is_none(), "All").clicked() {
                        self.filter_sessions.status = None;
                    }
                    for status in [SessionStatus::Active, SessionStatus::Paused, SessionStatus::Suspended] {
                        if ui.selectable_label(
                            self.filter_sessions.status == Some(status),
                            format!("{:?}", status)
                        ).clicked() {
                            self.filter_sessions.status = Some(status);
                        }
                    }
                });

            ui.checkbox(&mut self.filter_sessions.show_temporary_only, "Temporary only");
        });

        ui.add_space(8.0);

        // Sessions list
        let sessions: Vec<AccessSession> = self.manager.get_active_sessions().into_iter()
            .filter(|session| {
                if let Some(filter_status) = self.filter_sessions.status {
                    if session.status != filter_status {
                        return false;
                    }
                }
                if self.filter_sessions.show_temporary_only && session.temporary_permissions.is_empty() {
                    return false;
                }
                true
            })
            .cloned()
            .collect();

        if sessions.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("No active sessions");
                ui.add_space(20.0);
            });
        } else {
            for session in sessions.iter() {
                self.render_session_item(ui, session);
                ui.add_space(4.0);
            }
        }
    }

    /// Render a single session item
    fn render_session_item(&mut self, ui: &mut egui::Ui, session: &AccessSession) {
        let is_selected = self.selected_session.as_ref() == Some(&session.id);
        
        let frame = egui::Frame::none()
            .fill(if is_selected {
                egui::Color32::from_rgb(230, 240, 250)
            } else {
                ui.visuals().faint_bg_color
            })
            .stroke(if is_selected {
                egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE)
            } else {
                egui::Stroke::NONE
            })
            .rounding(6.0)
            .inner_margin(12.0);

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Session icon and status
                let status_icon = match session.status {
                    SessionStatus::Active => "🟢",
                    SessionStatus::Paused => "⏸️",
                    SessionStatus::Suspended => "⛔",
                    _ => "❌",
                };
                ui.label(egui::RichText::new(status_icon).size(20.0));

                ui.add_space(8.0);

                // Session info
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&session.metadata.device_name).strong());
                        ui.label(format!("({})", &session.id[..8]));
                        
                        // Status badge
                        let status_color = match session.status {
                            SessionStatus::Active => egui::Color32::GREEN,
                            SessionStatus::Paused => egui::Color32::YELLOW,
                            SessionStatus::Suspended => egui::Color32::RED,
                            _ => egui::Color32::GRAY,
                        };
                        ui.colored_label(status_color, format!("{:?}", session.status));
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("Remote: {}", session.remote_device_id));
                        ui.label("•");
                        ui.label(format!("Type: {:?}", session.metadata.connection_type));
                        
                        if session.metadata.encryption_enabled {
                            ui.label("•");
                            ui.colored_label(egui::Color32::GREEN, "🔒 Encrypted");
                        }
                    });
                });

                // Action buttons
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if session.status == SessionStatus::Active {
                        if ui.button("⏸ Pause").clicked() {
                            // TODO: Pause session
                        }
                        if ui.button("❌ Terminate").clicked() {
                            let _ = self.manager.terminate_session(&session.id);
                        }
                    }
                    
                    if ui.button("📋 Details").clicked() {
                        self.selected_session = Some(session.id.clone());
                    }
                });
            });
        });
    }

    /// Render permissions management tab
    fn render_permissions_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Permission Management");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕ Grant Permission").clicked() {
                    // Show permission request dialog
                }
            });
        });

        ui.add_space(8.0);

        // Permission request form
        egui::Frame::none()
            .fill(ui.visuals().faint_bg_color)
            .rounding(6.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading("Grant Temporary Permission");
                
                ui.horizontal(|ui| {
                    ui.label("Session ID:");
                    ui.text_edit_singleline(&mut self.permission_request.session_id);
                });

                ui.horizontal(|ui| {
                    ui.label("Permission:");
                    if let Some(selected) = &self.permission_request.permission {
                        ui.label(selected.name());
                    } else {
                        ui.label("Select...");
                    }
                    
                    ui.menu_button("Select", |ui| {
                        for category in [
                            PermissionCategory::Audio,
                            PermissionCategory::Control,
                            PermissionCategory::Privacy,
                            PermissionCategory::Files,
                            PermissionCategory::System,
                            PermissionCategory::Recording,
                            PermissionCategory::Advanced,
                        ] {
                            ui.heading(format!("{:?}", category));
                            for permission in self.get_permissions_by_category(category) {
                                if ui.button(permission.name()).clicked() {
                                    self.permission_request.permission = Some(permission);
                                    ui.close_menu();
                                }
                            }
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("Reason:");
                    ui.text_edit_singleline(&mut self.permission_request.reason);
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.permission_request.temporary, "Temporary");
                    if self.permission_request.temporary {
                        ui.add(egui::Slider::new(&mut self.permission_request.duration_minutes, 1..=120)
                            .text("Duration (minutes)"));
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Grant").clicked() {
                        if let Some(permission) = &self.permission_request.permission {
                            if !self.permission_request.session_id.is_empty() {
                                let duration = Duration::from_secs(self.permission_request.duration_minutes * 60);
                                let _ = self.manager.grant_temporary_permission(
                                    &self.permission_request.session_id,
                                    permission.clone(),
                                    duration,
                                    "admin".to_string(),
                                    self.permission_request.reason.clone(),
                                );
                                
                                // Reset form
                                self.permission_request = PermissionRequestForm::default();
                            }
                        }
                    }
                    
                    if ui.button("Clear").clicked() {
                        self.permission_request = PermissionRequestForm::default();
                    }
                });
            });
    }

    /// Render policies management tab
    fn render_policies_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Access Policies");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕ Create Policy").clicked() {
                    self.show_create_policy = !self.show_create_policy;
                }
            });
        });

        ui.add_space(8.0);

        // Placeholder for policies list
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label("No policies configured");
            ui.label("Create a policy to automate permission management");
            ui.add_space(20.0);
        });
    }

    /// Render audit log tab
    fn render_audit_log_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Audit Log");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{} events", self.manager.get_audit_log().len()));
            });
        });

        ui.add_space(8.0);

        // Placeholder for audit log
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label("Audit log will be displayed here");
            ui.add_space(20.0);
        });
    }

    /// Render settings tab
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Access Control Settings");
        ui.add_space(16.0);

        ui.vertical_centered(|ui| {
            ui.label("Settings configuration will be displayed here");
        });
    }

    /// Get permissions by category
    fn get_permissions_by_category(&self, category: PermissionCategory) -> Vec<Permission> {
        match category {
            PermissionCategory::Audio => vec![Permission::HearDeviceSound],
            PermissionCategory::Control => vec![
                Permission::ControlDevice,
                Permission::RestartDevice,
                Permission::SendCtrlAltDel,
                Permission::BlockInputDevices,
                Permission::LockDevice,
                Permission::SignOutUser,
            ],
            PermissionCategory::Privacy => vec![
                Permission::EnablePrivacyMode,
                Permission::ShowColoredCursor,
            ],
            PermissionCategory::Files => vec![
                Permission::AccessClipboard,
                Permission::AccessClipboardForFileTransfer,
                Permission::UseFileManager,
            ],
            PermissionCategory::System => vec![
                Permission::SeeSystemInformation,
                Permission::DrawOnScreen,
                Permission::CreateTcpTunnels,
            ],
            PermissionCategory::Recording => vec![Permission::RecordSession],
            PermissionCategory::Advanced => vec![Permission::InteractWithRestrictedWindows],
        }
    }
}
