use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use genxlink_relay::{
    RelayServer, RelayServerConfig, GeographicLocation, LoadBalancingStrategy, 
    BalancingAlgorithm, HealthStatus, RelayNode,
};

#[derive(Parser, Debug)]
#[command(name = "genxlink-relay")]
#[command(about = "GenXLink Advanced Relay Server")]
#[command(version)]
pub struct Args {
    /// Server configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Server bind address
    #[arg(short, long, default_value = "0.0.0.0")]
    bind: String,

    /// Server port
    #[arg(short, long, default_value = "8081")]
    port: u16,

    /// Enable geographic routing
    #[arg(long)]
    geographic_routing: bool,

    /// Enable bandwidth management
    #[arg(long)]
    bandwidth_management: bool,

    /// Enable metrics collection
    #[arg(long)]
    metrics: bool,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(&args.log_level);

    info!("Starting GenXLink Relay Server v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config(&args).await?;

    // Create and start relay server
    let mut relay_server = RelayServer::new(config).await?;
    
    // Setup signal handlers for graceful shutdown
    setup_signal_handlers().await?;

    // Start the server
    relay_server.start().await?;

    info!("Relay server started successfully on {}:{}", args.bind, args.port);

    // Run the server until shutdown
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal");

    // Graceful shutdown
    relay_server.shutdown().await?;
    info!("Relay server shutdown complete");

    Ok(())
}

async fn load_config(args: &Args) -> Result<RelayServerConfig> {
    // Default configuration
    let mut config = RelayServerConfig::default();
    
    // Override with command line arguments
    config.endpoint = format!("ws://{}:{}", args.bind, args.port);
    config.geographic_routing_enabled = args.geographic_routing;
    config.bandwidth_management_enabled = args.bandwidth_management;

    // Load from file if it exists
    if std::path::Path::new(&args.config).exists() {
        info!("Loading configuration from: {}", args.config);
        let settings = config::Config::builder()
            .add_source(config::File::with_name(&args.config))
            .build()?;

        // Override config with file settings
        if let Ok(endpoint) = settings.get_string("server.endpoint") {
            config.endpoint = endpoint;
        }
        if let Ok(location) = settings.get_table("server.location") {
            config.location.country = location.get("country").unwrap().clone().into_string().unwrap();
            config.location.region = location.get("region").unwrap().clone().into_string().unwrap();
            config.location.city = location.get("city").unwrap().clone().into_string().unwrap();
            config.location.latitude = location.get("latitude").unwrap().clone().into_float().unwrap() as f64;
            config.location.longitude = location.get("longitude").unwrap().clone().into_float().unwrap() as f64;
            config.location.timezone = location.get("timezone").unwrap().clone().into_string().unwrap();
        }
        if let Ok(capacity) = settings.get("server.capacity").and_then(|v| v.clone().into_int().ok()) {
            config.capacity = capacity as u32;
        }
        if let Ok(bandwidth) = settings.get("server.bandwidth_limit").and_then(|v| v.clone().into_int().ok()) {
            config.bandwidth_limit = bandwidth as u64;
        }
        if let Ok(strategy) = settings.get_table("load_balancing") {
            if let Ok(algorithm) = strategy.get_string("algorithm") {
                config.load_balancing_strategy.algorithm = match algorithm.as_str() {
                    "round_robin" => BalancingAlgorithm::RoundRobin,
                    "weighted_round_robin" => BalancingAlgorithm::WeightedRoundRobin,
                    "least_connections" => BalancingAlgorithm::LeastConnections,
                    "weighted_least_connections" => BalancingAlgorithm::WeightedLeastConnections,
                    "geographic" => BalancingAlgorithm::Geographic,
                    "performance_based" => BalancingAlgorithm::PerformanceBased,
                    "adaptive" => BalancingAlgorithm::Adaptive,
                    _ => BalancingAlgorithm::Adaptive,
                };
            }
            if let Ok(geo_weight) = strategy.get("geographic_weight").and_then(|v| v.clone().into_float().ok()) {
                config.load_balancing_strategy.geographic_weight = geo_weight;
            }
            if let Ok(perf_weight) = strategy.get("performance_weight").and_then(|v| v.clone().into_float().ok()) {
                config.load_balancing_strategy.performance_weight = perf_weight;
            }
            if let Ok(cap_weight) = strategy.get("capacity_weight").and_then(|v| v.clone().into_float().ok()) {
                config.load_balancing_strategy.capacity_weight = cap_weight;
            }
            if let Ok(latency_threshold) = strategy.get("latency_threshold").and_then(|v| v.clone().into_int().ok()) {
                config.load_balancing_strategy.latency_threshold = latency_threshold as u32;
            }
            if let Ok(bandwidth_threshold) = strategy.get("bandwidth_threshold").and_then(|v| v.clone().into_float().ok()) {
                config.load_balancing_strategy.bandwidth_threshold = bandwidth_threshold;
            }
        }
    } else {
        warn!("Configuration file not found: {}, using defaults", args.config);
    }

    info!("Loaded configuration:");
    info!("  Endpoint: {}", config.endpoint);
    info!("  Location: {}, {}, {}", config.location.city, config.location.region, config.location.country);
    info!("  Capacity: {} sessions", config.capacity);
    info!("  Bandwidth Limit: {} Mbps", config.bandwidth_limit);
    info!("  Load Balancing: {:?}", config.load_balancing_strategy.algorithm);
    info!("  Geographic Routing: {}", config.geographic_routing_enabled);
    info!("  Bandwidth Management: {}", config.bandwidth_management_enabled);

    Ok(config)
}

fn init_logging(level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn setup_signal_handlers() -> Result<()> {
    // Setup signal handlers for graceful shutdown
    tokio::spawn(async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed to setup Ctrl-C handler: {}", e);
        }
    });

    // Setup SIGTERM handler (Unix systems)
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        
        let mut sigterm = signal(SignalKind::terminate())?;
        tokio::spawn(async move {
            if let Err(e) = sigterm.recv().await {
                error!("Failed to setup SIGTERM handler: {}", e);
            }
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_loading() {
        let args = Args {
            config: "nonexistent.toml".to_string(),
            bind: "127.0.0.1".to_string(),
            port: 8081,
            geographic_routing: true,
            bandwidth_management: true,
            metrics: true,
            log_level: "debug".to_string(),
        };

        let config = load_config(&args).await.unwrap();
        
        assert_eq!(config.endpoint, "ws://127.0.0.1:8081");
        assert!(config.geographic_routing_enabled);
        assert!(config.bandwidth_management_enabled);
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = RelayServerConfig::default();
        
        let mut server = RelayServer::new(config).await.unwrap();
        server.start().await.unwrap();
        
        // Test that server starts without panicking
        assert!(true);
        
        server.shutdown().await.unwrap();
    }
}
