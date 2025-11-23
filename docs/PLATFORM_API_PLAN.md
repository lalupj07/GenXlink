# 🖥️ Platform-Specific API Implementation Plan

## 📋 Overview

Detailed implementation plan for platform-specific APIs needed for GenXLink's core features.

---

## 🎯 Implementation Priority

### P0 (Critical - Week 1-2)
- **Screen Capture** - Windows DXGI (3-4 days)
- **Input Injection** - Keyboard & Mouse (2-3 days)

### P1 (High - Week 3-4)
- **Audio Streaming** - WASAPI (3-4 days)
- **Clipboard Sync** - Windows Clipboard API (1-2 days)

### P2 (Medium - Week 5-6)
- **File Transfer** - Native file dialogs (2-3 days)
- **Multi-monitor** - Display enumeration (1-2 days)

---

## 🪟 Windows Implementation

### 1. Screen Capture (DXGI)

**Timeline:** 3-4 days  
**Complexity:** High  
**Dependencies:** windows-rs, openh264

**File Structure:**
```
client/windows/src/capture/
├── mod.rs
├── dxgi.rs          # DXGI implementation
├── encoder.rs       # H.264 encoding
└── frame_buffer.rs  # Frame management
```

**Cargo.toml additions:**
```toml
[dependencies.windows]
version = "0.52"
features = [
    "Win32_Graphics_Direct3D11",
    "Win32_Graphics_Dxgi",
]

openh264 = "0.5"
```

**Key APIs:**
- `D3D11CreateDevice` - Create Direct3D device
- `IDXGIOutput1::DuplicateOutput` - Desktop duplication
- `IDXGIOutputDuplication::AcquireNextFrame` - Get frame
- `ID3D11DeviceContext::CopyResource` - Copy to CPU

**Implementation checklist:**
- [ ] Initialize D3D11 device
- [ ] Create desktop duplication
- [ ] Capture frames at 60 FPS
- [ ] Encode to H.264
- [ ] Send via WebRTC

---

### 2. Input Injection

**Timeline:** 2-3 days  
**Complexity:** Medium  
**Dependencies:** windows-rs

**File Structure:**
```
client/windows/src/input/
├── mod.rs
├── keyboard.rs      # Keyboard injection
├── mouse.rs         # Mouse injection
└── types.rs         # Input types
```

**Cargo.toml additions:**
```toml
[dependencies.windows]
features = [
    "Win32_UI_Input_KeyboardAndMouse",
]
```

**Key APIs:**
- `SendInput` - Inject input events
- `VkKeyScan` - Virtual key codes
- `GetCursorPos` - Current cursor position
- `SetCursorPos` - Set cursor position

**Implementation checklist:**
- [ ] Receive input events from WebRTC
- [ ] Convert to Windows INPUT structures
- [ ] Inject keyboard events
- [ ] Inject mouse events (move, click, scroll)
- [ ] Handle special keys (Ctrl, Alt, Win)

---

### 3. Audio Streaming (WASAPI)

**Timeline:** 3-4 days  
**Complexity:** High  
**Dependencies:** windows-rs, opus

**File Structure:**
```
client/windows/src/audio/
├── mod.rs
├── wasapi.rs        # WASAPI capture
├── encoder.rs       # Opus encoding
└── player.rs        # Audio playback
```

**Cargo.toml additions:**
```toml
[dependencies.windows]
features = [
    "Win32_Media_Audio",
]

opus = "0.3"
cpal = "0.15"  # Cross-platform audio
```

**Key APIs:**
- `IMMDeviceEnumerator` - Enumerate audio devices
- `IAudioClient::Initialize` - Initialize audio client
- `IAudioCaptureClient::GetBuffer` - Get audio buffer
- `IAudioRenderClient::GetBuffer` - Render audio

**Implementation checklist:**
- [ ] Enumerate audio devices
- [ ] Capture system audio
- [ ] Capture microphone
- [ ] Encode to Opus
- [ ] Send via WebRTC
- [ ] Receive and decode audio
- [ ] Play audio output

---

### 4. Clipboard Sync

**Timeline:** 1-2 days  
**Complexity:** Low  
**Dependencies:** windows-rs

**File Structure:**
```
client/windows/src/clipboard/
├── mod.rs
└── sync.rs          # Clipboard sync
```

**Cargo.toml additions:**
```toml
[dependencies.windows]
features = [
    "Win32_System_DataExchange",
]
```

**Key APIs:**
- `OpenClipboard` - Open clipboard
- `GetClipboardData` - Get clipboard data
- `SetClipboardData` - Set clipboard data
- `AddClipboardFormatListener` - Monitor changes

**Implementation checklist:**
- [ ] Monitor clipboard changes
- [ ] Read text from clipboard
- [ ] Read images from clipboard
- [ ] Send clipboard data via WebRTC
- [ ] Receive and set clipboard data
- [ ] Handle clipboard formats

---

### 5. File Transfer

**Timeline:** 2-3 days  
**Complexity:** Medium  
**Dependencies:** windows-rs

**File Structure:**
```
client/windows/src/transfer/
├── mod.rs
├── sender.rs        # File sending
├── receiver.rs      # File receiving
└── progress.rs      # Progress tracking
```

**Key APIs:**
- `IFileDialog` - File picker dialog
- `CreateFile` - Open file
- `ReadFile` / `WriteFile` - Read/write file
- `GetFileSizeEx` - Get file size

**Implementation checklist:**
- [ ] Open file picker dialog
- [ ] Read file in chunks (1MB)
- [ ] Calculate checksums (SHA-256)
- [ ] Send via WebRTC data channel
- [ ] Show progress bar
- [ ] Receive and save files
- [ ] Verify checksums

---

### 6. Multi-monitor Support

**Timeline:** 1-2 days  
**Complexity:** Low  
**Dependencies:** windows-rs

**File Structure:**
```
client/windows/src/display/
├── mod.rs
└── monitors.rs      # Monitor enumeration
```

**Key APIs:**
- `EnumDisplayMonitors` - Enumerate monitors
- `GetMonitorInfo` - Get monitor info
- `MonitorFromPoint` - Get monitor from point

**Implementation checklist:**
- [ ] Enumerate all monitors
- [ ] Get monitor resolution
- [ ] Get monitor position
- [ ] Allow user to select monitor
- [ ] Capture specific monitor

---

## 🍎 macOS Implementation (Future)

### Screen Capture
- **API:** `CGDisplayStream`
- **Timeline:** 3-4 days

### Input Injection
- **API:** `CGEventPost`
- **Timeline:** 2-3 days

### Audio
- **API:** `CoreAudio`
- **Timeline:** 3-4 days

---

## 🐧 Linux Implementation (Future)

### Screen Capture
- **API:** X11 or Wayland
- **Timeline:** 3-4 days

### Input Injection
- **API:** `XTest` extension
- **Timeline:** 2-3 days

### Audio
- **API:** PulseAudio / PipeWire
- **Timeline:** 3-4 days

---

## 📊 Total Timeline Estimate

### Windows (Priority)
- **Week 1-2:** Screen capture + Input injection (5-7 days)
- **Week 3-4:** Audio + Clipboard (4-6 days)
- **Week 5-6:** File transfer + Multi-monitor (3-5 days)
- **Total:** 12-18 days (2.5-4 weeks)

### Cross-platform
- **Week 7-10:** macOS support (12-18 days)
- **Week 11-14:** Linux support (12-18 days)
- **Total:** 36-54 days (7-11 weeks)

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_screen_capture() {
        let capture = DxgiCapture::new(0).unwrap();
        let frame = capture.capture_frame().unwrap();
        assert!(frame.width > 0);
        assert!(frame.height > 0);
    }
}
```

### Integration Tests
- Test full capture → encode → send → receive → decode flow
- Test input injection with virtual machine
- Test audio capture and playback

### Performance Tests
- Measure FPS (target: 60 FPS)
- Measure latency (target: <100ms)
- Measure CPU usage (target: <30%)
- Measure bandwidth (target: <5 Mbps)

---

## 📚 Resources

### Documentation
- [Windows DXGI](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/dx-graphics-dxgi)
- [Windows Input](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [WASAPI](https://docs.microsoft.com/en-us/windows/win32/coreaudio/wasapi)

### Libraries
- [windows-rs](https://github.com/microsoft/windows-rs)
- [openh264](https://github.com/cisco/openh264)
- [opus](https://opus-codec.org/)

---

## 🎯 Success Criteria

### Screen Capture
- ✅ 60 FPS on 1080p display
- ✅ <100ms latency
- ✅ <30% CPU usage
- ✅ Hardware acceleration working

### Input Injection
- ✅ <50ms input latency
- ✅ All keys working (including special keys)
- ✅ Mouse movement smooth
- ✅ Scroll wheel working

### Audio
- ✅ Clear audio quality
- ✅ <200ms audio latency
- ✅ No audio dropouts
- ✅ System and mic audio working

---

**Ready to implement! Start with Screen Capture (P0)** 🚀
