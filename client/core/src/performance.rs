use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Performance metrics tracker
pub struct PerformanceMonitor {
    frame_times: VecDeque<Duration>,
    max_samples: usize,
    last_frame: Option<Instant>,
    total_frames: u64,
    dropped_frames: u64,
}

impl PerformanceMonitor {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_samples),
            max_samples,
            last_frame: None,
            total_frames: 0,
            dropped_frames: 0,
        }
    }
    
    /// Record a new frame
    pub fn record_frame(&mut self) {
        let now = Instant::now();
        
        if let Some(last) = self.last_frame {
            let duration = now.duration_since(last);
            
            if self.frame_times.len() >= self.max_samples {
                self.frame_times.pop_front();
            }
            
            self.frame_times.push_back(duration);
        }
        
        self.last_frame = Some(now);
        self.total_frames += 1;
    }
    
    /// Record a dropped frame
    pub fn record_dropped_frame(&mut self) {
        self.dropped_frames += 1;
    }
    
    /// Get average FPS
    pub fn get_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        let avg = total / self.frame_times.len() as u32;
        
        if avg.as_secs_f64() > 0.0 {
            1.0 / avg.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get average frame time in milliseconds
    pub fn get_avg_frame_time_ms(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        let avg = total / self.frame_times.len() as u32;
        avg.as_secs_f64() * 1000.0
    }
    
    /// Get minimum frame time
    pub fn get_min_frame_time_ms(&self) -> f64 {
        self.frame_times
            .iter()
            .min()
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }
    
    /// Get maximum frame time
    pub fn get_max_frame_time_ms(&self) -> f64 {
        self.frame_times
            .iter()
            .max()
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }
    
    /// Get total frames captured
    pub fn get_total_frames(&self) -> u64 {
        self.total_frames
    }
    
    /// Get dropped frames
    pub fn get_dropped_frames(&self) -> u64 {
        self.dropped_frames
    }
    
    /// Get drop rate percentage
    pub fn get_drop_rate(&self) -> f64 {
        if self.total_frames == 0 {
            return 0.0;
        }
        
        (self.dropped_frames as f64 / self.total_frames as f64) * 100.0
    }
    
    /// Print performance stats
    pub fn print_stats(&self) {
        println!("=== Performance Stats ===");
        println!("FPS: {:.2}", self.get_fps());
        println!("Avg Frame Time: {:.2} ms", self.get_avg_frame_time_ms());
        println!("Min Frame Time: {:.2} ms", self.get_min_frame_time_ms());
        println!("Max Frame Time: {:.2} ms", self.get_max_frame_time_ms());
        println!("Total Frames: {}", self.total_frames);
        println!("Dropped Frames: {} ({:.2}%)", self.dropped_frames, self.get_drop_rate());
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.last_frame = None;
        self.total_frames = 0;
        self.dropped_frames = 0;
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new(100) // Keep last 100 frame times
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new(10);
        
        // Simulate 30 FPS
        for _ in 0..10 {
            monitor.record_frame();
            thread::sleep(Duration::from_millis(33)); // ~30 FPS
        }
        
        let fps = monitor.get_fps();
        assert!(fps > 25.0 && fps < 35.0, "FPS should be around 30");
        
        let frame_time = monitor.get_avg_frame_time_ms();
        assert!(frame_time > 30.0 && frame_time < 40.0, "Frame time should be around 33ms");
    }
    
    #[test]
    fn test_dropped_frames() {
        let mut monitor = PerformanceMonitor::new(10);
        
        monitor.record_frame();
        monitor.record_frame();
        monitor.record_dropped_frame();
        monitor.record_frame();
        
        assert_eq!(monitor.get_total_frames(), 3);
        assert_eq!(monitor.get_dropped_frames(), 1);
        assert!(monitor.get_drop_rate() > 0.0);
    }
}
