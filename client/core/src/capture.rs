use async_trait::async_trait;
use crate::ClientError;

/// Frame data from screen capture
#[derive(Debug, Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// Screen capture trait
#[async_trait]
pub trait ScreenCapture: Send + Sync {
    /// Initialize the capture
    async fn init(&mut self) -> Result<(), ClientError>;
    
    /// Capture a frame
    async fn capture_frame(&mut self) -> Result<Frame, ClientError>;
    
    /// Get screen dimensions
    fn get_dimensions(&self) -> (u32, u32);
    
    /// Cleanup resources
    async fn cleanup(&mut self) -> Result<(), ClientError>;
}

/// Windows DXGI screen capture implementation
#[cfg(target_os = "windows")]
pub mod win_impl {
    use super::*;
    use async_trait::async_trait;
    use windows::Win32::Graphics::Dxgi::*;
    use windows::Win32::Graphics::Dxgi::Common::*;
    use windows::Win32::Graphics::Direct3D::*;
    use windows::Win32::Graphics::Direct3D11::*;
    use windows::core::ComInterface;
    use std::mem;
    
    pub struct DxgiCapture {
        width: u32,
        height: u32,
        initialized: bool,
        device: Option<ID3D11Device>,
        context: Option<ID3D11DeviceContext>,
        duplication: Option<IDXGIOutputDuplication>,
        staging_texture: Option<ID3D11Texture2D>,
    }
    
    impl DxgiCapture {
        pub fn new() -> Self {
            Self {
                width: 1920,
                height: 1080,
                initialized: false,
                device: None,
                context: None,
                duplication: None,
                staging_texture: None,
            }
        }
        
        fn create_d3d_device(&mut self) -> Result<(), ClientError> {
            unsafe {
                let mut device: Option<ID3D11Device> = None;
                let mut context: Option<ID3D11DeviceContext> = None;
                let mut feature_level = D3D_FEATURE_LEVEL_11_0;
                
                let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
                
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    None,
                    flags,
                    Some(&[D3D_FEATURE_LEVEL_11_0]),
                    D3D11_SDK_VERSION,
                    Some(&mut device),
                    Some(&mut feature_level),
                    Some(&mut context),
                ).map_err(|e| ClientError::CaptureError(format!("Failed to create D3D11 device: {}", e)))?;
                
                self.device = device;
                self.context = context;
                
                Ok(())
            }
        }
        
        fn create_duplication(&mut self) -> Result<(), ClientError> {
            unsafe {
                let device = self.device.as_ref()
                    .ok_or_else(|| ClientError::CaptureError("Device not initialized".to_string()))?;
                
                // Get DXGI device
                let dxgi_device: IDXGIDevice = device.cast()
                    .map_err(|e| ClientError::CaptureError(format!("Failed to get DXGI device: {}", e)))?;
                
                // Get adapter
                let adapter = dxgi_device.GetAdapter()
                    .map_err(|e| ClientError::CaptureError(format!("Failed to get adapter: {}", e)))?;
                
                // Get output (primary monitor)
                let output = adapter.EnumOutputs(0)
                    .map_err(|e| ClientError::CaptureError(format!("Failed to get output: {}", e)))?;
                
                let output1: IDXGIOutput1 = output.cast()
                    .map_err(|e| ClientError::CaptureError(format!("Failed to cast to IDXGIOutput1: {}", e)))?;
                
                // Get output description
                // TODO: Get actual screen dimensions from DXGI
                // For now, use common 1920x1080 default
                self.width = 1920;
                self.height = 1080;
                
                // Create desktop duplication
                let duplication = output1.DuplicateOutput(device)
                    .map_err(|e| ClientError::CaptureError(format!("Failed to create duplication: {}", e)))?;
                
                self.duplication = Some(duplication);
                
                // Create staging texture for CPU access
                self.create_staging_texture()?;
                
                Ok(())
            }
        }
        
        fn create_staging_texture(&mut self) -> Result<(), ClientError> {
            unsafe {
                let device = self.device.as_ref()
                    .ok_or_else(|| ClientError::CaptureError("Device not initialized".to_string()))?;
                
                let desc = D3D11_TEXTURE2D_DESC {
                    Width: self.width,
                    Height: self.height,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Usage: D3D11_USAGE_STAGING,
                    BindFlags: D3D11_BIND_FLAG(0).0 as u32,
                    CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                    MiscFlags: D3D11_RESOURCE_MISC_FLAG(0).0 as u32,
                };
                
                let mut texture: Option<ID3D11Texture2D> = None;
                device.CreateTexture2D(&desc, None, Some(&mut texture))
                    .map_err(|e| ClientError::CaptureError(format!("Failed to create staging texture: {}", e)))?;
                
                self.staging_texture = texture;
                
                Ok(())
            }
        }
    }
    
    impl Default for DxgiCapture {
        fn default() -> Self {
            Self::new()
        }
    }
    
    #[async_trait]
    impl ScreenCapture for DxgiCapture {
        async fn init(&mut self) -> Result<(), ClientError> {
            self.create_d3d_device()?;
            self.create_duplication()?;
            self.initialized = true;
            Ok(())
        }
        
        async fn capture_frame(&mut self) -> Result<Frame, ClientError> {
            if !self.initialized {
                return Err(ClientError::CaptureError("Not initialized".to_string()));
            }
            
            unsafe {
                let duplication = self.duplication.as_ref()
                    .ok_or_else(|| ClientError::CaptureError("Duplication not initialized".to_string()))?;
                
                let context = self.context.as_ref()
                    .ok_or_else(|| ClientError::CaptureError("Context not initialized".to_string()))?;
                
                let staging = self.staging_texture.as_ref()
                    .ok_or_else(|| ClientError::CaptureError("Staging texture not initialized".to_string()))?;
                
                // Acquire next frame
                let mut frame_info = mem::zeroed();
                let mut desktop_resource: Option<IDXGIResource> = None;
                
                // Try to acquire frame (with timeout)
                match duplication.AcquireNextFrame(100, &mut frame_info, &mut desktop_resource) {
                    Ok(_) => {
                        let resource = desktop_resource
                            .ok_or_else(|| ClientError::CaptureError("No desktop resource".to_string()))?;
                        
                        // Get texture from resource
                        let texture: ID3D11Texture2D = resource.cast()
                            .map_err(|e| ClientError::CaptureError(format!("Failed to cast to texture: {}", e)))?;
                        
                        // Copy to staging texture
                        context.CopyResource(staging, &texture);
                        
                        // Map staging texture to read data
                        let mut mapped = mem::zeroed();
                        context.Map(staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                            .map_err(|e| ClientError::CaptureError(format!("Failed to map texture: {}", e)))?;
                        
                        // Copy frame data
                        let row_pitch = mapped.RowPitch as usize;
                        let data_size = (self.height as usize) * row_pitch;
                        let mut data = vec![0u8; data_size];
                        
                        std::ptr::copy_nonoverlapping(
                            mapped.pData as *const u8,
                            data.as_mut_ptr(),
                            data_size,
                        );
                        
                        // Unmap
                        context.Unmap(staging, 0);
                        
                        // Release frame
                        let _ = duplication.ReleaseFrame();
                        
                        Ok(Frame {
                            width: self.width,
                            height: self.height,
                            stride: row_pitch as u32,
                            data,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                        })
                    }
                    Err(e) => {
                        // Handle timeout or other errors
                        if e.code() == DXGI_ERROR_WAIT_TIMEOUT {
                            // No new frame, return previous frame or empty
                            Err(ClientError::CaptureError("Frame timeout".to_string()))
                        } else if e.code() == DXGI_ERROR_ACCESS_LOST {
                            // Need to recreate duplication
                            self.initialized = false;
                            Err(ClientError::CaptureError("Access lost, need to reinitialize".to_string()))
                        } else {
                            Err(ClientError::CaptureError(format!("Failed to acquire frame: {}", e)))
                        }
                    }
                }
            }
        }
        
        fn get_dimensions(&self) -> (u32, u32) {
            (self.width, self.height)
        }
        
        async fn cleanup(&mut self) -> Result<(), ClientError> {
            self.duplication = None;
            self.staging_texture = None;
            self.context = None;
            self.device = None;
            self.initialized = false;
            Ok(())
        }
    }
}

/// Create platform-specific screen capture
pub fn create_screen_capture() -> Result<Box<dyn ScreenCapture>, ClientError> {
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(win_impl::DxgiCapture::new()))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err(ClientError::PlatformNotSupported)
    }
}
