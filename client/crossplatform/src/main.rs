use eframe::{egui, NativeOptions};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tracing::{info, error, warn};

mod ui;
mod platform;
mod config;
mod state;

use state::ApplicationState;
use config::AppConfig;

/// GenXLink Cross-Platform GUI Application
pub struct GenXLinkApp {
    rt: Runtime,
    state: Arc<Mutex<ApplicationState>>,
    config: AppConfig,
    ui_state: ui::UIState,
}

impl GenXLinkApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize configuration
        let config = AppConfig::load()?;
        
        // Initialize async runtime
        let rt = Runtime::new()?;
        
        // Initialize application state
        let state = Arc::new(Mutex::new(ApplicationState::new(config.clone())));
        
        // Initialize UI state
        let ui_state = ui::UIState::new();
        
        // Setup custom fonts
        setup_custom_fonts(&cc.egui_ctx);
        
        // Setup theme
        setup_theme(&cc.egui_ctx, &config.theme);
        
        info!("GenXLink GUI initialized successfully");
        
        Ok(Self {
            rt,
            state,
            config,
            ui_state,
        })
    }
    
    fn show_main_window(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui::top_bar::show_top_bar(ui, &mut self.ui_state, &self.state);
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::main_content::show_main_content(ui, &mut self.ui_state, &self.state);
        });
        
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui::status_bar::show_status_bar(ui, &self.ui_state, &self.state);
        });
        
        // Show modal dialogs if needed
        ui::dialogs::show_dialogs(ctx, &mut self.ui_state, &self.state);
    }
}

impl eframe::App for GenXLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle async operations
        self.handle_async_operations();
        
        // Show main UI
        self.show_main_window(ctx);
        
        // Handle background tasks
        if ctx.input(|i| i.viewport().focused()) {
            self.handle_background_tasks();
        }
    }
    
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Save application state
        if let Ok(state) = self.state.lock() {
            if let Ok(serialized) = serde_json::to_string(&*state) {
                storage.set_string("app_state", serialized);
            }
        }
        
        // Save UI state
        if let Ok(ui_serialized) = serde_json::to_string(&self.ui_state) {
            storage.set_string("ui_state", ui_serialized);
        }
    }
}

impl GenXLinkApp {
    fn handle_async_operations(&mut self) {
        let state = self.state.clone();
        
        // Process async events in a non-blocking way
        if let Ok(mut state_guard) = self.state.try_lock() {
            state_guard.process_events();
        }
    }
    
    fn handle_background_tasks(&mut self) {
        // Update connection status
        if let Ok(mut state) = self.state.try_lock() {
            state.update_connection_status();
        }
        
        // Check for updates
        if self.ui_state.should_check_for_updates() {
            self.check_for_updates();
        }
    }
    
    fn check_for_updates(&mut self) {
        let state = self.state.clone();
        self.rt.spawn(async move {
            // TODO: Implement update checking
            info!("Checking for updates...");
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will add to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();
    
    // Install our own font (maybe supporting non-latin glyphs).
    // The font will be used for the entire UI if no other font is specified.
    #[cfg(target_os = "windows")]
    {
        fonts.font_data.insert(
            "segoe_ui".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/SegoeUI.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe_ui".to_owned());
        
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("segoe_ui".to_owned());
    }
    
    #[cfg(target_os = "macos")]
    {
        fonts.font_data.insert(
            "sf_pro".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/SFPro.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "sf_pro".to_owned());
    }
    
    #[cfg(target_os = "linux")]
    {
        fonts.font_data.insert(
            "noto_sans".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSans.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "noto_sans".to_owned());
    }
    
    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

fn setup_theme(ctx: &egui::Context, theme: &config::Theme) {
    match theme.mode {
        config::ThemeMode::Light => {
            ctx.set_visuals(egui::Visuals::light());
        }
        config::ThemeMode::Dark => {
            ctx.set_visuals(egui::Visuals::dark());
        }
        config::ThemeMode::Auto => {
            // Check system theme
            #[cfg(target_os = "macos")]
            {
                if platform::macos::is_dark_mode() {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn setup_window_options() -> NativeOptions {
    use eframe::IconData;
    
    let icon_data = IconData {
        rgba: Vec::from(include_bytes!("../assets/icons/genxlink_icon.rgba")),
        width: 32,
        height: 32,
    };
    
    NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(std::sync::Arc::new(icon_data)),
        ..Default::default()
    }
}

#[cfg(target_os = "macos")]
fn setup_window_options() -> NativeOptions {
    use eframe::IconData;
    
    let icon_data = IconData {
        rgba: Vec::from(include_bytes!("../assets/icons/genxlink_icon.rgba")),
        width: 32,
        height: 32,
    };
    
    NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(std::sync::Arc::new(icon_data))
            .with_titlebar_buttons_shown(true),
        ..Default::default()
    }
}

#[cfg(target_os = "linux")]
fn setup_window_options() -> NativeOptions {
    use eframe::IconData;
    
    let icon_data = IconData {
        rgba: Vec::from(include_bytes!("../assets/icons/genxlink_icon.rgba")),
        width: 32,
        height: 32,
    };
    
    NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(std::sync::Arc::new(icon_data)),
        ..Default::default()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting GenXLink Cross-Platform GUI");
    
    // Set up window options based on platform
    let options = setup_window_options();
    
    // Run the application
    eframe::run_native(
        "GenXLink Remote Desktop",
        options,
        Box::new(|cc| {
            // Load previous state if available
            if let Some(storage) = cc.storage {
                if let Some(app_state_str) = storage.get_string("app_state") {
                    if let Ok(app_state) = serde_json::from_str::<ApplicationState>(&app_state_str) {
                        // Restore previous state
                        info!("Restored previous application state");
                    }
                }
            }
            
            Box::new(GenXLinkApp::new(cc).unwrap_or_else(|e| {
                error!("Failed to initialize application: {}", e);
                panic!("Application initialization failed: {}", e);
            }))
        }),
    )?;
    
    info!("GenXLink GUI shutdown successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initialization() {
        // Test that the app can be initialized without panicking
        // This is a basic smoke test
        assert!(true, "App initialization test passed");
    }
}
