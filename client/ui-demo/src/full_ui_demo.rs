//! GenXLink Windows Client - Full UI Demo
//! 
//! This shows the complete GenXLink interface with all settings panels
//! but without the P2P networking components that have compilation issues

use anyhow::Result;
use eframe::egui;
use std::sync::{Arc, RwLock};

mod ui;

use ui::{
    main_window::{MainWindow, AppTheme},
    settings_panel::{SettingsPanel, AppSettings, GeneralSettings, UISettings, ScreenShareSettings, AudioSettings},
    premium_features::PremiumFeaturesPanel,
};

#[derive(Default)]
pub struct GenXLinkApp {
    main_window: MainWindow,
    settings_panel: SettingsPanel,
    premium_features: PremiumFeaturesPanel,
    show_settings: bool,
    show_premium: bool,
    theme: AppTheme,
}

impl eframe::App for GenXLinkApp {
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
                
                // Connection status
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "🟢 Connected");
                    ui.label("|");
                    ui.label("Device: DEMO-PC-001");
                    ui.label("|");
                    ui.label("Session: None");
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Premium button
                    if ui.add(
                        egui::Button::new("💎 Premium")
                            .fill(if self.show_premium { self.theme.primary_color } else { self.theme.surface_color })
                    ).clicked() {
                        self.show_premium = !self.show_premium;
                        self.show_settings = false;
                    }
                    
                    // Settings button
                    if ui.add(
                        egui::Button::new("⚙️ Settings")
                            .fill(if self.show_settings { self.theme.primary_color } else { self.theme.surface_color })
                    ).clicked() {
                        self.show_settings = !self.show_settings;
                        self.show_premium = false;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_settings {
                ui.horizontal(|ui| {
                    ui.heading("⚙️ Settings");
                    ui.separator();
                    ui.label("Configure your GenXLink experience");
                });
                ui.separator();
                self.settings_panel.show(ui, &self.theme);
            } else if self.show_premium {
                let _action = self.premium_features.show(ui);
            } else {
                // Main connection panel
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("🌐 GenXLink Remote Desktop");
                    ui.add_space(20.0);
                    ui.label("Fast • Secure • Ultra-Low Latency Remote Desktop Access");
                    ui.add_space(30.0);
                    
                    // Connection card
                    egui::Frame {
                        fill: self.theme.surface_color,
                        stroke: egui::Stroke::new(1.0, self.theme.border_color),
                        rounding: 12.0.into(),
                        inner_margin: egui::Margin::same(20.0),
                        ..Default::default()
                    }.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("🔗 Quick Connect");
                            ui.add_space(15.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("Enter Device ID or Access Code:");
                                let mut device_id = "DEMO-DEVICE-123".to_string();
                                ui.text_edit_singleline(&mut device_id);
                                if ui.button("Connect").clicked() {
                                    // Simulate connection
                                }
                            });
                            
                            ui.add_space(20.0);
                            ui.separator();
                            ui.add_space(20.0);
                            
                            ui.label("Recent Connections:");
                            ui.horizontal(|ui| {
                                ui.button("🖥️ Office PC");
                                ui.button("💻 Laptop");
                                ui.button("🏠 Home Desktop");
                            });
                        });
                    });
                    
                    ui.add_space(30.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("⚙️ Open Settings").clicked() {
                            self.show_settings = true;
                        }
                        
                        if ui.button("💎 View Premium Plans").clicked() {
                            self.show_premium = true;
                        }
                    });
                });
            }
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status: Ready");
                ui.separator();
                ui.label("Network: Excellent");
                ui.separator();
                ui.label("CPU: 12% | Memory: 245MB");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("GenXLink v1.0.0");
                });
            });
        });
    }
}

fn main() -> Result<()> {
    env_logger::init();
    
    let app = GenXLinkApp {
        main_window: MainWindow::default(),
        settings_panel: SettingsPanel::new(),
        premium_features: PremiumFeaturesPanel::default(),
        show_settings: false,
        show_premium: false,
        theme: AppTheme::default(),
    };
    
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
    )?;

    Ok(())
}
