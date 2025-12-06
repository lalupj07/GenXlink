//! Ultra Simple Dropdown Test

use eframe::egui;

#[derive(Default)]
pub struct SimpleApp {
    selected_language: String,
    selected_theme: String,
}

impl eframe::App for SimpleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        match self.selected_theme.as_str() {
            "Light" => ctx.set_visuals(egui::Visuals::light()),
            _ => ctx.set_visuals(egui::Visuals::dark()),
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🧪 Ultra Simple Dropdown Test");
            ui.separator();
            
            ui.label(format!("Language: {}", self.selected_language));
            ui.label(format!("Theme: {}", self.selected_theme));
            
            ui.add_space(20.0);
            
            // Simple button test first
            ui.heading("Button Test:");
            if ui.button("Click Me - Language").clicked() {
                self.selected_language = "Spanish".to_string();
                println!("Button clicked - Language changed to Spanish");
            }
            
            if ui.button("Click Me - Theme").clicked() {
                self.selected_theme = "Light".to_string();
                println!("Button clicked - Theme changed to Light");
            }
            
            ui.add_space(20.0);
            
            // Simple radio button test
            ui.heading("Radio Button Test:");
            ui.radio_value(&mut self.selected_language, "English".to_string(), "English");
            ui.radio_value(&mut self.selected_language, "Spanish".to_string(), "Spanish");
            ui.radio_value(&mut self.selected_language, "French".to_string(), "French");
            
            ui.add_space(10.0);
            ui.radio_value(&mut self.selected_theme, "Dark".to_string(), "Dark");
            ui.radio_value(&mut self.selected_theme, "Light".to_string(), "Light");
            
            ui.add_space(20.0);
            
            // Try the simplest possible dropdown
            ui.heading("Simple Dropdown Test:");
            egui::ComboBox::from_label("Language")
                .selected_text(&self.selected_language)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.selected_language == "English", "English").clicked() {
                        self.selected_language = "English".to_string();
                        println!("Dropdown - English selected");
                    }
                    if ui.selectable_label(self.selected_language == "Spanish", "Spanish").clicked() {
                        self.selected_language = "Spanish".to_string();
                        println!("Dropdown - Spanish selected");
                    }
                    if ui.selectable_label(self.selected_language == "French", "French").clicked() {
                        self.selected_language = "French".to_string();
                        println!("Dropdown - French selected");
                    }
                });
        });
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();
    
    let app = SimpleApp {
        selected_language: "English".to_string(),
        selected_theme: "Dark".to_string(),
    };
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Simple Dropdown Test",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    )
}
