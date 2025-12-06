use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use serde_json;

// Import GenXLink modules
use genxlink_webrtc::{
    MediaManager, ScreenCapture, AudioCapture, FileTransferManager,
    MediaStream, MediaQuality, CaptureConfig,
};
use genxlink_crypto::{
    EndToEndEncryption, EncryptionKeyPair, SessionKey,
};
use genxlink_protocol::{
    SessionMessage, MediaType, TransferStatus,
};

#[cfg(test)]
mod webrtc_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_screen_sharing_end_to_end() -> Result<()> {
        info!("Starting screen sharing end-to-end test");
        
        // Initialize media manager
        let media_manager = Arc::new(MediaManager::new().await?);
        
        // Setup screen capture configuration
        let capture_config = CaptureConfig {
            quality: MediaQuality::High,
            frame_rate: 30,
            resolution: (1920, 1080),
            compression_enabled: true,
            cursor_capture: true,
        };
        
        // Start screen capture
        let session_id = Uuid::new_v4();
        let screen_stream = media_manager.start_screen_capture(session_id, capture_config).await?;
        
        // Verify stream is active
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(screen_stream.is_active());
        
        // Capture some frames
        let mut frame_count = 0;
        let timeout = Duration::from_secs(5);
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout && frame_count < 10 {
            if let Some(frame) = screen_stream.get_next_frame().await? {
                assert!(!frame.data.is_empty());
                assert!(frame.width > 0);
                assert!(frame.height > 0);
                frame_count += 1;
                debug!("Captured frame {} ({}x{})", frame_count, frame.width, frame.height);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        assert!(frame_count > 0, "Should have captured at least one frame");
        
        // Stop screen capture
        media_manager.stop_screen_capture(session_id).await?;
        
        info!("Screen sharing test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_audio_streaming_end_to_end() -> Result<()> {
        info!("Starting audio streaming end-to-end test");
        
        // Initialize media manager
        let media_manager = Arc::new(MediaManager::new().await?);
        
        // Setup audio capture configuration
        let capture_config = CaptureConfig {
            quality: MediaQuality::High,
            sample_rate: 48000,
            channels: 2,
            bitrate: 128000,
            noise_cancellation: true,
            echo_cancellation: true,
        };
        
        // Start audio capture
        let session_id = Uuid::new_v4();
        let audio_stream = media_manager.start_audio_capture(session_id, capture_config).await?;
        
        // Verify stream is active
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(audio_stream.is_active());
        
        // Capture some audio samples
        let mut sample_count = 0;
        let timeout = Duration::from_secs(3);
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout && sample_count < 100 {
            if let Some(audio_data) = audio_stream.get_next_samples().await? {
                assert!(!audio_data.data.is_empty());
                assert!(audio_data.sample_rate > 0);
                assert!(audio_data.channels > 0);
                sample_count += audio_data.samples_count;
                debug!("Captured {} audio samples", audio_data.samples_count);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        assert!(sample_count > 0, "Should have captured at least one audio sample");
        
        // Stop audio capture
        media_manager.stop_audio_capture(session_id).await?;
        
        info!("Audio streaming test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_file_transfer_end_to_end() -> Result<()> {
        info!("Starting file transfer end-to-end test");
        
        // Initialize file transfer manager
        let file_manager = Arc::new(FileTransferManager::new().await?);
        
        // Create test file
        let test_content = b"This is test file content for GenXLink file transfer testing. It contains enough data to test compression and encryption features.";
        let test_file_path = "test_transfer_file.txt";
        std::fs::write(test_file_path, test_content)?;
        
        // Start file upload
        let session_id = Uuid::new_v4();
        let upload_session = file_manager.start_upload(
            session_id,
            test_file_path.to_string(),
            "test_file.txt".to_string(),
        ).await?;
        
        // Monitor upload progress
        let mut progress_updates = 0;
        let timeout = Duration::from_secs(10);
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            let progress = file_manager.get_upload_progress(session_id).await?;
            debug!("Upload progress: {}%", progress.percentage);
            
            if progress.status == TransferStatus::Completed {
                break;
            }
            
            if progress.status == TransferStatus::Failed {
                return Err(anyhow::anyhow!("Upload failed: {:?}", progress.error));
            }
            
            progress_updates += 1;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Verify upload completed
        let final_progress = file_manager.get_upload_progress(session_id).await?;
        assert_eq!(final_progress.status, TransferStatus::Completed);
        assert_eq!(final_progress.percentage, 100.0);
        
        // Test file download
        let download_session = file_manager.start_download(
            Uuid::new_v4(),
            final_progress.file_id.unwrap(),
        ).await?;
        
        // Monitor download progress
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            let progress = file_manager.get_download_progress(download_session.session_id).await?;
            debug!("Download progress: {}%", progress.percentage);
            
            if progress.status == TransferStatus::Completed {
                break;
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Verify download completed
        let download_progress = file_manager.get_download_progress(download_session.session_id).await?;
        assert_eq!(download_progress.status, TransferStatus::Completed);
        
        // Verify file integrity
        let downloaded_content = std::fs::read(&download_progress.local_path.unwrap())?;
        assert_eq!(downloaded_content, test_content);
        
        // Cleanup
        std::fs::remove_file(test_file_path)?;
        if let Some(path) = download_progress.local_path {
            std::fs::remove_file(path)?;
        }
        
        info!("File transfer test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_encrypted_media_stream() -> Result<()> {
        info!("Starting encrypted media stream test");
        
        // Initialize encryption
        let encryption = Arc::new(EndToEndEncryption::new());
        
        // Generate key pairs for two peers
        let peer1_keys = encryption.generate_key_pair().await?;
        let peer2_keys = encryption.generate_key_pair().await?;
        
        // Establish session
        let session_key = encryption.establish_session(
            &peer1_keys,
            &peer2_keys.public_key,
        ).await?;
        
        // Initialize media manager with encryption
        let media_manager = Arc::new(MediaManager::new().await?);
        
        // Start encrypted screen capture
        let session_id = Uuid::new_v4();
        let capture_config = CaptureConfig {
            quality: MediaQuality::Medium,
            frame_rate: 15,
            resolution: (1280, 720),
            compression_enabled: true,
            cursor_capture: false,
        };
        
        let screen_stream = media_manager.start_screen_capture(session_id, capture_config).await?;
        
        // Capture and encrypt frames
        let mut encrypted_frames = 0;
        let timeout = Duration::from_secs(3);
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout && encrypted_frames < 5 {
            if let Some(frame) = screen_stream.get_next_frame().await? {
                // Serialize frame
                let frame_data = serde_json::to_vec(&frame)?;
                
                // Encrypt frame data
                let encrypted_data = encryption.encrypt_message(&session_key, &frame_data).await?;
                
                // Decrypt frame data
                let decrypted_data = encryption.decrypt_message(&session_key, &encrypted_data).await?;
                
                // Verify integrity
                assert_eq!(frame_data, decrypted_data);
                
                encrypted_frames += 1;
                debug!("Encrypted/decrypted frame {} successfully", encrypted_frames);
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        
        assert!(encrypted_frames > 0, "Should have encrypted at least one frame");
        
        // Stop screen capture
        media_manager.stop_screen_capture(session_id).await?;
        
        info!("Encrypted media stream test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_multi_session_concurrent() -> Result<()> {
        info!("Starting multi-session concurrent test");
        
        // Initialize media manager
        let media_manager = Arc::new(MediaManager::new().await?);
        
        // Start multiple concurrent sessions
        let session_count = 3;
        let mut sessions = Vec::new();
        let mut handles = Vec::new();
        
        for i in 0..session_count {
            let session_id = Uuid::new_v4();
            let manager_clone = media_manager.clone();
            
            // Start screen capture for each session
            let capture_config = CaptureConfig {
                quality: MediaQuality::Medium,
                frame_rate: 10,
                resolution: (800, 600),
                compression_enabled: true,
                cursor_capture: false,
            };
            
            let stream = manager_clone.start_screen_capture(session_id, capture_config).await?;
            sessions.push((session_id, stream));
            
            // Spawn task to monitor each session
            let handle = tokio::spawn(async move {
                let mut frame_count = 0;
                let timeout = Duration::from_secs(2);
                let start_time = std::time::Instant::now();
                
                while start_time.elapsed() < timeout && frame_count < 5 {
                    if let Some(_) = stream.get_next_frame().await.unwrap_or(None) {
                        frame_count += 1;
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                
                (session_id, frame_count)
            });
            
            handles.push(handle);
        }
        
        // Wait for all sessions to complete
        let mut successful_sessions = 0;
        for handle in handles {
            let (session_id, frame_count) = handle.await?;
            if frame_count > 0 {
                successful_sessions += 1;
                debug!("Session {} captured {} frames", session_id, frame_count);
            }
        }
        
        // Stop all sessions
        for (session_id, _) in sessions {
            media_manager.stop_screen_capture(session_id).await?;
        }
        
        assert!(successful_sessions > 0, "At least one session should be successful");
        
        info!("Multi-session concurrent test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_media_quality_adaptation() -> Result<()> {
        info!("Starting media quality adaptation test");
        
        // Initialize media manager
        let media_manager = Arc::new(MediaManager::new().await?);
        
        // Test different quality levels
        let quality_levels = vec![
            (MediaQuality::Low, (640, 480), 15),
            (MediaQuality::Medium, (1280, 720), 30),
            (MediaQuality::High, (1920, 1080), 60),
        ];
        
        for (quality, resolution, frame_rate) in quality_levels {
            let session_id = Uuid::new_v4();
            let capture_config = CaptureConfig {
                quality: quality.clone(),
                frame_rate,
                resolution,
                compression_enabled: true,
                cursor_capture: false,
            };
            
            let stream = media_manager.start_screen_capture(session_id, capture_config).await?;
            
            // Capture a few frames to verify quality
            let mut frames_captured = 0;
            let timeout = Duration::from_secs(2);
            let start_time = std::time::Instant::now();
            
            while start_time.elapsed() < timeout && frames_captured < 3 {
                if let Some(frame) = stream.get_next_frame().await? {
                    assert!(!frame.data.is_empty());
                    frames_captured += 1;
                    debug!("Quality {:?}: Captured frame {} ({}x{})", 
                           quality, frames_captured, frame.width, frame.height);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            assert!(frames_captured > 0, "Should capture frames with quality {:?}", quality);
            
            media_manager.stop_screen_capture(session_id).await?;
        }
        
        info!("Media quality adaptation test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() -> Result<()> {
        info!("Starting error handling and recovery test");
        
        // Initialize media manager
        let media_manager = Arc::new(MediaManager::new().await?);
        
        // Test with invalid configuration
        let invalid_config = CaptureConfig {
            quality: MediaQuality::High,
            frame_rate: 0, // Invalid frame rate
            resolution: (0, 0), // Invalid resolution
            compression_enabled: true,
            cursor_capture: false,
        };
        
        let session_id = Uuid::new_v4();
        
        // Should handle invalid configuration gracefully
        let result = media_manager.start_screen_capture(session_id, invalid_config).await;
        assert!(result.is_err(), "Should fail with invalid configuration");
        
        // Test stopping non-existent session
        let non_existent_session = Uuid::new_v4();
        let result = media_manager.stop_screen_capture(non_existent_session).await;
        // Should not panic, but may return error or succeed gracefully
        debug!("Stop non-existent session result: {:?}", result);
        
        // Test recovery with valid configuration
        let valid_config = CaptureConfig {
            quality: MediaQuality::Medium,
            frame_rate: 15,
            resolution: (800, 600),
            compression_enabled: true,
            cursor_capture: false,
        };
        
        let stream = media_manager.start_screen_capture(session_id, valid_config).await?;
        assert!(stream.is_active());
        
        media_manager.stop_screen_capture(session_id).await?;
        
        info!("Error handling and recovery test completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_screen_capture_performance() -> Result<()> {
        info!("Starting screen capture performance test");
        
        let media_manager = Arc::new(MediaManager::new().await?);
        
        let capture_config = CaptureConfig {
            quality: MediaQuality::High,
            frame_rate: 30,
            resolution: (1920, 1080),
            compression_enabled: true,
            cursor_capture: false,
        };
        
        let session_id = Uuid::new_v4();
        let stream = media_manager.start_screen_capture(session_id, capture_config).await?;
        
        // Measure performance over 10 seconds
        let test_duration = Duration::from_secs(10);
        let start_time = Instant::now();
        let mut frame_count = 0;
        let mut total_size = 0;
        
        while start_time.elapsed() < test_duration {
            if let Some(frame) = stream.get_next_frame().await? {
                frame_count += 1;
                total_size += frame.data.len();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        let elapsed = start_time.elapsed();
        let fps = frame_count as f64 / elapsed.as_secs_f64();
        let throughput_mbps = (total_size as f64 / elapsed.as_secs_f64()) / (1024.0 * 1024.0);
        
        info!("Performance results:");
        info!("  Frames captured: {}", frame_count);
        info!("  FPS: {:.2}", fps);
        info!("  Throughput: {:.2} MB/s", throughput_mbps);
        
        // Performance assertions
        assert!(fps > 10.0, "Should maintain at least 10 FPS");
        assert!(frame_count > 0, "Should capture frames");
        
        media_manager.stop_screen_capture(session_id).await?;
        
        info!("Screen capture performance test completed");
        Ok(())
    }

    #[tokio::test]
    async fn test_encryption_performance() -> Result<()> {
        info!("Starting encryption performance test");
        
        let encryption = Arc::new(EndToEndEncryption::new());
        let keys = encryption.generate_key_pair().await?;
        let session_key = encryption.establish_session(&keys, &keys.public_key).await?;
        
        // Test encryption/decryption performance
        let test_data = vec![0u8; 1024 * 1024]; // 1MB test data
        let iterations = 100;
        
        let start_time = Instant::now();
        
        for _ in 0..iterations {
            let encrypted = encryption.encrypt_message(&session_key, &test_data).await?;
            let decrypted = encryption.decrypt_message(&session_key, &encrypted).await?;
            assert_eq!(test_data, decrypted);
        }
        
        let elapsed = start_time.elapsed();
        let throughput_mbps = ((test_data.len() * iterations) as f64 / elapsed.as_secs_f64()) / (1024.0 * 1024.0);
        
        info!("Encryption performance results:");
        info!("  Iterations: {}", iterations);
        info!("  Data size: {} MB", (test_data.len() * iterations) / (1024 * 1024));
        info!("  Throughput: {:.2} MB/s", throughput_mbps);
        info!("  Time per operation: {:.2} ms", elapsed.as_millis() as f64 / iterations as f64);
        
        // Performance assertion
        assert!(throughput_mbps > 10.0, "Should achieve at least 10 MB/s encryption throughput");
        
        info!("Encryption performance test completed");
        Ok(())
    }
}
