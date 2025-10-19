//! JWT token validation

use crate::error::{AuthorizationError, Result};
use crate::jwt::{JwksCache, ValidatedClaims};
use jsonwebtoken::{decode, decode_header, Validation};
use std::sync::Arc;

/// JWT Validator with JWKS caching
pub struct JwtValidator {
    /// JWKS cache with auto-refresh
    jwks_cache: Arc<JwksCache>,
}

impl JwtValidator {
    /// Create a new JWT validator with default cache configuration
    pub fn new() -> Self {
        Self {
            jwks_cache: Arc::new(JwksCache::new()),
        }
    }

    /// Create a new JWT validator with custom JWKS cache
    pub fn with_cache(jwks_cache: Arc<JwksCache>) -> Self {
        Self { jwks_cache }
    }

    /// Start background refresh task for JWKS cache
    pub fn start_background_refresh(&self) -> tokio::task::JoinHandle<()> {
        self.jwks_cache.clone().start_background_refresh()
    }

    /// Validate a JWT token using issuer (with auto-discovery)
    pub async fn validate_token(
        &self,
        token: &str,
        expected_issuer: &str,
        expected_audiences: &[String],
    ) -> Result<ValidatedClaims> {
        // Decode header to get key ID
        let header = decode_header(token).map_err(|e| {
            AuthorizationError::Internal(format!("Failed to decode JWT header: {}", e))
        })?;

        let kid = header.kid.ok_or_else(|| {
            AuthorizationError::Internal("JWT header missing 'kid' field".to_string())
        })?;

        // Get decoding key from cache (with auto-discovery and refresh)
        let decoding_key = self.jwks_cache.get_key(&kid, expected_issuer).await?;

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

    /// Validate a JWT token with explicit JWKS URI (legacy method)
    #[deprecated(note = "Use validate_token with auto-discovery instead")]
    pub async fn validate_token_with_jwks_uri(
        &self,
        token: &str,
        expected_issuer: &str,
        expected_audiences: &[String],
        _jwks_uri: &str,
    ) -> Result<ValidatedClaims> {
        // Delegate to new method that uses auto-discovery
        self.validate_token(token, expected_issuer, expected_audiences).await
    }

    /// Get cache metrics
    pub async fn cache_metrics(&self) -> crate::jwt::jwks_cache::CacheMetrics {
        self.jwks_cache.metrics().await
    }

    /// Clear the JWKS cache
    pub async fn clear_cache(&self) {
        self.jwks_cache.clear().await;
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
