// Copyright (c) 2025 GenXis Innovations
// Licensed under the Apache License, Version 2.0
// Contact: genxisinnovation@outlook.com

pub mod config;
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
pub mod audio_capture;
pub mod audio_playback;
pub mod audio_stream_manager;
pub mod security;
pub mod webrtc_security;
pub mod file_transfer_enhanced;
pub mod large_file_transfer;
pub mod access_control;
pub mod role_based_access;
pub mod database;
pub mod auth_service;
pub mod device_registry;
pub mod webrtc_integration;
pub mod session_manager;
pub mod p2p_discovery;
pub mod p2p_signaling;
pub mod p2p_integration;
pub mod installation_id;
pub mod connection_id;
pub mod connection_manager;

pub use capture::{Frame, ScreenCapture, DxgiCapture};
pub use encoder::{VideoEncoder, EncodedFrame, EncoderConfig};
pub use streaming::{StreamingPipeline, StreamingStats, Frame as StreamingFrame};
pub use input::{InputInjector};
pub use clipboard::{ClipboardManager};

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
