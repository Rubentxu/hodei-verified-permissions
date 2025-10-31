//! JWT validation and claims extraction module
//!
//! This module provides functionality for validating JWT tokens and extracting claims
//! for use in authorization decisions.

pub mod validator;
pub mod transforms;
pub mod jwks_cache;
pub mod claims_mapper;
pub mod providers;
pub mod issuer_detection;

pub use validator::JwtValidator;
pub use transforms::ValueTransform;
pub use jwks_cache::JwksCache;
pub use issuer_detection::{extract_issuer_from_token, extract_subject_from_token, extract_claims_from_token};
pub use claims_mapper::{ClaimsMappingConfig, ClaimsMapper, ParentMapping};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validated JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedClaims {
    /// Subject (user ID)
    pub sub: String,
    
    /// Issuer
    pub iss: String,
    
    /// Audience
    pub aud: Vec<String>,
    
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    
    /// Issued at (Unix timestamp)
    pub iat: i64,
    
    /// All other claims
    #[serde(flatten)]
    pub additional_claims: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_claims_serialization() {
        let mut additional = HashMap::new();
        additional.insert("email".to_string(), serde_json::json!("user@example.com"));
        
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["client-id".to_string()],
            exp: 1234567890,
            iat: 1234567800,
            additional_claims: additional,
        };

        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("user123"));
        assert!(json.contains("user@example.com"));
    }
}
