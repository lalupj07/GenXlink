use crate::{Frame, ClientError};
use openh264::encoder::Encoder as OpenH264Encoder;

/// Encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
    pub codec: VideoCodec,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            bitrate: 2_000_000, // 2 Mbps
            codec: VideoCodec::H264,
        }
    }
}

/// Video codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    VP8,
    VP9,
}

/// Encoded frame data
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub is_keyframe: bool,
}

/// Video encoder trait
pub trait VideoEncoder: Send + Sync {
    /// Initialize the encoder
    fn init(&mut self, config: EncoderConfig) -> Result<(), ClientError>;
    
    /// Encode a frame
    fn encode(&mut self, frame: &Frame) -> Result<EncodedFrame, ClientError>;
    
    /// Flush any pending frames
    fn flush(&mut self) -> Result<Vec<EncodedFrame>, ClientError>;
    
    /// Get encoder configuration
    fn get_config(&self) -> &EncoderConfig;
}

/// H.264 encoder implementation
pub struct H264Encoder {
    config: Option<EncoderConfig>,
    encoder: Option<OpenH264Encoder>,
    frame_count: u64,
}

impl H264Encoder {
    pub fn new() -> Self {
        Self {
            config: None,
            encoder: None,
            frame_count: 0,
        }
    }
}

impl Default for H264Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEncoder for H264Encoder {
    fn init(&mut self, config: EncoderConfig) -> Result<(), ClientError> {
        // Create OpenH264 encoder
        let encoder = OpenH264Encoder::new()
            .map_err(|e| ClientError::EncodingError(format!("Failed to create encoder: {:?}", e)))?;
        
        tracing::info!("H.264 encoder initialized: {}x{} @ {} fps, {} bps", 
            config.width, config.height, config.fps, config.bitrate);
        
        self.encoder = Some(encoder);
        self.config = Some(config);
        self.frame_count = 0;
        
        Ok(())
    }
    
    fn encode(&mut self, frame: &Frame) -> Result<EncodedFrame, ClientError> {
        let config = self.config.as_ref()
            .ok_or_else(|| ClientError::EncodingError("Encoder not initialized".to_string()))?;
        
        let encoder = self.encoder.as_mut()
            .ok_or_else(|| ClientError::EncodingError("Encoder not initialized".to_string()))?;
        
        // Create YUV buffer for encoding
        let yuv_buffer = openh264::formats::YUVBuffer::new(
            config.width as usize, 
            config.height as usize
        );
        
        // Encode frame
        let bitstream = encoder.encode(&yuv_buffer)
            .map_err(|e| ClientError::EncodingError(format!("Encoding failed: {:?}", e)))?;
        
        self.frame_count += 1;
        let is_keyframe = self.frame_count % 30 == 1; // Keyframe every 30 frames
        
        Ok(EncodedFrame {
            data: bitstream.to_vec(),
            timestamp: frame.timestamp,
            is_keyframe,
        })
    }
    
    fn flush(&mut self) -> Result<Vec<EncodedFrame>, ClientError> {
        // OpenH264 doesn't require explicit flushing
        Ok(vec![])
    }
    
    fn get_config(&self) -> &EncoderConfig {
        self.config.as_ref().expect("Encoder not initialized")
    }
}

impl H264Encoder {
    /// Convert BGRA to YUV420 format
    fn bgra_to_yuv(&self, bgra: &[u8], width: u32, height: u32) -> Result<Vec<u8>, ClientError> {
        let pixel_count = (width * height) as usize;
        let mut yuv = vec![0u8; pixel_count * 3 / 2]; // YUV420 format
        
        // Simple BGRA to YUV conversion
        // Y plane
        for y in 0..height as usize {
            for x in 0..width as usize {
                let idx = (y * width as usize + x) * 4;
                if idx + 3 < bgra.len() {
                    let b = bgra[idx] as f32;
                    let g = bgra[idx + 1] as f32;
                    let r = bgra[idx + 2] as f32;
                    
                    // BT.601 conversion
                    let y_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                    yuv[y * width as usize + x] = y_val;
                }
            }
        }
        
        // U and V planes (subsampled)
        let u_offset = pixel_count;
        let v_offset = pixel_count + pixel_count / 4;
        
        for y in (0..height as usize).step_by(2) {
            for x in (0..width as usize).step_by(2) {
                let idx = (y * width as usize + x) * 4;
                if idx + 3 < bgra.len() {
                    let b = bgra[idx] as f32;
                    let g = bgra[idx + 1] as f32;
                    let r = bgra[idx + 2] as f32;
                    
                    let u_val = ((-0.169 * r - 0.331 * g + 0.500 * b) + 128.0) as u8;
                    let v_val = ((0.500 * r - 0.419 * g - 0.081 * b) + 128.0) as u8;
                    
                    let uv_idx = (y / 2) * (width as usize / 2) + (x / 2);
                    yuv[u_offset + uv_idx] = u_val;
                    yuv[v_offset + uv_idx] = v_val;
                }
            }
        }
        
        Ok(yuv)
    }
}

/// Create a video encoder
pub fn create_encoder(codec: VideoCodec) -> Result<Box<dyn VideoEncoder>, ClientError> {
    match codec {
        VideoCodec::H264 => Ok(Box::new(H264Encoder::new())),
        _ => Err(ClientError::EncodingError("Codec not supported".to_string())),
    }
}
