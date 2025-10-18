//! E2E tests for Policy Store Management using Testcontainers

mod testcontainers;

use testcontainers::clients::Cli;
use testcontainers::server_container::ServerContainer;

#[tokio::test]
#[ignore] // Requires Docker - run with: cargo test --ignored
async fn test_policy_store_lifecycle() {
    // Setup: Start server container
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    
    // Create SDK client
    let client = hodei_verified_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");
    
    // Test: Create policy store
    let store = client
        .create_policy_store(Some("Test Store".to_string()))
        .await
        .expect("Failed to create policy store");
    
    assert!(!store.policy_store_id.is_empty());
    assert_eq!(store.description, Some("Test Store".to_string()));
    
    // Test: Get policy store
    let retrieved = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get policy store");
    
    assert_eq!(retrieved.policy_store_id, store.policy_store_id);
    
    // Test: List policy stores
    let stores = client
        .list_policy_stores(None, None)
        .await
        .expect("Failed to list policy stores");
    
    assert!(stores.policy_stores.len() >= 1);
    assert!(stores.policy_stores.iter().any(|s| s.policy_store_id == store.policy_store_id));
    
    // Test: Delete policy store
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete policy store");
    
    // Test: Verify deletion
    let result = client.get_policy_store(&store.policy_store_id).await;
    assert!(result.is_err(), "Policy store should not exist after deletion");
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_multiple_policy_stores_isolation() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    
    let client = hodei_verified_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect");
    
    // Create 3 stores
    let store_a = client
        .create_policy_store(Some("Store A".to_string()))
        .await
        .expect("Failed to create store A");
    
    let store_b = client
        .create_policy_store(Some("Store B".to_string()))
        .await
        .expect("Failed to create store B");
    
    let store_c = client
        .create_policy_store(Some("Store C".to_string()))
        .await
        .expect("Failed to create store C");
    
    // Verify all exist
    let stores = client.list_policy_stores(None, None).await.unwrap();
    assert!(stores.policy_stores.len() >= 3);
    
    // Verify each can be retrieved independently
    let retrieved_a = client.get_policy_store(&store_a.policy_store_id).await.unwrap();
    let retrieved_b = client.get_policy_store(&store_b.policy_store_id).await.unwrap();
    let retrieved_c = client.get_policy_store(&store_c.policy_store_id).await.unwrap();
    
    assert_eq!(retrieved_a.policy_store_id, store_a.policy_store_id);
    assert_eq!(retrieved_b.policy_store_id, store_b.policy_store_id);
    assert_eq!(retrieved_c.policy_store_id, store_c.policy_store_id);
    
    // Cleanup
    client.delete_policy_store(&store_a.policy_store_id).await.unwrap();
    client.delete_policy_store(&store_b.policy_store_id).await.unwrap();
    client.delete_policy_store(&store_c.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_policy_store_validation() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    
    let client = hodei_verified_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect");
    
    // Test: Create store with empty description
    let store = client
        .create_policy_store(None)
        .await
        .expect("Should allow empty description");
    
    assert!(!store.policy_store_id.is_empty());
    assert_eq!(store.description, None);
    
    // Test: Get non-existent store
    let result = client.get_policy_store("non-existent-id").await;
    assert!(result.is_err(), "Should fail for non-existent store");
    
    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_policy_store_pagination() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    
    let client = hodei_verified_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect");
    
    // Create 5 stores
    let mut store_ids = Vec::new();
    for i in 0..5 {
        let store = client
            .create_policy_store(Some(format!("Store {}", i)))
            .await
            .unwrap();
        store_ids.push(store.policy_store_id);
    }
    
    // Test: List with limit
    let response = client.list_policy_stores(Some(2), None).await.unwrap();
    assert!(response.policy_stores.len() <= 2);
    
    // Test: Pagination with next_token if available
    if let Some(next_token) = response.next_token {
        let next_page = client
            .list_policy_stores(Some(2), Some(next_token))
            .await
            .unwrap();
        assert!(!next_page.policy_stores.is_empty());
    }
    
    // Cleanup
    for store_id in store_ids {
        client.delete_policy_store(&store_id).await.unwrap();
    }
}
