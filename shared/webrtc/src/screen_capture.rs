use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{info, error, warn, debug};
use windows::{
    core::*,
    Win32::{
        Graphics::{
            Dxgi::{IDXGIFactory, IDXGIAdapter, IDXGIOutput},
            Gdi::{HDC, HBITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB},
            Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE},
        },
        System::{
            Threading::{GetCurrentThreadId, GetCurrentProcessId},
            Diagnostics::Debug::WriteProcessMemory,
        },
        UI::{
            WindowsAndMessaging::{GetDesktopWindow, GetWindowDC, ReleaseDC},
            Shell::{Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NOTIFYICONDATAW},
        },
    },
};

use crate::media_manager::{ScreenCapture, ScreenInfo, ScreenShareConfig};

pub struct WindowsScreenCapture {
    is_capturing: Arc<Mutex<bool>>,
    capture_handle: Arc<Mutex<Option<CaptureHandle>>>,
}

struct CaptureHandle {
    stop_tx: mpsc::Sender<()>,
    config: ScreenShareConfig,
}

impl WindowsScreenCapture {
    pub fn new() -> Self {
        Self {
            is_capturing: Arc::new(Mutex::new(false)),
            capture_handle: Arc::new(Mutex::new(None)),
        }
    }

    async fn capture_screen_loop(
        config: ScreenShareConfig,
        mut stop_rx: mpsc::Receiver<()>,
        frame_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        info!("Starting screen capture with config: {:?}", config);

        // Get desktop window and device context
        let desktop_window = unsafe { GetDesktopWindow() };
        let desktop_dc = unsafe { GetWindowDC(desktop_window) };

        // Calculate screen dimensions
        let screen_width = config.width;
        let screen_height = config.height;
        let bytes_per_pixel = 4; // RGBA
        let row_size = ((screen_width as usize * bytes_per_pixel + 3) & !3);
        let buffer_size = row_size * screen_height as usize;

        // Create bitmap info
        let mut bitmap_info: BITMAPINFO = unsafe { std::mem::zeroed() };
        bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bitmap_info.bmiHeader.biWidth = screen_width as i32;
        bitmap_info.bmiHeader.biHeight = -(screen_height as i32); // Top-down
        bitmap_info.bmiHeader.biPlanes = 1;
        bitmap_info.bmiHeader.biBitCount = 32;
        bitmap_info.bmiHeader.biCompression = 0; // BI_RGB

        // Create bitmap and select into DC
        let bitmap = unsafe {
            CreateCompatibleBitmap(
                desktop_dc,
                screen_width as i32,
                screen_height as i32,
            )
        };

        if bitmap.is_null() {
            return Err(anyhow!("Failed to create bitmap"));
        }

        let mut frame_count = 0;
        let frame_interval = std::time::Duration::from_millis(1000 / config.frame_rate as u64);

        loop {
            tokio::select! {
                _ = stop_rx.recv() => {
                    info!("Received stop signal, ending screen capture");
                    break;
                }
                _ = tokio::time::sleep(frame_interval) => {
                    // Capture frame
                    match self.capture_frame(desktop_dc, &bitmap_info, bitmap, buffer_size).await {
                        Ok(frame_data) => {
                            frame_count += 1;
                            debug!("Captured frame {} ({} bytes)", frame_count, frame_data.len());
                            
                            if frame_tx.send(frame_data).await.is_err() {
                                warn!("Frame channel closed, stopping capture");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to capture frame: {}", e);
                        }
                    }
                }
            }
        }

        // Cleanup
        unsafe {
            DeleteObject(bitmap.into());
            ReleaseDC(desktop_window, desktop_dc);
        }

        info!("Screen capture loop ended");
        Ok(())
    }

    async fn capture_frame(
        desktop_dc: HDC,
        bitmap_info: &BITMAPINFO,
        bitmap: HBITMAP,
        buffer_size: usize,
    ) -> Result<Vec<u8>> {
        // Create memory DC
        let memory_dc = unsafe { CreateCompatibleDC(desktop_dc) };
        if memory_dc.is_null() {
            return Err(anyhow!("Failed to create memory DC"));
        }

        // Select bitmap into memory DC
        let old_bitmap = unsafe { SelectObject(memory_dc, bitmap.into()) };

        // BitBlt from desktop to memory DC
        let result = unsafe {
            BitBlt(
                memory_dc,
                0,
                0,
                bitmap_info.bmiHeader.biWidth,
                bitmap_info.bmiHeader.biHeight.abs(),
                desktop_dc,
                0,
                0,
                SRCCOPY,
            )
        };

        if result.is_ok() {
            // Get bitmap data
            let mut buffer = vec![0u8; buffer_size];
            let lines_copied = unsafe {
                GetDIBits(
                    memory_dc,
                    bitmap,
                    0,
                    bitmap_info.bmiHeader.biHeight.abs() as u32,
                    buffer.as_mut_ptr() as *mut _,
                    bitmap_info as *const _ as *mut _,
                    DIB_RGB_COLORS,
                )
            };

            if lines_copied > 0 {
                // Restore old bitmap and cleanup
                unsafe {
                    SelectObject(memory_dc, old_bitmap);
                    DeleteDC(memory_dc);
                }

                // Compress frame (simplified - in production use proper video compression)
                let compressed = self.compress_frame(&buffer).await?;
                Ok(compressed)
            } else {
                unsafe {
                    SelectObject(memory_dc, old_bitmap);
                    DeleteDC(memory_dc);
                };
                Err(anyhow!("Failed to get bitmap data"))
            }
        } else {
            unsafe {
                SelectObject(memory_dc, old_bitmap);
                DeleteDC(memory_dc);
            };
            Err(anyhow!("BitBlt failed"))
        }
    }

    async fn compress_frame(&self, frame_data: &[u8]) -> Result<Vec<u8>> {
        // Simplified compression - in production use VP8/H.264 encoder
        // For now, we'll just apply basic run-length encoding
        
        let mut compressed = Vec::new();
        let mut i = 0;
        
        while i < frame_data.len() {
            let current_byte = frame_data[i];
            let mut count = 1;
            
            // Count consecutive identical bytes
            while i + count < frame_data.len() && frame_data[i + count] == current_byte && count < 255 {
                count += 1;
            }
            
            if count > 3 || current_byte == 0 {
                // Use run-length encoding
                compressed.push(0); // Escape byte
                compressed.push(count as u8);
                compressed.push(current_byte);
            } else {
                // Use literal bytes
                for _ in 0..count {
                    compressed.push(current_byte);
                }
            }
            
            i += count;
        }
        
        debug!("Compressed frame from {} to {} bytes", frame_data.len(), compressed.len());
        Ok(compressed)
    }
}

#[async_trait::async_trait]
impl ScreenCapture for WindowsScreenCapture {
    async fn start_capture(&self, config: ScreenShareConfig) -> Result<mpsc::Receiver<Vec<u8>>> {
        // Check if already capturing
        let mut is_capturing = self.is_capturing.lock().await;
        if *is_capturing {
            return Err(anyhow!("Screen capture already in progress"));
        }

        *is_capturing = true;
        drop(is_capturing);

        let (frame_tx, frame_rx) = mpsc::channel(100);
        let (stop_tx, stop_rx) = mpsc::channel(1);

        // Start capture loop
        let config_clone = config.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::capture_screen_loop(config_clone, stop_rx, frame_tx).await {
                error!("Screen capture loop error: {}", e);
            }
        });

        // Store capture handle
        let mut capture_handle = self.capture_handle.lock().await;
        *capture_handle = Some(CaptureHandle {
            stop_tx,
            config,
        });

        info!("Started screen capture");
        Ok(frame_rx)
    }

    async fn stop_capture(&self) -> Result<()> {
        let mut is_capturing = self.is_capturing.lock().await;
        if !*is_capturing {
            return Ok(());
        }

        *is_capturing = false;
        drop(is_capturing);

        // Send stop signal
        let mut capture_handle = self.capture_handle.lock().await;
        if let Some(handle) = capture_handle.take() {
            if let Err(e) = handle.stop_tx.send(()).await {
                warn!("Failed to send stop signal: {}", e);
            }
        }

        info!("Stopped screen capture");
        Ok(())
    }

    async fn get_screen_list(&self) -> Result<Vec<ScreenInfo>> {
        let mut screens = Vec::new();
        
        // Use Windows APIs to enumerate displays
        unsafe {
            let mut factory: Option<IDXGIFactory> = None;
            let hr = CreateDXGIFactory(&IDXGIFactory::IID, &mut factory as *mut _ as *mut _);
            
            if hr.is_ok() {
                if let Some(factory) = factory {
                    let mut adapter_index = 0;
                    loop {
                        let mut adapter: Option<IDXGIAdapter> = None;
                        let hr = factory.EnumAdapters(adapter_index, &mut adapter);
                        
                        if hr.is_err() {
                            break;
                        }
                        
                        if let Some(adapter) = adapter {
                            let mut output_index = 0;
                            loop {
                                let mut output: Option<IDXGIOutput> = None;
                                let hr = adapter.EnumOutputs(output_index, &mut output);
                                
                                if hr.is_err() {
                                    break;
                                }
                                
                                if let Some(output) = output {
                                    let mut desc = DXGI_OUTPUT_DESC::default();
                                    let hr = output.GetDesc(&mut desc);
                                    
                                    if hr.is_ok() {
                                        let screen_info = ScreenInfo {
                                            id: format!("{}-{}", adapter_index, output_index),
                                            name: String::from_utf16_lossy(&desc.DeviceName)
                                                .trim_end_matches('\0')
                                                .to_string(),
                                            width: desc.DesktopCoordinates.right - desc.DesktopCoordinates.left,
                                            height: desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top,
                                            is_primary: adapter_index == 0 && output_index == 0,
                                        };
                                        screens.push(screen_info);
                                    }
                                }
                                output_index += 1;
                            }
                        }
                        adapter_index += 1;
                    }
                }
            }
        }

        // Fallback to primary display if enumeration fails
        if screens.is_empty() {
            screens.push(ScreenInfo {
                id: "primary".to_string(),
                name: "Primary Display".to_string(),
                width: 1920,
                height: 1080,
                is_primary: true,
            });
        }

        info!("Found {} screens", screens.len());
        Ok(screens)
    }
}

// Windows API imports
#[cfg(target_os = "windows")]
use windows::{
    core::{HRESULT, GUID},
    Win32::{
        Graphics::{
            Gdi::{
                CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
                GetDIBits, SelectObject, BitBlt, SRCCOPY, DIB_RGB_COLORS,
                HDC, HBITMAP,
            },
            Dxgi::{
                CreateDXGIFactory, IDXGIFactory, IDXGIAdapter, IDXGIOutput,
                DXGI_OUTPUT_DESC,
            },
        },
        System::Threading::{GetCurrentProcess},
        UI::WindowsAndMessaging::{GetDesktopWindow, GetWindowDC, ReleaseDC},
    },
};

#[cfg(not(target_os = "windows"))]
pub struct WindowsScreenCapture;

#[cfg(not(target_os = "windows"))]
#[async_trait::async_trait]
impl ScreenCapture for WindowsScreenCapture {
    async fn start_capture(&self, _config: ScreenShareConfig) -> Result<mpsc::Receiver<Vec<u8>>> {
        Err(anyhow!("Screen capture not supported on this platform"))
    }

    async fn stop_capture(&self) -> Result<()> {
        Ok(())
    }

    async fn get_screen_list(&self) -> Result<Vec<ScreenInfo>> {
        Ok(vec![])
    }
}
