use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Performance metrics for streaming
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub avg_frame_time: Duration,
    pub avg_encode_time: Duration,
    pub dropped_frames: u64,
    pub total_frames: u64,
    pub cpu_usage: f32,
    pub memory_usage: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            avg_frame_time: Duration::ZERO,
            avg_encode_time: Duration::ZERO,
            dropped_frames: 0,
            total_frames: 0,
            cpu_usage: 0.0,
            memory_usage: 0,
        }
    }
}

/// Performance monitor for tracking streaming performance
pub struct PerformanceMonitor {
    frame_times: VecDeque<Duration>,
    encode_times: VecDeque<Duration>,
    start_time: Instant,
    last_frame_time: Instant,
    total_frames: u64,
    dropped_frames: u64,
    max_samples: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(60),
            encode_times: VecDeque::with_capacity(60),
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            total_frames: 0,
            dropped_frames: 0,
            max_samples: 60, // Track last 60 frames
        }
    }

    /// Record a frame capture
    pub fn record_frame(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        
        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
        
        self.last_frame_time = now;
        self.total_frames += 1;
    }

    /// Record encoding time
    pub fn record_encode_time(&mut self, duration: Duration) {
        self.encode_times.push_back(duration);
        if self.encode_times.len() > self.max_samples {
            self.encode_times.pop_front();
        }
    }

    /// Record a dropped frame
    pub fn record_dropped_frame(&mut self) {
        self.dropped_frames += 1;
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let elapsed = self.start_time.elapsed();
        let fps = if elapsed.as_secs_f64() > 0.0 {
            self.total_frames as f64 / elapsed.as_secs_f64()
        } else if self.total_frames > 0 {
            // If elapsed time is very small but we have frames, estimate FPS
            self.total_frames as f64 * 1000.0
        } else {
            0.0
        };

        let avg_frame_time = if !self.frame_times.is_empty() {
            let sum: Duration = self.frame_times.iter().sum();
            sum / self.frame_times.len() as u32
        } else {
            Duration::ZERO
        };

        let avg_encode_time = if !self.encode_times.is_empty() {
            let sum: Duration = self.encode_times.iter().sum();
            sum / self.encode_times.len() as u32
        } else {
            Duration::ZERO
        };

        PerformanceMetrics {
            fps,
            avg_frame_time,
            avg_encode_time,
            dropped_frames: self.dropped_frames,
            total_frames: self.total_frames,
            cpu_usage: 0.0, // TODO: Implement CPU monitoring
            memory_usage: 0, // TODO: Implement memory monitoring
        }
    }

    /// Check if performance is acceptable
    pub fn is_performance_good(&self, target_fps: f64) -> bool {
        let metrics = self.get_metrics();
        metrics.fps >= target_fps * 0.9 && // Within 10% of target
        metrics.dropped_frames < metrics.total_frames / 100 // Less than 1% dropped
    }

    /// Get performance grade
    pub fn get_performance_grade(&self, target_fps: f64) -> PerformanceGrade {
        let metrics = self.get_metrics();
        let fps_ratio = metrics.fps / target_fps;
        let drop_ratio = metrics.dropped_frames as f64 / metrics.total_frames.max(1) as f64;

        if fps_ratio >= 0.95 && drop_ratio < 0.01 {
            PerformanceGrade::Excellent
        } else if fps_ratio >= 0.85 && drop_ratio < 0.05 {
            PerformanceGrade::Good
        } else if fps_ratio >= 0.70 && drop_ratio < 0.10 {
            PerformanceGrade::Fair
        } else {
            PerformanceGrade::Poor
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.encode_times.clear();
        self.start_time = Instant::now();
        self.last_frame_time = Instant::now();
        self.total_frames = 0;
        self.dropped_frames = 0;
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance grade
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl std::fmt::Display for PerformanceGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceGrade::Excellent => write!(f, "Excellent ⭐⭐⭐"),
            PerformanceGrade::Good => write!(f, "Good ⭐⭐"),
            PerformanceGrade::Fair => write!(f, "Fair ⭐"),
            PerformanceGrade::Poor => write!(f, "Poor ⚠️"),
        }
    }
}

/// Quality settings for adaptive streaming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

impl QualityPreset {
    /// Get recommended settings for this quality preset
    pub fn get_settings(&self) -> QualitySettings {
        match self {
            QualityPreset::Low => QualitySettings {
                width: 1280,
                height: 720,
                fps: 15,
                bitrate: 500_000, // 500 Kbps
            },
            QualityPreset::Medium => QualitySettings {
                width: 1920,
                height: 1080,
                fps: 30,
                bitrate: 2_000_000, // 2 Mbps
            },
            QualityPreset::High => QualitySettings {
                width: 1920,
                height: 1080,
                fps: 60,
                bitrate: 5_000_000, // 5 Mbps
            },
            QualityPreset::Ultra => QualitySettings {
                width: 2560,
                height: 1440,
                fps: 60,
                bitrate: 10_000_000, // 10 Mbps
            },
            QualityPreset::Custom => QualitySettings::default(),
        }
    }
}

/// Quality settings
#[derive(Debug, Clone, Copy)]
pub struct QualitySettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
}

impl Default for QualitySettings {
    fn default() -> Self {
        QualityPreset::Medium.get_settings()
    }
}

/// Adaptive quality controller
pub struct AdaptiveQualityController {
    current_preset: QualityPreset,
    current_settings: QualitySettings,
    monitor: PerformanceMonitor,
    adjustment_threshold: Duration,
    last_adjustment: Instant,
}

impl AdaptiveQualityController {
    /// Create a new adaptive quality controller
    pub fn new(initial_preset: QualityPreset) -> Self {
        let settings = initial_preset.get_settings();
        Self {
            current_preset: initial_preset,
            current_settings: settings,
            monitor: PerformanceMonitor::new(),
            adjustment_threshold: Duration::from_secs(5),
            last_adjustment: Instant::now(),
        }
    }

    /// Update performance metrics
    pub fn update_metrics(&mut self, _frame_time: Duration, encode_time: Duration) {
        self.monitor.record_frame();
        self.monitor.record_encode_time(encode_time);
    }

    /// Check if quality adjustment is needed
    pub fn should_adjust_quality(&mut self) -> Option<QualityPreset> {
        // Don't adjust too frequently
        if self.last_adjustment.elapsed() < self.adjustment_threshold {
            return None;
        }

        let metrics = self.monitor.get_metrics();
        let target_fps = self.current_settings.fps as f64;

        // If performance is poor, downgrade quality
        if metrics.fps < target_fps * 0.7 {
            self.last_adjustment = Instant::now();
            return Some(self.downgrade_quality());
        }

        // If performance is excellent, try upgrading
        if metrics.fps >= target_fps * 0.95 && 
           self.current_preset != QualityPreset::Ultra {
            self.last_adjustment = Instant::now();
            return Some(self.upgrade_quality());
        }

        None
    }

    /// Downgrade quality preset
    fn downgrade_quality(&mut self) -> QualityPreset {
        self.current_preset = match self.current_preset {
            QualityPreset::Ultra => QualityPreset::High,
            QualityPreset::High => QualityPreset::Medium,
            QualityPreset::Medium => QualityPreset::Low,
            QualityPreset::Low => QualityPreset::Low,
            QualityPreset::Custom => QualityPreset::Medium,
        };
        self.current_settings = self.current_preset.get_settings();
        self.current_preset
    }

    /// Upgrade quality preset
    fn upgrade_quality(&mut self) -> QualityPreset {
        self.current_preset = match self.current_preset {
            QualityPreset::Low => QualityPreset::Medium,
            QualityPreset::Medium => QualityPreset::High,
            QualityPreset::High => QualityPreset::Ultra,
            QualityPreset::Ultra => QualityPreset::Ultra,
            QualityPreset::Custom => QualityPreset::High,
        };
        self.current_settings = self.current_preset.get_settings();
        self.current_preset
    }

    /// Get current quality settings
    pub fn get_settings(&self) -> QualitySettings {
        self.current_settings
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.monitor.get_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        for _ in 0..30 {
            monitor.record_frame();
            monitor.record_encode_time(Duration::from_millis(10));
        }

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_frames, 30);
        assert!(metrics.fps > 0.0);
    }

    #[test]
    fn test_quality_presets() {
        let low = QualityPreset::Low.get_settings();
        assert_eq!(low.fps, 15);
        
        let high = QualityPreset::High.get_settings();
        assert_eq!(high.fps, 60);
    }

    #[test]
    fn test_adaptive_quality() {
        let mut controller = AdaptiveQualityController::new(QualityPreset::Medium);
        
        // Simulate poor performance
        for _ in 0..100 {
            controller.update_metrics(
                Duration::from_millis(100), // Slow frames
                Duration::from_millis(50),
            );
        }

        // Should suggest downgrade after threshold
        std::thread::sleep(Duration::from_secs(6));
        let adjustment = controller.should_adjust_quality();
        assert!(adjustment.is_some());
    }
}
