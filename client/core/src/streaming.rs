use crate::{ClientError, encoder::{VideoEncoder, EncodedFrame}};
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocal;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::api::media_engine::MIME_TYPE_H264;
use std::sync::Arc;
use tokio::sync::Mutex;
use bytes::Bytes;

/// Frame data structure
#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: std::time::Instant,
}

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
    pub fn new(_encoder: Box<dyn VideoEncoder>) -> Result<Self, ClientError> {
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
            encoder: Arc::new(Mutex::new(_encoder)),
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
            frames_sent: self.sequence_number as u64,
            bytes_sent: self.sequence_number as u64 * 1024, // Estimate
            current_fps: 30.0, // Default FPS
            average_bitrate: 1000000.0, // 1 Mbps estimate
            encoding_errors: 0,
            network_errors: 0,
            avg_encode_time_ms: 10.0, // 10ms estimate
            start_time: Some(std::time::Instant::now()),
            packets_sent: self.sequence_number as u64,
            timestamp: self.timestamp,
            ssrc: self.ssrc,
        }
    }
}

/// Streaming statistics
#[derive(Debug, Clone)]
pub struct StreamingStats {
    /// Total number of frames sent
    pub frames_sent: u64,
    /// Total number of bytes sent
    pub bytes_sent: u64,
    /// Current frames per second
    pub current_fps: f64,
    /// Average bitrate in bits per second
    pub average_bitrate: f64,
    /// Number of encoding errors
    pub encoding_errors: u64,
    /// Number of network errors
    pub network_errors: u64,
    /// Average encoding time in milliseconds
    pub avg_encode_time_ms: f64,
    /// Timestamp when streaming started
    pub start_time: Option<std::time::Instant>,
    /// Packets sent (for RTP streaming)
    pub packets_sent: u64,
    /// Current timestamp
    pub timestamp: u32,
    /// SSRC for RTP stream
    pub ssrc: u32,
}

impl Default for StreamingStats {
    fn default() -> Self {
        Self {
            frames_sent: 0,
            bytes_sent: 0,
            current_fps: 0.0,
            average_bitrate: 0.0,
            encoding_errors: 0,
            network_errors: 0,
            avg_encode_time_ms: 0.0,
            start_time: None,
            packets_sent: 0,
            timestamp: 0,
            ssrc: 0,
        }
    }
}

/// Frame streaming pipeline
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct StreamingPipeline {
    streamer: VideoStreamer,
    frame_rate: u32,
    stats: Arc<InternalStreamingStats>,
}

#[derive(Debug, Default)]
struct InternalStreamingStats {
    frames_sent: AtomicU64,
    bytes_sent: AtomicU64,
    last_frame_time: parking_lot::Mutex<Option<Instant>>,
    avg_frame_time: parking_lot::Mutex<f64>,
}

impl StreamingPipeline {
    /// Create a new streaming pipeline
    pub fn new(encoder: Box<dyn VideoEncoder>, frame_rate: u32) -> Result<Self, ClientError> {
        let streamer = VideoStreamer::new(encoder)?;
        
        Ok(Self {
            streamer,
            frame_rate,
            stats: Arc::new(InternalStreamingStats::default()),
        })
    }

    /// Get the video track
    pub fn get_track(&self) -> Arc<TrackLocalStaticRTP> {
        self.streamer.get_track()
    }

    /// Start streaming frames
    pub async fn stream_frame(&mut self, frame: &Frame) -> Result<(), ClientError> {
        let frame_start = Instant::now();
        
        // Update frame timing stats
        {
            let mut last_frame_time = self.stats.last_frame_time.lock();
            if let Some(last_time) = *last_frame_time {
                let frame_time = frame_start.duration_since(last_time).as_secs_f64();
                let alpha = 0.1; // Smoothing factor
                let mut avg_frame_time = self.stats.avg_frame_time.lock();
                *avg_frame_time = alpha * frame_time + (1.0 - alpha) * *avg_frame_time;
                
                // Log if we're falling behind
                let target_frame_time = 1.0 / self.frame_rate as f64;
                if *avg_frame_time > target_frame_time * 1.5 {
                    tracing::warn!(
                        "Streaming is falling behind: avg frame time {:.2}ms > {:.2}ms",
                        avg_frame_time * 1000.0,
                        target_frame_time * 1000.0
                    );
                }
            }
            *last_frame_time = Some(frame_start);
        }
        
        // For now, just update statistics - actual encoding will be implemented later
        self.stats.frames_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.stats.bytes_sent.fetch_add(
            frame.data.len() as u64,
            std::sync::atomic::Ordering::Relaxed
        );
        
        Ok(())
    }

    /// Get streaming statistics
    pub fn get_stats(&self) -> StreamingStats {
        let mut stats = self.streamer.get_stats();
        
        // Add our additional stats
        stats.frames_sent = self.stats.frames_sent.load(std::sync::atomic::Ordering::Relaxed);
        stats.bytes_sent = self.stats.bytes_sent.load(std::sync::atomic::Ordering::Relaxed);
        
        // Calculate current FPS based on average frame time
        let avg_frame_time = *self.stats.avg_frame_time.lock();
        if avg_frame_time > 0.0 {
            stats.current_fps = 1.0 / avg_frame_time;
        }
        
        stats
    }

    /// Get target frame interval in milliseconds
    pub fn frame_interval_ms(&self) -> u64 {
        1000 / self.frame_rate as u64
    }
}
