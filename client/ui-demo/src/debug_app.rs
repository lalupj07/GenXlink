//! GenXLink with Error Handling

use eframe::egui;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "English".to_string(),
            theme: "Dark".to_string(),
        }
    }
}

#[derive(Default)]
pub struct GenXLinkApp {
    settings: AppSettings,
}

impl GenXLinkApp {
    pub fn new() -> Self {
        Self::default()
    }
}

impl eframe::App for GenXLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        match self.settings.theme.as_str() {
            "Light" => ctx.set_visuals(egui::Visuals::light()),
            _ => ctx.set_visuals(egui::Visuals::dark()),
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌐 GenXLink Remote Desktop");
            ui.separator();
            
            ui.label("Application is running successfully!");
            ui.label(format!("Language: {}", self.settings.language));
            ui.label(format!("Theme: {}", self.settings.theme));
            
            ui.add_space(20.0);
            
            // Test language dropdown
            ui.horizontal(|ui| {
                ui.label("Language:");
                let languages = ["English", "Spanish", "French"];
                
                egui::ComboBox::from_label("")
                    .selected_text(&self.settings.language)
                    .show_ui(ui, |ui| {
                        for &lang in languages.iter() {
                            if ui.selectable_label(self.settings.language == lang, lang).clicked() {
                                self.settings.language = lang.to_string();
                                println!("✅ Language changed to: {}", lang);
                            }
                        }
                    });
            });
            
            // Test theme dropdown
            ui.horizontal(|ui| {
                ui.label("Theme:");
                let themes = ["Dark", "Light"];
                
                egui::ComboBox::from_label("")
                    .selected_text(&self.settings.theme)
                    .show_ui(ui, |ui| {
                        for &theme in themes.iter() {
                            if ui.selectable_label(self.settings.theme == theme, theme).clicked() {
                                self.settings.theme = theme.to_string();
                                println!("✅ Theme changed to: {}", theme);
                            }
                        }
                    });
            });
            
            ui.add_space(20.0);
            ui.separator();
            
            ui.heading("Instructions:");
            ui.label("1. Try the language dropdown above");
            ui.label("2. Try the theme dropdown (Light should turn interface white)");
            ui.label("3. Check console for confirmation messages");
            ui.label("4. Close window to exit");
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    println!("🚀 Starting GenXLink Remote Desktop...");
    
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("📦 Logger initialized");
    
    let app = GenXLinkApp::new();
    println!("🎯 Application created");
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("🌐 GENXLINK REMOTE DESKTOP - DEBUG VERSION"),
        ..Default::default()
    };
    
    println!("🖥️ Starting GUI...");
    println!("📋 If you see this, the app should open in a new window");
    println!("⚠️  Look for a window titled 'GENXLINK REMOTE DESKTOP'");
    
    let result = eframe::run_native(
        "GenXLink Remote Desktop - Debug Version",
        native_options,
        Box::new(|cc| {
            println!("⚙️ Configuring eframe...");
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    );
    
    match result {
        Ok(_) => println!("✅ Application closed successfully"),
        Err(e) => println!("❌ Application error: {}", e),
    }
    
    Ok(())
}
