use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs::File;
use std::io::Write;

/// Video encoder configuration
#[derive(Debug, Clone)]
pub struct VideoEncoderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
}

/// Simple video encoder for recording screen capture
/// Saves frames as raw BGRA data for now
/// TODO: Add proper H.264 encoding and MP4 container
pub struct VideoEncoder {
    config: VideoEncoderConfig,
    output_path: PathBuf,
    frame_count: Arc<Mutex<u64>>,
    is_recording: Arc<Mutex<bool>>,
    raw_frames: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl VideoEncoder {
    /// Create a new video encoder
    pub fn new(config: VideoEncoderConfig, output_path: PathBuf) -> Result<Self> {
        Ok(Self {
            config,
            output_path,
            frame_count: Arc::new(Mutex::new(0)),
            is_recording: Arc::new(Mutex::new(false)),
            raw_frames: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Start recording
    pub async fn start_recording(&self) -> Result<()> {
        let mut is_recording = self.is_recording.lock().await;
        
        if *is_recording {
            return Ok(());
        }
        
        *is_recording = true;
        
        tracing::info!("Video recording started: {}x{} @ {} fps", 
            self.config.width, self.config.height, self.config.fps);
        
        Ok(())
    }
    
    /// Encode a frame (BGRA format)
    pub async fn encode_frame(&self, frame_data: &[u8]) -> Result<()> {
        let is_recording = self.is_recording.lock().await;
        if !*is_recording {
            return Ok(());
        }
        drop(is_recording);
        
        // Store frame data
        let mut frames = self.raw_frames.lock().await;
        frames.push(frame_data.to_vec());
        
        // Increment frame count
        let mut frame_count = self.frame_count.lock().await;
        *frame_count += 1;
        
        if *frame_count % 30 == 0 {
            tracing::debug!("Captured {} frames", *frame_count);
        }
        
        Ok(())
    }
    
    /// Stop recording and save to file
    pub async fn stop_recording(&self) -> Result<PathBuf> {
        let mut is_recording = self.is_recording.lock().await;
        if !*is_recording {
            return Ok(self.output_path.clone());
        }
        
        *is_recording = false;
        drop(is_recording);
        
        let frames = self.raw_frames.lock().await;
        let frame_count = *self.frame_count.lock().await;
        
        tracing::info!("Stopping recording. Total frames: {}", frame_count);
        
        // Save frames as raw BGRA data
        // Format: width(4 bytes) + height(4 bytes) + frame_count(8 bytes) + frames
        let mut file = File::create(&self.output_path)
            .context("Failed to create output file")?;
        
        // Write header
        file.write_all(&self.config.width.to_le_bytes())?;
        file.write_all(&self.config.height.to_le_bytes())?;
        file.write_all(&frame_count.to_le_bytes())?;
        file.write_all(&self.config.fps.to_le_bytes())?;
        
        // Write all frames
        for frame in frames.iter() {
            file.write_all(frame)?;
        }
        
        tracing::info!("Video saved to: {} ({} frames)", self.output_path.display(), frame_count);
        
        Ok(self.output_path.clone())
    }
    
    /// Get current frame count
    pub async fn get_frame_count(&self) -> u64 {
        *self.frame_count.lock().await
    }
    
    /// Check if currently recording
    pub async fn is_recording(&self) -> bool {
        *self.is_recording.lock().await
    }
}
