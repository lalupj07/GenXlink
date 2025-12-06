use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tokio::time::sleep;
use tracing::{info, warn, error, debug};
use reqwest;

/// Auto-update system for GenXLink
pub struct AutoUpdater {
    config: UpdateConfig,
    current_version: Version,
    update_channel: UpdateChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Check for updates at startup
    pub check_at_startup: bool,
    /// Check for updates periodically (hours)
    pub check_interval_hours: u64,
    /// Auto-download updates
    pub auto_download: bool,
    /// Auto-install updates
    pub auto_install: bool,
    /// Update channel to use
    pub default_channel: UpdateChannel,
    /// Base URL for update server
    pub update_server_url: String,
    /// Timeout for update checks (seconds)
    pub check_timeout_seconds: u64,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_at_startup: true,
            check_interval_hours: 24,
            auto_download: false,
            auto_install: false,
            default_channel: UpdateChannel::Stable,
            update_server_url: "https://updates.genxlink.com".to_string(),
            check_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateChannel {
    Stable,
    Beta,
    Alpha,
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Beta => "beta",
            UpdateChannel::Alpha => "alpha",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }
    
    pub fn with_pre_release(major: u32, minor: u32, patch: u32, pre_release: String) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release),
        }
    }
    
    pub fn current() -> Self {
        // This would normally be set at compile time
        env!("CARGO_PKG_VERSION").parse().unwrap_or_else(|_| {
            Self::new(1, 0, 0)
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Version {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        let version_part = parts[0];
        
        let version_numbers: Vec<&str> = version_part.split('.').collect();
        if version_numbers.len() != 3 {
            return Err(anyhow!("Invalid version format"));
        }
        
        let major = version_numbers[0].parse()?;
        let minor = version_numbers[1].parse()?;
        let patch = version_numbers[2].parse()?;
        
        let pre_release = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };
        
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: Version,
    pub release_date: String,
    pub changelog: String,
    pub download_url: String,
    pub signature_url: String,
    pub size_bytes: u64,
    pub mandatory: bool,
    pub min_supported_version: Option<Version>,
}

#[derive(Debug)]
pub enum UpdateResult {
    UpToDate,
    UpdateAvailable(UpdateInfo),
    UpdateDownloaded(PathBuf),
    UpdateReady(PathBuf),
    Error(String),
}

impl AutoUpdater {
    pub fn new(config: UpdateConfig) -> Result<Self> {
        let current_version = Version::current();
        let update_channel = config.default_channel.clone();
        
        info!("AutoUpdater initialized with version {} on {} channel", 
              current_version, update_channel.as_str());
        
        Ok(Self {
            config,
            current_version,
            update_channel,
        })
    }
    
    /// Check for available updates
    pub async fn check_for_updates(&self) -> Result<UpdateResult> {
        info!("Checking for updates on {} channel", self.update_channel.as_str());
        
        let update_info = self.fetch_update_info().await?;
        
        if update_info.version > self.current_version {
            info!("Update available: {} -> {}", 
                  self.current_version, update_info.version);
            Ok(UpdateResult::UpdateAvailable(update_info))
        } else {
            info!("Application is up to date: {}", self.current_version);
            Ok(UpdateResult::UpToDate)
        }
    }
    
    /// Download an update
    pub async fn download_update(&self, update_info: &UpdateInfo) -> Result<PathBuf> {
        info!("Downloading update {} ({} bytes)", 
              update_info.version, update_info.size_bytes);
        
        let download_dir = self.get_download_directory()?;
        let filename = format!("genxlink-{}.exe", update_info.version);
        let download_path = download_dir.join(filename);
        
        // Download the update file
        let response = reqwest::get(&update_info.download_url).await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download update: {}", response.status()));
        }
        
        let mut file = tokio::fs::File::create(&download_path).await?;
        let mut content = response.bytes_stream();
        
        use futures_util::StreamExt;
        while let Some(chunk) = content.next().await {
            let chunk = chunk?;
            use tokio::io::AsyncWriteExt;
            file.write_all(&chunk).await?;
        }
        
        // Verify the download
        self.verify_download(&download_path, update_info).await?;
        
        info!("Update downloaded successfully to: {:?}", download_path);
        Ok(download_path)
    }
    
    /// Install an update
    pub async fn install_update(&self, update_path: &PathBuf) -> Result<()> {
        info!("Installing update from: {:?}", update_path);
        
        // On Windows, we need to spawn a separate process to replace the running executable
        #[cfg(target_os = "windows")]
        {
            self.install_update_windows(update_path).await?;
        }
        
        #[cfg(target_os = "macos")]
        {
            self.install_update_macos(update_path).await?;
        }
        
        #[cfg(target_os = "linux")]
        {
            self.install_update_linux(update_path).await?;
        }
        
        info!("Update installation initiated");
        Ok(())
    }
    
    /// Start the background update checker
    pub async fn start_background_checker(&self) {
        if !self.config.check_at_startup {
            return;
        }
        
        let check_interval = Duration::from_secs(self.config.check_interval_hours * 3600);
        let config = self.config.clone();
        let current_version = self.current_version.clone();
        let update_channel = self.update_channel.clone();
        
        tokio::spawn(async move {
            loop {
                debug!("Running background update check");
                
                match Self::new(config.clone()).and_then(|updater| {
                    Box::pin(updater.check_for_updates())
                }).await {
                    Ok(UpdateResult::UpdateAvailable(update_info)) => {
                        info!("Background update check found new version: {}", update_info.version);
                        
                        if config.auto_download {
                            info!("Auto-downloading update");
                            match Self::new(config.clone()).and_then(|updater| {
                                Box::pin(updater.download_update(&update_info))
                            }).await {
                                Ok(download_path) => {
                                    if config.auto_install {
                                        info!("Auto-installing update");
                                        let _ = Self::new(config.clone()).and_then(|updater| {
                                            Box::pin(updater.install_update(&download_path))
                                        }).await;
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to auto-download update: {}", e);
                                }
                            }
                        }
                    }
                    Ok(UpdateResult::UpToDate) => {
                        debug!("Background check: up to date");
                    }
                    Err(e) => {
                        warn!("Background update check failed: {}", e);
                    }
                    _ => {}
                }
                
                sleep(check_interval).await;
            }
        });
    }
    
    async fn fetch_update_info(&self) -> Result<UpdateInfo> {
        let url = format!("{}/api/v1/updates/check/{}", 
                         self.config.update_server_url, 
                         self.update_channel.as_str());
        
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", format!("GenXLink/{}", self.current_version))
            .header("X-Current-Version", self.current_version.to_string())
            .timeout(Duration::from_secs(self.config.check_timeout_seconds))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Update check failed: {}", response.status()));
        }
        
        let update_info: UpdateInfo = response.json().await?;
        Ok(update_info)
    }
    
    async fn verify_download(&self, download_path: &PathBuf, update_info: &UpdateInfo) -> Result<()> {
        // Verify file size
        let metadata = tokio::fs::metadata(download_path).await?;
        if metadata.len() != update_info.size_bytes {
            return Err(anyhow!("Downloaded file size mismatch"));
        }
        
        // TODO: Verify digital signature
        info!("Download verification passed");
        Ok(())
    }
    
    fn get_download_directory(&self) -> Result<PathBuf> {
        let mut path = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not find cache directory"))?;
        path.push("GenXLink");
        path.push("updates");
        
        tokio::fs::create_dir_all(&path).await?;
        Ok(path)
    }
    
    #[cfg(target_os = "windows")]
    async fn install_update_windows(&self, update_path: &PathBuf) -> Result<()> {
        use std::process::Command;
        
        // Create a batch script to handle the update
        let script_content = format!(
            r#"
@echo off
echo Installing GenXLink update...
timeout /t 3 /nobreak > nul
copy "{}" "{}" /Y
start "" "{}"
del "{}"
"#,
            update_path.display(),
            std::env::current_exe()?.display(),
            std::env::current_exe()?.display(),
            update_path.display()
        );
        
        let script_path = update_path.with_extension("bat");
        tokio::fs::write(&script_path, script_content).await?;
        
        // Start the update process and exit current application
        Command::new("cmd")
            .args(&["/C", &script_path.to_string_lossy()])
            .spawn()?;
        
        // Exit the current application
        std::process::exit(0);
    }
    
    #[cfg(target_os = "macos")]
    async fn install_update_macos(&self, update_path: &PathBuf) -> Result<()> {
        use std::process::Command;
        
        // Create update script
        let script_content = format!(
            r#"
#!/bin/bash
echo "Installing GenXLink update..."
sleep 3
cp "{}" "{}"
open "{}"
rm "{}"
"#,
            update_path.display(),
            std::env::current_exe()?.display(),
            std::env::current_exe()?.display(),
            update_path.display()
        );
        
        let script_path = update_path.with_extension("sh");
        tokio::fs::write(&script_path, script_content).await?;
        
        // Make script executable and run it
        Command::new("chmod")
            .args(&["+x", &script_path.to_string_lossy()])
            .spawn()?;
        
        Command::new("sh")
            .arg(&script_path.to_string_lossy())
            .spawn()?;
        
        std::process::exit(0);
    }
    
    #[cfg(target_os = "linux")]
    async fn install_update_linux(&self, update_path: &PathBuf) -> Result<()> {
        use std::process::Command;
        
        // Create update script
        let script_content = format!(
            r#"
#!/bin/bash
echo "Installing GenXLink update..."
sleep 3
cp "{}" "{}"
"{}" &
rm "{}"
"#,
            update_path.display(),
            std::env::current_exe()?.display(),
            std::env::current_exe()?.display(),
            update_path.display()
        );
        
        let script_path = update_path.with_extension("sh");
        tokio::fs::write(&script_path, script_content).await?;
        
        // Make script executable and run it
        Command::new("chmod")
            .args(&["+x", &script_path.to_string_lossy()])
            .spawn()?;
        
        Command::new("sh")
            .arg(&script_path.to_string_lossy())
            .spawn()?;
        
        std::process::exit(0);
    }
}

/// Update manager that handles the complete update lifecycle
pub struct UpdateManager {
    updater: AutoUpdater,
    last_check: Option<SystemTime>,
}

impl UpdateManager {
    pub fn new(config: UpdateConfig) -> Result<Self> {
        let updater = AutoUpdater::new(config)?;
        Ok(Self {
            updater,
            last_check: None,
        })
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        // Load last check time from storage
        self.last_check = self.load_last_check_time();
        
        // Start background checker
        self.updater.start_background_checker().await;
        
        // Check for updates at startup if enabled
        if self.updater.config.check_at_startup {
            self.check_for_updates_if_needed().await?;
        }
        
        Ok(())
    }
    
    pub async fn check_for_updates_now(&self) -> Result<UpdateResult> {
        self.updater.check_for_updates().await
    }
    
    pub async fn install_available_update(&self, update_info: &UpdateInfo) -> Result<()> {
        let download_path = self.updater.download_update(update_info).await?;
        self.updater.install_update(&download_path).await
    }
    
    async fn check_for_updates_if_needed(&mut self) -> Result<()> {
        let now = SystemTime::now();
        let check_interval = Duration::from_secs(self.updater.config.check_interval_hours * 3600);
        
        if let Some(last_check) = self.last_check {
            if now.duration_since(last_check)? < check_interval {
                debug!("Update check not needed yet");
                return Ok(());
            }
        }
        
        debug!("Performing scheduled update check");
        let result = self.updater.check_for_updates().await?;
        self.last_check = Some(now);
        self.save_last_check_time(now);
        
        Ok(result)
    }
    
    fn load_last_check_time(&self) -> Option<SystemTime> {
        // TODO: Load from persistent storage
        None
    }
    
    fn save_last_check_time(&self, time: SystemTime) {
        // TODO: Save to persistent storage
        debug!("Saved last check time: {:?}", time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let v1: Version = "1.2.3".parse().unwrap();
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 2);
        assert_eq!(v1.patch, 3);
        
        let v2: Version = "1.2.3-beta".parse().unwrap();
        assert_eq!(v2.pre_release, Some("beta".to_string()));
    }
    
    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 0, 1);
        let v3 = Version::new(1, 1, 0);
        let v4 = Version::new(2, 0, 0);
        
        assert!(v2 > v1);
        assert!(v3 > v2);
        assert!(v4 > v3);
    }
    
    #[tokio::test]
    async fn test_updater_initialization() {
        let config = UpdateConfig::default();
        let updater = AutoUpdater::new(config).unwrap();
        assert!(updater.current_version >= Version::new(1, 0, 0));
    }
}
