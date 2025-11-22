use genxlink_protocol::{Message, MessagePayload, DeviceId};
use genxlink_protocol::signaling::{SignalingMessage, PeerInfo, DeviceType};

#[test]
fn test_message_serialization() {
    let device_id = DeviceId::new();
    let payload = MessagePayload::Ping;
    
    let message = Message::new(device_id.clone(), payload);
    
    // Serialize to JSON
    let json = serde_json::to_string(&message).expect("Failed to serialize");
    assert!(!json.is_empty());
    
    // Deserialize back
    let deserialized: Message = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.device_id, device_id);
}

#[test]
fn test_signaling_message_offer() {
    let from = DeviceId::new();
    let to = DeviceId::new();
    let sdp = "v=0\r\no=- 123456 2 IN IP4 127.0.0.1\r\n".to_string();
    
    let message = SignalingMessage::Offer {
        sdp: sdp.clone(),
        from: from.clone(),
        to: to.clone(),
    };
    
    // Serialize
    let json = serde_json::to_string(&message).expect("Failed to serialize");
    
    // Deserialize
    let deserialized: SignalingMessage = serde_json::from_str(&json).expect("Failed to deserialize");
    
    match deserialized {
        SignalingMessage::Offer { sdp: s, from: f, to: t } => {
            assert_eq!(s, sdp);
            assert_eq!(f, from);
            assert_eq!(t, to);
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_peer_info_serialization() {
    let peer = PeerInfo {
        device_id: DeviceId::new(),
        device_name: "Test Device".to_string(),
        device_type: DeviceType::Desktop,
        online: true,
        last_seen: None,
    };
    
    let json = serde_json::to_string(&peer).expect("Failed to serialize");
    let deserialized: PeerInfo = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(deserialized.device_name, peer.device_name);
    assert_eq!(deserialized.device_type, peer.device_type);
    assert_eq!(deserialized.online, peer.online);
}

#[test]
fn test_device_id_generation() {
    let id1 = DeviceId::new();
    let id2 = DeviceId::new();
    
    // Device IDs should be unique
    assert_ne!(id1, id2);
    
    // Device IDs should not be empty
    assert!(!id1.0.is_empty());
    assert!(!id2.0.is_empty());
}

#[test]
fn test_device_id_from_string() {
    let id_string = "test-device-123".to_string();
    let device_id = DeviceId::from_string(id_string.clone());
    
    assert_eq!(device_id.0, id_string);
}
