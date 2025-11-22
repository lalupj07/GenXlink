use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use genxlink_protocol::DeviceId;

/// Manages connected peers
pub struct PeerManager {
    peers: Arc<RwLock<HashMap<DeviceId, PeerInfo>>>,
}

pub struct PeerInfo {
    pub device_id: DeviceId,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register_peer(&self, device_id: DeviceId) {
        let mut peers = self.peers.write().await;
        peers.insert(device_id.clone(), PeerInfo {
            device_id,
            connected_at: chrono::Utc::now(),
        });
    }
    
    pub async fn unregister_peer(&self, device_id: &DeviceId) {
        let mut peers = self.peers.write().await;
        peers.remove(device_id);
    }
    
    pub async fn is_peer_online(&self, device_id: &DeviceId) -> bool {
        let peers = self.peers.read().await;
        peers.contains_key(device_id)
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}
