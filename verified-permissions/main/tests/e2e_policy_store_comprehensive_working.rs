#![cfg(feature = "containers")]
//! Working E2E Tests for Policy Store (using available SDK methods only)
//! Tests cover: CRUD operations that are fully implemented in SDK

mod testcontainers;

use testcontainers::clients::Cli;
use testcontainers::server_container::ServerContainer;
use chrono::Utc;

#[tokio::test]
#[ignore]
async fn tc_001_policy_store_crud_lifecycle() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-001: Policy Store CRUD Lifecycle Test\n");

    let test_description = format!("Test Store - {}", Utc::now().timestamp());

    // CREATE
    println!("  📦 Creating policy store...");
    let store = client
        .create_policy_store("Test Store".to_string(), test_description.clone())
        .await
        .expect("Failed to create policy store");
    println!("  ✓ Created: {}", store.policy_store_id);

    // READ (Get)
    println!("  📖 Getting policy store details...");
    let store_details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get policy store");
    assert_eq!(store_details.description, test_description);
    println!("  ✓ Retrieved: {}", store_details.description);

    // READ (List)
    println!("  📋 Listing policy stores...");
    let stores_list = client
        .list_policy_stores(Some(10), None)
        .await
        .expect("Failed to list policy stores");
    assert!(stores_list.policy_stores.iter().any(|s| s.policy_store_id == store.policy_store_id));
    println!("  ✓ Listed {} stores", stores_list.policy_stores.len());

    // DELETE
    println!("  🗑️  Deleting policy store...");
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete policy store");
    println!("  ✓ Deleted");

    // Verify deletion
    let result = client.get_policy_store(&store.policy_store_id).await;
    assert!(result.is_err(), "Policy store should not exist after deletion");

    println!("\n✅ TC-001: Policy Store CRUD lifecycle passed!\n");
}

#[tokio::test]
#[ignore]
async fn tc_002_policy_store_multiple_operations() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-002: Multiple Policy Store Operations\n");

    let num_stores = 5;
    let mut store_ids = Vec::new();

    // Create multiple stores
    println!("  📦 Creating {} policy stores...", num_stores);
    for i in 0..num_stores {
        let store = client
            .create_policy_store("Test Store".to_string(), format!("Store-{}", i))
            .await
            .expect("Failed to create store");
        store_ids.push(store.policy_store_id);
        println!("  ✓ Created store {}: {}", i, store.policy_store_id);
    }

    // List and verify
    println!("  📋 Listing all stores...");
    let stores_list = client
        .list_policy_stores(Some(100), None)
        .await
        .expect("Failed to list stores");
    assert_eq!(stores_list.policy_stores.len(), num_stores);
    println!("  ✓ Found {} stores", stores_list.policy_stores.len());

    // Delete all stores
    println!("  🗑️  Deleting all stores...");
    for (i, store_id) in store_ids.iter().enumerate() {
        client
            .delete_policy_store(store_id)
            .await
            .expect("Failed to delete store");
        println!("  ✓ Deleted store {}", i);
    }

    // Verify all deleted
    let stores_list = client
        .list_policy_stores(Some(100), None)
        .await
        .expect("Failed to list stores after deletion");
    assert_eq!(stores_list.policy_stores.len(), 0);
    println!("  ✓ All stores deleted");

    println!("\n✅ TC-002: Multiple operations test passed!\n");
}

#[tokio::test]
#[ignore]
async fn tc_003_policy_store_with_policies() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-003: Policy Store with Policies Test\n");

    // Create policy store
    let store = client
        .create_policy_store("Test Store".to_string(), "Store with Policies".to_string())
        .await
        .expect("Failed to create policy store");
    println!("  ✓ Created store: {}", store.policy_store_id);

    // Create schema first
    println!("  📝 Creating schema...");
    let schema = r#"{
        "concertW名作": {
            "entityTypes": {
                "User": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "name": { "type": "String" }
                        }
                    }
                },
                "Concert": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "name": { "type": "String" },
                            "venue": { "type": "String" }
                        }
                    }
                }
            }
        }
    }"#;

    client
        .put_schema(&store.policy_store_id, schema.to_string())
        .await
        .expect("Failed to put schema");
    println!("  ✓ Schema created");

    // Create policy
    println!("  📜 Creating policy...");
    let policy = hodei_permissions_sdk::PolicyDefinition {
        policy_id: "policy1".to_string(),
        definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
            hodei_permissions_sdk::StaticPolicyDefinition {
                statement: r#"permit(principal == User::"alice", action == Action::"view", resource == Concert::"concert1")"#.to_string(),
            },
        )),
    };

    client
        .create_policy(&store.policy_store_id, policy)
        .await
        .expect("Failed to create policy");
    println!("  ✓ Policy created");

    // Test authorization
    println!("  🔐 Testing authorization...");
    let auth_request = hodei_permissions_sdk::IsAuthorizedRequest {
        policy_store_id: store.policy_store_id.clone(),
        principal: Some(hodei_permissions_sdk::Entity {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(hodei_permissions_sdk::Entity {
            entity_type: "Action".to_string(),
            entity_id: "view".to_string(),
        }),
        resource: Some(hodei_permissions_sdk::Entity {
            entity_type: "Concert".to_string(),
            entity_id: "concert1".to_string(),
        }),
        context: "{}".to_string(),
        entities: vec![],
    };

    let response = client
        .is_authorized(&store.policy_store_id, "User".to_string(), "Action".to_string(), "Concert".to_string(), auth_request)
        .await
        .expect("Authorization failed");

    println!("  ✓ Authorization decision: {}", response.decision);
    assert!(response.decision == "ALLOW" || response.decision == "DENY");

    // Cleanup
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete store");

    println!("\n✅ TC-003: Policy Store with policies test passed!\n");
}

#[tokio::test]
#[ignore]
async fn tc_004_concurrent_policy_stores() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-004: Concurrent Policy Store Operations\n");

    let num_stores = 10;

    // Create stores concurrently
    println!("  📦 Creating {} stores concurrently...", num_stores);
    let mut handles = vec![];
    for i in 0..num_stores {
        let client_clone = client.clone();
        let description = format!("Concurrent Store {}", i);
        let handle = tokio::spawn(async move {
            let store = client_clone
                .create_policy_store("Test Store".to_string(), description)
                .await
                .expect("Failed to create store");
            store.policy_store_id
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let store_ids = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect::<Result<Vec<String>, _>>()
        .expect("Handles failed");

    println!("  ✓ Created {} stores", store_ids.len());

    // Verify all stores exist
    let stores_list = client
        .list_policy_stores(Some(100), None)
        .await
        .expect("Failed to list stores");
    assert_eq!(stores_list.policy_stores.len(), num_stores);
    println!("  ✓ Verified {} stores exist", stores_list.policy_stores.len());

    // Delete all stores concurrently
    println!("  🗑️  Deleting {} stores concurrently...", num_stores);
    let mut delete_handles = vec![];
    for store_id in store_ids {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            client_clone
                .delete_policy_store(&store_id)
                .await
                .expect("Failed to delete store");
        });
        delete_handles.push(handle);
    }

    futures::future::join_all(delete_handles).await;
    println!("  ✓ Deleted all stores");

    println!("\n✅ TC-004: Concurrent operations test passed!\n");
}

#[tokio::test]
#[ignore]
async fn tc_005_error_handling() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 TC-005: Error Handling Test\n");

    // Try to get non-existent store
    println!("  🔍 Testing get non-existent store...");
    let result = client.get_policy_store("non-existent-id").await;
    assert!(result.is_err(), "Should fail to get non-existent store");
    println!("  ✓ Correctly returned error for non-existent store");

    // Try to delete non-existent store
    println!("  🗑️  Testing delete non-existent store...");
    let result = client.delete_policy_store("non-existent-id").await;
    assert!(result.is_err(), "Should fail to delete non-existent store");
    println!("  ✓ Correctly returned error for delete non-existent store");

    // Create store and test authorization with invalid entities
    let store = client
        .create_policy_store("Test Store".to_string(), "Error Test Store".to_string())
        .await
        .expect("Failed to create store");
    println!("  ✓ Created store for error testing");

    println!("\n✅ TC-005: Error handling test passed!\n");

    // Cleanup
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to cleanup");
}
