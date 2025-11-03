#![cfg(feature = "containers")]
//! Minimal Working E2E Tests for Policy Store
//! Tests only the SDK methods that are guaranteed to exist

mod testcontainers;

use testcontainers::clients::Cli;
use testcontainers::server_container::ServerContainer;
use chrono::Utc;

#[tokio::test]
#[ignore]
async fn tc_001_simple_crud() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-001: Simple CRUD Test\n");

    // CREATE
    let store = client
        .create_policy_store(Some("Test Store".to_string()))
        .await
        .expect("Failed to create store");
    println!("  ✓ Created: {}", store.policy_store_id);

    // READ
    let _details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get store");
    println!("  ✓ Retrieved store details");

    // DELETE
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete store");
    println!("  ✓ Deleted store");

    println!("\n✅ TC-001: PASSED!\n");
}

#[tokio::test]
#[ignore]
async fn tc_002_list_operations() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-002: List Operations Test\n");

    let num_stores = 3;
    let mut store_ids = Vec::new();

    // Create multiple stores
    for i in 0..num_stores {
        let store = client
            .create_policy_store(Some(format!("Store {}", i)))
            .await
            .expect("Failed to create store");
        store_ids.push(store.policy_store_id);
        println!("  ✓ Created store {}: {}", i, store.policy_store_id);
    }

    // List stores
    let list = client
        .list_policy_stores(Some(100), None)
        .await
        .expect("Failed to list stores");
    println!("  ✓ Listed {} stores", list.policy_stores.len());
    assert_eq!(list.policy_stores.len(), num_stores);

    // Delete all
    for store_id in store_ids {
        client
            .delete_policy_store(&store_id)
            .await
            .expect("Failed to delete store");
    }
    println!("  ✓ Deleted all stores");

    println!("\n✅ TC-002: PASSED!\n");
}

#[tokio::test]
#[ignore]
async fn tc_003_authorization_flow() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-003: Authorization Flow Test\n");

    // Create store
    let store = client
        .create_policy_store(Some("Auth Test Store".to_string()))
        .await
        .expect("Failed to create store");
    println!("  ✓ Created store");

    // Test authorization (simple version)
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

    println!("\n✅ TC-003: PASSED!\n");
}
