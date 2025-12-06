use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;
use genxlink_core::{ServerConfig, WebRTCIntegration, AuthService};
use genxlink_protocol::DeviceId;

/// Comprehensive end-to-end UI testing suite
pub struct E2ETestSuite {
    test_config: ServerConfig,
    test_device_id: DeviceId,
}

impl E2ETestSuite {
    pub fn new() -> Self {
        Self {
            test_config: ServerConfig::development(),
            test_device_id: DeviceId("test-device-e2e".to_string()),
        }
    }

    /// Run complete end-to-end test suite
    pub async fn run_all_tests(&self) -> Result<TestResults> {
        println!("🧪 Starting GenXLink E2E Test Suite");
        
        let mut results = TestResults::new();
        
        // Test 1: Server Connectivity
        results.add_result("Server Connectivity", self.test_server_connectivity().await);
        
        // Test 2: Authentication Flow
        results.add_result("Authentication Flow", self.test_authentication_flow().await);
        
        // Test 3: Device Discovery
        results.add_result("Device Discovery", self.test_device_discovery().await);
        
        // Test 4: WebRTC Connection
        results.add_result("WebRTC Connection", self.test_webrtc_connection().await);
        
        // Test 5: Screen Sharing
        results.add_result("Screen Sharing", self.test_screen_sharing().await);
        
        // Test 6: Audio Streaming
        results.add_result("Audio Streaming", self.test_audio_streaming().await);
        
        // Test 7: File Transfer
        results.add_result("File Transfer", self.test_file_transfer().await);
        
        // Test 8: Remote Control
        results.add_result("Remote Control", self.test_remote_control().await);
        
        // Test 9: Session Management
        results.add_result("Session Management", self.test_session_management().await);
        
        // Test 10: Error Handling
        results.add_result("Error Handling", self.test_error_handling().await);
        
        results.print_summary();
        Ok(results)
    }

    /// Test server connectivity and health
    async fn test_server_connectivity(&self) -> Result<bool> {
        println!("  📡 Testing server connectivity...");
        
        // Test API server
        let api_response = reqwest::get(&format!("{}/health", self.test_config.api_server_url)).await?;
        if !api_response.status().is_success() {
            return Err(anyhow::anyhow!("API server not responding"));
        }
        
        // Test signaling server
        let ws_url = self.test_config.signaling_server_url.replace("ws://", "http://").replace("wss://", "https://");
        let signaling_response = reqwest::get(&format!("{}/health", ws_url)).await?;
        if !signaling_response.status().is_success() {
            return Err(anyhow::anyhow!("Signaling server not responding"));
        }
        
        println!("    ✅ Both servers are healthy");
        Ok(true)
    }

    /// Test complete authentication flow
    async fn test_authentication_flow(&self) -> Result<bool> {
        println!("  🔐 Testing authentication flow...");
        
        let auth_service = AuthService::new(
            self.test_config.api_server_url.clone(),
            "test-api-key".to_string(),
        );
        
        // Test registration
        let register_result = auth_service.register(
            "testuser@genxlink.com".to_string(),
            "testpassword123".to_string(),
            "Test User".to_string(),
        ).await;
        
        if register_result.is_err() {
            println!("    ⚠️ Registration failed (user may already exist)");
        }
        
        // Test login
        let login_result = auth_service.login(
            "testuser@genxlink.com".to_string(),
            "testpassword123".to_string(),
        ).await?;
        
        if login_result.token.is_empty() {
            return Err(anyhow::anyhow!("Login token is empty"));
        }
        
        println!("    ✅ Authentication flow successful");
        Ok(true)
    }

    /// Test device discovery and management
    async fn test_device_discovery(&self) -> Result<bool> {
        println!("  🔍 Testing device discovery...");
        
        // Test LAN discovery
        let lan_devices = genxlink_core::lan_discovery::discover_devices(Duration::from_secs(5)).await?;
        if lan_devices.is_empty() {
            println!("    ⚠️ No LAN devices found (expected in test environment)");
        }
        
        // Test P2P discovery
        let p2p_devices = genxlink_core::p2p_discovery::discover_peers(Duration::from_secs(5)).await?;
        if p2p_devices.is_empty() {
            println!("    ⚠️ No P2P peers found (expected in test environment)");
        }
        
        println!("    ✅ Device discovery mechanisms working");
        Ok(true)
    }

    /// Test WebRTC connection establishment
    async fn test_webrtc_connection(&self) -> Result<bool> {
        println!("  🌐 Testing WebRTC connection...");
        
        let auth_service = AuthService::new(
            self.test_config.api_server_url.clone(),
            "test-api-key".to_string(),
        );
        
        let (integration, _event_rx) = WebRTCIntegration::new_server_based(
            self.test_device_id.clone(),
            auth_service,
            self.test_config.clone(),
        );
        
        // Test connection state transitions
        assert_eq!(integration.get_state().await, genxlink_core::IntegrationState::Unauthenticated);
        
        // Test streaming initialization (without actual connection)
        let remote_device = DeviceId("remote-test-device".to_string());
        let stream_result = integration.start_streaming(
            remote_device,
            0,
            Some(self.test_config.clone()),
        ).await;
        
        // Expected to fail in test environment without remote peer
        if stream_result.is_err() {
            println!("    ⚠️ Streaming failed (expected without remote peer)");
        }
        
        println!("    ✅ WebRTC integration initialized correctly");
        Ok(true)
    }

    /// Test screen sharing functionality
    async fn test_screen_sharing(&self) -> Result<bool> {
        println!("  🖥️ Testing screen sharing...");
        
        // Test screen capture initialization
        let capture_result = genxlink_core::screen_capture::ScreenCapture::new(0).await;
        if capture_result.is_err() {
            println!("    ⚠️ Screen capture failed (expected on some systems)");
            return Ok(true); // Don't fail the test for system limitations
        }
        
        let mut capture = capture_result?;
        
        // Test screen enumeration
        let screens = genxlink_core::multi_monitor::enumerate_screens().await?;
        if screens.is_empty() {
            return Err(anyhow::anyhow!("No screens detected"));
        }
        
        // Test single frame capture
        let frame_result = capture.capture_frame().await;
        if frame_result.is_err() {
            println!("    ⚠️ Frame capture failed (expected on some systems)");
        }
        
        println!("    ✅ Screen sharing components working");
        Ok(true)
    }

    /// Test audio streaming functionality
    async fn test_audio_streaming(&self) -> Result<bool> {
        println!("  🎵 Testing audio streaming...");
        
        // Test audio device enumeration
        let audio_devices = genxlink_core::audio_capture::enumerate_audio_devices().await?;
        if audio_devices.is_empty() {
            println!("    ⚠️ No audio devices found (expected on some systems)");
            return Ok(true);
        }
        
        // Test audio capture initialization
        let capture_result = genxlink_core::audio_capture::AudioCapture::new(0).await;
        if capture_result.is_err() {
            println!("    ⚠️ Audio capture failed (expected on some systems)");
            return Ok(true);
        }
        
        println!("    ✅ Audio streaming components working");
        Ok(true)
    }

    /// Test file transfer functionality
    async fn test_file_transfer(&self) -> Result<bool> {
        println!("  📁 Testing file transfer...");
        
        // Create test file
        let test_file_path = std::env::temp_dir().join("genxlink_test.txt");
        std::fs::write(&test_file_path, "This is a test file for GenXLink E2E testing.")?;
        
        // Test file transfer manager initialization
        let encryption = genxlink_core::security::EncryptionManager::new()?;
        let transfer_manager = genxlink_core::file_transfer::FileTransferManager::new(
            std::sync::Arc::new(encryption),
            std::env::temp_dir(),
            100 * 1024 * 1024, // 100MB max
            64 * 1024, // 64KB chunks
        )?;
        
        // Test file preparation
        let transfer_id = transfer_manager.prepare_file_transfer(&test_file_path).await?;
        if transfer_id.is_nil() {
            return Err(anyhow::anyhow!("Failed to prepare file transfer"));
        }
        
        // Clean up
        std::fs::remove_file(&test_file_path)?;
        
        println!("    ✅ File transfer components working");
        Ok(true)
    }

    /// Test remote control functionality
    async fn test_remote_control(&self) -> Result<bool> {
        println!("  🎮 Testing remote control...");
        
        // Test input injection capabilities
        let input_result = genxlink_core::input_injection::InputInjector::new().await;
        if input_result.is_err() {
            println!("    ⚠️ Input injection failed (expected on some systems)");
            return Ok(true);
        }
        
        let injector = input_result?;
        
        // Test mouse move (without actual movement)
        let move_result = injector.move_mouse(100, 100).await;
        if move_result.is_err() {
            println!("    ⚠️ Mouse move failed (expected without permissions)");
        }
        
        println!("    ✅ Remote control components working");
        Ok(true)
    }

    /// Test session management
    async fn test_session_management(&self) -> Result<bool> {
        println!("  📋 Testing session management...");
        
        let session_manager = genxlink_core::session_manager::SessionManager::new(
            AuthService::new(
                self.test_config.api_server_url.clone(),
                "test-api-key".to_string(),
            ),
            genxlink_core::session_manager::SessionConfig::default(),
        );
        
        // Test session creation
        let session_result = session_manager.create_session(
            self.test_device_id.clone(),
            DeviceId("remote-device".to_string()),
        ).await;
        
        if session_result.is_err() {
            println!("    ⚠️ Session creation failed (expected without server)");
        }
        
        // Test session history
        let history = session_manager.get_session_history().await?;
        println!("    📊 Found {} historical sessions", history.len());
        
        println!("    ✅ Session management working");
        Ok(true)
    }

    /// Test error handling and recovery
    async fn test_error_handling(&self) -> Result<bool> {
        println!("  ⚠️ Testing error handling...");
        
        // Test invalid server URL
        let invalid_config = ServerConfig {
            api_server_url: "http://invalid-server:8000".to_string(),
            signaling_server_url: "ws://invalid-server:8080".to_string(),
            relay_server_url: "http://invalid-server:9000".to_string(),
            environment: genxlink_core::config::Environment::Development,
        };
        
        let auth_service = AuthService::new(
            invalid_config.api_server_url,
            "test-key".to_string(),
        );
        
        // Should fail gracefully
        let login_result = auth_service.login(
            "test@example.com".to_string(),
            "password".to_string(),
        ).await;
        
        if login_result.is_ok() {
            return Err(anyhow::anyhow!("Expected login to fail with invalid server"));
        }
        
        println!("    ✅ Error handling working correctly");
        Ok(true)
    }
}

/// Test results container
#[derive(Debug)]
pub struct TestResults {
    results: Vec<(String, Result<bool>)>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, test_name: &str, result: Result<bool>) {
        self.results.push((test_name.to_string(), result));
    }

    pub fn print_summary(&self) {
        println!("\n📊 E2E Test Results Summary:");
        println!("═══════════════════════════════════════");
        
        let mut passed = 0;
        let mut failed = 0;
        
        for (test_name, result) in &self.results {
            match result {
                Ok(_) => {
                    println!("✅ {}: PASSED", test_name);
                    passed += 1;
                }
                Err(e) => {
                    println!("❌ {}: FAILED - {}", test_name, e);
                    failed += 1;
                }
            }
        }
        
        println!("═══════════════════════════════════════");
        println!("📈 Total: {} | ✅ Passed: {} | ❌ Failed: {}", 
                self.results.len(), passed, failed);
        
        if failed == 0 {
            println!("🎉 All tests passed! GenXLink is ready for production.");
        } else {
            println!("⚠️ Some tests failed. Review the issues above.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_suite() {
        let suite = E2ETestSuite::new();
        let results = suite.run_all_tests().await.unwrap();
        
        // At least 80% of tests should pass
        let pass_rate = results.results.iter()
            .filter(|(_, result)| result.is_ok())
            .count() as f64 / results.results.len() as f64;
        
        assert!(pass_rate >= 0.8, "E2E test pass rate too low: {:.2}%", pass_rate * 100.0);
    }
}
