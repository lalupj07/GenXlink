use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<TurnServerConfig>,
    pub video: VideoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServerConfig {
    pub url: String,
    pub username: String,
    pub credential: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: "wss://localhost:8080".to_string(),
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
            ],
            turn_servers: vec![],
            video: VideoConfig {
                width: 1920,
                height: 1080,
                fps: 30,
                bitrate: 2_000_000,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try to load from config file, otherwise use defaults
        // TODO: Implement config file loading
        Ok(Self::default())
    }
}
