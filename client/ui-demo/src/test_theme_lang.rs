//! Simple Test for Theme & Language Switching

use eframe::egui;

#[derive(Default)]
pub struct TestApp {
    language: String,
    theme: String,
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        match self.theme.as_str() {
            "Light" => ctx.set_visuals(egui::Visuals::light()),
            _ => ctx.set_visuals(egui::Visuals::dark()),
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🧪 Theme & Language Test");
            ui.separator();
            
            ui.label(format!("Current Language: {}", self.language));
            ui.label(format!("Current Theme: {}", self.theme));
            
            ui.add_space(20.0);
            
            // Language test
            ui.horizontal(|ui| {
                ui.label("Language:");
                let languages = ["English", "Spanish", "French"];
                
                egui::ComboBox::from_label("")
                    .selected_text(&self.language)
                    .show_ui(ui, |ui| {
                        for &lang in languages.iter() {
                            if ui.selectable_label(self.language == lang, lang).clicked() {
                                self.language = lang.to_string();
                                println!("✅ LANGUAGE CHANGED TO: {}", self.language);
                            }
                        }
                    });
            });
            
            // Theme test
            ui.horizontal(|ui| {
                ui.label("Theme:");
                let themes = ["Dark", "Light"];
                
                egui::ComboBox::from_label("")
                    .selected_text(&self.theme)
                    .show_ui(ui, |ui| {
                        for &theme in themes.iter() {
                            if ui.selectable_label(self.theme == theme, theme).clicked() {
                                self.theme = theme.to_string();
                                println!("✅ THEME CHANGED TO: {}", self.theme);
                            }
                        }
                    });
            });
            
            ui.add_space(20.0);
            ui.separator();
            
            ui.heading("Instructions:");
            ui.label("1. Try changing the language dropdown");
            ui.label("2. Try changing the theme dropdown");
            ui.label("3. Check the console/output window for messages");
            ui.label("4. Light theme should turn the interface white");
        });
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    
    let app = TestApp {
        language: "English".to_string(),
        theme: "Dark".to_string(),
    };
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GenXLink Theme & Language Test",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    )
}
