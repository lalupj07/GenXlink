use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Audio format configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            sample_rate: 48000, // 48 kHz (high quality)
            channels: 2,         // Stereo
            bit_depth: 16,       // 16-bit
        }
    }
}

impl AudioFormat {
    /// CD quality audio
    pub fn cd_quality() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            bit_depth: 16,
        }
    }
    
    /// High quality audio
    pub fn high_quality() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bit_depth: 24,
        }
    }
    
    /// Low latency audio
    pub fn low_latency() -> Self {
        Self {
            sample_rate: 32000,
            channels: 2,
            bit_depth: 16,
        }
    }
    
    /// Mono audio (lower bandwidth)
    pub fn mono() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1,
            bit_depth: 16,
        }
    }
}

/// Audio quality preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioQuality {
    Low,
    Medium,
    High,
    Lossless,
}

impl AudioQuality {
    pub fn bitrate(&self) -> u32 {
        match self {
            Self::Low => 64_000,      // 64 kbps
            Self::Medium => 128_000,   // 128 kbps
            Self::High => 256_000,     // 256 kbps
            Self::Lossless => 1411_000, // 1411 kbps (CD quality)
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Low => "Low (64 kbps)",
            Self::Medium => "Medium (128 kbps)",
            Self::High => "High (256 kbps)",
            Self::Lossless => "Lossless (CD Quality)",
        }
    }
}

/// Audio codec
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioCodec {
    Opus,
    AAC,
    PCM,
}

impl AudioCodec {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Opus => "Opus (Best for streaming)",
            Self::AAC => "AAC (High quality)",
            Self::PCM => "PCM (Uncompressed)",
        }
    }
}

/// Audio streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub enabled: bool,
    pub format: AudioFormat,
    pub quality: AudioQuality,
    pub codec: AudioCodec,
    pub buffer_size: usize,
    pub latency_ms: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            format: AudioFormat::default(),
            quality: AudioQuality::High,
            codec: AudioCodec::Opus,
            buffer_size: 4096,
            latency_ms: 50, // 50ms latency
        }
    }
}

/// Audio capture device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_input: bool,
}

/// Audio streaming manager
pub struct AudioStreamManager {
    config: Arc<Mutex<AudioConfig>>,
    devices: Arc<Mutex<Vec<AudioDevice>>>,
    is_streaming: Arc<Mutex<bool>>,
}

impl Default for AudioStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStreamManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AudioConfig::default())),
            devices: Arc::new(Mutex::new(Vec::new())),
            is_streaming: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start audio streaming
    pub fn start_streaming(&self) -> Result<(), String> {
        let mut is_streaming = self.is_streaming.lock().unwrap();
        
        if *is_streaming {
            return Err("Audio streaming already active".to_string());
        }
        
        // TODO: Initialize audio capture
        // This would use platform-specific APIs:
        // - Windows: WASAPI
        // - Linux: PulseAudio/ALSA
        // - macOS: CoreAudio
        
        *is_streaming = true;
        Ok(())
    }
    
    /// Stop audio streaming
    pub fn stop_streaming(&self) {
        let mut is_streaming = self.is_streaming.lock().unwrap();
        *is_streaming = false;
        
        // TODO: Cleanup audio capture
    }
    
    /// Check if streaming
    pub fn is_streaming(&self) -> bool {
        *self.is_streaming.lock().unwrap()
    }
    
    /// Get configuration
    pub fn get_config(&self) -> AudioConfig {
        self.config.lock().unwrap().clone()
    }
    
    /// Update configuration
    pub fn set_config(&self, config: AudioConfig) {
        *self.config.lock().unwrap() = config;
    }
    
    /// List available audio devices
    pub fn list_devices(&self) -> Vec<AudioDevice> {
        // TODO: Enumerate audio devices
        // This would use platform-specific APIs
        
        // Return mock devices for now
        vec![
            AudioDevice {
                id: "default".to_string(),
                name: "Default Audio Device".to_string(),
                is_default: true,
                is_input: false,
            },
            AudioDevice {
                id: "speakers".to_string(),
                name: "Speakers".to_string(),
                is_default: false,
                is_input: false,
            },
            AudioDevice {
                id: "headphones".to_string(),
                name: "Headphones".to_string(),
                is_default: false,
                is_input: false,
            },
        ]
    }
    
    /// Set audio device
    pub fn set_device(&self, device_id: &str) -> Result<(), String> {
        // TODO: Set audio capture device
        tracing::info!("Setting audio device: {}", device_id);
        Ok(())
    }
    
    /// Get current volume level (0.0 - 1.0)
    pub fn get_volume(&self) -> f32 {
        // TODO: Get actual volume
        1.0
    }
    
    /// Set volume level (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        tracing::info!("Setting audio volume: {}", volume);
        // TODO: Set actual volume
    }
    
    /// Mute/unmute audio
    pub fn set_mute(&self, muted: bool) {
        tracing::info!("Setting audio mute: {}", muted);
        // TODO: Set actual mute state
    }
}

/// Audio statistics
#[derive(Debug, Clone, Default)]
pub struct AudioStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub current_latency_ms: u32,
    pub average_latency_ms: u32,
    pub jitter_ms: u32,
}

impl AudioStats {
    pub fn packet_loss_rate(&self) -> f32 {
        if self.packets_sent == 0 {
            return 0.0;
        }
        (self.packets_lost as f32 / self.packets_sent as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_format() {
        let format = AudioFormat::default();
        assert_eq!(format.sample_rate, 48000);
        assert_eq!(format.channels, 2);
    }
    
    #[test]
    fn test_audio_quality() {
        assert_eq!(AudioQuality::High.bitrate(), 256_000);
        assert_eq!(AudioQuality::Low.bitrate(), 64_000);
    }
    
    #[test]
    fn test_audio_manager() {
        let manager = AudioStreamManager::new();
        assert!(!manager.is_streaming());
        
        let devices = manager.list_devices();
        assert!(!devices.is_empty());
    }
}
