use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};
use genxlink_protocol::{
    DeviceId, InputEvent, RemoteControlMessage, RemoteControlRequest, 
    RemoteControlResponse, RemoteControlState
};
use crate::input_injection::InputInjector;

/// Remote control manager
/// Handles both controlling remote devices and being controlled
pub struct RemoteControlManager {
    device_id: DeviceId,
    state: Arc<RwLock<RemoteControlState>>,
    injector: Arc<Mutex<Option<InputInjector>>>,
    
    // Channels for input events
    input_tx: mpsc::UnboundedSender<RemoteControlMessage>,
    input_rx: Arc<Mutex<mpsc::UnboundedReceiver<RemoteControlMessage>>>,
    
    // Permission settings
    auto_accept: bool,
    allowed_devices: Arc<RwLock<Vec<DeviceId>>>,
}

impl RemoteControlManager {
    pub fn new(device_id: DeviceId) -> Self {
        let (input_tx, input_rx) = mpsc::unbounded_channel();
        
        Self {
            device_id,
            state: Arc::new(RwLock::new(RemoteControlState::Idle)),
            injector: Arc::new(Mutex::new(None)),
            input_tx,
            input_rx: Arc::new(Mutex::new(input_rx)),
            auto_accept: false,
            allowed_devices: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Request control of a remote device
    pub async fn request_control(&self, remote_device_id: DeviceId) -> Result<RemoteControlResponse> {
        let mut state = self.state.write().await;
        *state = RemoteControlState::Requesting;
        drop(state);
        
        let request = RemoteControlRequest {
            from: self.device_id.clone(),
            to: remote_device_id.clone(),
        };
        
        tracing::info!("Requesting remote control of device: {}", remote_device_id);
        
        // In a real implementation, this would send via network
        // For now, return a mock response
        Ok(RemoteControlResponse {
            from: remote_device_id,
            to: self.device_id.clone(),
            granted: true,
            reason: None,
        })
    }
    
    /// Handle incoming control request
    pub async fn handle_control_request(&self, request: RemoteControlRequest) -> RemoteControlResponse {
        tracing::info!("Received remote control request from: {}", request.from);
        
        // Check if auto-accept is enabled
        if self.auto_accept {
            return self.grant_control(request.from.clone()).await;
        }
        
        // Check if device is in allowed list
        let allowed = self.allowed_devices.read().await;
        if allowed.contains(&request.from) {
            drop(allowed);
            return self.grant_control(request.from.clone()).await;
        }
        
        // Otherwise, deny (in real app, would show UI prompt)
        RemoteControlResponse {
            from: self.device_id.clone(),
            to: request.from,
            granted: false,
            reason: Some("Permission denied by user".to_string()),
        }
    }
    
    /// Grant control to a device
    async fn grant_control(&self, device_id: DeviceId) -> RemoteControlResponse {
        let mut state = self.state.write().await;
        *state = RemoteControlState::Active;
        drop(state);
        
        // Initialize input injector
        let mut injector_guard = self.injector.lock().await;
        if injector_guard.is_none() {
            match InputInjector::new() {
                Ok(injector) => {
                    *injector_guard = Some(injector);
                    tracing::info!("Input injector initialized");
                }
                Err(e) => {
                    tracing::error!("Failed to initialize input injector: {}", e);
                    return RemoteControlResponse {
                        from: self.device_id.clone(),
                        to: device_id,
                        granted: false,
                        reason: Some(format!("Failed to initialize: {}", e)),
                    };
                }
            }
        }
        
        tracing::info!("Remote control granted to: {}", device_id);
        
        RemoteControlResponse {
            from: self.device_id.clone(),
            to: device_id,
            granted: true,
            reason: None,
        }
    }
    
    /// Send input event to remote device
    pub async fn send_input(&self, event: InputEvent, target: DeviceId) -> Result<()> {
        let state = self.state.read().await;
        if *state != RemoteControlState::Active {
            return Err(anyhow::anyhow!("Remote control not active"));
        }
        drop(state);
        
        let message = RemoteControlMessage {
            from: self.device_id.clone(),
            to: target,
            event,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        
        self.input_tx.send(message)
            .context("Failed to send input event")?;
        
        Ok(())
    }
    
    /// Handle incoming input event (inject locally)
    pub async fn handle_input(&self, message: RemoteControlMessage) -> Result<()> {
        let state = self.state.read().await;
        if *state != RemoteControlState::Active {
            tracing::warn!("Received input while not in active state");
            return Ok(());
        }
        drop(state);
        
        let injector_guard = self.injector.lock().await;
        if let Some(injector) = injector_guard.as_ref() {
            injector.inject_event(&message.event)
                .context("Failed to inject input event")?;
            
            tracing::trace!("Injected input event: {:?}", message.event);
        } else {
            tracing::warn!("Input injector not initialized");
        }
        
        Ok(())
    }
    
    /// End remote control session
    pub async fn end_session(&self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = RemoteControlState::Ended;
        drop(state);
        
        let mut injector_guard = self.injector.lock().await;
        *injector_guard = None;
        
        tracing::info!("Remote control session ended");
        
        Ok(())
    }
    
    /// Get current state
    pub async fn get_state(&self) -> RemoteControlState {
        *self.state.read().await
    }
    
    /// Set auto-accept for incoming requests
    pub fn set_auto_accept(&mut self, enabled: bool) {
        self.auto_accept = enabled;
        tracing::info!("Auto-accept remote control: {}", enabled);
    }
    
    /// Add device to allowed list
    pub async fn add_allowed_device(&self, device_id: DeviceId) {
        let mut allowed = self.allowed_devices.write().await;
        if !allowed.contains(&device_id) {
            allowed.push(device_id.clone());
            tracing::info!("Added device to allowed list: {}", device_id);
        }
    }
    
    /// Remove device from allowed list
    pub async fn remove_allowed_device(&self, device_id: &DeviceId) {
        let mut allowed = self.allowed_devices.write().await;
        allowed.retain(|d| d != device_id);
        tracing::info!("Removed device from allowed list: {}", device_id);
    }
    
    /// Get outgoing input channel
    pub fn get_input_channel(&self) -> mpsc::UnboundedSender<RemoteControlMessage> {
        self.input_tx.clone()
    }
    
    /// Start processing incoming input events
    pub async fn start_input_processor(&self) {
        let injector = Arc::clone(&self.injector);
        let state = Arc::clone(&self.state);
        let rx = Arc::clone(&self.input_rx);
        
        tokio::spawn(async move {
            let mut rx_guard = rx.lock().await;
            
            while let Some(message) = rx_guard.recv().await {
                let state_guard = state.read().await;
                if *state_guard != RemoteControlState::Active {
                    continue;
                }
                drop(state_guard);
                
                let injector_guard = injector.lock().await;
                if let Some(inj) = injector_guard.as_ref() {
                    if let Err(e) = inj.inject_event(&message.event) {
                        tracing::error!("Failed to inject input: {}", e);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_manager_creation() {
        let device_id = DeviceId::new();
        let manager = RemoteControlManager::new(device_id);
        
        let state = manager.get_state().await;
        assert_eq!(state, RemoteControlState::Idle);
    }
    
    #[tokio::test]
    async fn test_request_control() {
        let device_id = DeviceId::new();
        let manager = RemoteControlManager::new(device_id);
        let remote_id = DeviceId::new();
        
        let response = manager.request_control(remote_id).await;
        assert!(response.is_ok());
    }
}
