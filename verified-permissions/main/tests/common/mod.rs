//! Common test utilities for repository testing
//! 
//! Este módulo proporciona tests reutilizables que pueden ejecutarse
//! contra cualquier implementación de PolicyRepository.

use hodei_domain::PolicyRepository;
use hodei_domain::DomainResult as Result;

/// Suite de tests completa para cualquier implementación de PolicyRepository
/// 
/// Esta función ejecuta todos los tests contra una implementación de repository.
/// Útil para verificar que todas las implementaciones (SQLite, Postgres, SurrealDB)
/// cumplen con el contrato del trait.
pub async fn run_repository_test_suite<F, Fut>(create_repo: F) 
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<Box<dyn PolicyRepository>>>,
{
    // Policy Store Tests
    test_policy_store_lifecycle(create_repo().await.unwrap().as_ref()).await;
    test_multiple_policy_stores(create_repo().await.unwrap().as_ref()).await;
    
    // Schema Tests
    test_schema_management(create_repo().await.unwrap().as_ref()).await;
    
    // Policy Tests
    test_policy_crud(create_repo().await.unwrap().as_ref()).await;
    test_policy_with_conditions(create_repo().await.unwrap().as_ref()).await;
    
    // Identity Source Tests
    test_identity_source_crud(create_repo().await.unwrap().as_ref()).await;
    
    // Cascade Delete Tests
    test_cascade_delete(create_repo().await.unwrap().as_ref()).await;
}

/// Test: Policy Store Lifecycle (CRUD completo)
pub async fn test_policy_store_lifecycle(repo: &dyn PolicyRepository) {
    // Create
    let store = repo.create_policy_store("Test Store".to_string(), "Test Store".to_string())
        .await
        .expect("Failed to create policy store");
    
    assert!(!store.id.is_empty());
    assert_eq!(store.description, Some("Test Store".to_string()));
    
    // Read
    let retrieved = repo.get_policy_store(&store.id)
        .await
        .expect("Failed to get policy store");
    
    assert_eq!(retrieved.id, store.id);
    
    // List
    let stores = repo.list_policy_stores()
        .await
        .expect("Failed to list policy stores");
    
    assert!(stores.len() >= 1);
    assert!(stores.iter().any(|s| s.id == store.id));
    
    // Delete
    repo.delete_policy_store(&store.id)
        .await
        .expect("Failed to delete policy store");
    
    // Verify deletion
    let result = repo.get_policy_store(&store.id).await;
    assert!(result.is_err(), "Policy store should not exist after deletion");
}

/// Test: Múltiples Policy Stores (aislamiento)
pub async fn test_multiple_policy_stores(repo: &dyn PolicyRepository) {
    let store_a = repo.create_policy_store("Test Store".to_string(), "Store A".to_string()).await.unwrap();
    let store_b = repo.create_policy_store("Test Store".to_string(), "Store B".to_string()).await.unwrap();
    let store_c = repo.create_policy_store("Test Store".to_string(), "Store C".to_string()).await.unwrap();
    
    let stores = repo.list_policy_stores().await.unwrap();
    assert!(stores.len() >= 3);
    
    let retrieved_a = repo.get_policy_store(&store_a.id).await.unwrap();
    let retrieved_b = repo.get_policy_store(&store_b.id).await.unwrap();
    let retrieved_c = repo.get_policy_store(&store_c.id).await.unwrap();
    
    assert_eq!(retrieved_a.description, Some("Store A".to_string()));
    assert_eq!(retrieved_b.description, Some("Store B".to_string()));
    assert_eq!(retrieved_c.description, Some("Store C".to_string()));
}

/// Test: Schema Management
pub async fn test_schema_management(repo: &dyn PolicyRepository) {
    let store = repo.create_policy_store("Test Store".to_string(), "Schema Test".to_string()).await.unwrap();
    
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
    
    repo.put_schema(&store.id, schema.to_string()).await.unwrap();
    
    let retrieved_schema = repo.get_schema(&store.id).await.unwrap();
    assert!(!retrieved_schema.schema_json.is_empty());
    assert!(retrieved_schema.schema_json.contains("User"));
    assert!(retrieved_schema.schema_json.contains("Document"));
}

/// Test: Policy CRUD
pub async fn test_policy_crud(repo: &dyn PolicyRepository) {
    let store = repo.create_policy_store("Test Store".to_string(), "Policy Test".to_string()).await.unwrap();
    
    let schema = r#"{"": {"entityTypes": {"User": {}, "Document": {}}, "actions": {"view": {}}}}"#;
    repo.put_schema(&store.id, schema.to_string()).await.unwrap();
    
    let policy = r#"permit(principal == User::"alice", action == Action::"view", resource == Document::"doc123");"#;
    
    repo.create_policy(&store.id, "policy-1", policy.to_string(), Some("Test policy".to_string()))
        .await
        .unwrap();
    
    let policies = repo.list_policies(&store.id).await.unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].policy_id, "policy-1");
    
    let retrieved = repo.get_policy(&store.id, "policy-1").await.unwrap();
    assert_eq!(retrieved.policy_id, "policy-1");
    assert_eq!(retrieved.description, Some("Test policy".to_string()));
    
    repo.delete_policy(&store.id, "policy-1").await.unwrap();
    
    let policies = repo.list_policies(&store.id).await.unwrap();
    assert_eq!(policies.len(), 0);
}

/// Test: Policy con condiciones
pub async fn test_policy_with_conditions(repo: &dyn PolicyRepository) {
    let store = repo.create_policy_store("Test Store".to_string(), "Conditions Test".to_string()).await.unwrap();
    
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
    repo.put_schema(&store.id, schema.to_string()).await.unwrap();
    
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

/// Test: Identity Source CRUD
pub async fn test_identity_source_crud(repo: &dyn PolicyRepository) {
    let store = repo.create_policy_store("Test Store".to_string(), "Identity Test".to_string()).await.unwrap();
    
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
    
    let sources = repo.list_identity_sources(&store.id).await.unwrap();
    assert_eq!(sources.len(), 1);
    
    let retrieved = repo.get_identity_source(&store.id, &identity_source.id).await.unwrap();
    assert_eq!(retrieved.id, identity_source.id);
    
    repo.delete_identity_source(&store.id, &identity_source.id).await.unwrap();
    
    let sources = repo.list_identity_sources(&store.id).await.unwrap();
    assert_eq!(sources.len(), 0);
}

/// Test: Cascade Delete
pub async fn test_cascade_delete(repo: &dyn PolicyRepository) {
    let store = repo.create_policy_store("Test Store".to_string(), "Cascade Test".to_string()).await.unwrap();
    
    let schema = r#"{"": {"entityTypes": {"User": {}}, "actions": {"view": {}}}}"#;
    repo.put_schema(&store.id, schema.to_string()).await.unwrap();
    
    repo.create_policy(&store.id, "p1", "permit(principal, action, resource);".to_string(), None)
        .await
        .unwrap();
    
    let oidc_config = r#"{"issuer":"https://example.com"}"#;
    repo.create_identity_source(&store.id, "oidc", oidc_config, None, None)
        .await
        .unwrap();
    
    repo.delete_policy_store(&store.id).await.unwrap();
    
    assert!(repo.get_policy_store(&store.id).await.is_err());
}
