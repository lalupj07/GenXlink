use crate::{ClientError, encoder::{VideoEncoder, EncodedFrame}};
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocal;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::api::media_engine::MIME_TYPE_H264;
use std::sync::Arc;
use tokio::sync::Mutex;
use bytes::Bytes;

/// Video streaming manager
pub struct VideoStreamer {
    track: Arc<TrackLocalStaticRTP>,
    encoder: Arc<Mutex<Box<dyn VideoEncoder>>>,
    sequence_number: u16,
    timestamp: u32,
    ssrc: u32,
}

impl VideoStreamer {
    /// Create a new video streamer
    pub fn new(encoder: Box<dyn VideoEncoder>) -> Result<Self, ClientError> {
        // Create H.264 video track
        let track = Arc::new(TrackLocalStaticRTP::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000, // Standard for H.264
                channels: 0,
                sdp_fmtp_line: "".to_owned(),
                rtcp_feedback: vec![],
            },
            "video".to_owned(),
            "genxlink_video".to_owned(),
        ));

        Ok(Self {
            track,
            encoder: Arc::new(Mutex::new(encoder)),
            sequence_number: 0,
            timestamp: 0,
            ssrc: rand::random(),
        })
    }

    /// Get the video track for adding to peer connection
    pub fn get_track(&self) -> Arc<TrackLocalStaticRTP> {
        Arc::clone(&self.track)
    }

    /// Stream an encoded frame
    pub async fn stream_frame(&mut self, frame: EncodedFrame) -> Result<(), ClientError> {
        // For now, just update counters - actual RTP writing will be implemented
        // when we integrate with the full WebRTC pipeline
        
        // Update sequence number and timestamp
        self.sequence_number = self.sequence_number.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(3000); // 90000 Hz / 30 FPS = 3000

        tracing::debug!("Frame streamed: seq={}, ts={}, keyframe={}", 
            self.sequence_number, self.timestamp, frame.is_keyframe);

        Ok(())
    }

    /// Create RTP packet from encoded frame
    fn create_rtp_packet(&self, frame: &EncodedFrame) -> Result<webrtc::rtp::packet::Packet, ClientError> {
        let mut packet = webrtc::rtp::packet::Packet {
            header: webrtc::rtp::header::Header {
                version: 2,
                padding: false,
                extension: false,
                marker: frame.is_keyframe,
                payload_type: 96, // Dynamic payload type for H.264
                sequence_number: self.sequence_number,
                timestamp: self.timestamp,
                ssrc: self.ssrc,
                ..Default::default()
            },
            payload: Bytes::copy_from_slice(&frame.data),
        };

        Ok(packet)
    }

    /// Get current streaming statistics
    pub fn get_stats(&self) -> StreamingStats {
        StreamingStats {
            packets_sent: self.sequence_number as u64,
            timestamp: self.timestamp,
            ssrc: self.ssrc,
        }
    }
}

/// Streaming statistics
#[derive(Debug, Clone)]
pub struct StreamingStats {
    pub packets_sent: u64,
    pub timestamp: u32,
    pub ssrc: u32,
}

/// Frame streaming pipeline
pub struct StreamingPipeline {
    streamer: VideoStreamer,
    frame_rate: u32,
}

impl StreamingPipeline {
    /// Create a new streaming pipeline
    pub fn new(encoder: Box<dyn VideoEncoder>, frame_rate: u32) -> Result<Self, ClientError> {
        let streamer = VideoStreamer::new(encoder)?;
        
        Ok(Self {
            streamer,
            frame_rate,
        })
    }

    /// Get the video track
    pub fn get_track(&self) -> Arc<TrackLocalStaticRTP> {
        self.streamer.get_track()
    }

    /// Start streaming frames
    pub async fn stream_frame(&mut self, frame: EncodedFrame) -> Result<(), ClientError> {
        self.streamer.stream_frame(frame).await
    }

    /// Get streaming statistics
    pub fn get_stats(&self) -> StreamingStats {
        self.streamer.get_stats()
    }

    /// Get target frame interval in milliseconds
    pub fn frame_interval_ms(&self) -> u64 {
        1000 / self.frame_rate as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::{H264Encoder, EncoderConfig, VideoCodec};

    #[tokio::test]
    async fn test_video_streamer_creation() {
        let encoder = Box::new(H264Encoder::new());
        let streamer = VideoStreamer::new(encoder);
        assert!(streamer.is_ok());
    }

    #[tokio::test]
    async fn test_streaming_pipeline() {
        let mut encoder = Box::new(H264Encoder::new());
        let config = EncoderConfig {
            width: 1920,
            height: 1080,
            fps: 30,
            bitrate: 2_000_000,
            codec: VideoCodec::H264,
        };
        encoder.init(config).unwrap();

        let pipeline = StreamingPipeline::new(encoder, 30);
        assert!(pipeline.is_ok());
    }
}
