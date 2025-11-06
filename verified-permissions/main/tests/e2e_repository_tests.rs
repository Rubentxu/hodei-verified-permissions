//! E2E tests for Repository layer

use hodei_infrastructure::SqliteRepository;

/// Helper para crear un repositorio de prueba
async fn create_test_repo() -> SqliteRepository {
    SqliteRepository::new(":memory:")
        .await
        .expect("Failed to create test repository")
}

#[tokio::test]
async fn test_policy_store_lifecycle() {
    let repo = create_test_repo().await;

    // Test: Create policy store
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .expect("Failed to create policy store");

    assert!(!store.id.is_empty());
    assert_eq!(store.description, Some("Test Store".to_string()));

    // Test: Get policy store
    let retrieved = repo
        .get_policy_store(&store.id)
        .await
        .expect("Failed to get policy store");

    assert_eq!(retrieved.id, store.id);
    assert_eq!(retrieved.description, store.description);

    // Test: List policy stores
    let stores = repo
        .list_policy_stores()
        .await
        .expect("Failed to list policy stores");

    assert!(stores.len() >= 1);
    assert!(stores.iter().any(|s| s.id == store.id));

    // Test: Delete policy store
    repo.delete_policy_store(&store.id)
        .await
        .expect("Failed to delete policy store");

    // Test: Verify deletion
    let result = repo.get_policy_store(&store.id).await;
    assert!(
        result.is_err(),
        "Policy store should not exist after deletion"
    );
}

#[tokio::test]
async fn test_multiple_policy_stores() {
    let repo = create_test_repo().await;

    // Create 3 stores
    let store_a = repo
        .create_policy_store("Test Store".to_string(), Some("Store A".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();
    let store_b = repo
        .create_policy_store("Test Store".to_string(), Some("Store B".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();
    let store_c = repo
        .create_policy_store("Test Store".to_string(), Some("Store C".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Verify all exist
    let stores = repo.list_policy_stores().await.unwrap();
    assert!(stores.len() >= 3);

    // Verify each can be retrieved
    let retrieved_a = repo.get_policy_store(&store_a.id).await.unwrap();
    let retrieved_b = repo.get_policy_store(&store_b.id).await.unwrap();
    let retrieved_c = repo.get_policy_store(&store_c.id).await.unwrap();

    assert_eq!(retrieved_a.description, Some("Store A".to_string()));
    assert_eq!(retrieved_b.description, Some("Store B".to_string()));
    assert_eq!(retrieved_c.description, Some("Store C".to_string()));
}

#[tokio::test]
async fn test_schema_management() {
    let repo = create_test_repo().await;

    // Create store
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Schema Test".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Add schema
    let schema = r#"{
        "": {
            "entityTypes": {
                "User": {},
                "Document": {}
            },
            "actions": {
                "view": {},
                "edit": {}
            }
        }
    }"#;

    repo.put_schema(&store.id, schema.to_string())
        .await
        .unwrap();

    // Verify schema was saved
    let retrieved_schema = repo.get_schema(&store.id).await.unwrap();
    assert!(!retrieved_schema.schema_json.is_empty());
    assert!(retrieved_schema.schema_json.contains("User"));
    assert!(retrieved_schema.schema_json.contains("Document"));
}

#[tokio::test]
async fn test_policy_crud() {
    let repo = create_test_repo().await;

    // Create store with schema
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Policy Test".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    let schema = r#"{"": {"entityTypes": {"User": {}, "Document": {}}, "actions": {"view": {}}}}"#;
    repo.put_schema(&store.id, schema.to_string())
        .await
        .unwrap();

    // Create policy
    let policy = r#"permit(principal == User::"alice", action == Action::"view", resource == Document::"doc123");"#;

    repo.create_policy(
        &store.id,
        "policy-1",
        policy.to_string(),
        Some("Test policy".to_string()),
    )
    .await
    .unwrap();

    // List policies
    let policies = repo.list_policies(&store.id).await.unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].policy_id, "policy-1");

    // Get policy
    let retrieved = repo.get_policy(&store.id, "policy-1").await.unwrap();
    assert_eq!(retrieved.policy_id, "policy-1");
    assert_eq!(retrieved.description, Some("Test policy".to_string()));

    // Delete policy
    repo.delete_policy(&store.id, "policy-1").await.unwrap();

    let policies = repo.list_policies(&store.id).await.unwrap();
    assert_eq!(policies.len(), 0);
}

#[tokio::test]
async fn test_policy_with_conditions() {
    let repo = create_test_repo().await;

    let store = repo
        .create_policy_store(
            "Test Store".to_string(),
            Some("Conditions Test".to_string()),
            vec![],
            "test_user".to_string(),
        )
        .await
        .unwrap();

    let schema = r#"{
        "": {
            "entityTypes": {
                "User": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "department": { "type": "String" }
                        }
                    }
                },
                "Document": {}
            },
            "actions": {"view": {}}
        }
    }"#;
    repo.put_schema(&store.id, schema.to_string())
        .await
        .unwrap();

    // Policy with when condition
    let policy = r#"
        permit(
            principal,
            action == Action::"view",
            resource
        )
        when {
            principal.department == "engineering"
        };
    "#;

    repo.create_policy(&store.id, "cond-policy", policy.to_string(), None)
        .await
        .unwrap();

    let policies = repo.list_policies(&store.id).await.unwrap();
    assert_eq!(policies.len(), 1);
    assert!(policies[0].statement.contains("when"));
}

#[tokio::test]
async fn test_identity_source_crud() {
    let repo = create_test_repo().await;

    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Identity Test".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Create OIDC identity source
    let oidc_config = r#"{"issuer":"https://accounts.google.com","client_ids":["client-123"],"jwks_uri":"https://www.googleapis.com/oauth2/v3/certs"}"#;
    let claims_mapping = r#"{"principal_id_claim":"sub","group_claim":"groups"}"#;

    let identity_source = repo
        .create_identity_source(
            &store.id,
            "oidc",
            oidc_config,
            Some(claims_mapping),
            Some("Google OAuth"),
        )
        .await
        .unwrap();

    assert!(!identity_source.id.is_empty());
    assert_eq!(identity_source.configuration_type, "oidc");

    // List identity sources
    let sources = repo.list_identity_sources(&store.id).await.unwrap();
    assert_eq!(sources.len(), 1);

    // Get identity source
    let retrieved = repo
        .get_identity_source(&store.id, &identity_source.id)
        .await
        .unwrap();
    assert_eq!(retrieved.id, identity_source.id);

    // Delete identity source
    repo.delete_identity_source(&store.id, &identity_source.id)
        .await
        .unwrap();

    let sources = repo.list_identity_sources(&store.id).await.unwrap();
    assert_eq!(sources.len(), 0);
}

#[tokio::test]
async fn test_cascade_delete() {
    let repo = create_test_repo().await;

    // Create store with schema, policies, and identity source
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Cascade Test".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    let schema = r#"{"": {"entityTypes": {"User": {}}, "actions": {"view": {}}}}"#;
    repo.put_schema(&store.id, schema.to_string())
        .await
        .unwrap();

    repo.create_policy(
        &store.id,
        "p1",
        "permit(principal, action, resource);".to_string(),
        None,
    )
    .await
    .unwrap();

    let oidc_config = r#"{"issuer":"https://example.com"}"#;
    repo.create_identity_source(&store.id, "oidc", oidc_config, None, None)
        .await
        .unwrap();

    // Delete store should cascade
    repo.delete_policy_store(&store.id).await.unwrap();

    // Verify everything is deleted
    assert!(repo.get_policy_store(&store.id).await.is_err());
    assert!(
        repo.list_policies(&store.id).await.is_err()
            || repo.list_policies(&store.id).await.unwrap().is_empty()
    );
}
