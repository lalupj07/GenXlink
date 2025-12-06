use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use crate::audio_capture::{AudioCapturer, AudioConfig, AudioFrame};
use crate::audio_playback::{AudioPlayer, PlaybackConfig};

/// Audio streaming manager
/// Handles audio capture and streaming with bidirectional support
pub struct AudioStreamManager {
    capturer: Arc<Mutex<Option<AudioCapturer>>>,
    player: Arc<Mutex<Option<AudioPlayer>>>,
    is_streaming: Arc<Mutex<bool>>,
    is_playing: Arc<Mutex<bool>>,
    config: AudioConfig,
    playback_config: PlaybackConfig,
    
    // Channels for audio frames
    capture_tx: mpsc::UnboundedSender<AudioFrame>,
    capture_rx: Arc<Mutex<mpsc::UnboundedReceiver<AudioFrame>>>,
    playback_tx: mpsc::UnboundedSender<AudioFrame>,
    playback_rx: Arc<Mutex<mpsc::UnboundedReceiver<AudioFrame>>>,
}

impl AudioStreamManager {
    pub fn new() -> Self {
        let (capture_tx, capture_rx) = mpsc::unbounded_channel();
        let (playback_tx, playback_rx) = mpsc::unbounded_channel();
        
        Self {
            capturer: Arc::new(Mutex::new(None)),
            player: Arc::new(Mutex::new(None)),
            is_streaming: Arc::new(Mutex::new(false)),
            is_playing: Arc::new(Mutex::new(false)),
            config: AudioConfig::default(),
            playback_config: PlaybackConfig::default(),
            capture_tx,
            capture_rx: Arc::new(Mutex::new(capture_rx)),
            playback_tx,
            playback_rx: Arc::new(Mutex::new(playback_rx)),
        }
    }
    
    /// Start audio streaming (capture)
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
        let frame_tx = self.capture_tx.clone();
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
    
    /// Stop audio streaming (capture)
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
    
    /// Start audio playback
    pub async fn start_playback(&self) -> Result<()> {
        let mut is_playing = self.is_playing.lock().await;
        if *is_playing {
            return Err(anyhow::anyhow!("Already playing"));
        }
        *is_playing = true;
        drop(is_playing);
        
        // Create audio player
        let player = AudioPlayer::new(self.playback_config.clone())
            .context("Failed to create audio player")?;
        
        // Start playback with frame callback
        let playback_rx = Arc::clone(&self.playback_rx);
        player.start_playback(move || {
            // Try to get next frame from receiver
            let mut rx = playback_rx.blocking_lock();
            match rx.try_recv() {
                Ok(frame) => Ok(Some(frame)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => Err(anyhow::anyhow!("Playback channel disconnected")),
            }
        }).await?;
        
        // Store player
        let mut player_guard = self.player.lock().await;
        *player_guard = Some(player);
        
        tracing::info!("Audio playback started");
        
        Ok(())
    }
    
    /// Stop audio playback
    pub async fn stop_playback(&self) -> Result<()> {
        let mut is_playing = self.is_playing.lock().await;
        *is_playing = false;
        drop(is_playing);
        
        // Stop player
        let mut player_guard = self.player.lock().await;
        if let Some(player) = player_guard.as_ref() {
            player.stop_playback().await;
        }
        *player_guard = None;
        
        tracing::info!("Audio playback stopped");
        
        Ok(())
    }
    
    /// Start full duplex audio (capture and playback)
    pub async fn start_full_duplex(&self) -> Result<()> {
        self.start_streaming().await?;
        self.start_playback().await?;
        tracing::info!("Full duplex audio started");
        Ok(())
    }
    
    /// Stop full duplex audio
    pub async fn stop_full_duplex(&self) -> Result<()> {
        self.stop_streaming().await?;
        self.stop_playback().await?;
        tracing::info!("Full duplex audio stopped");
        Ok(())
    }
    
    /// Check if currently streaming
    pub async fn is_streaming(&self) -> bool {
        *self.is_streaming.lock().await
    }
    
    /// Check if currently playing
    pub async fn is_playing(&self) -> bool {
        *self.is_playing.lock().await
    }
    
    /// Get audio frame receiver (for sending to remote)
    pub fn get_capture_receiver(&self) -> Arc<Mutex<mpsc::UnboundedReceiver<AudioFrame>>> {
        Arc::clone(&self.capture_rx)
    }
    
    /// Get audio frame sender (for receiving from remote)
    pub fn get_playback_sender(&self) -> mpsc::UnboundedSender<AudioFrame> {
        self.playback_tx.clone()
    }
    
    /// Send audio frame for playback (received from remote)
    pub async fn send_for_playback(&self, frame: AudioFrame) -> Result<()> {
        self.playback_tx.send(frame)
            .context("Failed to send frame for playback")?;
        Ok(())
    }
    
    /// Receive audio frame for streaming (captured locally)
    pub async fn receive_for_streaming(&self) -> Result<Option<AudioFrame>> {
        let mut rx = self.capture_rx.lock().await;
        match rx.try_recv() {
            Ok(frame) => Ok(Some(frame)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(anyhow::anyhow!("Capture channel disconnected")),
        }
    }
    
    /// Set audio configuration
    pub fn set_config(&mut self, config: AudioConfig) {
        self.config = config;
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &AudioConfig {
        &self.config
    }
    
    /// Set playback configuration
    pub fn set_playback_config(&mut self, config: PlaybackConfig) {
        self.playback_config = config;
    }
    
    /// Get current playback configuration
    pub fn get_playback_config(&self) -> &PlaybackConfig {
        &self.playback_config
    }
    
    /// Set volume for playback
    pub async fn set_volume(&self, volume: f32) -> Result<()> {
        let player_guard = self.player.lock().await;
        if let Some(player) = player_guard.as_ref() {
            player.set_volume(volume).await;
        } else {
            return Err(anyhow::anyhow!("Audio player not active"));
        }
        Ok(())
    }
    
    /// Get current volume
    pub async fn get_volume(&self) -> Result<f32> {
        let player_guard = self.player.lock().await;
        if let Some(player) = player_guard.as_ref() {
            Ok(player.get_volume().await)
        } else {
            Err(anyhow::anyhow!("Audio player not active"))
        }
    }
    
    /// Mute/unmute audio
    pub async fn set_muted(&self, muted: bool) -> Result<()> {
        let player_guard = self.player.lock().await;
        if let Some(player) = player_guard.as_ref() {
            player.set_muted(muted).await;
        } else {
            return Err(anyhow::anyhow!("Audio player not active"));
        }
        Ok(())
    }
    
    /// Check if muted
    pub async fn is_muted(&self) -> Result<bool> {
        let player_guard = self.player.lock().await;
        if let Some(player) = player_guard.as_ref() {
            Ok(player.is_muted().await)
        } else {
            Err(anyhow::anyhow!("Audio player not active"))
        }
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
            codec: crate::audio_capture::AudioCodec::PCM,
        };
        manager.set_config(config.clone());
        assert_eq!(manager.get_config().sample_rate, 44100);
    }
}
