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

        // Modern header with gradient feel
        ui.vertical_centered(|ui| {
            ui.add_space(12.0);
            ui.label(egui::RichText::new("⚙️ Settings")
                .size(24.0)
                .strong()
                .color(egui::Color32::WHITE));
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Configure your GenXLink experience")
                .size(13.0)
                .color(egui::Color32::from_rgb(156, 163, 175)));
            ui.add_space(20.0);
        });

        // Modern card-based grid layout
        egui::Grid::new("settings_grid")
            .num_columns(2)
            .spacing([20.0, 20.0])
            .show(ui, |ui| {
                
                // Appearance Section
                ui.vertical(|ui| {
                    ui.heading("🎨 Appearance");
                    ui.add_space(8.0);
                    
                    // Theme
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        ui.add_space(8.0);
                        
                        let theme_text = match self.selected_theme {
                            AppTheme::Light => "☀️ Light",
                            AppTheme::Dark => "🌙 Dark", 
                            AppTheme::System => "💻 System",
                        };
                        
                        let mut theme_changed = false;
                        egui::ComboBox::from_id_source("theme_combo")
                            .selected_text(theme_text)
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut self.selected_theme, AppTheme::Light, "☀️ Light").clicked() {
                                    theme_changed = true;
                                }
                                if ui.selectable_value(&mut self.selected_theme, AppTheme::Dark, "🌙 Dark").clicked() {
                                    theme_changed = true;
                                }
                                if ui.selectable_value(&mut self.selected_theme, AppTheme::System, "💻 System").clicked() {
                                    theme_changed = true;
                                }
                            });
                        
                        // Apply theme change immediately
                        if theme_changed {
                            self.apply_theme_change(ui.ctx());
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // Language
                    ui.horizontal(|ui| {
                        ui.label("Language:");
                        ui.add_space(8.0);
                        
                        let lang_text = match self.selected_language {
                            AppLanguage::English => "🇬🇧 English",
                            AppLanguage::Hindi => "🇮🇳 हिंदी",
                            AppLanguage::Tamil => "🇮🇳 தமிழ்",
                            AppLanguage::Telugu => "🇮🇳 తెలుగు",
                            AppLanguage::Bengali => "🇮🇳 বাংলা",
                        };
                        
                        let mut lang_changed = false;
                        egui::ComboBox::from_id_source("lang_combo")
                            .selected_text(lang_text)
                            .show_ui(ui, |ui| {
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
                        
                        // Apply language change immediately
                        if lang_changed {
                            self.apply_language_change(ui.ctx());
                        }
                    });
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);
                    
                    ui.label(egui::RichText::new("💡 Theme changes apply immediately")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(107, 114, 128)));
                    ui.label(egui::RichText::new("🌍 Language support is active")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(34, 197, 94)));
                });

                // Behavior Section
                ui.vertical(|ui| {
                    ui.heading("🔧 Behavior");
                    ui.add_space(8.0);
                    
                    ui.checkbox(&mut self.auto_start, "Start with Windows");
                    ui.add_space(6.0);
                    ui.checkbox(&mut self.minimize_to_tray, "Minimize to tray");
                    ui.add_space(6.0);
                    ui.checkbox(&mut self.enable_notifications, "Desktop notifications");
                });

                ui.end_row();

                // Advanced Section
                ui.vertical(|ui| {
                    ui.heading("🔬 Advanced");
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Log Level:");
                        ui.add_space(8.0);
                        
                        let log_text = match self.log_level {
                            LogLevel::Error => "❌ Error",
                            LogLevel::Warn => "⚠️ Warning",
                            LogLevel::Info => "ℹ️ Info",
                            LogLevel::Debug => "🐛 Debug",
                        };
                        
                        egui::ComboBox::from_id_source("log_combo")
                            .selected_text(log_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.log_level, LogLevel::Error, "❌ Error");
                                ui.selectable_value(&mut self.log_level, LogLevel::Warn, "⚠️ Warning");
                                ui.selectable_value(&mut self.log_level, LogLevel::Info, "ℹ️ Info");
                                ui.selectable_value(&mut self.log_level, LogLevel::Debug, "🐛 Debug");
                            });
                    });

                    ui.add_space(8.0);
                    
                    if ui.button("📂 Open Log Folder").clicked() {
                        action = SettingsAction::OpenLogFolder;
                    }
                });

                // About Section - Redesigned with modern card feel
                ui.vertical_centered(|ui| {
                    ui.add_space(5.0);
                    
                    // About heading with icon
                    ui.label(egui::RichText::new("ℹ️ About")
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::WHITE));
                    
                    ui.add_space(12.0);
                    
                    // App name with gradient-like styling
                    ui.label(egui::RichText::new("🚀 GenXLink")
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::from_rgb(96, 165, 250)));
                    
                    ui.add_space(8.0);
                    
                    // Version and details
                    ui.label(egui::RichText::new("Version 0.1.0")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(156, 163, 175)));
                    
                    ui.add_space(4.0);
                    
                    ui.label(egui::RichText::new("🇮🇳 Created in India")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(156, 163, 175)));
                    
                    ui.add_space(4.0);
                    
                    ui.label(egui::RichText::new("📧 genxisinnovation@outlook.com")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(156, 163, 175)));
                    
                    ui.add_space(12.0);
                    
                    // Action buttons with vibrant colors
                    ui.horizontal(|ui| {
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("📄 License")
                                    .color(egui::Color32::WHITE)
                                    .size(12.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(6, 182, 212))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(90.0, 32.0))
                        ).clicked() {
                            action = SettingsAction::ViewLicense;
                        }
                        
                        ui.add_space(10.0);
                        
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("📚 Documentation")
                                    .color(egui::Color32::WHITE)
                                    .size(12.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(34, 197, 94))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(140.0, 32.0))
                        ).clicked() {
                            action = SettingsAction::OpenDocumentation;
                        }
                        
                        ui.add_space(10.0);
                        
                        let _ = ui.add(
                            egui::Button::new(
                                egui::RichText::new("🔗 GitHub")
                                    .color(egui::Color32::WHITE)
                                    .size(12.0)
                                    .strong()
                            )
                                .fill(egui::Color32::from_rgb(168, 85, 247))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::vec2(85.0, 32.0))
                        );
                    });
                    
                    ui.add_space(5.0);
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
