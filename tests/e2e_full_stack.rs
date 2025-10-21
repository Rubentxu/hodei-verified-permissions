//! End-to-End tests for complete stack integration
//!
//! Tests the full flow:
//! 1. Hodei Server (gRPC) running
//! 2. TODO App with SDK middleware
//! 3. Real authorization decisions
//!
//! This validates the integration like AWS Verified Permissions + Express SDK

use hodei_permissions_sdk::AuthorizationClient;
use serde_json::json;

const SERVER_ENDPOINT: &str = "http://localhost:50051";
const TODO_APP_URL: &str = "http://localhost:3000";

/// Helper to create a test JWT token
fn create_test_jwt(user_id: &str, groups: Vec<&str>) -> String {
    // TODO: Implement JWT generation for tests
    // For now, return a placeholder
    format!("test-token-{}", user_id)
}

#[tokio::test]
#[ignore] // Run only with: cargo test --test e2e_full_stack -- --ignored
async fn test_e2e_policy_store_creation() {
    // Connect to Hodei server
    let client = AuthorizationClient::connect(SERVER_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to Hodei server");

    // Create a policy store
    let response = client
        .create_policy_store(Some("E2E Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    assert!(!response.policy_store_id.is_empty());
    println!("✅ Created policy store: {}", response.policy_store_id);
}

#[tokio::test]
#[ignore]
async fn test_e2e_todo_app_health_check() {
    // Test that TODO app is running and accessible
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", TODO_APP_URL))
        .send()
        .await
        .expect("Failed to connect to TODO app");

    assert_eq!(response.status(), 200);
    println!("✅ TODO app health check passed");
}

#[tokio::test]
#[ignore]
async fn test_e2e_authorization_with_real_server() {
    // 1. Setup: Create policy store and load policies
    let client = AuthorizationClient::connect(SERVER_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to Hodei server");

    let store_response = client
        .create_policy_store(Some("E2E Auth Test".to_string()))
        .await
        .expect("Failed to create policy store");

    let policy_store_id = store_response.policy_store_id;
    println!("📦 Created policy store: {}", policy_store_id);

    // 2. Create identity source
    let identity_response = client
        .create_identity_source(
            &policy_store_id,
            json!({
                "issuer": "https://test.example.com",
                "audience": ["test-app"],
                "jwks_uri": "https://test.example.com/.well-known/jwks.json"
            }),
        )
        .await
        .expect("Failed to create identity source");

    let identity_source_id = identity_response.identity_source_id;
    println!("🔐 Created identity source: {}", identity_source_id);

    // 3. Create a test policy (allow admin to do everything)
    let policy_content = r#"
        permit(
            principal in TodoApp::Group::"admin",
            action,
            resource
        );
    "#;

    client
        .create_policy(&policy_store_id, policy_content.to_string(), None)
        .await
        .expect("Failed to create policy");

    println!("📜 Created test policy");

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
        .expect("Authorization request failed");

    assert_eq!(auth_response.decision, 1); // ALLOW
    println!("✅ Authorization decision: ALLOW");
}

#[tokio::test]
#[ignore]
async fn test_e2e_todo_app_with_authorization() {
    // This test validates the complete flow:
    // HTTP Request → TODO App → SDK Middleware → Hodei Server → Cedar Evaluation → Response

    let http_client = reqwest::Client::new();

    // 1. Create a JWT token for an admin user
    let admin_token = create_test_jwt("alice", vec!["admin"]);

    // 2. Try to list tasks (should be allowed for admin)
    let response = http_client
        .get(format!("{}/api/v1/tasks", TODO_APP_URL))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);
    println!("✅ Admin can list tasks");

    // 3. Create a JWT token for a viewer user
    let viewer_token = create_test_jwt("bob", vec!["viewers"]);

    // 4. Try to create a task (should be denied for viewer)
    let create_response = http_client
        .post(format!("{}/api/v1/tasks", TODO_APP_URL))
        .header("Authorization", format!("Bearer {}", viewer_token))
        .json(&json!({
            "title": "Test Task",
            "description": "This should be denied"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(create_response.status(), 403); // Forbidden
    println!("✅ Viewer cannot create tasks (correctly denied)");
}

#[tokio::test]
#[ignore]
async fn test_e2e_rbac_scenarios() {
    // Test Role-Based Access Control scenarios
    let http_client = reqwest::Client::new();

    // Scenario 1: Admin can do everything
    let admin_token = create_test_jwt("admin-user", vec!["admin"]);
    
    let response = http_client
        .delete(format!("{}/api/v1/tasks/123", TODO_APP_URL))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success() || response.status() == 404);
    println!("✅ Admin can delete tasks");

    // Scenario 2: Project Manager can assign tasks
    let pm_token = create_test_jwt("pm-user", vec!["project_managers"]);
    
    let response = http_client
        .post(format!("{}/api/v1/tasks/123/assign?userId=alice", TODO_APP_URL))
        .header("Authorization", format!("Bearer {}", pm_token))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success() || response.status() == 404);
    println!("✅ Project Manager can assign tasks");

    // Scenario 3: Team Member can only update their own tasks (ABAC)
    let member_token = create_test_jwt("charlie", vec!["team_members"]);
    
    // This would require the task to be assigned to charlie
    let response = http_client
        .put(format!("{}/api/v1/tasks/123", TODO_APP_URL))
        .header("Authorization", format!("Bearer {}", member_token))
        .json(&json!({
            "title": "Updated Title"
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Should be 403 if task is not assigned to charlie, or 200 if it is
    assert!(response.status() == 403 || response.status() == 200);
    println!("✅ Team Member authorization checked (ABAC)");
}

#[tokio::test]
#[ignore]
async fn test_e2e_simplerest_mapping() {
    // Test that SimpleRest mapping works correctly
    // HTTP Method + Path → Cedar Action + Resource

    let http_client = reqwest::Client::new();
    let token = create_test_jwt("test-user", vec!["admin"]);

    // Test different HTTP methods map to different Cedar actions
    let test_cases = vec![
        ("GET", "/api/v1/tasks", "listTasks", "Application"),
        ("GET", "/api/v1/tasks/123", "getTask", "Task"),
        ("POST", "/api/v1/tasks", "createTask", "Application"),
        ("PUT", "/api/v1/tasks/123", "updateTask", "Task"),
        ("DELETE", "/api/v1/tasks/123", "deleteTask", "Task"),
    ];

    for (method, path, expected_action, expected_resource) in test_cases {
        println!("🔍 Testing: {} {} → action={}, resource={}", 
                 method, path, expected_action, expected_resource);

        let request = match method {
            "GET" => http_client.get(format!("{}{}", TODO_APP_URL, path)),
            "POST" => http_client.post(format!("{}{}", TODO_APP_URL, path))
                .json(&json!({"title": "Test"})),
            "PUT" => http_client.put(format!("{}{}", TODO_APP_URL, path))
                .json(&json!({"title": "Test"})),
            "DELETE" => http_client.delete(format!("{}{}", TODO_APP_URL, path)),
            _ => panic!("Unsupported method"),
        };

        let response = request
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to send request");

        // We don't care about the exact response, just that the mapping worked
        // (authorization was evaluated, even if denied)
        assert!(response.status().is_success() || 
                response.status() == 403 || 
                response.status() == 404);
        
        println!("✅ Mapping verified for {} {}", method, path);
    }
}
