use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{info, error, warn, debug};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, StreamConfig, SampleFormat, SupportedStreamConfigRange,
};

use crate::media_manager::{AudioCapture, AudioDeviceInfo, AudioConfig};

pub struct SystemAudioCapture {
    is_capturing: Arc<Mutex<bool>>,
    capture_handle: Arc<Mutex<Option<CaptureHandle>>>,
    host: Host,
}

struct CaptureHandle {
    stop_tx: mpsc::Sender<()>,
    config: AudioConfig,
    stream: cpal::Stream,
}

impl SystemAudioCapture {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        
        Ok(Self {
            is_capturing: Arc::new(Mutex::new(false)),
            capture_handle: Arc::new(Mutex::new(None)),
            host,
        })
    }

    async fn capture_audio_loop(
        config: AudioConfig,
        mut stop_rx: mpsc::Receiver<()>,
        frame_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        info!("Starting audio capture with config: {:?}", config);

        // Get default input device
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| anyhow!("No audio input device found"))?;

        // Get supported config
        let supported_configs = device.supported_input_configs()
            .map_err(|e| anyhow!("Failed to get supported audio configs: {}", e))?;

        // Find best matching config
        let supported_config = supported_configs
            .find(|config| {
                config.channels() == config.channels as u32 &&
                config.sample_rate().0 >= config.sample_rate &&
                config.sample_format() == SampleFormat::F32
            })
            .or_else(|| supported_configs.find(|config| config.sample_format() == SampleFormat::F32))
            .ok_or_else(|| anyhow!("No suitable audio configuration found"))?;

        let config: StreamConfig = supported_config.clone().into();

        info!("Using audio config: {:?}", config);

        // Create audio buffer
        let frame_tx_clone = frame_tx.clone();
        let config_clone = config.clone();
        
        // Build audio stream
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                Self::process_audio_frame(data, &frame_tx_clone, &config_clone);
            },
            move |err| {
                error!("Audio stream error: {}", err);
            },
            None,
        ).map_err(|e| anyhow!("Failed to build audio stream: {}", e))?;

        // Start the stream
        stream.play().map_err(|e| anyhow!("Failed to start audio stream: {}", e))?;

        info!("Audio capture started");

        // Wait for stop signal
        let _ = stop_rx.recv().await;

        // Stop the stream
        drop(stream);
        info!("Audio capture stopped");

        Ok(())
    }

    fn process_audio_frame(
        data: &[f32],
        frame_tx: &mpsc::Sender<Vec<u8>>,
        config: &StreamConfig,
    ) {
        // Apply audio processing
        let processed_data = Self::process_audio_samples(data, config);
        
        // Convert to bytes
        let byte_data: Vec<u8> = processed_data
            .iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect();

        debug!("Processed audio frame: {} samples, {} bytes", data.len(), byte_data.len());

        // Send frame
        if let Err(e) = frame_tx.try_send(byte_data) {
            warn!("Failed to send audio frame: {}", e);
        }
    }

    fn process_audio_samples(data: &[f32], config: &StreamConfig) -> Vec<f32> {
        let mut processed = Vec::with_capacity(data.len());

        // Apply audio processing
        for &sample in data {
            // 1. Apply noise gate
            let gated = if sample.abs() < 0.001 { 0.0 } else { sample };

            // 2. Apply basic noise reduction (simple high-pass filter)
            let noise_reduced = Self::apply_high_pass_filter(gated, config.sample_rate().0);

            // 3. Apply compression
            let compressed = Self::apply_compression(noise_reduced);

            // 4. Apply echo cancellation (simplified)
            let echo_cancelled = Self::apply_echo_cancellation(compressed);

            processed.push(echo_cancelled);
        }

        processed
    }

    fn apply_high_pass_filter(sample: f32, sample_rate: u32) -> f32 {
        // Simple high-pass filter to reduce low-frequency noise
        const CUTOFF_FREQ: f32 = 80.0;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * CUTOFF_FREQ);
        let dt = 1.0 / sample_rate as f32;
        let alpha = rc / (rc + dt);

        // This is a simplified version - in production, maintain filter state
        sample * (1.0 - alpha)
    }

    fn apply_compression(sample: f32) -> f32 {
        // Simple audio compression
        const THRESHOLD: f32 = 0.7;
        const RATIO: f32 = 4.0;

        if sample.abs() > THRESHOLD {
            let excess = sample.abs() - THRESHOLD;
            let compressed_excess = excess / RATIO;
            if sample > 0.0 {
                THRESHOLD + compressed_excess
            } else {
                -(THRESHOLD + compressed_excess)
            }
        } else {
            sample
        }
    }

    fn apply_echo_cancellation(sample: f32) -> f32 {
        // Simplified echo cancellation
        // In production, use adaptive filters and echo reference
        sample * 0.95 // Simple attenuation
    }

    async fn compress_audio_frame(&self, audio_data: &[f32]) -> Result<Vec<u8>> {
        // Simplified audio compression - in production use Opus encoder
        // For now, we'll use delta encoding
        
        if audio_data.is_empty() {
            return Ok(Vec::new());
        }

        let mut compressed = Vec::new();
        let mut prev_sample = 0.0f32;

        // Store first sample as-is
        compressed.extend_from_slice(&audio_data[0].to_le_bytes());

        // Store deltas for remaining samples
        for &sample in &audio_data[1..] {
            let delta = sample - prev_sample;
            compressed.extend_from_slice(&delta.to_le_bytes());
            prev_sample = sample;
        }

        debug!("Compressed audio from {} to {} bytes", audio_data.len() * 4, compressed.len());
        Ok(compressed)
    }
}

#[async_trait::async_trait]
impl AudioCapture for SystemAudioCapture {
    async fn start_capture(&self, config: AudioConfig) -> Result<mpsc::Receiver<Vec<u8>>> {
        // Check if already capturing
        let mut is_capturing = self.is_capturing.lock().await;
        if *is_capturing {
            return Err(anyhow!("Audio capture already in progress"));
        }

        *is_capturing = true;
        drop(is_capturing);

        let (frame_tx, frame_rx) = mpsc::channel(100);
        let (stop_tx, stop_rx) = mpsc::channel(1);

        // Start capture loop
        let config_clone = config.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::capture_audio_loop(config_clone, stop_rx, frame_tx).await {
                error!("Audio capture loop error: {}", e);
            }
        });

        info!("Started audio capture");
        Ok(frame_rx)
    }

    async fn stop_capture(&self) -> Result<()> {
        let mut is_capturing = self.is_capturing.lock().await;
        if !*is_capturing {
            return Ok(());
        }

        *is_capturing = false;
        drop(is_capturing);

        // Send stop signal
        let mut capture_handle = self.capture_handle.lock().await;
        if let Some(handle) = capture_handle.take() {
            if let Err(e) = handle.stop_tx.send(()).await {
                warn!("Failed to send stop signal: {}", e);
            }
            drop(handle.stream); // This will stop the audio stream
        }

        info!("Stopped audio capture");
        Ok(())
    }

    async fn get_device_list(&self) -> Result<Vec<AudioDeviceInfo>> {
        let mut devices = Vec::new();

        // Get input devices
        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    if let Ok(default_config) = device.default_input_config() {
                        let device_info = AudioDeviceInfo {
                            id: name.clone(),
                            name,
                            device_type: "input".to_string(),
                            channels: default_config.channels() as u8,
                            sample_rate: default_config.sample_rate().0,
                        };
                        devices.push(device_info);
                    }
                }
            }
        }

        // Get output devices
        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    if let Ok(default_config) = device.default_output_config() {
                        let device_info = AudioDeviceInfo {
                            id: name.clone(),
                            name,
                            device_type: "output".to_string(),
                            channels: default_config.channels() as u8,
                            sample_rate: default_config.sample_rate().0,
                        };
                        devices.push(device_info);
                    }
                }
            }
        }

        info!("Found {} audio devices", devices.len());
        Ok(devices)
    }
}

// Audio capture for microphone input
pub struct MicrophoneCapture {
    inner: SystemAudioCapture,
}

impl MicrophoneCapture {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: SystemAudioCapture::new()?,
        })
    }
}

#[async_trait::async_trait]
impl AudioCapture for MicrophoneCapture {
    async fn start_capture(&self, config: AudioConfig) -> Result<mpsc::Receiver<Vec<u8>>> {
        self.inner.start_capture(config).await
    }

    async fn stop_capture(&self) -> Result<()> {
        self.inner.stop_capture().await
    }

    async fn get_device_list(&self) -> Result<Vec<AudioDeviceInfo>> {
        let mut devices = self.inner.get_device_list().await?;
        // Filter to only input devices
        devices.retain(|d| d.device_type == "input");
        Ok(devices)
    }
}

// Audio capture for system audio (what you hear)
pub struct SystemAudioOutputCapture {
    inner: SystemAudioCapture,
}

impl SystemAudioOutputCapture {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: SystemAudioCapture::new()?,
        })
    }
}

#[async_trait::async_trait]
impl AudioCapture for SystemAudioOutputCapture {
    async fn start_capture(&self, config: AudioConfig) -> Result<mpsc::Receiver<Vec<u8>>> {
        // For system audio capture, we need to use loopback recording
        // This is platform-specific and requires additional setup
        self.inner.start_capture(config).await
    }

    async fn stop_capture(&self) -> Result<()> {
        self.inner.stop_capture().await
    }

    async fn get_device_list(&self) -> Result<Vec<AudioDeviceInfo>> {
        let mut devices = self.inner.get_device_list().await?;
        // Filter to only output devices that support loopback
        devices.retain(|d| d.device_type == "output");
        Ok(devices)
    }
}
