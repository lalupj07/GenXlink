use crate::{
    ClientError,
    capture::{ScreenCapture, Frame},
    encoder::{VideoEncoder, EncoderConfig, EncodedFrame},
    streaming::{StreamingPipeline, StreamingStats},
};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{interval, Duration};

/// Video capture and streaming pipeline
use std::collections::VecDeque;
use tokio::sync::mpsc;

pub struct VideoPipeline {
    capture: Arc<Mutex<Box<dyn ScreenCapture>>>,
    streaming: Arc<Mutex<StreamingPipeline>>,
    frame_rate: u32,
    running: Arc<Mutex<bool>>,
    frame_sender: Option<mpsc::Sender<Frame>>,
    frame_buffer_size: usize,
}

impl VideoPipeline {
    /// Create a new video pipeline
    pub fn new(
        capture: Box<dyn ScreenCapture>,
        encoder: Box<dyn VideoEncoder>,
        frame_rate: u32,
    ) -> Result<Self, ClientError> {
        let streaming = StreamingPipeline::new(encoder, frame_rate)?;
        
        Ok(Self {
            capture: Arc::new(Mutex::new(capture)),
            streaming: Arc::new(Mutex::new(streaming)),
            frame_rate,
            running: Arc::new(Mutex::new(false)),
            frame_sender: None,
            frame_buffer_size: 5, // Default buffer size of 5 frames
        })
    }

    /// Start the streaming pipeline
    pub async fn start(&self) -> Result<(), ClientError> {
        if self.frame_sender.is_some() {
            return Err(ClientError::new("Pipeline already started"));
        }

        // Create channel for frame buffering
        let (tx, mut rx) = mpsc::channel(self.frame_buffer_size);
        
        let capture = self.capture.clone();
        let streaming = self.streaming.clone();
        let running = self.running.clone();
        let frame_rate = self.frame_rate;
        
        // Update the frame sender
        let mut this = self.clone();
        this.frame_sender = Some(tx);
        
        // Start the capture task
        tokio::spawn(async move {
            let frame_interval = Duration::from_secs_f64(1.0 / frame_rate as f64);
            let mut last_frame_time = std::time::Instant::now();
            
            // Start capture
            if let Err(e) = capture.lock().await.start().await {
                tracing::error!("Failed to start capture: {}", e);
                return;
            }
            
            // Main capture loop
            *running.lock().await = true;
            while *running.lock().await {
                let frame_start = std::time::Instant::now();
                
                // Calculate time until next frame should be captured
                let now = std::time::Instant::now();
                let elapsed_since_last = now.duration_since(last_frame_time);
                if elapsed_since_last < frame_interval {
                    // Sleep until it's time for the next frame
                    let sleep_time = frame_interval - elapsed_since_last;
                    tokio::time::sleep(sleep_time).await;
                }
                
                // Capture frame with timeout
                match tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    capture.lock().await.capture_frame()
                ).await {
                    Ok(Ok(Some(frame))) => {
                        // Send frame to processing thread
                        if let Err(e) = tx.send(frame).await {
                            tracing::error!("Failed to send frame to processing thread: {}", e);
                            break;
                        }
                    }
                    Ok(Ok(None)) => {
                        // No frame available, continue
                        continue;
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Capture error: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(_) => {
                        tracing::warn!("Frame capture timed out");
                    }
                }
                
                last_frame_time = std::time::Instant::now();
                let frame_time = last_frame_time.duration_since(frame_start);
                
                // Log if we're falling behind
                if frame_time > frame_interval {
                    tracing::warn!("Frame capture is falling behind: {:?} > {:?}", 
                        frame_time, frame_interval);
                }
            }
            
            // Stop capture
            if let Err(e) = capture.lock().await.stop().await {
                tracing::error!("Failed to stop capture: {}", e);
            }
        });
        
        // Start the processing task
        let streaming_clone = self.streaming.clone();
        let running_clone = self.running.clone();
        
        tokio::spawn(async move {
            while *running_clone.lock().await {
                match rx.recv().await {
                    Some(frame) => {
                        // Process frame
                        if let Err(e) = streaming_clone.lock().await.stream_frame(&frame).await {
                            tracing::error!("Failed to stream frame: {}", e);
                        }
                    }
                    None => {
                        // Channel closed
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Stop the streaming pipeline
    pub async fn stop(&self) -> Result<(), ClientError> {
        // Signal all tasks to stop
        *self.running.lock().await = false;
        
        // Drop the frame sender to close the channel
        drop(self.frame_sender.take());
        
        Ok(())
    }

    /// Check if pipeline is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// Get streaming statistics
    pub async fn get_stats(&self) -> StreamingStats {
        let stream = self.streaming.lock().await;
        stream.get_stats()
    }

    /// Get the video track for WebRTC
    pub async fn get_video_track(&self) -> Arc<webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP> {
        let stream = self.streaming.lock().await;
        stream.get_track()
    }
}

/// Pipeline builder for easier configuration
pub struct VideoPipelineBuilder {
    capture: Option<Box<dyn ScreenCapture>>,
    encoder: Option<Box<dyn VideoEncoder>>,
    frame_rate: u32,
}

impl VideoPipelineBuilder {
    /// Create a new pipeline builder
    pub fn new() -> Self {
        Self {
            capture: None,
            encoder: None,
            frame_rate: 30,
        }
    }

    /// Set the screen capture
    pub fn with_capture(mut self, capture: Box<dyn ScreenCapture>) -> Self {
        self.capture = Some(capture);
        self
    }

    /// Set the video encoder
    pub fn with_encoder(mut self, encoder: Box<dyn VideoEncoder>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    /// Set the frame rate
    pub fn with_frame_rate(mut self, fps: u32) -> Self {
        self.frame_rate = fps;
        self
    }

    /// Build the pipeline
    pub fn build(self) -> Result<VideoPipeline, ClientError> {
        let capture = self.capture
            .ok_or_else(|| ClientError::StreamingError("Screen capture not configured".to_string()))?;
        
        let encoder = self.encoder
            .ok_or_else(|| ClientError::StreamingError("Video encoder not configured".to_string()))?;

        VideoPipeline::new(capture, encoder, self.frame_rate)
    }
}

impl Default for VideoPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline manager for multiple streams
pub struct PipelineManager {
    pipelines: Arc<Mutex<Vec<Arc<VideoPipeline>>>>,
}

impl PipelineManager {
    /// Create a new pipeline manager
    pub fn new() -> Self {
        Self {
            pipelines: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a pipeline
    pub async fn add_pipeline(&self, pipeline: VideoPipeline) -> usize {
        let mut pipelines = self.pipelines.lock().await;
        let id = pipelines.len();
        pipelines.push(Arc::new(pipeline));
        id
    }

    /// Start a pipeline by ID
    pub async fn start_pipeline(&self, id: usize) -> Result<(), ClientError> {
        let pipelines = self.pipelines.lock().await;
        let pipeline = pipelines.get(id)
            .ok_or_else(|| ClientError::StreamingError(format!("Pipeline {} not found", id)))?;
        
        pipeline.start().await
    }

    /// Stop a pipeline by ID
    pub async fn stop_pipeline(&self, id: usize) -> Result<(), ClientError> {
        let pipelines = self.pipelines.lock().await;
        let pipeline = pipelines.get(id)
            .ok_or_else(|| ClientError::StreamingError(format!("Pipeline {} not found", id)))?;
        
        pipeline.stop().await
    }

    /// Stop all pipelines
    pub async fn stop_all(&self) -> Result<(), ClientError> {
        let pipelines = self.pipelines.lock().await;
        for pipeline in pipelines.iter() {
            pipeline.stop().await?;
        }
        Ok(())
    }

    /// Get pipeline count
    pub async fn pipeline_count(&self) -> usize {
        let pipelines = self.pipelines.lock().await;
        pipelines.len()
    }
}

impl Default for PipelineManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::{H264Encoder, VideoCodec};

    #[tokio::test]
    async fn test_pipeline_builder() {
        let encoder = Box::new(H264Encoder::new());
        let mut enc = encoder;
        let config = EncoderConfig {
            width: 1920,
            height: 1080,
            fps: 30,
            bitrate: 2_000_000,
            codec: VideoCodec::H264,
        };
        enc.init(config).unwrap();

        // Note: Can't test full pipeline without actual capture implementation
        // This would need a mock capture for testing
    }

    #[tokio::test]
    async fn test_pipeline_manager() {
        let manager = PipelineManager::new();
        assert_eq!(manager.pipeline_count().await, 0);
    }
}
