# 🚀 GenXLink Implementation Guide

**Complete step-by-step guide for implementing authentication and Windows platform APIs**

---

## 📋 Table of Contents

1. [Authentication System](#authentication-system)
2. [Windows Screen Capture](#windows-screen-capture)
3. [Input Injection](#input-injection)
4. [Audio Streaming](#audio-streaming)
5. [Testing & Deployment](#testing--deployment)

---

## 🔐 Authentication System

### Status: ✅ **Foundation Created**

I've already added:
- ✅ JWT dependencies (`jsonwebtoken`, `bcrypt`, `actix-web-httpauth`)
- ✅ Auth module (`server/src/auth.rs`)
- ✅ Token generation and validation
- ✅ Password hashing

### Next Steps:

#### 1. Update Database Schema (Supabase)

Run this SQL in Supabase SQL Editor:

```sql
-- Users table (Supabase auth.users already exists, but we need our own)
CREATE TABLE IF NOT EXISTS app_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE
);

-- Update devices table to link to users
ALTER TABLE devices 
ADD COLUMN IF NOT EXISTS app_user_id UUID REFERENCES app_users(id) ON DELETE CASCADE;

-- Create index
CREATE INDEX IF NOT EXISTS idx_app_users_email ON app_users(email);
```

#### 2. Add Auth Endpoints to Server

Add to `server/src/main.rs`:

```rust
mod auth;

// Add these endpoints
.route("/auth/register", web::post().to(register))
.route("/auth/login", web::post().to(login))
.route("/auth/me", web::get().to(get_current_user))
```

#### 3. Implement Registration Endpoint

```rust
#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    full_name: Option<String>,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserInfo,
}

async fn register(
    req: web::Json<RegisterRequest>,
    db: web::Data<Arc<Database>>,
) -> impl Responder {
    // 1. Validate email format
    // 2. Check if user exists
    // 3. Hash password
    // 4. Insert into database
    // 5. Generate JWT token
    // 6. Return token + user info
}
```

#### 4. Implement Login Endpoint

```rust
#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

async fn login(
    req: web::Json<LoginRequest>,
    db: web::Data<Arc<Database>>,
) -> impl Responder {
    // 1. Find user by email
    // 2. Verify password
    // 3. Update last_login
    // 4. Generate JWT token
    // 5. Return token + user info
}
```

#### 5. Protect Routes with Middleware

```rust
use actix_web_httpauth::middleware::HttpAuthentication;

// Protect specific routes
.service(
    web::scope("/api")
        .wrap(HttpAuthentication::bearer(auth::validator))
        .route("/devices", web::get().to(list_devices))
        .route("/connections", web::get().to(list_connections))
)
```

---

## 🖥️ Windows Screen Capture (DXGI)

### Prerequisites

```toml
# Add to client/windows/Cargo.toml
[dependencies.windows]
version = "0.52"
features = [
    "Win32_Graphics_Direct3D11",
    "Win32_Graphics_Dxgi",
    "Win32_Graphics_Dxgi_Common",
    "Win32_Foundation",
]

openh264 = "0.5"  # For H.264 encoding
```

### Implementation Steps

#### 1. Create Capture Module

**File:** `client/windows/src/capture/mod.rs`

```rust
pub mod dxgi;
pub mod encoder;
pub mod frame;

pub use dxgi::DxgiCapture;
pub use encoder::H264Encoder;
pub use frame::Frame;
```

#### 2. DXGI Capture Implementation

**File:** `client/windows/src/capture/dxgi.rs`

```rust
use windows::{
    core::*,
    Win32::Graphics::{
        Direct3D11::*,
        Dxgi::*,
    },
};

pub struct DxgiCapture {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
    width: u32,
    height: u32,
}

impl DxgiCapture {
    pub fn new(monitor_index: u32) -> Result<Self> {
        // 1. Create D3D11 device
        let mut device = None;
        let mut context = None;
        
        unsafe {
            D3D11CreateDevice(
                None,  // Use default adapter
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
        }
        
        let device = device.unwrap();
        let context = context.unwrap();
        
        // 2. Get DXGI adapter and output
        let dxgi_device: IDXGIDevice = device.cast()?;
        let adapter = unsafe { dxgi_device.GetAdapter()? };
        let output = unsafe { adapter.EnumOutputs(monitor_index)? };
        let output1: IDXGIOutput1 = output.cast()?;
        
        // 3. Get output description for dimensions
        let desc = unsafe { output.GetDesc()? };
        let width = (desc.DesktopCoordinates.right - desc.DesktopCoordinates.left) as u32;
        let height = (desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top) as u32;
        
        // 4. Create desktop duplication
        let duplication = unsafe {
            output1.DuplicateOutput(&device)?
        };
        
        Ok(Self {
            device,
            context,
            duplication,
            width,
            height,
        })
    }
    
    pub fn capture_frame(&mut self) -> Result<Frame> {
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut desktop_resource = None;
        
        // Acquire next frame (with timeout)
        unsafe {
            self.duplication.AcquireNextFrame(
                1000,  // 1 second timeout
                &mut frame_info,
                &mut desktop_resource,
            )?;
        }
        
        let desktop_resource = desktop_resource.unwrap();
        let texture: ID3D11Texture2D = desktop_resource.cast()?;
        
        // Create staging texture for CPU access
        let staging_texture = self.create_staging_texture(&texture)?;
        
        // Copy to staging texture
        unsafe {
            self.context.CopyResource(&staging_texture, &texture);
        }
        
        // Map texture to get pixel data
        let mapped = unsafe {
            let mut mapped_resource = D3D11_MAPPED_SUBRESOURCE::default();
            self.context.Map(
                &staging_texture,
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped_resource),
            )?;
            mapped_resource
        };
        
        // Copy pixel data
        let data_size = (self.height * mapped.RowPitch) as usize;
        let mut pixel_data = vec![0u8; data_size];
        unsafe {
            std::ptr::copy_nonoverlapping(
                mapped.pData as *const u8,
                pixel_data.as_mut_ptr(),
                data_size,
            );
        }
        
        // Unmap and release
        unsafe {
            self.context.Unmap(&staging_texture, 0);
            self.duplication.ReleaseFrame()?;
        }
        
        Ok(Frame {
            width: self.width,
            height: self.height,
            data: pixel_data,
            stride: mapped.RowPitch,
        })
    }
    
    fn create_staging_texture(&self, source: &ID3D11Texture2D) -> Result<ID3D11Texture2D> {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { source.GetDesc(&mut desc) };
        
        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = D3D11_BIND_FLAG(0);
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
        desc.MiscFlags = D3D11_RESOURCE_MISC_FLAG(0);
        
        let mut staging = None;
        unsafe {
            self.device.CreateTexture2D(&desc, None, Some(&mut staging))?;
        }
        
        Ok(staging.unwrap())
    }
}
```

#### 3. Frame Structure

**File:** `client/windows/src/capture/frame.rs`

```rust
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub stride: u32,
}

impl Frame {
    pub fn to_rgb(&self) -> Vec<u8> {
        // Convert BGRA to RGB
        let mut rgb = Vec::with_capacity((self.width * self.height * 3) as usize);
        
        for y in 0..self.height {
            let row_start = (y * self.stride) as usize;
            for x in 0..self.width {
                let pixel_start = row_start + (x * 4) as usize;
                rgb.push(self.data[pixel_start + 2]); // R
                rgb.push(self.data[pixel_start + 1]); // G
                rgb.push(self.data[pixel_start]);     // B
            }
        }
        
        rgb
    }
}
```

#### 4. H.264 Encoder

**File:** `client/windows/src/capture/encoder.rs`

```rust
use openh264::encoder::{Encoder, EncoderConfig};

pub struct H264Encoder {
    encoder: Encoder,
    width: u32,
    height: u32,
}

impl H264Encoder {
    pub fn new(width: u32, height: u32, bitrate: u32) -> Result<Self> {
        let config = EncoderConfig::new(width, height)
            .bitrate_bps(bitrate)
            .max_frame_rate(60.0);
        
        let encoder = Encoder::with_config(config)?;
        
        Ok(Self {
            encoder,
            width,
            height,
        })
    }
    
    pub fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>> {
        let rgb_data = frame.to_rgb();
        let encoded = self.encoder.encode(&rgb_data)?;
        Ok(encoded.to_vec())
    }
}
```

#### 5. Integration with WebRTC

```rust
// In your WebRTC connection handler
let mut capture = DxgiCapture::new(0)?;  // Primary monitor
let mut encoder = H264Encoder::new(1920, 1080, 2_000_000)?;

loop {
    // Capture frame
    let frame = capture.capture_frame()?;
    
    // Encode to H.264
    let encoded = encoder.encode(&frame)?;
    
    // Send via WebRTC data channel
    data_channel.send(&encoded)?;
    
    // Target 60 FPS
    std::thread::sleep(std::time::Duration::from_millis(16));
}
```

---

## 🖱️ Input Injection

### Implementation

**File:** `client/windows/src/input/injection.rs`

```rust
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub struct InputInjector;

impl InputInjector {
    pub fn send_key_down(vk_code: u16) -> Result<()> {
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk_code),
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    pub fn send_key_up(vk_code: u16) -> Result<()> {
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk_code),
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    pub fn send_mouse_move(x: i32, y: i32) -> Result<()> {
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: x,
                    dy: y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    pub fn send_mouse_click(button: MouseButton) -> Result<()> {
        let (down_flag, up_flag) = match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        };
        
        // Mouse down
        let mut input_down = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: down_flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        // Mouse up
        let mut input_up = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: up_flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input_down, input_up], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}
```

---

## 🎵 Audio Streaming (WASAPI)

### Implementation

**File:** `client/windows/src/audio/wasapi.rs`

```rust
use windows::Win32::Media::Audio::*;

pub struct AudioCapture {
    client: IAudioClient,
    capture_client: IAudioCaptureClient,
}

impl AudioCapture {
    pub fn new() -> Result<Self> {
        // 1. Get default audio endpoint
        // 2. Initialize audio client
        // 3. Get capture client
        // 4. Start capture
        
        todo!("Implement WASAPI capture")
    }
    
    pub fn capture_buffer(&self) -> Result<Vec<f32>> {
        // 1. Get next buffer
        // 2. Copy audio data
        // 3. Release buffer
        
        todo!("Implement buffer capture")
    }
}
```

---

## 🧪 Testing & Deployment

### Local Testing

```bash
# Test screen capture
cd client/windows
cargo test --features capture

# Test with actual capture
cargo run --example capture_test

# Test input injection
cargo run --example input_test
```

### Integration Testing

1. Start server locally
2. Run two client instances
3. Connect them
4. Test screen sharing
5. Test remote control

### Deployment Checklist

- [ ] Authentication endpoints working
- [ ] JWT tokens validated
- [ ] Screen capture at 60 FPS
- [ ] Input injection responsive (<50ms)
- [ ] Audio streaming clear
- [ ] Database logging connections
- [ ] Error handling robust
- [ ] Memory leaks checked

---

## 📚 Resources

- [DXGI Documentation](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/dx-graphics-dxgi)
- [SendInput API](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [WASAPI Guide](https://docs.microsoft.com/en-us/windows/win32/coreaudio/wasapi)
- [JWT Best Practices](https://jwt.io/introduction)

---

**This is a comprehensive guide. Implementation will take 2-4 weeks for all features.**
