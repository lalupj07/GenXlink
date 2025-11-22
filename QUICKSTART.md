# GenXLink Quick Start Guide

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository
- **Visual Studio 2022** (Windows) - For Windows development
- **PostgreSQL 14+** (optional) - For running servers
- **Redis 7+** (optional) - For running servers

## Quick Setup (5 minutes)

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/genxlink.git
cd genxlink
```

### 2. Build the Project

**Windows (PowerShell):**
```powershell
.\build.ps1
```

**Manual Build:**
```bash
cargo build --workspace
```

### 3. Run the Windows Client

```bash
cd client/windows
cargo run
```

You should see:
```
=== GenXLink Remote Desktop ===
Device ID: [Your unique device ID]
No license activated. Running in Free mode.
Session limit: 10 minutes
```

## Project Status

✅ **Completed:**
- Project structure and workspace setup
- Core protocol definitions
- Cryptography module (AES-256, RSA)
- Licensing system framework
- Client core modules (placeholders)
- Server API endpoints (skeleton)
- Comprehensive documentation

🚧 **In Progress:**
- Windows DXGI screen capture implementation
- FFmpeg H.264 encoder integration
- WebRTC transport layer
- Database layer with SQLx

📋 **Planned:**
- Tauri UI for Windows
- Android client
- Full server implementation
- Production deployment

## Architecture Overview

```
GenXLink/
├── client/
│   ├── core/           # Cross-platform logic
│   │   ├── capture/    # Screen capture
│   │   ├── encoder/    # H.264 encoding
│   │   ├── input/      # Keyboard/mouse injection
│   │   └── transport/  # WebRTC communication
│   └── windows/        # Windows-specific client
│
├── server/
│   ├── api/            # REST API (auth, licensing)
│   ├── signaling/      # WebRTC signaling
│   └── relay/          # TURN relay server
│
└── shared/
    ├── protocol/       # Message definitions
    ├── crypto/         # Encryption utilities
    └── licensing/      # License validation
```

## Key Technologies

- **Language:** Rust (performance + safety)
- **Async:** Tokio
- **Web:** Axum
- **Database:** PostgreSQL + Redis
- **Transport:** WebRTC
- **Encoding:** FFmpeg H.264
- **UI:** Tauri (planned)

## Development Workflow

### Running Tests

```bash
cargo test --workspace
```

### Code Formatting

```bash
cargo fmt --all
```

### Linting

```bash
cargo clippy --workspace -- -D warnings
```

### Building for Release

```bash
cargo build --release --workspace
```

Release binaries will be in `target/release/`

## Next Development Steps

### Phase 2: Screen Capture & Encoding (Current)

**Priority Tasks:**

1. **Implement Windows DXGI Screen Capture**
   - File: `client/core/src/capture.rs`
   - Use Windows Desktop Duplication API
   - Capture at 30 FPS

2. **Integrate FFmpeg H.264 Encoder**
   - File: `client/core/src/encoder.rs`
   - Add FFmpeg bindings
   - Configure for low latency

3. **Complete WebRTC Transport**
   - File: `client/core/src/transport.rs`
   - Implement peer connection
   - Add data channels

### Testing Your Changes

```bash
# Test specific module
cargo test -p genxlink-client-core

# Test with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Common Issues & Solutions

### Build Errors

**Issue: FFmpeg not found**
```bash
# Windows (using vcpkg)
vcpkg install ffmpeg:x64-windows
```

**Issue: OpenSSL errors**
```bash
# Windows (using vcpkg)
vcpkg install openssl:x64-windows
```

**Issue: Windows SDK not found**
- Install Visual Studio 2022 with C++ Desktop Development workload

### Runtime Errors

**Issue: "Device ID generation failed"**
- Ensure you have admin rights (for registry access)
- Check Windows version compatibility

**Issue: "License validation failed"**
- RSA keys not configured yet (expected in Phase 1)
- Will be implemented in Phase 6

## Configuration

Default configuration is in `client/windows/src/config.rs`:

```rust
Config {
    server_url: "wss://localhost:8080",
    video: VideoConfig {
        width: 1920,
        height: 1080,
        fps: 30,
        bitrate: 2_000_000,
    },
}
```

## License

This project is licensed under **Apache License 2.0**.

Key points:
- ✅ Commercial use allowed
- ✅ Modification allowed
- ✅ Distribution allowed
- ✅ Patent grant included
- ⚠️ Must include license and copyright notice
- ⚠️ Must state changes made

See [LICENSE](LICENSE) for full terms.

## Getting Help

- **Documentation:** See `/docs` folder
- **Issues:** GitHub Issues
- **Discussions:** GitHub Discussions
- **Email:** dev@genxis.com

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Quick checklist:
- [ ] Fork the repository
- [ ] Create a feature branch
- [ ] Write tests for new features
- [ ] Run `cargo fmt` and `cargo clippy`
- [ ] Update documentation
- [ ] Submit a pull request

## Resources

### Documentation
- [API Documentation](docs/API.md)
- [Database Schema](docs/DATABASE_SCHEMA.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Roadmap](docs/ROADMAP.md)

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [WebRTC Specification](https://www.w3.org/TR/webrtc/)
- [Windows DXGI](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/d3d10-graphics-programming-guide-dxgi)

## What's Working Now

✅ Project compiles successfully  
✅ All workspace members build  
✅ Core libraries functional  
✅ License system framework ready  
✅ Server endpoints defined  
✅ Client structure complete  

## What's Next

The foundation is solid. Now we need to:

1. **Implement actual screen capture** using Windows APIs
2. **Add FFmpeg integration** for encoding
3. **Complete WebRTC** peer connections
4. **Build database layer** for licensing
5. **Create UI** with Tauri

Follow the [ROADMAP.md](docs/ROADMAP.md) for the complete development plan.

---

**Ready to contribute?** Pick a task from Phase 2 and start coding! 🚀
