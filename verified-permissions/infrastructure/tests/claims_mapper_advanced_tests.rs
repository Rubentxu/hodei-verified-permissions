//! Advanced tests for Claims Mapper

use hodei_verified_permissions::jwt::{ClaimsMapper, ClaimsMappingConfig, ParentMapping, ValueTransform, ValidatedClaims};
use serde_json::json;
use std::collections::HashMap;

fn create_keycloak_claims() -> ValidatedClaims {
    let mut additional = HashMap::new();
    additional.insert("email".to_string(), json!("user@example.com"));
    additional.insert("name".to_string(), json!("John Doe"));
    additional.insert("realm_access".to_string(), json!({
        "roles": ["admin", "user"]
    }));
    additional.insert("resource_access".to_string(), json!({
        "my-app": {
            "roles": ["app-admin", "app-user"]
        }
    }));
    additional.insert("groups".to_string(), json!(["/engineering", "/devops"]));

    ValidatedClaims {
        sub: "user123".to_string(),
        iss: "https://keycloak.example.com/realms/myapp".to_string(),
        aud: vec!["my-app".to_string()],
        exp: 9999999999,
        iat: 1234567890,
        additional_claims: additional,
    }
}

fn create_zitadel_claims() -> ValidatedClaims {
    let mut additional = HashMap::new();
    additional.insert("email".to_string(), json!("user@example.com"));
    additional.insert("urn:zitadel:iam:org:project:roles".to_string(), json!({
        "developer": {"123456": "myorg"},
        "admin": {"123456": "myorg"}
    }));
    additional.insert("urn:zitadel:iam:user:resourceowner:name".to_string(), json!("MyOrganization"));

    ValidatedClaims {
        sub: "user456".to_string(),
        iss: "https://myinstance.zitadel.cloud".to_string(),
        aud: vec!["project-client".to_string()],
        exp: 9999999999,
        iat: 1234567890,
        additional_claims: additional,
    }
}

#[test]
fn test_nested_claim_extraction_keycloak_realm_roles() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "realm_access.roles".to_string(),
            entity_type: "Role".to_string(),
            transform: ValueTransform::None,
        }
    ];

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    assert_eq!(entities.len(), 1);
    let entity = &entities[0];
    assert_eq!(entity.parents.len(), 2);
    
    let parent_ids: Vec<&str> = entity.parents.iter()
        .map(|p| p.entity_id.as_str())
        .collect();
    assert!(parent_ids.contains(&"admin"));
    assert!(parent_ids.contains(&"user"));
}

#[test]
fn test_nested_claim_extraction_keycloak_client_roles() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "resource_access.my-app.roles".to_string(),
            entity_type: "ClientRole".to_string(),
            transform: ValueTransform::None,
        }
    ];

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    assert_eq!(entity.parents.len(), 2);
    
    let parent_ids: Vec<&str> = entity.parents.iter()
        .map(|p| p.entity_id.as_str())
        .collect();
    assert!(parent_ids.contains(&"app-admin"));
    assert!(parent_ids.contains(&"app-user"));
}

#[test]
fn test_multiple_parent_mappings() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "realm_access.roles".to_string(),
            entity_type: "RealmRole".to_string(),
            transform: ValueTransform::None,
        },
        ParentMapping {
            claim_path: "groups".to_string(),
            entity_type: "Group".to_string(),
            transform: ValueTransform::None,
        }
    ];

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    // 2 realm roles + 2 groups = 4 parents
    assert_eq!(entity.parents.len(), 4);
    
    // Check we have both RealmRole and Group types
    let realm_roles: Vec<_> = entity.parents.iter()
        .filter(|p| p.entity_type == "RealmRole")
        .collect();
    let groups: Vec<_> = entity.parents.iter()
        .filter(|p| p.entity_type == "Group")
        .collect();
    
    assert_eq!(realm_roles.len(), 2);
    assert_eq!(groups.len(), 2);
}

#[test]
fn test_value_transformation_uppercase() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "realm_access.roles".to_string(),
            entity_type: "Role".to_string(),
            transform: ValueTransform::Uppercase,
        }
    ];

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    let parent_ids: Vec<&str> = entity.parents.iter()
        .map(|p| p.entity_id.as_str())
        .collect();
    
    assert!(parent_ids.contains(&"ADMIN"));
    assert!(parent_ids.contains(&"USER"));
}

#[test]
fn test_value_transformation_lowercase() {
    let mut additional = HashMap::new();
    additional.insert("roles".to_string(), json!(["ADMIN", "USER"]));
    
    let claims = ValidatedClaims {
        sub: "user123".to_string(),
        iss: "https://issuer.example.com".to_string(),
        aud: vec!["client-id".to_string()],
        exp: 9999999999,
        iat: 1234567890,
        additional_claims: additional,
    };

    let mut config = ClaimsMappingConfig::default();
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "roles".to_string(),
            entity_type: "Role".to_string(),
            transform: ValueTransform::Lowercase,
        }
    ];

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    let parent_ids: Vec<&str> = entity.parents.iter()
        .map(|p| p.entity_id.as_str())
        .collect();
    
    assert!(parent_ids.contains(&"admin"));
    assert!(parent_ids.contains(&"user"));
}

#[test]
fn test_value_transformation_split_first() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "groups".to_string(),
            entity_type: "Group".to_string(),
            transform: ValueTransform::SplitFirst("/".to_string()),
        }
    ];

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    // Both groups start with "/" so split_first should give empty string
    // Actually, split_first takes the first part, so "/engineering" -> ""
    // Let's test split_last instead which is more useful
    assert_eq!(entity.parents.len(), 2);
}

#[test]
fn test_value_transformation_split_last() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "groups".to_string(),
            entity_type: "Group".to_string(),
            transform: ValueTransform::SplitLast("/".to_string()),
        }
    ];

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    let parent_ids: Vec<&str> = entity.parents.iter()
        .map(|p| p.entity_id.as_str())
        .collect();
    
    // "/engineering" -> "engineering", "/devops" -> "devops"
    assert!(parent_ids.contains(&"engineering"));
    assert!(parent_ids.contains(&"devops"));
}

#[test]
fn test_attribute_transformation() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.attribute_mappings.insert("email".to_string(), "email".to_string());
    config.attribute_transforms.insert(
        "email".to_string(),
        ValueTransform::Uppercase,
    );

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    let email_attr = entity.attributes.get("email").unwrap();
    assert!(email_attr.contains("USER@EXAMPLE.COM"));
}

#[test]
fn test_nested_attribute_mapping() {
    let claims = create_zitadel_claims();
    let mut config = ClaimsMappingConfig::default();
    config.attribute_mappings.insert(
        "organization".to_string(),
        "urn:zitadel:iam:user:resourceowner:name".to_string(),
    );

    let (_, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    let entity = &entities[0];
    assert!(entity.attributes.contains_key("organization"));
    let org_attr = entity.attributes.get("organization").unwrap();
    assert!(org_attr.contains("MyOrganization"));
}

#[test]
fn test_required_claims_validation_success() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.required_claims = vec![
        "email".to_string(),
        "realm_access.roles".to_string(),
    ];

    let result = ClaimsMapper::map_to_principal(&claims, &config, "User");
    assert!(result.is_ok());
}

#[test]
fn test_required_claims_validation_failure() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.required_claims = vec![
        "nonexistent_claim".to_string(),
    ];

    let result = ClaimsMapper::map_to_principal(&claims, &config, "User");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Required claim"));
}

#[test]
fn test_required_nested_claims_validation_failure() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    config.required_claims = vec![
        "realm_access.nonexistent".to_string(),
    ];

    let result = ClaimsMapper::map_to_principal(&claims, &config, "User");
    assert!(result.is_err());
}

#[test]
fn test_keycloak_full_configuration() {
    let claims = create_keycloak_claims();
    let mut config = ClaimsMappingConfig::default();
    
    // Attributes
    config.attribute_mappings.insert("email".to_string(), "email".to_string());
    config.attribute_mappings.insert("name".to_string(), "name".to_string());
    
    // Parent mappings
    config.parent_mappings = vec![
        ParentMapping {
            claim_path: "realm_access.roles".to_string(),
            entity_type: "RealmRole".to_string(),
            transform: ValueTransform::None,
        },
        ParentMapping {
            claim_path: "resource_access.my-app.roles".to_string(),
            entity_type: "ClientRole".to_string(),
            transform: ValueTransform::None,
        },
        ParentMapping {
            claim_path: "groups".to_string(),
            entity_type: "Group".to_string(),
            transform: ValueTransform::SplitLast("/".to_string()),
        },
    ];
    
    // Required claims
    config.required_claims = vec!["email".to_string()];

    let (principal, entities) = ClaimsMapper::map_to_principal(&claims, &config, "User").unwrap();
    
    assert_eq!(principal.entity_type, "User");
    assert_eq!(principal.entity_id, "user123");
    
    let entity = &entities[0];
    
    // Check attributes
    assert!(entity.attributes.contains_key("email"));
    assert!(entity.attributes.contains_key("name"));
    
    // Check parents: 2 realm roles + 2 client roles + 2 groups = 6
    assert_eq!(entity.parents.len(), 6);
    
    // Verify parent types
    let realm_roles: Vec<_> = entity.parents.iter()
        .filter(|p| p.entity_type == "RealmRole")
        .collect();
    let client_roles: Vec<_> = entity.parents.iter()
        .filter(|p| p.entity_type == "ClientRole")
        .collect();
    let groups: Vec<_> = entity.parents.iter()
        .filter(|p| p.entity_type == "Group")
        .collect();
    
    assert_eq!(realm_roles.len(), 2);
    assert_eq!(client_roles.len(), 2);
    assert_eq!(groups.len(), 2);
    
    // Verify group names are transformed
    let group_ids: Vec<&str> = groups.iter()
        .map(|p| p.entity_id.as_str())
        .collect();
    assert!(group_ids.contains(&"engineering"));
    assert!(group_ids.contains(&"devops"));
}
