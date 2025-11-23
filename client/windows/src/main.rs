// Copyright (c) 2025 GenXis Innovations
// Licensed under the Apache License, Version 2.0
// Contact: genxisinnovation@outlook.com

use anyhow::Result;
use tracing::info;
use genxlink_protocol::device::generate_device_id;

mod app;
mod config;
mod license_manager;
mod ui;
mod icon;

use ui::GenXLinkApp;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("GenXLink Windows Client v{}", env!("CARGO_PKG_VERSION"));
    
    // Generate or load device ID
    let device_id = generate_device_id();
    info!("Device ID: {}", device_id);
    
    // Configure window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(icon::load_icon()),
        ..Default::default()
    };
    
    // Run the application
    eframe::run_native(
        "GenXLink",
        options,
        Box::new(|cc| Ok(Box::new(GenXLinkApp::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}

