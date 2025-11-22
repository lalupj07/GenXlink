use anyhow::Result;
use genxlink_protocol::DeviceId;
use crate::config::Config;
use crate::license_manager::LicenseManager;

/// Main application
pub struct App {
    device_id: DeviceId,
    config: Config,
    license_manager: LicenseManager,
}

impl App {
    pub fn new(device_id: DeviceId, config: Config) -> Result<Self> {
        let license_manager = LicenseManager::new(device_id.clone())?;
        
        Ok(Self {
            device_id,
            config,
            license_manager,
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        println!("=== GenXLink Remote Desktop ===");
        println!("Device ID: {}", self.device_id);
        println!();
        
        // Check license
        match self.license_manager.get_license() {
            Some(license) => {
                println!("License: {:?}", license.plan);
                if let Some(limit) = license.session_time_limit() {
                    println!("Session limit: {} minutes", limit);
                }
            }
            None => {
                println!("No license activated. Running in Free mode.");
                println!("Session limit: 10 minutes");
            }
        }
        
        println!();
        println!("Commands:");
        println!("  1. Connect to remote device");
        println!("  2. Wait for incoming connection");
        println!("  3. Activate license");
        println!("  4. Exit");
        println!();
        
        // TODO: Implement command loop
        // For now, just exit
        
        Ok(())
    }
}
