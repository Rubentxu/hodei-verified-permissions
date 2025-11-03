#![cfg(feature = "containers")]
//! Comprehensive E2E Tests for Policy Store with all Features (Phases 1-3.1)
//! Tests cover: CRUD, Metrics, Audit, Tags, Snapshots, Batch Operations, Authorization

mod testcontainers;

use testcontainers::clients::Cli;
use testcontainers::server_container::ServerContainer;
use chrono::Utc;

// Test Constants
const TEST_DATABASE_URL: &str = "sqlite::memory:";
const TEST_TIMEOUT_SECS: u64 = 60;

// ============================================================================
// FASE 1: CRUD + METRICS TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn tc_001_policy_store_crud_lifecycle() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    let test_description = format!("Test Store - {}", Utc::now().timestamp());

    // CREATE
    let store = client
        .create_policy_store(Some(test_description.clone()))
        .await
        .expect("Failed to create policy store");

    assert!(!store.policy_store_id.is_empty());
    assert_eq!(store.description, Some(test_description.clone()));

    // READ (Get by ID)
    let retrieved = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get policy store");

    assert_eq!(retrieved.policy_store_id, store.policy_store_id);
    assert_eq!(retrieved.description, Some(test_description.clone()));

    // UPDATE
    let updated_description = format!("Updated - {}", Utc::now().timestamp());
    let _ = client
        .update_policy_store(&store.policy_store_id, Some(updated_description.clone()))
        .await
        .expect("Failed to update policy store");

    let updated = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get updated policy store");

    assert_eq!(updated.description, Some(updated_description.clone()));

    // DELETE
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete policy store");

    // Verify deletion
    let result = client.get_policy_store(&store.policy_store_id).await;
    assert!(result.is_err(), "Policy store should not exist after deletion");
}

#[tokio::test]
#[ignore]
async fn tc_002_policy_store_metrics_real_data() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Metrics Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    // Verify metrics show 0 policies and 0 schemas initially
    let details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get policy store details");

    assert_eq!(details.metrics.policies, 0);
    assert_eq!(details.metrics.schemas, 0);
    assert_eq!(details.metrics.status, "active");
    assert!(details.metrics.version.len() > 0);
    assert!(details.metrics.author.len() > 0);

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_003_policy_store_pagination() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect");

    // Create 10 stores
    let mut store_ids = Vec::new();
    for i in 0..10 {
        let store = client
            .create_policy_store(Some(format!("Store {}", i)))
            .await
            .unwrap();
        store_ids.push(store.policy_store_id);
    }

    // Test pagination with page size 3
    let page1 = client.list_policy_stores(Some(3), None).await.unwrap();
    assert!(page1.policy_stores.len() <= 3);
    assert!(!page1.policy_stores.is_empty());

    // Test second page if available
    if let Some(next_token) = page1.next_token {
        let page2 = client.list_policy_stores(Some(3), Some(next_token)).await.unwrap();
        assert!(!page2.policy_stores.is_empty());

        // Ensure no duplicates between pages
        let page1_ids: std::collections::HashSet<String> =
            page1.policy_stores.iter().map(|s| s.policy_store_id.clone()).collect();
        let page2_ids: std::collections::HashSet<String> =
            page2.policy_stores.iter().map(|s| s.policy_store_id.clone()).collect();

        assert!(page1_ids.is_disjoint(&page2_ids));
    }

    // Cleanup
    for store_id in store_ids {
        client.delete_policy_store(&store_id).await.unwrap();
    }
}

// ============================================================================
// FASE 2: AUDIT TRAIL TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn tc_010_audit_log_policy_store_operations() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Audit Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    // Update store
    let _ = client
        .update_policy_store(&store.policy_store_id, Some("Updated Audit Test".to_string()))
        .await
        .expect("Failed to update policy store");

    // Get audit log
    let audit_logs = client
        .get_policy_store_audit_log(&store.policy_store_id)
        .await
        .expect("Failed to get audit log");

    assert!(audit_logs.audit_logs.len() >= 1);

    // Verify audit log structure
    if let Some(first_log) = audit_logs.audit_logs.first() {
        assert!(first_log.id > 0);
        assert_eq!(first_log.policy_store_id, store.policy_store_id);
        assert!(["CREATE", "UPDATE", "DELETE"].contains(&first_log.action.as_str()));
        assert!(first_log.user_id.len() > 0);
        assert!(first_log.timestamp.len() > 0);
    }

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_011_audit_log_user_tracking() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create store and perform operations
    let store = client
        .create_policy_store(Some("User Tracking Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Get audit log and verify user tracking
    let audit_logs = client
        .get_policy_store_audit_log(&store.policy_store_id)
        .await
        .expect("Failed to get audit log");

    if let Some(create_log) = audit_logs.audit_logs.first() {
        assert!(create_log.user_id.len() > 0);
        assert!(create_log.action == "CREATE");
        assert!(create_log.timestamp.len() > 0);
        assert!(create_log.ip_address.is_some()); // IP should be recorded
    }

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

// ============================================================================
// FASE 3: TAGS AND FILTERING TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn tc_020_tags_add_remove_update() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Tags Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    // Add tags
    client
        .add_policy_store_tag(&store.policy_store_id, "production".to_string())
        .await
        .expect("Failed to add tag");

    client
        .add_policy_store_tag(&store.policy_store_id, "frontend".to_string())
        .await
        .expect("Failed to add second tag");

    client
        .add_policy_store_tag(&store.policy_store_id, "critical".to_string())
        .await
        .expect("Failed to add third tag");

    // Verify tags were added
    let tags = client
        .get_policy_store_tags(&store.policy_store_id)
        .await
        .expect("Failed to get tags");

    assert_eq!(tags.tags.len(), 3);
    assert!(tags.tags.contains(&"production".to_string()));
    assert!(tags.tags.contains(&"frontend".to_string()));
    assert!(tags.tags.contains(&"critical".to_string()));

    // Remove a tag
    client
        .remove_policy_store_tag(&store.policy_store_id, "critical".to_string())
        .await
        .expect("Failed to remove tag");

    // Verify tag was removed
    let tags_after_remove = client
        .get_policy_store_tags(&store.policy_store_id)
        .await
        .expect("Failed to get tags after removal");

    assert_eq!(tags_after_remove.tags.len(), 2);
    assert!(!tags_after_remove.tags.contains(&"critical".to_string()));

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_021_tag_autocomplete() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create stores with common tags
    let store1 = client
        .create_policy_store(Some("Store 1".to_string()))
        .await
        .unwrap();
    let store2 = client
        .create_policy_store(Some("Store 2".to_string()))
        .await
        .unwrap();

    client
        .add_policy_store_tag(&store1.policy_store_id, "production".to_string())
        .await
        .unwrap();
    client
        .add_policy_store_tag(&store2.policy_store_id, "production".to_string())
        .await
        .unwrap();
    client
        .add_policy_store_tag(&store1.policy_store_id, "testing".to_string())
        .await
        .unwrap();

    // Get all tags (for autocomplete)
    let all_tags = client
        .list_all_policy_store_tags()
        .await
        .expect("Failed to get all tags");

    assert!(all_tags.tags.contains(&"production".to_string()));
    assert!(all_tags.tags.contains(&"testing".to_string()));

    // Cleanup
    client.delete_policy_store(&store1.policy_store_id).await.unwrap();
    client.delete_policy_store(&store2.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_022_filter_by_tags_and_status() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create stores with different tags and statuses
    let store1 = client
        .create_policy_store(Some("Active Store 1".to_string()))
        .await
        .unwrap();
    let store2 = client
        .create_policy_store(Some("Active Store 2".to_string()))
        .await
        .unwrap();

    client
        .add_policy_store_tag(&store1.policy_store_id, "production".to_string())
        .await
        .unwrap();
    client
        .add_policy_store_tag(&store2.policy_store_id, "testing".to_string())
        .await
        .unwrap();

    // List stores and verify filtering works
    let stores = client
        .list_policy_stores_with_filters(None, None, Some(vec!["production".to_string()]))
        .await
        .expect("Failed to filter by tags");

    assert!(stores.policy_stores.len() >= 1);

    // Cleanup
    client.delete_policy_store(&store1.policy_store_id).await.unwrap();
    client.delete_policy_store(&store2.policy_store_id).await.unwrap();
}

// ============================================================================
// FASE 3.1: SNAPSHOTS AND VERSIONING TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn tc_030_snapshot_create_list_get() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Snapshot Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    // Add some policies for snapshot
    let policy1 = hodei_permissions_sdk::PolicyDefinition {
        policy_id: "policy-1".to_string(),
        definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
            hodei_permissions_sdk::StaticPolicyDefinition {
                statement: "permit(principal, action, resource)".to_string(),
            },
        )),
    };

    let _ = client
        .create_policy(&store.policy_store_id, policy1)
        .await
        .expect("Failed to create policy");

    // Create snapshot
    let snapshot = client
        .create_policy_store_snapshot(&store.policy_store_id, Some("Initial snapshot".to_string()))
        .await
        .expect("Failed to create snapshot");

    assert!(!snapshot.snapshot_id.is_empty());
    assert_eq!(snapshot.policy_store_id, store.policy_store_id);

    // List snapshots
    let snapshots = client
        .list_policy_store_snapshots(&store.policy_store_id)
        .await
        .expect("Failed to list snapshots");

    assert!(snapshots.snapshots.len() >= 1);
    assert!(snapshots.snapshots.iter().any(|s| s.snapshot_id == snapshot.snapshot_id));

    // Get specific snapshot
    let retrieved_snapshot = client
        .get_policy_store_snapshot(&store.policy_store_id, &snapshot.snapshot_id)
        .await
        .expect("Failed to get snapshot");

    assert_eq!(retrieved_snapshot.snapshot_id, snapshot.snapshot_id);
    assert!(retrieved_snapshot.policies.len() >= 1);

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_031_snapshot_rollback() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Rollback Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    // Add initial policy
    let policy1 = hodei_permissions_sdk::PolicyDefinition {
        policy_id: "initial-policy".to_string(),
        definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
            hodei_permissions_sdk::StaticPolicyDefinition {
                statement: "permit(principal, action, resource)".to_string(),
            },
        )),
    };

    let _ = client
        .create_policy(&store.policy_store_id, policy1)
        .await
        .expect("Failed to create initial policy");

    // Create snapshot
    let snapshot = client
        .create_policy_store_snapshot(&store.policy_store_id, Some("Before changes".to_string()))
        .await
        .expect("Failed to create snapshot");

    // Add more policies
    let policy2 = hodei_permissions_sdk::PolicyDefinition {
        policy_id: "second-policy".to_string(),
        definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
            hodei_permissions_sdk::StaticPolicyDefinition {
                statement: "deny(principal, action, resource)".to_string(),
            },
        )),
    };

    let _ = client
        .create_policy(&store.policy_store_id, policy2)
        .await
        .expect("Failed to create second policy");

    // Verify 2 policies exist
    let policies_before_rollback = client
        .list_policies(&store.policy_store_id, None, None)
        .await
        .expect("Failed to list policies");
    assert_eq!(policies_before_rollback.policies.len(), 2);

    // Rollback to snapshot
    let rollback_result = client
        .rollback_to_snapshot(&store.policy_store_id, &snapshot.snapshot_id, Some("Rollback test".to_string()))
        .await
        .expect("Failed to rollback to snapshot");

    assert_eq!(rollback_result.policy_store_id, store.policy_store_id);
    assert_eq!(rollback_result.policies_restored, 1);

    // Verify only 1 policy exists after rollback
    let policies_after_rollback = client
        .list_policies(&store.policy_store_id, None, None)
        .await
        .expect("Failed to list policies after rollback");
    assert_eq!(policies_after_rollback.policies.len(), 1);

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_032_snapshot_delete() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Delete Snapshot Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Create two snapshots
    let snapshot1 = client
        .create_policy_store_snapshot(&store.policy_store_id, Some("Snapshot 1".to_string()))
        .await
        .expect("Failed to create snapshot 1");

    let snapshot2 = client
        .create_policy_store_snapshot(&store.policy_store_id, Some("Snapshot 2".to_string()))
        .await
        .expect("Failed to create snapshot 2");

    // Verify both exist
    let snapshots_before = client
        .list_policy_store_snapshots(&store.policy_store_id)
        .await
        .expect("Failed to list snapshots");
    assert_eq!(snapshots_before.snapshots.len(), 2);

    // Delete one snapshot
    client
        .delete_snapshot(&store.policy_store_id, &snapshot1.snapshot_id)
        .await
        .expect("Failed to delete snapshot");

    // Verify only one remains
    let snapshots_after = client
        .list_policy_store_snapshots(&store.policy_store_id)
        .await
        .expect("Failed to list snapshots after deletion");
    assert_eq!(snapshots_after.snapshots.len(), 1);
    assert!(!snapshots_after.snapshots.iter().any(|s| s.snapshot_id == snapshot1.snapshot_id));

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

// ============================================================================
// FASE 3.1: BATCH OPERATIONS TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn tc_040_batch_create_policies() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Batch Create Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Prepare batch create request
    let policies = vec![
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "batch-policy-1".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "permit(principal, action, resource)".to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "batch-policy-2".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "deny(principal, action, resource)".to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "batch-policy-3".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "permit(principal == User::\"alice\", action, resource)".to_string(),
                },
            )),
        },
    ];

    // Execute batch create
    let result = client
        .batch_create_policies(&store.policy_store_id, policies)
        .await
        .expect("Failed to batch create policies");

    assert_eq!(result.results.len(), 3);
    assert_eq!(result.errors.len(), 0);

    // Verify all policies were created
    let created_policies = result.results;
    assert_eq!(created_policies.len(), 3);

    // Verify policies exist in store
    let listed_policies = client
        .list_policies(&store.policy_store_id, None, None)
        .await
        .expect("Failed to list policies");
    assert_eq!(listed_policies.policies.len(), 3);

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_041_batch_update_policies() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Batch Update Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Create initial policies
    let policies = vec![
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "update-policy-1".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "permit(principal, action, resource)".to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "update-policy-2".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "deny(principal, action, resource)".to_string(),
                },
            )),
        },
    ];

    let _ = client
        .batch_create_policies(&store.policy_store_id, policies)
        .await
        .expect("Failed to batch create policies");

    // Prepare batch update request
    let updates = vec![
        hodei_permissions_sdk::BatchPolicyUpdate {
            policy_id: "update-policy-1".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "permit(principal == User::\"bob\", action, resource)".to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyUpdate {
            policy_id: "update-policy-2".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "deny(principal == User::\"charlie\", action, resource)".to_string(),
                },
            )),
        },
    ];

    // Execute batch update
    let result = client
        .batch_update_policies(&store.policy_store_id, updates)
        .await
        .expect("Failed to batch update policies");

    assert_eq!(result.results.len(), 2);
    assert_eq!(result.errors.len(), 0);

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_042_batch_delete_policies() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Batch Delete Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Create initial policies
    let policies = vec![
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "delete-policy-1".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "permit(principal, action, resource)".to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "delete-policy-2".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "deny(principal, action, resource)".to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "delete-policy-3".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "permit(principal, action == Action::\"read\", resource)".to_string(),
                },
            )),
        },
    ];

    let _ = client
        .batch_create_policies(&store.policy_store_id, policies)
        .await
        .expect("Failed to batch create policies");

    // Verify 3 policies exist
    let policies_before = client
        .list_policies(&store.policy_store_id, None, None)
        .await
        .expect("Failed to list policies");
    assert_eq!(policies_before.policies.len(), 3);

    // Prepare batch delete request (delete 2 policies)
    let policy_ids = vec![
        "delete-policy-1".to_string(),
        "delete-policy-3".to_string(),
    ];

    // Execute batch delete
    let result = client
        .batch_delete_policies(&store.policy_store_id, policy_ids)
        .await
        .expect("Failed to batch delete policies");

    assert_eq!(result.results.len(), 2);
    assert_eq!(result.errors.len(), 0);

    // Verify only 1 policy remains
    let policies_after = client
        .list_policies(&store.policy_store_id, None, None)
        .await
        .expect("Failed to list policies after deletion");
    assert_eq!(policies_after.policies.len(), 1);
    assert_eq!(policies_after.policies[0].policy_id, "delete-policy-2");

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_043_batch_operations_error_handling() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Batch Error Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Create one valid policy
    let policies = vec![
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "valid-policy".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "permit(principal, action, resource)".to_string(),
                },
            )),
        },
        // Invalid: Missing definition
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "invalid-policy-1".to_string(),
            definition: None,
        },
        // Invalid: Empty statement
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "invalid-policy-2".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: "".to_string(),
                },
            )),
        },
    ];

    // Execute batch create (should have partial success)
    let result = client
        .batch_create_policies(&store.policy_store_id, policies)
        .await
        .expect("Failed to batch create policies");

    // Should have 1 success and 2 errors
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.errors.len(), 2);

    // Verify the valid policy was created
    assert_eq!(result.results[0].policy_id, "valid-policy");

    // Verify errors are recorded
    assert!(!result.errors[0].error.is_empty());

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

// ============================================================================
// FASE 3.1: AUTHORIZATION TESTING
// ============================================================================

#[tokio::test]
#[ignore]
async fn tc_050_authorization_basic() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Authorization Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Add a simple policy
    let policy = hodei_permissions_sdk::PolicyDefinition {
        policy_id: "allow-alice".to_string(),
        definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
            hodei_permissions_sdk::StaticPolicyDefinition {
                statement: r#"permit(principal == User::"alice", action, resource)"#.to_string(),
            },
        )),
    };

    let _ = client
        .create_policy(&store.policy_store_id, policy)
        .await
        .expect("Failed to create policy");

    // Test authorization - should ALLOW
    let auth_request = hodei_permissions_sdk::IsAuthorizedRequest {
        policy_store_id: store.policy_store_id.clone(),
        principal: Some(hodei_permissions_sdk::Entity {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(hodei_permissions_sdk::Entity {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(hodei_permissions_sdk::Entity {
            entity_type: "Document".to_string(),
            entity_id: "doc1".to_string(),
        }),
        context: "{}".to_string(),
        entities: vec![],
    };

    let auth_response = client
        .is_authorized(&store.policy_store_id, auth_request)
        .await
        .expect("Failed to check authorization");

    assert_eq!(auth_response.decision, "ALLOW");

    // Test authorization for different user - should DENY
    let auth_request_deny = hodei_permissions_sdk::IsAuthorizedRequest {
        policy_store_id: store.policy_store_id.clone(),
        principal: Some(hodei_permissions_sdk::Entity {
            entity_type: "User".to_string(),
            entity_id: "bob".to_string(),
        }),
        action: Some(hodei_permissions_sdk::Entity {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(hodei_permissions_sdk::Entity {
            entity_type: "Document".to_string(),
            entity_id: "doc1".to_string(),
        }),
        context: "{}".to_string(),
        entities: vec![],
    };

    let auth_response_deny = client
        .is_authorized(&store.policy_store_id, auth_request_deny)
        .await
        .expect("Failed to check authorization for bob");

    assert_eq!(auth_response_deny.decision, "DENY");

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn tc_051_authorization_with_context() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // Create policy store
    let store = client
        .create_policy_store(Some("Context Authorization Test".to_string()))
        .await
        .expect("Failed to create policy store");

    // Add policy with context
    let policy = hodei_permissions_sdk::PolicyDefinition {
        policy_id: "context-policy".to_string(),
        definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
            hodei_permissions_sdk::StaticPolicyDefinition {
                statement: r#"permit(principal, action, resource) when { context.time >= "09:00" && context.time <= "17:00" }"#.to_string(),
            },
        )),
    };

    let _ = client
        .create_policy(&store.policy_store_id, policy)
        .await
        .expect("Failed to create policy");

    // Test authorization with valid context
    let auth_request = hodei_permissions_sdk::IsAuthorizedRequest {
        policy_store_id: store.policy_store_id.clone(),
        principal: Some(hodei_permissions_sdk::Entity {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(hodei_permissions_sdk::Entity {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(hodei_permissions_sdk::Entity {
            entity_type: "Document".to_string(),
            entity_id: "doc1".to_string(),
        }),
        context: r#"{"time": "14:00"}"#.to_string(),
        entities: vec![],
    };

    let auth_response = client
        .is_authorized(&store.policy_store_id, auth_request)
        .await
        .expect("Failed to check authorization with context");

    assert_eq!(auth_response.decision, "ALLOW");

    // Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn tc_100_integration_all_features() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    // 1. Create policy store with tags
    let store = client
        .create_policy_store(Some("Integration Test Store".to_string()))
        .await
        .expect("Failed to create policy store");

    client
        .add_policy_store_tag(&store.policy_store_id, "integration".to_string())
        .await
        .expect("Failed to add tag");

    // 2. Create multiple policies via batch
    let policies = vec![
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "policy-1".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: r#"permit(principal == User::"alice", action, resource)"#.to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyDefinition {
            policy_id: "policy-2".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: r#"deny(principal == User::"bob", action == Action::"delete", resource)"#.to_string(),
                },
            )),
        },
    ];

    let _ = client
        .batch_create_policies(&store.policy_store_id, policies)
        .await
        .expect("Failed to batch create policies");

    // 3. Create snapshot
    let snapshot = client
        .create_policy_store_snapshot(&store.policy_store_id, Some("Integration test snapshot".to_string()))
        .await
        .expect("Failed to create snapshot");

    // 4. Test authorization
    let auth_request = hodei_permissions_sdk::IsAuthorizedRequest {
        policy_store_id: store.policy_store_id.clone(),
        principal: Some(hodei_permissions_sdk::Entity {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(hodei_permissions_sdk::Entity {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(hodei_permissions_sdk::Entity {
            entity_type: "Document".to_string(),
            entity_id: "doc1".to_string(),
        }),
        context: "{}".to_string(),
        entities: vec![],
    };

    let auth_response = client
        .is_authorized(&store.policy_store_id, auth_request)
        .await
        .expect("Failed to check authorization");
    assert_eq!(auth_response.decision, "ALLOW");

    // 5. Verify all metrics
    let details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get policy store details");

    assert_eq!(details.metrics.policies, 2);
    assert_eq!(details.metrics.tags.len(), 1);
    assert!(details.metrics.tags.contains(&"integration".to_string()));

    // 6. Verify audit log exists
    let audit_logs = client
        .get_policy_store_audit_log(&store.policy_store_id)
        .await
        .expect("Failed to get audit log");
    assert!(audit_logs.audit_logs.len() > 0);

    // 7. Cleanup
    client.delete_policy_store(&store.policy_store_id).await.unwrap();
}
