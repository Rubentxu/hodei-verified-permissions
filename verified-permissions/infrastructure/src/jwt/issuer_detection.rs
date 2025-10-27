//! JWT issuer detection without signature validation
//!
//! This module provides utilities to extract the issuer (iss claim) from a JWT
//! without validating the signature. This is useful for selecting the correct
//! identity source when multiple issuers are configured.

use crate::error::{AuthorizationError, Result};
use serde_json::Value;

/// Extract issuer from JWT without validating signature
///
/// # Arguments
///
/// * `token` - The JWT token string
///
/// # Returns
///
/// The issuer (iss claim) value or an error if extraction fails
pub fn extract_issuer_from_token(token: &str) -> Result<String> {
    let parts: Vec<&str> = token.split('.').collect();
    
    if parts.len() != 3 {
        return Err(AuthorizationError::InvalidArgument(
            "Invalid JWT format: expected 3 parts separated by dots".to_string(),
        ));
    }

    // Decode the payload (second part)
    let payload_part = parts[1];
    let payload = decode_base64_url(payload_part)?;
    
    // Parse as JSON
    let claims: Value = serde_json::from_slice(&payload)
        .map_err(|e| AuthorizationError::Serialization(e))?;

    // Extract issuer claim
    claims["iss"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AuthorizationError::InvalidArgument(
            "Missing 'iss' (issuer) claim in JWT".to_string(),
        ))
}

/// Extract subject from JWT without validating signature
pub fn extract_subject_from_token(token: &str) -> Result<String> {
    let parts: Vec<&str> = token.split('.').collect();
    
    if parts.len() != 3 {
        return Err(AuthorizationError::InvalidArgument(
            "Invalid JWT format: expected 3 parts separated by dots".to_string(),
        ));
    }

    let payload_part = parts[1];
    let payload = decode_base64_url(payload_part)?;
    
    let claims: Value = serde_json::from_slice(&payload)
        .map_err(|e| AuthorizationError::Serialization(e))?;

    claims["sub"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AuthorizationError::InvalidArgument(
            "Missing 'sub' (subject) claim in JWT".to_string(),
        ))
}

/// Extract all claims from JWT without validating signature
pub fn extract_claims_from_token(token: &str) -> Result<Value> {
    let parts: Vec<&str> = token.split('.').collect();
    
    if parts.len() != 3 {
        return Err(AuthorizationError::InvalidArgument(
            "Invalid JWT format: expected 3 parts separated by dots".to_string(),
        ));
    }

    let payload_part = parts[1];
    let payload = decode_base64_url(payload_part)?;
    
    serde_json::from_slice(&payload)
        .map_err(|e| AuthorizationError::Serialization(e))
}

/// Decode base64url encoded string
fn decode_base64_url(encoded: &str) -> Result<Vec<u8>> {
    // Add padding if needed
    let padding = match encoded.len() % 4 {
        0 => 0,
        2 => 2,
        3 => 1,
        _ => return Err(AuthorizationError::InvalidArgument(
            "Invalid base64url encoding".to_string(),
        )),
    };

    let padded = if padding > 0 {
        format!("{}{}", encoded, "=".repeat(padding))
    } else {
        encoded.to_string()
    };

    // Replace URL-safe characters
    let standard = padded
        .replace('-', "+")
        .replace('_', "/");

    base64::decode(&standard)
        .map_err(|e| AuthorizationError::InvalidArgument(
            format!("Failed to decode base64url: {}", e),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example JWT token (not a real one, just for testing structure)
    // Header: {"alg":"RS256","typ":"JWT"}
    // Payload: {"iss":"https://issuer.example.com","sub":"user123","aud":"client-id"}
    // This is a valid base64url encoded payload
    const VALID_JWT_PAYLOAD: &str = "eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlLmNvbSIsInN1YiI6InVzZXIxMjMiLCJhdWQiOiJjbGllbnQtaWQifQ";

    #[test]
    fn test_decode_base64_url_with_padding() {
        let encoded = "SGVsbG8gV29ybGQ";
        let result = decode_base64_url(encoded).unwrap();
        assert_eq!(result, b"Hello World");
    }

    #[test]
    fn test_decode_base64_url_with_url_safe_chars() {
        // Test with - and _ characters
        let encoded = "Pj8-Pz8";
        let result = decode_base64_url(encoded).unwrap();
        assert_eq!(result, vec![62, 63, 62, 63, 63]);
    }

    #[test]
    fn test_extract_issuer_valid() {
        let token = format!("header.{}.signature", VALID_JWT_PAYLOAD);
        let issuer = extract_issuer_from_token(&token).unwrap();
        assert_eq!(issuer, "https://issuer.example.com");
    }

    #[test]
    fn test_extract_subject_valid() {
        let token = format!("header.{}.signature", VALID_JWT_PAYLOAD);
        let subject = extract_subject_from_token(&token).unwrap();
        assert_eq!(subject, "user123");
    }

    #[test]
    fn test_extract_claims_valid() {
        let token = format!("header.{}.signature", VALID_JWT_PAYLOAD);
        let claims = extract_claims_from_token(&token).unwrap();
        assert_eq!(claims["iss"].as_str().unwrap(), "https://issuer.example.com");
        assert_eq!(claims["sub"].as_str().unwrap(), "user123");
        assert_eq!(claims["aud"].as_str().unwrap(), "client-id");
    }

    #[test]
    fn test_extract_issuer_invalid_format() {
        let token = "invalid.token";
        assert!(extract_issuer_from_token(token).is_err());
    }

    #[test]
    fn test_extract_issuer_missing_iss_claim() {
        // Payload without iss claim: {"sub":"user123"}
        let payload = "eyJzdWIiOiJ1c2VyMTIzIn0";
        let token = format!("header.{}.signature", payload);
        assert!(extract_issuer_from_token(&token).is_err());
    }
}
