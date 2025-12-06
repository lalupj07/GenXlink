/// GenXLink Streaming Demo
/// 
/// This example demonstrates the complete video streaming pipeline:
/// 1. Screen capture with DXGI
/// 2. H.264 video encoding
/// 3. RTP streaming over WebRTC
/// 
/// Run with: cargo run --example streaming_demo

use genxlink_client_core::{
    encoder::{H264Encoder, EncoderConfig, VideoCodec, VideoEncoder},
    streaming::{StreamingPipeline, Frame},
    webrtc::{WebRTCManager, WebRTCConfig},
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║          GenXLink Streaming Demo                          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Step 1: Initialize encoder
    println!("📹 Step 1: Initializing H.264 encoder...");
    let mut encoder = Box::new(H264Encoder::new());
    
    let config = EncoderConfig {
        width: 1920,
        height: 1080,
        fps: 30,
        bitrate: 2_000_000, // 2 Mbps
        codec: VideoCodec::H264,
    };
    
    encoder.init(config)?;
    println!("   ✅ Encoder initialized: 1920x1080 @ 30 FPS, 2 Mbps\n");

    // Step 2: Create streaming pipeline
    println!("🌐 Step 2: Creating streaming pipeline...");
    let mut pipeline = StreamingPipeline::new(encoder, 30)?;
    println!("   ✅ Streaming pipeline created\n");

    // Step 3: Initialize WebRTC
    println!("🔗 Step 3: Initializing WebRTC connection...");
    let webrtc_config = WebRTCConfig::default();
    let mut webrtc = WebRTCManager::new("demo-device".to_string(), webrtc_config);
    webrtc.initialize().await?;
    println!("   ✅ WebRTC initialized\n");

    // Step 4: Add video track
    println!("📡 Step 4: Adding video track to WebRTC...");
    let track = pipeline.get_track();
    webrtc.add_video_track(track).await?;
    println!("   ✅ Video track added\n");

    // Step 5: Simulate streaming
    println!("🎥 Step 5: Simulating video streaming...");
    println!("   (In production, this would capture and encode real frames)\n");
    
    for i in 0..10 {
        // Create dummy frame for streaming
        let frame = Frame {
            width: 1920,
            height: 1080,
            data: vec![0u8; 1920 * 1080 * 4], // RGBA
            timestamp: std::time::Instant::now(),
        };
        
        pipeline.stream_frame(&frame).await?;
        
        if i % 3 == 0 {
            let stats = pipeline.get_stats();
            println!("   📊 Frame {}: {} packets sent, timestamp: {}", 
                i, stats.packets_sent, stats.timestamp);
        }
        
        sleep(Duration::from_millis(33)).await;
    }

    println!("\n✅ Demo completed successfully!\n");

    // Display statistics
    let stats = pipeline.get_stats();
    println!("📈 Final Statistics:");
    println!("   Total packets sent: {}", stats.packets_sent);
    println!("   Final timestamp: {}", stats.timestamp);
    println!("   SSRC: {}", stats.ssrc);
    println!("   Frame rate: 30 FPS");
    println!("   Bitrate: 2 Mbps\n");

    println!("🎉 GenXLink streaming pipeline is working!\n");
    println!("Next steps:");
    println!("   • Connect to actual screen capture");
    println!("   • Establish peer-to-peer connection");
    println!("   • Test with remote client");
    println!("   • Optimize performance\n");

    Ok(())
}
