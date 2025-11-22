use genxlink_client_core::webrtc::{WebRTCManager, WebRTCConfig, ConnectionState};

#[tokio::test]
async fn test_webrtc_manager_creation() {
    let device_id = "test-device-123".to_string();
    let config = WebRTCConfig::default();
    
    let manager = WebRTCManager::new(device_id.clone(), config);
    
    let state = manager.get_state().await;
    assert_eq!(state, ConnectionState::Disconnected);
}

#[tokio::test]
async fn test_webrtc_initialization() {
    let device_id = "test-device-456".to_string();
    let config = WebRTCConfig::default();
    
    let mut manager = WebRTCManager::new(device_id, config);
    
    let result = manager.initialize().await;
    assert!(result.is_ok(), "WebRTC initialization should succeed");
    
    let state = manager.get_state().await;
    assert_eq!(state, ConnectionState::SignalingConnected);
}

#[tokio::test]
async fn test_offer_creation() {
    let device_id = "test-device-789".to_string();
    let config = WebRTCConfig::default();
    
    let mut manager = WebRTCManager::new(device_id, config);
    manager.initialize().await.expect("Failed to initialize");
    
    let offer = manager.create_offer().await;
    assert!(offer.is_ok(), "Offer creation should succeed");
    
    let sdp = offer.unwrap();
    assert!(!sdp.is_empty(), "SDP should not be empty");
    assert!(sdp.contains("v=0"), "SDP should contain version");
}

#[tokio::test]
async fn test_data_channel_creation() {
    let device_id = "test-device-abc".to_string();
    let config = WebRTCConfig::default();
    
    let mut manager = WebRTCManager::new(device_id, config);
    manager.initialize().await.expect("Failed to initialize");
    
    let result = manager.create_data_channel("test-channel").await;
    assert!(result.is_ok(), "Data channel creation should succeed");
}

#[tokio::test]
async fn test_connection_close() {
    let device_id = "test-device-def".to_string();
    let config = WebRTCConfig::default();
    
    let mut manager = WebRTCManager::new(device_id, config);
    manager.initialize().await.expect("Failed to initialize");
    
    let result = manager.close().await;
    assert!(result.is_ok(), "Connection close should succeed");
    
    let state = manager.get_state().await;
    assert_eq!(state, ConnectionState::Closed);
}
