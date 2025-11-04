//! Simple Integration Test - Direct connection to running server
//! This test connects to localhost:50051 (real server)

#[tokio::test]
async fn test_simple_policy_store_crud() {
    println!("\n🚀 Running Simple Integration Test\n");

    // Connect to server
    let client = hodei_permissions_sdk::AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Failed to connect to server");

    println!("  ✓ Connected to server");

    // CREATE
    let store = client
        .create_policy_store("Test Store".to_string(), "Integration Test Store".to_string())
        .await
        .expect("Failed to create policy store");

    println!("  ✓ Created store: {}", store.policy_store_id);

    // READ
    let details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get policy store");

    println!("  ✓ Retrieved store: {:?}", details.description);

    // LIST
    let list = client
        .list_policy_stores(Some(10), None)
        .await
        .expect("Failed to list policy stores");

    println!("  ✓ Listed {} stores", list.policy_stores.len());

    // DELETE
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete policy store");

    println!("  ✓ Deleted store");

    println!("\n✅ Simple Integration Test PASSED!\n");
}

#[tokio::test]
async fn test_multiple_stores() {
    println!("\n🚀 Running Multiple Stores Test\n");

    let client = hodei_permissions_sdk::AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Failed to connect");

    let num_stores = 3;
    let mut store_ids = Vec::new();

    // Create multiple stores
    for i in 0..num_stores {
        let store = client
            .create_policy_store("Test Store".to_string(), format!("Store {}", i))
            .await
            .expect("Failed to create store");
        let store_id = store.policy_store_id.clone();
        store_ids.push(store_id.clone());
        println!("  ✓ Created store {}: {}", i, store_id);
    }

    // List and verify
    let list = client
        .list_policy_stores(Some(100), None)
        .await
        .expect("Failed to list stores");

    assert_eq!(list.policy_stores.len(), num_stores);
    println!("  ✓ Verified {} stores in list", list.policy_stores.len());

    // Delete all
    for store_id in store_ids {
        client
            .delete_policy_store(&store_id)
            .await
            .expect("Failed to delete store");
    }

    println!("  ✓ Deleted all stores");
    println!("\n✅ Multiple Stores Test PASSED!\n");
}

#[tokio::test]
async fn test_authorization() {
    println!("\n🚀 Running Authorization Test\n");

    let client = hodei_permissions_sdk::AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Failed to connect");

    // Create store
    let store = client
        .create_policy_store("Test Store".to_string(), "Auth Test Store".to_string())
        .await
        .expect("Failed to create store");

    println!("  ✓ Created store for auth test");

    // Test authorization
    let response = client
        .is_authorized(&store.policy_store_id, "User::alice", "Action::view", "Resource::doc1")
        .await
        .expect("Authorization failed");

    println!("  ✓ Authorization decision: {}", response.decision);

    // Cleanup
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete store");

    println!("  ✓ Cleaned up");
    println!("\n✅ Authorization Test PASSED!\n");
}
