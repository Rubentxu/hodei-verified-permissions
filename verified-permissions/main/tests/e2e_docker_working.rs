//! E2E Test - Connects to Docker containers that are already running
//! Uses only SDK methods that actually exist

#[tokio::test]
async fn test_001_create_and_get_store() {
    let client = hodei_permissions_sdk::AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TEST-001: Create and Get Policy Store\n");

    // CREATE
    let store = client
        .create_policy_store("Test Store".to_string(), "Test Store 001".to_string())
        .await
        .expect("Failed to create store");

    println!("  ✓ Created: {}", store.policy_store_id);

    // GET
    let details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get store");

    println!("  ✓ Retrieved store details");
    println!("  ✓ Store ID: {}", details.policy_store_id);

    println!("\n✅ TEST-001: PASSED!\n");
}

#[tokio::test]
async fn test_002_list_stores() {
    let client = hodei_permissions_sdk::AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Failed to connect");

    println!("\n🚀 TEST-002: List Policy Stores\n");

    // Create multiple stores
    let num_stores = 3;
    for i in 0..num_stores {
        let _ = client
            .create_policy_store("Test Store".to_string(), format!("List Test Store {}", i))
            .await
            .expect("Failed to create store");
        println!("  ✓ Created store {}", i);
    }

    // LIST
    let list = client
        .list_policy_stores(Some(100), None)
        .await
        .expect("Failed to list stores");

    println!("  ✓ Listed {} stores", list.policy_stores.len());
    assert!(list.policy_stores.len() >= num_stores);

    println!("\n✅ TEST-002: PASSED!\n");
}

#[tokio::test]
async fn test_003_authorization_check() {
    let client = hodei_permissions_sdk::AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Failed to connect");

    println!("\n🚀 TEST-003: Authorization Check\n");

    // Create store
    let store = client
        .create_policy_store("Test Store".to_string(), "Auth Test Store".to_string())
        .await
        .expect("Failed to create store");

    println!("  ✓ Created store for auth test");

    // Authorization check
    let response = client
        .is_authorized(&store.policy_store_id, "User::alice", "Action::view", "Resource::doc1")
        .await
        .expect("Authorization failed");

    println!("  ✓ Authorization decision received");

    // Decision is a number, not string
    println!("  ✓ Decision value: {}", response.decision);

    println!("\n✅ TEST-003: PASSED!\n");
}
