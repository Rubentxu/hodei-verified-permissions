#![cfg(feature = "containers")]
//! E2E integration tests with real Keycloak instance
//!
//! These tests validate the complete authorization flow:
//! 1. Start Keycloak container
//! 2. Configure identity source with Keycloak
//! 3. Create Cedar policies
//! 4. Get real JWT tokens from Keycloak
//! 5. Call IsAuthorizedWithToken
//! 6. Verify Allow/Deny decisions

mod testcontainers;
use hodei_permissions_sdk::AuthorizationClient;
use hodei_permissions_sdk::proto::{
    IdentitySourceConfiguration, OidcConfiguration, ClaimsMappingConfiguration,
    identity_source_configuration, Decision,
};
use std::collections::HashMap;
use testcontainers::{KeycloakContainer, KeycloakConfig};

#[tokio::test]
#[ignore] // Requires Docker
async fn test_keycloak_e2e_allow_with_realm_role() {
    println!("\n🧪 Test: Keycloak E2E - Allow with realm role\n");

    // 1. Start Keycloak
    let mut kc_config = KeycloakConfig::default();
    kc_config.users = vec![
        ("admin_user".to_string(), "password".to_string(), vec!["admin".to_string()]),
        ("regular_user".to_string(), "password".to_string(), vec!["user".to_string()]),
    ];

    let keycloak = KeycloakContainer::start_with_config(kc_config)
        .await
        .expect("Failed to start Keycloak");

    println!("✅ Keycloak started at: {}", keycloak.issuer());

    // 2. Start our server (assuming it's running on localhost:50051)
    // In a real E2E test, you would start the server here
    let server_url = "http://localhost:50051";
    let client = AuthorizationClient::connect(server_url)
        .await
        .expect("Failed to create client");

    // 3. Create policy store
    let store = client
        .create_policy_store(Some("Keycloak E2E Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    println!("✅ Policy store created: {}", store.policy_store_id);

    // 4. Create identity source with Keycloak
    let mut attribute_mappings = HashMap::new();
    attribute_mappings.insert("email".to_string(), "email".to_string());
    attribute_mappings.insert("name".to_string(), "name".to_string());

    let oidc_config = OidcConfiguration {
        issuer: keycloak.issuer(),
        client_ids: vec![keycloak.client_id().to_string()],
        jwks_uri: keycloak.jwks_uri(),
        group_claim: "realm_access.roles".to_string(), // Keycloak realm roles
    };

    let identity_source_config = IdentitySourceConfiguration {
        configuration_type: Some(identity_source_configuration::ConfigurationType::Oidc(oidc_config)),
    };

    let claims_mapping = ClaimsMappingConfiguration {
        principal_id_claim: "sub".to_string(),
        group_claim: String::new(), // Already configured in OIDC config
        attribute_mappings,
    };

    let identity_source = client
        .create_identity_source(
            &store.policy_store_id,
            identity_source_config,
            Some(claims_mapping),
            Some("Keycloak identity source".to_string()),
        )
        .await
        .expect("Failed to create identity source");

    println!("✅ Identity source created: {}", identity_source.identity_source_id);

    // 5. Create Cedar policy: Allow admins to read documents
    let policy_statement = r#"
        permit(
            principal in RealmRole::"admin",
            action == Action::"read",
            resource == Document::"doc1"
        );
    "#;

    client
        .create_policy(
            &store.policy_store_id,
            "allow-admin-read",
            policy_statement,
            Some("Allow admins to read documents".to_string()),
        )
        .await
        .expect("Failed to create policy");

    println!("✅ Policy created");

    // 6. Get JWT token for admin user
    let token_response = keycloak
        .get_user_token("admin_user", "password")
        .await
        .expect("Failed to get token");

    println!("✅ JWT token obtained for admin_user");

    // 7. Call IsAuthorizedWithToken - Should ALLOW
    let response = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token_response.access_token,
            "Action::\"read\"",
            "Document::\"doc1\"",
        )
        .await
        .expect("Failed to call IsAuthorizedWithToken");

    println!("📋 Authorization decision: {:?}", response.decision);

    // Verify Allow decision
    assert_eq!(
        response.decision,
        Decision::Allow as i32,
        "Expected Allow decision for admin user"
    );

    println!("✅ Test passed: Admin user allowed to read document");
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_keycloak_e2e_deny_without_role() {
    println!("\n🧪 Test: Keycloak E2E - Deny without required role\n");

    // 1. Start Keycloak
    let mut kc_config = KeycloakConfig::default();
    kc_config.users = vec![
        ("admin_user".to_string(), "password".to_string(), vec!["admin".to_string()]),
        ("regular_user".to_string(), "password".to_string(), vec!["user".to_string()]),
    ];

    let keycloak = KeycloakContainer::start_with_config(kc_config)
        .await
        .expect("Failed to start Keycloak");

    // 2. Start server and create client
    let server_url = "http://localhost:50051";
    let client = AuthorizationClient::connect(server_url)
        .await
        .expect("Failed to create client");

    // 3. Create policy store
    let store = client
        .create_policy_store(Some("Keycloak Deny Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // 4. Create identity source
    let oidc_config = OidcConfiguration {
        issuer: keycloak.issuer(),
        client_ids: vec![keycloak.client_id().to_string()],
        jwks_uri: keycloak.jwks_uri(),
        group_claim: "realm_access.roles".to_string(),
    };

    let identity_source_config = IdentitySourceConfiguration {
        configuration_type: Some(identity_source_configuration::ConfigurationType::Oidc(oidc_config)),
    };

    let claims_mapping = ClaimsMappingConfiguration {
        principal_id_claim: "sub".to_string(),
        group_claim: String::new(),
        attribute_mappings: HashMap::new(),
    };

    let identity_source = client
        .create_identity_source(
            &store.policy_store_id,
            identity_source_config,
            Some(claims_mapping),
            None,
        )
        .await
        .expect("Failed to create identity source");

    // 5. Create policy: Only admins can delete
    let policy_statement = r#"
        permit(
            principal in RealmRole::"admin",
            action == Action::"delete",
            resource == Document::"doc1"
        );
    "#;

    client
        .create_policy(
            &store.policy_store_id,
            "admin-only-delete",
            policy_statement,
            Some("Only admins can delete".to_string()),
        )
        .await
        .expect("Failed to create policy");

    // 6. Get token for regular user (without admin role)
    let token_response = keycloak
        .get_user_token("regular_user", "password")
        .await
        .expect("Failed to get token");

    println!("✅ JWT token obtained for regular_user");

    // 7. Call IsAuthorizedWithToken - Should DENY
    let response = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token_response.access_token,
            "Action::\"delete\"",
            "Document::\"doc1\"",
        )
        .await
        .expect("Failed to call IsAuthorizedWithToken");

    println!("📋 Authorization decision: {:?}", response.decision);

    // Verify Deny decision
    assert_eq!(
        response.decision,
        Decision::Deny as i32,
        "Expected Deny decision for regular user"
    );

    println!("✅ Test passed: Regular user denied delete action");
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_keycloak_e2e_multiple_roles() {
    println!("\n🧪 Test: Keycloak E2E - Multiple roles\n");

    // 1. Start Keycloak with users having multiple roles
    let mut kc_config = KeycloakConfig::default();
    kc_config.users = vec![
        (
            "power_user".to_string(),
            "password".to_string(),
            vec!["admin".to_string(), "user".to_string(), "developer".to_string()],
        ),
        ("basic_user".to_string(), "password".to_string(), vec!["user".to_string()]),
    ];

    let keycloak = KeycloakContainer::start_with_config(kc_config)
        .await
        .expect("Failed to start Keycloak");

    // 2. Setup server and client
    let server_url = "http://localhost:50051";
    let client = AuthorizationClient::connect(server_url)
        .await
        .expect("Failed to create client");

    // 3. Create policy store
    let store = client
        .create_policy_store(Some("Multi-role Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // 4. Create identity source
    let oidc_config = OidcConfiguration {
        issuer: keycloak.issuer(),
        client_ids: vec![keycloak.client_id().to_string()],
        jwks_uri: keycloak.jwks_uri(),
        group_claim: "realm_access.roles".to_string(),
    };

    let identity_source_config = IdentitySourceConfiguration {
        configuration_type: Some(identity_source_configuration::ConfigurationType::Oidc(oidc_config)),
    };

    let identity_source = client
        .create_identity_source(
            &store.policy_store_id,
            identity_source_config,
            Some(ClaimsMappingConfiguration {
                principal_id_claim: "sub".to_string(),
                group_claim: String::new(),
                attribute_mappings: HashMap::new(),
            }),
            None,
        )
        .await
        .expect("Failed to create identity source");

    // 5. Create policy: Developers can deploy
    let policy_statement = r#"
        permit(
            principal in RealmRole::"developer",
            action == Action::"deploy",
            resource == Application::"myapp"
        );
    "#;

    client
        .create_policy(
            &store.policy_store_id,
            "developer-deploy",
            policy_statement,
            Some("Developers can deploy".to_string()),
        )
        .await
        .expect("Failed to create policy");

    // 6. Test power_user (has developer role) - Should ALLOW
    let token = keycloak
        .get_user_token("power_user", "password")
        .await
        .expect("Failed to get token");

    let response = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token.access_token,
            "Action::\"deploy\"",
            "Application::\"myapp\"",
        )
        .await
        .expect("Failed to call IsAuthorizedWithToken");

    assert_eq!(
        response.decision,
        Decision::Allow as i32,
        "Power user should be allowed to deploy"
    );

    println!("✅ Power user (with developer role) allowed to deploy");

    // 7. Test basic_user (no developer role) - Should DENY
    let token = keycloak
        .get_user_token("basic_user", "password")
        .await
        .expect("Failed to get token");

    let response = client
        .is_authorized_with_token(
            &store.policy_store_id,
            &identity_source.identity_source_id,
            &token.access_token,
            "Action::\"deploy\"",
            "Application::\"myapp\"",
        )
        .await
        .expect("Failed to call IsAuthorizedWithToken");

    assert_eq!(
        response.decision,
        Decision::Deny as i32,
        "Basic user should be denied deploy"
    );

    println!("✅ Basic user (without developer role) denied deploy");
}
