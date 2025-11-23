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

        // Beautiful header with gradient effect simulation
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            
            // Main title with styling
            ui.label(egui::RichText::new("⚙️ Settings")
                .size(28.0)
                .strong()
                .color(egui::Color32::from_rgb(59, 130, 246)));
            
            ui.add_space(5.0);
            ui.label(egui::RichText::new("Personalize your GenXLink experience")
                .size(14.0)
                .color(egui::Color32::from_rgb(107, 114, 128)));
            
            ui.add_space(20.0);
        });

        // Main container with beautiful cards
        ui.vertical(|ui| {
            // Appearance Card - Full width
            ui.group(|ui| {
                // Card header with accent
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎨 Appearance")
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::from_rgb(139, 92, 246)));
                    
                    ui.add_space(10.0);
                    
                    // Add a subtle accent line
                    ui.add_space(ui.available_width() - 100.0);
                    
                    ui.separator();
                });
                
                ui.add_space(10.0);
                
                // Theme selection with beautiful styling
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Theme")
                        .size(14.0)
                        .strong());
                    ui.add_space(15.0);
                    
                    let theme_text = match self.selected_theme {
                        AppTheme::Light => "☀️ Light",
                        AppTheme::Dark => "🌙 Dark", 
                        AppTheme::System => "💻 System",
                    };
                    
                    egui::ComboBox::from_label("")
                        .selected_text(theme_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_theme, AppTheme::Light, "☀️ Light");
                            ui.selectable_value(&mut self.selected_theme, AppTheme::Dark, "🌙 Dark");
                            ui.selectable_value(&mut self.selected_theme, AppTheme::System, "💻 System");
                        });
                });
                
                ui.add_space(12.0);
                
                // Language selection
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Language")
                        .size(14.0)
                        .strong());
                    ui.add_space(15.0);
                    
                    let lang_text = match self.selected_language {
                        AppLanguage::English => "🇬🇧 English",
                        AppLanguage::Hindi => "🇮🇳 हिंदी",
                        AppLanguage::Tamil => "🇮🇳 தமிழ்",
                        AppLanguage::Telugu => "🇮🇳 తెలుగు",
                        AppLanguage::Bengali => "🇮🇳 বাংলা",
                    };
                    
                    egui::ComboBox::from_label("")
                        .selected_text(lang_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_language, AppLanguage::English, "🇬🇧 English");
                            ui.selectable_value(&mut self.selected_language, AppLanguage::Hindi, "🇮🇳 हिंदी");
                            ui.selectable_value(&mut self.selected_language, AppLanguage::Tamil, "🇮🇳 தமிழ்");
                            ui.selectable_value(&mut self.selected_language, AppLanguage::Telugu, "🇮🇳 తెలుగు");
                            ui.selectable_value(&mut self.selected_language, AppLanguage::Bengali, "🇮🇳 বাংলা");
                        });
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("💡")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(59, 130, 246)));
                    ui.label(egui::RichText::new("Theme changes apply immediately")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(107, 114, 128)));
                });
                
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🌍")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(34, 197, 94)));
                    ui.label(egui::RichText::new("Language support coming soon")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(107, 114, 128)));
                });
                
                ui.add_space(10.0);
            });
            
            ui.add_space(15.0);
            
            // Two column layout for Behavior and Advanced
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Behavior Card
                    ui.group(|ui| {
                        // Card header with accent
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🔧 Behavior")
                                .size(16.0)
                                .strong()
                                .color(egui::Color32::from_rgb(34, 197, 94)));
                            
                            ui.add_space(10.0);
                            
                            // Add a subtle accent line
                            ui.add_space(ui.available_width() - 100.0);
                            
                            ui.separator();
                        });
                        
                        ui.add_space(10.0);
                        
                        ui.checkbox(&mut self.auto_start, "🚀 Start with Windows");
                        ui.add_space(8.0);
                        ui.checkbox(&mut self.minimize_to_tray, "📌 Minimize to tray");
                        ui.add_space(8.0);
                        ui.checkbox(&mut self.enable_notifications, "🔔 Desktop notifications");
                        
                        ui.add_space(10.0);
                    });
                });
                
                ui.add_space(15.0);
                
                ui.vertical(|ui| {
                    // Advanced Card
                    ui.group(|ui| {
                        // Card header with accent
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🔬 Advanced")
                                .size(16.0)
                                .strong()
                                .color(egui::Color32::from_rgb(251, 146, 60)));
                            
                            ui.add_space(10.0);
                            
                            // Add a subtle accent line
                            ui.add_space(ui.available_width() - 100.0);
                            
                            ui.separator();
                        });
                        
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Log Level")
                                .size(14.0)
                                .strong());
                            ui.add_space(15.0);
                            
                            let log_text = match self.log_level {
                                LogLevel::Error => "❌ Error",
                                LogLevel::Warn => "⚠️ Warning",
                                LogLevel::Info => "ℹ️ Info",
                                LogLevel::Debug => "🐛 Debug",
                            };
                            
                            egui::ComboBox::from_label("")
                                .selected_text(log_text)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.log_level, LogLevel::Error, "❌ Error");
                                    ui.selectable_value(&mut self.log_level, LogLevel::Warn, "⚠️ Warning");
                                    ui.selectable_value(&mut self.log_level, LogLevel::Info, "ℹ️ Info");
                                    ui.selectable_value(&mut self.log_level, LogLevel::Debug, "🐛 Debug");
                                });
                        });

                        ui.add_space(12.0);
                        
                        if ui.button(egui::RichText::new("📂 Open Log Folder")
                            .size(14.0))
                            .clicked() {
                            action = SettingsAction::OpenLogFolder;
                        }
                        
                        ui.add_space(10.0);
                    });
                });
            });
            
            ui.add_space(15.0);
            
            // About Card - Full width
            ui.group(|ui| {
                // Card header with accent
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("ℹ️ About GenXLink")
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::from_rgb(59, 130, 246)));
                    
                    ui.add_space(10.0);
                    
                    // Add a subtle accent line
                    ui.add_space(ui.available_width() - 100.0);
                    
                    ui.separator();
                });
                
                ui.add_space(10.0);
                
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("🚀 GenXLink Remote Desktop")
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::from_rgb(59, 130, 246)));
                    
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("Version 0.1.0")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(107, 114, 128)));
                    
                    ui.add_space(5.0);
                    
                    ui.label(egui::RichText::new("🇮🇳 Created in India • Crafted by Indians")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(34, 197, 94)));
                    
                    ui.add_space(5.0);
                    
                    ui.label(egui::RichText::new("📧 genxisinnovation@outlook.com")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(107, 114, 128)));
                    
                    ui.add_space(12.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("📄 License")
                            .size(14.0))
                            .clicked() {
                            action = SettingsAction::ViewLicense;
                        }
                        
                        ui.add_space(10.0);
                        
                        if ui.button(egui::RichText::new("📚 Documentation")
                            .size(14.0))
                            .clicked() {
                            action = SettingsAction::OpenDocumentation;
                        }
                        
                        ui.add_space(10.0);
                        
                        ui.button(egui::RichText::new("🔗 GitHub")
                            .size(14.0));
                    });
                });
                
                ui.add_space(10.0);
            });
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
