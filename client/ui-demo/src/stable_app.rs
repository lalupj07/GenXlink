//! GenXLink Stable Version

use eframe::egui;

#[derive(Default)]
pub struct GenXLinkApp {
    language: String,
    theme: String,
    counter: i32,
}

impl eframe::App for GenXLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        match self.theme.as_str() {
            "Light" => ctx.set_visuals(egui::Visuals::light()),
            _ => ctx.set_visuals(egui::Visuals::dark()),
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌐 GenXLink Remote Desktop - STABLE VERSION");
            ui.separator();
            
            ui.label("✅ Application is running successfully!");
            ui.label(format!("Frame: {} | Language: {} | Theme: {}", self.counter, self.language, self.theme));
            
            ui.add_space(20.0);
            
            // Test language dropdown
            ui.horizontal(|ui| {
                ui.label("Language:");
                let languages = ["English", "Spanish", "French", "German", "Chinese", "Japanese"];
                
                egui::ComboBox::from_label("")
                    .selected_text(&self.language)
                    .show_ui(ui, |ui| {
                        for &lang in languages.iter() {
                            if ui.selectable_label(self.language == lang, lang).clicked() {
                                self.language = lang.to_string();
                                println!("✅ Language changed to: {}", lang);
                            }
                        }
                    });
            });
            
            // Test theme dropdown
            ui.horizontal(|ui| {
                ui.label("Theme:");
                let themes = ["Dark", "Light", "System"];
                
                egui::ComboBox::from_label("")
                    .selected_text(&self.theme)
                    .show_ui(ui, |ui| {
                        for &theme in themes.iter() {
                            if ui.selectable_label(self.theme == theme, theme).clicked() {
                                self.theme = theme.to_string();
                                println!("✅ Theme changed to: {}", theme);
                            }
                        }
                    });
            });
            
            ui.add_space(20.0);
            
            // Interactive elements
            if ui.button("Test Button (Click Me!)").clicked() {
                self.counter += 1;
                println!("🎯 Button clicked! Counter: {}", self.counter);
            }
            
            ui.add_space(20.0);
            ui.separator();
            
            ui.heading("📋 Instructions:");
            ui.label("1. ✅ This window should stay open");
            ui.label("2. 🎨 Try the theme dropdown (Light turns interface white)");
            ui.label("3. 🌍 Try the language dropdown (6 languages available)");
            ui.label("4. 🖱️ Click the test button to verify interactivity");
            ui.label("5. 📊 Watch the frame counter update");
            ui.label("6. ❌ Close this window manually to exit");
            
            ui.add_space(10.0);
            ui.label("🔍 Check console for confirmation messages");
        });
        
        // Update counter to show the app is running
        self.counter += 1;
        
        // Request repaint to keep the app running
        ctx.request_repaint();
    }
}

fn main() {
    println!("🚀 Starting GenXLink Remote Desktop...");
    
    // Simple panic handler
    std::panic::set_hook(Box::new(|panic_info| {
        println!("💥 PANIC: {}", panic_info);
    }));
    
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("📦 Logger initialized");
    
    let app = GenXLinkApp {
        language: "English".to_string(),
        theme: "Dark".to_string(),
        counter: 0,
    };
    
    println!("🎯 Application created");
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    println!("🖥️ Starting GUI...");
    println!("📋 WINDOW SHOULD OPEN NOW - Look for GenXLink window");
    
    match eframe::run_native(
        "GenXLink Remote Desktop - Stable Version",
        native_options,
        Box::new(|cc| {
            println!("⚙️ Configuring eframe...");
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app)
        }),
    ) {
        Ok(_) => println!("✅ Application closed successfully"),
        Err(e) => println!("❌ Application error: {}", e),
    }
}
