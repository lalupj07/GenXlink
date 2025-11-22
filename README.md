# GenXLink - Lightweight Remote Desktop Solution

A cross-platform, lightweight remote desktop application optimized for small binary size, low latency, privacy, and licensing support.

## Features

- **Desktop Screen Sharing** - High-quality screen streaming with H.264 encoding
- **Remote Control** - Full keyboard and mouse control
- **Secure P2P Communication** - End-to-end encryption with AES-256 + DTLS
- **NAT Traversal** - Automatic peer-to-peer connection with relay fallback
- **Licensing System** - Flexible activation with online/offline support
- **Multi-Platform** - Windows and Android support
- **Self-Hosted** - Deploy your own servers

## Architecture

```
GenXLink/
├── client/          # Windows & Android clients
├── server/          # Backend services
├── shared/          # Shared libraries
└── docs/            # Documentation
```

## Technology Stack

### Client (Windows)
- **Language**: Rust
- **Screen Capture**: Windows DXGI/Desktop Duplication
- **Encoder**: FFmpeg H.264
- **Transport**: WebRTC
- **UI**: Tauri

### Client (Android)
- **Language**: Kotlin + Rust
- **Screen Capture**: MediaProjection
- **Encoder**: Hardware H.264
- **Transport**: WebRTC

### Server
- **Language**: Rust
- **Database**: PostgreSQL + Redis
- **Deployment**: Docker

## License Tiers

### Free Tier
- 10 min/session
- Basic screen sharing
- No file transfer

### Pro Tier
- Unlimited sessions
- Unattended access
- File transfer
- Priority relay

## Binary Size Targets

- Windows: 6-10 MB
- Android: 8-15 MB

## Development Phases

1. **Foundation** - Project structure, device IDs, WebRTC signaling
2. **Screen Control** - Screen capture, encoding, input injection
3. **Servers** - Rendezvous, relay, NAT traversal
4. **Licensing** - Activation system, premium features
5. **Final App** - Branding, installers, optimization

## 🚀 Getting Started

### Quick Start (5 minutes)

1. **Install Rust** (if not already installed)
   ```bash
   # Visit https://rustup.rs/ and run the installer
   ```

2. **Setup the project**
   ```powershell
   cd GenXlink
   .\scripts\setup.ps1
   ```

3. **Test screen capture**
   ```powershell
   cd client\windows
   cargo run --example screen_capture_test
   ```

### Documentation

📖 **Start here:** [INDEX.md](INDEX.md) - Complete documentation index

**Quick Links:**
- [QUICKSTART.md](QUICKSTART.md) - 5-minute setup guide
- [STATUS.md](STATUS.md) - Current project status
- [TEST_INSTRUCTIONS.md](TEST_INSTRUCTIONS.md) - How to test
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [docs/](docs/) - Full technical documentation

## 📊 Current Status

| Component | Status | Progress |
|-----------|--------|----------|
| Foundation | ✅ Complete | 100% |
| Screen Capture | ✅ Complete | 100% |
| Video Encoding | ⏳ Next | 0% |
| **Overall** | 🚧 In Progress | **25%** |

**Latest:** Working screen capture at 30 FPS with full DXGI implementation!

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Current needs:**
- FFmpeg integration
- Hardware encoding
- Testing
- Documentation improvements

## 📞 Support

- **Documentation:** [INDEX.md](INDEX.md)
- **Issues:** GitHub Issues (add URL)
- **Discussions:** GitHub Discussions (add URL)
- **Email:** dev@genxis.com

## 📄 License

Apache License 2.0 - See [LICENSE](LICENSE) file for details

---

**Built with ❤️ by GenXis Innovations**
