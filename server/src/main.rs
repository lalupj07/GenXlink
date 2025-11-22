use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod signaling;
mod device;

use signaling::SignalingServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tracing::info!("Starting GenXLink Signaling Server v0.1.0");
    
    // Create shared server state
    let server = Arc::new(Mutex::new(SignalingServer::new()));
    
    tracing::info!("Server listening on http://0.0.0.0:8080");
    tracing::info!("WebSocket endpoint: ws://0.0.0.0:8080/ws");
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            .route("/ws", web::get().to(websocket))
            .route("/devices", web::get().to(list_devices))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>GenXLink Server</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
                h1 { color: #3B82F6; }
                .status { color: #22C55E; font-weight: bold; }
                code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
            </style>
        </head>
        <body>
            <h1>🚀 GenXLink Signaling Server</h1>
            <p class="status">✅ Server is running!</p>
            
            <h2>Endpoints:</h2>
            <ul>
                <li><code>GET /</code> - This page</li>
                <li><code>GET /health</code> - Health check</li>
                <li><code>GET /devices</code> - List connected devices</li>
                <li><code>WS /ws</code> - WebSocket connection</li>
            </ul>
            
            <h2>WebSocket Protocol:</h2>
            <pre>
// Register device
{ "type": "register", "device_id": "abc-123", "device_name": "My PC" }

// Request connection
{ "type": "connect", "target_device_id": "xyz-789" }

// WebRTC signaling
{ "type": "offer", "sdp": "..." }
{ "type": "answer", "sdp": "..." }
{ "type": "ice_candidate", "candidate": "..." }
            </pre>
            
            <p><strong>Version:</strong> 0.1.0</p>
            <p><strong>Status:</strong> <span class="status">Online</span></p>
        </body>
        </html>
        "#
    )
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": "0.1.0",
        "service": "genxlink-signaling-server"
    }))
}

async fn list_devices(server: web::Data<Arc<Mutex<SignalingServer>>>) -> impl Responder {
    let server = server.lock().unwrap();
    let devices = server.get_devices();
    HttpResponse::Ok().json(devices)
}

async fn websocket(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    server: web::Data<Arc<Mutex<SignalingServer>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    
    let device_id = Uuid::new_v4().to_string();
    tracing::info!("New WebSocket connection: {}", device_id);
    
    // Spawn task to handle messages
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!("Received message: {}", text);
                    
                    // Parse and handle message
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        let response = handle_message(&server, &device_id, msg).await;
                        
                        if let Some(response_text) = response {
                            let _ = session.text(response_text).await;
                        }
                    }
                }
                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                Message::Close(_) => {
                    tracing::info!("WebSocket closed: {}", device_id);
                    break;
                }
                _ => {}
            }
        }
    });
    
    Ok(response)
}

async fn handle_message(
    server: &web::Data<Arc<Mutex<SignalingServer>>>,
    device_id: &str,
    msg: serde_json::Value,
) -> Option<String> {
    let msg_type = msg.get("type")?.as_str()?;
    
    match msg_type {
        "register" => {
            let device_name = msg.get("device_name")?.as_str()?.to_string();
            let mut server = server.lock().unwrap();
            server.register_device(device_id.to_string(), device_name);
            
            Some(serde_json::json!({
                "type": "registered",
                "device_id": device_id,
                "status": "success"
            }).to_string())
        }
        "ping" => {
            Some(serde_json::json!({
                "type": "pong",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }).to_string())
        }
        _ => {
            tracing::warn!("Unknown message type: {}", msg_type);
            None
        }
    }
}
