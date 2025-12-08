use anyhow::Result;
use eframe::egui;
use std::sync::Arc;

use crate::ui::{
    main_window::{MainWindow, AppTheme},
    settings_panel::{SettingsPanel, AppSettings, GeneralSettings, UISettings},
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌐 GenXLink Remote Desktop");
                ui.separator();
                
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
                self.settings_panel.show(ui, &self.theme);
            } else if self.show_premium {
                let _action = self.premium_features.show(ui);
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Welcome to GenXLink");
                    ui.add_space(20.0);
                    ui.label("Fast • Secure • Ultra-Low Latency Remote Desktop Access");
                    ui.add_space(30.0);
                    
                    if ui.button("⚙️ Open Settings").clicked() {
                        self.show_settings = true;
                    }
                    
                    if ui.button("💎 View Premium Plans").clicked() {
                        self.show_premium = true;
                    }
                });
            }
        });
    }
}

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_decorations(true)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "GenXLink Remote Desktop",
        native_options,
        Box::new(|_cc| Ok(Box::<GenXLinkApp>::default())),
    ).map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}
