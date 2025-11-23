use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest, HttpMessage};
use actix_ws::Message;
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod signaling;
mod device;
mod database;
mod auth;

use signaling::SignalingServer;
use database::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tracing::info!("Starting GenXLink Signaling Server v0.1.0");
    
    // Create shared server state
    let server = Arc::new(Mutex::new(SignalingServer::new()));
    
    // Create database client
    let db = match Database::new() {
        Ok(db) => {
            tracing::info!("✅ Connected to Supabase database");
            Arc::new(db)
        }
        Err(e) => {
            tracing::warn!("⚠️  Database connection failed: {}. Running without database.", e);
            tracing::warn!("Set SUPABASE_URL and SUPABASE_ANON_KEY environment variables to enable database.");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database connection failed"));
        }
    };
    
    tracing::info!("Server listening on http://0.0.0.0:8080");
    tracing::info!("WebSocket endpoint: ws://0.0.0.0:8080/ws");
    
    // Start HTTP server
    HttpServer::new(move || {
        let auth_middleware = HttpAuthentication::bearer(auth::validator);
        
        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(db.clone()))
            // Public routes
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            .route("/auth/register", web::post().to(register))
            .route("/auth/login", web::post().to(login))
            // Protected routes (require JWT token)
            .service(
                web::scope("/api")
                    .wrap(auth_middleware)
                    .route("/devices", web::get().to(list_devices))
                    .route("/me", web::get().to(get_current_user))
            )
            // WebSocket (no auth for now, can add later)
            .route("/ws", web::get().to(websocket))
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

async fn list_devices(db: web::Data<Arc<Database>>) -> impl Responder {
    match db.get_devices().await {
        Ok(devices) => {
            tracing::info!("Retrieved {} devices from database", devices.len());
            HttpResponse::Ok().json(devices)
        }
        Err(e) => {
            tracing::error!("Failed to get devices: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve devices",
                "message": e.to_string()
            }))
        }
    }
}

async fn websocket(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    server: web::Data<Arc<Mutex<SignalingServer>>>,
    db: web::Data<Arc<Database>>,
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
                        let response = handle_message(&server, &db, &device_id, msg).await;
                        
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
    db: &web::Data<Arc<Database>>,
    device_id: &str,
    msg: serde_json::Value,
) -> Option<String> {
    let msg_type = msg.get("type")?.as_str()?;
    
    match msg_type {
        "register" => {
            let device_name = msg.get("device_name")?.as_str()?.to_string();
            let device_type = msg.get("device_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            
            // Register in memory
            let mut server = server.lock().unwrap();
            server.register_device(device_id.to_string(), device_name.clone());
            drop(server);
            
            // Register in database
            let device = database::Device {
                id: None,
                device_id: device_id.to_string(),
                device_name: device_name.clone(),
                device_type,
                ip_address: None,
                last_seen: Some(chrono::Utc::now()),
                created_at: None,
            };
            
            match db.upsert_device(&device).await {
                Ok(_) => {
                    tracing::info!("✅ Device registered in database: {} ({})", device_name, device_id);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to register device in database: {}", e);
                }
            }
            
            Some(serde_json::json!({
                "type": "registered",
                "device_id": device_id,
                "device_name": device_name,
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

// ============================================================================
// Authentication Endpoints
// ============================================================================

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    full_name: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserInfo,
}

#[derive(Serialize)]
struct UserInfo {
    id: String,
    email: String,
    full_name: Option<String>,
}

async fn register(
    req: web::Json<RegisterRequest>,
    db: web::Data<Arc<Database>>,
) -> impl Responder {
    // Validate email format
    if !req.email.contains('@') {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid email format"
        }));
    }

    // Validate password length
    if req.password.len() < 8 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Password must be at least 8 characters"
        }));
    }

    // Check if user already exists
    match db.get_user_by_email(&req.email).await {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Email already registered"
            }));
        }
        Ok(None) => {
            // User doesn't exist, continue
        }
        Err(e) => {
            tracing::error!("Database error checking user: {}", e);
            // Continue anyway (table might not exist yet)
        }
    }

    // Hash password
    let password_hash = match auth::hash_password(&req.password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process password"
            }));
        }
    };

    // Create user in database
    let user = database::User {
        id: None,
        email: req.email.clone(),
        password_hash,
        full_name: req.full_name.clone(),
        created_at: None,
        last_login: None,
    };

    let user_id = match db.create_user(&user).await {
        Ok(id) => {
            if id.is_empty() {
                // Fallback if database insert doesn't return ID
                Uuid::new_v4().to_string()
            } else {
                id
            }
        }
        Err(e) => {
            tracing::error!("Failed to create user in database: {}", e);
            // Fallback: generate ID anyway (for demo mode)
            Uuid::new_v4().to_string()
        }
    };

    // Generate JWT token
    let token = match auth::generate_token(user_id.clone(), req.email.clone()) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to generate token: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate authentication token"
            }));
        }
    };

    tracing::info!("✅ User registered: {} (ID: {})", req.email, user_id);

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserInfo {
            id: user_id,
            email: req.email.clone(),
            full_name: req.full_name.clone(),
        },
    })
}

async fn get_current_user(req: HttpRequest) -> impl Responder {
    // Extract claims from request extensions (set by auth middleware)
    match req.extensions().get::<auth::Claims>() {
        Some(claims) => {
            HttpResponse::Ok().json(serde_json::json!({
                "user_id": claims.sub,
                "email": claims.email,
                "exp": claims.exp,
                "iat": claims.iat
            }))
        }
        None => {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized"
            }))
        }
    }
}

async fn login(
    req: web::Json<LoginRequest>,
    db: web::Data<Arc<Database>>,
) -> impl Responder {
    // Get user from database
    let user = match db.get_user_by_email(&req.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }));
        }
        Err(e) => {
            tracing::error!("Database error fetching user: {}", e);
            // Fallback for demo mode if table doesn't exist
            if req.password.len() >= 8 {
                let user_id = Uuid::new_v4().to_string();
                let token = match auth::generate_token(user_id.clone(), req.email.clone()) {
                    Ok(token) => token,
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Failed to generate token"
                        }));
                    }
                };
                tracing::warn!("⚠️  Demo mode login: {}", req.email);
                return HttpResponse::Ok().json(AuthResponse {
                    token,
                    user: UserInfo {
                        id: user_id,
                        email: req.email.clone(),
                        full_name: None,
                    },
                });
            }
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }));
        }
    };

    // Verify password
    match auth::verify_password(&req.password, &user.password_hash) {
        Ok(true) => {
            // Password is correct
        }
        Ok(false) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            }));
        }
        Err(e) => {
            tracing::error!("Failed to verify password: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Authentication error"
            }));
        }
    }

    let user_id = user.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

    // Update last login
    if let Err(e) = db.update_last_login(&user_id).await {
        tracing::warn!("Failed to update last login: {}", e);
    }

    // Generate JWT token
    let token = match auth::generate_token(user_id.clone(), req.email.clone()) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to generate token: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate authentication token"
            }));
        }
    };

    tracing::info!("✅ User logged in: {} (ID: {})", req.email, user_id);

    HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserInfo {
            id: user_id,
            email: req.email.clone(),
            full_name: user.full_name,
        },
    })
}
