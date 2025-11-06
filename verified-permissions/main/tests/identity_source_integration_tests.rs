//! Integration tests for Identity Source functionality

use hodei_infrastructure::SqliteRepository;

#[tokio::test]
async fn test_identity_source_crud() {
    // Create in-memory database
    let repo = SqliteRepository::new(":memory:").await.unwrap();

    // Create a policy store first
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Create OIDC identity source
    let oidc_config = r#"{"issuer":"https://accounts.google.com","client_ids":["client-123"],"jwks_uri":"https://www.googleapis.com/oauth2/v3/certs","group_claim":"groups"}"#;
    let claims_mapping =
        r#"{"principal_id_claim":"sub","group_claim":"groups","attribute_mappings":{}}"#;

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

    assert_eq!(identity_source.policy_store_id, store.id);
    assert_eq!(identity_source.configuration_type, "oidc");
    assert!(identity_source.description.is_some());

    // Get identity source
    let retrieved = repo
        .get_identity_source(&store.id, &identity_source.id)
        .await
        .unwrap();

    assert_eq!(retrieved.id, identity_source.id);
    assert_eq!(retrieved.configuration_type, "oidc");

    // List identity sources
    let sources = repo.list_identity_sources(&store.id).await.unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].id, identity_source.id);

    // Delete identity source
    repo.delete_identity_source(&store.id, &identity_source.id)
        .await
        .unwrap();

    // Verify deletion
    let result = repo
        .get_identity_source(&store.id, &identity_source.id)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_identity_sources() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Create OIDC source
    let oidc_config = r#"{"issuer":"https://accounts.google.com","client_ids":["client-123"],"jwks_uri":"https://www.googleapis.com/oauth2/v3/certs","group_claim":"groups"}"#;
    repo.create_identity_source(&store.id, "oidc", oidc_config, None, Some("Google"))
        .await
        .unwrap();

    // Create Cognito source
    let cognito_config = r#"{"user_pool_arn":"arn:aws:cognito-idp:us-east-1:123456789012:userpool/us-east-1_ABC123","client_ids":"client1,client2","group_configuration_group_claim":"cognito:groups"}"#;
    repo.create_identity_source(&store.id, "cognito", cognito_config, None, Some("Cognito"))
        .await
        .unwrap();

    // List all sources
    let sources = repo.list_identity_sources(&store.id).await.unwrap();
    assert_eq!(sources.len(), 2);

    let types: Vec<&str> = sources
        .iter()
        .map(|s| s.configuration_type.as_str())
        .collect();
    assert!(types.contains(&"oidc"));
    assert!(types.contains(&"cognito"));
}

#[tokio::test]
async fn test_identity_source_not_found() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    let result = repo.get_identity_source(&store.id, "nonexistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_identity_source_cascade_delete() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    let oidc_config = r#"{"issuer":"https://test.com","client_ids":["client"],"jwks_uri":"https://test.com/jwks","group_claim":"groups"}"#;
    let source = repo
        .create_identity_source(&store.id, "oidc", oidc_config, None, None)
        .await
        .unwrap();

    // Delete policy store (should cascade delete identity source)
    repo.delete_policy_store(&store.id).await.unwrap();

    // Verify identity source is also deleted
    let result = repo.get_identity_source(&store.id, &source.id).await;
    assert!(result.is_err());
}
