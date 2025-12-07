// Copyright (c) 2025 GenXis Innovations
// Licensed under the Apache License, Version 2.0
// Contact: genxisinnovation@outlook.com

use anyhow::Result;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tracing::info;
use wry::WebViewBuilder;

mod html_content;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Starting GenXLink WebView Test Application");

    // Create event loop
    let event_loop = EventLoop::new();

    // Create window
    let window = WindowBuilder::new()
        .with_title("GenXLink WebView Test")
        .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)?;

    // Get HTML content
    let html_content = html_content::get_test_html();

    // Create webview
    let _webview = WebViewBuilder::new(&window)
        .with_html(&html_content)
        .with_devtools(true)
        .with_ipc_handler(|request| {
            // Handle IPC messages from JavaScript
            let body = request.body();
            info!("IPC message from JS: {}", body);
            
            // Parse and handle different message types
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) {
                handle_ipc_message(msg);
            }
        })
        .build()?;

    info!("WebView created successfully");

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                // Window resized - webview handles this automatically
            }
            _ => {}
        }
    });
}

fn handle_ipc_message(msg: serde_json::Value) {
    if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
        match msg_type {
            "connect" => {
                let connection_id = msg.get("connectionId").and_then(|v| v.as_str()).unwrap_or("");
                info!("Connect request for ID: {}", connection_id);
            }
            "disconnect" => {
                info!("Disconnect request");
            }
            "startScreenShare" => {
                info!("Start screen share request");
            }
            "stopScreenShare" => {
                info!("Stop screen share request");
            }
            "log" => {
                let message = msg.get("message").and_then(|v| v.as_str()).unwrap_or("");
                info!("JS Log: {}", message);
            }
            _ => {
                info!("Unknown message type: {}", msg_type);
            }
        }
    }
}
