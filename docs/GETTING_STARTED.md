# Getting Started with GenXLink

## Quick Start

GenXLink is a lightweight remote desktop solution that allows you to control computers remotely with minimal latency and small binary sizes.

## Installation

### Windows Client

1. Download the latest release from the releases page
2. Run the installer (`GenXLink-Setup.msi`)
3. Launch GenXLink from the Start menu

### Android Client

1. Download from Google Play Store (coming soon)
2. Or install the APK manually
3. Grant necessary permissions (screen capture, accessibility)

## First Connection

### As the Host (Computer Being Controlled)

1. Launch GenXLink
2. Note your Device ID (displayed on main screen)
3. Set a connection password
4. Share your Device ID with the person who wants to connect

### As the Client (Controlling Computer)

1. Launch GenXLink
2. Click "Connect to Remote Device"
3. Enter the host's Device ID
4. Enter the connection password
5. Click "Connect"

## License Activation

### Free Tier

- No activation required
- 10-minute session limit
- Basic features only

### Pro Tier

1. Purchase a license key
2. Click "Activate License" in the app
3. Enter your license key
4. Enter your device name
5. Click "Activate"

The license will be validated online and stored locally.

### Offline Activation

If you don't have internet access:

1. Request an offline license file from support
2. Click "Activate License" → "Offline Activation"
3. Select the license file
4. License will be validated using the embedded signature

## Features by License Tier

| Feature | Free | Pro |
|---------|------|-----|
| Session Duration | 10 min | Unlimited |
| Screen Sharing | ✓ | ✓ |
| Remote Control | ✓ | ✓ |
| File Transfer | ✗ | ✓ |
| Clipboard Sync | ✓ | ✓ |
| Unattended Access | ✗ | ✓ |
| Multi-Monitor | ✗ | ✓ |
| Priority Relay | ✗ | ✓ |

## Configuration

### Video Quality

Adjust in Settings → Video:
- Resolution: 720p, 1080p, 1440p, 4K
- FPS: 15, 30, 60
- Bitrate: Auto, 1 Mbps, 2 Mbps, 5 Mbps

Lower settings = better performance on slow connections

### Network

Settings → Network:
- STUN servers (for NAT traversal)
- TURN servers (relay fallback)
- Connection timeout
- Auto-reconnect

### Security

Settings → Security:
- Require password for connections
- Auto-lock after inactivity
- Allowed devices list
- Connection notifications

## Troubleshooting

### Cannot Connect

**Check:**
- Both devices have internet connection
- Firewall isn't blocking GenXLink
- Device ID is correct
- Password is correct

**Try:**
- Restart both applications
- Check if relay server is accessible
- Verify license is active

### Poor Performance

**Solutions:**
- Lower video quality settings
- Close other network-intensive applications
- Use wired connection instead of WiFi
- Check CPU usage on host computer

### License Issues

**Problems:**
- "License expired" → Renew your license
- "Device limit reached" → Deactivate unused devices
- "Invalid license" → Contact support

## Self-Hosting

To run your own servers:

1. Set up PostgreSQL and Redis
2. Deploy the API server
3. Deploy the signaling server
4. Deploy the relay server (optional)
5. Configure clients to use your servers

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed instructions.

## Support

- Documentation: https://docs.genxlink.com
- Community Forum: https://community.genxlink.com
- Email Support: support@genxis.com
- GitHub Issues: https://github.com/genxis/genxlink/issues

## Next Steps

- Read the [User Guide](USER_GUIDE.md)
- Watch [Video Tutorials](https://youtube.com/genxlink)
- Join the [Community](https://discord.gg/genxlink)
- Check the [FAQ](FAQ.md)

## Security Best Practices

1. **Use strong passwords** for connections
2. **Keep software updated** to latest version
3. **Enable notifications** for connection attempts
4. **Review connected devices** regularly
5. **Use unattended access** only when necessary
6. **Enable two-factor authentication** (Pro feature)

## Privacy

GenXLink respects your privacy:
- End-to-end encryption for all connections
- No data stored on servers (self-hosted option)
- No tracking or analytics by default
- Open source code for transparency

## Performance Tips

### For Best Quality
- Use wired Ethernet connection
- Close unnecessary applications
- Use 1080p @ 30fps
- Enable hardware encoding

### For Best Compatibility
- Use 720p @ 15fps
- Lower bitrate to 1 Mbps
- Enable relay fallback
- Disable multi-monitor

### For Low Bandwidth
- Use 720p @ 15fps
- Set bitrate to 500 Kbps
- Disable clipboard sync
- Minimize window movements

## Keyboard Shortcuts

- `Ctrl+Alt+D` - Disconnect
- `Ctrl+Alt+F` - Toggle fullscreen
- `Ctrl+Alt+M` - Switch monitor
- `Ctrl+Alt+Q` - Show quality stats
- `Ctrl+Alt+S` - Screenshot
- `Ctrl+Alt+T` - Transfer file

## Known Limitations

- Audio streaming not yet supported
- iOS client in development
- Maximum 4K resolution
- File transfer limited to 2GB per file

## Roadmap

See [ROADMAP.md](ROADMAP.md) for upcoming features and timeline.

---

**Welcome to GenXLink!** We hope you enjoy using our lightweight remote desktop solution.
