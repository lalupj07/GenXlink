use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use uuid::Uuid;

use genxlink_client_core::{
    auth_service::AuthService,
    webrtc_integration::{WebRTCIntegration, P2PConfiguration, ConnectionMode},
    p2p_integration::P2PIntegrationEvent,
};

/// Simple P2P client example
/// Demonstrates peer discovery and direct connection without central server
pub struct P2PClient {
    device_id: String,
    integration: Arc<WebRTCIntegration>,
    event_rx: mpsc::UnboundedReceiver<genxlink_client_core::webrtc_integration::IntegrationEvent>,
}

impl P2PClient {
    /// Create a new P2P client
    pub async fn new(device_name: String) -> Result<Self> {
        // Generate unique device ID
        let device_id = Uuid::new_v4().to_string();
        
        // Create auth service (minimal for P2P mode)
        let auth_service = AuthService::new(
            "http://localhost:8000".to_string(), // Won't be used in P2P mode
            "test-api-key".to_string(),
        );
        
        // Create P2P configuration
        let p2p_config = P2PConfiguration {
            device_name: device_name.clone(),
            auto_connect: false,
            auto_stream: false,
            discovery_interval_secs: 30,
            connection_timeout_secs: 30,
        };
        
        // Create WebRTC integration in P2P mode
        let (integration, event_rx) = WebRTCIntegration::new_p2p(
            genxlink_protocol::DeviceId(device_id.clone()),
            auth_service,
            p2p_config,
        );
        
        let client = Self {
            device_id,
            integration: Arc::new(integration),
            event_rx,
        };
        
        Ok(client)
    }
    
    /// Start the P2P client
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting P2P client with device ID: {}", self.device_id);
        
        // Initialize the integration
        self.integration.initialize().await?;
        
        info!("P2P client started successfully");
        Ok(())
    }
    
    /// Run the client event loop
    pub async fn run_event_loop(&mut self) -> Result<()> {
        info!("Starting P2P client event loop");
        
        loop {
            tokio::select! {
                event = self.event_rx.recv() => {
                    match event {
                        Some(event) => self.handle_integration_event(event).await?,
                        None => {
                            warn!("Event channel closed, stopping event loop");
                            break;
                        }
                    }
                }
                
                // Periodic status check
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                    self.print_status().await;
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle integration events
    async fn handle_integration_event(
        &self,
        event: genxlink_client_core::webrtc_integration::IntegrationEvent,
    ) -> Result<()> {
        match event {
            genxlink_client_core::webrtc_integration::IntegrationEvent::AuthenticationRequired => {
                info!("Authentication required (not used in P2P mode)");
            }
            genxlink_client_core::webrtc_integration::IntegrationEvent::Authenticated => {
                info!("Authenticated successfully");
            }
            genxlink_client_core::webrtc_integration::IntegrationEvent::Connected => {
                info!("Connected to peer");
            }
            genxlink_client_core::webrtc_integration::IntegrationEvent::StreamingStarted => {
                info!("Streaming started");
            }
            genxlink_client_core::webrtc_integration::IntegrationEvent::StreamingStopped => {
                info!("Streaming stopped");
            }
            genxlink_client_core::webrtc_integration::IntegrationEvent::Error(error) => {
                error!("Integration error: {}", error);
            }
            genxlink_client_core::webrtc_integration::IntegrationEvent::StateChanged(state) => {
                info!("State changed to: {:?}", state);
            }
        }
        
        Ok(())
    }
    
    /// Print current status
    async fn print_status(&self) {
        let state = self.integration.get_state().await;
        let is_p2p = self.integration.is_p2p_mode();
        
        if is_p2p {
            let peers = self.integration.get_discovered_peers().await;
            let stats = self.integration.get_p2p_stats().await;
            
            info!("=== P2P Client Status ===");
            info!("Device ID: {}", self.device_id);
            info!("State: {:?}", state);
            info!("Discovered peers: {}", peers.len());
            
            if let Some(stats) = stats {
                info!("Active connections: {}", stats.active_connections_count);
                info!("Active sessions: {}", stats.active_sessions_count);
            }
            
            // List discovered peers
            for peer in peers {
                info!("  - {} ({}) at {}", peer.device_name, peer.device_id, peer.ip_address);
            }
            
            info!("========================");
        } else {
            info!("=== Client Status ===");
            info!("Device ID: {}", self.device_id);
            info!("State: {:?}", state);
            info!("Mode: Server-based");
            info!("====================");
        }
    }
    
    /// Connect to a discovered peer
    pub async fn connect_to_peer(&self, peer_device_id: String) -> Result<()> {
        info!("Connecting to peer: {}", peer_device_id);
        
        self.integration.connect_to_p2p_peer(genxlink_protocol::DeviceId(peer_device_id)).await?;
        
        info!("Connection initiated");
        Ok(())
    }
    
    /// Start streaming with a peer
    pub async fn start_streaming(&self, peer_device_id: String, monitor_index: usize) -> Result<()> {
        info!("Starting streaming with peer: {} on monitor {}", peer_device_id, monitor_index);
        
        self.integration.start_streaming(
            genxlink_protocol::DeviceId(peer_device_id),
            monitor_index,
            None, // No signaling server URL in P2P mode
        ).await?;
        
        info!("Streaming started");
        Ok(())
    }
    
    /// Stop streaming
    pub async fn stop_streaming(&self) -> Result<()> {
        info!("Stopping streaming");
        
        self.integration.stop_streaming().await?;
        
        info!("Streaming stopped");
        Ok(())
    }
    
    /// Disconnect from a peer
    pub async fn disconnect_from_peer(&self, peer_device_id: String) -> Result<()> {
        info!("Disconnecting from peer: {}", peer_device_id);
        
        self.integration.disconnect_from_p2p_peer(genxlink_protocol::DeviceId(peer_device_id)).await?;
        
        info!("Disconnected");
        Ok(())
    }
    
    /// Get discovered peers
    pub async fn get_discovered_peers(&self) -> Vec<genxlink_client_core::p2p_discovery::PeerInfo> {
        self.integration.get_discovered_peers().await
    }
    
    /// Stop the client
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping P2P client");
        
        // Stop streaming if active
        if self.integration.is_streaming().await {
            self.integration.stop_streaming().await?;
        }
        
        info!("P2P client stopped");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Create P2P client
    let mut client = P2PClient::new("GenXLink P2P Test Device".to_string()).await?;
    
    // Start the client
    client.start().await?;
    
    info!("GenXLink P2P Client started!");
    info!("This client will discover other GenXLink devices on the local network");
    info!("Press Ctrl+C to stop");
    
    // Run the event loop
    tokio::select! {
        result = client.run_event_loop() => {
            if let Err(e) = result {
                error!("Event loop error: {}", e);
            }
        }
        
        // Handle Ctrl+C
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
    }
    
    // Stop the client
    client.stop().await?;
    
    info!("GenXLink P2P Client stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_p2p_client_creation() {
        let client = P2PClient::new("Test Device".to_string()).await;
        assert!(client.is_ok());
    }
    
    #[tokio::test]
    async fn test_p2p_client_start_stop() {
        let mut client = P2PClient::new("Test Device".to_string()).await.unwrap();
        
        // Start client
        assert!(client.start().await.is_ok());
        
        // Check if P2P mode is enabled
        assert!(client.integration.is_p2p_mode());
        
        // Stop client
        assert!(client.stop().await.is_ok());
    }
}
