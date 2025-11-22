use crate::{ClientError, remote_control::{RemoteControlEvent, RemoteControlHandler}};
use genxlink_protocol::{MessagePayload, MouseEvent, KeyboardEvent};
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::data_channel::RTCDataChannel;
use bytes::Bytes;

/// Control channel for remote control events
pub struct ControlChannel {
    data_channel: Arc<RTCDataChannel>,
    handler: Arc<RemoteControlHandler>,
    enabled: Arc<Mutex<bool>>,
}

impl ControlChannel {
    /// Create a new control channel
    pub fn new(
        data_channel: Arc<RTCDataChannel>,
        handler: Arc<RemoteControlHandler>,
    ) -> Self {
        Self {
            data_channel,
            handler,
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// Start listening for control events
    pub async fn start(&self) -> Result<(), ClientError> {
        let handler = Arc::clone(&self.handler);
        let enabled = Arc::clone(&self.enabled);

        self.data_channel.on_message(Box::new(move |msg| {
            let handler = Arc::clone(&handler);
            let enabled = Arc::clone(&enabled);
            
            Box::pin(async move {
                // Check if enabled
                let is_enabled = *enabled.lock().await;
                if !is_enabled {
                    tracing::warn!("Control channel disabled, ignoring message");
                    return;
                }

                // Parse message
                let data = msg.data.to_vec();
                match serde_json::from_slice::<MessagePayload>(&data) {
                    Ok(payload) => {
                        // Convert to remote control event
                        let event = match payload {
                            MessagePayload::MouseEvent(mouse_event) => {
                                RemoteControlEvent::Mouse(mouse_event)
                            }
                            MessagePayload::KeyboardEvent(keyboard_event) => {
                                RemoteControlEvent::Keyboard(keyboard_event)
                            }
                            _ => {
                                tracing::warn!("Unsupported control message");
                                return;
                            }
                        };

                        // Handle event
                        if let Err(e) = handler.handle_event(event).await {
                            tracing::error!("Failed to handle control event: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse control message: {}", e);
                    }
                }
            })
        }));

        tracing::info!("Control channel started");
        Ok(())
    }

    /// Send a control event (for testing/feedback)
    pub async fn send_event(&self, event: RemoteControlEvent) -> Result<(), ClientError> {
        let payload = match event {
            RemoteControlEvent::Mouse(mouse_event) => MessagePayload::MouseEvent(mouse_event),
            RemoteControlEvent::Keyboard(keyboard_event) => MessagePayload::KeyboardEvent(keyboard_event),
        };

        let data = serde_json::to_vec(&payload)
            .map_err(|e| ClientError::TransportError(format!("Serialization failed: {}", e)))?;

        self.data_channel
            .send(&Bytes::from(data))
            .await
            .map_err(|e| ClientError::TransportError(format!("Failed to send: {}", e)))?;

        Ok(())
    }

    /// Enable control channel
    pub async fn enable(&self) {
        let mut enabled = self.enabled.lock().await;
        *enabled = true;
        tracing::info!("Control channel enabled");
    }

    /// Disable control channel
    pub async fn disable(&self) {
        let mut enabled = self.enabled.lock().await;
        *enabled = false;
        tracing::info!("Control channel disabled");
    }

    /// Check if enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }
}

/// Control channel manager
pub struct ControlChannelManager {
    channels: Arc<Mutex<Vec<Arc<ControlChannel>>>>,
}

impl ControlChannelManager {
    /// Create a new control channel manager
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a control channel
    pub async fn add_channel(&self, channel: Arc<ControlChannel>) {
        let mut channels = self.channels.lock().await;
        channels.push(channel);
        tracing::info!("Control channel added, total: {}", channels.len());
    }

    /// Remove a control channel
    pub async fn remove_channel(&self, index: usize) -> Option<Arc<ControlChannel>> {
        let mut channels = self.channels.lock().await;
        if index < channels.len() {
            Some(channels.remove(index))
        } else {
            None
        }
    }

    /// Get channel count
    pub async fn channel_count(&self) -> usize {
        let channels = self.channels.lock().await;
        channels.len()
    }

    /// Enable all channels
    pub async fn enable_all(&self) {
        let channels = self.channels.lock().await;
        for channel in channels.iter() {
            channel.enable().await;
        }
        tracing::info!("All control channels enabled");
    }

    /// Disable all channels
    pub async fn disable_all(&self) {
        let channels = self.channels.lock().await;
        for channel in channels.iter() {
            channel.disable().await;
        }
        tracing::info!("All control channels disabled");
    }

    /// Clear all channels
    pub async fn clear(&self) {
        let mut channels = self.channels.lock().await;
        channels.clear();
        tracing::info!("All control channels cleared");
    }
}

impl Default for ControlChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Control channel builder
pub struct ControlChannelBuilder {
    data_channel: Option<Arc<RTCDataChannel>>,
    handler: Option<Arc<RemoteControlHandler>>,
}

impl ControlChannelBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            data_channel: None,
            handler: None,
        }
    }

    /// Set data channel
    pub fn with_data_channel(mut self, channel: Arc<RTCDataChannel>) -> Self {
        self.data_channel = Some(channel);
        self
    }

    /// Set remote control handler
    pub fn with_handler(mut self, handler: Arc<RemoteControlHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Build the control channel
    pub fn build(self) -> Result<ControlChannel, ClientError> {
        let data_channel = self.data_channel
            .ok_or_else(|| ClientError::TransportError("Data channel not set".to_string()))?;
        
        let handler = self.handler
            .ok_or_else(|| ClientError::TransportError("Handler not set".to_string()))?;

        Ok(ControlChannel::new(data_channel, handler))
    }
}

impl Default for ControlChannelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_control_channel_manager() {
        let manager = ControlChannelManager::new();
        assert_eq!(manager.channel_count().await, 0);
    }

    #[test]
    fn test_control_channel_builder() {
        let builder = ControlChannelBuilder::new();
        // Can't test full build without real WebRTC data channel
        assert!(builder.data_channel.is_none());
        assert!(builder.handler.is_none());
    }
}
