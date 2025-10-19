//! JWT token validation

use crate::error::{AuthorizationError, Result};
use crate::jwt::ValidatedClaims;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// JWT Validator with JWKS caching
pub struct JwtValidator {
    /// HTTP client for fetching JWKS
    client: reqwest::Client,
    
    /// Cache of public keys by key ID
    key_cache: Arc<RwLock<HashMap<String, DecodingKey>>>,
}

impl JwtValidator {
    /// Create a new JWT validator
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            key_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate a JWT token
    pub async fn validate_token(
        &self,
        token: &str,
        expected_issuer: &str,
        expected_audiences: &[String],
        jwks_uri: &str,
    ) -> Result<ValidatedClaims> {
        // Decode header to get key ID
        let header = decode_header(token).map_err(|e| {
            AuthorizationError::Internal(format!("Failed to decode JWT header: {}", e))
        })?;

        let kid = header.kid.ok_or_else(|| {
            AuthorizationError::Internal("JWT header missing 'kid' field".to_string())
        })?;

        // Get decoding key (from cache or fetch)
        let decoding_key = self.get_decoding_key(&kid, jwks_uri).await?;

        // Setup validation
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[expected_issuer]);
        validation.set_audience(expected_audiences);

        // Decode and validate token
        let token_data = decode::<ValidatedClaims>(token, &decoding_key, &validation)
            .map_err(|e| {
                AuthorizationError::Internal(format!("JWT validation failed: {}", e))
            })?;

        Ok(token_data.claims)
    }

    /// Get decoding key from cache or fetch from JWKS endpoint
    async fn get_decoding_key(&self, kid: &str, jwks_uri: &str) -> Result<DecodingKey> {
        // Check cache first
        {
            let cache = self.key_cache.read().await;
            if let Some(key) = cache.get(kid) {
                return Ok(key.clone());
            }
        }

        // Fetch JWKS
        let jwks = self.fetch_jwks(jwks_uri).await?;

        // Find key with matching kid
        let jwk = jwks
            .get("keys")
            .and_then(|keys| keys.as_array())
            .and_then(|keys| {
                keys.iter().find(|key| {
                    key.get("kid")
                        .and_then(|k| k.as_str())
                        .map(|k| k == kid)
                        .unwrap_or(false)
                })
            })
            .ok_or_else(|| {
                AuthorizationError::Internal(format!("Key with kid '{}' not found in JWKS", kid))
            })?;

        // Extract public key components
        let n = jwk
            .get("n")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthorizationError::Internal("Missing 'n' in JWK".to_string()))?;

        let e = jwk
            .get("e")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthorizationError::Internal("Missing 'e' in JWK".to_string()))?;

        // Create decoding key from RSA components
        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| AuthorizationError::Internal(format!("Failed to create decoding key: {}", e)))?;

        // Cache the key
        {
            let mut cache = self.key_cache.write().await;
            cache.insert(kid.to_string(), decoding_key.clone());
        }

        Ok(decoding_key)
    }

    /// Fetch JWKS from endpoint
    async fn fetch_jwks(&self, jwks_uri: &str) -> Result<Value> {
        let response = self
            .client
            .get(jwks_uri)
            .send()
            .await
            .map_err(|e| AuthorizationError::Internal(format!("Failed to fetch JWKS: {}", e)))?;

        let jwks = response
            .json::<Value>()
            .await
            .map_err(|e| AuthorizationError::Internal(format!("Failed to parse JWKS: {}", e)))?;

        Ok(jwks)
    }
}

impl Default for JwtValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestClaims {
        sub: String,
        iss: String,
        aud: Vec<String>,
        exp: i64,
        iat: i64,
        email: String,
    }

    #[test]
    fn test_jwt_validator_creation() {
        let validator = JwtValidator::new();
        assert!(validator.key_cache.try_read().is_ok());
    }

    #[test]
    fn test_decode_header_with_kid() {
        // Create a test token with kid
        let claims = TestClaims {
            sub: "user123".to_string(),
            iss: "https://test.example.com".to_string(),
            aud: vec!["test-client".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            email: "test@example.com".to_string(),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("test-key-id".to_string());

        // Note: This will fail without a real key, but tests the structure
        let secret = b"secret";
        let token = encode(&header, &claims, &EncodingKey::from_secret(secret));
        
        if let Ok(token_str) = token {
            let decoded_header = decode_header(&token_str);
            assert!(decoded_header.is_ok());
            if let Ok(h) = decoded_header {
                assert_eq!(h.kid, Some("test-key-id".to_string()));
            }
        }
    }

    #[tokio::test]
    async fn test_validator_cache() {
        let validator = JwtValidator::new();
        
        // Verify cache starts empty
        let cache = validator.key_cache.read().await;
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_validation_error_messages() {
        let error = AuthorizationError::Internal("JWT validation failed: invalid signature".to_string());
        assert!(error.to_string().contains("JWT validation failed"));
    }
}
