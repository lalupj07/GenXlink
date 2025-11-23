pub mod capture;
pub mod encoder;
pub mod input;
pub mod remote_control;
pub mod control_channel;
pub mod file_transfer;
pub mod session_password;
pub mod multi_monitor;
pub mod clipboard;
pub mod session_history;
pub mod chat;
pub mod hardware_encoder;
pub mod adaptive_bitrate;
pub mod permission_profiles;
pub mod audio_streaming;
pub mod localization;
pub mod theme;
pub mod zero_setup;
pub mod gst_tunnel;
pub mod lan_discovery;
pub mod transport;
pub mod streaming;
pub mod pipeline;
pub mod performance;
pub mod performance_optimizer;
pub mod webrtc;
pub mod signaling_client;
pub mod screen_capture;
pub mod video_encoder;
pub mod screen_streamer;
pub mod webrtc_session;
pub mod input_injection;
pub mod remote_control_manager;

pub use capture::*;
pub use encoder::*;
pub use input::*;
pub use clipboard::*;
pub use transport::*;
pub use performance::*;
pub use webrtc::*;
pub use signaling_client::*;

use thiserror::Error;

/// Core client errors
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Screen capture error: {0}")]
    CaptureError(String),
    
    #[error("Encoding error: {0}")]
    EncodingError(String),
    
    #[error("Input injection error: {0}")]
    InputError(String),
    
    #[error("Transport error: {0}")]
    TransportError(String),
    
    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    
    #[error("WebRTC error: {0}")]
    WebRTCError(String),
    
    #[error("Streaming error: {0}")]
    StreamingError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Not supported on this platform")]
    PlatformNotSupported,
}
