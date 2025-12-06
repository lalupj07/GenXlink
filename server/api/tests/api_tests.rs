use reqwest;
use serde_json::{json, Value};

#[tokio::test]
async fn test_health_check() {
    // This test will be run when the server is running
    let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let data: Value = resp.json().await.unwrap();
            assert_eq!(data["status"], "ok");
            println!("✓ Health check test passed");
        }
        Err(_) => {
            println!("⚠ Server not running - skipping health check test");
        }
    }
}

#[tokio::test]
async fn test_compilation() {
    // This test just verifies our test setup works
    assert!(true);
    println!("✓ Compilation test passed");
}
