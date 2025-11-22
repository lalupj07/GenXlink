use crate::ClientError;
use serde::{Serialize, Deserialize};

/// Multi-monitor manager
pub struct MultiMonitorManager {
    monitors: Vec<MonitorInfo>,
    active_monitor: Option<usize>,
}

impl MultiMonitorManager {
    /// Create a new multi-monitor manager
    pub fn new() -> Self {
        Self {
            monitors: Vec::new(),
            active_monitor: None,
        }
    }

    /// Detect available monitors
    pub fn detect_monitors(&mut self) -> Result<(), ClientError> {
        self.monitors.clear();
        
        #[cfg(target_os = "windows")]
        {
            self.detect_windows_monitors()?;
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Fallback: single monitor
            self.monitors.push(MonitorInfo {
                id: 0,
                name: "Primary Monitor".to_string(),
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                is_primary: true,
            });
        }

        if !self.monitors.is_empty() {
            self.active_monitor = Some(0);
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_monitors(&mut self) -> Result<(), ClientError> {
        // For now, add a default monitor
        // Full implementation would use EnumDisplayMonitors
        self.monitors.push(MonitorInfo {
            id: 0,
            name: "Primary Monitor".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        });

        Ok(())
    }

    /// Get all monitors
    pub fn get_monitors(&self) -> &[MonitorInfo] {
        &self.monitors
    }

    /// Get active monitor
    pub fn get_active_monitor(&self) -> Option<&MonitorInfo> {
        self.active_monitor.and_then(|idx| self.monitors.get(idx))
    }

    /// Set active monitor
    pub fn set_active_monitor(&mut self, index: usize) -> Result<(), ClientError> {
        if index < self.monitors.len() {
            self.active_monitor = Some(index);
            Ok(())
        } else {
            Err(ClientError::InvalidInput("Invalid monitor index".to_string()))
        }
    }

    /// Get monitor count
    pub fn monitor_count(&self) -> usize {
        self.monitors.len()
    }

    /// Get primary monitor
    pub fn get_primary_monitor(&self) -> Option<&MonitorInfo> {
        self.monitors.iter().find(|m| m.is_primary)
    }
}

impl Default for MultiMonitorManager {
    fn default() -> Self {
        let mut manager = Self::new();
        let _ = manager.detect_monitors();
        manager
    }
}

/// Monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

impl MonitorInfo {
    /// Get monitor resolution as string
    pub fn resolution_string(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// Get monitor position as string
    pub fn position_string(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_manager() {
        let manager = MultiMonitorManager::default();
        assert!(manager.monitor_count() > 0);
        assert!(manager.get_primary_monitor().is_some());
    }

    #[test]
    fn test_monitor_info() {
        let monitor = MonitorInfo {
            id: 0,
            name: "Test Monitor".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        };

        assert_eq!(monitor.resolution_string(), "1920x1080");
        assert_eq!(monitor.position_string(), "(0, 0)");
    }
}
