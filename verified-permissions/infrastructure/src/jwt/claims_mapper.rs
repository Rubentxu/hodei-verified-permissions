//! Claims mapping to Cedar entities

use crate::error::{AuthorizationError, Result};
use crate::jwt::{ValidatedClaims, ValueTransform};
use std::collections::HashMap;

/// Simplified entity identifier (avoids proto dependency)
#[derive(Debug, Clone)]
pub struct EntityId {
    pub entity_type: String,
    pub entity_id: String,
}

/// Simplified entity representation (avoids proto dependency)
#[derive(Debug, Clone)]
pub struct EntityData {
    pub identifier: Option<EntityId>,
    pub attributes: HashMap<String, String>,
    pub parents: Vec<EntityId>,
}

/// Mapping of a JWT claim to a parent entity
#[derive(Debug, Clone)]
pub struct ParentMapping {
    /// Path to the claim in the JWT (e.g., "groups", "cognito:groups")
    pub claim_path: String,
    
    /// Entity type for the parent (e.g., "Group", "Role")
    pub entity_type: String,
    
    /// Optional transformation to apply to the claim value
    pub transform: ValueTransform,
}

/// Configuration for mapping JWT claims to Cedar entities
#[derive(Debug, Clone)]
pub struct ClaimsMappingConfig {
    /// Claim to use as principal entity ID (default: "sub")
    pub principal_id_claim: String,
    
    /// Claim containing group membership (deprecated, use parent_mappings instead)
    pub group_claim: Option<String>,
    
    /// Map of Cedar attribute names to JWT claim names
    pub attribute_mappings: HashMap<String, String>,
    
    /// Mappings for parent entities (groups, roles, etc.)
    pub parent_mappings: Vec<ParentMapping>,
}

impl Default for ClaimsMappingConfig {
    fn default() -> Self {
        Self {
            principal_id_claim: "sub".to_string(),
            group_claim: Some("groups".to_string()),
            attribute_mappings: HashMap::new(),
            parent_mappings: Vec::new(),
        }
    }
}

/// Claims mapper for converting JWT claims to Cedar entities
pub struct ClaimsMapper;

impl ClaimsMapper {
    /// Map JWT claims to a Cedar principal entity
    pub fn map_to_principal(
        claims: &ValidatedClaims,
        config: &ClaimsMappingConfig,
        principal_type: &str,
    ) -> Result<(EntityId, Vec<EntityData>)> {
        // Extract principal ID
        let principal_id = if config.principal_id_claim == "sub" {
            claims.sub.clone()
        } else {
            claims
                .additional_claims
                .get(&config.principal_id_claim)
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    AuthorizationError::Internal(format!(
                        "Principal ID claim '{}' not found in token",
                        config.principal_id_claim
                    ))
                })?
                .to_string()
        };

        let principal = EntityId {
            entity_type: principal_type.to_string(),
            entity_id: principal_id.clone(),
        };

        // Build principal entity with attributes
        let mut attributes = HashMap::new();
        
        // Map configured attributes
        for (cedar_attr, claim_name) in &config.attribute_mappings {
            if let Some(value) = claims.additional_claims.get(claim_name) {
                // Convert to JSON string for Cedar
                let attr_value = serde_json::to_string(value)
                    .unwrap_or_else(|_| format!("\"{}\"", value));
                attributes.insert(cedar_attr.clone(), attr_value);
            }
        }

        // Extract groups as parent entities
        let mut parents = Vec::new();
        if let Some(group_claim) = &config.group_claim {
            if let Some(groups_value) = claims.additional_claims.get(group_claim) {
                if let Some(groups_array) = groups_value.as_array() {
                    for group in groups_array {
                        if let Some(group_str) = group.as_str() {
                            parents.push(EntityId {
                                entity_type: "Role".to_string(), // or "Group"
                                entity_id: group_str.to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Create principal entity
        let principal_entity = EntityData {
            identifier: Some(principal.clone()),
            attributes,
            parents,
        };

        Ok((principal, vec![principal_entity]))
    }

    /// Extract all entities from claims (principal + any referenced entities)
    pub fn extract_entities(
        claims: &ValidatedClaims,
        config: &ClaimsMappingConfig,
        principal_type: &str,
    ) -> Result<Vec<EntityData>> {
        let (_, entities) = Self::map_to_principal(claims, config, principal_type)?;
        Ok(entities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_claims() -> ValidatedClaims {
        let mut additional = HashMap::new();
        additional.insert("email".to_string(), json!("user@example.com"));
        additional.insert("department".to_string(), json!("engineering"));
        additional.insert("groups".to_string(), json!(["admins", "developers"]));

        ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["client-id".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: additional,
        }
    }

    #[test]
    fn test_default_config() {
        let config = ClaimsMappingConfig::default();
        assert_eq!(config.principal_id_claim, "sub");
        assert_eq!(config.group_claim, Some("groups".to_string()));
        assert!(config.attribute_mappings.is_empty());
    }

    #[test]
    fn test_map_to_principal_with_sub() {
        let claims = create_test_claims();
        let config = ClaimsMappingConfig::default();

        let result = ClaimsMapper::map_to_principal(&claims, &config, "User");
        assert!(result.is_ok());

        let (principal, entities) = result.unwrap();
        assert_eq!(principal.entity_type, "User");
        assert_eq!(principal.entity_id, "user123");
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn test_map_groups_to_parents() {
        let claims = create_test_claims();
        let config = ClaimsMappingConfig::default();

        let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
        
        assert_eq!(entities.len(), 1);
        let entity = &entities[0];
        assert_eq!(entity.parents.len(), 2);
        
        let parent_ids: Vec<&str> = entity.parents.iter()
            .map(|p| p.entity_id.as_str())
            .collect();
        assert!(parent_ids.contains(&"admins"));
        assert!(parent_ids.contains(&"developers"));
    }

    #[test]
    fn test_map_custom_attributes() {
        let claims = create_test_claims();
        let mut config = ClaimsMappingConfig::default();
        config.attribute_mappings.insert("dept".to_string(), "department".to_string());
        config.attribute_mappings.insert("email".to_string(), "email".to_string());

        let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
        
        let entity = &entities[0];
        assert!(entity.attributes.contains_key("dept"));
        assert!(entity.attributes.contains_key("email"));
        assert!(entity.attributes.get("email").unwrap().contains("user@example.com"));
    }

    #[test]
    fn test_custom_principal_id_claim() {
        let mut additional = HashMap::new();
        additional.insert("custom_id".to_string(), json!("custom123"));
        
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["client-id".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: additional,
        };

        let mut config = ClaimsMappingConfig::default();
        config.principal_id_claim = "custom_id".to_string();

        let (principal, _) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
        assert_eq!(principal.entity_id, "custom123");
    }

    #[test]
    fn test_missing_principal_id_claim() {
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["client-id".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: HashMap::new(),
        };

        let mut config = ClaimsMappingConfig::default();
        config.principal_id_claim = "nonexistent".to_string();

        let result = ClaimsMapper::map_to_principal(&claims, &config, "User");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found in token"));
    }

    #[test]
    fn test_extract_entities() {
        let claims = create_test_claims();
        let config = ClaimsMappingConfig::default();

        let entities = ClaimsMapper::extract_entities(&claims, &config, "User").unwrap();
        assert_eq!(entities.len(), 1);
        assert!(entities[0].identifier.is_some());
    }

    #[test]
    fn test_no_groups_claim() {
        let claims = ValidatedClaims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["client-id".to_string()],
            exp: 9999999999,
            iat: 1234567890,
            additional_claims: HashMap::new(),
        };

        let config = ClaimsMappingConfig::default();
        let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
        
        assert_eq!(entities[0].parents.len(), 0);
    }
}
