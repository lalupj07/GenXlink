use crate::ClientError;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Chat manager for text messaging
pub struct ChatManager {
    messages: Vec<ChatMessage>,
    max_messages: usize,
}

impl ChatManager {
    /// Create a new chat manager
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 1000,
        }
    }

    /// Send a message
    pub fn send_message(&mut self, content: String, sender: String) -> String {
        let message_id = format!("msg_{}", Self::current_timestamp());
        
        let message = ChatMessage {
            id: message_id.clone(),
            content,
            sender,
            timestamp: Self::current_timestamp(),
            message_type: MessageType::Text,
            status: MessageStatus::Sent,
        };
        
        self.messages.push(message);
        
        // Limit messages
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        
        message_id
    }

    /// Receive a message
    pub fn receive_message(&mut self, content: String, sender: String) -> String {
        let message_id = format!("msg_{}", Self::current_timestamp());
        
        let message = ChatMessage {
            id: message_id.clone(),
            content,
            sender,
            timestamp: Self::current_timestamp(),
            message_type: MessageType::Text,
            status: MessageStatus::Received,
        };
        
        self.messages.push(message);
        
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        
        message_id
    }

    /// Send system message
    pub fn system_message(&mut self, content: String) -> String {
        let message_id = format!("sys_{}", Self::current_timestamp());
        
        let message = ChatMessage {
            id: message_id.clone(),
            content,
            sender: "System".to_string(),
            timestamp: Self::current_timestamp(),
            message_type: MessageType::System,
            status: MessageStatus::Received,
        };
        
        self.messages.push(message);
        
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        
        message_id
    }

    /// Get all messages
    pub fn get_messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get recent messages
    pub fn get_recent_messages(&self, count: usize) -> Vec<&ChatMessage> {
        let start = self.messages.len().saturating_sub(count);
        self.messages[start..].iter().collect()
    }

    /// Get message by ID
    pub fn get_message(&self, message_id: &str) -> Option<&ChatMessage> {
        self.messages.iter().find(|m| m.id == message_id)
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Set max messages
    pub fn set_max_messages(&mut self, max: usize) {
        self.max_messages = max;
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub content: String,
    pub sender: String,
    pub timestamp: u64,
    pub message_type: MessageType,
    pub status: MessageStatus,
}

impl ChatMessage {
    /// Format timestamp as human-readable string
    pub fn format_time(&self) -> String {
        use std::time::Duration;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let elapsed = now.saturating_sub(self.timestamp);
        
        if elapsed < 60 {
            "Just now".to_string()
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else if elapsed < 86400 {
            format!("{}h ago", elapsed / 3600)
        } else {
            format!("{}d ago", elapsed / 86400)
        }
    }

    /// Check if message is from system
    pub fn is_system(&self) -> bool {
        self.message_type == MessageType::System
    }
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    System,
    File,
}

/// Message status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Sending,
    Sent,
    Received,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_manager() {
        let mut manager = ChatManager::new();
        
        let msg_id = manager.send_message("Hello".to_string(), "User1".to_string());
        assert_eq!(manager.message_count(), 1);
        
        let message = manager.get_message(&msg_id).unwrap();
        assert_eq!(message.content, "Hello");
        assert_eq!(message.sender, "User1");
    }

    #[test]
    fn test_system_message() {
        let mut manager = ChatManager::new();
        
        manager.system_message("Connection established".to_string());
        assert_eq!(manager.message_count(), 1);
        
        let messages = manager.get_messages();
        assert!(messages[0].is_system());
    }

    #[test]
    fn test_max_messages() {
        let mut manager = ChatManager::new();
        manager.set_max_messages(5);

        for i in 0..10 {
            manager.send_message(format!("Message {}", i), "User".to_string());
        }

        assert_eq!(manager.message_count(), 5);
    }

    #[test]
    fn test_recent_messages() {
        let mut manager = ChatManager::new();

        for i in 0..10 {
            manager.send_message(format!("Message {}", i), "User".to_string());
        }

        let recent = manager.get_recent_messages(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[2].content, "Message 9");
    }
}
