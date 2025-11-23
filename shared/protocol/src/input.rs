use serde::{Deserialize, Serialize};

/// Input event types for remote control
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputEvent {
    /// Mouse movement (absolute coordinates)
    MouseMove {
        x: i32,
        y: i32,
    },
    
    /// Mouse movement (relative delta)
    MouseMoveDelta {
        dx: i32,
        dy: i32,
    },
    
    /// Mouse button press
    MouseDown {
        button: MouseButton,
        x: i32,
        y: i32,
    },
    
    /// Mouse button release
    MouseUp {
        button: MouseButton,
        x: i32,
        y: i32,
    },
    
    /// Mouse wheel scroll
    MouseWheel {
        delta: i32,
        x: i32,
        y: i32,
    },
    
    /// Keyboard key press
    KeyDown {
        key: KeyCode,
        modifiers: KeyModifiers,
    },
    
    /// Keyboard key release
    KeyUp {
        key: KeyCode,
        modifiers: KeyModifiers,
    },
    
    /// Text input (for complex characters)
    TextInput {
        text: String,
    },
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Keyboard key codes (virtual key codes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyCode(pub u32);

impl KeyCode {
    // Common key codes (Windows VK codes)
    pub const BACKSPACE: KeyCode = KeyCode(0x08);
    pub const TAB: KeyCode = KeyCode(0x09);
    pub const ENTER: KeyCode = KeyCode(0x0D);
    pub const SHIFT: KeyCode = KeyCode(0x10);
    pub const CONTROL: KeyCode = KeyCode(0x11);
    pub const ALT: KeyCode = KeyCode(0x12);
    pub const ESCAPE: KeyCode = KeyCode(0x1B);
    pub const SPACE: KeyCode = KeyCode(0x20);
    pub const LEFT: KeyCode = KeyCode(0x25);
    pub const UP: KeyCode = KeyCode(0x26);
    pub const RIGHT: KeyCode = KeyCode(0x27);
    pub const DOWN: KeyCode = KeyCode(0x28);
    pub const DELETE: KeyCode = KeyCode(0x2E);
    
    // Letters (A-Z)
    pub const A: KeyCode = KeyCode(0x41);
    pub const Z: KeyCode = KeyCode(0x5A);
    
    // Numbers (0-9)
    pub const NUM_0: KeyCode = KeyCode(0x30);
    pub const NUM_9: KeyCode = KeyCode(0x39);
    
    // Function keys
    pub const F1: KeyCode = KeyCode(0x70);
    pub const F12: KeyCode = KeyCode(0x7B);
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Windows key / Command key
}

impl KeyModifiers {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }
    
    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }
    
    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }
    
    pub fn with_meta(mut self) -> Self {
        self.meta = true;
        self
    }
    
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }
}

/// Remote control message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteControlMessage {
    /// Device ID of the sender
    pub from: crate::DeviceId,
    
    /// Device ID of the receiver
    pub to: crate::DeviceId,
    
    /// Input event
    pub event: InputEvent,
    
    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,
}

/// Remote control permission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteControlRequest {
    pub from: crate::DeviceId,
    pub to: crate::DeviceId,
}

/// Remote control permission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteControlResponse {
    pub from: crate::DeviceId,
    pub to: crate::DeviceId,
    pub granted: bool,
    pub reason: Option<String>,
}

/// Remote control session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteControlState {
    Idle,
    Requesting,
    Active,
    Denied,
    Ended,
}
