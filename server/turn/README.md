# GenXLink TURN Server

TURN (Traversal Using Relays around NAT) server for WebRTC NAT traversal.

## Why TURN?

WebRTC uses ICE (Interactive Connectivity Establishment) to establish P2P connections:

1. **STUN** - Discovers your public IP (works for ~80% of NATs)
2. **TURN** - Relays traffic when direct P2P fails (symmetric NAT, strict firewalls)

## Quick Start

### Option 1: Docker (Recommended)

```bash
cd server/turn
docker-compose up -d
```

### Option 2: Manual Installation

```bash
# Ubuntu/Debian
sudo apt install coturn

# Copy config
sudo cp turnserver.conf /etc/coturn/turnserver.conf

# Start service
sudo systemctl enable coturn
sudo systemctl start coturn
```

## Configuration

Edit `turnserver.conf`:

```conf
# Set your server's public IP
external-ip=YOUR_PUBLIC_IP

# Change credentials (important for security!)
user=genxlink:YOUR_SECURE_PASSWORD

# Set your domain
realm=yourdomain.com
```

## Cloud Deployment

### Deploy to DigitalOcean/AWS/GCP

1. Create a VM with public IP
2. Open firewall ports:
   - 3478 UDP/TCP (STUN/TURN)
   - 5349 TCP (TURN over TLS)
   - 49152-65535 UDP (media relay)
3. Install Docker and run:

```bash
EXTERNAL_IP=your.public.ip docker-compose up -d
```

### Using Managed TURN Services

If you don't want to self-host:

- **Twilio**: https://www.twilio.com/stun-turn
- **Xirsys**: https://xirsys.com/
- **Metered**: https://www.metered.ca/tools/openrelay/

## Testing

Test your TURN server:

```bash
# Using turnutils
turnutils_uclient -T -u genxlink -w GenXL1nk2025! turn.genxlink.io

# Or use Trickle ICE
# https://webrtc.github.io/samples/src/content/peerconnection/trickle-ice/
```

## ICE Server Configuration

Add to your WebRTC client:

```javascript
const iceServers = [
  // STUN (free, public)
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun.cloudflare.com:3478' },
  
  // TURN (your server)
  {
    urls: 'turn:turn.genxlink.io:3478',
    username: 'genxlink',
    credential: 'GenXL1nk2025!'
  },
  {
    urls: 'turns:turn.genxlink.io:5349',
    username: 'genxlink',
    credential: 'GenXL1nk2025!'
  }
];
```

## Security Notes

1. **Change default credentials** before production use
2. **Use TLS** (turns://) for encrypted relay
3. **Limit relay ports** to reduce attack surface
4. **Monitor bandwidth** - TURN relays all traffic through your server
5. **Set quotas** to prevent abuse

## Troubleshooting

### Connection fails
- Check firewall rules (UDP ports must be open)
- Verify external-ip is set correctly
- Test with `turnutils_uclient`

### High latency
- TURN adds ~50-100ms latency (relay overhead)
- Use STUN when possible, TURN as fallback

### Bandwidth issues
- TURN relays ALL traffic through server
- Ensure sufficient bandwidth (10Mbps+ per concurrent session)
