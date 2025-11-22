# GenXLink Windows Client

Windows client application for GenXLink remote desktop.

## Features

- **Screen Capture**: Windows DXGI Desktop Duplication
- **Input Injection**: Keyboard and mouse control
- **License Management**: Online and offline activation
- **Performance Monitoring**: FPS and latency tracking

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Examples

### Screen Capture Test

Test the screen capture functionality:

```bash
cargo run --example screen_capture_test
```

This will:
- Initialize DXGI screen capture
- Capture frames for 10 seconds
- Display performance metrics
- Evaluate capture quality

Expected output:
```
=== GenXLink Screen Capture Test ===

Initializing screen capture...
Screen resolution: 1920x1080
Starting capture for 10 seconds...

Captured 30 frames... FPS: 29.85
Captured 60 frames... FPS: 30.12
...

=== Test Complete ===
Total frames captured: 300
Errors encountered: 0

=== Performance Stats ===
FPS: 30.05
Avg Frame Time: 33.28 ms
Min Frame Time: 32.10 ms
Max Frame Time: 35.42 ms
Total Frames: 300
Dropped Frames: 0 (0.00%)

=== Evaluation ===
✓ PASS: Screen capture working well!
```

## Requirements

- Windows 10 or later
- DirectX 11 compatible graphics card
- Visual Studio 2022 (for building)

## Configuration

Edit `src/config.rs` to change:
- Server URL
- Video quality settings
- Network configuration

## Troubleshooting

### "Failed to create D3D11 device"

- Update graphics drivers
- Ensure DirectX 11 is installed
- Check if running in a VM (may not support DXGI)

### "Access lost, need to reinitialize"

- This happens when display settings change
- The application will automatically reinitialize
- Common during driver updates or display changes

### High CPU usage

- Lower the FPS in configuration
- Reduce resolution
- Enable hardware encoding (when implemented)

## Development

### Adding Features

1. Core functionality goes in `../core/src/`
2. Windows-specific code in `src/`
3. Update tests in `tests/`

### Running Tests

```bash
cargo test
```

### Benchmarking

```bash
cargo bench
```

## License

Apache License 2.0 - See LICENSE file for details
