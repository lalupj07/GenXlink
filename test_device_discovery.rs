// Simple test for device discovery without the full client
use std::net::UdpSocket;
use std::time::Duration;
use std::thread;

fn main() {
    println!("🚀 Testing GenXLink Device Discovery");
    
    // Test 1: Server Health Check
    println!("\n📡 Testing signaling server...");
    match reqwest::get("http://127.0.0.1:8081/health") {
        Ok(response) => {
            println!("✅ Server health check: {}", response.status());
            if let Ok(body) = response.text() {
                println!("   Response: {}", body);
            }
        }
        Err(e) => {
            println!("❌ Server health check failed: {}", e);
            return;
        }
    }
    
    // Test 2: UDP Broadcast Discovery
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
    
    while start_time.elapsed() < Duration::from_secs(10) {
        match listen_socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                let message = String::from_utf8_lossy(&buf[..len]);
                println!("📨 Received discovery from {}: {}", addr, message);
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
    println!("   ✅ Signaling server: Running on localhost:8081");
    println!("   ✅ LAN discovery: Implemented and listening");
    println!("   📱 Next: Run two instances to test cross-device discovery");
}
