//! Connection ID Generator
//! 
//! Creates a unique, easy-to-share ID for remote connections.
//! Similar to TeamViewer/AnyDesk IDs - short, memorable, and unique.

use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use rand::Rng;

/// Connection ID for remote access
/// Format: XXX-XXX-XXX (9 digits, easy to read/share)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionId {
    /// The 9-digit connection ID
    pub id: String,
    /// Display format with dashes (XXX-XXX-XXX)
    pub display_id: String,
    /// When this ID was generated
    pub created_at: String,
    /// Device name
    pub device_name: String,
}

impl ConnectionId {
    /// Get or create the connection ID
    pub fn get_or_create() -> Result<Self, String> {
        let config_dir = Self::get_config_dir()?;
        let id_file = config_dir.join("connection_id.json");
        
        if id_file.exists() {
            Self::load_from_file(&id_file)
        } else {
            Self::create_new(&id_file)
        }
    }
    
    /// Generate a new connection ID (for reset)
    pub fn regenerate() -> Result<Self, String> {
        let config_dir = Self::get_config_dir()?;
        let id_file = config_dir.join("connection_id.json");
        Self::create_new(&id_file)
    }
    
    /// Get the config directory
    fn get_config_dir() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("GenXLink");
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        Ok(config_dir)
    }
    
    /// Create a new connection ID
    fn create_new(id_file: &PathBuf) -> Result<Self, String> {
        let mut rng = rand::thread_rng();
        
        // Generate 9 random digits
        let digits: String = (0..9)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();
        
        // Format as XXX-XXX-XXX
        let display_id = format!(
            "{}-{}-{}",
            &digits[0..3],
            &digits[3..6],
            &digits[6..9]
        );
        
        // Get device name
        let device_name = whoami::devicename();
        
        let connection_id = Self {
            id: digits,
            display_id: display_id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            device_name,
        };
        
        // Save to file
        let json = serde_json::to_string_pretty(&connection_id)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        fs::write(id_file, json)
            .map_err(|e| format!("Failed to write connection ID: {}", e))?;
        
        log::info!("Created new connection ID: {}", display_id);
        
        Ok(connection_id)
    }
    
    /// Load existing connection ID
    fn load_from_file(id_file: &PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(id_file)
            .map_err(|e| format!("Failed to read connection ID: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse connection ID: {}", e))
    }
    
    /// Parse a connection ID from user input (handles various formats)
    pub fn parse(input: &str) -> Option<String> {
        // Remove spaces, dashes, and other separators
        let cleaned: String = input
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        
        // Must be exactly 9 digits
        if cleaned.len() == 9 {
            Some(cleaned)
        } else {
            None
        }
    }
    
    /// Format any 9-digit ID for display
    pub fn format_display(id: &str) -> String {
        if id.len() == 9 {
            format!("{}-{}-{}", &id[0..3], &id[3..6], &id[6..9])
        } else {
            id.to_string()
        }
    }
}

/// Global connection ID accessor
static CONNECTION_ID: std::sync::OnceLock<ConnectionId> = std::sync::OnceLock::new();

/// Get the global connection ID
pub fn get_connection_id() -> &'static ConnectionId {
    CONNECTION_ID.get_or_init(|| {
        ConnectionId::get_or_create()
            .unwrap_or_else(|e| {
                log::error!("Failed to get connection ID: {}", e);
                // Fallback to random ID
                let mut rng = rand::thread_rng();
                let digits: String = (0..9)
                    .map(|_| rng.gen_range(0..10).to_string())
                    .collect();
                ConnectionId {
                    id: digits.clone(),
                    display_id: ConnectionId::format_display(&digits),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    device_name: "Unknown".to_string(),
                }
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_connection_id() {
        assert_eq!(ConnectionId::parse("123-456-789"), Some("123456789".to_string()));
        assert_eq!(ConnectionId::parse("123 456 789"), Some("123456789".to_string()));
        assert_eq!(ConnectionId::parse("123456789"), Some("123456789".to_string()));
        assert_eq!(ConnectionId::parse("12345"), None); // Too short
        assert_eq!(ConnectionId::parse("1234567890"), None); // Too long
    }
    
    #[test]
    fn test_format_display() {
        assert_eq!(ConnectionId::format_display("123456789"), "123-456-789");
    }
}
