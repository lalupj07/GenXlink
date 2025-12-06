//! GenXLink Simple Working UI

use eframe::egui;

#[derive(Default)]
pub struct GenXLinkApp {
    language: String,
    theme: String,
    font_size: f32,
    show_notifications: bool,
    current_view: View,
}

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    Dashboard,
    Settings,
    Premium,
}

impl GenXLinkApp {
    pub fn new() -> Self {
        Self {
            language: "English".to_string(),
            theme: "Dark".to_string(),
            font_size: 14.0,
            show_notifications: true,
            current_view: View::Dashboard,
        }
    }
}

impl eframe::App for GenXLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        match self.theme.as_str() {
            "Light" => {
                ctx.set_visuals(egui::Visuals::light());
            }
            _ => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌐 GenXLink Remote Desktop");
                ui.separator();
                
                // Navigation buttons
                if ui.selectable_label(matches!(self.current_view, View::Dashboard), "🏠 Dashboard").clicked() {
                    self.current_view = View::Dashboard;
                }
                if ui.selectable_label(matches!(self.current_view, View::Settings), "⚙️ Settings").clicked() {
                    self.current_view = View::Settings;
                }
                if ui.selectable_label(matches!(self.current_view, View::Premium), "💎 Premium").clicked() {
                    self.current_view = View::Premium;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("v1.0.0");
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(40, 167, 69), "🟢 Ready");
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::Dashboard => {
                    ui.heading("🏠 Dashboard");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(40, 167, 69), "🟢 Ready to connect");
                        ui.separator();
                        ui.label("Device: DESKTOP-DEMO");
                        ui.separator();
                        ui.label("Network: Excellent");
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.heading("🔗 Quick Connect");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        let mut device_id = "Enter device ID".to_string();
                        ui.text_edit_singleline(&mut device_id);
                        if ui.button("Connect").clicked() {
                            println!("Connect clicked!");
                        }
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.heading("📱 Recent Connections");
                    ui.add_space(10.0);
                    
                    // Office Desktop
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(40, 167, 69), "●");
                            ui.vertical(|ui| {
                                ui.heading("Office Desktop");
                                ui.label("ID: OFFICE-PC-001 | Active now");
                                ui.label("Screen Share • Audio • File Transfer");
                            });
                            if ui.button("Connect").clicked() {
                                println!("Connecting to Office Desktop");
                            }
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Personal Laptop
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(220, 53, 69), "●");
                            ui.vertical(|ui| {
                                ui.heading("Personal Laptop");
                                ui.label("ID: LAPTOP-002 | 2 hours ago");
                                ui.label("Screen Share • Audio");
                            });
                            ui.add_enabled(false, egui::Button::new("Offline"));
                        });
                    });
                }
                
                View::Settings => {
                    ui.heading("⚙️ Settings");
                    ui.separator();
                    
                    // Language
                    ui.horizontal(|ui| {
                        ui.label("Language:");
                        let languages = ["English", "Spanish", "French", "German", "Chinese", "Japanese"];
                        let mut selected_index = languages.iter().position(|&l| l == self.language).unwrap_or(0);
                        let mut selected_text = languages[selected_index].to_string();
                        if egui::ComboBox::from_label("")
                            .selected_text(&selected_text)
                            .show_index(ui, &mut selected_index, languages.len(), |i| languages[i].to_string()).changed() {
                            self.language = languages[selected_index].to_string();
                            println!("Language changed to: {}", self.language);
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    // Theme
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        let themes = ["Dark", "Light", "System"];
                        let mut selected_index = themes.iter().position(|&t| t == self.theme).unwrap_or(0);
                        let mut selected_text = themes[selected_index].to_string();
                        if egui::ComboBox::from_label("")
                            .selected_text(&selected_text)
                            .show_index(ui, &mut selected_index, themes.len(), |i| themes[i].to_string()).changed() {
                            self.theme = themes[selected_index].to_string();
                            println!("Theme changed to: {}", self.theme);
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    // Font Size
                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0).text("px"));
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.heading("Notifications");
                    ui.separator();
                    
                    ui.checkbox(&mut self.show_notifications, "Show notifications");
                    
                    ui.add_space(20.0);
                    
                    ui.heading("System");
                    ui.separator();
                    
                    ui.checkbox(&mut false, "Auto-start with Windows");
                    ui.checkbox(&mut true, "Minimize to tray on close");
                    
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Save Settings").clicked() {
                            println!("Settings saved: Language={}, Theme={}, Font Size={}", 
                                   self.language, self.theme, self.font_size);
                        }
                        if ui.button("Reset to Defaults").clicked() {
                            self.language = "English".to_string();
                            self.theme = "Dark".to_string();
                            self.font_size = 14.0;
                            self.show_notifications = true;
                            println!("Settings reset to defaults");
                        }
                    });
                }
                
                View::Premium => {
                    ui.heading("💎 Premium Features");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Choose the plan that's right for you:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label("Current: Free Plan");
                        });
                    });
                    
                    ui.add_space(20.0);
                    
                    ui.columns(3, |columns| {
                        // Free Plan
                        columns[0].vertical_centered(|ui| {
                            ui.group(|ui| {
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
                            ui.group(|ui| {
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
                                    println!("Upgrade to Solo clicked");
                                }
                            });
                        });
                        
                        // Team Plan
                        columns[2].vertical_centered(|ui| {
                            ui.group(|ui| {
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
                                    println!("Upgrade to Team clicked");
                                }
                            });
                        });
                    });
                }
            }
        });
        
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

fn main() -> eframe::Result<()> {
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
    )
}
