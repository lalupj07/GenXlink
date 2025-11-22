use serde::{Deserialize, Serialize};
use crate::{DeviceId, SessionId};

/// Main message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub session_id: SessionId,
    pub sequence: u64,
    pub payload: MessagePayload,
}

/// Message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    // Connection messages
    ConnectionRequest(ConnectionRequest),
    ConnectionResponse(ConnectionResponse),
    Disconnect(DisconnectReason),
    
    // Screen streaming
    VideoFrame(VideoFrame),
    VideoConfig(VideoConfig),
    
    // Input events
    KeyboardEvent(KeyboardEvent),
    MouseEvent(MouseEvent),
    ClipboardSync(ClipboardData),
    
    // Control messages
    Ping,
    Pong,
    QualityReport(QualityReport),
    
    // File transfer
    FileTransferRequest(FileTransferRequest),
    FileTransferAccept(FileTransferAccept),
    FileTransferReject(FileTransferReject),
    FileChunk(FileChunk),
    FileTransferComplete(FileTransferComplete),
    FileTransferCancel(FileTransferCancel),
}

/// Connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub device_id: DeviceId,
    pub password: String,
    pub protocol_version: u32,
    pub capabilities: Vec<String>,
}

/// Connection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResponse {
    pub accepted: bool,
    pub reason: Option<String>,
    pub server_capabilities: Vec<String>,
}

/// Disconnect reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectReason {
    UserInitiated,
    Timeout,
    Error(String),
    LicenseExpired,
    SessionLimitReached,
}

/// Video frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    pub timestamp: u64,
    pub frame_type: FrameType,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameType {
    KeyFrame,
    DeltaFrame,
}

/// Video configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
}

/// Keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    pub key_code: u32,
    pub scan_code: u32,
    pub pressed: bool,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Mouse event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub event_type: MouseEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseEventType {
    Move,
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
    MiddleDown,
    MiddleUp,
    Wheel { delta: i32 },
}

/// Clipboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub content_type: String,
    pub data: Vec<u8>,
}

/// Quality report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub latency_ms: u32,
    pub packet_loss: f32,
    pub fps: u32,
    pub bandwidth_kbps: u32,
}

/// File transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferRequest {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
}

/// File transfer accept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferAccept {
    pub file_id: String,
}

/// File transfer reject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferReject {
    pub file_id: String,
    pub reason: String,
}

/// File chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub file_id: String,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub data: Vec<u8>,
}

/// File transfer complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferComplete {
    pub file_id: String,
}

/// File transfer cancel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferCancel {
    pub file_id: String,
    pub reason: String,
}
