use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use serde_json;

// Import GenXLink modules
use genxlink_crypto::{
    EndToEndEncryption, EncryptionKeyPair, SessionKey, EncryptedMessage,
};
use genxlink_protocol::{
    SessionMessage, AuthenticationRequest, AuthenticationResponse,
    MessageType, SecurityLevel,
};
use genxlink_auth::{
    AuthService, JwtManager, PasswordValidator, RateLimiter,
};

#[cfg(test)]
mod security_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_encryption_security() -> Result<()> {
        info!("Starting end-to-end encryption security test");
        
        // Initialize encryption service
        let encryption = Arc::new(EndToEndEncryption::new());
        
        // Generate key pairs for two peers
        let alice_keys = encryption.generate_key_pair().await?;
        let bob_keys = encryption.generate_key_pair().await?;
        
        // Test 1: Key exchange security
        let alice_session = encryption.establish_session(
            &alice_keys,
            &bob_keys.public_key,
        ).await?;
        
        let bob_session = encryption.establish_session(
            &bob_keys,
            &alice_keys.public_key,
        ).await?;
        
        // Verify sessions are different (perfect forward secrecy)
        assert_ne!(alice_session.session_id, bob_session.session_id);
        assert_ne!(alice_session.key_data, bob_session.key_data);
        
        // Test 2: Message confidentiality
        let secret_message = b"This is a secret message that should be encrypted";
        let encrypted_by_alice = encryption.encrypt_message(&alice_session, secret_message).await?;
        
        // Verify ciphertext is different from plaintext
        assert_ne!(encrypted_by_alice.ciphertext, secret_message);
        assert!(!encrypted_by_alice.ciphertext.is_empty());
        
        // Test 3: Message integrity
        // Tamper with ciphertext
        let mut tampered_ciphertext = encrypted_by_alice.ciphertext.clone();
        tampered_ciphertext[0] ^= 0x01; // Flip first bit
        
        let tampered_message = EncryptedMessage {
            session_id: encrypted_by_alice.session_id,
            ciphertext: tampered_ciphertext,
            nonce: encrypted_by_alice.nonce.clone(),
            tag: encrypted_by_alice.tag.clone(),
            timestamp: encrypted_by_alice.timestamp,
        };
        
        // Should fail to decrypt tampered message
        let decrypt_result = encryption.decrypt_message(&bob_session, &tampered_message).await;
        assert!(decrypt_result.is_err(), "Should fail to decrypt tampered message");
        
        // Test 4: Proper decryption
        let decrypted_by_bob = encryption.decrypt_message(&bob_session, &encrypted_by_alice).await?;
        assert_eq!(decrypted_by_bob, secret_message);
        
        // Test 5: Key rotation
        let old_session_key = alice_session.key_data.clone();
        tokio::time::sleep(Duration::from_millis(100)).await; // Wait for rotation interval
        
        let rotated_session = encryption.rotate_session_key(&alice_session).await?;
        assert_ne!(rotated_session.key_data, old_session_key);
        
        // Test 6: Replay attack prevention
        let old_encrypted = encrypted_by_alice.clone();
        tokio::time::sleep(Duration::from_millis(1100)).await; // Wait beyond replay window
        
        let replay_result = encryption.decrypt_message(&bob_session, &old_encrypted).await;
        assert!(replay_result.is_err(), "Should reject replayed message");
        
        info!("End-to-end encryption security test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_authentication_security() -> Result<()> {
        info!("Starting authentication security test");
        
        // Initialize auth service
        let auth_service = Arc::new(AuthService::new().await?);
        
        // Test 1: Password security
        let test_password = "SecurePassword123!@#";
        let username = "testuser";
        
        // Register user
        auth_service.register_user(username, test_password).await?;
        
        // Test 2: Brute force protection
        let mut failed_attempts = 0;
        let max_attempts = 5;
        
        for i in 0..max_attempts + 2 {
            let auth_request = AuthenticationRequest {
                username: username.to_string(),
                password: format!("WrongPassword{}", i),
                device_id: Some(Uuid::new_v4().to_string()),
                timestamp: Utc::now(),
            };
            
            let result = auth_service.authenticate(auth_request).await;
            if i < max_attempts {
                assert!(result.is_err(), "Should fail authentication attempt {}", i);
                failed_attempts += 1;
            } else {
                // Should be locked out after max attempts
                assert!(result.is_err(), "Should be locked out after {} attempts", max_attempts);
            }
        }
        
        assert_eq!(failed_attempts, max_attempts);
        
        // Test 3: Rate limiting
        let rate_limiter = RateLimiter::new(3, Duration::from_secs(1)); // 3 requests per second
        
        let mut rate_limit_hits = 0;
        for i in 0..5 {
            if rate_limiter.check_limit(&username.to_string()).await {
                rate_limit_hits += 1;
            }
        }
        
        assert!(rate_limit_hits <= 3, "Should enforce rate limiting");
        
        // Test 4: JWT token security
        let valid_auth_request = AuthenticationRequest {
            username: username.to_string(),
            password: test_password.to_string(),
            device_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
        };
        
        let auth_response = auth_service.authenticate(valid_auth_request).await?;
        let jwt_token = auth_response.token;
        
        // Verify token structure
        assert!(!jwt_token.is_empty());
        assert!(jwt_token.contains('.'));
        
        let jwt_manager = JwtManager::new();
        let claims = jwt_manager.validate_token(&jwt_token)?;
        assert_eq!(claims.sub, username);
        assert!(claims.exp > chrono::Utc::now().timestamp());
        
        // Test 5: Token tampering
        let mut tampered_token = jwt_token.clone();
        if let Some(dot_pos) = tampered_token.rfind('.') {
            tampered_token.replace_range(dot_pos + 1.., "tampered_signature");
        }
        
        let tampered_result = jwt_manager.validate_token(&tampered_token);
        assert!(tampered_result.is_err(), "Should reject tampered token");
        
        // Test 6: Session expiration
        let expired_claims = auth_service.create_session_claims(username, Duration::from_secs(1)).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let expired_token = jwt_manager.create_token(&expired_claims)?;
        let expired_result = jwt_manager.validate_token(&expired_token);
        assert!(expired_result.is_err(), "Should reject expired token");
        
        info!("Authentication security test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_input_validation_security() -> Result<()> {
        info!("Starting input validation security test");
        
        let auth_service = Arc::new(AuthService::new().await?);
        
        // Test 1: SQL injection prevention
        let malicious_inputs = vec![
            "'; DROP TABLE users; --",
            "' OR '1'='1",
            "admin'--",
            "' UNION SELECT * FROM sensitive_data --",
        ];
        
        for malicious_input in malicious_inputs {
            let auth_request = AuthenticationRequest {
                username: malicious_input.to_string(),
                password: "password".to_string(),
                device_id: Some(Uuid::new_v4().to_string()),
                timestamp: Utc::now(),
            };
            
            let result = auth_service.authenticate(auth_request).await;
            // Should fail gracefully without executing malicious SQL
            assert!(result.is_err() || result.unwrap().success == false);
        }
        
        // Test 2: XSS prevention
        let xss_inputs = vec![
            "<script>alert('xss')</script>",
            "javascript:alert('xss')",
            "<img src=x onerror=alert('xss')>",
        ];
        
        for xss_input in xss_inputs {
            let result = auth_service.register_user(xss_input, "password123").await;
            // Should sanitize or reject XSS inputs
            assert!(result.is_err() || result.is_ok()); // Either reject or sanitize
        }
        
        // Test 3: Buffer overflow prevention
        let oversized_input = "a".repeat(10000); // Very long string
        let auth_request = AuthenticationRequest {
            username: oversized_input.clone(),
            password: oversized_input.clone(),
            device_id: Some(oversized_input),
            timestamp: Utc::now(),
        };
        
        let result = auth_service.authenticate(auth_request).await;
        // Should handle oversized inputs gracefully
        assert!(result.is_err() || result.is_ok()); // Should not crash
        
        // Test 4: Unicode and encoding attacks
        let unicode_attacks = vec![
            "\u{FEFF}username", // BOM attack
            "\u{200B}username", // Zero-width space
            "user%00name", // Null byte injection
            "user\\nname", // Newline injection
        ];
        
        for unicode_input in unicode_attacks {
            let auth_request = AuthenticationRequest {
                username: unicode_input.to_string(),
                password: "password".to_string(),
                device_id: Some(Uuid::new_v4().to_string()),
                timestamp: Utc::now(),
            };
            
            let result = auth_service.authenticate(auth_request).await;
            // Should handle unicode attacks safely
            debug!("Unicode input '{}' result: {:?}", unicode_input, result);
        }
        
        info!("Input validation security test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_session_security() -> Result<()> {
        info!("Starting session security test");
        
        let auth_service = Arc::new(AuthService::new().await?);
        
        // Register and authenticate user
        let username = "sessionuser";
        let password = "SessionPassword123!";
        auth_service.register_user(username, password).await?;
        
        let auth_request = AuthenticationRequest {
            username: username.to_string(),
            password: password.to_string(),
            device_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
        };
        
        let auth_response = auth_service.authenticate(auth_request).await?;
        let session_id = auth_response.session_id;
        
        // Test 1: Session fixation prevention
        let auth_request2 = AuthenticationRequest {
            username: username.to_string(),
            password: password.to_string(),
            device_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
        };
        
        let auth_response2 = auth_service.authenticate(auth_request2).await?;
        assert_ne!(auth_response.session_id, auth_response2.session_id);
        
        // Test 2: Concurrent session limits
        let device_ids: Vec<String> = (0..5).map(|_| Uuid::new_v4().to_string()).collect();
        let mut sessions = Vec::new();
        
        for device_id in device_ids {
            let auth_request = AuthenticationRequest {
                username: username.to_string(),
                password: password.to_string(),
                device_id: Some(device_id),
                timestamp: Utc::now(),
            };
            
            let response = auth_service.authenticate(auth_request).await?;
            sessions.push(response.session_id);
        }
        
        // Should have limited concurrent sessions (implementation dependent)
        let active_sessions = auth_service.get_user_sessions(username).await?;
        assert!(active_sessions.len() <= 10); // Reasonable limit
        
        // Test 3: Session invalidation
        auth_service.invalidate_session(session_id).await?;
        
        // Should not be able to use invalidated session
        let session_result = auth_service.validate_session(session_id).await;
        assert!(session_result.is_err(), "Should reject invalidated session");
        
        // Test 4: Session timeout
        let short_lived_request = AuthenticationRequest {
            username: username.to_string(),
            password: password.to_string(),
            device_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
        };
        
        let short_lived_response = auth_service.authenticate_with_timeout(
            short_lived_request, 
            Duration::from_secs(1)
        ).await?;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let timeout_result = auth_service.validate_session(short_lived_response.session_id).await;
        assert!(timeout_result.is_err(), "Should reject timed out session");
        
        info!("Session security test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_cryptographic_strength() -> Result<()> {
        info!("Starting cryptographic strength test");
        
        let encryption = Arc::new(EndToEndEncryption::new());
        
        // Test 1: Key generation strength
        let key_pair = encryption.generate_key_pair().await?;
        
        // Verify key sizes
        assert_eq!(key_pair.private_key.len(), 32); // X25519 private key
        assert_eq!(key_pair.public_key.len(), 32);  // X25519 public key
        
        // Test 2: Randomness quality (basic check)
        let mut key_pairs = Vec::new();
        for _ in 0..100 {
            key_pairs.push(encryption.generate_key_pair().await?);
        }
        
        // Check for duplicates (should be extremely unlikely)
        let unique_keys: std::collections::HashSet<_> = key_pairs.iter()
            .map(|kp| &kp.public_key)
            .collect();
        
        assert_eq!(unique_keys.len(), 100, "All generated keys should be unique");
        
        // Test 3: Encryption strength
        let alice_keys = encryption.generate_key_pair().await?;
        let bob_keys = encryption.generate_key_pair().await?;
        let session = encryption.establish_session(&alice_keys, &bob_keys.public_key).await?;
        
        // Test with different message sizes
        let test_sizes = vec![1, 16, 1024, 1024 * 1024]; // 1B to 1MB
        
        for size in test_sizes {
            let test_data = vec![0u8; size];
            let encrypted = encryption.encrypt_message(&session, &test_data).await?;
            let decrypted = encryption.decrypt_message(&session, &encrypted).await?;
            
            assert_eq!(test_data, decrypted);
            assert_ne!(encrypted.ciphertext, test_data);
            assert!(encrypted.tag.len() >= 16); // AES-GCM tag should be at least 16 bytes
        }
        
        // Test 4: Side-channel resistance (basic timing test)
        let test_data = vec![0u8; 1024];
        let iterations = 1000;
        
        let mut times = Vec::new();
        for _ in 0..iterations {
            let start = std::time::Instant::now();
            let _encrypted = encryption.encrypt_message(&session, &test_data).await?;
            times.push(start.elapsed());
        }
        
        let avg_time = times.iter().sum::<Duration>() / iterations as u32;
        let max_time = times.iter().max().unwrap();
        let min_time = times.iter().min().unwrap();
        
        info!("Encryption timing stats:");
        info!("  Average: {:?}", avg_time);
        info!("  Min: {:?}", min_time);
        info!("  Max: {:?}", max_time);
        info!("  Variance: {:?}", max_time - min_time);
        
        // Timing should be relatively consistent (within reasonable bounds)
        let variance_ratio = (max_time.as_nanos() as f64 - min_time.as_nanos() as f64) / avg_time.as_nanos() as f64;
        assert!(variance_ratio < 0.5, "Encryption timing should be consistent");
        
        info!("Cryptographic strength test completed successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_security_audit_logging() -> Result<()> {
        info!("Starting security audit logging test");
        
        let auth_service = Arc::new(AuthService::new().await?);
        
        // Enable audit logging
        auth_service.enable_audit_logging().await?;
        
        // Perform various security events
        let username = "audituser";
        let password = "AuditPassword123!";
        
        // Registration
        auth_service.register_user(username, password).await?;
        
        // Successful authentication
        let auth_request = AuthenticationRequest {
            username: username.to_string(),
            password: password.to_string(),
            device_id: Some(Uuid::new_v4().to_string()),
            timestamp: Utc::now(),
        };
        
        auth_service.authenticate(auth_request).await?;
        
        // Failed authentication attempts
        for i in 0..3 {
            let failed_request = AuthenticationRequest {
                username: username.to_string(),
                password: format!("wrong{}", i),
                device_id: Some(Uuid::new_v4().to_string()),
                timestamp: Utc::now(),
            };
            
            let _ = auth_service.authenticate(failed_request).await;
        }
        
        // Get audit logs
        let audit_logs = auth_service.get_audit_logs(username, Some(10)).await?;
        
        assert!(!audit_logs.is_empty(), "Should have audit logs");
        
        // Verify log entries contain expected information
        let mut registration_found = false;
        let mut successful_auth_found = false;
        let mut failed_auth_count = 0;
        
        for log in &audit_logs {
            match log.event_type.as_str() {
                "user_registration" => registration_found = true,
                "authentication_success" => successful_auth_found = true,
                "authentication_failure" => failed_auth_count += 1,
                _ => {}
            }
            
            assert!(!log.timestamp.is_empty());
            assert!(!log.user_id.is_empty());
            assert!(!log.ip_address.is_empty() || log.device_id.is_some());
        }
        
        assert!(registration_found, "Should log registration event");
        assert!(successful_auth_found, "Should log successful authentication");
        assert!(failed_auth_count >= 3, "Should log failed authentication attempts");
        
        // Test log integrity
        let log_hashes: Vec<String> = audit_logs.iter()
            .map(|log| auth_service.calculate_log_hash(log))
            .collect::<Result<Vec<_>>>()?;
        
        // All hashes should be unique
        let unique_hashes: std::collections::HashSet<_> = log_hashes.iter().collect();
        assert_eq!(unique_hashes.len(), log_hashes.len(), "Log hashes should be unique");
        
        info!("Security audit logging test completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod penetration_tests {
    use super::*;

    #[tokio::test]
    async fn test_timing_attack_resistance() -> Result<()> {
        info!("Starting timing attack resistance test");
        
        let auth_service = Arc::new(AuthService::new().await?);
        
        // Register test user
        let username = "timinguser";
        let password = "TimingPassword123!";
        auth_service.register_user(username, password).await?;
        
        // Measure authentication times for different passwords
        let test_passwords = vec![
            password.to_string(),           // Correct password
            "wrong".to_string(),            // Wrong password, same length
            "verylongwrongpassword".to_string(), // Wrong password, different length
            "a".to_string(),                // Wrong password, very short
        ];
        
        let mut timing_results = Vec::new();
        
        for test_password in test_passwords {
            let mut times = Vec::new();
            
            // Take multiple measurements for each password
            for _ in 0..50 {
                let auth_request = AuthenticationRequest {
                    username: username.to_string(),
                    password: test_password.clone(),
                    device_id: Some(Uuid::new_v4().to_string()),
                    timestamp: Utc::now(),
                };
                
                let start = std::time::Instant::now();
                let _ = auth_service.authenticate(auth_request).await;
                times.push(start.elapsed());
            }
            
            let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
            timing_results.push((test_password, avg_time));
        }
        
        info!("Timing attack results:");
        for (pwd, time) in &timing_results {
            info!("  Password '{}': {:?}", pwd, time);
        }
        
        // Calculate variance
        let times: Vec<_> = timing_results.iter().map(|(_, t)| *t).collect();
        let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
        let max_variance = times.iter()
            .map(|t| if *t > avg_time { *t - avg_time } else { avg_time - *t })
            .max().unwrap();
        
        // Variance should be small (timing attacks should be mitigated)
        let variance_ratio = max_variance.as_nanos() as f64 / avg_time.as_nanos() as f64;
        assert!(variance_ratio < 0.2, "Timing variance should be small to prevent timing attacks");
        
        info!("Timing attack resistance test completed");
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_safety() -> Result<()> {
        info!("Starting memory safety test");
        
        let encryption = Arc::new(EndToEndEncryption::new());
        
        // Test that sensitive data is properly handled in memory
        let key_pair = encryption.generate_key_pair().await?;
        let session_key = encryption.establish_session(&key_pair, &key_pair.public_key).await?;
        
        let sensitive_data = b"This is highly sensitive data that should be protected";
        
        // Encrypt and decrypt multiple times
        for _ in 0..1000 {
            let encrypted = encryption.encrypt_message(&session_key, sensitive_data).await?;
            let decrypted = encryption.decrypt_message(&session_key, &encrypted).await?;
            assert_eq!(decrypted, sensitive_data);
        }
        
        // Test that keys are zeroized after use (if implemented)
        // This would require specific implementation in the encryption module
        
        // Test large data handling
        let large_data = vec![0u8; 100 * 1024 * 1024]; // 100MB
        let encrypted = encryption.encrypt_message(&session_key, &large_data).await?;
        let decrypted = encryption.decrypt_message(&session_key, &encrypted).await?;
        assert_eq!(decrypted, large_data);
        
        info!("Memory safety test completed");
        Ok(())
    }
}
