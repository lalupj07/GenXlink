use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use crate::audio_capture::{AudioCapturer, AudioConfig, AudioFrame};

/// Audio streaming manager
/// Handles audio capture and streaming
pub struct AudioStreamManager {
    capturer: Arc<Mutex<Option<AudioCapturer>>>,
    is_streaming: Arc<Mutex<bool>>,
    config: AudioConfig,
    
    // Channel for audio frames
    frame_tx: mpsc::UnboundedSender<AudioFrame>,
    frame_rx: Arc<Mutex<mpsc::UnboundedReceiver<AudioFrame>>>,
}

impl AudioStreamManager {
    pub fn new() -> Self {
        let (frame_tx, frame_rx) = mpsc::unbounded_channel();
        
        Self {
            capturer: Arc::new(Mutex::new(None)),
            is_streaming: Arc::new(Mutex::new(false)),
            config: AudioConfig::default(),
            frame_tx,
            frame_rx: Arc::new(Mutex::new(frame_rx)),
        }
    }
    
    /// Start audio streaming
    pub async fn start_streaming(&self) -> Result<()> {
        let mut is_streaming = self.is_streaming.lock().await;
        if *is_streaming {
            return Err(anyhow::anyhow!("Already streaming"));
        }
        *is_streaming = true;
        drop(is_streaming);
        
        // Create audio capturer
        let capturer = AudioCapturer::new(self.config.clone())
            .context("Failed to create audio capturer")?;
        
        // Start capture with callback
        let frame_tx = self.frame_tx.clone();
        capturer.start_capture(move |frame| {
            // Send frame to channel
            frame_tx.send(frame).ok();
            Ok(())
        }).await?;
        
        // Store capturer
        let mut capturer_guard = self.capturer.lock().await;
        *capturer_guard = Some(capturer);
        
        tracing::info!("Audio streaming started");
        
        Ok(())
    }
    
    /// Stop audio streaming
    pub async fn stop_streaming(&self) -> Result<()> {
        let mut is_streaming = self.is_streaming.lock().await;
        *is_streaming = false;
        drop(is_streaming);
        
        // Stop capturer
        let mut capturer_guard = self.capturer.lock().await;
        if let Some(capturer) = capturer_guard.as_ref() {
            capturer.stop_capture().await;
        }
        *capturer_guard = None;
        
        tracing::info!("Audio streaming stopped");
        
        Ok(())
    }
    
    /// Check if currently streaming
    pub async fn is_streaming(&self) -> bool {
        *self.is_streaming.lock().await
    }
    
    /// Get audio frame receiver
    pub fn get_frame_receiver(&self) -> Arc<Mutex<mpsc::UnboundedReceiver<AudioFrame>>> {
        Arc::clone(&self.frame_rx)
    }
    
    /// Set audio configuration
    pub fn set_config(&mut self, config: AudioConfig) {
        self.config = config;
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &AudioConfig {
        &self.config
    }
}

impl Default for AudioStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Audio statistics
#[derive(Debug, Clone, Default)]
pub struct AudioStats {
    pub frames_captured: u64,
    pub frames_dropped: u64,
    pub bytes_processed: u64,
    pub average_latency_ms: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audio_stream_manager_creation() {
        let manager = AudioStreamManager::new();
        assert!(!manager.is_streaming().await);
    }
    
    #[tokio::test]
    async fn test_audio_config() {
        let mut manager = AudioStreamManager::new();
        let config = AudioConfig {
            sample_rate: 44100,
            channels: 1,
            bits_per_sample: 16,
        };
        manager.set_config(config.clone());
        assert_eq!(manager.get_config().sample_rate, 44100);
    }
}
