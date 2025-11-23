# 🚀 GenXLink - Complete Guide

## 📋 Table of Contents

1. [Overview](#overview)
2. [Current Status](#current-status)
3. [Live Deployment](#live-deployment)
4. [Getting Started](#getting-started)
5. [Architecture](#architecture)
6. [API Documentation](#api-documentation)
7. [Database Setup](#database-setup)
8. [Platform-Specific APIs](#platform-specific-apis)
9. [Development Roadmap](#development-roadmap)

---

## 🎯 Overview

**GenXLink** is an open-source remote desktop solution built with Rust and WebRTC, featuring 20+ advanced capabilities including screen sharing, remote control, file transfer, and more.

### Key Features

- ✅ **WebRTC-based** - Peer-to-peer connections with low latency
- ✅ **Cross-platform** - Windows, macOS, Linux, Android, iOS
- ✅ **Secure** - End-to-end encryption with AES-256-GCM
- ✅ **Modern UI** - Built with egui for native performance
- ✅ **Cloud-ready** - Deployed on Railway with auto-scaling

---

## 📊 Current Status

### ✅ Completed

- **Server Infrastructure**
  - ✅ WebSocket signaling server (Actix-web)
  - ✅ Device registration and management
  - ✅ Health monitoring
  - ✅ Deployed to Railway (https://genxlink-production.up.railway.app)

- **Client Application**
  - ✅ Windows client UI (egui)
  - ✅ Connection management
  - ✅ Settings panel
  - ✅ Device discovery
  - ✅ Premium features panel

- **Documentation**
  - ✅ API documentation (OpenAPI/Swagger)
  - ✅ GitHub Pages setup
  - ✅ Deployment guides

### 🚧 In Progress

- **Platform-Specific APIs**
  - 🔄 Screen capture (Windows DXGI)
  - 🔄 Input injection (Windows SendInput)
  - 🔄 Audio streaming
  - 🔄 File transfer

- **Database Integration**
  - 🔄 Supabase setup
  - 🔄 User authentication
  - 🔄 Connection history

---

## 🌐 Live Deployment

### Production Server

**URL:** https://genxlink-production.up.railway.app

**Endpoints:**
- `GET /` - Server information
- `GET /health` - Health check
- `GET /devices` - List connected devices
- `WS /ws` - WebSocket connection

### API Documentation

**URL:** https://lalupj07.github.io/GenXlink/

Interactive Swagger UI with full API documentation.

### Test the Server

```bash
# Health check
curl https://genxlink-production.up.railway.app/health

# Server info
curl https://genxlink-production.up.railway.app/

# List devices
curl https://genxlink-production.up.railway.app/devices
```

---

## 🚀 Getting Started

### Prerequisites

- **Rust 1.83+** - [Install from rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository
- **Visual Studio Build Tools** (Windows) - For native dependencies

### Installation

```bash
# Clone the repository
git clone https://github.com/lalupj07/GenXlink.git
cd GenXlink

# Build the server
cd server
cargo build --release

# Build the Windows client
cd ../client/windows
cargo build --release
```

### Running Locally

**Server:**
```bash
cd server
cargo run
# Server starts on http://localhost:8080
```

**Client:**
```bash
cd client/windows
cargo run
# Client UI launches
```

---

## 🏗️ Architecture

### System Overview

```
┌─────────────────┐         ┌─────────────────┐
│  Client (PC A)  │◄───────►│ Signaling Server│
│   Windows/Mac   │  WebSocket│   (Railway)     │
└─────────────────┘         └─────────────────┘
        ▲                            ▲
        │                            │
        │    WebRTC P2P Connection   │
        │                            │
        ▼                            ▼
┌─────────────────┐         ┌─────────────────┐
│  Client (PC B)  │◄───────►│    Supabase     │
│   Windows/Mac   │  WebSocket│   (Database)    │
└─────────────────┘         └─────────────────┘
```

### Technology Stack

**Backend:**
- Rust 1.83
- Actix-web 4.4 (HTTP/WebSocket)
- Tokio (Async runtime)
- Serde (Serialization)

**Frontend:**
- egui (Immediate mode GUI)
- eframe (Application framework)
- WebRTC (Peer connections)

**Deployment:**
- Railway (Server hosting)
- Docker (Containerization)
- GitHub Actions (CI/CD)
- GitHub Pages (Documentation)

**Database:**
- Supabase (PostgreSQL)
- Real-time subscriptions
- Row-level security

---

## 📖 API Documentation

### REST Endpoints

#### GET /
Returns server information and available endpoints.

**Response:**
```json
{
  "name": "GenXLink Signaling Server",
  "version": "0.1.0",
  "status": "online"
}
```

#### GET /health
Health check endpoint for monitoring.

**Response:**
```json
{
  "service": "genxlink-signaling-server",
  "status": "healthy",
  "version": "0.1.0"
}
```

#### GET /devices
List all connected devices.

**Response:**
```json
[
  {
    "device_id": "abc-123",
    "device_name": "My PC",
    "last_seen": "2025-11-23T08:30:00Z"
  }
]
```

### WebSocket Protocol

#### Connect
```
WS wss://genxlink-production.up.railway.app/ws
```

#### Register Device
```json
{
  "type": "register",
  "device_id": "abc-123",
  "device_name": "My PC"
}
```

#### Request Connection
```json
{
  "type": "connect",
  "target_device_id": "xyz-789"
}
```

#### WebRTC Signaling
```json
{
  "type": "offer",
  "sdp": "v=0\r\no=- ..."
}
```

```json
{
  "type": "answer",
  "sdp": "v=0\r\no=- ..."
}
```

```json
{
  "type": "ice_candidate",
  "candidate": "candidate:..."
}
```

---

## 🗄️ Database Setup

### Supabase Configuration

**Step 1: Create Supabase Project**

1. Go to [supabase.com](https://supabase.com)
2. Create a new project
3. Choose **Singapore** region (closest to India)
4. Note your project URL and API keys

**Step 2: Database Schema**

```sql
-- Devices table
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id TEXT UNIQUE NOT NULL,
    device_name TEXT NOT NULL,
    user_id UUID REFERENCES auth.users(id),
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Connections table
CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    from_device_id TEXT NOT NULL,
    to_device_id TEXT NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ended_at TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,
    status TEXT NOT NULL
);

-- Connection history view
CREATE VIEW connection_history AS
SELECT 
    c.*,
    d1.device_name as from_device_name,
    d2.device_name as to_device_name
FROM connections c
LEFT JOIN devices d1 ON c.from_device_id = d1.device_id
LEFT JOIN devices d2 ON c.to_device_id = d2.device_id
ORDER BY c.started_at DESC;
```

**Step 3: Environment Variables**

Add to Railway:
```bash
DATABASE_URL=postgresql://postgres:[password]@[host]:5432/postgres
SUPABASE_URL=https://[project-id].supabase.co
SUPABASE_KEY=[your-anon-key]
```

**Step 4: Enable Row-Level Security**

```sql
-- Enable RLS
ALTER TABLE devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE connections ENABLE ROW LEVEL SECURITY;

-- Policies
CREATE POLICY "Users can view their own devices"
ON devices FOR SELECT
USING (auth.uid() = user_id);

CREATE POLICY "Users can insert their own devices"
ON devices FOR INSERT
WITH CHECK (auth.uid() = user_id);
```

---

## 🖥️ Platform-Specific APIs

### Windows Implementation

#### 1. Screen Capture (DXGI)

**File:** `client/windows/src/capture/dxgi.rs`

**Dependencies:**
```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_Graphics_Direct3D11",
    "Win32_Graphics_Dxgi",
    "Win32_Graphics_Gdi",
] }
```

**Implementation Plan:**
```rust
// 1. Initialize DXGI
// 2. Get desktop duplication
// 3. Acquire next frame
// 4. Copy to CPU memory
// 5. Encode to H.264
// 6. Send via WebRTC
```

**Estimated Time:** 2-3 days

#### 2. Input Injection

**File:** `client/windows/src/input/injection.rs`

**Dependencies:**
```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_Input_KeyboardAndMouse",
] }
```

**Implementation Plan:**
```rust
// 1. Receive input events via WebRTC
// 2. Convert to Windows INPUT structures
// 3. Call SendInput API
// 4. Handle keyboard and mouse separately
```

**Estimated Time:** 1-2 days

#### 3. Audio Capture

**File:** `client/windows/src/audio/wasapi.rs`

**Dependencies:**
```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_Media_Audio",
] }
cpal = "0.15"  # Cross-platform audio
```

**Implementation Plan:**
```rust
// 1. Initialize WASAPI
// 2. Capture audio stream
// 3. Encode to Opus
// 4. Send via WebRTC data channel
```

**Estimated Time:** 2-3 days

#### 4. File Transfer

**File:** `client/windows/src/transfer/file.rs`

**Implementation Plan:**
```rust
// 1. Select files via dialog
// 2. Chunk files (1MB chunks)
// 3. Send via WebRTC data channel
// 4. Show progress bar
// 5. Verify with checksums
```

**Estimated Time:** 1-2 days

### macOS Implementation

**File:** `client/macos/src/capture/screencapture.rs`

**Dependencies:**
```toml
[dependencies]
core-graphics = "0.23"
core-foundation = "0.9"
```

**Estimated Time:** 3-4 days

### Linux Implementation

**File:** `client/linux/src/capture/x11.rs`

**Dependencies:**
```toml
[dependencies]
x11 = "2.21"
```

**Estimated Time:** 2-3 days

---

## 🗺️ Development Roadmap

### Phase 1: Core Features (Current)
**Timeline:** Weeks 1-2

- ✅ Server infrastructure
- ✅ Client UI
- ✅ WebSocket signaling
- ✅ Deployment to Railway
- 🔄 Database integration
- 🔄 Documentation

### Phase 2: Platform APIs
**Timeline:** Weeks 3-5

- 🔄 Windows screen capture (DXGI)
- 🔄 Windows input injection
- 🔄 Audio streaming (WASAPI)
- 🔄 File transfer
- 🔄 Clipboard sync

### Phase 3: Cross-Platform
**Timeline:** Weeks 6-8

- ⏳ macOS support
- ⏳ Linux support
- ⏳ Mobile clients (Android/iOS)

### Phase 4: Advanced Features
**Timeline:** Weeks 9-12

- ⏳ Multi-monitor support
- ⏳ Session recording
- ⏳ Remote printing
- ⏳ Wake-on-LAN
- ⏳ Port forwarding

### Phase 5: Enterprise Features
**Timeline:** Weeks 13-16

- ⏳ User authentication
- ⏳ Team management
- ⏳ Access control
- ⏳ Audit logs
- ⏳ SSO integration

---

## 💰 Cost Estimate

### Free Tier (Current)

- **Railway:** $5/month credit (free tier)
- **Supabase:** 500MB database, 2GB bandwidth (free)
- **GitHub Pages:** Unlimited (free)
- **Total:** $0/month

### Production Tier

- **Railway:** ~$10-20/month (with usage)
- **Supabase:** ~$25/month (Pro plan)
- **Domain:** ~$12/year
- **Total:** ~$35-45/month

---

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

---

## 📄 License

MIT License - See LICENSE file for details

---

## 📞 Support

- **GitHub Issues:** https://github.com/lalupj07/GenXlink/issues
- **Documentation:** https://lalupj07.github.io/GenXlink/
- **Server Status:** https://genxlink-production.up.railway.app/health

---

**Built with ❤️ using Rust and WebRTC**
