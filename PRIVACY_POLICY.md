# Privacy Policy for GenXLink

**Last Updated: December 7, 2025**

## Introduction

GenXLink ("we", "our", or "us") is committed to protecting your privacy. This Privacy Policy explains how we collect, use, and safeguard your information when you use our remote desktop application.

## Information We Collect

### Information You Provide

- **Connection ID**: A unique 9-digit identifier generated locally on your device
- **Device Name**: Optional name you assign to your device for identification

### Information Collected Automatically

- **Connection Metadata**: Timestamps of connections for session management
- **Technical Data**: Operating system version, app version for compatibility

### Information We Do NOT Collect

- Screen content or recordings
- Keyboard or mouse input data
- Personal files or documents
- Passwords or credentials
- Location data
- Browsing history

## How We Use Your Information

We use the collected information solely to:

- Establish peer-to-peer connections between devices
- Maintain active sessions
- Improve app performance and stability

## Data Storage and Security

### Local Storage

- All connection data is stored locally on your device
- Connection IDs are saved in your user profile directory
- No data is transmitted to external servers except for connection signaling

### Encryption

- All connections use end-to-end encryption (AES-256-GCM)
- Key exchange uses X25519 elliptic curve cryptography
- No unencrypted data is transmitted over the network

### Peer-to-Peer Architecture

- Screen sharing data flows directly between connected devices
- No video, audio, or input data passes through our servers
- Signaling server only facilitates initial connection establishment

## Third-Party Services

### Signaling Server

We use a signaling server to help establish peer-to-peer connections. This server:

- Only receives connection requests and ICE candidates
- Does not store any user data permanently
- Does not have access to screen content or input data

### STUN/TURN Servers

We use standard STUN/TURN servers for NAT traversal:

- These servers help establish connections through firewalls
- They do not have access to the content of your communications

## Your Rights

You have the right to:

- **Access**: View what data is stored locally
- **Delete**: Remove all local data by uninstalling the app
- **Control**: Decide who can connect to your device

## Children's Privacy

GenXLink is not intended for children under 13. We do not knowingly collect information from children under 13.

## Changes to This Policy

We may update this Privacy Policy from time to time. We will notify you of any changes by updating the "Last Updated" date.

## Contact Us

If you have questions about this Privacy Policy, please contact us:

- **Email**: genxisinnovations@gmail.com
- **GitHub**: https://github.com/lalupj07/GenXlink/issues

## Open Source

GenXLink is open source software. You can review our code at:
https://github.com/lalupj07/GenXlink

---

© 2025 GenXis Innovations. All rights reserved.
