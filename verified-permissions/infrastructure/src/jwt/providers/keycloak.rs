//! Keycloak Identity Provider integration
//!
//! Keycloak is an open-source identity and access management solution.
//! This module provides specific support for Keycloak's JWT token structure.

use crate::error::Result;
use crate::jwt::{ClaimsMappingConfig, ParentMapping, ValidatedClaims, ValueTransform};
use super::IdentityProvider;

/// Keycloak-specific configuration
#[derive(Debug, Clone)]
pub struct KeycloakConfig {
    /// Realm name
    pub realm: Option<String>,
    
    /// Client ID
    pub client_id: Option<String>,
    
    /// Whether to include realm roles
    pub include_realm_roles: bool,
    
    /// Whether to include client roles
    pub include_client_roles: bool,
    
    /// Whether to include groups
    pub include_groups: bool,
    
    /// Transform group paths (e.g., "/engineering" -> "engineering")
    pub transform_group_paths: bool,
}

impl Default for KeycloakConfig {
    fn default() -> Self {
        Self {
            realm: None,
            client_id: None,
            include_realm_roles: true,
            include_client_roles: true,
            include_groups: true,
            transform_group_paths: true,
        }
    }
}

/// Keycloak Identity Provider
#[derive(Debug, Clone)]
pub struct KeycloakProvider {
    config: KeycloakConfig,
}

impl Default for KeycloakProvider {
    fn default() -> Self {
        Self {
            config: KeycloakConfig::default(),
        }
    }
}

impl KeycloakProvider {
    /// Create a new Keycloak provider with custom configuration
    pub fn new(config: KeycloakConfig) -> Self {
        Self { config }
    }
    
    /// Create provider with realm name
    pub fn with_realm(mut self, realm: impl Into<String>) -> Self {
        self.config.realm = Some(realm.into());
        self
    }
    
    /// Create provider with client ID
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.config.client_id = Some(client_id.into());
        self
    }
    
    /// Extract realm name from issuer URL
    pub fn extract_realm_from_issuer(issuer: &str) -> Option<String> {
        // Keycloak issuer format: https://{host}/realms/{realm}
        if let Some(realms_pos) = issuer.find("/realms/") {
            let after_realms = &issuer[realms_pos + 8..];
            let realm = after_realms.split('/').next()?;
            return Some(realm.to_string());
        }
        None
    }
}

impl IdentityProvider for KeycloakProvider {
    fn name(&self) -> &'static str {
        "Keycloak"
    }
    
    fn matches_issuer(&self, issuer: &str) -> bool {
        // Keycloak issuer contains "/realms/"
        issuer.contains("/realms/")
    }
    
    fn create_claims_config(&self) -> ClaimsMappingConfig {
        let mut config = ClaimsMappingConfig::default();
        let mut parent_mappings = Vec::new();
        
        // Realm roles: realm_access.roles
        if self.config.include_realm_roles {
            parent_mappings.push(ParentMapping {
                claim_path: "realm_access.roles".to_string(),
                entity_type: "RealmRole".to_string(),
                transform: ValueTransform::None,
            });
        }
        
        // Client roles: resource_access.{client_id}.roles
        if self.config.include_client_roles {
            if let Some(client_id) = &self.config.client_id {
                parent_mappings.push(ParentMapping {
                    claim_path: format!("resource_access.{}.roles", client_id),
                    entity_type: "ClientRole".to_string(),
                    transform: ValueTransform::None,
                });
            }
        }
        
        // Groups
        if self.config.include_groups {
            let transform = if self.config.transform_group_paths {
                ValueTransform::SplitLast("/".to_string())
            } else {
                ValueTransform::None
            };
            
            parent_mappings.push(ParentMapping {
                claim_path: "groups".to_string(),
                entity_type: "Group".to_string(),
                transform,
            });
        }
        
        config.parent_mappings = parent_mappings;
        
        // Common attributes
        config.attribute_mappings.insert("email".to_string(), "email".to_string());
        config.attribute_mappings.insert("name".to_string(), "name".to_string());
        config.attribute_mappings.insert("preferred_username".to_string(), "preferred_username".to_string());
        config.attribute_mappings.insert("given_name".to_string(), "given_name".to_string());
        config.attribute_mappings.insert("family_name".to_string(), "family_name".to_string());
        
        config
    }
    
    fn validate_claims(&self, claims: &ValidatedClaims) -> Result<()> {
        // Keycloak-specific validation
        // Could check for required Keycloak claims here
        let _ = claims;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_matches_keycloak_issuer() {
        let provider = KeycloakProvider::default();
        
        assert!(provider.matches_issuer("https://keycloak.example.com/realms/myapp"));
        assert!(provider.matches_issuer("http://localhost:8080/realms/test"));
        assert!(!provider.matches_issuer("https://auth0.example.com"));
        assert!(!provider.matches_issuer("https://accounts.google.com"));
    }

    #[test]
    fn test_extract_realm_from_issuer() {
        let issuer = "https://keycloak.example.com/realms/myapp";
        let realm = KeycloakProvider::extract_realm_from_issuer(issuer);
        assert_eq!(realm, Some("myapp".to_string()));
        
        let issuer2 = "http://localhost:8080/realms/test-realm";
        let realm2 = KeycloakProvider::extract_realm_from_issuer(issuer2);
        assert_eq!(realm2, Some("test-realm".to_string()));
    }

    #[test]
    fn test_create_claims_config_default() {
        let provider = KeycloakProvider::default();
        let config = provider.create_claims_config();
        
        // Should have 3 parent mappings by default (realm roles, client roles, groups)
        // But client roles won't be added without client_id
        assert!(config.parent_mappings.len() >= 2);
        
        // Check realm roles mapping exists
        let realm_roles = config.parent_mappings.iter()
            .find(|m| m.claim_path == "realm_access.roles");
        assert!(realm_roles.is_some());
        assert_eq!(realm_roles.unwrap().entity_type, "RealmRole");
        
        // Check groups mapping exists
        let groups = config.parent_mappings.iter()
            .find(|m| m.claim_path == "groups");
        assert!(groups.is_some());
        assert_eq!(groups.unwrap().entity_type, "Group");
        
        // Check attributes
        assert!(config.attribute_mappings.contains_key("email"));
        assert!(config.attribute_mappings.contains_key("name"));
    }

    #[test]
    fn test_create_claims_config_with_client() {
        let provider = KeycloakProvider::default()
            .with_client_id("my-app");
        
        let config = provider.create_claims_config();
        
        // Should have client roles mapping
        let client_roles = config.parent_mappings.iter()
            .find(|m| m.claim_path == "resource_access.my-app.roles");
        assert!(client_roles.is_some());
        assert_eq!(client_roles.unwrap().entity_type, "ClientRole");
    }

    #[test]
    fn test_group_path_transformation() {
        let provider = KeycloakProvider::default();
        let config = provider.create_claims_config();
        
        let groups_mapping = config.parent_mappings.iter()
            .find(|m| m.claim_path == "groups")
            .unwrap();
        
        // Should have SplitLast transform by default
        assert!(matches!(groups_mapping.transform, ValueTransform::SplitLast(_)));
    }

    #[test]
    fn test_keycloak_config_builder() {
        let provider = KeycloakProvider::default()
            .with_realm("myapp")
            .with_client_id("verified-permissions");
        
        assert_eq!(provider.config.realm, Some("myapp".to_string()));
        assert_eq!(provider.config.client_id, Some("verified-permissions".to_string()));
    }

    #[test]
    fn test_validate_claims() {
        let provider = KeycloakProvider::default();
        
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://keycloak.example.com/realms/myapp".to_string(),
            aud: vec!["my-app".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: HashMap::new(),
        };
        
        let result = provider.validate_claims(&claims);
        assert!(result.is_ok());
    }
}
