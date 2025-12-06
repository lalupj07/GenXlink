/// Screen capture test example
/// 
/// This example demonstrates the Windows DXGI screen capture functionality.
/// It captures frames for 10 seconds and displays performance metrics.
/// 
/// Run with: cargo run --example screen_capture_test

use genxlink_client_core::{
    capture::{ScreenCapture},
    capture::win_impl::DxgiCapture,
    performance::PerformanceMonitor,
};
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GenXLink Screen Capture Test ===\n");
    
    // Create screen capture
    println!("Initializing screen capture...");
    let mut capture = DxgiCapture::new();
    
    // Initialize
    capture.init().await?;
    
    let (width, height) = capture.get_dimensions();
    println!("Screen resolution: {}x{}", width, height);
    println!("Starting capture for 10 seconds...\n");
    
    // Performance monitor
    let mut perf = PerformanceMonitor::new(100);
    
    // Capture frames for 10 seconds
    let start = std::time::Instant::now();
    let duration = Duration::from_secs(10);
    
    let mut frame_count = 0;
    let mut error_count = 0;
    
    while start.elapsed() < duration {
        match capture.capture_frame().await {
            Ok(frame) => {
                frame_count += 1;
                perf.record_frame();
                
                // Print progress every second
                if frame_count % 30 == 0 {
                    println!("Captured {} frames... FPS: {:.2}", frame_count, perf.get_fps());
                }
                
                // Verify frame data
                if frame.data.is_empty() {
                    println!("Warning: Empty frame data");
                }
            }
            Err(e) => {
                error_count += 1;
                perf.record_dropped_frame();
                
                // Don't spam errors
                if error_count < 10 {
                    eprintln!("Frame capture error: {}", e);
                }
                
                // Small delay on error
                time::sleep(Duration::from_millis(10)).await;
            }
        }
        
        // Target 30 FPS
        time::sleep(Duration::from_millis(33)).await;
    }
    
    // Cleanup
    capture.cleanup().await?;
    
    // Print final stats
    println!("\n=== Test Complete ===");
    println!("Total frames captured: {}", frame_count);
    println!("Errors encountered: {}", error_count);
    println!();
    perf.print_stats();
    
    // Evaluate results
    println!("\n=== Evaluation ===");
    let fps = perf.get_fps();
    let drop_rate = perf.get_drop_rate();
    
    if fps >= 25.0 && drop_rate < 5.0 {
        println!("✓ PASS: Screen capture working well!");
    } else if fps >= 15.0 && drop_rate < 10.0 {
        println!("⚠ PARTIAL: Screen capture working but needs optimization");
    } else {
        println!("✗ FAIL: Screen capture needs attention");
    }
    
    Ok(())
}
