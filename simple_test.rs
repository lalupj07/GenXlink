// Simple test for device discovery without external dependencies
use std::net::UdpSocket;
use std::time::Duration;
use std::thread;

fn main() {
    println!("🚀 Testing GenXLink Device Discovery");
    
    // Test 1: UDP Broadcast Discovery
    println!("\n🔍 Testing LAN discovery...");
    
    // Start listening for discovery messages
    let listen_socket = UdpSocket::bind("0.0.0.0:9090").expect("Failed to bind to port 9090");
    listen_socket.set_nonblocking(true).expect("Failed to set non-blocking");
    
    println!("✅ Listening for device discovery on port 9090");
    
    // Send a discovery broadcast
    let broadcast_socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to create broadcast socket");
    let discovery_message = r#"{"device_id":"test-device-123","device_name":"Test-Device","port":9090}"#;
    
    // Broadcast to common network ranges
    let broadcast_addresses = [
        "255.255.255.255:9090",
        "192.168.255.255:9090", 
        "10.255.255.255:9090",
        "172.16.255.255:9090",
    ];
    
    for addr in &broadcast_addresses {
        if let Err(e) = broadcast_socket.send_to(discovery_message.as_bytes(), addr) {
            println!("⚠️  Failed to send broadcast to {}: {}", addr, e);
        } else {
            println!("📡 Sent discovery broadcast to {}", addr);
        }
    }
    
    // Listen for responses
    println!("\n👂 Listening for device responses (10 seconds)...");
    let mut buf = [0u8; 1024];
    let start_time = std::time::Instant::now();
    let mut devices_found = 0;
    
    while start_time.elapsed() < Duration::from_secs(10) {
        match listen_socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                let message = String::from_utf8_lossy(&buf[..len]);
                println!("📨 Received discovery from {}: {}", addr, message);
                devices_found += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                println!("❌ Error receiving: {}", e);
                break;
            }
        }
    }
    
    println!("\n🎯 Test completed!");
    println!("📊 Summary:");
    println!("   ✅ LAN discovery: Implemented and listening");
    println!("   📱 Devices found: {}", devices_found);
    println!("   🌐 Server: Should be running on localhost:8081");
    println!("   📋 Next: Run this test in multiple terminals to simulate device discovery");
    
    // Test instructions
    println!("\n📝 Instructions for full test:");
    println!("1. Start signaling server: cd server/signaling && cargo run");
    println!("2. Run this test in 2+ terminals: rustc simple_test.rs && ./simple_test.exe");
    println!("3. Devices should discover each other on the network");
}
