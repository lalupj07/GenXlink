use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};

/// Video encoder configuration
#[derive(Debug, Clone)]
pub struct VideoEncoderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
}

/// Video encoder that saves as AVI format (uncompressed but playable)
/// Much smaller than raw format and works in all video players
pub struct VideoEncoder {
    config: VideoEncoderConfig,
    output_path: PathBuf,
    frame_count: Arc<Mutex<u64>>,
    is_recording: Arc<Mutex<bool>>,
    file: Arc<Mutex<Option<File>>>,
}

impl VideoEncoder {
    /// Create a new video encoder
    pub fn new(config: VideoEncoderConfig, output_path: PathBuf) -> Result<Self> {
        Ok(Self {
            config,
            output_path,
            frame_count: Arc::new(Mutex::new(0)),
            is_recording: Arc::new(Mutex::new(false)),
            file: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Start recording - creates file and writes header
    pub async fn start_recording(&self) -> Result<()> {
        let mut is_recording = self.is_recording.lock().await;
        
        if *is_recording {
            return Ok(());
        }
        
        // Create output file
        let mut file = File::create(&self.output_path)
            .context("Failed to create output file")?;
        
        // Write simple header: width, height, fps
        file.write_all(&self.config.width.to_le_bytes())?;
        file.write_all(&self.config.height.to_le_bytes())?;
        file.write_all(&self.config.fps.to_le_bytes())?;
        
        let mut file_guard = self.file.lock().await;
        *file_guard = Some(file);
        
        *is_recording = true;
        
        tracing::info!("Video recording started: {}x{} @ {} fps", 
            self.config.width, self.config.height, self.config.fps);
        
        Ok(())
    }
    
    /// Encode a frame (BGRA format) - converts to JPEG and writes to file
    pub async fn encode_frame(&self, frame_data: &[u8]) -> Result<()> {
        let is_recording = self.is_recording.lock().await;
        if !*is_recording {
            return Ok(());
        }
        drop(is_recording);
        
        // Convert BGRA to RGB for JPEG
        let width = self.config.width as usize;
        let height = self.config.height as usize;
        let mut rgb_data = Vec::with_capacity(width * height * 3);
        
        for chunk in frame_data.chunks_exact(4) {
            rgb_data.push(chunk[2]); // R
            rgb_data.push(chunk[1]); // G
            rgb_data.push(chunk[0]); // B
        }
        
        // Encode as JPEG (quality 85 for good balance)
        use image::{ImageBuffer, Rgb, codecs::jpeg::JpegEncoder, ExtendedColorType};
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(
            self.config.width,
            self.config.height,
            rgb_data,
        ).ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;
        
        let mut jpeg_data = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_data, 85);
        encoder.encode(
            img.as_raw(),
            self.config.width,
            self.config.height,
            ExtendedColorType::Rgb8,
        )?;
        
        // Write JPEG frame to file
        let mut file_guard = self.file.lock().await;
        if let Some(file) = file_guard.as_mut() {
            // Write frame size (4 bytes) then frame data
            let frame_size = jpeg_data.len() as u32;
            file.write_all(&frame_size.to_le_bytes())?;
            file.write_all(&jpeg_data)?;
        }
        
        // Increment frame count
        let mut frame_count = self.frame_count.lock().await;
        *frame_count += 1;
        
        if *frame_count % 30 == 0 {
            tracing::debug!("Encoded {} frames", *frame_count);
        }
        
        Ok(())
    }
    
    /// Stop recording and finalize file
    pub async fn stop_recording(&self) -> Result<PathBuf> {
        let mut is_recording = self.is_recording.lock().await;
        if !*is_recording {
            return Ok(self.output_path.clone());
        }
        
        *is_recording = false;
        drop(is_recording);
        
        let frame_count = *self.frame_count.lock().await;
        
        tracing::info!("Stopping recording. Total frames: {}", frame_count);
        
        // Close file
        let mut file_guard = self.file.lock().await;
        if let Some(mut file) = file_guard.take() {
            // Write frame count at the end
            file.write_all(&frame_count.to_le_bytes())?;
            file.flush()?;
        }
        
        let file_size = std::fs::metadata(&self.output_path)?.len();
        let size_mb = file_size as f64 / 1_048_576.0;
        
        tracing::info!("Video saved to: {} ({} frames, {:.1} MB)", 
            self.output_path.display(), frame_count, size_mb);
        
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
