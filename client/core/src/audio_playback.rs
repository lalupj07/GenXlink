use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::audio_capture::{AudioFrame, AudioCodec};

/// Audio playback configuration
#[derive(Debug, Clone)]
pub struct PlaybackConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub buffer_size_ms: u32,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 16,
            buffer_size_ms: 100, // 100ms buffer
        }
    }
}

/// Audio player using Windows WASAPI
pub struct AudioPlayer {
    config: PlaybackConfig,
    is_playing: Arc<Mutex<bool>>,
    volume: Arc<Mutex<f32>>, // 0.0 to 1.0
}

impl AudioPlayer {
    pub fn new(config: PlaybackConfig) -> Result<Self> {
        Ok(Self {
            config,
            is_playing: Arc::new(Mutex::new(false)),
            volume: Arc::new(Mutex::new(1.0)),
        })
    }
    
    /// Start audio playback with frame callback
    pub async fn start_playback<F>(&self, frame_callback: F) -> Result<()>
    where
        F: FnMut() -> Result<Option<AudioFrame>> + Send + 'static,
    {
        let mut is_playing = self.is_playing.lock().await;
        if *is_playing {
            return Err(anyhow::anyhow!("Already playing"));
        }
        *is_playing = true;
        drop(is_playing);
        
        let config = self.config.clone();
        let is_playing_clone = Arc::clone(&self.is_playing);
        let volume_clone = Arc::clone(&self.volume);
        
        std::thread::spawn(move || {
            if let Err(e) = Self::playback_loop(config, frame_callback, is_playing_clone, volume_clone) {
                tracing::error!("Audio playback error: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Main playback loop (runs in separate thread)
    fn playback_loop<F>(
        _config: PlaybackConfig,
        mut frame_callback: F,
        is_playing: Arc<Mutex<bool>>,
        volume: Arc<Mutex<f32>>,
    ) -> Result<()>
    where
        F: FnMut() -> Result<Option<AudioFrame>>,
    {
        // Simplified implementation for now - simulate audio playback
        // In production, this would use real WASAPI playback
        tracing::info!("Audio playback loop started (mock mode with decompression)");
        
        let mut frames_played = 0u64;
        let mut last_frame_time = std::time::Instant::now();
        
        loop {
            // Check if still playing
            let playing = is_playing.blocking_lock();
            if !*playing {
                break;
            }
            drop(playing);
            
            // Target 20ms frame intervals
            let target_duration = std::time::Duration::from_millis(20);
            let elapsed = last_frame_time.elapsed();
            
            if elapsed < target_duration {
                std::thread::sleep(target_duration - elapsed);
            }
            
            // Get next frame
            let audio_frame = match frame_callback() {
                Ok(Some(frame)) => frame,
                Ok(None) => {
                    // No frame available, continue
                    continue;
                }
                Err(e) => {
                    tracing::error!("Frame callback error: {}", e);
                    continue;
                }
            };
            
            // Decompress if needed
            let final_frame = match audio_frame.codec {
                AudioCodec::PCM => audio_frame,
                AudioCodec::Opus => {
                    match audio_frame.decompress_to_pcm() {
                        Ok(decompressed) => {
                            tracing::debug!("Decompressed audio frame: {} -> {} bytes", 
                                audio_frame.data.len(), decompressed.data.len());
                            decompressed
                        }
                        Err(e) => {
                            tracing::warn!("Audio decompression failed, skipping frame: {}", e);
                            continue;
                        }
                    }
                }
            };
            
            // Apply volume and "play" (simulate playback)
            let current_volume = *volume.blocking_lock();
            if current_volume > 0.0 && !final_frame.data.is_empty() {
                // Simulate audio processing time
                std::thread::sleep(std::time::Duration::from_millis(1));
                
                if current_volume != 1.0 {
                    // In real implementation, would apply volume to samples
                    tracing::debug!("Applied volume {:.2} to audio frame", current_volume);
                }
                
                frames_played += 1;
                
                // Log playback progress every 100 frames
                if frames_played % 100 == 0 {
                    tracing::debug!("Played {} audio frames (last: {} bytes)", 
                        frames_played, final_frame.data.len());
                }
            }
            
            last_frame_time = std::time::Instant::now();
        }
        
        tracing::info!("Audio playback loop stopped after {} frames", frames_played);
        Ok(())
    }
    
    /// Stop audio playback
    pub async fn stop_playback(&self) {
        let mut is_playing = self.is_playing.lock().await;
        *is_playing = false;
        tracing::info!("Audio playback stop requested");
    }
    
    /// Check if currently playing
    pub async fn is_playing(&self) -> bool {
        *self.is_playing.lock().await
    }
    
    /// Set volume (0.0 to 1.0)
    pub async fn set_volume(&self, volume: f32) {
        let mut vol = self.volume.lock().await;
        *vol = volume.clamp(0.0, 1.0);
        tracing::info!("Audio volume set to: {:.2}", *vol);
    }
    
    /// Get current volume
    pub async fn get_volume(&self) -> f32 {
        *self.volume.lock().await
    }
    
    /// Mute/unmute audio
    pub async fn set_muted(&self, muted: bool) {
        let mut vol = self.volume.lock().await;
        *vol = if muted { 0.0 } else { 1.0 };
        tracing::info!("Audio mute set to: {}", muted);
    }
    
    /// Check if muted
    pub async fn is_muted(&self) -> bool {
        *self.volume.lock().await == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_playback_config_default() {
        let config = PlaybackConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.buffer_size_ms, 100);
    }
    
    #[tokio::test]
    async fn test_audio_player_creation() {
        let config = PlaybackConfig::default();
        let player = AudioPlayer::new(config);
        assert!(player.is_ok());
    }
    
    #[tokio::test]
    async fn test_volume_control() {
        let config = PlaybackConfig::default();
        let player = AudioPlayer::new(config).unwrap();
        
        player.set_volume(0.5).await;
        assert_eq!(player.get_volume().await, 0.5);
        
        player.set_muted(true).await;
        assert!(player.is_muted().await);
        assert_eq!(player.get_volume().await, 0.0);
    }
}
