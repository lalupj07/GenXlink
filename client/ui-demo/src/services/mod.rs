// Copyright (c) 2025 GenXis Innovations
// Backend Services Module - Connects UI to Core Libraries

pub mod connection_service;
pub mod file_transfer_service;
pub mod session_service;
pub mod discovery_service;

pub use connection_service::ConnectionService;
pub use file_transfer_service::FileTransferService;
pub use session_service::SessionService;
pub use discovery_service::DiscoveryService;

use std::sync::{Arc, Mutex};

/// Main backend service manager
pub struct BackendServices {
    pub connection: Arc<Mutex<ConnectionService>>,
    pub file_transfer: Arc<Mutex<FileTransferService>>,
    pub session: Arc<Mutex<SessionService>>,
    pub discovery: Arc<Mutex<DiscoveryService>>,
}

impl BackendServices {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(ConnectionService::new())),
            file_transfer: Arc::new(Mutex::new(FileTransferService::new())),
            session: Arc::new(Mutex::new(SessionService::new())),
            discovery: Arc::new(Mutex::new(DiscoveryService::new())),
        }
    }

    /// Start all background services
    pub fn start(&self) {
        println!("🚀 Starting backend services...");
        
        // Start discovery service (synchronous initialization)
        if let Ok(mut d) = self.discovery.lock() {
            d.start_discovery_sync();
        }

        println!("✅ Backend services started");
    }

    /// Stop all services
    pub fn stop(&self) {
        println!("🛑 Stopping backend services...");
        if let Ok(mut d) = self.discovery.lock() {
            d.stop_discovery();
        }
        println!("✅ Backend services stopped");
    }
}

impl Default for BackendServices {
    fn default() -> Self {
        Self::new()
    }
}
