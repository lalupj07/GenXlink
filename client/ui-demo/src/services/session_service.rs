// Copyright (c) 2025 GenXis Innovations
// Session Service - Manages remote desktop sessions

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Session type
#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    ScreenShare,
    RemoteControl,
    FileTransfer,
    AudioOnly,
}

/// Session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Initializing,
    Active,
    Paused,
    Ended,
    Error(String),
}

/// Session quality
#[derive(Debug, Clone)]
pub struct SessionQuality {
    pub resolution: String,
    pub fps: u32,
    pub bitrate_kbps: u32,
    pub latency_ms: u32,
    pub packet_loss_percent: f32,
}

/// Remote session info
#[derive(Debug, Clone)]
pub struct RemoteSession {
    pub id: String,
    pub peer_id: String,
    pub peer_name: String,
    pub session_type: SessionType,
    pub state: SessionState,
    pub started_at: Instant,
    pub quality: SessionQuality,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_host: bool,
}

impl RemoteSession {
    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn duration_string(&self) -> String {
        let secs = self.duration().as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }

    pub fn bandwidth_mbps(&self) -> f32 {
        let total_bytes = self.bytes_sent + self.bytes_received;
        let duration_secs = self.duration().as_secs_f32();
        if duration_secs > 0.0 {
            (total_bytes as f32 / duration_secs) / 125_000.0 // Convert to Mbps
        } else {
            0.0
        }
    }
}

/// Session Service
pub struct SessionService {
    sessions: HashMap<String, RemoteSession>,
    session_counter: u64,
    max_concurrent_sessions: usize,
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_counter: 0,
            max_concurrent_sessions: 10,
        }
    }

    /// Create a new session
    pub fn create_session(
        &mut self,
        peer_id: &str,
        peer_name: &str,
        session_type: SessionType,
        is_host: bool,
    ) -> Result<String, String> {
        // Check concurrent session limit
        let active_count = self.sessions.values()
            .filter(|s| s.state == SessionState::Active)
            .count();

        if active_count >= self.max_concurrent_sessions {
            return Err("Maximum concurrent sessions reached".to_string());
        }

        self.session_counter += 1;
        let session_id = format!("session-{}", self.session_counter);

        let session = RemoteSession {
            id: session_id.clone(),
            peer_id: peer_id.to_string(),
            peer_name: peer_name.to_string(),
            session_type,
            state: SessionState::Initializing,
            started_at: Instant::now(),
            quality: SessionQuality {
                resolution: "1920x1080".to_string(),
                fps: 60,
                bitrate_kbps: 5000,
                latency_ms: 20,
                packet_loss_percent: 0.0,
            },
            bytes_sent: 0,
            bytes_received: 0,
            is_host,
        };

        println!("🎬 Creating session: {} with peer {} ({})", 
            session_id, peer_name, 
            if is_host { "hosting" } else { "viewing" });

        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Start a session (transition from Initializing to Active)
    pub fn start_session(&mut self, session_id: &str) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        if session.state != SessionState::Initializing {
            return Err("Session is not in initializing state".to_string());
        }

        session.state = SessionState::Active;
        println!("▶️ Session started: {}", session_id);
        Ok(())
    }

    /// Pause a session
    pub fn pause_session(&mut self, session_id: &str) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        if session.state == SessionState::Active {
            session.state = SessionState::Paused;
            println!("⏸️ Session paused: {}", session_id);
        }
        Ok(())
    }

    /// Resume a session
    pub fn resume_session(&mut self, session_id: &str) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        if session.state == SessionState::Paused {
            session.state = SessionState::Active;
            println!("▶️ Session resumed: {}", session_id);
        }
        Ok(())
    }

    /// End a session
    pub fn end_session(&mut self, session_id: &str) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        session.state = SessionState::Ended;
        println!("🛑 Session ended: {} (duration: {})", 
            session_id, session.duration_string());
        Ok(())
    }

    /// Update session quality
    pub fn update_quality(&mut self, session_id: &str, quality: SessionQuality) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        session.quality = quality;
        Ok(())
    }

    /// Update session bandwidth stats
    pub fn update_bandwidth(&mut self, session_id: &str, bytes_sent: u64, bytes_received: u64) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.bytes_sent += bytes_sent;
            session.bytes_received += bytes_received;
        }
    }

    /// Get all sessions
    pub fn get_sessions(&self) -> Vec<&RemoteSession> {
        self.sessions.values().collect()
    }

    /// Get active sessions
    pub fn get_active_sessions(&self) -> Vec<&RemoteSession> {
        self.sessions.values()
            .filter(|s| s.state == SessionState::Active)
            .collect()
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&RemoteSession> {
        self.sessions.get(session_id)
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.values()
            .filter(|s| s.state == SessionState::Active)
            .count()
    }

    /// Cleanup ended sessions
    pub fn cleanup_sessions(&mut self) {
        self.sessions.retain(|_, s| s.state != SessionState::Ended);
    }

    /// Set maximum concurrent sessions
    pub fn set_max_sessions(&mut self, max: usize) {
        self.max_concurrent_sessions = max;
    }
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new()
    }
}
