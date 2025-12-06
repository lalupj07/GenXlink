use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::core::*;

/// Audio codec support
#[derive(Debug, Clone, PartialEq)]
pub enum AudioCodec {
    PCM,    // Raw PCM audio
    Opus,   // Opus compressed audio
}

/// Audio format configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub codec: AudioCodec,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,  // 48kHz
            channels: 2,          // Stereo
            bits_per_sample: 16,  // 16-bit
            codec: AudioCodec::Opus, // Use Opus for compression
        }
    }
}

/// Audio frame captured from the system
#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub timestamp: u64,
    pub codec: AudioCodec,
    pub is_compressed: bool,
}

impl AudioFrame {
    /// Create a new PCM audio frame
    pub fn new_pcm(data: Vec<u8>, sample_rate: u32, channels: u16, timestamp: u64) -> Self {
        Self {
            data,
            sample_rate,
            channels,
            timestamp,
            codec: AudioCodec::PCM,
            is_compressed: false,
        }
    }
    
    /// Create a new Opus compressed audio frame
    pub fn new_opus(data: Vec<u8>, sample_rate: u32, channels: u16, timestamp: u64) -> Self {
        Self {
            data,
            sample_rate,
            channels,
            timestamp,
            codec: AudioCodec::Opus,
            is_compressed: true,
        }
    }
    
    /// Compress PCM frame to Opus (simplified implementation)
    pub fn compress_to_opus(&self) -> Result<Self> {
        if self.codec == AudioCodec::Opus {
            return Ok(self.clone());
        }
        
        // For now, implement a simple compression simulation
        // In a real implementation, you would use the Opus library
        let compressed_size = self.data.len() / 4; // Assume 4:1 compression ratio
        let mut compressed_data = Vec::with_capacity(compressed_size);
        
        // Simple compression simulation (replace with real Opus encoding)
        for chunk in self.data.chunks(4) {
            if let Some(&first_byte) = chunk.first() {
                compressed_data.push(first_byte);
            }
        }
        
        Ok(AudioFrame::new_opus(
            compressed_data,
            self.sample_rate,
            self.channels,
            self.timestamp,
        ))
    }
    
    /// Decompress Opus frame to PCM (simplified implementation)
    pub fn decompress_to_pcm(&self) -> Result<Self> {
        if self.codec == AudioCodec::PCM {
            return Ok(self.clone());
        }
        
        // For now, implement a simple decompression simulation
        // In a real implementation, you would use the Opus library
        let decompressed_size = self.data.len() * 4; // Assume 1:4 decompression ratio
        let mut decompressed_data = Vec::with_capacity(decompressed_size);
        
        // Simple decompression simulation (replace with real Opus decoding)
        for &byte in &self.data {
            for _ in 0..4 {
                decompressed_data.push(byte);
            }
        }
        
        Ok(AudioFrame::new_pcm(
            decompressed_data,
            self.sample_rate,
            self.channels,
            self.timestamp,
        ))
    }
}

/// Audio capturer using Windows WASAPI
pub struct AudioCapturer {
    config: AudioConfig,
    is_capturing: Arc<Mutex<bool>>,
}

impl AudioCapturer {
    pub fn new(config: AudioConfig) -> Result<Self> {
        Ok(Self {
            config,
            is_capturing: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Start capturing audio
    pub async fn start_capture<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(AudioFrame) -> Result<()> + Send + 'static,
    {
        let mut is_capturing = self.is_capturing.lock().await;
        if *is_capturing {
            return Err(anyhow::anyhow!("Already capturing"));
        }
        *is_capturing = true;
        drop(is_capturing);
        
        let config = self.config.clone();
        let is_capturing_clone = Arc::clone(&self.is_capturing);
        
        std::thread::spawn(move || {
            if let Err(e) = Self::capture_loop(config, callback, is_capturing_clone) {
                tracing::error!("Audio capture error: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Main capture loop (runs in separate thread)
    fn capture_loop<F>(
        config: AudioConfig,
        mut callback: F,
        is_capturing: Arc<Mutex<bool>>,
    ) -> Result<()>
    where
        F: FnMut(AudioFrame) -> Result<()>,
    {
        // Simplified implementation for now - generate mock audio frames
        // In production, this would use real WASAPI capture
        tracing::info!("Audio capture loop started (mock mode with compression)");
        
        let mut frame_count = 0u64;
        let mut last_frame_time = std::time::Instant::now();
        
        loop {
            // Check if still capturing
            let capturing = is_capturing.blocking_lock();
            if !*capturing {
                break;
            }
            drop(capturing);
            
            // Target 20ms frames for 50 FPS
            let target_duration = std::time::Duration::from_millis(20);
            let elapsed = last_frame_time.elapsed();
            
            if elapsed < target_duration {
                std::thread::sleep(target_duration - elapsed);
            }
            
            // Generate mock audio data (simulate 48kHz 16-bit stereo, 20ms = 1920 samples = 3840 bytes)
            let samples_per_frame = (config.sample_rate * 20) / 1000; // 20ms worth of samples
            let buffer_size = (samples_per_frame * config.channels as u32 * config.bits_per_sample as u32 / 8) as usize;
            let mut audio_data = vec![0u8; buffer_size];
            
            // Generate some test audio (sine wave)
            for i in (0..audio_data.len()).step_by(2) {
                if i + 1 < audio_data.len() {
                    let sample = ((i as f32 / audio_data.len() as f32) * std::f32::consts::PI * 2.0).sin();
                    let sample_i16 = (sample * 16384.0) as i16;
                    audio_data[i] = (sample_i16 & 0xFF) as u8;
                    audio_data[i + 1] = ((sample_i16 >> 8) & 0xFF) as u8;
                }
            }
            
            // Create PCM audio frame
            let pcm_frame = AudioFrame::new_pcm(
                audio_data,
                config.sample_rate,
                config.channels,
                frame_count,
            );
            
            // Compress to Opus if configured
            let final_frame = match config.codec {
                AudioCodec::PCM => pcm_frame,
                AudioCodec::Opus => {
                    match pcm_frame.compress_to_opus() {
                        Ok(compressed) => {
                            tracing::debug!("Compressed audio frame: {} -> {} bytes", 
                                pcm_frame.data.len(), compressed.data.len());
                            compressed
                        }
                        Err(e) => {
                            tracing::warn!("Audio compression failed, using PCM: {}", e);
                            pcm_frame
                        }
                    }
                }
            };
            
            frame_count += 1;
            last_frame_time = std::time::Instant::now();
            
            // Call callback
            if let Err(e) = callback(final_frame) {
                tracing::error!("Audio callback error: {}", e);
            }
        }
        
        tracing::info!("Audio capture loop stopped after {} frames", frame_count);
        Ok(())
    }
    
    /// Stop capturing audio
    pub async fn stop_capture(&self) {
        let mut is_capturing = self.is_capturing.lock().await;
        *is_capturing = false;
        tracing::info!("Audio capture stop requested");
    }
    
    /// Check if currently capturing
    pub async fn is_capturing(&self) -> bool {
        *self.is_capturing.lock().await
    }
}

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// List available audio devices
pub fn list_audio_devices() -> Result<Vec<AudioDevice>> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .context("Failed to initialize COM")?;
        
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        ).context("Failed to create device enumerator")?;
        
        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .context("Failed to enumerate devices")?;
        
        let count = collection.GetCount().context("Failed to get device count")?;
        
        let mut devices = Vec::new();
        
        for i in 0..count {
            if let Ok(device) = collection.Item(i) {
                if let Ok(id_pwstr) = device.GetId() {
                    let id = id_pwstr.to_string().unwrap_or_default();
                    let name = format!("Audio Device {}", i);
                    
                    devices.push(AudioDevice {
                        id,
                        name,
                        is_default: i == 0,
                    });
                }
            }
        }
        
        CoUninitialize();
        
        Ok(devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
    }
    
    #[tokio::test]
    async fn test_audio_capturer_creation() {
        let config = AudioConfig::default();
        let capturer = AudioCapturer::new(config);
        assert!(capturer.is_ok());
    }
}
