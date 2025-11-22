use crate::ClientError;
use serde::{Serialize, Deserialize};

/// Hardware encoder manager for GPU-accelerated encoding
pub struct HardwareEncoder {
    encoder_type: HardwareEncoderType,
    enabled: bool,
    quality_preset: QualityPreset,
}

impl HardwareEncoder {
    /// Create a new hardware encoder
    pub fn new() -> Self {
        Self {
            encoder_type: Self::detect_available_encoder(),
            enabled: true,
            quality_preset: QualityPreset::Balanced,
        }
    }

    /// Detect available hardware encoder
    fn detect_available_encoder() -> HardwareEncoderType {
        #[cfg(target_os = "windows")]
        {
            // Check for NVIDIA NVENC
            if Self::is_nvenc_available() {
                return HardwareEncoderType::NVENC;
            }

            // Check for Intel Quick Sync
            if Self::is_quicksync_available() {
                return HardwareEncoderType::QuickSync;
            }

            // Check for AMD VCE
            if Self::is_amd_vce_available() {
                return HardwareEncoderType::AMDVCE;
            }
        }

        // Fallback to software encoding
        HardwareEncoderType::Software
    }

    #[cfg(target_os = "windows")]
    fn is_nvenc_available() -> bool {
        // Check for NVIDIA GPU and NVENC support
        // This would use NVML or similar to detect NVIDIA GPU
        // For now, return false (implement later with proper detection)
        false
    }

    #[cfg(target_os = "windows")]
    fn is_quicksync_available() -> bool {
        // Check for Intel GPU with Quick Sync support
        // This would check for Intel integrated graphics
        // For now, return false (implement later with proper detection)
        false
    }

    #[cfg(target_os = "windows")]
    fn is_amd_vce_available() -> bool {
        // Check for AMD GPU with VCE support
        // This would check for AMD graphics card
        // For now, return false (implement later with proper detection)
        false
    }

    /// Get current encoder type
    pub fn encoder_type(&self) -> HardwareEncoderType {
        self.encoder_type
    }

    /// Check if hardware encoding is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.encoder_type != HardwareEncoderType::Software
    }

    /// Enable hardware encoding
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable hardware encoding (fallback to software)
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Set quality preset
    pub fn set_quality_preset(&mut self, preset: QualityPreset) {
        self.quality_preset = preset;
    }

    /// Get quality preset
    pub fn quality_preset(&self) -> QualityPreset {
        self.quality_preset
    }

    /// Get encoder capabilities
    pub fn capabilities(&self) -> EncoderCapabilities {
        match self.encoder_type {
            HardwareEncoderType::NVENC => EncoderCapabilities {
                max_resolution: (7680, 4320), // 8K
                max_fps: 240,
                supports_hevc: true,
                supports_av1: true,
                latency_ms: 5,
            },
            HardwareEncoderType::QuickSync => EncoderCapabilities {
                max_resolution: (4096, 2160), // 4K
                max_fps: 120,
                supports_hevc: true,
                supports_av1: false,
                latency_ms: 8,
            },
            HardwareEncoderType::AMDVCE => EncoderCapabilities {
                max_resolution: (7680, 4320), // 8K
                max_fps: 240,
                supports_hevc: true,
                supports_av1: true,
                latency_ms: 6,
            },
            HardwareEncoderType::Software => EncoderCapabilities {
                max_resolution: (1920, 1080), // 1080p
                max_fps: 60,
                supports_hevc: false,
                supports_av1: false,
                latency_ms: 30,
            },
        }
    }

    /// Get encoder settings for current preset
    pub fn get_encoder_settings(&self) -> EncoderSettings {
        let base_settings = match self.quality_preset {
            QualityPreset::UltraLowLatency => EncoderSettings {
                bitrate_kbps: 2000,
                keyframe_interval: 15, // More frequent keyframes
                preset: "ultrafast".to_string(),
                tune: "zerolatency".to_string(),
                target_latency_ms: 5,
            },
            QualityPreset::LowLatency => EncoderSettings {
                bitrate_kbps: 3000,
                keyframe_interval: 30,
                preset: "veryfast".to_string(),
                tune: "zerolatency".to_string(),
                target_latency_ms: 10,
            },
            QualityPreset::Balanced => EncoderSettings {
                bitrate_kbps: 5000,
                keyframe_interval: 60,
                preset: "medium".to_string(),
                tune: "film".to_string(),
                target_latency_ms: 30,
            },
            QualityPreset::HighQuality => EncoderSettings {
                bitrate_kbps: 8000,
                keyframe_interval: 120,
                preset: "slow".to_string(),
                tune: "film".to_string(),
                target_latency_ms: 50,
            },
        };

        base_settings
    }
}

impl Default for HardwareEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Hardware encoder types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareEncoderType {
    /// NVIDIA NVENC
    NVENC,
    /// Intel Quick Sync
    QuickSync,
    /// AMD VCE
    AMDVCE,
    /// Software encoding (CPU)
    Software,
}

impl std::fmt::Display for HardwareEncoderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NVENC => write!(f, "NVIDIA NVENC"),
            Self::QuickSync => write!(f, "Intel Quick Sync"),
            Self::AMDVCE => write!(f, "AMD VCE"),
            Self::Software => write!(f, "Software (CPU)"),
        }
    }
}

/// Quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    /// Ultra-low latency (<10ms) - for gaming/real-time
    UltraLowLatency,
    /// Low latency (~10-20ms) - for interactive use
    LowLatency,
    /// Balanced quality and latency (~30-50ms)
    Balanced,
    /// High quality, higher latency (~50-100ms)
    HighQuality,
}

impl std::fmt::Display for QualityPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UltraLowLatency => write!(f, "Ultra-Low Latency (<10ms)"),
            Self::LowLatency => write!(f, "Low Latency (~10-20ms)"),
            Self::Balanced => write!(f, "Balanced (~30-50ms)"),
            Self::HighQuality => write!(f, "High Quality (~50-100ms)"),
        }
    }
}

/// Encoder capabilities
#[derive(Debug, Clone)]
pub struct EncoderCapabilities {
    pub max_resolution: (u32, u32),
    pub max_fps: u32,
    pub supports_hevc: bool,
    pub supports_av1: bool,
    pub latency_ms: u32,
}

/// Encoder settings
#[derive(Debug, Clone)]
pub struct EncoderSettings {
    pub bitrate_kbps: u32,
    pub keyframe_interval: u32,
    pub preset: String,
    pub tune: String,
    pub target_latency_ms: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_encoder() {
        let encoder = HardwareEncoder::new();
        assert!(encoder.encoder_type() == HardwareEncoderType::Software || 
                encoder.encoder_type() == HardwareEncoderType::NVENC ||
                encoder.encoder_type() == HardwareEncoderType::QuickSync ||
                encoder.encoder_type() == HardwareEncoderType::AMDVCE);
    }

    #[test]
    fn test_quality_presets() {
        let mut encoder = HardwareEncoder::new();
        
        encoder.set_quality_preset(QualityPreset::UltraLowLatency);
        let settings = encoder.get_encoder_settings();
        assert_eq!(settings.target_latency_ms, 5);
        
        encoder.set_quality_preset(QualityPreset::HighQuality);
        let settings = encoder.get_encoder_settings();
        assert_eq!(settings.target_latency_ms, 50);
    }

    #[test]
    fn test_encoder_capabilities() {
        let encoder = HardwareEncoder::new();
        let caps = encoder.capabilities();
        assert!(caps.max_fps > 0);
        assert!(caps.max_resolution.0 > 0);
    }
}
