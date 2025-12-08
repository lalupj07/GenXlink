//! GenXLink UI Demo - Shows Settings and Premium Features
//! 
//! This demo showcases:
//! - Language selection (English, Spanish, French, German, Chinese, Japanese)
//! - Theme options (Dark, Light, System)
//! - Font size adjustment
//! - Window opacity control
//! - Premium pricing plans

use eframe::egui;
use anyhow::Result;

#[derive(Default)]
pub struct GenXLinkUIDemo {
    show_settings: bool,
    show_premium: bool,
    selected_language: String,
    selected_theme: String,
    font_size: f32,
    window_opacity: f32,
    show_notifications: bool,
    notification_sound: bool,
    auto_start: bool,
    minimize_to_tray: bool,
}

impl eframe::App for GenXLinkUIDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configure dark theme
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = 8.0.into();
        visuals.button_frame = true;
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌐 GenXLink Remote Desktop");
                ui.separator();
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Premium button
                    if ui.add(
                        egui::Button::new("💎 Premium")
                            .fill(if self.show_premium { egui::Color32::from_rgb(30, 136, 229) } else { egui::Color32::from_rgb(45, 45, 48) })
                    ).clicked() {
                        self.show_premium = !self.show_premium;
                        self.show_settings = false;
                    }
                    
                    // Settings button
                    if ui.add(
                        egui::Button::new("⚙️ Settings")
                            .fill(if self.show_settings { egui::Color32::from_rgb(30, 136, 229) } else { egui::Color32::from_rgb(45, 45, 48) })
                    ).clicked() {
                        self.show_settings = !self.show_settings;
                        self.show_premium = false;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_settings {
                self.show_settings_ui(ui);
            } else if self.show_premium {
                self.show_premium_ui(ui);
            } else {
                self.show_welcome_ui(ui);
            }
        });
    }
}

impl GenXLinkUIDemo {
    fn show_welcome_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Welcome to GenXLink");
            ui.add_space(20.0);
            ui.label("Fast • Secure • Ultra-Low Latency Remote Desktop Access");
            ui.add_space(30.0);
            
            ui.label("✅ Features Implemented:");
            ui.label("   • Language Selection (6 languages)");
            ui.label("   • Theme Options (Dark/Light/System)");
            ui.label("   • Font Size Adjustment");
            ui.label("   • Window Opacity Control");
            ui.label("   • Premium Pricing Plans");
            ui.add_space(20.0);
            
            if ui.button("⚙️ Open Settings").clicked() {
                self.show_settings = true;
            }
            
            if ui.button("💎 View Premium Plans").clicked() {
                self.show_premium = true;
            }
        });
    }

    fn show_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Settings");
            ui.separator();
            ui.label("🎨 Interface Settings");
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Language Setting
            ui.horizontal(|ui| {
                ui.label("Language:");
                let languages = ["English", "Spanish", "French", "German", "Chinese", "Japanese"];
                let mut selected_index = languages.iter().position(|&l| l == self.selected_language).unwrap_or(0);
                
                if egui::ComboBox::from_label("")
                    .selected_text(&self.selected_language)
                    .show_index(ui, &mut selected_index, languages.len(), |i| languages[i].to_string())
                    .changed()
                {
                    self.selected_language = languages[selected_index].to_string();
                }
            });
            
            ui.add_space(10.0);
            
            // Theme Setting
            ui.horizontal(|ui| {
                ui.label("Theme:");
                let themes = ["Dark", "Light", "System"];
                let mut selected_index = themes.iter().position(|&t| t == self.selected_theme).unwrap_or(0);
                
                if egui::ComboBox::from_label("")
                    .selected_text(&self.selected_theme)
                    .show_index(ui, &mut selected_index, themes.len(), |i| themes[i].to_string())
                    .changed()
                {
                    self.selected_theme = themes[selected_index].to_string();
                }
            });
            
            ui.add_space(10.0);
            
            // Font Size
            ui.horizontal(|ui| {
                ui.label("Font Size:");
                if ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0)).changed() {
                    // Font size changed
                }
                ui.label("px");
            });
            
            ui.add_space(10.0);
            
            // Window Opacity
            ui.horizontal(|ui| {
                ui.label("Window Opacity:");
                if ui.add(egui::Slider::new(&mut self.window_opacity, 0.3..=1.0)).changed() {
                    // Opacity changed
                }
            });
            
            ui.add_space(10.0);
            
            // Notifications
            ui.horizontal(|ui| {
                ui.label("Show Notifications:");
                if ui.checkbox(&mut self.show_notifications, "").changed() {
                    // Setting changed
                }
            });
            
            ui.add_space(10.0);
            
            // Notification Sound
            ui.horizontal(|ui| {
                ui.label("Notification Sound:");
                if ui.checkbox(&mut self.notification_sound, "").changed() {
                    // Setting changed
                }
            });
            
            ui.add_space(20.0);
            ui.separator();
            ui.heading("🏠 General Settings");
            ui.add_space(10.0);
            
            // Auto-start
            ui.horizontal(|ui| {
                ui.label("Auto-start with Windows:");
                if ui.checkbox(&mut self.auto_start, "").changed() {
                    // Setting changed
                }
            });
            
            ui.add_space(10.0);
            
            // Minimize to tray
            ui.horizontal(|ui| {
                ui.label("Minimize to tray:");
                if ui.checkbox(&mut self.minimize_to_tray, "").changed() {
                    // Setting changed
                }
            });
        });
    }

    fn show_premium_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🌐 GenXLink Pricing");
            ui.separator();
            ui.label("Fast • Secure • Ultra-Low Latency Remote Desktop Access");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(15.0);

        // Current plan badge
        ui.horizontal(|ui| {
            ui.label("Current Plan:");
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "🟢 Free Tier");
        });

        ui.add_space(15.0);

        // Pricing cards
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                // Free Tier
                self.show_pricing_card(
                    ui,
                    "🟢 Free Tier",
                    "₹0",
                    "month",
                    None,
                    "Perfect for personal & occasional use",
                    &[
                        "✔ All core remote-access features",
                        "✔ GPU Acceleration",
                        "✔ Ultra-Low Latency",
                        "✔ Adaptive Bitrate",
                        "✔ Smooth streaming & control",
                    ],
                    &[
                        "1 registered device",
                        "Login from 1 device",
                        "No unattended access",
                        "No recording",
                    ],
                    "Best for: Students, casual users, home use",
                    egui::Color32::from_rgb(100, 200, 100),
                    true,
                );

                ui.add_space(10.0);

                // Solo Plan
                self.show_pricing_card(
                    ui,
                    "🔵 Solo Plan",
                    "₹199",
                    "month",
                    Some("🔥 Best Value"),
                    "Ideal for creators, professionals & freelancers",
                    &[
                        "✔ Everything in Free +",
                        "✔ Audio streaming",
                        "✔ AI-powered enhancements",
                        "✔ Unattended access",
                        "✔ Session recording",
                        "✔ Multi-user sessions",
                    ],
                    &[
                        "1 registered device",
                        "Login from up to 5 devices",
                        "2 concurrent sessions",
                    ],
                    "Best for: Creators, freelancers • $2.39",
                    egui::Color32::from_rgb(100, 150, 255),
                    false,
                );

                ui.add_space(10.0);

                // Team Plan
                self.show_pricing_card(
                    ui,
                    "🟣 Team Plan",
                    "₹399",
                    "month",
                    Some("⭐ Most Popular"),
                    "Built for support teams, studios & IT admins",
                    &[
                        "✔ Everything in Solo +",
                        "✔ Team Dashboard",
                        "✔ Role-based access control",
                        "✔ Technician switching",
                        "✔ Shared device groups",
                        "✔ Advanced reports & logs",
                        "✔ Priority routing",
                    ],
                    &[
                        "5 registered devices",
                        "Login from up to 20 devices",
                        "5 concurrent sessions",
                    ],
                    "Best for: Teams, IT support • $4.79",
                    egui::Color32::from_rgb(156, 39, 176),
                    false,
                );
            });
        });
    }

    fn show_pricing_card(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        price: &str,
        period: &str,
        badge: Option<&str>,
        description: &str,
        features: &[&str],
        limitations: &[&str],
        best_for: &str,
        color: egui::Color32,
        is_current: bool,
    ) {
        ui.vertical(|ui| {
            // Card background
            let frame = egui::Frame {
                fill: if is_current {
                    egui::Color32::from_rgb(30, 30, 30)
                } else {
                    egui::Color32::from_rgb(40, 40, 40)
                },
                stroke: egui::Stroke::new(1.0, color),
                rounding: 8.0.into(),
                ..Default::default()
            };
            
            frame.show(ui, |ui| {
                ui.add_space(15.0);
                ui.vertical_centered(|ui| {
                    // Badge
                    if let Some(badge_text) = badge {
                        ui.colored_label(color, badge_text);
                    }
                    
                    // Title
                    ui.heading(title);
                    
                    // Price
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(price)
                                .size(32.0)
                                .strong()
                                .color(color)
                        ));
                        ui.label(format!("/{}", period));
                    });
                    
                    ui.add_space(10.0);
                    
                    // Description
                    ui.label(description);
                    
                    ui.add_space(15.0);
                    
                    // Features
                    ui.horizontal(|ui| {
                        ui.heading("Features:");
                    });
                    
                    for feature in features {
                        ui.horizontal(|ui| {
                            ui.label(*feature);
                        });
                    }
                    
                    ui.add_space(15.0);
                    
                    // Limitations
                    for limitation in limitations {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::GRAY, *limitation);
                        });
                    }
                    
                    ui.add_space(15.0);
                    
                    // Best for
                    ui.label(best_for);
                    
                    ui.add_space(20.0);
                    
                    // CTA button
                    if is_current {
                        ui.add_enabled(false, egui::Button::new("Current Plan"));
                    } else {
                        if ui.add(egui::Button::new("Upgrade Now").fill(color)).clicked() {
                            // Handle upgrade
                        }
                    }
                });
                
                ui.add_space(15.0);
            });
        });
    }
}

fn main() -> Result<()> {
    env_logger::init(); // Initialize logger
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_decorations(true)
            .with_resizable(true)
            .with_title("GenXLink Remote Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "GenXLink Remote Desktop",
        native_options,
        Box::new(|cc| {
            // Configure egui
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            Ok(Box::new(GenXLinkUIDemo {
                selected_language: "English".to_string(),
                selected_theme: "Dark".to_string(),
                font_size: 14.0,
                window_opacity: 1.0,
                show_notifications: true,
                notification_sound: true,
                auto_start: false,
                minimize_to_tray: true,
                ..Default::default()
            }))
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}
