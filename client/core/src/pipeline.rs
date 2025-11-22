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
pub struct VideoPipeline {
    capture: Arc<Mutex<Box<dyn ScreenCapture>>>,
    streaming: Arc<Mutex<StreamingPipeline>>,
    frame_rate: u32,
    running: Arc<Mutex<bool>>,
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
        })
    }

    /// Start the streaming pipeline
    pub async fn start(&self) -> Result<(), ClientError> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(ClientError::StreamingError("Pipeline already running".to_string()));
        }
        *running = true;
        drop(running);

        let capture = Arc::clone(&self.capture);
        let streaming = Arc::clone(&self.streaming);
        let running = Arc::clone(&self.running);
        let frame_interval = Duration::from_millis(1000 / self.frame_rate as u64);

        // Spawn streaming task
        tokio::spawn(async move {
            let mut ticker = interval(frame_interval);
            let mut frame_count = 0u64;
            let mut error_count = 0u32;

            tracing::info!("Video pipeline started at {} FPS", 1000 / frame_interval.as_millis());

            loop {
                // Check if still running
                {
                    let is_running = running.lock().await;
                    if !*is_running {
                        tracing::info!("Video pipeline stopped");
                        break;
                    }
                }

                ticker.tick().await;

                // Capture frame
                let frame = {
                    let mut cap = capture.lock().await;
                    match cap.capture_frame().await {
                        Ok(frame) => frame,
                        Err(e) => {
                            error_count += 1;
                            if error_count > 10 {
                                tracing::error!("Too many capture errors, stopping pipeline");
                                break;
                            }
                            tracing::warn!("Frame capture failed: {}", e);
                            continue;
                        }
                    }
                };

                // Encode and stream frame
                let mut stream = streaming.lock().await;
                
                // Get encoder from streaming pipeline
                // For now, we'll create a placeholder encoded frame
                // In a real implementation, this would encode the actual frame
                let encoded_frame = EncodedFrame {
                    data: vec![0u8; 1024], // Placeholder
                    timestamp: frame.timestamp,
                    is_keyframe: frame_count % 30 == 0,
                };

                match stream.stream_frame(encoded_frame).await {
                    Ok(_) => {
                        frame_count += 1;
                        if frame_count % 30 == 0 {
                            tracing::debug!("Streamed {} frames", frame_count);
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        tracing::warn!("Frame streaming failed: {}", e);
                    }
                }
            }

            tracing::info!("Video pipeline task completed. Total frames: {}", frame_count);
        });

        Ok(())
    }

    /// Stop the streaming pipeline
    pub async fn stop(&self) -> Result<(), ClientError> {
        let mut running = self.running.lock().await;
        *running = false;
        tracing::info!("Video pipeline stop requested");
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
