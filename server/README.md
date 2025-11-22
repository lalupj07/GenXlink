# GenXLink Signaling Server

WebSocket-based signaling server for GenXLink remote desktop connections.

## Features

- ✅ Device registration and discovery
- ✅ WebSocket communication
- ✅ Health check endpoint
- ✅ Device listing API
- 🚧 WebRTC signaling (coming soon)
- 🚧 Authentication (coming soon)

## Quick Start

### Build and Run

```bash
cd server
cargo build --release
cargo run --release
```

### Access

- **Web UI:** http://localhost:8080
- **Health Check:** http://localhost:8080/health
- **Device List:** http://localhost:8080/devices
- **WebSocket:** ws://localhost:8080/ws

## API Documentation

### REST Endpoints

#### GET /
Server information and documentation page.

#### GET /health
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "service": "genxlink-signaling-server"
}
```

#### GET /devices
List all registered devices.

**Response:**
```json
[
  {
    "device_id": "abc-123",
    "device_name": "My PC",
    "connected_at": "2025-11-23T00:00:00Z",
    "last_seen": "2025-11-23T00:05:00Z",
    "is_online": true
  }
]
```

### WebSocket Protocol

#### Register Device
```json
{
  "type": "register",
  "device_id": "abc-123",
  "device_name": "My PC"
}
```

**Response:**
```json
{
  "type": "registered",
  "device_id": "abc-123",
  "status": "success"
}
```

#### Ping/Pong
```json
{
  "type": "ping"
}
```

**Response:**
```json
{
  "type": "pong",
  "timestamp": "2025-11-23T00:00:00Z"
}
```

## Deployment

### Local Development
```bash
cargo run
```

### Production (Docker)
```bash
docker build -t genxlink-server .
docker run -p 8080:8080 genxlink-server
```

### Production (Systemd)
```bash
# Build release
cargo build --release

# Copy binary
sudo cp target/release/genxlink-server /usr/local/bin/

# Create systemd service
sudo nano /etc/systemd/system/genxlink-server.service
```

Service file:
```ini
[Unit]
Description=GenXLink Signaling Server
After=network.target

[Service]
Type=simple
User=genxlink
ExecStart=/usr/local/bin/genxlink-server
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start
sudo systemctl enable genxlink-server
sudo systemctl start genxlink-server
```

## Environment Variables

- `RUST_LOG` - Log level (default: info)
- `SERVER_PORT` - Server port (default: 8080)
- `SERVER_HOST` - Server host (default: 0.0.0.0)

## Monitoring

### Logs
```bash
# View logs
sudo journalctl -u genxlink-server -f

# Or with cargo
RUST_LOG=debug cargo run
```

### Health Check
```bash
curl http://localhost:8080/health
```

## Security

### TODO:
- [ ] Add TLS/SSL support
- [ ] Implement authentication
- [ ] Add rate limiting
- [ ] Add CORS configuration
- [ ] Implement session management

## License

See main project LICENSE file.
