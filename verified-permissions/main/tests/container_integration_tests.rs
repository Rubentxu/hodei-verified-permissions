#![cfg(feature = "containers")]
//! Integration tests using Testcontainers for PostgreSQL and SurrealDB

mod common;

use testcontainers_modules::{postgres::Postgres, surrealdb::SurrealDb};
use testcontainers::{clients, Container};

// TODO: Update container tests for new architecture
// use hodei_infrastructure::config::{DatabaseConfig, DatabaseProvider};
// use hodei_infrastructure::create_repository;

use common::*;

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_repository_with_containers() {
    let docker = clients::Cli::default();
    let postgres_container = docker.run(Postgres::default());
    let connection_string = format!(
        "postgresql://postgres:postgres@localhost:{}/postgres",
        postgres_container.get_host_port_ipv4(5432)
    );

    // Create repository
    let config = DatabaseConfig {
        provider: DatabaseProvider::Postgres,
        url: connection_string,
        max_connections: 5,
    };

    let repo = create_repository(&config).await.expect("Failed to create PostgreSQL repository");
    let repo_ref = repo.as_ref();

    // Run all repository tests
    run_repository_test_suite(|| async { Ok(repo.clone()) }).await;

    // Additional PostgreSQL-specific tests
    postgres_specific_tests(repo_ref).await;
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_surreal_repository_with_containers() {
    let docker = clients::Cli::default();
    let surreal_container = docker.run(SurrealDb::default());
    let connection_string = format!(
        "ws://localhost:{}",
        surreal_container.get_host_port_ipv4(8000)
    );

    // Create repository
    let config = DatabaseConfig {
        provider: DatabaseProvider::Surreal,
        url: connection_string,
        max_connections: 5,
    };

    let repo = create_repository(&config).await.expect("Failed to create SurrealDB repository");
    let repo_ref = repo.as_ref();

    // Run all repository tests
    run_repository_test_suite(|| async { Ok(repo.clone()) }).await;

    // Additional SurrealDB-specific tests
    surreal_specific_tests(repo_ref).await;
}

/// PostgreSQL-specific tests
async fn postgres_specific_tests(repo: &dyn PolicyRepository) {
    // Test UUID generation
    let store = repo.create_policy_store(Some("UUID Test".to_string())).await.unwrap();

    // PostgreSQL uses UUIDs, so the ID should be a valid UUID
    assert_eq!(store.id.len(), 36); // UUID v4 length
    assert!(store.id.contains('-')); // UUID format

    // Test concurrent connections (PostgreSQL handles this well)
    let store_clone = store.clone();
    let repo_clone = repo;

    let handle = tokio::spawn(async move {
        // This should work with PostgreSQL's connection pooling
        let retrieved = repo_clone.get_policy_store(&store_clone.id).await.unwrap();
        assert_eq!(retrieved.id, store_clone.id);
    });

    handle.await.unwrap();
}

/// SurrealDB-specific tests
async fn surreal_specific_tests(repo: &dyn PolicyRepository) {
    // Test record ID generation
    let store = repo.create_policy_store(Some("Record ID Test".to_string())).await.unwrap();

    // SurrealDB generates unique IDs
    assert!(!store.id.is_empty());

    // Test SurrealQL graph-like operations
    let user_store = repo.create_policy_store(Some("User Store".to_string())).await.unwrap();
    let admin_store = repo.create_policy_store(Some("Admin Store".to_string())).await.unwrap();

    // Both stores should be accessible
    let retrieved_user = repo.get_policy_store(&user_store.id).await.unwrap();
    let retrieved_admin = repo.get_policy_store(&admin_store.id).await.unwrap();

    assert_eq!(retrieved_user.description, Some("User Store".to_string()));
    assert_eq!(retrieved_admin.description, Some("Admin Store".to_string()));
}

/// Test that all repositories behave identically
#[tokio::test]
async fn test_repository_consistency() {
    // Test with SQLite (always available)
    let sqlite_config = DatabaseConfig {
        provider: DatabaseProvider::Sqlite,
        url: ":memory:".to_string(),
        max_connections: 5,
    };

    let sqlite_repo = create_repository(&sqlite_config).await.expect("SQLite repo creation failed");

    // Run basic tests to ensure consistency
    let store = sqlite_repo.create_policy_store(Some("Consistency Test".to_string())).await.unwrap();
    let retrieved = sqlite_repo.get_policy_store(&store.id).await.unwrap();

    assert_eq!(retrieved.id, store.id);
    assert_eq!(retrieved.description, store.description);

    // Test schema
    let schema = r#"{"": {"entityTypes": {"User": {}}, "actions": {"read": {}}}}"#;
    sqlite_repo.put_schema(&store.id, schema.to_string()).await.unwrap();
    let retrieved_schema = sqlite_repo.get_schema(&store.id).await.unwrap();
    assert_eq!(retrieved_schema.schema_json, schema);

    // Test policy
    let policy = r#"permit(principal == User::"alice", action == Action::"read", resource);"#;
    sqlite_repo.create_policy(&store.id, "test-policy", policy.to_string(), Some("Test policy".to_string())).await.unwrap();
    let policies = sqlite_repo.list_policies(&store.id).await.unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0].policy_id, "test-policy");
}

/// Integration test combining cache + repository
#[cfg(test)]
mod cache_integration_tests {
    use super::*;
    use hodei_verified_permissions::cache::CacheManager;
    use hodei_verified_permissions::authorization::AuthorizationService;

    #[tokio::test]
    async fn test_full_stack_integration() {
        // Setup repository
        let config = DatabaseConfig {
            provider: DatabaseProvider::Sqlite,
            url: ":memory:".to_string(),
            max_connections: 5,
        };

        let repo = create_repository(&config).await.expect("Repository creation failed");
        let cache_manager = CacheManager::new(repo.clone());
        cache_manager.initialize().await.expect("Cache initialization failed");

        let auth_service = AuthorizationService::new(cache_manager);

        // Create policy store with policy
        let store = repo.create_policy_store(Some("Integration Test".to_string())).await.unwrap();

        let schema = r#"{"": {"entityTypes": {"User": {}, "Document": {}}, "actions": {"view": {}}}}"#;
        repo.put_schema(&store.id, schema.to_string()).await.unwrap();

        let policy = r#"permit(principal == User::"alice", action == Action::"view", resource == Document::"doc123");"#;
        repo.create_policy(&store.id, "p1", policy.to_string(), None).await.unwrap();

        // Test authorization
        let response = auth_service.is_authorized(
            &store.id,
            "User::\"alice\"",
            "Action::\"view\"",
            "Document::\"doc123\"",
            None,
            None,
        ).await.expect("Authorization failed");

        assert_eq!(response.decision, cedar_policy::Decision::Allow);
        assert_eq!(response.determining_policies.len(), 1);
        assert!(response.errors.is_empty());
    }
}
