# Phase 3: Input Injection & Remote Control - Task List

## Overview

Phase 3 focuses on implementing remote control functionality - allowing the viewer to control the remote desktop through keyboard and mouse input injection.

**Timeline:** 1-2 weeks  
**Status:** Starting now  
**Prerequisites:** Phase 1 ✅, Phase 2 Screen Capture ✅

---

## 🎯 Goals

- Implement reliable keyboard input injection
- Implement precise mouse input injection
- Add clipboard synchronization
- Handle special keys and shortcuts
- Test latency and responsiveness

---

## 📋 Tasks

### 1. Keyboard Input Injection (HIGH PRIORITY)

**File:** `client/core/src/input.rs`  
**Estimated Time:** 2-3 days

#### Current Status
- ✅ Basic structure exists
- ✅ Windows SendInput implementation started
- ⏳ Need to complete and test

#### Requirements
- Inject keyboard events (key down/up)
- Handle special keys (Ctrl, Alt, Shift, Win)
- Support key combinations (Ctrl+C, Alt+Tab, etc.)
- Handle international keyboards
- Low latency (<50ms)

#### Implementation Tasks

**1.1 Complete Key Mapping**
```rust
// Map protocol keys to Windows virtual key codes
fn map_key_to_vk(key: Key) -> u16 {
    match key {
        Key::A => VK_A,
        Key::Control => VK_CONTROL,
        // ... complete mapping
    }
}
```

**1.2 Implement Key Injection**
```rust
pub fn inject_key_down(&mut self, key: Key) -> Result<()> {
    let vk = map_key_to_vk(key);
    let input = create_keyboard_input(vk, false);
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    Ok(())
}

pub fn inject_key_up(&mut self, key: Key) -> Result<()> {
    let vk = map_key_to_vk(key);
    let input = create_keyboard_input(vk, true);
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    Ok(())
}
```

**1.3 Handle Key Combinations**
```rust
pub fn inject_key_combo(&mut self, keys: &[Key]) -> Result<()> {
    // Press all keys
    for key in keys {
        self.inject_key_down(*key)?;
    }
    // Release in reverse order
    for key in keys.iter().rev() {
        self.inject_key_up(*key)?;
    }
    Ok(())
}
```

#### Testing
- [ ] Test all alphanumeric keys
- [ ] Test special keys (F1-F12, Esc, Tab, etc.)
- [ ] Test modifiers (Ctrl, Alt, Shift, Win)
- [ ] Test key combinations
- [ ] Test international characters
- [ ] Measure latency

---

### 2. Mouse Input Injection (HIGH PRIORITY)

**File:** `client/core/src/input.rs`  
**Estimated Time:** 2-3 days

#### Current Status
- ✅ Basic structure exists
- ✅ Windows SetCursorPos implementation started
- ⏳ Need to complete mouse buttons and wheel

#### Requirements
- Move mouse cursor
- Click (left, right, middle)
- Double-click
- Mouse wheel scroll
- Drag and drop support
- Absolute and relative positioning

#### Implementation Tasks

**2.1 Mouse Movement**
```rust
pub fn move_mouse(&mut self, x: i32, y: i32) -> Result<()> {
    unsafe {
        SetCursorPos(x, y)
            .map_err(|e| ClientError::InputError(format!("Move failed: {}", e)))?;
    }
    Ok(())
}

pub fn move_mouse_relative(&mut self, dx: i32, dy: i32) -> Result<()> {
    let (current_x, current_y) = self.get_mouse_position()?;
    self.move_mouse(current_x + dx, current_y + dy)
}
```

**2.2 Mouse Buttons**
```rust
pub fn mouse_down(&mut self, button: MouseButton) -> Result<()> {
    let input = create_mouse_input(button, true);
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    Ok(())
}

pub fn mouse_up(&mut self, button: MouseButton) -> Result<()> {
    let input = create_mouse_input(button, false);
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    Ok(())
}

pub fn mouse_click(&mut self, button: MouseButton) -> Result<()> {
    self.mouse_down(button)?;
    std::thread::sleep(Duration::from_millis(10));
    self.mouse_up(button)?;
    Ok(())
}
```

**2.3 Mouse Wheel**
```rust
pub fn mouse_wheel(&mut self, delta: i32) -> Result<()> {
    let input = create_wheel_input(delta);
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    Ok(())
}
```

**2.4 Drag and Drop**
```rust
pub fn drag(&mut self, from: (i32, i32), to: (i32, i32)) -> Result<()> {
    self.move_mouse(from.0, from.1)?;
    self.mouse_down(MouseButton::Left)?;
    std::thread::sleep(Duration::from_millis(50));
    self.move_mouse(to.0, to.1)?;
    self.mouse_up(MouseButton::Left)?;
    Ok(())
}
```

#### Testing
- [ ] Test mouse movement (absolute)
- [ ] Test mouse movement (relative)
- [ ] Test all mouse buttons
- [ ] Test double-click
- [ ] Test mouse wheel (up/down)
- [ ] Test drag and drop
- [ ] Measure latency

---

### 3. Clipboard Synchronization (MEDIUM PRIORITY)

**File:** `client/core/src/clipboard.rs` (NEW)  
**Estimated Time:** 2 days

#### Requirements
- Copy text from remote to local
- Paste text from local to remote
- Handle large clipboard data
- Support Unicode text
- Optional: Images and files

#### Implementation

**3.1 Create Clipboard Module**
```rust
use windows::Win32::System::DataExchange::*;

pub struct ClipboardManager {
    last_content: Option<String>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self { last_content: None }
    }
    
    pub fn get_text(&mut self) -> Result<Option<String>, ClientError> {
        unsafe {
            if !OpenClipboard(None).as_bool() {
                return Ok(None);
            }
            
            let handle = GetClipboardData(CF_UNICODETEXT.0);
            if handle.is_invalid() {
                CloseClipboard();
                return Ok(None);
            }
            
            let ptr = GlobalLock(handle) as *const u16;
            let text = String::from_utf16_lossy(
                std::slice::from_raw_parts(ptr, wcslen(ptr))
            );
            
            GlobalUnlock(handle);
            CloseClipboard();
            
            Ok(Some(text))
        }
    }
    
    pub fn set_text(&mut self, text: &str) -> Result<(), ClientError> {
        unsafe {
            if !OpenClipboard(None).as_bool() {
                return Err(ClientError::ClipboardError("Failed to open".into()));
            }
            
            EmptyClipboard();
            
            let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
            let size = wide.len() * 2;
            
            let handle = GlobalAlloc(GMEM_MOVEABLE, size)?;
            let ptr = GlobalLock(handle) as *mut u16;
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            GlobalUnlock(handle);
            
            SetClipboardData(CF_UNICODETEXT.0, handle);
            CloseClipboard();
            
            Ok(())
        }
    }
}
```

**3.2 Add to Protocol**
```rust
// In shared/protocol/src/messages.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    // ... existing
    ClipboardUpdate { content: String },
}
```

#### Testing
- [ ] Copy text remote → local
- [ ] Paste text local → remote
- [ ] Test large text (>1MB)
- [ ] Test Unicode characters
- [ ] Test special characters

---

### 4. Input Event Protocol (MEDIUM PRIORITY)

**File:** `shared/protocol/src/messages.rs`  
**Estimated Time:** 1 day

#### Requirements
- Define input event messages
- Serialize/deserialize efficiently
- Handle event ordering
- Add timestamps

#### Implementation

**4.1 Define Input Events**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    KeyDown { key: Key, timestamp: u64 },
    KeyUp { key: Key, timestamp: u64 },
    MouseMove { x: i32, y: i32, timestamp: u64 },
    MouseDown { button: MouseButton, timestamp: u64 },
    MouseUp { button: MouseButton, timestamp: u64 },
    MouseWheel { delta: i32, timestamp: u64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A, B, C, /* ... */ Z,
    // Numbers
    Num0, Num1, /* ... */ Num9,
    // Special
    Enter, Escape, Backspace, Tab, Space,
    // Modifiers
    Control, Alt, Shift, Win,
    // Function keys
    F1, F2, /* ... */ F12,
    // Arrow keys
    Left, Right, Up, Down,
    // ... more keys
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}
```

#### Testing
- [ ] Serialize/deserialize all event types
- [ ] Test event ordering
- [ ] Measure serialization overhead
- [ ] Test with bincode and JSON

---

### 5. Input Testing & Examples (HIGH PRIORITY)

**File:** `client/windows/examples/input_test.rs` (NEW)  
**Estimated Time:** 1 day

#### Create Test Program

```rust
/// Input injection test example
/// Tests keyboard and mouse input injection
/// 
/// Run with: cargo run --example input_test

use genxlink_client_core::{WindowsInputInjector, InputInjector};
use genxlink_protocol::{Key, MouseButton};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GenXLink Input Injection Test ===\n");
    println!("This will test keyboard and mouse input.");
    println!("Make sure you have Notepad open!\n");
    
    sleep(Duration::from_secs(3)).await;
    
    let mut injector = WindowsInputInjector::new();
    
    // Test keyboard
    println!("Testing keyboard input...");
    injector.inject_key_down(Key::H).await?;
    injector.inject_key_up(Key::H).await?;
    injector.inject_key_down(Key::E).await?;
    injector.inject_key_up(Key::E).await?;
    // ... type "Hello World"
    
    // Test mouse
    println!("Testing mouse movement...");
    injector.move_mouse(500, 500).await?;
    sleep(Duration::from_millis(500)).await;
    
    println!("Testing mouse click...");
    injector.mouse_click(MouseButton::Left).await?;
    
    println!("\n✓ Input test complete!");
    
    Ok(())
}
```

#### Testing Checklist
- [ ] Keyboard injection works
- [ ] Mouse injection works
- [ ] No input lag
- [ ] Works across applications
- [ ] Handles errors gracefully

---

## 🎯 Success Criteria

Phase 3 is complete when:

- [x] Keyboard injection working for all keys
- [x] Mouse injection working (move, click, wheel)
- [x] Clipboard sync working
- [x] Input latency < 50ms
- [x] All tests passing
- [x] Example program working
- [x] Documentation updated

---

## 📊 Performance Targets

| Metric | Target | Acceptable |
|--------|--------|------------|
| Input Latency | <30ms | <50ms |
| Event Processing | >100/sec | >50/sec |
| CPU Usage | <5% | <10% |
| Memory Usage | <50MB | <100MB |

---

## 🐛 Known Challenges

### Challenge 1: UAC Elevation
**Problem:** Input injection may not work for elevated applications  
**Solution:** Run client as administrator or use driver-level injection

### Challenge 2: Input Lag
**Problem:** Network latency affects responsiveness  
**Solution:** Implement input prediction and buffering

### Challenge 3: Key Mapping
**Problem:** Different keyboard layouts  
**Solution:** Use scan codes instead of virtual keys

### Challenge 4: Special Keys
**Problem:** Some key combinations are captured by OS  
**Solution:** Document limitations, handle gracefully

---

## 📝 Development Order

**Week 1:**
1. Day 1-2: Complete keyboard injection
2. Day 3-4: Complete mouse injection
3. Day 5: Testing and bug fixes

**Week 2:**
1. Day 1-2: Clipboard synchronization
2. Day 3: Input event protocol
3. Day 4: Example program and testing
4. Day 5: Documentation and polish

---

## 🔗 Dependencies

**Internal:**
- `genxlink-protocol` - Input event definitions
- `genxlink-client-core` - Input injection implementation

**External:**
- Windows crate - SendInput, SetCursorPos APIs
- Tokio - Async runtime

---

## 📚 Resources

**Windows APIs:**
- [SendInput](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [SetCursorPos](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcursorpos)
- [Clipboard API](https://docs.microsoft.com/en-us/windows/win32/dataxchg/clipboard)

**References:**
- RustDesk input handling
- Synergy input injection
- TeamViewer protocol

---

## ✅ Next Phase

After Phase 3 completion, move to **Phase 4: WebRTC Transport** to enable actual remote connections.

---

**Let's build amazing remote control functionality!** 🎮
