use serde::{Deserialize, Serialize};
use std::env;

/// Server configuration for GenXLink client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub api_server_url: String,
    pub signaling_server_url: String,
    pub relay_server_url: String,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Production,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::from_environment()
    }
}

impl ServerConfig {
    /// Create configuration from environment variables
    pub fn from_environment() -> Self {
        let environment = match env::var("GENXLINK_ENV").unwrap_or_else(|_| "development".to_string()).as_str() {
            "production" => Environment::Production,
            _ => Environment::Development,
        };

        match environment {
            Environment::Production => Self::production(),
            Environment::Development => Self::development(),
        }
    }

    /// Development configuration (localhost)
    pub fn development() -> Self {
        Self {
            api_server_url: env::var("GENXLINK_API_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            signaling_server_url: env::var("GENXLINK_SIGNALING_URL")
                .unwrap_or_else(|_| "ws://localhost:8080".to_string()),
            relay_server_url: env::var("GENXLINK_RELAY_URL")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            environment: Environment::Development,
        }
    }

    /// Production configuration
    pub fn production() -> Self {
        Self {
            api_server_url: env::var("GENXLINK_API_URL")
                .unwrap_or_else(|_| "https://your-domain.com/api".to_string()),
            signaling_server_url: env::var("GENXLINK_SIGNALING_URL")
                .unwrap_or_else(|_| "wss://your-domain.com/ws".to_string()),
            relay_server_url: env::var("GENXLINK_RELAY_URL")
                .unwrap_or_else(|_| "https://your-domain.com/relay".to_string()),
            environment: Environment::Production,
        }
    }

    /// Get WebSocket URL for signaling
    pub fn get_websocket_url(&self) -> String {
        format!("{}/ws", self.signaling_server_url)
    }

    /// Get authentication endpoint
    pub fn get_auth_endpoint(&self) -> String {
        format!("{}/auth", self.api_server_url)
    }

    /// Get devices endpoint
    pub fn get_devices_endpoint(&self) -> String {
        format!("{}/devices", self.api_server_url)
    }

    /// Get sessions endpoint
    pub fn get_sessions_endpoint(&self) -> String {
        format!("{}/sessions", self.api_server_url)
    }

    /// Check if using production configuration
    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    /// Check if using development configuration
    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_development_config() {
        let config = ServerConfig::development();
        assert_eq!(config.api_server_url, "http://localhost:8000");
        assert_eq!(config.signaling_server_url, "ws://localhost:8080");
        assert_eq!(config.relay_server_url, "http://localhost:9000");
        assert!(config.is_development());
        assert!(!config.is_production());
    }

    #[test]
    fn test_production_config() {
        let config = ServerConfig::production();
        assert_eq!(config.api_server_url, "https://your-domain.com/api");
        assert_eq!(config.signaling_server_url, "wss://your-domain.com/ws");
        assert_eq!(config.relay_server_url, "https://your-domain.com/relay");
        assert!(config.is_production());
        assert!(!config.is_development());
    }

    #[test]
    fn test_environment_override() {
        env::set_var("GENXLINK_ENV", "production");
        env::set_var("GENXLINK_API_URL", "https://custom.example.com/api");
        
        let config = ServerConfig::from_environment();
        assert!(config.is_production());
        assert_eq!(config.api_server_url, "https://custom.example.com/api");
        
        env::remove_var("GENXLINK_ENV");
        env::remove_var("GENXLINK_API_URL");
    }

    #[test]
    fn test_endpoint_urls() {
        let config = ServerConfig::development();
        assert_eq!(config.get_websocket_url(), "ws://localhost:8080/ws");
        assert_eq!(config.get_auth_endpoint(), "http://localhost:8000/auth");
        assert_eq!(config.get_devices_endpoint(), "http://localhost:8000/devices");
        assert_eq!(config.get_sessions_endpoint(), "http://localhost:8000/sessions");
    }
}
