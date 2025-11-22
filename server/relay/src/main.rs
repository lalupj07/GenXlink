use anyhow::Result;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("GenXLink Relay Server starting...");
    
    // TODO: Implement TURN relay server
    // This will relay traffic when P2P connection fails
    // Uses UDP/TCP for relaying data between peers
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3478));
    info!("Relay server listening on {}", addr);
    
    // Keep server running
    tokio::signal::ctrl_c().await?;
    info!("Relay server shutting down");
    
    Ok(())
}
