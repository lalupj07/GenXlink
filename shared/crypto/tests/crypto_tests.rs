use genxlink_crypto::{sign_data, verify_signature, generate_keypair};

#[test]
fn test_keypair_generation() {
    let result = generate_keypair();
    assert!(result.is_ok(), "Keypair generation should succeed");
    
    let (private_key, public_key) = result.unwrap();
    // Keys are generated, just verify they exist
    assert!(private_key.size() > 0);
    assert!(public_key.size() > 0);
}

#[test]
fn test_signature_creation_and_verification() {
    let (private_key, public_key) = generate_keypair().expect("Failed to generate keypair");
    
    let data = b"Test data for signing";
    
    // Sign the data
    let signature = sign_data(&private_key, data);
    assert!(signature.is_ok(), "Signing should succeed");
    
    let sig = signature.unwrap();
    assert!(!sig.is_empty());
    
    // Verify the signature
    let verification = verify_signature(&public_key, data, &sig);
    assert!(verification.is_ok(), "Verification should succeed");
    assert!(verification.unwrap(), "Signature should be valid");
}

#[test]
fn test_signature_verification_fails_with_wrong_data() {
    let (private_key, public_key) = generate_keypair().expect("Failed to generate keypair");
    
    let data = b"Original data";
    let wrong_data = b"Modified data";
    
    // Sign original data
    let signature = sign_data(&private_key, data).expect("Failed to sign");
    
    // Try to verify with wrong data
    let verification = verify_signature(&public_key, wrong_data, &signature);
    assert!(verification.is_ok(), "Verification should not error");
    assert!(!verification.unwrap(), "Signature should be invalid for wrong data");
}

#[test]
fn test_signature_verification_fails_with_wrong_key() {
    let (private_key1, _) = generate_keypair().expect("Failed to generate keypair 1");
    let (_, public_key2) = generate_keypair().expect("Failed to generate keypair 2");
    
    let data = b"Test data";
    
    // Sign with key 1
    let signature = sign_data(&private_key1, data).expect("Failed to sign");
    
    // Try to verify with key 2
    let verification = verify_signature(&public_key2, data, &signature);
    assert!(verification.is_ok(), "Verification should not error");
    assert!(!verification.unwrap(), "Signature should be invalid with wrong key");
}
