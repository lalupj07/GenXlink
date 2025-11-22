use crate::ClientError;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Adaptive bitrate controller
pub struct AdaptiveBitrateController {
    current_bitrate: u32,
    min_bitrate: u32,
    max_bitrate: u32,
    target_bitrate: u32,
    
    // Network metrics
    rtt_samples: VecDeque<Duration>,
    packet_loss_samples: VecDeque<f32>,
    bandwidth_samples: VecDeque<u32>,
    
    // Adjustment parameters
    adjustment_interval: Duration,
    last_adjustment: Instant,
    
    // Thresholds
    rtt_threshold_ms: u32,
    packet_loss_threshold: f32,
}

impl AdaptiveBitrateController {
    /// Create a new adaptive bitrate controller
    pub fn new(initial_bitrate: u32) -> Self {
        Self {
            current_bitrate: initial_bitrate,
            min_bitrate: 500,      // 500 kbps minimum
            max_bitrate: 20000,    // 20 Mbps maximum
            target_bitrate: initial_bitrate,
            
            rtt_samples: VecDeque::with_capacity(10),
            packet_loss_samples: VecDeque::with_capacity(10),
            bandwidth_samples: VecDeque::with_capacity(10),
            
            adjustment_interval: Duration::from_secs(1),
            last_adjustment: Instant::now(),
            
            rtt_threshold_ms: 100,
            packet_loss_threshold: 0.05, // 5%
        }
    }

    /// Update network metrics
    pub fn update_metrics(&mut self, rtt: Duration, packet_loss: f32, bandwidth: u32) {
        // Add samples
        self.rtt_samples.push_back(rtt);
        self.packet_loss_samples.push_back(packet_loss);
        self.bandwidth_samples.push_back(bandwidth);
        
        // Keep only recent samples
        if self.rtt_samples.len() > 10 {
            self.rtt_samples.pop_front();
        }
        if self.packet_loss_samples.len() > 10 {
            self.packet_loss_samples.pop_front();
        }
        if self.bandwidth_samples.len() > 10 {
            self.bandwidth_samples.pop_front();
        }
    }

    /// Adjust bitrate based on network conditions
    pub fn adjust_bitrate(&mut self) -> Option<u32> {
        // Check if enough time has passed
        if self.last_adjustment.elapsed() < self.adjustment_interval {
            return None;
        }
        
        self.last_adjustment = Instant::now();
        
        // Calculate average metrics
        let avg_rtt = self.average_rtt();
        let avg_packet_loss = self.average_packet_loss();
        let avg_bandwidth = self.average_bandwidth();
        
        // Determine adjustment
        let new_bitrate = if avg_packet_loss > self.packet_loss_threshold {
            // High packet loss - reduce bitrate aggressively
            (self.current_bitrate as f32 * 0.8) as u32
        } else if avg_rtt.as_millis() > self.rtt_threshold_ms as u128 {
            // High RTT - reduce bitrate moderately
            (self.current_bitrate as f32 * 0.9) as u32
        } else if avg_packet_loss < 0.01 && avg_rtt.as_millis() < 50 {
            // Good conditions - increase bitrate gradually
            let increase = (self.current_bitrate as f32 * 0.1) as u32;
            let max_increase = (avg_bandwidth as f32 * 0.8) as u32;
            self.current_bitrate + increase.min(max_increase)
        } else {
            // Stable conditions - maintain current bitrate
            self.current_bitrate
        };
        
        // Clamp to min/max
        self.current_bitrate = new_bitrate.clamp(self.min_bitrate, self.max_bitrate);
        
        Some(self.current_bitrate)
    }

    /// Get current bitrate
    pub fn current_bitrate(&self) -> u32 {
        self.current_bitrate
    }

    /// Set bitrate limits
    pub fn set_limits(&mut self, min: u32, max: u32) {
        self.min_bitrate = min;
        self.max_bitrate = max;
        self.current_bitrate = self.current_bitrate.clamp(min, max);
    }

    /// Get network quality score (0-100)
    pub fn network_quality_score(&self) -> u8 {
        let rtt_score = self.rtt_score();
        let loss_score = self.packet_loss_score();
        let bandwidth_score = self.bandwidth_score();
        
        ((rtt_score + loss_score + bandwidth_score) / 3.0) as u8
    }

    fn average_rtt(&self) -> Duration {
        if self.rtt_samples.is_empty() {
            return Duration::from_millis(50);
        }
        
        let sum: Duration = self.rtt_samples.iter().sum();
        sum / self.rtt_samples.len() as u32
    }

    fn average_packet_loss(&self) -> f32 {
        if self.packet_loss_samples.is_empty() {
            return 0.0;
        }
        
        self.packet_loss_samples.iter().sum::<f32>() / self.packet_loss_samples.len() as f32
    }

    fn average_bandwidth(&self) -> u32 {
        if self.bandwidth_samples.is_empty() {
            return 5000; // 5 Mbps default
        }
        
        self.bandwidth_samples.iter().sum::<u32>() / self.bandwidth_samples.len() as u32
    }

    fn rtt_score(&self) -> f32 {
        let avg_rtt_ms = self.average_rtt().as_millis() as f32;
        
        if avg_rtt_ms < 20.0 {
            100.0
        } else if avg_rtt_ms < 50.0 {
            90.0
        } else if avg_rtt_ms < 100.0 {
            70.0
        } else if avg_rtt_ms < 200.0 {
            50.0
        } else {
            20.0
        }
    }

    fn packet_loss_score(&self) -> f32 {
        let avg_loss = self.average_packet_loss();
        
        if avg_loss < 0.01 {
            100.0
        } else if avg_loss < 0.05 {
            80.0
        } else if avg_loss < 0.10 {
            50.0
        } else {
            20.0
        }
    }

    fn bandwidth_score(&self) -> f32 {
        let avg_bandwidth = self.average_bandwidth();
        
        if avg_bandwidth > 10000 {
            100.0
        } else if avg_bandwidth > 5000 {
            80.0
        } else if avg_bandwidth > 2000 {
            60.0
        } else if avg_bandwidth > 1000 {
            40.0
        } else {
            20.0
        }
    }
}

impl Default for AdaptiveBitrateController {
    fn default() -> Self {
        Self::new(5000) // 5 Mbps default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_bitrate() {
        let mut controller = AdaptiveBitrateController::new(5000);
        assert_eq!(controller.current_bitrate(), 5000);
    }

    #[test]
    fn test_bitrate_adjustment() {
        let mut controller = AdaptiveBitrateController::new(5000);
        
        // Simulate good network conditions
        controller.update_metrics(Duration::from_millis(20), 0.001, 10000);
        std::thread::sleep(Duration::from_secs(1));
        
        if let Some(new_bitrate) = controller.adjust_bitrate() {
            assert!(new_bitrate >= 5000); // Should increase or stay same
        }
    }

    #[test]
    fn test_network_quality_score() {
        let mut controller = AdaptiveBitrateController::new(5000);
        
        // Good conditions
        controller.update_metrics(Duration::from_millis(20), 0.001, 10000);
        let score = controller.network_quality_score();
        assert!(score > 80);
    }

    #[test]
    fn test_bitrate_limits() {
        let mut controller = AdaptiveBitrateController::new(5000);
        controller.set_limits(1000, 8000);
        
        assert!(controller.current_bitrate() >= 1000);
        assert!(controller.current_bitrate() <= 8000);
    }
}
