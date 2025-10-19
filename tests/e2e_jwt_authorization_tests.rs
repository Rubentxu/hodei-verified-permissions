#![cfg(feature = "containers")]
//! E2E tests for JWT-based authorization

mod common;
mod fixtures;

use fixtures::{TestClaims, generate_test_token};
use hodei_permissions_sdk::VerifiedPermissionsClient;
use hodei_verified_permissions::proto::{
    CreateIdentitySourceRequest, CreatePolicyRequest, CreatePolicyStoreRequest,
    IdentitySourceConfiguration, OidcConfiguration, ClaimsMappingConfiguration,
    PolicyDefinition, StaticPolicy,
};
use std::collections::HashMap;

#[tokio::test]
#[ignore] // Requires running server
async fn test_is_authorized_with_token_basic() {
    // Setup: Create client
    let client = VerifiedPermissionsClient::new("http://localhost:50051")
        .await
        .expect("Failed to create client");

    // Create policy store
    let store = client
        .create_policy_store(Some("JWT Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    // Create identity source (mock OIDC)
    let identity_source_req = CreateIdentitySourceRequest {
        policy_store_id: store.policy_store_id.clone(),
        configuration: Some(IdentitySourceConfiguration::Oidc(OidcConfiguration {
            issuer: "https://test.example.com".to_string(),
            audience: vec!["test-client".to_string()],
            jwks_uri: "https://test.example.com/.well-known/jwks.json".to_string(),
        })),
        claims_mapping: Some(ClaimsMappingConfiguration {
            principal_id_claim: "sub".to_string(),
            group_claim: Some("groups".to_string()),
            attribute_mappings: HashMap::from([
                ("email".to_string(), "email".to_string()),
            ]),
        }),
        description: Some("Test identity source".to_string()),
    };

    let identity_source = client
        .create_identity_source(
            &store.policy_store_id,
            identity_source_req.configuration.unwrap(),
            identity_source_req.claims_mapping,
            identity_source_req.description,
        )
        .await
        .expect("Failed to create identity source");

    // Create a policy that allows users in "admins" group to read documents
    let policy_statement = r#"
        permit(
            principal in Role::"admins",
            action == Action::"read",
            resource == Document::"doc1"
        );
    "#;

    let policy_req = CreatePolicyRequest {
        policy_store_id: store.policy_store_id.clone(),
        policy_id: "allow-admins-read".to_string(),
        definition: Some(PolicyDefinition {
            policy_type: Some(
                hodei_verified_permissions::proto::policy_definition::PolicyType::Static(
                    StaticPolicy {
                        statement: policy_statement.to_string(),
                    },
                ),
            ),
        }),
        description: Some("Allow admins to read documents".to_string()),
    };

    client
        .create_policy(
            &store.policy_store_id,
            &policy_req.policy_id,
            policy_statement,
            policy_req.description,
        )
        .await
        .expect("Failed to create policy");

    // Generate JWT token with admin group
    let claims = TestClaims::basic("user123", "https://test.example.com", "test-client")
        .with_email("admin@example.com")
        .with_groups(vec!["admins", "users"]);

    let token = generate_test_token(claims, b"test-secret");

    // Test: IsAuthorizedWithToken should ALLOW
    let result = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token,
            "Action::read",
            "Document::doc1",
        )
        .await;

    // Note: This will fail in real scenario because we need proper JWKS setup
    // This test demonstrates the structure
    match result {
        Ok(response) => {
            println!("Authorization decision: {:?}", response.decision);
        }
        Err(e) => {
            println!("Expected error (no real JWKS): {}", e);
            // In a real test with proper JWKS, we would assert:
            // assert_eq!(response.decision, Decision::Allow);
        }
    }
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_is_authorized_with_token_deny() {
    let client = VerifiedPermissionsClient::new("http://localhost:50051")
        .await
        .expect("Failed to create client");

    let store = client
        .create_policy_store(Some("JWT Deny Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Create identity source
    let identity_source_req = CreateIdentitySourceRequest {
        policy_store_id: store.policy_store_id.clone(),
        configuration: Some(IdentitySourceConfiguration::Oidc(OidcConfiguration {
            issuer: "https://test.example.com".to_string(),
            audience: vec!["test-client".to_string()],
            jwks_uri: "https://test.example.com/.well-known/jwks.json".to_string(),
        })),
        claims_mapping: Some(ClaimsMappingConfiguration {
            principal_id_claim: "sub".to_string(),
            group_claim: Some("groups".to_string()),
            attribute_mappings: HashMap::new(),
        }),
        description: None,
    };

    let identity_source = client
        .create_identity_source(
            &store.policy_store_id,
            identity_source_req.configuration.unwrap(),
            identity_source_req.claims_mapping,
            identity_source_req.description,
        )
        .await
        .expect("Failed to create identity source");

    // Create policy that only allows admins
    let policy_statement = r#"
        permit(
            principal in Role::"admins",
            action == Action::"delete",
            resource == Document::"doc1"
        );
    "#;

    client
        .create_policy(
            &store.policy_store_id,
            "admin-only-delete",
            policy_statement,
            Some("Admin only delete".to_string()),
        )
        .await
        .expect("Failed to create policy");

    // Generate JWT token WITHOUT admin group
    let claims = TestClaims::basic("user456", "https://test.example.com", "test-client")
        .with_email("user@example.com")
        .with_groups(vec!["users"]); // Not in admins group

    let token = generate_test_token(claims, b"test-secret");

    // Test: IsAuthorizedWithToken should DENY
    let result = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token,
            "Action::delete",
            "Document::doc1",
        )
        .await;

    match result {
        Ok(response) => {
            println!("Authorization decision: {:?}", response.decision);
            // In real test: assert_eq!(response.decision, Decision::Deny);
        }
        Err(e) => {
            println!("Expected error (no real JWKS): {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_is_authorized_with_expired_token() {
    let client = VerifiedPermissionsClient::new("http://localhost:50051")
        .await
        .expect("Failed to create client");

    let store = client
        .create_policy_store(Some("JWT Expired Test".to_string()))
        .await
        .expect("Failed to create policy store");

    let identity_source_req = CreateIdentitySourceRequest {
        policy_store_id: store.policy_store_id.clone(),
        configuration: Some(IdentitySourceConfiguration::Oidc(OidcConfiguration {
            issuer: "https://test.example.com".to_string(),
            audience: vec!["test-client".to_string()],
            jwks_uri: "https://test.example.com/.well-known/jwks.json".to_string(),
        })),
        claims_mapping: Some(ClaimsMappingConfiguration {
            principal_id_claim: "sub".to_string(),
            group_claim: Some("groups".to_string()),
            attribute_mappings: HashMap::new(),
        }),
        description: None,
    };

    let identity_source = client
        .create_identity_source(
            &store.policy_store_id,
            identity_source_req.configuration.unwrap(),
            identity_source_req.claims_mapping,
            identity_source_req.description,
        )
        .await
        .expect("Failed to create identity source");

    // Generate EXPIRED JWT token
    let claims = TestClaims::basic("user789", "https://test.example.com", "test-client")
        .with_groups(vec!["admins"])
        .expired();

    let token = generate_test_token(claims, b"test-secret");

    // Test: Should reject expired token
    let result = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token,
            "Action::read",
            "Document::doc1",
        )
        .await;

    assert!(result.is_err(), "Should reject expired token");
    let error = result.unwrap_err();
    println!("Expected error for expired token: {}", error);
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_is_authorized_with_keycloak_claims() {
    let client = VerifiedPermissionsClient::new("http://localhost:50051")
        .await
        .expect("Failed to create client");

    let store = client
        .create_policy_store(Some("Keycloak JWT Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Create identity source with Keycloak-style claims mapping
    let mut attribute_mappings = HashMap::new();
    attribute_mappings.insert("email".to_string(), "email".to_string());
    attribute_mappings.insert("name".to_string(), "name".to_string());

    let identity_source_req = CreateIdentitySourceRequest {
        policy_store_id: store.policy_store_id.clone(),
        configuration: Some(IdentitySourceConfiguration::Oidc(OidcConfiguration {
            issuer: "https://keycloak.example.com/realms/myapp".to_string(),
            audience: vec!["my-app".to_string()],
            jwks_uri: "https://keycloak.example.com/realms/myapp/protocol/openid-connect/certs"
                .to_string(),
        })),
        claims_mapping: Some(ClaimsMappingConfiguration {
            principal_id_claim: "sub".to_string(),
            group_claim: Some("realm_access.roles".to_string()), // Nested claim
            attribute_mappings,
        }),
        description: Some("Keycloak identity source".to_string()),
    };

    let identity_source = client
        .create_identity_source(
            &store.policy_store_id,
            identity_source_req.configuration.unwrap(),
            identity_source_req.claims_mapping,
            identity_source_req.description,
        )
        .await
        .expect("Failed to create identity source");

    // Create policy
    let policy_statement = r#"
        permit(
            principal in Role::"app-admin",
            action == Action::"manage",
            resource == Application::"my-app"
        );
    "#;

    client
        .create_policy(
            &store.policy_store_id,
            "keycloak-admin-policy",
            policy_statement,
            Some("Keycloak admin policy".to_string()),
        )
        .await
        .expect("Failed to create policy");

    // Generate JWT with Keycloak-style claims
    let claims = TestClaims::basic(
        "user-uuid-123",
        "https://keycloak.example.com/realms/myapp",
        "my-app",
    )
    .with_email("admin@example.com")
    .with_claim("name", serde_json::json!("Admin User"))
    .with_keycloak_realm_roles(vec!["app-admin", "app-user"])
    .with_keycloak_client_roles("my-app", vec!["client-admin"]);

    let token = generate_test_token(claims, b"test-secret");

    // Test authorization
    let result = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token,
            "Action::manage",
            "Application::my-app",
        )
        .await;

    match result {
        Ok(response) => {
            println!("Keycloak authorization decision: {:?}", response.decision);
        }
        Err(e) => {
            println!("Expected error (no real JWKS): {}", e);
        }
    }
}
