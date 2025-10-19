//! JWT token generation helpers for testing

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Test JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TestClaims {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    #[serde(flatten)]
    pub additional: serde_json::Map<String, serde_json::Value>,
}

impl TestClaims {
    /// Create basic test claims
    pub fn basic(sub: &str, issuer: &str, audience: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            sub: sub.to_string(),
            iss: issuer.to_string(),
            aud: vec![audience.to_string()],
            exp: now + 3600, // 1 hour from now
            iat: now,
            additional: serde_json::Map::new(),
        }
    }

    /// Add a custom claim
    pub fn with_claim(mut self, key: &str, value: serde_json::Value) -> Self {
        self.additional.insert(key.to_string(), value);
        self
    }

    /// Add email claim
    pub fn with_email(self, email: &str) -> Self {
        self.with_claim("email", serde_json::json!(email))
    }

    /// Add groups claim
    pub fn with_groups(self, groups: Vec<&str>) -> Self {
        let groups_json: Vec<String> = groups.iter().map(|g| g.to_string()).collect();
        self.with_claim("groups", serde_json::json!(groups_json))
    }

    /// Add Keycloak realm roles
    pub fn with_keycloak_realm_roles(self, roles: Vec<&str>) -> Self {
        let roles_json: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        self.with_claim(
            "realm_access",
            serde_json::json!({
                "roles": roles_json
            }),
        )
    }

    /// Add Keycloak client roles
    pub fn with_keycloak_client_roles(self, client: &str, roles: Vec<&str>) -> Self {
        let roles_json: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        let mut resource_access = self
            .additional
            .get("resource_access")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        resource_access.insert(
            client.to_string(),
            serde_json::json!({
                "roles": roles_json
            }),
        );

        self.with_claim("resource_access", serde_json::json!(resource_access))
    }

    /// Set expiration to past (expired token)
    pub fn expired(mut self) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.exp = now - 3600; // 1 hour ago
        self
    }
}

/// Generate a test JWT token
pub fn generate_test_token(claims: TestClaims, secret: &[u8]) -> String {
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some("test-key-id".to_string());

    encode(&header, &claims, &EncodingKey::from_secret(secret))
        .expect("Failed to encode JWT")
}

/// Generate an RSA test token (for more realistic testing)
pub fn generate_rsa_test_token(claims: TestClaims, private_key_pem: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-rsa-key".to_string());

    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .expect("Failed to create RSA encoding key");

    encode(&header, &claims, &encoding_key).expect("Failed to encode RSA JWT")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_claims() {
        let claims = TestClaims::basic("user123", "https://issuer.example.com", "test-client");
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.iss, "https://issuer.example.com");
        assert_eq!(claims.aud, vec!["test-client"]);
    }

    #[test]
    fn test_with_email() {
        let claims = TestClaims::basic("user123", "https://issuer.example.com", "test-client")
            .with_email("user@example.com");

        assert_eq!(
            claims.additional.get("email").unwrap().as_str().unwrap(),
            "user@example.com"
        );
    }

    #[test]
    fn test_with_groups() {
        let claims = TestClaims::basic("user123", "https://issuer.example.com", "test-client")
            .with_groups(vec!["admins", "developers"]);

        let groups = claims.additional.get("groups").unwrap().as_array().unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_with_keycloak_realm_roles() {
        let claims = TestClaims::basic("user123", "https://issuer.example.com", "test-client")
            .with_keycloak_realm_roles(vec!["admin", "user"]);

        let realm_access = claims
            .additional
            .get("realm_access")
            .unwrap()
            .as_object()
            .unwrap();
        let roles = realm_access.get("roles").unwrap().as_array().unwrap();
        assert_eq!(roles.len(), 2);
    }

    #[test]
    fn test_generate_token() {
        let claims = TestClaims::basic("user123", "https://issuer.example.com", "test-client");
        let token = generate_test_token(claims, b"secret");
        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }

    #[test]
    fn test_expired_token() {
        let claims = TestClaims::basic("user123", "https://issuer.example.com", "test-client")
            .expired();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(claims.exp < now);
    }
}
