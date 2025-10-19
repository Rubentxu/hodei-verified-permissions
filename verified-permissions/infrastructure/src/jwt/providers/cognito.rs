//! AWS Cognito Identity Provider integration
//!
//! AWS Cognito is Amazon's identity and access management service.
//! This module provides specific support for Cognito User Pool JWT tokens.

use crate::error::{AuthorizationError, Result};
use crate::jwt::{ClaimsMappingConfig, ParentMapping, ValidatedClaims, ValueTransform};
use super::IdentityProvider;

/// Cognito-specific configuration
#[derive(Debug, Clone)]
pub struct CognitoConfig {
    /// AWS Region
    pub region: Option<String>,
    
    /// User Pool ID
    pub user_pool_id: Option<String>,
    
    /// Client IDs (app clients)
    pub client_ids: Vec<String>,
    
    /// Whether to include Cognito groups
    pub include_groups: bool,
}

impl Default for CognitoConfig {
    fn default() -> Self {
        Self {
            region: None,
            user_pool_id: None,
            client_ids: Vec::new(),
            include_groups: true,
        }
    }
}

/// AWS Cognito Identity Provider
#[derive(Debug, Clone)]
pub struct CognitoProvider {
    config: CognitoConfig,
}

impl Default for CognitoProvider {
    fn default() -> Self {
        Self {
            config: CognitoConfig::default(),
        }
    }
}

impl CognitoProvider {
    /// Create a new Cognito provider with custom configuration
    pub fn new(config: CognitoConfig) -> Self {
        Self { config }
    }
    
    /// Create provider from User Pool ARN
    /// Format: arn:aws:cognito-idp:{region}:{account-id}:userpool/{user-pool-id}
    pub fn from_arn(arn: &str) -> Result<Self> {
        let parts: Vec<&str> = arn.split(':').collect();
        
        if parts.len() < 6 || parts[0] != "arn" || parts[2] != "cognito-idp" {
            return Err(AuthorizationError::Internal(
                "Invalid Cognito User Pool ARN format".to_string()
            ));
        }
        
        let region = parts[3].to_string();
        
        // Extract user pool ID from last part (userpool/{id})
        let pool_part = parts[5];
        let user_pool_id = pool_part
            .strip_prefix("userpool/")
            .ok_or_else(|| {
                AuthorizationError::Internal("Invalid User Pool ID in ARN".to_string())
            })?
            .to_string();
        
        Ok(Self {
            config: CognitoConfig {
                region: Some(region),
                user_pool_id: Some(user_pool_id),
                client_ids: Vec::new(),
                include_groups: true,
            },
        })
    }
    
    /// Create provider with region and pool ID
    pub fn with_pool(region: impl Into<String>, user_pool_id: impl Into<String>) -> Self {
        Self {
            config: CognitoConfig {
                region: Some(region.into()),
                user_pool_id: Some(user_pool_id.into()),
                client_ids: Vec::new(),
                include_groups: true,
            },
        }
    }
    
    /// Add a client ID
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.config.client_ids.push(client_id.into());
        self
    }
    
    /// Extract region and pool ID from issuer URL
    /// Format: https://cognito-idp.{region}.amazonaws.com/{user-pool-id}
    pub fn parse_issuer(issuer: &str) -> Option<(String, String)> {
        if !issuer.contains("cognito-idp") || !issuer.contains("amazonaws.com") {
            return None;
        }
        
        // Extract region from domain
        let domain_start = issuer.find("://")? + 3;
        let domain_end = issuer[domain_start..].find('/')?;
        let domain = &issuer[domain_start..domain_start + domain_end];
        
        // cognito-idp.{region}.amazonaws.com
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 4 || parts[0] != "cognito-idp" {
            return None;
        }
        let region = parts[1].to_string();
        
        // Extract user pool ID from path
        let path_start = domain_start + domain_end + 1;
        let user_pool_id = issuer[path_start..].split('/').next()?.to_string();
        
        Some((region, user_pool_id))
    }
}

impl IdentityProvider for CognitoProvider {
    fn name(&self) -> &'static str {
        "Cognito"
    }
    
    fn matches_issuer(&self, issuer: &str) -> bool {
        // Cognito issuer contains "cognito-idp" and "amazonaws.com"
        issuer.contains("cognito-idp") && issuer.contains("amazonaws.com")
    }
    
    fn create_claims_config(&self) -> ClaimsMappingConfig {
        let mut config = ClaimsMappingConfig::default();
        let mut parent_mappings = Vec::new();
        
        // Cognito groups
        if self.config.include_groups {
            parent_mappings.push(ParentMapping {
                claim_path: "cognito:groups".to_string(),
                entity_type: "Group".to_string(),
                transform: ValueTransform::None,
            });
        }
        
        config.parent_mappings = parent_mappings;
        
        // Common Cognito attributes
        config.attribute_mappings.insert("email".to_string(), "email".to_string());
        config.attribute_mappings.insert("email_verified".to_string(), "email_verified".to_string());
        config.attribute_mappings.insert("phone_number".to_string(), "phone_number".to_string());
        config.attribute_mappings.insert("phone_number_verified".to_string(), "phone_number_verified".to_string());
        
        // Cognito-specific attributes
        config.attribute_mappings.insert("username".to_string(), "cognito:username".to_string());
        
        config
    }
    
    fn validate_claims(&self, claims: &ValidatedClaims) -> Result<()> {
        // Validate client_id if configured
        if !self.config.client_ids.is_empty() {
            let token_client_id = claims.additional_claims
                .get("client_id")
                .and_then(|v| v.as_str());
            
            if let Some(client_id) = token_client_id {
                if !self.config.client_ids.iter().any(|id| id == client_id) {
                    return Err(AuthorizationError::Internal(format!(
                        "Token client_id '{}' not in allowed list",
                        client_id
                    )));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_matches_cognito_issuer() {
        let provider = CognitoProvider::default();
        
        assert!(provider.matches_issuer("https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123"));
        assert!(provider.matches_issuer("https://cognito-idp.eu-west-1.amazonaws.com/eu-west-1_XYZ789"));
        assert!(!provider.matches_issuer("https://keycloak.example.com"));
        assert!(!provider.matches_issuer("https://myinstance.zitadel.cloud"));
    }

    #[test]
    fn test_parse_issuer() {
        let issuer = "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123";
        let result = CognitoProvider::parse_issuer(issuer);
        
        assert!(result.is_some());
        let (region, pool_id) = result.unwrap();
        assert_eq!(region, "us-east-1");
        assert_eq!(pool_id, "us-east-1_ABC123");
    }

    #[test]
    fn test_from_arn() {
        let arn = "arn:aws:cognito-idp:us-east-1:123456789:userpool/us-east-1_ABC123";
        let provider = CognitoProvider::from_arn(arn).unwrap();
        
        assert_eq!(provider.config.region, Some("us-east-1".to_string()));
        assert_eq!(provider.config.user_pool_id, Some("us-east-1_ABC123".to_string()));
    }

    #[test]
    fn test_from_arn_invalid() {
        let invalid_arn = "arn:aws:s3:::my-bucket";
        let result = CognitoProvider::from_arn(invalid_arn);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_pool() {
        let provider = CognitoProvider::with_pool("us-west-2", "us-west-2_XYZ789");
        
        assert_eq!(provider.config.region, Some("us-west-2".to_string()));
        assert_eq!(provider.config.user_pool_id, Some("us-west-2_XYZ789".to_string()));
    }

    #[test]
    fn test_with_client_id() {
        let provider = CognitoProvider::default()
            .with_client_id("app-client-1")
            .with_client_id("app-client-2");
        
        assert_eq!(provider.config.client_ids.len(), 2);
        assert!(provider.config.client_ids.contains(&"app-client-1".to_string()));
        assert!(provider.config.client_ids.contains(&"app-client-2".to_string()));
    }

    #[test]
    fn test_create_claims_config() {
        let provider = CognitoProvider::default();
        let config = provider.create_claims_config();
        
        // Should have groups mapping
        assert!(!config.parent_mappings.is_empty());
        
        let groups = config.parent_mappings.iter()
            .find(|m| m.claim_path == "cognito:groups");
        assert!(groups.is_some());
        assert_eq!(groups.unwrap().entity_type, "Group");
        
        // Check attributes
        assert!(config.attribute_mappings.contains_key("email"));
        assert!(config.attribute_mappings.contains_key("username"));
        assert_eq!(config.attribute_mappings.get("username").unwrap(), "cognito:username");
    }

    #[test]
    fn test_validate_claims_no_client_ids() {
        let provider = CognitoProvider::default();
        
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123".to_string(),
            aud: vec!["app-client".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: HashMap::new(),
        };
        
        let result = provider.validate_claims(&claims);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_claims_with_client_id_valid() {
        let provider = CognitoProvider::default()
            .with_client_id("allowed-client");
        
        let mut additional = HashMap::new();
        additional.insert("client_id".to_string(), serde_json::json!("allowed-client"));
        
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123".to_string(),
            aud: vec!["app-client".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: additional,
        };
        
        let result = provider.validate_claims(&claims);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_claims_with_client_id_invalid() {
        let provider = CognitoProvider::default()
            .with_client_id("allowed-client");
        
        let mut additional = HashMap::new();
        additional.insert("client_id".to_string(), serde_json::json!("unauthorized-client"));
        
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123".to_string(),
            aud: vec!["app-client".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: additional,
        };
        
        let result = provider.validate_claims(&claims);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not in allowed list"));
    }
}
