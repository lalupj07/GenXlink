use async_trait::async_trait;
use crate::ClientError;
use serde::{Serialize, Deserialize};
use genxlink_protocol::{KeyboardEvent, MouseEvent};

/// Input injector trait
pub trait InputInjector: Send + Sync {
    /// Inject a keyboard event
    fn inject_keyboard(&mut self, event: &KeyboardEvent) -> Result<(), ClientError>;
    
    /// Inject a mouse event
    fn inject_mouse(&mut self, event: &MouseEvent) -> Result<(), ClientError>;
}

/// Windows input injection implementation
#[cfg(target_os = "windows")]
pub mod win_impl {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use genxlink_protocol::MouseEventType;
    
    pub struct WindowsInputInjector;
    
    impl WindowsInputInjector {
        pub fn new() -> Self {
            Self
        }
    }
    
    impl Default for WindowsInputInjector {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl InputInjector for WindowsInputInjector {
        fn inject_keyboard(&mut self, event: &KeyboardEvent) -> Result<(), ClientError> {
            unsafe {
                let input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(event.key_code as u16),
                            wScan: event.scan_code as u16,
                            dwFlags: if event.pressed {
                                KEYBD_EVENT_FLAGS(0)
                            } else {
                                KEYEVENTF_KEYUP
                            },
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            
            Ok(())
        }
        
        fn inject_mouse(&mut self, event: &MouseEvent) -> Result<(), ClientError> {
            unsafe {
                match event.event_type {
                    MouseEventType::Move => {
                        let _ = SetCursorPos(event.x, event.y);
                    }
                    MouseEventType::LeftDown => {
                        let input = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: INPUT_0 {
                                mi: MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: MOUSEEVENTF_LEFTDOWN,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                    MouseEventType::LeftUp => {
                        let input = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: INPUT_0 {
                                mi: MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: MOUSEEVENTF_LEFTUP,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                    MouseEventType::RightDown => {
                        let input = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: INPUT_0 {
                                mi: MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: MOUSEEVENTF_RIGHTDOWN,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                    MouseEventType::RightUp => {
                        let input = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: INPUT_0 {
                                mi: MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: MOUSEEVENTF_RIGHTUP,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                    MouseEventType::MiddleDown => {
                        let input = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: INPUT_0 {
                                mi: MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: MOUSEEVENTF_MIDDLEDOWN,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                    MouseEventType::MiddleUp => {
                        let input = INPUT {
                            r#type: INPUT_MOUSE,
                            Anonymous: INPUT_0 {
                                mi: MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: MOUSEEVENTF_MIDDLEUP,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        };
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                    MouseEventType::Wheel { delta } => {
                        let input = INPUT {
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
                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                    }
                }
            }
            
            Ok(())
        }
    }
}

/// Create platform-specific input injector
pub fn create_input_injector() -> Result<Box<dyn InputInjector>, ClientError> {
    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(win_impl::WindowsInputInjector::new()))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err(ClientError::PlatformNotSupported)
    }
}
