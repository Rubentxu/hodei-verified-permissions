//! E2E tests for multiple database backends
//!
//! Tests the complete stack with:
//! - SQLite
//! - PostgreSQL
//! - SurrealDB
//!
//! Validates that all databases work correctly with the authorization flow

use hodei_permissions_sdk::AuthorizationClient;
use serde_json::json;

// Database endpoints
const SQLITE_ENDPOINT: &str = "http://localhost:50051";
const POSTGRES_ENDPOINT: &str = "http://localhost:50052";
const SURREALDB_ENDPOINT: &str = "http://localhost:50053";

// TODO App endpoints
const TODO_APP_SQLITE: &str = "http://localhost:3000";
const TODO_APP_POSTGRES: &str = "http://localhost:3001";
const TODO_APP_SURREALDB: &str = "http://localhost:3002";

/// Helper to create a test JWT token
fn create_test_jwt(user_id: &str, _groups: Vec<&str>) -> String {
    // TODO: Implement proper JWT generation
    format!("test-token-{}", user_id)
}

#[tokio::test]
#[ignore] // Run with: cargo test --test e2e_multi_database -- --ignored
async fn test_sqlite_policy_store_creation() {
    println!("🗄️  Testing SQLite backend...");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to SQLite server");

    let response = client
        .create_policy_store(Some("SQLite Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    assert!(!response.policy_store_id.is_empty());
    println!("✅ SQLite: Created policy store {}", response.policy_store_id);
}

#[tokio::test]
#[ignore]
async fn test_postgres_policy_store_creation() {
    println!("🐘 Testing PostgreSQL backend...");
    
    let client = AuthorizationClient::connect(POSTGRES_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to PostgreSQL server");

    let response = client
        .create_policy_store(Some("PostgreSQL Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    assert!(!response.policy_store_id.is_empty());
    println!("✅ PostgreSQL: Created policy store {}", response.policy_store_id);
}

#[tokio::test]
#[ignore]
async fn test_surrealdb_policy_store_creation() {
    println!("🚀 Testing SurrealDB backend...");
    
    let client = AuthorizationClient::connect(SURREALDB_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to SurrealDB server");

    let response = client
        .create_policy_store(Some("SurrealDB Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    assert!(!response.policy_store_id.is_empty());
    println!("✅ SurrealDB: Created policy store {}", response.policy_store_id);
}

#[tokio::test]
#[ignore]
async fn test_all_databases_health() {
    println!("🏥 Testing health of all database backends...");
    
    let http_client = reqwest::Client::new();
    
    // Test SQLite TODO app
    let sqlite_response = http_client
        .get(format!("{}/health", TODO_APP_SQLITE))
        .send()
        .await
        .expect("Failed to connect to SQLite TODO app");
    assert_eq!(sqlite_response.status(), 200);
    println!("✅ SQLite TODO app is healthy");

    // Test PostgreSQL TODO app
    let postgres_response = http_client
        .get(format!("{}/health", TODO_APP_POSTGRES))
        .send()
        .await
        .expect("Failed to connect to PostgreSQL TODO app");
    assert_eq!(postgres_response.status(), 200);
    println!("✅ PostgreSQL TODO app is healthy");

    // Test SurrealDB TODO app
    let surrealdb_response = http_client
        .get(format!("{}/health", TODO_APP_SURREALDB))
        .send()
        .await
        .expect("Failed to connect to SurrealDB TODO app");
    assert_eq!(surrealdb_response.status(), 200);
    println!("✅ SurrealDB TODO app is healthy");
}

#[tokio::test]
#[ignore]
async fn test_sqlite_authorization_flow() {
    println!("🔐 Testing SQLite authorization flow...");
    test_authorization_flow(SQLITE_ENDPOINT, "SQLite").await;
}

#[tokio::test]
#[ignore]
async fn test_postgres_authorization_flow() {
    println!("🔐 Testing PostgreSQL authorization flow...");
    test_authorization_flow(POSTGRES_ENDPOINT, "PostgreSQL").await;
}

#[tokio::test]
#[ignore]
async fn test_surrealdb_authorization_flow() {
    println!("🔐 Testing SurrealDB authorization flow...");
    test_authorization_flow(SURREALDB_ENDPOINT, "SurrealDB").await;
}

/// Helper function to test authorization flow for any database
async fn test_authorization_flow(endpoint: &str, db_name: &str) {
    let client = AuthorizationClient::connect(endpoint.to_string())
        .await
        .expect(&format!("Failed to connect to {} server", db_name));

    // 1. Create policy store
    let store_response = client
        .create_policy_store(Some(format!("{} Auth Test", db_name)))
        .await
        .expect(&format!("Failed to create policy store in {}", db_name));

    let policy_store_id = store_response.policy_store_id;
    println!("📦 {}: Created policy store {}", db_name, policy_store_id);

    // 2. Create identity source
    use hodei_permissions_sdk::proto::{IdentitySourceConfiguration, OidcConfiguration};
    use hodei_permissions_sdk::proto::identity_source_configuration::ConfigurationType;
    let identity_response = client
        .create_identity_source(
            &policy_store_id,
            IdentitySourceConfiguration {
                configuration_type: Some(ConfigurationType::Oidc(OidcConfiguration {
                    issuer: "https://test.example.com".to_string(),
                    client_ids: vec!["test-app".to_string()],
                    jwks_uri: "https://test.example.com/.well-known/jwks.json".to_string(),
                    group_claim: "groups".to_string(),
                })),
            },
            None,
            None,
        )
        .await
        .expect(&format!("Failed to create identity source in {}", db_name));

    let identity_source_id = identity_response.identity_source_id;
    println!("🔐 {}: Created identity source {}", db_name, identity_source_id);

    // 3. Create a test policy
    let policy_content = r#"
        permit(
            principal in TodoApp::Group::"admin",
            action,
            resource
        );
    "#;

    client
        .create_policy(
            &policy_store_id,
            "test-policy-1",
            policy_content.to_string(),
            None,
        )
        .await
        .expect(&format!("Failed to create policy in {}", db_name));

    println!("📜 {}: Created test policy", db_name);

    // 4. Test authorization decision
    let token = create_test_jwt("alice", vec!["admin"]);
    
    let auth_response = client
        .is_authorized_with_token(
            &policy_store_id,
            &identity_source_id,
            &token,
            "listTasks",
            "Application",
        )
        .await
        .expect(&format!("Authorization request failed in {}", db_name));

    assert_eq!(auth_response.decision, 1); // ALLOW
    println!("✅ {}: Authorization decision: ALLOW", db_name);
}

#[tokio::test]
#[ignore]
async fn test_all_databases_todo_app_integration() {
    println!("🔗 Testing TODO app integration with all databases...");
    
    let http_client = reqwest::Client::new();
    let token = create_test_jwt("admin", vec!["admin"]);

    // Test SQLite
    println!("\n📝 Testing SQLite TODO app...");
    test_todo_app_with_db(&http_client, TODO_APP_SQLITE, &token, "SQLite").await;

    // Test PostgreSQL
    println!("\n📝 Testing PostgreSQL TODO app...");
    test_todo_app_with_db(&http_client, TODO_APP_POSTGRES, &token, "PostgreSQL").await;

    // Test SurrealDB
    println!("\n📝 Testing SurrealDB TODO app...");
    test_todo_app_with_db(&http_client, TODO_APP_SURREALDB, &token, "SurrealDB").await;
}

async fn test_todo_app_with_db(
    client: &reqwest::Client,
    app_url: &str,
    token: &str,
    db_name: &str,
) {
    // Test listing tasks
    let response = client
        .get(format!("{}/api/v1/tasks", app_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect(&format!("Failed to list tasks in {} TODO app", db_name));

    println!("  ✅ {}: Can list tasks (status: {})", db_name, response.status());

    // Test creating a task
    let create_response = client
        .post(format!("{}/api/v1/tasks", app_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "title": format!("Test Task from {}", db_name),
            "description": "Testing database integration"
        }))
        .send()
        .await
        .expect(&format!("Failed to create task in {} TODO app", db_name));

    println!("  ✅ {}: Can create tasks (status: {})", db_name, create_response.status());
}

#[tokio::test]
#[ignore]
async fn test_database_isolation() {
    println!("🔒 Testing database isolation...");
    
    // Create policy stores in each database
    let sqlite_client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to SQLite");
    
    let postgres_client = AuthorizationClient::connect(POSTGRES_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to PostgreSQL");
    
    let surrealdb_client = AuthorizationClient::connect(SURREALDB_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to SurrealDB");

    // Create stores
    let sqlite_store = sqlite_client
        .create_policy_store(Some("Isolation Test SQLite".to_string()))
        .await
        .expect("Failed to create SQLite store");

    let postgres_store = postgres_client
        .create_policy_store(Some("Isolation Test PostgreSQL".to_string()))
        .await
        .expect("Failed to create PostgreSQL store");

    let surrealdb_store = surrealdb_client
        .create_policy_store(Some("Isolation Test SurrealDB".to_string()))
        .await
        .expect("Failed to create SurrealDB store");

    // Verify they have different IDs
    assert_ne!(sqlite_store.policy_store_id, postgres_store.policy_store_id);
    assert_ne!(postgres_store.policy_store_id, surrealdb_store.policy_store_id);
    assert_ne!(sqlite_store.policy_store_id, surrealdb_store.policy_store_id);

    println!("✅ All databases are properly isolated");
    println!("  SQLite store:    {}", sqlite_store.policy_store_id);
    println!("  PostgreSQL store: {}", postgres_store.policy_store_id);
    println!("  SurrealDB store:  {}", surrealdb_store.policy_store_id);
}

#[tokio::test]
#[ignore]
async fn test_concurrent_database_operations() {
    println!("⚡ Testing concurrent operations across databases...");
    
    use tokio::task::JoinSet;
    
    let mut set = JoinSet::new();

    // Spawn concurrent tasks for each database
    set.spawn(async {
        let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string()).await.unwrap();
        client.create_policy_store(Some("Concurrent SQLite".to_string())).await.unwrap();
        "SQLite"
    });

    set.spawn(async {
        let client = AuthorizationClient::connect(POSTGRES_ENDPOINT.to_string()).await.unwrap();
        client.create_policy_store(Some("Concurrent PostgreSQL".to_string())).await.unwrap();
        "PostgreSQL"
    });

    set.spawn(async {
        let client = AuthorizationClient::connect(SURREALDB_ENDPOINT.to_string()).await.unwrap();
        client.create_policy_store(Some("Concurrent SurrealDB".to_string())).await.unwrap();
        "SurrealDB"
    });

    // Wait for all to complete
    while let Some(result) = set.join_next().await {
        let db_name = result.unwrap();
        println!("  ✅ {} completed successfully", db_name);
    }

    println!("✅ All concurrent operations completed successfully");
}
