# Development Guide

## Setup Development Environment

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- PostgreSQL 14+
- Redis 7+
- Node.js 18+ (for Tauri UI)
- Android Studio (for Android development)
- Visual Studio 2022 (for Windows development)

### Clone and Build

```bash
git clone https://github.com/your-org/genxlink.git
cd genxlink

# Build all workspace members
cargo build

# Run tests
cargo test
```

## Project Structure

```
GenXlink/
├── client/
│   ├── core/           # Cross-platform client logic
│   ├── windows/        # Windows-specific client
│   └── android/        # Android-specific client
├── server/
│   ├── api/            # REST API server
│   ├── signaling/      # WebRTC signaling server
│   └── relay/          # TURN relay server
├── shared/
│   ├── protocol/       # Protocol definitions
│   ├── crypto/         # Cryptography utilities
│   └── licensing/      # License management
└── docs/               # Documentation
```

## Development Workflow

### 1. Feature Development

Create a new branch:
```bash
git checkout -b feature/your-feature-name
```

### 2. Code Style

Follow Rust conventions:
```bash
cargo fmt
cargo clippy
```

### 3. Testing

Write tests for new features:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test code
    }
}
```

Run tests:
```bash
cargo test
```

### 4. Documentation

Document public APIs:
```rust
/// Brief description
///
/// # Arguments
///
/// * `param` - Parameter description
///
/// # Examples
///
/// ```
/// let result = function(param);
/// ```
pub fn function(param: Type) -> Result<()> {
    // Implementation
}
```

## Running Locally

### Start Database

```bash
# PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=dev postgres:14

# Redis
docker run -d -p 6379:6379 redis:7-alpine
```

### Start Servers

```bash
# API Server
cd server/api
cargo run

# Signaling Server
cd server/signaling
cargo run

# Relay Server
cd server/relay
cargo run
```

### Start Client

```bash
cd client/windows
cargo run
```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

### VS Code Configuration

`.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Windows Client",
      "cargo": {
        "args": ["build", "--bin=genxlink", "--package=genxlink-windows"]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## Performance Optimization

### Profile with cargo-flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin genxlink
```

### Benchmark

```bash
cargo bench
```

## Platform-Specific Development

### Windows

- Use Windows DXGI for screen capture
- Test with different Windows versions (10, 11)
- Handle DPI scaling properly

### Android

- Use MediaProjection API
- Test on various Android versions (8+)
- Handle different screen sizes

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

### Commit Message Format

```
type(scope): subject

body

footer
```

Types: feat, fix, docs, style, refactor, test, chore

Example:
```
feat(client): add multi-monitor support

Implement screen selection for multiple monitors
on Windows using DXGI output enumeration.

Closes #123
```

## Troubleshooting

### Build Errors

**FFmpeg not found:**
```bash
# Windows
vcpkg install ffmpeg:x64-windows

# Linux
sudo apt-get install libavcodec-dev libavformat-dev libavutil-dev
```

**OpenSSL errors:**
```bash
# Windows
vcpkg install openssl:x64-windows

# Linux
sudo apt-get install libssl-dev
```

### Runtime Errors

**Connection refused:**
- Check if servers are running
- Verify firewall settings
- Check port availability

**License validation fails:**
- Ensure RSA keys are properly configured
- Check system time synchronization

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [WebRTC Specification](https://www.w3.org/TR/webrtc/)
- [Axum Documentation](https://docs.rs/axum/)
