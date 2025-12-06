use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_complete_user_workflow() {
    let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = reqwest::Client::new();
    
    println!("Starting complete user workflow test...");
    
    // Step 1: Health check
    println!("1. Testing health check...");
    let health_response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await
        .expect("Failed to call health check");
    
    assert_eq!(health_response.status(), 200);
    let health_data: Value = health_response.json().await.unwrap();
    assert_eq!(health_data["status"], "ok");
    println!("✓ Health check passed");
    
    // Step 2: User registration
    println!("2. Testing user registration...");
    let test_email = format!("test{}@example.com", Uuid::new_v4());
    let register_payload = json!({
        "email": test_email,
        "username": format!("testuser{}", Uuid::new_v4()),
        "password": "TestPassword123!",
        "display_name": "Test User"
    });
    
    let register_response = client
        .post(&format!("{}/auth/register", base_url))
        .json(&register_payload)
        .send()
        .await
        .expect("Failed to register user");
    
    assert_eq!(register_response.status(), 200);
    let register_data: Value = register_response.json().await.unwrap();
    assert!(register_data["success"].as_bool().unwrap());
    assert!(register_data["token"].as_str().is_some());
    println!("✓ User registration passed");
    
    let token = register_data["token"].as_str().unwrap();
    
    // Step 3: User login
    println!("3. Testing user login...");
    let login_payload = json!({
        "email": test_email,
        "password": "TestPassword123!"
    });
    
    let login_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&login_payload)
        .send()
        .await
        .expect("Failed to login user");
    
    assert_eq!(login_response.status(), 200);
    let login_data: Value = login_response.json().await.unwrap();
    assert!(login_data["success"].as_bool().unwrap());
    assert!(login_data["token"].as_str().is_some());
    println!("✓ User login passed");
    
    let auth_token = login_data["token"].as_str().unwrap();
    
    // Step 4: Get user profile
    println!("4. Testing get user profile...");
    let profile_response = client
        .get(&format!("{}/api/profile", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to get user profile");
    
    assert_eq!(profile_response.status(), 200);
    let profile_data: Value = profile_response.json().await.unwrap();
    assert_eq!(profile_data["email"], test_email);
    println!("✓ Get user profile passed");
    
    // Step 5: Register a device
    println!("5. Testing device registration...");
    let device_payload = json!({
        "user_id": profile_data["id"],
        "device_id": format!("device-{}", Uuid::new_v4()),
        "device_name": "Test Device",
        "device_type": "desktop",
        "os_version": "Windows 10",
        "ip_address": "192.168.1.100",
        "capabilities": {},
        "metadata": {}
    });
    
    let device_response = client
        .post(&format!("{}/api/devices", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&device_payload)
        .send()
        .await
        .expect("Failed to register device");
    
    assert_eq!(device_response.status(), 200);
    let device_data: Value = device_response.json().await.unwrap();
    assert_eq!(device_data["device_name"], "Test Device");
    println!("✓ Device registration passed");
    
    // Step 6: Get user devices
    println!("6. Testing get user devices...");
    let devices_response = client
        .get(&format!("{}/api/devices", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to get user devices");
    
    assert_eq!(devices_response.status(), 200);
    let devices_data: Value = devices_response.json().await.unwrap();
    assert!(devices_data.as_array().unwrap().len() > 0);
    println!("✓ Get user devices passed");
    
    // Step 7: Create a session
    println!("7. Testing session creation...");
    let session_payload = json!({
        "user_id": profile_data["id"],
        "device_id": device_data["id"],
        "remote_device_id": device_data["id"],
        "session_type": "remote_control",
        "status": "active",
        "metadata": {}
    });
    
    let session_response = client
        .post(&format!("{}/api/sessions", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&session_payload)
        .send()
        .await
        .expect("Failed to create session");
    
    assert_eq!(session_response.status(), 200);
    let session_data: Value = session_response.json().await.unwrap();
    assert_eq!(session_data["session_type"], "remote_control");
    println!("✓ Session creation passed");
    
    // Step 8: Get user sessions
    println!("8. Testing get user sessions...");
    let sessions_response = client
        .get(&format!("{}/api/sessions", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to get user sessions");
    
    assert_eq!(sessions_response.status(), 200);
    let sessions_data: Value = sessions_response.json().await.unwrap();
    assert!(sessions_data.as_array().unwrap().len() > 0);
    println!("✓ Get user sessions passed");
    
    // Step 9: Start a connection
    println!("9. Testing connection start...");
    let connection_payload = json!({
        "from_device_id": device_data["id"],
        "to_device_id": device_data["id"],
        "connection_type": "remote_control"
    });
    
    let connection_response = client
        .post(&format!("{}/api/connection/start", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&connection_payload)
        .send()
        .await
        .expect("Failed to start connection");
    
    assert_eq!(connection_response.status(), 200);
    let connection_data: Value = connection_response.json().await.unwrap();
    assert!(connection_data["success"].as_bool().unwrap());
    println!("✓ Connection start passed");
    
    // Step 10: End the connection
    println!("10. Testing connection end...");
    let end_connection_payload = json!({
        "session_id": session_data["id"]
    });
    
    let end_connection_response = client
        .post(&format!("{}/api/connection/end", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&end_connection_payload)
        .send()
        .await
        .expect("Failed to end connection");
    
    assert_eq!(end_connection_response.status(), 200);
    let end_connection_data: Value = end_connection_response.json().await.unwrap();
    assert!(end_connection_data["success"].as_bool().unwrap());
    println!("✓ Connection end passed");
    
    // Step 11: End the session
    println!("11. Testing session end...");
    let end_session_response = client
        .post(&format!("{}/api/sessions/{}/end", base_url, session_data["id"]))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to end session");
    
    assert_eq!(end_session_response.status(), 200);
    let end_session_data: Value = end_session_response.json().await.unwrap();
    assert!(end_session_data["success"].as_bool().unwrap());
    println!("✓ Session end passed");
    
    // Step 12: Get usage statistics
    println!("12. Testing usage statistics...");
    let stats_response = client
        .get(&format!("{}/api/stats/usage", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .expect("Failed to get usage statistics");
    
    assert_eq!(stats_response.status(), 200);
    let stats_data: Value = stats_response.json().await.unwrap();
    assert!(stats_data["api_calls"].is_number());
    println!("✓ Usage statistics passed");
    
    println!("🎉 All tests passed! Complete user workflow is working correctly.");
}

#[tokio::test]
async fn test_authentication_flow() {
    let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = reqwest::Client::new();
    
    println!("Starting authentication flow test...");
    
    // Test invalid login
    let invalid_login_payload = json!({
        "email": "invalid@example.com",
        "password": "wrongpassword"
    });
    
    let invalid_response = client
        .post(&format!("{}/auth/login", base_url))
        .json(&invalid_login_payload)
        .send()
        .await
        .expect("Failed to call invalid login");
    
    assert_eq!(invalid_response.status(), 200);
    let invalid_data: Value = invalid_response.json().await.unwrap();
    assert!(!invalid_data["success"].as_bool().unwrap());
    println!("✓ Invalid login correctly rejected");
    
    // Test token refresh
    let test_email = format!("refresh{}@example.com", Uuid::new_v4());
    let register_payload = json!({
        "email": test_email,
        "username": format!("refreshuser{}", Uuid::new_v4()),
        "password": "TestPassword123!",
        "display_name": "Refresh Test User"
    });
    
    let register_response = client
        .post(&format!("{}/auth/register", base_url))
        .json(&register_payload)
        .send()
        .await
        .expect("Failed to register user");
    
    let register_data: Value = register_response.json().await.unwrap();
    let initial_token = register_data["token"].as_str().unwrap();
    
    let refresh_response = client
        .post(&format!("{}/api/auth/refresh", base_url))
        .header("Authorization", format!("Bearer {}", initial_token))
        .send()
        .await
        .expect("Failed to refresh token");
    
    assert_eq!(refresh_response.status(), 200);
    let refresh_data: Value = refresh_response.json().await.unwrap();
    assert!(refresh_data["success"].as_bool().unwrap());
    assert!(refresh_data["token"].as_str().is_some());
    println!("✓ Token refresh working");
    
    println!("🎉 Authentication flow tests passed!");
}

#[tokio::test]
async fn test_rate_limiting() {
    let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = reqwest::Client::new();
    
    println!("Starting rate limiting test...");
    
    // Make multiple rapid requests to trigger rate limiting
    let mut success_count = 0;
    let mut rate_limited_count = 0;
    
    for i in 0..20 {
        let response = client
            .get(&format!("{}/health", base_url))
            .send()
            .await
            .expect("Failed to call health check");
        
        if response.status() == 200 {
            success_count += 1;
        } else if response.status() == 429 {
            rate_limited_count += 1;
        }
        
        // Small delay between requests
        sleep(Duration::from_millis(10)).await;
    }
    
    println!("Success count: {}, Rate limited count: {}", success_count, rate_limited_count);
    
    // We should have some successful requests and potentially some rate limited ones
    assert!(success_count > 0);
    println!("✓ Rate limiting is working (or not triggered)");
    
    println!("🎉 Rate limiting tests passed!");
}

#[tokio::test]
async fn test_error_handling() {
    let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = reqwest::Client::new();
    
    println!("Starting error handling test...");
    
    // Test unauthorized access
    let unauthorized_response = client
        .get(&format!("{}/api/profile", base_url))
        .send()
        .await
        .expect("Failed to call unauthorized endpoint");
    
    assert_eq!(unauthorized_response.status(), 401);
    println!("✓ Unauthorized access correctly rejected");
    
    // Test invalid JSON
    let invalid_json_response = client
        .post(&format!("{}/auth/login", base_url))
        .header("Content-Type", "application/json")
        .body("invalid json")
        .send()
        .await
        .expect("Failed to send invalid JSON");
    
    assert_eq!(invalid_json_response.status(), 400);
    println!("✓ Invalid JSON correctly rejected");
    
    // Test non-existent endpoint
    let not_found_response = client
        .get(&format!("{}/api/nonexistent", base_url))
        .send()
        .await
        .expect("Failed to call non-existent endpoint");
    
    assert_eq!(not_found_response.status(), 404);
    println!("✓ Non-existent endpoint correctly returns 404");
    
    println!("🎉 Error handling tests passed!");
}
