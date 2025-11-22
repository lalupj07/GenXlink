use crate::ClientError;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Session history manager
pub struct SessionHistoryManager {
    sessions: Vec<SessionRecord>,
    max_records: usize,
}

impl SessionHistoryManager {
    /// Create a new session history manager
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            max_records: 100,
        }
    }

    /// Add a new session record
    pub fn add_session(&mut self, device_name: String, device_id: String) -> String {
        let session_id = format!("session_{}", Self::current_timestamp());
        
        let record = SessionRecord {
            session_id: session_id.clone(),
            device_name,
            device_id,
            start_time: Self::current_timestamp(),
            end_time: None,
            duration_seconds: 0,
            bytes_transferred: 0,
            status: SessionStatus::Active,
        };
        
        self.sessions.push(record);
        
        // Limit records
        if self.sessions.len() > self.max_records {
            self.sessions.remove(0);
        }
        
        session_id
    }

    /// End a session
    pub fn end_session(&mut self, session_id: &str, status: SessionStatus) {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.session_id == session_id) {
            let end_time = Self::current_timestamp();
            session.end_time = Some(end_time);
            session.duration_seconds = end_time.saturating_sub(session.start_time);
            session.status = status;
        }
    }

    /// Update bytes transferred
    pub fn update_bytes(&mut self, session_id: &str, bytes: u64) {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.session_id == session_id) {
            session.bytes_transferred = bytes;
        }
    }

    /// Get all sessions
    pub fn get_sessions(&self) -> &[SessionRecord] {
        &self.sessions
    }

    /// Get active sessions
    pub fn get_active_sessions(&self) -> Vec<&SessionRecord> {
        self.sessions.iter()
            .filter(|s| s.status == SessionStatus::Active)
            .collect()
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&SessionRecord> {
        self.sessions.iter().find(|s| s.session_id == session_id)
    }

    /// Get total session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get total duration
    pub fn total_duration(&self) -> u64 {
        self.sessions.iter().map(|s| s.duration_seconds).sum()
    }

    /// Get total bytes transferred
    pub fn total_bytes(&self) -> u64 {
        self.sessions.iter().map(|s| s.bytes_transferred).sum()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.sessions.clear();
    }

    /// Set max records
    pub fn set_max_records(&mut self, max: usize) {
        self.max_records = max;
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for SessionHistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub device_name: String,
    pub device_id: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_seconds: u64,
    pub bytes_transferred: u64,
    pub status: SessionStatus,
}

impl SessionRecord {
    /// Format duration as human-readable string
    pub fn format_duration(&self) -> String {
        let seconds = self.duration_seconds;
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else {
            format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
        }
    }

    /// Format bytes as human-readable string
    pub fn format_bytes(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.bytes_transferred as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Disconnected,
    Failed,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Completed => write!(f, "Completed"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_history() {
        let mut manager = SessionHistoryManager::new();
        
        let session_id = manager.add_session("Test Device".to_string(), "device123".to_string());
        assert_eq!(manager.session_count(), 1);
        
        manager.update_bytes(&session_id, 1024 * 1024);
        manager.end_session(&session_id, SessionStatus::Completed);
        
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.bytes_transferred, 1024 * 1024);
        assert_eq!(session.status, SessionStatus::Completed);
    }

    #[test]
    fn test_session_formatting() {
        let record = SessionRecord {
            session_id: "test".to_string(),
            device_name: "Test".to_string(),
            device_id: "123".to_string(),
            start_time: 0,
            end_time: Some(3665),
            duration_seconds: 3665,
            bytes_transferred: 1024 * 1024,
            status: SessionStatus::Completed,
        };

        assert_eq!(record.format_duration(), "1h 1m");
        assert_eq!(record.format_bytes(), "1.00 MB");
    }

    #[test]
    fn test_max_records() {
        let mut manager = SessionHistoryManager::new();
        manager.set_max_records(5);

        for i in 0..10 {
            manager.add_session(format!("Device {}", i), format!("id{}", i));
        }

        assert_eq!(manager.session_count(), 5);
    }
}
