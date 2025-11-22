use serde::{Deserialize, Serialize};
use crate::{DeviceId, SessionId};

/// Signaling messages for WebRTC connection setup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    /// WebRTC offer
    Offer {
        sdp: String,
        from: DeviceId,
        to: DeviceId,
    },
    
    /// WebRTC answer
    Answer {
        sdp: String,
        from: DeviceId,
        to: DeviceId,
    },
    
    /// ICE candidate
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
        from: DeviceId,
        to: DeviceId,
    },
    
    /// Request list of available peers
    ListPeers,
    
    /// Response with available peers
    PeerList {
        peers: Vec<PeerInfo>,
    },
    
    /// Notification that a peer joined
    PeerJoined {
        peer: PeerInfo,
    },
    
    /// Notification that a peer left
    PeerLeft {
        device_id: DeviceId,
    },
    
    /// Request connection to a peer
    ConnectionRequest {
        target: DeviceId,
        from: DeviceId,
    },
    
    /// Connection request accepted
    ConnectionAccepted {
        session_id: SessionId,
        from: DeviceId,
    },
    
    /// Connection request rejected
    ConnectionRejected {
        reason: String,
        from: DeviceId,
    },
    
    /// Heartbeat/ping
    Ping,
    
    /// Heartbeat response
    Pong,
    
    /// Error message
    Error {
        message: String,
    },
}

/// Information about a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub device_id: DeviceId,
    pub device_name: String,
    pub device_type: DeviceType,
    pub online: bool,
    pub last_seen: Option<u64>,
}

/// Type of device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Tablet,
    Unknown,
}

impl SignalingMessage {
    /// Check if this message is for a specific device
    pub fn is_for_device(&self, device_id: &DeviceId) -> bool {
        match self {
            SignalingMessage::Offer { to, .. } => to == device_id,
            SignalingMessage::Answer { to, .. } => to == device_id,
            SignalingMessage::IceCandidate { to, .. } => to == device_id,
            SignalingMessage::ConnectionRequest { target, .. } => target == device_id,
            SignalingMessage::ConnectionAccepted { from, .. } => from == device_id,
            SignalingMessage::ConnectionRejected { from, .. } => from == device_id,
            _ => false,
        }
    }
    
    /// Get the sender device ID if applicable
    pub fn from_device(&self) -> Option<&DeviceId> {
        match self {
            SignalingMessage::Offer { from, .. } => Some(from),
            SignalingMessage::Answer { from, .. } => Some(from),
            SignalingMessage::IceCandidate { from, .. } => Some(from),
            SignalingMessage::ConnectionRequest { from, .. } => Some(from),
            SignalingMessage::ConnectionAccepted { from, .. } => Some(from),
            SignalingMessage::ConnectionRejected { from, .. } => Some(from),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signaling_message_serialization() {
        let msg = SignalingMessage::Offer {
            sdp: "v=0...".to_string(),
            from: DeviceId::new(),
            to: DeviceId::new(),
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: SignalingMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            SignalingMessage::Offer { .. } => {},
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_is_for_device() {
        let target = DeviceId::new();
        let from = DeviceId::new();
        
        let msg = SignalingMessage::Offer {
            sdp: "v=0...".to_string(),
            from: from.clone(),
            to: target.clone(),
        };
        
        assert!(msg.is_for_device(&target));
        assert!(!msg.is_for_device(&from));
    }
}
