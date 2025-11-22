use serde::{Deserialize, Serialize};

/// GenX Secure Tunnel (GST) - Proprietary encrypted tunnel
/// Optimized for low latency, weak networks, and mobile data

/// GST tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GstConfig {
    pub enabled: bool,
    pub compression_level: CompressionLevel,
    pub encryption_mode: EncryptionMode,
    pub adaptive_quality: bool,
    pub mobile_optimization: bool,
    pub packet_loss_recovery: bool,
    pub ai_compression: bool,
}

impl Default for GstConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            compression_level: CompressionLevel::Adaptive,
            encryption_mode: EncryptionMode::Aes256Gcm,
            adaptive_quality: true,
            mobile_optimization: true,
            packet_loss_recovery: true,
            ai_compression: true,
        }
    }
}

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    None,
    Low,
    Medium,
    High,
    Adaptive,
}

impl CompressionLevel {
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None (No compression)",
            Self::Low => "Low (Fast, less compression)",
            Self::Medium => "Medium (Balanced)",
            Self::High => "High (Slow, max compression)",
            Self::Adaptive => "Adaptive (AI-based)",
        }
    }
    
    pub fn compression_ratio(&self) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Low => 0.7,
            Self::Medium => 0.5,
            Self::High => 0.3,
            Self::Adaptive => 0.4, // Average
        }
    }
}

/// Encryption mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionMode {
    Aes128Gcm,
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl EncryptionMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Aes128Gcm => "AES-128-GCM (Fast)",
            Self::Aes256Gcm => "AES-256-GCM (Secure)",
            Self::ChaCha20Poly1305 => "ChaCha20-Poly1305 (Mobile)",
        }
    }
}

/// Network condition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkCondition {
    Excellent,
    Good,
    Fair,
    Poor,
    VeryPoor,
}

impl NetworkCondition {
    pub fn from_metrics(latency_ms: u32, packet_loss: f32, bandwidth_kbps: u32) -> Self {
        // Excellent: <50ms, <1% loss, >5Mbps
        if latency_ms < 50 && packet_loss < 1.0 && bandwidth_kbps > 5000 {
            return Self::Excellent;
        }
        
        // Good: <100ms, <3% loss, >2Mbps
        if latency_ms < 100 && packet_loss < 3.0 && bandwidth_kbps > 2000 {
            return Self::Good;
        }
        
        // Fair: <200ms, <5% loss, >1Mbps
        if latency_ms < 200 && packet_loss < 5.0 && bandwidth_kbps > 1000 {
            return Self::Fair;
        }
        
        // Poor: <500ms, <10% loss, >500Kbps
        if latency_ms < 500 && packet_loss < 10.0 && bandwidth_kbps > 500 {
            return Self::Poor;
        }
        
        // Very Poor: Everything else
        Self::VeryPoor
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor",
            Self::VeryPoor => "Very Poor",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Excellent => "📶",
            Self::Good => "📶",
            Self::Fair => "📶",
            Self::Poor => "📶",
            Self::VeryPoor => "📵",
        }
    }
}

/// GST tunnel manager
pub struct GstTunnelManager {
    config: GstConfig,
    is_active: bool,
    network_condition: NetworkCondition,
    bytes_sent: u64,
    bytes_received: u64,
    bytes_saved: u64, // Bytes saved by compression
}

impl Default for GstTunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GstTunnelManager {
    pub fn new() -> Self {
        Self {
            config: GstConfig::default(),
            is_active: false,
            network_condition: NetworkCondition::Good,
            bytes_sent: 0,
            bytes_received: 0,
            bytes_saved: 0,
        }
    }
    
    /// Start GST tunnel
    pub fn start(&mut self) -> Result<(), String> {
        if self.is_active {
            return Err("GST tunnel already active".to_string());
        }
        
        // TODO: Initialize tunnel
        // - Set up encryption
        // - Initialize compression
        // - Start AI compression if enabled
        
        self.is_active = true;
        tracing::info!("GST tunnel started");
        Ok(())
    }
    
    /// Stop GST tunnel
    pub fn stop(&mut self) {
        self.is_active = false;
        tracing::info!("GST tunnel stopped");
    }
    
    /// Check if tunnel is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &GstConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: GstConfig) {
        self.config = config;
    }
    
    /// Update network condition
    pub fn update_network_condition(&mut self, latency_ms: u32, packet_loss: f32, bandwidth_kbps: u32) {
        self.network_condition = NetworkCondition::from_metrics(latency_ms, packet_loss, bandwidth_kbps);
        
        // Adjust compression based on network condition
        if self.config.adaptive_quality {
            self.adjust_compression();
        }
    }
    
    /// Adjust compression based on network condition
    fn adjust_compression(&mut self) {
        match self.network_condition {
            NetworkCondition::Excellent | NetworkCondition::Good => {
                // Good network - use less compression for lower latency
                self.config.compression_level = CompressionLevel::Low;
            }
            NetworkCondition::Fair => {
                // Fair network - balanced compression
                self.config.compression_level = CompressionLevel::Medium;
            }
            NetworkCondition::Poor | NetworkCondition::VeryPoor => {
                // Poor network - max compression to save bandwidth
                self.config.compression_level = CompressionLevel::High;
            }
        }
    }
    
    /// Get network condition
    pub fn network_condition(&self) -> NetworkCondition {
        self.network_condition
    }
    
    /// Compress data (simulated)
    pub fn compress_data(&mut self, data: &[u8]) -> Vec<u8> {
        let ratio = self.config.compression_level.compression_ratio();
        let compressed_size = (data.len() as f32 * ratio) as usize;
        
        self.bytes_sent += compressed_size as u64;
        self.bytes_saved += (data.len() - compressed_size) as u64;
        
        // TODO: Actual compression implementation
        // For now, return original data
        data.to_vec()
    }
    
    /// Decompress data (simulated)
    pub fn decompress_data(&mut self, data: &[u8]) -> Vec<u8> {
        self.bytes_received += data.len() as u64;
        
        // TODO: Actual decompression implementation
        data.to_vec()
    }
    
    /// Get compression statistics
    pub fn get_stats(&self) -> GstStats {
        GstStats {
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            bytes_saved: self.bytes_saved,
            compression_ratio: if self.bytes_sent > 0 {
                (self.bytes_saved as f32 / (self.bytes_sent + self.bytes_saved) as f32) * 100.0
            } else {
                0.0
            },
            network_condition: self.network_condition,
        }
    }
}

/// GST statistics
#[derive(Debug, Clone)]
pub struct GstStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub bytes_saved: u64,
    pub compression_ratio: f32,
    pub network_condition: NetworkCondition,
}

impl GstStats {
    pub fn format_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_condition() {
        let excellent = NetworkCondition::from_metrics(30, 0.5, 10000);
        assert_eq!(excellent, NetworkCondition::Excellent);
        
        let poor = NetworkCondition::from_metrics(400, 8.0, 600);
        assert_eq!(poor, NetworkCondition::Poor);
    }
    
    #[test]
    fn test_gst_tunnel() {
        let mut tunnel = GstTunnelManager::new();
        assert!(!tunnel.is_active());
        
        assert!(tunnel.start().is_ok());
        assert!(tunnel.is_active());
        
        tunnel.stop();
        assert!(!tunnel.is_active());
    }
    
    #[test]
    fn test_adaptive_compression() {
        let mut tunnel = GstTunnelManager::new();
        tunnel.config.adaptive_quality = true;
        
        // Good network - should use low compression
        tunnel.update_network_condition(50, 1.0, 5000);
        assert_eq!(tunnel.config.compression_level, CompressionLevel::Low);
        
        // Poor network - should use high compression
        tunnel.update_network_condition(400, 8.0, 600);
        assert_eq!(tunnel.config.compression_level, CompressionLevel::High);
    }
}
