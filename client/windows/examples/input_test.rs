/// Input Injection Test Example
/// 
/// This example demonstrates the input injection capabilities:
/// - Keyboard events
/// - Mouse events  
/// - Clipboard synchronization
///
/// Run with: cargo run --example input_test

use genxlink_client_core::{create_input_injector, create_clipboard_manager, InputInjector, ClipboardManager};
use genxlink_protocol::{KeyboardEvent, MouseEvent, MouseEventType, KeyModifiers, ClipboardData};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GenXLink Input Injection Test");
    println!("==============================\n");
    
    // Create input injector
    let mut injector = create_input_injector()?;
    println!("✓ Input injector created");
    
    // Create clipboard manager
    let mut clipboard = create_clipboard_manager()?;
    println!("✓ Clipboard manager created\n");
    
    // Test 1: Mouse Movement
    println!("Test 1: Mouse Movement");
    println!("Moving mouse to center of screen...");
    let mouse_move = MouseEvent {
        x: 960,
        y: 540,
        event_type: MouseEventType::Move,
    };
    injector.inject_mouse(&mouse_move)?;
    thread::sleep(Duration::from_millis(500));
    println!("✓ Mouse moved\n");
    
    // Test 2: Mouse Clicks
    println!("Test 2: Mouse Clicks");
    println!("Simulating left click...");
    
    let left_down = MouseEvent {
        x: 960,
        y: 540,
        event_type: MouseEventType::LeftDown,
    };
    injector.inject_mouse(&left_down)?;
    thread::sleep(Duration::from_millis(50));
    
    let left_up = MouseEvent {
        x: 960,
        y: 540,
        event_type: MouseEventType::LeftUp,
    };
    injector.inject_mouse(&left_up)?;
    println!("✓ Left click simulated\n");
    
    // Test 3: Keyboard Input
    println!("Test 3: Keyboard Input");
    println!("Simulating key press (A key)...");
    
    let key_down = KeyboardEvent {
        key_code: 0x41, // 'A' key
        scan_code: 0,
        pressed: true,
        modifiers: KeyModifiers {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        },
    };
    injector.inject_keyboard(&key_down)?;
    thread::sleep(Duration::from_millis(50));
    
    let key_up = KeyboardEvent {
        key_code: 0x41,
        scan_code: 0,
        pressed: false,
        modifiers: KeyModifiers {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        },
    };
    injector.inject_keyboard(&key_up)?;
    println!("✓ Key press simulated\n");
    
    // Test 4: Clipboard
    println!("Test 4: Clipboard Synchronization");
    
    // Set clipboard
    let test_data = ClipboardData {
        content_type: "text/plain".to_string(),
        data: "Hello from GenXLink!".as_bytes().to_vec(),
    };
    clipboard.set_clipboard(&test_data)?;
    println!("✓ Clipboard set to: 'Hello from GenXLink!'");
    
    // Get clipboard
    thread::sleep(Duration::from_millis(100));
    let clipboard_content = clipboard.get_clipboard()?;
    let text = String::from_utf8(clipboard_content.data)?;
    println!("✓ Clipboard read: '{}'", text);
    
    if text == "Hello from GenXLink!" {
        println!("✓ Clipboard synchronization working!\n");
    } else {
        println!("⚠ Clipboard content mismatch\n");
    }
    
    // Test 5: Mouse Wheel
    println!("Test 5: Mouse Wheel");
    println!("Simulating mouse wheel scroll...");
    let wheel = MouseEvent {
        x: 960,
        y: 540,
        event_type: MouseEventType::Wheel { delta: 120 },
    };
    injector.inject_mouse(&wheel)?;
    println!("✓ Mouse wheel scrolled\n");
    
    // Test 6: Middle Mouse Button
    println!("Test 6: Middle Mouse Button");
    println!("Simulating middle click...");
    
    let middle_down = MouseEvent {
        x: 960,
        y: 540,
        event_type: MouseEventType::MiddleDown,
    };
    injector.inject_mouse(&middle_down)?;
    thread::sleep(Duration::from_millis(50));
    
    let middle_up = MouseEvent {
        x: 960,
        y: 540,
        event_type: MouseEventType::MiddleUp,
    };
    injector.inject_mouse(&middle_up)?;
    println!("✓ Middle click simulated\n");
    
    println!("==============================");
    println!("All input injection tests completed successfully!");
    println!("\nPhase 3 Features:");
    println!("  ✓ Keyboard input injection");
    println!("  ✓ Mouse input injection (move, click, wheel)");
    println!("  ✓ Middle mouse button support");
    println!("  ✓ Clipboard synchronization");
    println!("\nReady for remote control functionality!");
    
    Ok(())
}
