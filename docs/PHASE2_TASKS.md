# Phase 2: Screen Capture & Encoding - Task List

## Overview

Phase 2 focuses on implementing the core screen capture and encoding functionality for Windows. This is the foundation for remote desktop streaming.

**Timeline:** 2-3 weeks  
**Status:** Ready to start  
**Prerequisites:** Phase 1 complete ✅

## Tasks

### 1. Windows DXGI Screen Capture

**File:** `client/core/src/capture.rs`  
**Priority:** HIGH  
**Estimated Time:** 3-4 days

#### Requirements
- Implement Desktop Duplication API
- Capture full screen at configurable FPS (15/30/60)
- Handle multi-monitor scenarios
- Efficient frame buffer management
- Error handling for driver updates/display changes

#### Implementation Steps

1. **Initialize DXGI**
   ```rust
   // Create D3D11 device
   // Get DXGI adapter
   // Get output (monitor)
   // Create desktop duplication
   ```

2. **Frame Capture Loop**
   ```rust
   // Acquire next frame
   // Get frame data
   // Copy to buffer
   // Release frame
   ```

3. **Optimization**
   - Use dirty rectangles for partial updates
   - Implement frame skipping for low bandwidth
   - Add frame rate limiting

#### Resources
- [DXGI Desktop Duplication](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [Rust Windows Crate](https://docs.rs/windows/latest/windows/)

#### Acceptance Criteria
- [ ] Captures full screen at 30 FPS
- [ ] Works on Windows 10/11
- [ ] Handles display changes gracefully
- [ ] Memory usage < 100 MB
- [ ] CPU usage < 10% on modern hardware

---

### 2. FFmpeg H.264 Encoder Integration

**File:** `client/core/src/encoder.rs`  
**Priority:** HIGH  
**Estimated Time:** 3-4 days

#### Requirements
- Integrate FFmpeg library
- Configure H.264 encoder for low latency
- Support hardware acceleration (Intel QSV, NVIDIA NVENC, AMD VCE)
- Dynamic bitrate adjustment
- Keyframe interval control

#### Implementation Steps

1. **Add FFmpeg Dependency**
   ```toml
   [dependencies]
   ffmpeg-next = "6.0"
   ```

2. **Initialize Encoder**
   ```rust
   // Create encoder context
   // Set codec parameters (H.264)
   // Configure for low latency
   // Open encoder
   ```

3. **Encode Frames**
   ```rust
   // Convert frame format (BGRA -> YUV420)
   // Encode frame
   // Get encoded packet
   // Return compressed data
   ```

4. **Hardware Acceleration**
   - Detect available encoders
   - Prefer hardware over software
   - Fallback to software if needed

#### Resources
- [FFmpeg Documentation](https://ffmpeg.org/documentation.html)
- [ffmpeg-next Crate](https://docs.rs/ffmpeg-next/)
- [H.264 Encoding Guide](https://trac.ffmpeg.org/wiki/Encode/H.264)

#### Acceptance Criteria
- [ ] Encodes 1080p at 30 FPS
- [ ] Latency < 50ms
- [ ] Bitrate: 1-5 Mbps (configurable)
- [ ] Hardware acceleration working
- [ ] Graceful fallback to software

---

### 3. Frame Buffer Management

**File:** `client/core/src/capture.rs`  
**Priority:** MEDIUM  
**Estimated Time:** 2 days

#### Requirements
- Efficient frame queue
- Memory pooling to avoid allocations
- Thread-safe buffer access
- Backpressure handling

#### Implementation Steps

1. **Create Frame Pool**
   ```rust
   struct FramePool {
       available: Vec<Frame>,
       in_use: Vec<Frame>,
   }
   ```

2. **Implement Queue**
   ```rust
   // Producer: Screen capture
   // Consumer: Encoder
   // Use tokio channels
   ```

#### Acceptance Criteria
- [ ] No frame drops at 30 FPS
- [ ] Memory usage stable
- [ ] Thread-safe operations

---

### 4. Performance Optimization

**File:** Multiple  
**Priority:** MEDIUM  
**Estimated Time:** 2-3 days

#### Requirements
- Profile CPU usage
- Profile memory usage
- Optimize hot paths
- Reduce allocations

#### Tasks
- [ ] Add performance benchmarks
- [ ] Profile with cargo-flamegraph
- [ ] Optimize frame copying
- [ ] Reduce memory allocations
- [ ] Add performance metrics

#### Target Metrics
- CPU: < 10% on Intel i5 or equivalent
- Memory: < 150 MB total
- Latency: < 100ms end-to-end

---

### 5. Error Handling & Recovery

**File:** Multiple  
**Priority:** MEDIUM  
**Estimated Time:** 2 days

#### Requirements
- Handle display driver updates
- Recover from encoder errors
- Handle system sleep/wake
- Graceful degradation

#### Implementation
```rust
// Retry logic for transient errors
// Fallback to lower quality
// Notify user of issues
```

#### Acceptance Criteria
- [ ] Recovers from driver updates
- [ ] Handles sleep/wake correctly
- [ ] No crashes on errors
- [ ] User-friendly error messages

---

### 6. Configuration & Testing

**File:** `client/core/src/encoder.rs`, tests  
**Priority:** MEDIUM  
**Estimated Time:** 2 days

#### Requirements
- Configurable resolution
- Configurable FPS
- Configurable bitrate
- Configurable codec settings

#### Configuration Structure
```rust
pub struct CaptureConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

pub struct EncoderConfig {
    pub codec: VideoCodec,
    pub bitrate: u32,
    pub preset: EncoderPreset,
    pub use_hardware: bool,
}
```

#### Testing
- [ ] Unit tests for each module
- [ ] Integration tests for capture + encode
- [ ] Performance tests
- [ ] Error condition tests

---

## Development Order

**Week 1:**
1. Day 1-2: DXGI screen capture basic implementation
2. Day 3-4: FFmpeg encoder integration
3. Day 5: Frame buffer management

**Week 2:**
1. Day 1-2: Hardware acceleration
2. Day 3: Multi-monitor support
3. Day 4-5: Error handling

**Week 3:**
1. Day 1-2: Performance optimization
2. Day 3-4: Testing and bug fixes
3. Day 5: Documentation

## Testing Strategy

### Unit Tests
```bash
cargo test -p genxlink-client-core
```

### Integration Tests
```bash
cargo test --test integration_capture_encode
```

### Performance Tests
```bash
cargo bench
```

### Manual Testing
1. Run on Windows 10
2. Run on Windows 11
3. Test with multiple monitors
4. Test with different resolutions
5. Test with driver updates

## Success Criteria

Phase 2 is complete when:

- [x] Screen capture works at 30 FPS
- [x] H.264 encoding produces valid stream
- [x] Hardware acceleration functional
- [x] Memory usage < 150 MB
- [x] CPU usage < 10%
- [x] All tests passing
- [x] Documentation updated

## Dependencies

**External:**
- FFmpeg libraries
- Windows SDK
- Visual Studio 2022

**Internal:**
- `genxlink-protocol` (for frame types)
- `genxlink-crypto` (for future encryption)

## Notes

- Focus on Windows first, Android later
- Prioritize stability over features
- Keep code modular for easy testing
- Document performance characteristics
- Add metrics for monitoring

## Next Phase

After Phase 2 completion, move to **Phase 3: Input Injection** to enable remote control functionality.
