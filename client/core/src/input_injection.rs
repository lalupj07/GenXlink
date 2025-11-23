use anyhow::Result;
use genxlink_protocol::input::{InputEvent, KeyCode, KeyModifiers, MouseButton};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Input injection for Windows
/// Injects mouse and keyboard events into the system
pub struct InputInjector {
    /// Screen dimensions for coordinate mapping
    screen_width: i32,
    screen_height: i32,
}

impl InputInjector {
    pub fn new() -> Result<Self> {
        // Get screen dimensions
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        
        Ok(Self {
            screen_width,
            screen_height,
        })
    }
    
    /// Inject an input event
    pub fn inject_event(&self, event: &InputEvent) -> Result<()> {
        match event {
            InputEvent::MouseMove { x, y } => {
                self.inject_mouse_move(*x, *y)?;
            }
            InputEvent::MouseMoveDelta { dx, dy } => {
                self.inject_mouse_move_delta(*dx, *dy)?;
            }
            InputEvent::MouseDown { button, x, y } => {
                self.inject_mouse_move(*x, *y)?;
                self.inject_mouse_button(*button, true)?;
            }
            InputEvent::MouseUp { button, x, y } => {
                self.inject_mouse_move(*x, *y)?;
                self.inject_mouse_button(*button, false)?;
            }
            InputEvent::MouseWheel { delta, x, y } => {
                self.inject_mouse_move(*x, *y)?;
                self.inject_mouse_wheel(*delta)?;
            }
            InputEvent::KeyDown { key, modifiers } => {
                self.inject_modifiers(modifiers, true)?;
                self.inject_key(key.0, true)?;
            }
            InputEvent::KeyUp { key, modifiers } => {
                self.inject_key(key.0, false)?;
                self.inject_modifiers(modifiers, false)?;
            }
            InputEvent::TextInput { text } => {
                self.inject_text(text)?;
            }
        }
        
        Ok(())
    }
    
    /// Inject mouse movement (absolute coordinates)
    fn inject_mouse_move(&self, x: i32, y: i32) -> Result<()> {
        // Convert to normalized coordinates (0-65535)
        let normalized_x = (x * 65535) / self.screen_width;
        let normalized_y = (y * 65535) / self.screen_height;
        
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: normalized_x,
                    dy: normalized_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    /// Inject mouse movement (relative delta)
    fn inject_mouse_move_delta(&self, dx: i32, dy: i32) -> Result<()> {
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    /// Inject mouse button press/release
    fn inject_mouse_button(&self, button: MouseButton, down: bool) -> Result<()> {
        let flags = match (button, down) {
            (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
            (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
            (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
            (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
            (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
            (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEUP,
            (MouseButton::X1, true) => MOUSEEVENTF_XDOWN,
            (MouseButton::X1, false) => MOUSEEVENTF_XUP,
            (MouseButton::X2, true) => MOUSEEVENTF_XDOWN,
            (MouseButton::X2, false) => MOUSEEVENTF_XUP,
        };
        
        let mouse_data = match button {
            MouseButton::X1 => 1,
            MouseButton::X2 => 2,
            _ => 0,
        };
        
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: mouse_data,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    /// Inject mouse wheel scroll
    fn inject_mouse_wheel(&self, delta: i32) -> Result<()> {
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: delta as u32,
                    dwFlags: MOUSEEVENTF_WHEEL,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    /// Inject keyboard key press/release
    fn inject_key(&self, vk_code: u32, down: bool) -> Result<()> {
        let flags = if down {
            KEYBD_EVENT_FLAGS(0)
        } else {
            KEYEVENTF_KEYUP
        };
        
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk_code as u16),
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        unsafe {
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        
        Ok(())
    }
    
    /// Inject modifier keys
    fn inject_modifiers(&self, modifiers: &KeyModifiers, down: bool) -> Result<()> {
        if modifiers.shift {
            self.inject_key(VK_SHIFT.0 as u32, down)?;
        }
        if modifiers.ctrl {
            self.inject_key(VK_CONTROL.0 as u32, down)?;
        }
        if modifiers.alt {
            self.inject_key(VK_MENU.0 as u32, down)?;
        }
        if modifiers.meta {
            self.inject_key(VK_LWIN.0 as u32, down)?;
        }
        
        Ok(())
    }
    
    /// Inject text input (Unicode)
    fn inject_text(&self, text: &str) -> Result<()> {
        for ch in text.chars() {
            // Send as Unicode
            let mut input_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            
            let mut input_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            
            unsafe {
                SendInput(&[input_down, input_up], std::mem::size_of::<INPUT>() as i32);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_injector_creation() {
        let injector = InputInjector::new();
        assert!(injector.is_ok());
    }
}
