//! GenXLink Original UI - Standalone Version
//! 
//! This shows the complete original GenXLink interface without networking dependencies

use anyhow::Result;
use eframe::egui;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AppSettings {
    // General Settings
    pub language: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    
    // UI Settings
    pub theme: String,
    pub font_size: f32,
    pub window_opacity: f32,
    pub show_notifications: bool,
    pub notification_sound: bool,
    
    // Screen Share Settings
    pub resolution: String,
    pub quality: String,
    pub frame_rate: u32,
    pub share_audio: bool,
    pub show_cursor: bool,
    
    // Audio Settings
    pub input_device: String,
    pub output_device: String,
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
    pub auto_gain: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "English".to_string(),
            auto_start: false,
            minimize_to_tray: true,
            theme: "Dark".to_string(),
            font_size: 14.0,
            window_opacity: 1.0,
            show_notifications: true,
            notification_sound: true,
            resolution: "1920x1080".to_string(),
            quality: "High".to_string(),
            frame_rate: 60,
            share_audio: true,
            show_cursor: true,
            input_device: "Default".to_string(),
            output_device: "Default".to_string(),
            noise_suppression: true,
            echo_cancellation: true,
            auto_gain: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub device_id: String,
    pub device_name: String,
    pub status: String,
    pub last_seen: String,
    pub capabilities: Vec<String>,
}

#[derive(Default)]
pub struct GenXLinkApp {
    settings: AppSettings,
    connections: Vec<ConnectionInfo>,
    show_settings: bool,
    show_premium: bool,
    current_view: View,
    selected_device: Option<usize>,
}

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    Dashboard,
    Connections,
    Settings,
    Premium,
    About,
}

impl GenXLinkApp {
    pub fn new() -> Self {
        let app = Self {
            settings: AppSettings::default(),
            connections: vec![
                ConnectionInfo {
                    device_id: "OFFICE-PC-001".to_string(),
                    device_name: "Office Desktop".to_string(),
                    status: "Online".to_string(),
                    last_seen: "Active now".to_string(),
                    capabilities: vec!["Screen Share".to_string(), "Audio".to_string(), "File Transfer".to_string()],
                },
                ConnectionInfo {
                    device_id: "LAPTOP-002".to_string(),
                    device_name: "Personal Laptop".to_string(),
                    status: "Offline".to_string(),
                    last_seen: "2 hours ago".to_string(),
                    capabilities: vec!["Screen Share".to_string(), "Audio".to_string()],
                },
            ],
            show_settings: false,
            show_premium: false,
            current_view: View::Dashboard,
            selected_device: None,
        };
        app
    }
    
    fn apply_theme(&self, ctx: &egui::Context) {
        match self.settings.theme.as_str() {
            "Light" => {
                let mut visuals = egui::Visuals::light();
                visuals.window_rounding = 8.0.into();
                visuals.button_frame = true;
                ctx.set_visuals(visuals);
            }
            "System" => {
                // For demo purposes, use dark theme
                let mut visuals = egui::Visuals::dark();
                visuals.window_rounding = 8.0.into();
                visuals.button_frame = true;
                ctx.set_visuals(visuals);
            }
            _ => {
                // Dark theme (default)
                let mut visuals = egui::Visuals::dark();
                visuals.window_rounding = 8.0.into();
                visuals.button_frame = true;
                ctx.set_visuals(visuals);
            }
        }
    }
    
    fn show_dashboard(&mut self, ui: &egui::Context) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("🌐 GenXLink Remote Desktop");
            ui.separator();
            
            // Connection status
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(40, 167, 69), "🟢 Ready to connect");
                ui.separator();
                ui.label("Device: DESKTOP-DEMO");
                ui.separator();
                ui.label("Network: Excellent");
            });
            
            ui.add_space(20.0);
            
            // Quick connect
            egui::Frame {
                fill: egui::Color32::from_rgb(35, 35, 35),
                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                rounding: 8.0.into(),
                inner_margin: egui::Margin::same(16.0),
                ..Default::default()
            }.show(ui, |ui| {
                ui.heading("🔗 Quick Connect");
                ui.add_space(10.0);
                
                let mut device_id = "Enter device ID or access code".to_string();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut device_id);
                    if ui.button("Connect").clicked() {
                        // Handle connection
                    }
                });
            });
            
            ui.add_space(20.0);
            
            // Recent connections
            ui.heading("📱 Recent Connections");
            ui.add_space(10.0);
            
            for (i, conn) in self.connections.iter().enumerate() {
                egui::Frame {
                    fill: if conn.status == "Online" { 
                        egui::Color32::from_rgb(40, 44, 52) 
                    } else { 
                        egui::Color32::from_rgb(30, 30, 30) 
                    },
                    stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                    rounding: 6.0.into(),
                    inner_margin: egui::Margin::same(12.0),
                    ..Default::default()
                }.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Status indicator
                        let status_color = if conn.status == "Online" {
                            egui::Color32::from_rgb(40, 167, 69)
                        } else {
                            egui::Color32::from_rgb(220, 53, 69)
                        };
                        ui.colored_label(status_color, "●");
                        
                        // Device info
                        ui.vertical(|ui| {
                            ui.heading(&conn.device_name);
                            ui.label(format!("ID: {} | {}", conn.device_id, conn.last_seen));
                            ui.horizontal(|ui| {
                                for cap in &conn.capabilities {
                                    ui.colored_label(egui::Color32::from_rgb(0, 120, 215), cap);
                                    ui.label("•");
                                }
                            });
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if conn.status == "Online" {
                                if ui.button("Connect").clicked() {
                                    self.selected_device = Some(i);
                                }
                            } else {
                                ui.add_enabled(false, egui::Button::new("Offline"));
                            }
                        });
                    });
                });
                ui.add_space(8.0);
            }
        });
    }
    
    fn show_settings_panel(&mut self, ui: &egui::Context) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("⚙️ Settings");
            ui.separator();
            
            egui::SidePanel::left("settings_categories").show_inside(ui, |ui| {
                ui.heading("Categories");
                ui.separator();
                
                if ui.selectable_label(matches!(self.current_view, View::Settings), "🎨 General").clicked() {
                    self.current_view = View::Settings;
                }
                if ui.selectable_label(false, "🖥️ Screen Share").clicked() {
                    // Handle screen share settings
                }
                if ui.selectable_label(false, "🎤 Audio").clicked() {
                    // Handle audio settings
                }
                if ui.selectable_label(false, "📁 File Transfer").clicked() {
                    // Handle file transfer settings
                }
                if ui.selectable_label(false, "🔒 Security").clicked() {
                    // Handle security settings
                }
                if ui.selectable_label(false, "📊 Network").clicked() {
                    // Handle network settings
                }
            });
            
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.heading("General Settings");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Language:");
                    let languages = ["English", "Spanish", "French", "German", "Chinese", "Japanese"];
                    let mut selected_index = languages.iter().position(|&l| l.to_lowercase() == self.settings.language).unwrap_or(0);
                    let mut selected_text = languages[selected_index].to_string();
                    if egui::ComboBox::from_label("")
                        .selected_text(&selected_text)
                        .show_index(ui, &mut selected_index, languages.len(), |i| languages[i].to_string()).changed() {
                        self.settings.language = languages[selected_index].to_lowercase();
                    }
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let themes = ["Dark", "Light", "System"];
                    let mut selected_index = themes.iter().position(|&t| t == self.settings.theme).unwrap_or(0);
                    let mut selected_text = themes[selected_index].to_string();
                    if egui::ComboBox::from_label("")
                        .selected_text(&selected_text)
                        .show_index(ui, &mut selected_index, themes.len(), |i| themes[i].to_string()).changed() {
                        self.settings.theme = themes[selected_index].to_string();
                        // Theme will be applied in the next frame
                    }
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    ui.add(egui::Slider::new(&mut self.settings.font_size, 10.0..=24.0).text("px"));
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Window Opacity:");
                    ui.add(egui::Slider::new(&mut self.settings.window_opacity, 0.3..=1.0).text(""));
                });
                
                ui.add_space(20.0);
                
                ui.heading("Notifications");
                ui.separator();
                
                ui.checkbox(&mut self.settings.show_notifications, "Show notifications");
                ui.checkbox(&mut self.settings.notification_sound, "Play notification sound");
                
                ui.add_space(20.0);
                
                ui.heading("System");
                ui.separator();
                
                ui.checkbox(&mut self.settings.auto_start, "Auto-start with Windows");
                ui.checkbox(&mut self.settings.minimize_to_tray, "Minimize to tray on close");
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Save Settings").clicked() {
                        // In a real app, this would save to a config file
                        println!("Settings saved: {:?}", self.settings);
                    }
                    if ui.button("Reset to Defaults").clicked() {
                        self.settings = AppSettings::default();
                    }
                });
            });
        });
    }
    
    fn show_premium_panel(&mut self, ui: &egui::Context) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("💎 Premium Features");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Choose the plan that's right for you:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Current: Free Plan");
                });
            });
            
            ui.add_space(20.0);
            
            // Pricing cards
            ui.columns(3, |columns| {
                // Free Plan
                columns[0].vertical_centered(|ui| {
                    egui::Frame {
                        fill: egui::Color32::from_rgb(35, 35, 35),
                        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
                        rounding: 8.0.into(),
                        inner_margin: egui::Margin::same(16.0),
                        ..Default::default()
                    }.show(ui, |ui| {
                        ui.heading("Free");
                        ui.label("₹0/month");
                        ui.separator();
                        ui.label("✅ Basic screen sharing");
                        ui.label("✅ 1 concurrent session");
                        ui.label("✅ Standard quality");
                        ui.label("❌ No audio sharing");
                        ui.label("❌ No file transfer");
                        ui.add_space(10.0);
                        ui.add_enabled(false, egui::Button::new("Current Plan"));
                    });
                });
                
                // Solo Plan
                columns[1].vertical_centered(|ui| {
                    egui::Frame {
                        fill: egui::Color32::from_rgb(40, 44, 52),
                        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 120, 215)),
                        rounding: 8.0.into(),
                        inner_margin: egui::Margin::same(16.0),
                        ..Default::default()
                    }.show(ui, |ui| {
                        ui.colored_label(egui::Color32::from_rgb(0, 120, 215), "🏆 Best Value");
                        ui.heading("Solo");
                        ui.label("₹199/month");
                        ui.label("≈ $2.39 USD");
                        ui.separator();
                        ui.label("✅ HD screen sharing");
                        ui.label("✅ 3 concurrent sessions");
                        ui.label("✅ Audio sharing");
                        ui.label("✅ File transfer");
                        ui.label("✅ Priority support");
                        ui.add_space(10.0);
                        if ui.button("Upgrade Now").clicked() {
                            // Handle upgrade
                        }
                    });
                });
                
                // Team Plan
                columns[2].vertical_centered(|ui| {
                    egui::Frame {
                        fill: egui::Color32::from_rgb(45, 35, 35),
                        stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(220, 53, 69)),
                        rounding: 8.0.into(),
                        inner_margin: egui::Margin::same(16.0),
                        ..Default::default()
                    }.show(ui, |ui| {
                        ui.colored_label(egui::Color32::from_rgb(220, 53, 69), "🔥 Most Popular");
                        ui.heading("Team");
                        ui.label("₹399/month");
                        ui.label("≈ $4.79 USD");
                        ui.separator();
                        ui.label("✅ 4K screen sharing");
                        ui.label("✅ Unlimited sessions");
                        ui.label("✅ Premium audio");
                        ui.label("✅ Advanced file transfer");
                        ui.label("✅ Team management");
                        ui.label("✅ Enterprise support");
                        ui.add_space(10.0);
                        if ui.button("Upgrade Now").clicked() {
                            // Handle upgrade
                        }
                    });
                });
            });
        });
    }
}

impl eframe::App for GenXLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply current theme
        self.apply_theme(ctx);

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌐 GenXLink Remote Desktop");
                ui.separator();
                
                // Navigation buttons
                if ui.selectable_label(matches!(self.current_view, View::Dashboard), "🏠 Dashboard").clicked() {
                    self.current_view = View::Dashboard;
                }
                if ui.selectable_label(matches!(self.current_view, View::Connections), "🔗 Connections").clicked() {
                    self.current_view = View::Connections;
                }
                if ui.selectable_label(matches!(self.current_view, View::Settings), "⚙️ Settings").clicked() {
                    self.current_view = View::Settings;
                }
                if ui.selectable_label(matches!(self.current_view, View::Premium), "💎 Premium").clicked() {
                    self.current_view = View::Premium;
                }
                if ui.selectable_label(matches!(self.current_view, View::About), "ℹ️ About").clicked() {
                    self.current_view = View::About;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("v1.0.0");
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(40, 167, 69), "🟢 Ready");
                });
            });
        });

        // Main content
        match self.current_view {
            View::Dashboard => self.show_dashboard(ctx),
            View::Settings => self.show_settings_panel(ctx),
            View::Premium => self.show_premium_panel(ctx),
            View::Connections => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("🔗 Connections");
                    ui.label("Connection management view - work in progress");
                });
            }
            View::About => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("ℹ️ About GenXLink");
                    ui.label("GenXLink Remote Desktop v1.0.0");
                    ui.label("© 2024 GenXis Innovations");
                    ui.label("Licensed under Apache License 2.0");
                });
            }
        }
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status: Ready");
                ui.separator();
                ui.label("Network: Excellent");
                ui.separator();
                ui.label("CPU: 8% | Memory: 156MB");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("GenXlink Remote Desktop v1.0.0");
                });
            });
        });
    }
}

fn main() {
    env_logger::init();
    
    let app = GenXLinkApp::new();
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GenXLink Remote Desktop",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    ).expect("Failed to start GenXLink");
}
