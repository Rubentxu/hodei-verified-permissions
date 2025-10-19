//! Zitadel Identity Provider integration
//!
//! Zitadel is a modern, cloud-native identity and access management solution.
//! This module provides specific support for Zitadel's JWT token structure.

use crate::error::Result;
use crate::jwt::{ClaimsMappingConfig, ParentMapping, ValidatedClaims, ValueTransform};
use super::IdentityProvider;
use std::collections::HashMap;

/// Zitadel-specific configuration
#[derive(Debug, Clone)]
pub struct ZitadelConfig {
    /// Project ID
    pub project_id: Option<String>,
    
    /// Organization ID
    pub organization_id: Option<String>,
    
    /// Whether to include project roles
    pub include_project_roles: bool,
    
    /// Custom roles claim URN
    pub roles_claim: Option<String>,
    
    /// Custom organization claim URN
    pub organization_claim: Option<String>,
}

impl Default for ZitadelConfig {
    fn default() -> Self {
        Self {
            project_id: None,
            organization_id: None,
            include_project_roles: true,
            roles_claim: Some("urn:zitadel:iam:org:project:roles".to_string()),
            organization_claim: Some("urn:zitadel:iam:user:resourceowner:name".to_string()),
        }
    }
}

/// Zitadel Identity Provider
#[derive(Debug, Clone)]
pub struct ZitadelProvider {
    config: ZitadelConfig,
}

impl Default for ZitadelProvider {
    fn default() -> Self {
        Self {
            config: ZitadelConfig::default(),
        }
    }
}

impl ZitadelProvider {
    /// Create a new Zitadel provider with custom configuration
    pub fn new(config: ZitadelConfig) -> Self {
        Self { config }
    }
    
    /// Create provider with project ID
    pub fn with_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.config.project_id = Some(project_id.into());
        self
    }
    
    /// Create provider with organization ID
    pub fn with_organization_id(mut self, organization_id: impl Into<String>) -> Self {
        self.config.organization_id = Some(organization_id.into());
        self
    }
    
    /// Extract instance name from issuer URL
    pub fn extract_instance_from_issuer(issuer: &str) -> Option<String> {
        // Zitadel issuer formats:
        // - https://{instance}.zitadel.cloud
        // - https://zitadel.{domain}.com
        // - https://{custom-domain}
        
        if let Some(domain_start) = issuer.find("://") {
            let after_protocol = &issuer[domain_start + 3..];
            let domain = after_protocol.split('/').next()?;
            
            // Check for .zitadel.cloud pattern
            if domain.contains(".zitadel.cloud") {
                let instance = domain.split('.').next()?;
                return Some(instance.to_string());
            }
            
            // Return full domain for custom domains
            return Some(domain.to_string());
        }
        
        None
    }
    
    /// Parse Zitadel project roles from claims
    /// Zitadel roles format: { "role_name": { "project_id": "org_id" } }
    pub fn parse_project_roles(roles_claim: &serde_json::Value, project_id: Option<&str>) -> Vec<String> {
        let mut roles = Vec::new();
        
        if let Some(roles_obj) = roles_claim.as_object() {
            for (role_name, role_data) in roles_obj {
                // If project_id is specified, filter by it
                if let Some(pid) = project_id {
                    if let Some(role_obj) = role_data.as_object() {
                        if role_obj.contains_key(pid) {
                            roles.push(role_name.clone());
                        }
                    }
                } else {
                    // No filter, include all roles
                    roles.push(role_name.clone());
                }
            }
        }
        
        roles
    }
}

impl IdentityProvider for ZitadelProvider {
    fn name(&self) -> &'static str {
        "Zitadel"
    }
    
    fn matches_issuer(&self, issuer: &str) -> bool {
        // Zitadel issuer contains "zitadel" in domain
        issuer.contains("zitadel")
    }
    
    fn create_claims_config(&self) -> ClaimsMappingConfig {
        let mut config = ClaimsMappingConfig::default();
        let mut parent_mappings = Vec::new();
        
        // Project roles using URN claim
        if self.config.include_project_roles {
            if let Some(roles_claim) = &self.config.roles_claim {
                parent_mappings.push(ParentMapping {
                    claim_path: roles_claim.clone(),
                    entity_type: "ProjectRole".to_string(),
                    transform: ValueTransform::None,
                });
            }
        }
        
        config.parent_mappings = parent_mappings;
        
        // Common attributes
        config.attribute_mappings.insert("email".to_string(), "email".to_string());
        config.attribute_mappings.insert("email_verified".to_string(), "email_verified".to_string());
        config.attribute_mappings.insert("name".to_string(), "name".to_string());
        config.attribute_mappings.insert("preferred_username".to_string(), "preferred_username".to_string());
        config.attribute_mappings.insert("given_name".to_string(), "given_name".to_string());
        config.attribute_mappings.insert("family_name".to_string(), "family_name".to_string());
        
        // Zitadel-specific attributes
        if let Some(org_claim) = &self.config.organization_claim {
            config.attribute_mappings.insert("organization".to_string(), org_claim.clone());
        }
        
        config
    }
    
    fn validate_claims(&self, claims: &ValidatedClaims) -> Result<()> {
        // Zitadel-specific validation
        // Could check for required Zitadel claims here
        let _ = claims;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_matches_zitadel_issuer() {
        let provider = ZitadelProvider::default();
        
        assert!(provider.matches_issuer("https://myinstance.zitadel.cloud"));
        assert!(provider.matches_issuer("https://zitadel.example.com"));
        assert!(provider.matches_issuer("https://custom.zitadel.io"));
        assert!(!provider.matches_issuer("https://keycloak.example.com"));
        assert!(!provider.matches_issuer("https://auth0.example.com"));
    }

    #[test]
    fn test_extract_instance_from_issuer() {
        let issuer = "https://myinstance.zitadel.cloud";
        let instance = ZitadelProvider::extract_instance_from_issuer(issuer);
        assert_eq!(instance, Some("myinstance".to_string()));
        
        let issuer2 = "https://custom.example.com";
        let instance2 = ZitadelProvider::extract_instance_from_issuer(issuer2);
        assert_eq!(instance2, Some("custom.example.com".to_string()));
    }

    #[test]
    fn test_parse_project_roles_no_filter() {
        let roles_claim = json!({
            "developer": { "123456": "myorg" },
            "admin": { "123456": "myorg" },
            "viewer": { "789012": "otherorg" }
        });
        
        let roles = ZitadelProvider::parse_project_roles(&roles_claim, None);
        assert_eq!(roles.len(), 3);
        assert!(roles.contains(&"developer".to_string()));
        assert!(roles.contains(&"admin".to_string()));
        assert!(roles.contains(&"viewer".to_string()));
    }

    #[test]
    fn test_parse_project_roles_with_filter() {
        let roles_claim = json!({
            "developer": { "123456": "myorg" },
            "admin": { "123456": "myorg" },
            "viewer": { "789012": "otherorg" }
        });
        
        let roles = ZitadelProvider::parse_project_roles(&roles_claim, Some("123456"));
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&"developer".to_string()));
        assert!(roles.contains(&"admin".to_string()));
        assert!(!roles.contains(&"viewer".to_string()));
    }

    #[test]
    fn test_create_claims_config_default() {
        let provider = ZitadelProvider::default();
        let config = provider.create_claims_config();
        
        // Should have project roles mapping
        assert!(!config.parent_mappings.is_empty());
        
        let project_roles = config.parent_mappings.iter()
            .find(|m| m.claim_path == "urn:zitadel:iam:org:project:roles");
        assert!(project_roles.is_some());
        assert_eq!(project_roles.unwrap().entity_type, "ProjectRole");
        
        // Check attributes
        assert!(config.attribute_mappings.contains_key("email"));
        assert!(config.attribute_mappings.contains_key("organization"));
    }

    #[test]
    fn test_create_claims_config_with_project() {
        let provider = ZitadelProvider::default()
            .with_project_id("123456")
            .with_organization_id("myorg");
        
        assert_eq!(provider.config.project_id, Some("123456".to_string()));
        assert_eq!(provider.config.organization_id, Some("myorg".to_string()));
    }

    #[test]
    fn test_zitadel_config_builder() {
        let provider = ZitadelProvider::default()
            .with_project_id("project-123")
            .with_organization_id("org-456");
        
        assert_eq!(provider.config.project_id, Some("project-123".to_string()));
        assert_eq!(provider.config.organization_id, Some("org-456".to_string()));
    }

    #[test]
    fn test_validate_claims() {
        let provider = ZitadelProvider::default();
        
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://myinstance.zitadel.cloud".to_string(),
            aud: vec!["project-client".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: HashMap::new(),
        };
        
        let result = provider.validate_claims(&claims);
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_roles_claim() {
        let mut config = ZitadelConfig::default();
        config.roles_claim = Some("custom:roles".to_string());
        
        let provider = ZitadelProvider::new(config);
        let claims_config = provider.create_claims_config();
        
        let roles_mapping = claims_config.parent_mappings.iter()
            .find(|m| m.claim_path == "custom:roles");
        assert!(roles_mapping.is_some());
    }
}
