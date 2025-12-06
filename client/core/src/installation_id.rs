//! Installation ID Generator
//! 
//! Creates and persists a unique identifier for each installation.
//! This ID is generated once on first run and stored locally.

use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique installation identifier
#[derive(Debug, Clone)]
pub struct InstallationId {
    /// The unique ID for this installation
    pub id: String,
    /// Short display ID (first 8 characters)
    pub short_id: String,
    /// When the installation was first created
    pub created_at: String,
    /// Installation type
    pub install_type: InstallationType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstallationType {
    /// First time installation
    Fresh,
    /// Existing installation (ID was loaded)
    Existing,
}

impl InstallationId {
    /// Get or create the installation ID
    /// Creates a new ID on first run, loads existing ID on subsequent runs
    pub fn get_or_create() -> Result<Self, String> {
        let config_dir = Self::get_config_dir()?;
        let id_file = config_dir.join("installation_id.json");
        
        if id_file.exists() {
            // Load existing ID
            Self::load_from_file(&id_file)
        } else {
            // Create new ID
            Self::create_new(&id_file)
        }
    }
    
    /// Force create a new installation ID (for reset purposes)
    pub fn create_fresh() -> Result<Self, String> {
        let config_dir = Self::get_config_dir()?;
        let id_file = config_dir.join("installation_id.json");
        Self::create_new(&id_file)
    }
    
    /// Get the config directory path
    fn get_config_dir() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("GenXLink");
        
        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        Ok(config_dir)
    }
    
    /// Create a new installation ID
    fn create_new(id_file: &PathBuf) -> Result<Self, String> {
        let id = Uuid::new_v4().to_string();
        let short_id = id[..8].to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        
        let installation = Self {
            id: id.clone(),
            short_id: short_id.clone(),
            created_at: created_at.clone(),
            install_type: InstallationType::Fresh,
        };
        
        // Save to file
        let json = serde_json::json!({
            "id": id,
            "short_id": short_id,
            "created_at": created_at,
            "version": env!("CARGO_PKG_VERSION"),
            "machine_id": Self::get_machine_fingerprint(),
        });
        
        let json_str = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        fs::write(id_file, json_str)
            .map_err(|e| format!("Failed to write installation ID: {}", e))?;
        
        log::info!("Created new installation ID: {}", short_id);
        
        Ok(installation)
    }
    
    /// Load existing installation ID from file
    fn load_from_file(id_file: &PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(id_file)
            .map_err(|e| format!("Failed to read installation ID: {}", e))?;
        
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse installation ID: {}", e))?;
        
        let id = json["id"].as_str()
            .ok_or("Missing id field")?
            .to_string();
        
        let short_id = json["short_id"].as_str()
            .unwrap_or(&id[..8])
            .to_string();
        
        let created_at = json["created_at"].as_str()
            .unwrap_or("unknown")
            .to_string();
        
        log::info!("Loaded existing installation ID: {}", short_id);
        
        Ok(Self {
            id,
            short_id,
            created_at,
            install_type: InstallationType::Existing,
        })
    }
    
    /// Generate a machine fingerprint (for additional identification)
    fn get_machine_fingerprint() -> String {
        // Combine various system identifiers
        let mut fingerprint_parts = Vec::new();
        
        // Computer name
        if let Ok(name) = std::env::var("COMPUTERNAME") {
            fingerprint_parts.push(name);
        }
        
        // Username
        if let Ok(user) = std::env::var("USERNAME") {
            fingerprint_parts.push(user);
        }
        
        // OS info
        #[cfg(target_os = "windows")]
        fingerprint_parts.push("windows".to_string());
        
        #[cfg(target_os = "macos")]
        fingerprint_parts.push("macos".to_string());
        
        #[cfg(target_os = "linux")]
        fingerprint_parts.push("linux".to_string());
        
        // Create hash of fingerprint
        let combined = fingerprint_parts.join("-");
        format!("{:x}", md5::compute(combined.as_bytes()))
    }
    
    /// Get the display string for UI
    pub fn display_id(&self) -> String {
        format!("GXL-{}", self.short_id.to_uppercase())
    }
    
    /// Check if this is a fresh installation
    pub fn is_fresh(&self) -> bool {
        self.install_type == InstallationType::Fresh
    }
}

/// Global installation ID accessor
static INSTALLATION_ID: std::sync::OnceLock<InstallationId> = std::sync::OnceLock::new();

/// Get the global installation ID
pub fn get_installation_id() -> &'static InstallationId {
    INSTALLATION_ID.get_or_init(|| {
        InstallationId::get_or_create()
            .unwrap_or_else(|e| {
                log::error!("Failed to get installation ID: {}", e);
                // Fallback to a temporary ID
                InstallationId {
                    id: Uuid::new_v4().to_string(),
                    short_id: "TEMP".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    install_type: InstallationType::Fresh,
                }
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_installation_id_creation() {
        let id = InstallationId::get_or_create();
        assert!(id.is_ok());
        
        let id = id.unwrap();
        assert!(!id.id.is_empty());
        assert_eq!(id.short_id.len(), 8);
        assert!(id.display_id().starts_with("GXL-"));
    }
    
    #[test]
    fn test_display_id_format() {
        let id = InstallationId {
            id: "12345678-1234-1234-1234-123456789abc".to_string(),
            short_id: "12345678".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            install_type: InstallationType::Fresh,
        };
        
        assert_eq!(id.display_id(), "GXL-12345678");
    }
}
