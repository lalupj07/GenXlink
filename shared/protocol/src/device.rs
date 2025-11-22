use serde::{Deserialize, Serialize};
use crate::DeviceId;

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: DeviceId,
    pub device_name: String,
    pub platform: Platform,
    pub version: String,
    pub capabilities: DeviceCapabilities,
}

/// Platform types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    Android,
    Linux,
    MacOS,
    #[allow(non_camel_case_types)]
    iOS,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub screen_capture: bool,
    pub remote_control: bool,
    pub file_transfer: bool,
    pub clipboard_sync: bool,
    pub audio_streaming: bool,
    pub multi_monitor: bool,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            screen_capture: true,
            remote_control: true,
            file_transfer: false,
            clipboard_sync: true,
            audio_streaming: false,
            multi_monitor: false,
        }
    }
}

/// Generate a unique device ID based on hardware
pub fn generate_device_id() -> DeviceId {
    #[cfg(target_os = "windows")]
    {
        generate_windows_device_id()
    }
    
    #[cfg(target_os = "android")]
    {
        generate_android_device_id()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        // Fallback for other platforms
        use uuid::Uuid;
        DeviceId::from_string(Uuid::new_v4().to_string())
    }
}

#[cfg(target_os = "windows")]
fn generate_windows_device_id() -> DeviceId {
    use std::process::Command;
    
    // Try to get machine GUID from registry
    let output = Command::new("reg")
        .args(&["query", "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Cryptography", "/v", "MachineGuid"])
        .output();
    
    if let Ok(output) = output {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // Parse the registry output
            for line in stdout.lines() {
                if line.contains("MachineGuid") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(guid) = parts.last() {
                        return DeviceId::from_string(guid.to_string());
                    }
                }
            }
        }
    }
    
    // Fallback to UUID
    use uuid::Uuid;
    DeviceId::from_string(Uuid::new_v4().to_string())
}

#[cfg(target_os = "android")]
fn generate_android_device_id() -> DeviceId {
    // This will be implemented in the Android-specific code
    // using Android's ANDROID_ID or device fingerprinting
    use uuid::Uuid;
    DeviceId::from_string(Uuid::new_v4().to_string())
}
