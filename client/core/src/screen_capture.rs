use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(target_os = "windows")]
use windows::{
    core::ComInterface,
    Win32::Graphics::Dxgi::{
        IDXGIFactory1, IDXGIOutputDuplication, DXGI_OUTPUT_DESC,
        DXGI_OUTDUPL_FRAME_INFO,
        CreateDXGIFactory1, DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT,
    },
    Win32::Graphics::Direct3D11::{
        ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
        D3D11CreateDevice, D3D11_SDK_VERSION, D3D11_CREATE_DEVICE_FLAG,
    },
    Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0,
};

/// Screen capture frame
#[derive(Clone)]
pub struct CaptureFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: std::time::Instant,
}

/// Screen capture configuration
#[derive(Clone, Debug)]
pub struct CaptureConfig {
    pub monitor_index: usize,
    pub capture_cursor: bool,
    pub target_fps: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            monitor_index: 0,
            capture_cursor: true,
            target_fps: 30,
        }
    }
}

/// Screen capturer using DXGI Desktop Duplication
pub struct ScreenCapturer {
    #[cfg(target_os = "windows")]
    device: Option<ID3D11Device>,
    #[cfg(target_os = "windows")]
    context: Option<ID3D11DeviceContext>,
    #[cfg(target_os = "windows")]
    duplication: Option<IDXGIOutputDuplication>,
    
    config: CaptureConfig,
    is_capturing: Arc<Mutex<bool>>,
}

impl ScreenCapturer {
    /// Create a new screen capturer
    pub fn new(config: CaptureConfig) -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            let (device, context, duplication) = Self::init_dxgi(&config)?;
            
            Ok(Self {
                device: Some(device),
                context: Some(context),
                duplication: Some(duplication),
                config,
                is_capturing: Arc::new(Mutex::new(false)),
            })
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Ok(Self {
                config,
                is_capturing: Arc::new(Mutex::new(false)),
            })
        }
    }
    
    #[cfg(target_os = "windows")]
    fn init_dxgi(config: &CaptureConfig) -> Result<(ID3D11Device, ID3D11DeviceContext, IDXGIOutputDuplication)> {
        unsafe {
            // Create D3D11 device
            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            let mut feature_level = D3D_FEATURE_LEVEL_11_0;
            
            D3D11CreateDevice(
                None,
                windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE(1), // D3D_DRIVER_TYPE_HARDWARE = 1
                None,
                D3D11_CREATE_DEVICE_FLAG(0),
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                Some(&mut feature_level),
                Some(&mut context),
            ).context("Failed to create D3D11 device")?;
            
            let device = device.context("Device is None")?;
            let context = context.context("Context is None")?;
            
            // Get DXGI device
            let dxgi_device: windows::Win32::Graphics::Dxgi::IDXGIDevice = 
                device.cast().context("Failed to cast to IDXGIDevice")?;
            
            // Get adapter
            let adapter = dxgi_device.GetAdapter()
                .context("Failed to get adapter")?;
            
            // Get output (monitor)
            let output = adapter.EnumOutputs(config.monitor_index as u32)
                .context("Failed to enumerate outputs")?;
            
            let output1: windows::Win32::Graphics::Dxgi::IDXGIOutput1 = 
                output.cast().context("Failed to cast to IDXGIOutput1")?;
            
            // Create desktop duplication
            let duplication = output1.DuplicateOutput(&device)
                .context("Failed to create desktop duplication")?;
            
            Ok((device, context, duplication))
        }
    }
    
    /// Start capturing
    pub async fn start_capture<F>(&self, callback: F) -> Result<()>
    where
        F: FnMut(CaptureFrame) -> Result<()> + Send + 'static,
    {
        let mut is_capturing = self.is_capturing.lock().await;
        *is_capturing = true;
        drop(is_capturing);
        
        #[cfg(target_os = "windows")]
        {
            self.capture_loop_windows(callback).await?;
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            tracing::warn!("Screen capture not implemented for this platform");
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    async fn capture_loop_windows<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(CaptureFrame) -> Result<()> + Send + 'static,
    {
        let duplication = self.duplication.as_ref()
            .context("Duplication not initialized")?;
        let device = self.device.as_ref()
            .context("Device not initialized")?;
        let context = self.context.as_ref()
            .context("Context not initialized")?;
        
        let frame_interval = std::time::Duration::from_secs_f64(1.0 / self.config.target_fps as f64);
        let mut last_frame_time = std::time::Instant::now();
        let mut frame_count = 0;
        let mut last_log_time = std::time::Instant::now();
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 5;
        
        tracing::info!("Starting capture loop at {} FPS", self.config.target_fps);
        
        loop {
            let is_capturing = *self.is_capturing.lock().await;
            if !is_capturing {
                tracing::info!("Capture loop stopping");
                break;
            }

            let frame_start = std::time::Instant::now();

            // Calculate time until next frame should be captured
            let now = std::time::Instant::now();
            let elapsed_since_last = now.duration_since(last_frame_time);
            if elapsed_since_last < frame_interval {
                // Sleep until it's time for the next frame
                let sleep_time = frame_interval - elapsed_since_last;
                tokio::time::sleep(sleep_time).await;
            }

            // Capture frame (simplified for now)
            let capture_result = self.capture_frame_windows(duplication, device, context);

            match capture_result {
                Ok(Some(frame)) => {
                    if let Err(e) = callback(frame) {
                        tracing::error!("Frame callback error: {}", e);
                    } else {
                        consecutive_errors = 0; // Reset error counter on successful frame
                        frame_count += 1;
                    }
                }
                Ok(None) => {
                    // No new frame available, this is normal
                    continue;
                }
                Err(e) => {
                    consecutive_errors += 1;
                    tracing::error!("Capture error ({} of {}): {}", 
                        consecutive_errors, MAX_CONSECUTIVE_ERRORS, e);
                    
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        tracing::error!("Too many consecutive errors, stopping capture");
                        break;
                    }
                    
                    // Exponential backoff on errors
                    let backoff = std::time::Duration::from_millis(100 * u64::pow(2, consecutive_errors - 1));
                    tokio::time::sleep(backoff).await;
                    continue;
                }
            }

            // Update timing statistics
            last_frame_time = std::time::Instant::now();
            let frame_time = last_frame_time.duration_since(frame_start);

            // Log FPS every second
            if last_frame_time.duration_since(last_log_time) > std::time::Duration::from_secs(1) {
                let fps = frame_count as f64 / last_frame_time.duration_since(last_log_time).as_secs_f64();
                tracing::debug!("Capture FPS: {:.1}, Frame time: {:?}", fps, frame_time);
                frame_count = 0;
                last_log_time = last_frame_time;
            }

            // Log warning if we're falling behind
            if frame_time > frame_interval {
                tracing::warn!("Frame capture is falling behind: {:?} > {:?}", 
                    frame_time, frame_interval);
            }
        }
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn capture_frame_windows(
        &self,
        duplication: &IDXGIOutputDuplication,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
    ) -> Result<Option<CaptureFrame>> {
        unsafe {
            let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut desktop_resource: Option<windows::Win32::Graphics::Dxgi::IDXGIResource> = None;
            
            // Acquire next frame
            match duplication.AcquireNextFrame(100, &mut frame_info, &mut desktop_resource) {
                Ok(_) => {}
                Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => {
                    return Ok(None); // No new frame
                }
                Err(e) if e.code() == DXGI_ERROR_ACCESS_LOST => {
                    return Err(anyhow::anyhow!("Desktop duplication access lost"));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to acquire frame: {:?}", e));
                }
            }
            
            let desktop_resource = desktop_resource.context("Desktop resource is None")?;
            
            // Get texture
            let texture: ID3D11Texture2D = desktop_resource.cast()
                .context("Failed to cast to ID3D11Texture2D")?;
            
            // Get texture description
            let mut desc = std::mem::zeroed();
            texture.GetDesc(&mut desc);
            
            // Create staging texture for CPU access
            let mut staging_desc = desc.clone();
            staging_desc.Usage = windows::Win32::Graphics::Direct3D11::D3D11_USAGE_STAGING;
            staging_desc.BindFlags = 0;
            staging_desc.CPUAccessFlags = windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_READ.0 as u32;
            staging_desc.MiscFlags = 0;
            
            let staging_texture: ID3D11Texture2D = {
                let mut texture: Option<ID3D11Texture2D> = None;
                device.CreateTexture2D(&staging_desc, None, Some(&mut texture))
                    .context("Failed to create staging texture")?;
                texture.expect("CreateTexture2D returned None")
            };
            
            // Copy texture to staging
            context.CopyResource(&staging_texture, &texture);
            
            // Map staging texture
            let mut mapped = std::mem::zeroed();
            context.Map(
                &staging_texture,
                0,
                windows::Win32::Graphics::Direct3D11::D3D11_MAP_READ,
                0,
                Some(&mut mapped),
            ).context("Failed to map staging texture")?;
            
            // Copy data
            let width = desc.Width as usize;
            let height = desc.Height as usize;
            let row_pitch = mapped.RowPitch as usize;
            let data_size = height * width * 4; // BGRA format
            let mut frame_data = vec![0u8; data_size];
            
            let src_ptr = mapped.pData as *const u8;
            for y in 0..height {
                let src_offset = y * row_pitch;
                let dst_offset = y * width * 4;
                std::ptr::copy_nonoverlapping(
                    src_ptr.add(src_offset),
                    frame_data.as_mut_ptr().add(dst_offset),
                    width * 4,
                );
            }
            
            // Unmap
            context.Unmap(&staging_texture, 0);
            
            // Release frame
            duplication.ReleaseFrame().ok();
            
            Ok(Some(CaptureFrame {
                width: width as u32,
                height: height as u32,
                data: frame_data,
                timestamp: std::time::Instant::now(),
            }))
        }
    }
    
    /// Stop capturing
    pub async fn stop_capture(&self) {
        let mut is_capturing = self.is_capturing.lock().await;
        *is_capturing = false;
    }
    
    /// Get available monitors
    #[cfg(target_os = "windows")]
    pub fn get_monitors() -> Result<Vec<MonitorInfo>> {
        unsafe {
            let factory: IDXGIFactory1 = CreateDXGIFactory1()
                .context("Failed to create DXGI factory")?;
            
            let mut monitors = Vec::new();
            let mut adapter_index = 0;
            
            while let Ok(adapter) = factory.EnumAdapters1(adapter_index) {
                let mut output_index = 0;
                
                while let Ok(output) = adapter.EnumOutputs(output_index) {
                    let mut desc = DXGI_OUTPUT_DESC::default();
                    output.GetDesc(&mut desc).ok();
                    
                    let name = String::from_utf16_lossy(&desc.DeviceName);
                    let name = name.trim_end_matches('\0');
                    
                    monitors.push(MonitorInfo {
                        index: monitors.len(),
                        name: name.to_string(),
                        width: (desc.DesktopCoordinates.right - desc.DesktopCoordinates.left) as u32,
                        height: (desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top) as u32,
                        is_primary: desc.AttachedToDesktop.as_bool(),
                    });
                    
                    output_index += 1;
                }
                
                adapter_index += 1;
            }
            
            Ok(monitors)
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn get_monitors() -> Result<Vec<MonitorInfo>> {
        Ok(vec![MonitorInfo {
            index: 0,
            name: "Primary Monitor".to_string(),
            width: 1920,
            height: 1080,
            is_primary: true,
        }])
    }
}

/// Monitor information
#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_monitors() {
        let monitors = ScreenCapturer::get_monitors().unwrap();
        assert!(!monitors.is_empty());
        println!("Found {} monitors", monitors.len());
        for monitor in monitors {
            println!("  - {}: {}x{}", monitor.name, monitor.width, monitor.height);
        }
    }
}
