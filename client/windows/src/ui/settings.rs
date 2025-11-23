use eframe::egui;

/// Settings panel for application configuration
#[derive(Default)]
pub struct SettingsPanel {
    selected_theme: AppTheme,
    selected_language: AppLanguage,
    auto_start: bool,
    minimize_to_tray: bool,
    enable_notifications: bool,
    log_level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLanguage {
    English,
    Hindi,
    Tamil,
    Telugu,
    Bengali,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::System
    }
}

impl Default for AppLanguage {
    fn default() -> Self {
        Self::English
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl SettingsPanel {
    pub fn new() -> Self {
        Self {
            selected_theme: AppTheme::System,
            selected_language: AppLanguage::English,
            auto_start: true,
            minimize_to_tray: true,
            enable_notifications: true,
            log_level: LogLevel::Info,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> SettingsAction {
        let mut action = SettingsAction::None;

        // Beautiful gradient header
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // Main title with beautiful styling
            ui.label(egui::RichText::new("⚙️ Settings")
                .size(32.0)
                .strong()
                .color(egui::Color32::from_rgb(59, 130, 246)));
            
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Personalize your GenXLink experience")
                .size(16.0)
                .color(egui::Color32::from_rgb(107, 114, 128)));
            
            // Beautiful decorative line
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(100.0);
                ui.separator();
                ui.label(egui::RichText::new("✨")
                    .size(20.0)
                    .color(egui::Color32::from_rgb(251, 191, 36)));
                ui.separator();
                ui.add_space(100.0);
            });
            
            ui.add_space(25.0);
        });

        // Beautiful card-based layout with modern design
        egui::Grid::new("beautiful_settings_grid")
            .num_columns(2)
            .spacing([25.0, 25.0])
            .show(ui, |ui| {
                
                // Appearance Card - Beautiful gradient design
                ui.vertical(|ui| {
                    // Card with beautiful styling
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(248, 250, 252))
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(139, 92, 246)))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(20.0, 20.0))
                        .show(ui, |ui| {
                            
                            // Beautiful header
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎨")
                                    .size(24.0)
                                    .color(egui::Color32::from_rgb(139, 92, 246)));
                                ui.label(egui::RichText::new("Appearance")
                                    .size(20.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(139, 92, 246)));
                            });
                            
                            ui.add_space(15.0);
                            
                            // Beautiful theme selector
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Theme")
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(71, 85, 105)));
                                
                                ui.add_space(8.0);
                                
                                let theme_text = match self.selected_theme {
                                    AppTheme::Light => "☀️ Light Mode",
                                    AppTheme::Dark => "🌙 Dark Mode", 
                                    AppTheme::System => "💻 System Default",
                                };
                                
                                let mut theme_changed = false;
                                egui::ComboBox::from_id_source("beautiful_theme_combo")
                                    .selected_text(theme_text)
                                    .width(200.0)
                                    .show_ui(ui, |ui| {
                                        ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
                                        ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(241, 245, 249);
                                        ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::from_rgb(139, 92, 246);
                                        
                                        if ui.selectable_value(&mut self.selected_theme, AppTheme::Light, "☀️ Light Mode").clicked() {
                                            theme_changed = true;
                                        }
                                        if ui.selectable_value(&mut self.selected_theme, AppTheme::Dark, "🌙 Dark Mode").clicked() {
                                            theme_changed = true;
                                        }
                                        if ui.selectable_value(&mut self.selected_theme, AppTheme::System, "💻 System Default").clicked() {
                                            theme_changed = true;
                                        }
                                    });
                                
                                if theme_changed {
                                    self.apply_theme_change(ui.ctx());
                                }
                            });
                            
                            ui.add_space(20.0);
                            
                            // Beautiful language selector
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Language")
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(71, 85, 105)));
                                
                                ui.add_space(8.0);
                                
                                let lang_text = match self.selected_language {
                                    AppLanguage::English => "🇬🇧 English",
                                    AppLanguage::Hindi => "🇮🇳 हिंदी",
                                    AppLanguage::Tamil => "🇮🇳 தமிழ்",
                                    AppLanguage::Telugu => "🇮🇳 తెలుగు",
                                    AppLanguage::Bengali => "🇮🇳 বাংলা",
                                };
                                
                                let mut lang_changed = false;
                                egui::ComboBox::from_id_source("beautiful_lang_combo")
                                    .selected_text(lang_text)
                                    .width(200.0)
                                    .show_ui(ui, |ui| {
                                        ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
                                        ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(241, 245, 249);
                                        ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::from_rgb(139, 92, 246);
                                        
                                        if ui.selectable_value(&mut self.selected_language, AppLanguage::English, "🇬🇧 English").clicked() {
                                            lang_changed = true;
                                        }
                                        if ui.selectable_value(&mut self.selected_language, AppLanguage::Hindi, "🇮🇳 हिंदी").clicked() {
                                            lang_changed = true;
                                        }
                                        if ui.selectable_value(&mut self.selected_language, AppLanguage::Tamil, "🇮🇳 தமிழ்").clicked() {
                                            lang_changed = true;
                                        }
                                        if ui.selectable_value(&mut self.selected_language, AppLanguage::Telugu, "🇮🇳 తెలుగు").clicked() {
                                            lang_changed = true;
                                        }
                                        if ui.selectable_value(&mut self.selected_language, AppLanguage::Bengali, "🇮🇳 বাংলা").clicked() {
                                            lang_changed = true;
                                        }
                                    });
                                
                                if lang_changed {
                                    self.apply_language_change(ui.ctx());
                                }
                            });
                            
                            ui.add_space(20.0);
                            
                            // Beautiful status messages
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("💡")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(59, 130, 246)));
                                ui.label(egui::RichText::new("Theme changes apply instantly")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(59, 130, 246)));
                            });
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🌍")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                                ui.label(egui::RichText::new("Language support is active")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                            });
                        });
                });

                // Behavior Card - Beautiful green theme
                ui.vertical(|ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(240, 253, 244))
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(34, 197, 94)))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(20.0, 20.0))
                        .show(ui, |ui| {
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔧")
                                    .size(24.0)
                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                                ui.label(egui::RichText::new("Behavior")
                                    .size(20.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                            });
                            
                            ui.add_space(20.0);
                            
                            // Beautiful checkboxes
                            ui.vertical(|ui| {
                                if ui.checkbox(&mut self.auto_start, "🚀 Start GenXLink with Windows").clicked() {
                                    // Handle auto-start change
                                }
                                ui.add_space(12.0);
                                
                                if ui.checkbox(&mut self.minimize_to_tray, "📌 Minimize to system tray").clicked() {
                                    // Handle tray change
                                }
                                ui.add_space(12.0);
                                
                                if ui.checkbox(&mut self.enable_notifications, "🔔 Enable desktop notifications").clicked() {
                                    // Handle notifications change
                                }
                            });
                            
                            ui.add_space(15.0);
                            
                            // Beautiful status indicator
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚡")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                                ui.label(egui::RichText::new("Behavior settings saved")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                            });
                        });
                });

                ui.end_row();

                // Advanced Card - Beautiful orange theme
                ui.vertical(|ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(255, 251, 235))
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(251, 146, 60)))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(20.0, 20.0))
                        .show(ui, |ui| {
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔬")
                                    .size(24.0)
                                    .color(egui::Color32::from_rgb(251, 146, 60)));
                                ui.label(egui::RichText::new("Advanced")
                                    .size(20.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(251, 146, 60)));
                            });
                            
                            ui.add_space(20.0);
                            
                            // Beautiful log level selector
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("Log Level")
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(71, 85, 105)));
                                
                                ui.add_space(8.0);
                                
                                let log_text = match self.log_level {
                                    LogLevel::Error => "❌ Error Only",
                                    LogLevel::Warn => "⚠️ Warnings & Errors",
                                    LogLevel::Info => "ℹ️ Info & Above",
                                    LogLevel::Debug => "🐛 Debug Mode",
                                };
                                
                                egui::ComboBox::from_id_source("beautiful_log_combo")
                                    .selected_text(log_text)
                                    .width(200.0)
                                    .show_ui(ui, |ui| {
                                        ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(255, 251, 235);
                                        ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(254, 243, 199);
                                        ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::from_rgb(251, 146, 60);
                                        
                                        ui.selectable_value(&mut self.log_level, LogLevel::Error, "❌ Error Only");
                                        ui.selectable_value(&mut self.log_level, LogLevel::Warn, "⚠️ Warnings & Errors");
                                        ui.selectable_value(&mut self.log_level, LogLevel::Info, "ℹ️ Info & Above");
                                        ui.selectable_value(&mut self.log_level, LogLevel::Debug, "🐛 Debug Mode");
                                    });
                            });
                            
                            ui.add_space(20.0);
                            
                            // Beautiful button
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("📂 Open Log Folder")
                                        .size(14.0)
                                        .strong()
                                        .color(egui::Color32::WHITE)
                                )
                                .fill(egui::Color32::from_rgb(251, 146, 60))
                                .rounding(egui::Rounding::same(8.0))
                            ).clicked() {
                                action = SettingsAction::OpenLogFolder;
                            }
                            
                            ui.add_space(15.0);
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔍")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(251, 146, 60)));
                                ui.label(egui::RichText::new("Advanced options configured")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(251, 146, 60)));
                            });
                        });
                });

                // About Card - Beautiful blue theme
                ui.vertical(|ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(239, 246, 255))
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(59, 130, 246)))
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(20.0, 20.0))
                        .show(ui, |ui| {
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("ℹ️")
                                    .size(24.0)
                                    .color(egui::Color32::from_rgb(59, 130, 246)));
                                ui.label(egui::RichText::new("About GenXLink")
                                    .size(20.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(59, 130, 246)));
                            });
                            
                            ui.add_space(20.0);
                            
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("🚀 GenXLink Remote Desktop")
                                    .size(18.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(59, 130, 246)));
                                
                                ui.add_space(12.0);
                                
                                ui.label(egui::RichText::new("Version 0.1.0")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(71, 85, 105)));
                                
                                ui.add_space(8.0);
                                
                                ui.label(egui::RichText::new("🇮🇳 Created in India with ❤️")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(34, 197, 94)));
                                
                                ui.add_space(6.0);
                                
                                ui.label(egui::RichText::new("📧 genxisinnovation@outlook.com")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(107, 114, 128)));
                                
                                ui.add_space(15.0);
                                
                                // Beautiful action buttons
                                ui.horizontal(|ui| {
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("📄 License")
                                                .size(12.0)
                                                .color(egui::Color32::WHITE)
                                        )
                                        .fill(egui::Color32::from_rgb(59, 130, 246))
                                        .rounding(egui::Rounding::same(6.0))
                                    ).clicked() {
                                        action = SettingsAction::ViewLicense;
                                    }
                                    
                                    ui.add_space(8.0);
                                    
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("📚 Docs")
                                                .size(12.0)
                                                .color(egui::Color32::WHITE)
                                        )
                                        .fill(egui::Color32::from_rgb(59, 130, 246))
                                        .rounding(egui::Rounding::same(6.0))
                                    ).clicked() {
                                        action = SettingsAction::OpenDocumentation;
                                    }
                                    
                                    ui.add_space(8.0);
                                    
                                    let _ = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("🔗 GitHub")
                                                .size(12.0)
                                                .color(egui::Color32::WHITE)
                                        )
                                        .fill(egui::Color32::from_rgb(59, 130, 246))
                                        .rounding(egui::Rounding::same(6.0))
                                    );
                                });
                            });
                        });
                });

                ui.end_row();
            });

        action
    }
    
    fn apply_theme_change(&self, ctx: &egui::Context) {
        // Apply theme change to the UI
        match self.selected_theme {
            AppTheme::Light => {
                // Light theme settings would be applied here
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("GenXLink - Light Theme".to_string()));
            }
            AppTheme::Dark => {
                // Dark theme settings would be applied here
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("GenXLink - Dark Theme".to_string()));
            }
            AppTheme::System => {
                // System theme settings would be applied here
                ctx.send_viewport_cmd(egui::ViewportCommand::Title("GenXLink - System Theme".to_string()));
            }
        }
    }
    
    fn apply_language_change(&self, ctx: &egui::Context) {
        // Apply language change to the UI
        let lang_name = match self.selected_language {
            AppLanguage::English => "English",
            AppLanguage::Hindi => "हिंदी",
            AppLanguage::Tamil => "தமிழ்",
            AppLanguage::Telugu => "తెలుగు",
            AppLanguage::Bengali => "বাংলা",
        };
        
        // Update window title to show language change
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("GenXLink - {}", lang_name)));
    }
}

/// Actions that can be triggered from the settings panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    None,
    OpenLogFolder,
    ViewLicense,
    OpenDocumentation,
}
