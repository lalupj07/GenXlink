use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub email: String,    // User email
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user_id: String, email: String) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token valid for 24 hours

        Self {
            sub: user_id,
            email,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

/// Get JWT secret from environment or use default
fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        "your-secret-key-change-this-in-production".to_string()
    })
}

/// Generate JWT token for a user
pub fn generate_token(user_id: String, email: String) -> Result<String> {
    let claims = Claims::new(user_id, email);
    let secret = get_jwt_secret();
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Validate JWT token and extract claims
pub fn validate_token(token: &str) -> Result<Claims> {
    let secret = get_jwt_secret();
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Middleware validator for protected routes
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();

    match validate_token(token) {
        Ok(claims) => {
            // Store claims in request extensions for later use
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            let config = req.app_data::<Config>()
                .cloned()
                .unwrap_or_default();
            
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> Result<String> {
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    Ok(hash)
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let valid = bcrypt::verify(password, hash)?;
    Ok(valid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_token() {
        let user_id = "test-user-123".to_string();
        let email = "test@example.com".to_string();

        let token = generate_token(user_id.clone(), email.clone()).unwrap();
        let claims = validate_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
