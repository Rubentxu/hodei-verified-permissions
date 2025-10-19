//! Claims mapping to Cedar entities

use crate::error::{AuthorizationError, Result};
use crate::jwt::ValidatedClaims;
use crate::proto::{Entity, EntityIdentifier};
use serde_json::Value;
use std::collections::HashMap;

/// Value transformation types
#[derive(Debug, Clone, PartialEq)]
pub enum ValueTransform {
    /// No transformation
    None,
    /// Convert to uppercase
    Uppercase,
    /// Convert to lowercase
    Lowercase,
    /// Split string by delimiter and take first part
    SplitFirst(String),
    /// Split string by delimiter and take last part
    SplitLast(String),
}

/// Parent entity mapping configuration
#[derive(Debug, Clone)]
pub struct ParentMapping {
    /// Claim path (supports dot notation)
    pub claim_path: String,
    /// Entity type for parents (e.g., "Role", "Group")
    pub entity_type: String,
    /// Optional transformation
    pub transform: ValueTransform,
}

/// Configuration for mapping JWT claims to Cedar entities
#[derive(Debug, Clone)]
pub struct ClaimsMappingConfig {
    /// Claim to use as principal entity ID (default: "sub")
    pub principal_id_claim: String,
    
    /// Claim containing group membership (legacy, use parent_mappings instead)
    #[deprecated(note = "Use parent_mappings for more flexibility")]
    pub group_claim: Option<String>,
    
    /// Map of Cedar attribute names to JWT claim paths (supports dot notation)
    pub attribute_mappings: HashMap<String, String>,
    
    /// Parent entity mappings (roles, groups, etc.)
    pub parent_mappings: Vec<ParentMapping>,
    
    /// Required claims that must be present
    pub required_claims: Vec<String>,
    
    /// Attribute transformations
    pub attribute_transforms: HashMap<String, ValueTransform>,
}

impl Default for ClaimsMappingConfig {
    fn default() -> Self {
        Self {
            principal_id_claim: "sub".to_string(),
            group_claim: Some("groups".to_string()),
            attribute_mappings: HashMap::new(),
            parent_mappings: vec![
                ParentMapping {
                    claim_path: "groups".to_string(),
                    entity_type: "Role".to_string(),
                    transform: ValueTransform::None,
                }
            ],
            required_claims: vec![],
            attribute_transforms: HashMap::new(),
        }
    }
}

/// Claims mapper for converting JWT claims to Cedar entities
pub struct ClaimsMapper;

impl ClaimsMapper {
    /// Get nested claim value using dot notation (e.g., "realm_access.roles")
    fn get_nested_claim(claims: &ValidatedClaims, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        
        if parts.is_empty() {
            return None;
        }
        
        // Start with additional_claims
        let mut current = claims.additional_claims.get(parts[0])?.clone();
        
        // Navigate through nested objects
        for part in &parts[1..] {
            current = current.get(part)?.clone();
        }
        
        Some(current)
    }
    
    /// Apply transformation to a string value
    fn apply_transform(value: &str, transform: &ValueTransform) -> String {
        match transform {
            ValueTransform::None => value.to_string(),
            ValueTransform::Uppercase => value.to_uppercase(),
            ValueTransform::Lowercase => value.to_lowercase(),
            ValueTransform::SplitFirst(delimiter) => {
                value.split(delimiter).next().unwrap_or(value).to_string()
            }
            ValueTransform::SplitLast(delimiter) => {
                value.split(delimiter).last().unwrap_or(value).to_string()
            }
        }
    }
    
    /// Validate that all required claims are present
    fn validate_required_claims(
        claims: &ValidatedClaims,
        required: &[String],
    ) -> Result<()> {
        for claim_path in required {
            if Self::get_nested_claim(claims, claim_path).is_none() {
                return Err(AuthorizationError::Internal(format!(
                    "Required claim '{}' not found in token",
                    claim_path
                )));
            }
        }
        Ok(())
    }

    /// Map JWT claims to a Cedar principal entity
    pub fn map_to_principal(
        claims: &ValidatedClaims,
        config: &ClaimsMappingConfig,
        principal_type: &str,
    ) -> Result<(EntityIdentifier, Vec<Entity>)> {
        // Validate required claims
        Self::validate_required_claims(claims, &config.required_claims)?;
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

        let principal = EntityIdentifier {
            entity_type: principal_type.to_string(),
            entity_id: principal_id.clone(),
        };

        // Build principal entity with attributes
        let mut attributes = HashMap::new();
        
        // Map configured attributes (with dot notation support)
        for (cedar_attr, claim_path) in &config.attribute_mappings {
            if let Some(value) = Self::get_nested_claim(claims, claim_path) {
                // Apply transformation if configured
                let transformed_value = if let Some(transform) = config.attribute_transforms.get(cedar_attr) {
                    if let Some(str_val) = value.as_str() {
                        Value::String(Self::apply_transform(str_val, transform))
                    } else {
                        value
                    }
                } else {
                    value
                };
                
                // Convert to JSON string for Cedar
                let attr_value = serde_json::to_string(&transformed_value)
                    .unwrap_or_else(|_| format!("\"{}\"", transformed_value));
                attributes.insert(cedar_attr.clone(), attr_value);
            }
        }

        // Extract parent entities using parent_mappings
        let mut parents = Vec::new();
        
        for mapping in &config.parent_mappings {
            if let Some(value) = Self::get_nested_claim(claims, &mapping.claim_path) {
                // Handle both arrays and single values
                let values_array = if let Some(arr) = value.as_array() {
                    arr.clone()
                } else if let Some(str_val) = value.as_str() {
                    vec![Value::String(str_val.to_string())]
                } else {
                    continue;
                };
                
                for item in values_array {
                    if let Some(item_str) = item.as_str() {
                        let transformed = Self::apply_transform(item_str, &mapping.transform);
                        parents.push(EntityIdentifier {
                            entity_type: mapping.entity_type.clone(),
                            entity_id: transformed,
                        });
                    }
                }
            }
        }
        
        // Legacy support for group_claim
        #[allow(deprecated)]
        if let Some(group_claim) = &config.group_claim {
            if config.parent_mappings.is_empty() {
                if let Some(groups_value) = claims.additional_claims.get(group_claim) {
                    if let Some(groups_array) = groups_value.as_array() {
                        for group in groups_array {
                            if let Some(group_str) = group.as_str() {
                                parents.push(EntityIdentifier {
                                    entity_type: "Role".to_string(),
                                    entity_id: group_str.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Create principal entity
        let principal_entity = Entity {
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
    ) -> Result<Vec<Entity>> {
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
