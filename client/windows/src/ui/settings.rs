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

        // Clean, professional header
        ui.vertical_centered(|ui| {
            ui.add_space(15.0);
            ui.label(egui::RichText::new("Settings")
                .size(24.0)
                .strong());
            ui.add_space(5.0);
            ui.label(egui::RichText::new("Configure your GenXLink experience")
                .size(14.0)
                .color(egui::Color32::from_rgb(107, 114, 128)));
            ui.add_space(25.0);
        });

        // Use proper grid layout for perfect organization
        egui::Grid::new("settings_grid")
            .num_columns(2)
            .spacing([20.0, 20.0])
            .striped(true)
            .show(ui, |ui| {
                
                // Appearance Section
                ui.vertical(|ui| {
                    ui.heading("🎨 Appearance");
                    ui.add_space(12.0);
                    
                    // Theme
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        ui.add_space(12.0);
                        
                        let theme_text = match self.selected_theme {
                            AppTheme::Light => "☀️ Light",
                            AppTheme::Dark => "🌙 Dark", 
                            AppTheme::System => "💻 System",
                        };
                        
                        egui::ComboBox::from_id_source("theme_combo")
                            .selected_text(theme_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.selected_theme, AppTheme::Light, "☀️ Light");
                                ui.selectable_value(&mut self.selected_theme, AppTheme::Dark, "🌙 Dark");
                                ui.selectable_value(&mut self.selected_theme, AppTheme::System, "💻 System");
                            });
                    });
                    
                    ui.add_space(12.0);
                    
                    // Language
                    ui.horizontal(|ui| {
                        ui.label("Language:");
                        ui.add_space(12.0);
                        
                        let lang_text = match self.selected_language {
                            AppLanguage::English => "🇬🇧 English",
                            AppLanguage::Hindi => "🇮🇳 हिंदी",
                            AppLanguage::Tamil => "🇮🇳 தமிழ்",
                            AppLanguage::Telugu => "🇮🇳 తెలుగు",
                            AppLanguage::Bengali => "🇮🇳 বাংলা",
                        };
                        
                        egui::ComboBox::from_id_source("lang_combo")
                            .selected_text(lang_text)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.selected_language, AppLanguage::English, "🇬🇧 English");
                                ui.selectable_value(&mut self.selected_language, AppLanguage::Hindi, "🇮🇳 हिंदी");
                                ui.selectable_value(&mut self.selected_language, AppLanguage::Tamil, "🇮🇳 தமிழ்");
                                ui.selectable_value(&mut self.selected_language, AppLanguage::Telugu, "🇮🇳 తెలుగు");
                                ui.selectable_value(&mut self.selected_language, AppLanguage::Bengali, "🇮🇳 বাংলा");
                            });
                    });
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("💡 Theme changes apply immediately")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(107, 114, 128)));
                    ui.label(egui::RichText::new("🌍 Language support coming soon")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(107, 114, 128)));
                });

                // Behavior Section
                ui.vertical(|ui| {
                    ui.heading("🔧 Behavior");
                    ui.add_space(12.0);
                    
                    ui.checkbox(&mut self.auto_start, "Start with Windows");
                    ui.add_space(8.0);
                    ui.checkbox(&mut self.minimize_to_tray, "Minimize to tray");
                    ui.add_space(8.0);
                    ui.checkbox(&mut self.enable_notifications, "Desktop notifications");
                });

                ui.end_row();

                // Advanced Section
                ui.vertical(|ui| {
                    ui.heading("🔬 Advanced");
                    ui.add_space(12.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Log Level:");
                        ui.add_space(12.0);
                        
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

                    ui.add_space(12.0);
                    
                    if ui.button("📂 Open Log Folder").clicked() {
                        action = SettingsAction::OpenLogFolder;
                    }
                });

                // About Section
                ui.vertical(|ui| {
                    ui.heading("ℹ️ About");
                    ui.add_space(12.0);
                    
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("🚀 GenXLink")
                            .size(16.0)
                            .strong());
                        
                        ui.add_space(8.0);
                        
                        ui.label("Version 0.1.0");
                        ui.label("🇮🇳 Created in India");
                        ui.label("📧 genxisinnovation@outlook.com");
                        
                        ui.add_space(12.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("📄 License").clicked() {
                                action = SettingsAction::ViewLicense;
                            }
                            
                            ui.add_space(8.0);
                            
                            if ui.button("📚 Documentation").clicked() {
                                action = SettingsAction::OpenDocumentation;
                            }
                            
                            ui.add_space(8.0);
                            
                            let _ = ui.button("🔗 GitHub");
                        });
                    });
                });

                ui.end_row();
            });

        action
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
