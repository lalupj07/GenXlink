# GenXLink - Quick Reference Card

**Version:** 0.1.0 | **License:** Apache 2.0 | **Status:** Phase 2 (25% Complete)

---

## 🚀 Essential Commands

### Build & Test

```powershell
# Build everything
cargo build --workspace

# Build in release mode (optimized)
cargo build --workspace --release

# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace -- -D warnings
```

### Run Applications

```powershell
# Test screen capture (10 seconds)
cd client\windows
cargo run --example screen_capture_test

# Run Windows client
cd client\windows
cargo run

# Run API server
cd server\api
cargo run

# Run signaling server
cd server\signaling
cargo run
```

---

## 📁 Project Structure

```
GenXlink/
├── client/
│   ├── core/              # Cross-platform logic
│   │   ├── capture.rs     # Screen capture ✅
│   │   ├── encoder.rs     # Video encoding (TODO)
│   │   ├── input.rs       # Input injection ✅
│   │   ├── transport.rs   # WebRTC (TODO)
│   │   └── performance.rs # Monitoring ✅
│   └── windows/           # Windows client ✅
│
├── server/
│   ├── api/               # REST API ✅
│   ├── signaling/         # WebRTC signaling ✅
│   └── relay/             # TURN relay ✅
│
├── shared/
│   ├── protocol/          # Protocol definitions ✅
│   ├── crypto/            # Encryption ✅
│   └── licensing/         # License management ✅
│
├── docs/                  # Documentation ✅
├── scripts/               # Automation ✅
└── [Root]                 # Config & docs ✅
```

---

## 📚 Documentation Quick Links

| Document | Purpose |
|----------|---------|
| [INDEX.md](INDEX.md) | Complete navigation |
| [QUICKSTART.md](QUICKSTART.md) | 5-minute setup |
| [STATUS.md](STATUS.md) | Current status |
| [PROGRESS.md](PROGRESS.md) | Detailed progress |
| [TEST_INSTRUCTIONS.md](TEST_INSTRUCTIONS.md) | Testing guide |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [docs/PHASE2_TASKS.md](docs/PHASE2_TASKS.md) | Current tasks |

---

## 🎯 Current Phase: Phase 2 - Video Encoding

### Completed ✅
- Screen capture (DXGI)
- Performance monitoring
- Test infrastructure

### In Progress 🚧
- FFmpeg integration
- H.264 encoder
- Hardware acceleration

### Next Steps
1. Add ffmpeg-next dependency
2. Implement H.264 encoder
3. Test encoding pipeline
4. Optimize performance

---

## 🐛 Common Issues

### Build Errors

**Issue:** `cargo: command not found`
```powershell
# Add Rust to PATH
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
```

**Issue:** `failed to compile`
```powershell
# Clean and rebuild
cargo clean
cargo build --workspace
```

**Issue:** `linker error`
- Install Visual Studio 2022 with C++ tools
- Or install Windows SDK

### Runtime Errors

**Issue:** Screen capture fails
- Requires Windows 10/11
- Needs DirectX 11 support
- Update graphics drivers

**Issue:** Access denied
- Run as administrator (for device ID generation)

---

## 📊 Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Screen Capture FPS | 30 | ✅ 30 |
| CPU Usage | <10% | ~15% |
| Memory Usage | <150MB | ~200MB |
| Latency | <100ms | TBD |
| Binary Size | 5-12MB | TBD |

---

## 🔧 Development Workflow

### 1. Make Changes
```powershell
# Edit files in your IDE
# Save changes
```

### 2. Format & Lint
```powershell
cargo fmt --all
cargo clippy --workspace
```

### 3. Test
```powershell
cargo test --workspace
```

### 4. Build
```powershell
cargo build --workspace
```

### 5. Run
```powershell
cd client\windows
cargo run
```

---

## 🧪 Testing Checklist

- [ ] Unit tests pass
- [ ] Screen capture test passes
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Documentation updated
- [ ] Performance acceptable

---

## 🎓 Learning Resources

**Rust:**
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

**Project:**
- [DEVELOPMENT.md](docs/DEVELOPMENT.md) - Dev guide
- [API.md](docs/API.md) - API docs
- [ROADMAP.md](docs/ROADMAP.md) - Project plan

**Windows:**
- [DXGI Documentation](https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/)
- [Windows Crate](https://docs.rs/windows/latest/windows/)

---

## 💡 Tips & Tricks

### Speed Up Builds
```powershell
# Use release mode for faster runtime
cargo build --release

# Build specific package only
cargo build -p genxlink-client-core

# Use cargo-watch for auto-rebuild
cargo install cargo-watch
cargo watch -x build
```

### Debug Output
```powershell
# Enable tracing
$env:RUST_LOG="debug"
cargo run

# More verbose
$env:RUST_LOG="trace"
cargo run
```

### Clean Build
```powershell
# Remove build artifacts
cargo clean

# Remove and rebuild
cargo clean && cargo build --workspace
```

---

## 📞 Quick Help

**Build failing?** → Check [DEVELOPMENT.md](docs/DEVELOPMENT.md)  
**Test failing?** → Check [TEST_INSTRUCTIONS.md](TEST_INSTRUCTIONS.md)  
**Need to contribute?** → Check [CONTRIBUTING.md](CONTRIBUTING.md)  
**Lost?** → Check [INDEX.md](INDEX.md)

---

## 🎯 Current Sprint Goals

**This Week:**
- [ ] Integrate FFmpeg
- [ ] Implement H.264 encoder
- [ ] Test encoding pipeline
- [ ] Measure performance

**Success Criteria:**
- 30 FPS encoding
- <20% CPU usage
- Hardware acceleration working
- All tests passing

---

## 📈 Project Stats

- **Files:** 65+
- **Lines of Code:** ~5,500
- **Modules:** 12
- **Documentation:** 18 files
- **Progress:** 25% (Phase 2 of 11)
- **Status:** 🟢 Healthy

---

**Last Updated:** November 23, 2024

**Keep this file handy for quick reference during development!** 📌
