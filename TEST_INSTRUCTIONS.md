# GenXLink Test Instructions

## Quick Test (5 minutes)

### 1. Build the Project

```powershell
# Run the automated build script
.\build.ps1
```

Expected output:
- ✓ Build successful
- ⚠ Some tests may fail (expected for incomplete features)

### 2. Test Screen Capture

```powershell
cd client\windows
cargo run --example screen_capture_test
```

Expected output:
```
=== GenXLink Screen Capture Test ===

Initializing screen capture...
Screen resolution: 1920x1080
Starting capture for 10 seconds...

Captured 30 frames... FPS: 29.85
Captured 60 frames... FPS: 30.12
...

=== Performance Stats ===
FPS: 30.05
Avg Frame Time: 33.28 ms

✓ PASS: Screen capture working well!
```

### 3. Run the Windows Client

```powershell
cd client\windows
cargo run
```

Expected output:
```
=== GenXLink Remote Desktop ===
Device ID: [Your unique device ID]

No license activated. Running in Free mode.
Session limit: 10 minutes

Commands:
  1. Connect to remote device
  2. Wait for incoming connection
  3. Activate license
  4. Exit
```

## Detailed Testing

### Unit Tests

```bash
# Test all modules
cargo test --workspace

# Test specific module
cargo test -p genxlink-client-core
cargo test -p genxlink-protocol
cargo test -p genxlink-crypto
cargo test -p genxlink-licensing
```

### Performance Tests

```bash
# Run performance monitoring tests
cargo test -p genxlink-client-core performance

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Coming soon - Phase 3
cargo test --test integration
```

## Manual Testing Checklist

### Screen Capture

- [ ] Initializes without errors
- [ ] Captures at target FPS (30)
- [ ] Handles display changes
- [ ] Recovers from driver updates
- [ ] Works on multiple monitors
- [ ] CPU usage < 15%
- [ ] Memory usage < 200MB

### Performance

- [ ] FPS >= 25
- [ ] Frame time < 40ms average
- [ ] Drop rate < 5%
- [ ] No memory leaks
- [ ] Stable over time

### Error Handling

- [ ] Graceful initialization failure
- [ ] Timeout handling works
- [ ] Access lost recovery
- [ ] Clean shutdown

## Troubleshooting Tests

### Test 1: Driver Update Simulation

1. Start screen capture test
2. Change display resolution during capture
3. Verify automatic recovery

### Test 2: Multi-Monitor

1. Connect second monitor
2. Run screen capture test
3. Verify correct resolution detection

### Test 3: Performance Under Load

1. Open multiple applications
2. Run screen capture test
3. Verify FPS remains stable

## Expected Results

### Pass Criteria

✅ **Screen Capture**
- FPS: 25-35
- Frame time: 30-40ms
- Drop rate: <5%
- No crashes

✅ **Performance**
- CPU: <15%
- Memory: <200MB
- Stable over 10+ minutes

✅ **Error Handling**
- Recovers from access lost
- Handles timeouts gracefully
- Clean shutdown

### Known Limitations

⚠️ **Current Phase**
- Video encoding not yet implemented
- No network transmission
- No remote control yet
- UI is console-based

## Reporting Issues

If tests fail, please report:

1. **System Information**
   - Windows version
   - Graphics card
   - Driver version
   - Screen resolution

2. **Test Output**
   - Full console output
   - Error messages
   - Performance stats

3. **Steps to Reproduce**
   - Exact commands run
   - System state
   - Any changes made

## Next Testing Phase

After FFmpeg integration:

- [ ] Video encoding tests
- [ ] End-to-end capture + encode
- [ ] Bitrate testing
- [ ] Quality evaluation
- [ ] Hardware acceleration tests

---

**Current Status:** Screen capture fully functional and ready for encoding integration! 🎉
