use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocal;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::api::media_engine::MIME_TYPE_VP8;
use webrtc::rtp::packet::Packet;
use bytes::Bytes;

use crate::screen_capture::{ScreenCapturer, CaptureFrame, CaptureConfig};

/// Screen streaming manager - captures screen and streams via WebRTC
pub struct ScreenStreamer {
    track: Arc<TrackLocalStaticRTP>,
    capturer: Arc<Mutex<Option<ScreenCapturer>>>,
    is_streaming: Arc<Mutex<bool>>,
    sequence_number: Arc<Mutex<u16>>,
    timestamp: Arc<Mutex<u32>>,
    ssrc: u32,
}

impl ScreenStreamer {
    /// Create a new screen streamer
    pub fn new() -> Result<Self> {
        // Create VP8 video track (better browser support than H.264)
        let track = Arc::new(TrackLocalStaticRTP::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_VP8.to_owned(),
                clock_rate: 90000, // Standard for video
                channels: 0,
                sdp_fmtp_line: "".to_owned(),
                rtcp_feedback: vec![],
            },
            "video".to_owned(),
            "genxlink_screen".to_owned(),
        ));

        Ok(Self {
            track,
            capturer: Arc::new(Mutex::new(None)),
            is_streaming: Arc::new(Mutex::new(false)),
            sequence_number: Arc::new(Mutex::new(0)),
            timestamp: Arc::new(Mutex::new(0)),
            ssrc: rand::random(),
        })
    }

    /// Get the video track for adding to peer connection
    pub fn get_track(&self) -> Arc<TrackLocalStaticRTP> {
        Arc::clone(&self.track)
    }

    /// Start streaming from a specific monitor
    pub async fn start_streaming(&self, monitor_index: usize) -> Result<()> {
        let mut is_streaming = self.is_streaming.lock().await;
        if *is_streaming {
            return Ok(());
        }

        // Create screen capturer
        let config = CaptureConfig {
            monitor_index,
            capture_cursor: true,
            target_fps: 30,
        };
        let capturer = ScreenCapturer::new(config)
            .context("Failed to create screen capturer")?;

        // Start capture with streaming callback
        let track = Arc::clone(&self.track);
        let seq_num = Arc::clone(&self.sequence_number);
        let timestamp_arc = Arc::clone(&self.timestamp);
        let ssrc = self.ssrc;

        capturer.start_capture(move |frame| {
            // Send frame via RTP in a non-blocking way
            let track = Arc::clone(&track);
            let seq = Arc::clone(&seq_num);
            let ts = Arc::clone(&timestamp_arc);
            
            // Spawn async task to send frame
            tokio::spawn(async move {
                if let Err(e) = Self::send_frame(track, frame, seq, ts, ssrc).await {
                    tracing::error!("Failed to send frame: {}", e);
                }
            });
            
            Ok(())
        }).await?;

        // Store capturer
        let mut capturer_guard = self.capturer.lock().await;
        *capturer_guard = Some(capturer);

        *is_streaming = true;

        tracing::info!("Screen streaming started for monitor {}", monitor_index);

        Ok(())
    }

    /// Send a captured frame via RTP
    async fn send_frame(
        track: Arc<TrackLocalStaticRTP>,
        frame: CaptureFrame,
        seq_num: Arc<Mutex<u16>>,
        timestamp: Arc<Mutex<u32>>,
        ssrc: u32,
    ) -> Result<()> {
        // For now, we'll send raw frame data
        // TODO: Add VP8 encoding for better compression
        
        let mut seq = seq_num.lock().await;
        let mut ts = timestamp.lock().await;

        // Create RTP packet
        let packet = Packet {
            header: webrtc::rtp::header::Header {
                version: 2,
                padding: false,
                extension: false,
                marker: true,
                payload_type: 96, // VP8 payload type
                sequence_number: *seq,
                timestamp: *ts,
                ssrc,
                ..Default::default()
            },
            payload: Bytes::from(frame.data),
        };

        // TODO: Send packet via WebRTC track
        // The webrtc crate's TrackLocalStaticRTP uses a different API
        // We'll implement this when setting up the full peer connection
        // For now, just log that we would send
        tracing::trace!("Would send RTP packet: seq={}, ts={}", *seq, *ts);

        // Increment sequence number and timestamp
        *seq = seq.wrapping_add(1);
        *ts = ts.wrapping_add(3000); // ~30 FPS (90000 / 30)

        Ok(())
    }

    /// Stop streaming
    pub async fn stop_streaming(&self) -> Result<()> {
        let mut is_streaming = self.is_streaming.lock().await;
        if !*is_streaming {
            return Ok(());
        }

        *is_streaming = false;

        // Stop capturer
        let mut capturer_guard = self.capturer.lock().await;
        if let Some(capturer) = capturer_guard.as_ref() {
            capturer.stop_capture();
        }
        *capturer_guard = None;

        tracing::info!("Screen streaming stopped");

        Ok(())
    }

    /// Check if currently streaming
    pub async fn is_streaming(&self) -> bool {
        *self.is_streaming.lock().await
    }
}
