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
    
    // Network Settings
    auto_connect: bool,
    port_forwarding: bool,
    use_relay_server: bool,
    max_bandwidth: u32,
    
    // Security Settings
    require_password: bool,
    enable_encryption: bool,
    session_timeout: u32,
    allow_file_transfers: bool,
    
    // Performance Settings
    hardware_acceleration: bool,
    adaptive_quality: bool,
    max_fps: u32,
    compression_level: u32,
    
    // Display Settings
    show_fps: bool,
    full_screen_on_connect: bool,
    cursor_scaling: f32,
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
            
            // Network Settings
            auto_connect: false,
            port_forwarding: true,
            use_relay_server: true,
            max_bandwidth: 10000, // 10 Mbps
            
            // Security Settings
            require_password: true,
            enable_encryption: true,
            session_timeout: 30, // 30 minutes
            allow_file_transfers: true,
            
            // Performance Settings
            hardware_acceleration: true,
            adaptive_quality: true,
            max_fps: 60,
            compression_level: 5, // Medium compression
            
            // Display Settings
            show_fps: false,
            full_screen_on_connect: false,
            cursor_scaling: 1.0,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> SettingsAction {
        let mut action = SettingsAction::None;

        // Header
        ui.vertical_centered(|ui| {
            ui.heading("⚙️ Settings");
            ui.add_space(5.0);
            ui.label("Configure your GenXLink experience");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Appearance Section
        ui.heading("🎨 Appearance");
        ui.add_space(10.0);
        
        // Theme selection
        ui.horizontal(|ui| {
            ui.label("🎨 Theme:");
            ui.add_space(10.0);

            let mut theme_changed = false;
            let old_theme = self.selected_theme;
            
            egui::ComboBox::from_label("")
                .selected_text(match self.selected_theme {
                    AppTheme::Light => "☀️ Light",
                    AppTheme::Dark => "🌙 Dark",
                    AppTheme::System => "🖥️ System",
                })
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.selected_theme, AppTheme::Light, "☀️ Light").clicked() {
                        theme_changed = self.selected_theme != old_theme;
                    }
                    if ui.selectable_value(&mut self.selected_theme, AppTheme::Dark, "🌙 Dark").clicked() {
                        theme_changed = self.selected_theme != old_theme;
                    }
                    if ui.selectable_value(&mut self.selected_theme, AppTheme::System, "🖥️ System").clicked() {
                        theme_changed = self.selected_theme != old_theme;
                    }
                });

            if theme_changed {
                self.apply_theme_change(ui.ctx());
            }
        });
        
        // Language selection
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("🌐 Language:");
            ui.add_space(10.0);

            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.selected_language))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_language, AppLanguage::English, "🇺🇸 English");
                    ui.selectable_value(&mut self.selected_language, AppLanguage::Hindi, "🇮🇳 हिंदी");
                    ui.selectable_value(&mut self.selected_language, AppLanguage::Tamil, "🇮🇳 தமிழ்");
                    ui.selectable_value(&mut self.selected_language, AppLanguage::Telugu, "🇮🇳 తెలుగు");
                    ui.selectable_value(&mut self.selected_language, AppLanguage::Bengali, "🇮🇳 বাংলা");
                });
        });
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                "ℹ️ Theme changes apply immediately • Language support coming soon");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Behavior Section
        ui.heading("⚙️ Behavior");
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.auto_start, "🚀 Start GenXLink with Windows");
        ui.checkbox(&mut self.minimize_to_tray, "🗔 Minimize to system tray");
        ui.checkbox(&mut self.enable_notifications, "🔔 Enable desktop notifications");

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Network Section
        ui.heading("🌐 Network");
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.auto_connect, "🔗 Auto-connect to last device on startup");
        ui.checkbox(&mut self.port_forwarding, "🚪 Enable port forwarding (UPnP)");
        ui.checkbox(&mut self.use_relay_server, "☁️ Use relay server when direct connection fails");
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("📊 Max Bandwidth (Mbps):");
            ui.add_space(10.0);
            ui.add(egui::Slider::new(&mut self.max_bandwidth, 1..=100).text("Mbps"));
        });
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                "ℹ️ Lower values save bandwidth • Higher values improve quality");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Security Section
        ui.heading("🔒 Security");
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.require_password, "🔐 Require password for incoming connections");
        ui.checkbox(&mut self.enable_encryption, "🛡️ End-to-end encryption");
        ui.checkbox(&mut self.allow_file_transfers, "📁 Allow file transfers");
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("⏰ Session Timeout (minutes):");
            ui.add_space(10.0);
            ui.add(egui::Slider::new(&mut self.session_timeout, 5..=120).text("min"));
        });
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                "ℹ️ Auto-disconnect after inactivity • 0 = never timeout");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Performance Section
        ui.heading("⚡ Performance");
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.hardware_acceleration, "🎮 Hardware acceleration (GPU)");
        ui.checkbox(&mut self.adaptive_quality, "📈 Adaptive quality based on network");
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("🎬 Max FPS:");
            ui.add_space(10.0);
            ui.add(egui::Slider::new(&mut self.max_fps, 15..=144).text("FPS"));
        });
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("🗜️ Compression Level:");
            ui.add_space(10.0);
            ui.add(egui::Slider::new(&mut self.compression_level, 1..=9).text("Level"));
        });
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                "ℹ️ Lower compression = better quality • Higher = lower bandwidth");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Display Section
        ui.heading("🖥️ Display");
        ui.add_space(10.0);
        
        ui.checkbox(&mut self.show_fps, "📊 Show FPS counter");
        ui.checkbox(&mut self.full_screen_on_connect, "🖼️ Full screen on connect");
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("🖱️ Cursor Scaling:");
            ui.add_space(10.0);
            ui.add(egui::Slider::new(&mut self.cursor_scaling, 0.5..=3.0).text("x"));
        });
        
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), 
                "ℹ️ Make cursor larger/smaller for better visibility");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Advanced Section
        ui.heading("⚙️ Advanced");
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("📝 Log Level:");
            ui.add_space(10.0);

            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.log_level))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.log_level, LogLevel::Error, "❌ Error");
                    ui.selectable_value(&mut self.log_level, LogLevel::Warn, "⚠️ Warning");
                    ui.selectable_value(&mut self.log_level, LogLevel::Info, "ℹ️ Info");
                    ui.selectable_value(&mut self.log_level, LogLevel::Debug, "🐛 Debug");
                });
        });

        ui.add_space(10.0);
        if ui.button("📂 Open Log Folder").clicked() {
            action = SettingsAction::OpenLogFolder;
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // About Section
        ui.heading("ℹ️ About");
        ui.add_space(10.0);
        
        ui.vertical(|ui| {
            ui.label("🚀 GenXLink Remote Desktop");
            ui.label("📌 Version: 0.1.0");
            ui.label("🇮🇳 Created in India • Crafted by Indians");
            ui.label("📧 Contact: genxisinnovation@outlook.com");
            ui.label("🌐 GitHub: https://github.com/lalupj07/GenXlink");
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("📄 License").clicked() {
                action = SettingsAction::ViewLicense;
            }
            ui.add_space(10.0);
            if ui.button("📚 Documentation").clicked() {
                action = SettingsAction::OpenDocumentation;
            }
        });

        action
    }

    pub fn apply_theme_change(&self, ctx: &egui::Context) {
        match self.selected_theme {
            AppTheme::Light => {
                let visuals = egui::Visuals::light();
                // You can customize the light theme here if needed
                ctx.set_visuals(visuals);
            }
            AppTheme::Dark => {
                let visuals = egui::Visuals::dark();
                // You can customize the dark theme here if needed
                ctx.set_visuals(visuals);
            }
            AppTheme::System => {
                // For system theme, you might want to detect the system preference
                // For now, we'll use dark as the default system theme
                // You can implement system theme detection here
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
        // Request a repaint to see the theme change immediately
        ctx.request_repaint();
    }

    pub fn apply_language_change(&self, _ctx: &egui::Context) {
        // Language change logic will be implemented here
        // Currently, it's a placeholder
    }
}

/// Actions that can be triggered from the settings panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    None,
    OpenLogFolder,
    ViewLicense,
    OpenDocumentation,
    ResetToDefaults,
    ExportSettings,
    ImportSettings,
}