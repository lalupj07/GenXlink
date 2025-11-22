# Phase 4: WebRTC & Networking - Task List

## Overview

Phase 4 focuses on implementing real-time peer-to-peer communication using WebRTC for low-latency screen streaming and input control.

**Timeline:** 2-3 weeks  
**Status:** 🚀 Starting now  
**Prerequisites:** Phase 1 ✅, Phase 2 ✅, Phase 3 ✅

---

## 🎯 Goals

- Implement WebRTC peer connections
- Add STUN/TURN server support
- Create reliable data channels
- Implement connection state management
- Add bandwidth adaptation
- Handle network failures gracefully

---

## 📋 Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    GenXLink Network Layer                │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐         ┌──────────────┐              │
│  │   Client A   │◄───────►│ Signaling    │              │
│  │  (Viewer)    │  WSS    │   Server     │              │
│  └──────┬───────┘         └──────┬───────┘              │
│         │                        │                       │
│         │  WebRTC Offer/Answer   │                       │
│         │  ICE Candidates        │                       │
│         │                        │                       │
│         ▼                        ▼                       │
│  ┌──────────────┐         ┌──────────────┐              │
│  │   Client B   │◄───────►│ STUN/TURN    │              │
│  │  (Host)      │  P2P    │   Servers    │              │
│  └──────────────┘         └──────────────┘              │
│                                                           │
│  Data Flow:                                              │
│  1. WebSocket Signaling (connection setup)               │
│  2. WebRTC Data Channels (screen + input)                │
│  3. STUN/TURN (NAT traversal)                           │
└─────────────────────────────────────────────────────────┘
```

---

## 📦 Tasks

### 1. WebRTC Dependencies (HIGH PRIORITY)

**Estimated Time:** 1 day

#### Add Dependencies

Update `Cargo.toml`:

```toml
[dependencies]
# WebRTC
webrtc = "0.9"
tokio-tungstenite = "0.21"  # WebSocket for signaling
futures = "0.3"

# Existing dependencies
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### Tasks
- [ ] Add webrtc crate
- [ ] Add WebSocket client/server
- [ ] Configure async runtime
- [ ] Test basic WebRTC setup

---

### 2. Signaling Server Enhancement (HIGH PRIORITY)

**File:** `server/signaling/src/main.rs`  
**Estimated Time:** 2-3 days

#### Current Status
- ✅ Basic WebSocket server exists
- ✅ Peer manager structure
- ⏳ Need WebRTC signaling logic

#### Requirements

**Signaling Messages:**
```rust
#[derive(Serialize, Deserialize)]
pub enum SignalingMessage {
    // Connection setup
    Offer { sdp: String, from: DeviceId },
    Answer { sdp: String, from: DeviceId },
    IceCandidate { candidate: String, from: DeviceId },
    
    // Peer management
    PeerList { peers: Vec<PeerInfo> },
    PeerJoined { peer: PeerInfo },
    PeerLeft { device_id: DeviceId },
    
    // Connection control
    ConnectionRequest { target: DeviceId },
    ConnectionAccepted { session_id: SessionId },
    ConnectionRejected { reason: String },
}
```

#### Implementation Tasks

**2.1 WebRTC Signaling Handler**
```rust
async fn handle_signaling(
    ws: WebSocket,
    peer_manager: Arc<RwLock<PeerManager>>,
) -> Result<()> {
    // Handle WebRTC offer/answer exchange
    // Forward ICE candidates
    // Manage peer connections
}
```

**2.2 Peer Discovery**
```rust
impl PeerManager {
    pub async fn list_available_peers(&self) -> Vec<PeerInfo>;
    pub async fn request_connection(&mut self, target: DeviceId) -> Result<SessionId>;
    pub async fn accept_connection(&mut self, session_id: SessionId) -> Result<()>;
}
```

#### Testing
- [ ] Test offer/answer exchange
- [ ] Test ICE candidate forwarding
- [ ] Test peer discovery
- [ ] Test connection requests

---

### 3. WebRTC Connection Manager (HIGH PRIORITY)

**File:** `client/core/src/webrtc.rs` (new)  
**Estimated Time:** 3-4 days

#### Create WebRTC Manager

```rust
pub struct WebRTCManager {
    peer_connection: Arc<RTCPeerConnection>,
    data_channels: HashMap<String, Arc<RTCDataChannel>>,
    signaling_client: SignalingClient,
    state: ConnectionState,
}

impl WebRTCManager {
    pub async fn new(config: WebRTCConfig) -> Result<Self>;
    
    // Connection lifecycle
    pub async fn create_offer(&mut self) -> Result<String>;
    pub async fn set_remote_answer(&mut self, sdp: String) -> Result<()>;
    pub async fn add_ice_candidate(&mut self, candidate: String) -> Result<()>;
    
    // Data channels
    pub async fn create_data_channel(&mut self, label: &str) -> Result<Arc<RTCDataChannel>>;
    pub async fn send_data(&self, channel: &str, data: &[u8]) -> Result<()>;
    
    // State management
    pub fn get_state(&self) -> ConnectionState;
    pub async fn close(&mut self) -> Result<()>;
}
```

#### Data Channels

**Channel Types:**
1. **`screen`** - Video frame data (ordered, reliable)
2. **`input`** - Keyboard/mouse events (ordered, reliable)
3. **`control`** - Connection control messages (ordered, reliable)
4. **`clipboard`** - Clipboard sync (unordered, reliable)

#### Implementation Tasks

**3.1 Peer Connection Setup**
```rust
async fn create_peer_connection(config: &WebRTCConfig) -> Result<RTCPeerConnection> {
    let api = APIBuilder::new().build();
    let config = RTCConfiguration {
        ice_servers: vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    api.new_peer_connection(config).await
}
```

**3.2 Data Channel Creation**
```rust
async fn setup_data_channels(pc: &RTCPeerConnection) -> Result<HashMap<String, Arc<RTCDataChannel>>> {
    let mut channels = HashMap::new();
    
    // Screen data channel
    let screen = pc.create_data_channel("screen", None).await?;
    channels.insert("screen".to_string(), screen);
    
    // Input data channel
    let input = pc.create_data_channel("input", None).await?;
    channels.insert("input".to_string(), input);
    
    Ok(channels)
}
```

**3.3 Event Handlers**
```rust
// Handle ICE candidates
pc.on_ice_candidate(Box::new(move |candidate| {
    if let Some(c) = candidate {
        // Send to signaling server
        signaling_tx.send(SignalingMessage::IceCandidate {
            candidate: c.to_json()?,
            from: device_id,
        })?;
    }
    Box::pin(async {})
}));

// Handle connection state changes
pc.on_peer_connection_state_change(Box::new(move |state| {
    println!("Connection state: {:?}", state);
    Box::pin(async {})
}));
```

#### Testing
- [ ] Test peer connection creation
- [ ] Test data channel creation
- [ ] Test ICE candidate gathering
- [ ] Test connection establishment
- [ ] Test data transmission

---

### 4. STUN/TURN Configuration (MEDIUM PRIORITY)

**File:** `shared/config/src/webrtc.rs` (new)  
**Estimated Time:** 1 day

#### Configuration Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCConfig {
    pub ice_servers: Vec<IceServer>,
    pub ice_transport_policy: IceTransportPolicy,
    pub bundle_policy: BundlePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IceTransportPolicy {
    All,
    Relay,  // Force TURN
}
```

#### Default Configuration

```rust
impl Default for WebRTCConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                IceServer {
                    urls: vec![
                        "stun:stun.l.google.com:19302".to_string(),
                        "stun:stun1.l.google.com:19302".to_string(),
                    ],
                    username: None,
                    credential: None,
                },
            ],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::MaxBundle,
        }
    }
}
```

#### Tasks
- [ ] Create configuration structure
- [ ] Add STUN server list
- [ ] Add TURN server support
- [ ] Add configuration validation
- [ ] Document server setup

---

### 5. Connection State Machine (MEDIUM PRIORITY)

**File:** `client/core/src/connection_state.rs` (new)  
**Estimated Time:** 2 days

#### State Machine

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    SignalingConnected,
    GatheringCandidates,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
    Closed,
}

pub struct ConnectionStateMachine {
    current_state: ConnectionState,
    state_history: Vec<(ConnectionState, Instant)>,
    callbacks: HashMap<ConnectionState, Vec<StateCallback>>,
}

impl ConnectionStateMachine {
    pub fn transition_to(&mut self, new_state: ConnectionState) -> Result<()>;
    pub fn can_transition_to(&self, new_state: &ConnectionState) -> bool;
    pub fn on_state_change(&mut self, state: ConnectionState, callback: StateCallback);
}
```

#### Valid Transitions

```
Disconnected → Connecting
Connecting → SignalingConnected
SignalingConnected → GatheringCandidates
GatheringCandidates → Connecting
Connecting → Connected
Connected → Reconnecting
Reconnecting → Connected
Any → Failed
Any → Closed
```

#### Tasks
- [ ] Implement state machine
- [ ] Add state validation
- [ ] Add state callbacks
- [ ] Add state persistence
- [ ] Test all transitions

---

### 6. Bandwidth Adaptation (LOW PRIORITY)

**File:** `client/core/src/bandwidth.rs` (new)  
**Estimated Time:** 2 days

#### Adaptive Bitrate Control

```rust
pub struct BandwidthController {
    current_bitrate: u32,
    target_bitrate: u32,
    min_bitrate: u32,
    max_bitrate: u32,
    rtt_history: VecDeque<Duration>,
    packet_loss_history: VecDeque<f32>,
}

impl BandwidthController {
    pub fn update_stats(&mut self, rtt: Duration, packet_loss: f32);
    pub fn get_target_bitrate(&self) -> u32;
    pub fn adjust_quality(&mut self) -> QualitySettings;
}

#[derive(Debug, Clone)]
pub struct QualitySettings {
    pub resolution: (u32, u32),
    pub fps: u32,
    pub bitrate: u32,
    pub compression: CompressionLevel,
}
```

#### Adaptation Algorithm

```rust
fn calculate_target_bitrate(&self) -> u32 {
    let avg_rtt = self.average_rtt();
    let avg_loss = self.average_packet_loss();
    
    // Reduce bitrate if high latency or packet loss
    if avg_rtt > Duration::from_millis(200) || avg_loss > 0.05 {
        (self.current_bitrate as f32 * 0.8) as u32
    } else if avg_rtt < Duration::from_millis(50) && avg_loss < 0.01 {
        (self.current_bitrate as f32 * 1.2) as u32
    } else {
        self.current_bitrate
    }
    .clamp(self.min_bitrate, self.max_bitrate)
}
```

#### Tasks
- [ ] Implement bandwidth controller
- [ ] Add RTT tracking
- [ ] Add packet loss tracking
- [ ] Implement adaptation algorithm
- [ ] Test with varying network conditions

---

### 7. Error Handling & Reconnection (MEDIUM PRIORITY)

**Estimated Time:** 2 days

#### Reconnection Strategy

```rust
pub struct ReconnectionPolicy {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f32,
}

impl ReconnectionPolicy {
    pub fn next_delay(&self, attempt: u32) -> Duration {
        let delay = self.initial_delay.as_millis() as f32 
            * self.backoff_multiplier.powi(attempt as i32);
        Duration::from_millis(delay as u64).min(self.max_delay)
    }
}
```

#### Error Types

```rust
#[derive(Debug, Error)]
pub enum WebRTCError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Signaling error: {0}")]
    SignalingError(String),
    
    #[error("ICE gathering failed")]
    IceGatheringFailed,
    
    #[error("Data channel error: {0}")]
    DataChannelError(String),
    
    #[error("Timeout")]
    Timeout,
}
```

#### Tasks
- [ ] Implement reconnection logic
- [ ] Add exponential backoff
- [ ] Handle ICE failures
- [ ] Handle signaling failures
- [ ] Test reconnection scenarios

---

## 🧪 Testing Strategy

### Unit Tests
- [ ] WebRTC manager creation
- [ ] Data channel operations
- [ ] State machine transitions
- [ ] Bandwidth calculations

### Integration Tests
- [ ] Full connection establishment
- [ ] Data transmission
- [ ] Reconnection handling
- [ ] Multi-peer scenarios

### Manual Tests
- [ ] Local network connection
- [ ] Internet connection (with STUN)
- [ ] NAT traversal (with TURN)
- [ ] Network interruption handling
- [ ] Bandwidth adaptation

---

## 📊 Success Criteria

- [ ] Peer-to-peer connection established
- [ ] Data channels working reliably
- [ ] STUN/TURN integration complete
- [ ] Connection state properly managed
- [ ] Bandwidth adaptation functional
- [ ] Reconnection working
- [ ] Clean error handling
- [ ] Documentation complete

---

## 🚀 Deliverables

1. **WebRTC Manager** - Core connection handling
2. **Signaling Enhancement** - Offer/answer exchange
3. **Data Channels** - Screen, input, control, clipboard
4. **STUN/TURN Config** - NAT traversal support
5. **State Machine** - Connection lifecycle management
6. **Bandwidth Controller** - Adaptive quality
7. **Error Handling** - Robust reconnection
8. **Tests** - Comprehensive test suite
9. **Documentation** - Setup and usage guides

---

## 📝 Notes

### WebRTC Challenges
- NAT traversal complexity
- Firewall restrictions
- Network quality variations
- Browser compatibility (for web viewer)

### Performance Targets
- **Connection Time:** < 5 seconds
- **Latency:** < 100ms
- **Packet Loss Tolerance:** < 5%
- **Reconnection Time:** < 3 seconds

### Security Considerations
- DTLS encryption (built into WebRTC)
- Signaling server authentication
- TURN server credentials
- Session validation

---

**Next:** Phase 5 - UI & User Experience  
**Estimated Start:** After Phase 4 completion
