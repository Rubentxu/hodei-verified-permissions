#![cfg(feature = "containers")]
//! Full Integration Test - Complete Policy Store Workflow
//! Tests the entire lifecycle from creation to deletion with all features

mod testcontainers;

use testcontainers::clients::Cli;
use testcontainers::server_container::ServerContainer;
use chrono::Utc;

#[tokio::test]
#[ignore]
async fn integration_complete_workflow() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 Starting Complete Integration Test Workflow\n");

    // ============================================================================
    // PHASE 1: SETUP - Create Policy Store
    // ============================================================================
    println!("📦 PHASE 1: Creating Policy Store...");

    let store_description = format!("Integration Test Store - {}", Utc::now().timestamp());
    let store = client
        .create_policy_store(Some(store_description.clone()))
        .await
        .expect("Failed to create policy store");

    println!("  ✓ Created policy store: {}", store.policy_store_id);

    // Verify initial metrics
    let initial_details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get policy store details");

    assert_eq!(initial_details.metrics.policies, 0);
    assert_eq!(initial_details.metrics.schemas, 0);
    assert_eq!(initial_details.metrics.status, "active");
    println!("  ✓ Verified initial metrics");

    // ============================================================================
    // PHASE 2: TAGS - Add and manage tags
    // ============================================================================
    println!("\n🏷️  PHASE 2: Managing Tags...");

    let tags = vec!["production", "frontend", "critical", "api"];

    for tag in &tags {
        client
            .add_policy_store_tag(&store.policy_store_id, tag.to_string())
            .await
            .expect("Failed to add tag");

        println!("  ✓ Added tag: {}", tag);
    }

    // Verify tags
    let store_tags = client
        .get_policy_store_tags(&store.policy_store_id)
        .await
        .expect("Failed to get tags");

    assert_eq!(store_tags.tags.len(), 4);
    for tag in &tags {
        assert!(store_tags.tags.contains(&tag.to_string()));
    }
    println!("  ✓ Verified all tags were added");

    // Remove one tag
    client
        .remove_policy_store_tag(&store.policy_store_id, "critical".to_string())
        .await
        .expect("Failed to remove tag");

    let tags_after_remove = client
        .get_policy_store_tags(&store.policy_store_id)
        .await
        .expect("Failed to get tags after removal");

    assert_eq!(tags_after_remove.tags.len(), 3);
    assert!(!tags_after_remove.tags.contains(&"critical".to_string()));
    println!("  ✓ Verified tag removal");

    // ============================================================================
    // PHASE 3: POLICIES - Create multiple policies
    // ============================================================================
    println!("\n📜 PHASE 3: Creating Policies...");

    let policies = vec![
        ("policy-1", r#"permit(principal == User::"alice", action, resource)"#),
        ("policy-2", r#"deny(principal == User::"bob", action == Action::"delete", resource)"#),
        ("policy-3", r#"permit(principal == User::"charlie", action == Action::"read", resource)"#),
    ];

    for (policy_id, statement) in &policies {
        let policy = hodei_permissions_sdk::PolicyDefinition {
            policy_id: policy_id.to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: statement.to_string(),
                },
            )),
        };

        client
            .create_policy(&store.policy_store_id, policy)
            .await
            .expect("Failed to create policy");

        println!("  ✓ Created policy: {}", policy_id);
    }

    // Verify policy count
    let policies_list = client
        .list_policies(&store.policy_store_id, None, None)
        .await
        .expect("Failed to list policies");

    assert_eq!(policies_list.policies.len(), 3);
    println!("  ✓ Verified all policies were created");

    // ============================================================================
    // PHASE 4: SNAPSHOTS - Create multiple snapshots
    // ============================================================================
    println!("\n📸 PHASE 4: Creating Snapshots...");

    let snapshots = vec![
        "Initial snapshot with 3 policies",
        "Snapshot after policy modifications",
        "Final production snapshot",
    ];

    let mut snapshot_ids = Vec::new();

    for description in &snapshots {
        let snapshot = client
            .create_policy_store_snapshot(&store.policy_store_id, Some(description.to_string()))
            .await
            .expect("Failed to create snapshot");

        snapshot_ids.push(snapshot.snapshot_id.clone());
        println!("  ✓ Created snapshot: {}", description);
    }

    // Verify snapshots
    let snapshots_list = client
        .list_policy_store_snapshots(&store.policy_store_id)
        .await
        .expect("Failed to list snapshots");

    assert_eq!(snapshots_list.snapshots.len(), 3);
    println!("  ✓ Verified all snapshots were created");

    // Verify snapshot metrics
    for snapshot in &snapshots_list.snapshots {
        assert_eq!(snapshot.policy_count, 3);
        assert!(snapshot.size_bytes > 0);
    }
    println!("  ✓ Verified snapshot metrics");

    // ============================================================================
    // PHASE 5: BATCH OPERATIONS - Test batch updates
    // ============================================================================
    println!("\n⚡ PHASE 5: Batch Operations...");

    // Prepare batch update
    let updates = vec![
        hodei_permissions_sdk::BatchPolicyUpdate {
            policy_id: "policy-1".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: r#"permit(principal == User::"alice", action == Action::"read", resource)"#.to_string(),
                },
            )),
        },
        hodei_permissions_sdk::BatchPolicyUpdate {
            policy_id: "policy-3".to_string(),
            definition: Some(hodei_permissions_sdk::policy_definition::PolicyType::Static(
                hodei_permissions_sdk::StaticPolicyDefinition {
                    statement: r#"permit(principal == User::"charlie", action, resource)"#.to_string(),
                },
            )),
        },
    ];

    let batch_result = client
        .batch_update_policies(&store.policy_store_id, updates)
        .await
        .expect("Failed to batch update policies");

    assert_eq!(batch_result.results.len(), 2);
    assert_eq!(batch_result.errors.len(), 0);
    println!("  ✓ Completed batch update (2 policies)");

    // ============================================================================
    // PHASE 6: ROLLBACK - Test rollback to snapshot
    // ============================================================================
    println!("\n🔄 PHASE 6: Rollback to Snapshot...");

    // Rollback to first snapshot
    let rollback_result = client
        .rollback_to_snapshot(&store.policy_store_id, &snapshot_ids[0], Some("Integration test rollback".to_string()))
        .await
        .expect("Failed to rollback to snapshot");

    assert_eq!(rollback_result.policies_restored, 3);
    println!("  ✓ Rolled back to first snapshot");

    // Verify policies were restored to original state
    let policies_after_rollback = client
        .list_policies(&store.policy_store_id, None, None)
        .await
        .expect("Failed to list policies after rollback");

    assert_eq!(policies_after_rollback.policies.len(), 3);
    println!("  ✓ Verified policies were restored");

    // ============================================================================
    // PHASE 7: AUTHORIZATION - Test authorization decisions
    // ============================================================================
    println!("\n🔐 PHASE 7: Testing Authorization...");

    // Test ALLOW decision
    let auth_request_allow = hodei_permissions_sdk::IsAuthorizedRequest {
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

    let auth_response_allow = client
        .is_authorized(&store.policy_store_id, auth_request_allow)
        .await
        .expect("Failed to check authorization");

    assert_eq!(auth_response_allow.decision, "ALLOW");
    println!("  ✓ ALLOW decision for alice/read");

    // Test DENY decision
    let auth_request_deny = hodei_permissions_sdk::IsAuthorizedRequest {
        policy_store_id: store.policy_store_id.clone(),
        principal: Some(hodei_permissions_sdk::Entity {
            entity_type: "User".to_string(),
            entity_id: "bob".to_string(),
        }),
        action: Some(hodei_permissions_sdk::Entity {
            entity_type: "Action".to_string(),
            entity_id: "delete".to_string(),
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
        .expect("Failed to check authorization");

    assert_eq!(auth_response_deny.decision, "DENY");
    println!("  ✓ DENY decision for bob/delete");

    // ============================================================================
    // PHASE 8: AUDIT LOG - Verify audit trail
    // ============================================================================
    println!("\n📊 PHASE 8: Verifying Audit Log...");

    let audit_logs = client
        .get_policy_store_audit_log(&store.policy_store_id)
        .await
        .expect("Failed to get audit log");

    // Should have multiple audit entries
    assert!(audit_logs.audit_logs.len() > 0);
    println!("  ✓ Found {} audit log entries", audit_logs.audit_logs.len());

    // Verify audit log structure
    for log in &audit_logs.audit_logs {
        assert!(log.id > 0);
        assert_eq!(log.policy_store_id, store.policy_store_id);
        assert!(["CREATE", "UPDATE", "DELETE"].contains(&log.action.as_str()));
        assert!(log.user_id.len() > 0);
        assert!(log.timestamp.len() > 0);
    }
    println!("  ✓ Verified audit log structure");

    // ============================================================================
    // PHASE 9: METRICS - Verify all metrics are correct
    // ============================================================================
    println!("\n📈 PHASE 9: Verifying Final Metrics...");

    let final_details = client
        .get_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to get final policy store details");

    assert_eq!(final_details.metrics.policies, 3);
    assert_eq!(final_details.metrics.schemas, 0);
    assert_eq!(final_details.metrics.status, "active");
    assert_eq!(final_details.metrics.tags.len(), 3);
    assert!(final_details.metrics.tags.contains(&"production".to_string()));
    assert!(final_details.metrics.tags.contains(&"frontend".to_string()));
    assert!(final_details.metrics.tags.contains(&"api".to_string()));

    println!("  ✓ Policies: {}", final_details.metrics.policies);
    println!("  ✓ Schemas: {}", final_details.metrics.schemas);
    println!("  ✓ Status: {}", final_details.metrics.status);
    println!("  ✓ Tags: {:?}", final_details.metrics.tags);

    // ============================================================================
    // PHASE 10: CLEANUP - Delete snapshot and policy store
    // ============================================================================
    println!("\n🧹 PHASE 10: Cleanup...");

    // Delete one snapshot
    client
        .delete_snapshot(&store.policy_store_id, &snapshot_ids[1])
        .await
        .expect("Failed to delete snapshot");

    let snapshots_after_delete = client
        .list_policy_store_snapshots(&store.policy_store_id)
        .await
        .expect("Failed to list snapshots after delete");

    assert_eq!(snapshots_after_delete.snapshots.len(), 2);
    println!("  ✓ Deleted snapshot");

    // Delete policy store
    client
        .delete_policy_store(&store.policy_store_id)
        .await
        .expect("Failed to delete policy store");

    // Verify deletion
    let result = client.get_policy_store(&store.policy_store_id).await;
    assert!(result.is_err(), "Policy store should not exist after deletion");
    println!("  ✓ Deleted policy store");

    // ============================================================================
    // FINAL SUMMARY
    // ============================================================================
    println!("\n" & "");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Integration Test Complete!                                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("\n✅ All phases completed successfully:\n");
    println!("  1. ✅ Policy Store Creation");
    println!("  2. ✅ Tag Management (add/remove)");
    println!("  3. ✅ Policy Creation (3 policies)");
    println!("  4. ✅ Snapshot Creation (3 snapshots)");
    println!("  5. ✅ Batch Operations (update)");
    println!("  6. ✅ Rollback to Snapshot");
    println!("  7. ✅ Authorization Testing (ALLOW/DENY)");
    println!("  8. ✅ Audit Log Verification");
    println!("  9. ✅ Metrics Verification");
    println!(" 10. ✅ Cleanup (delete)");
    println!("\n🎉 Integration test passed!\n");
}

#[tokio::test]
#[ignore]
async fn integration_stress_test() {
    let docker = Cli::default();
    let server = ServerContainer::start(&docker).await;
    let client = hodei_permissions_sdk::AuthorizationClient::connect(server.grpc_url())
        .await
        .expect("Failed to connect to server");

    println!("\n🚀 Starting Stress Test\n");

    // Create multiple policy stores concurrently
    let num_stores = 20;
    let mut handles = vec![];

    for i in 0..num_stores {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let description = format!("Stress Test Store {}", i);
            let store = client_clone
                .create_policy_store(Some(description))
                .await
                .expect("Failed to create store");

            // Add tags
            client_clone
                .add_policy_store_tag(&store.policy_store_id, format!("tag-{}", i))
                .await
                .expect("Failed to add tag");

            // Create snapshot
            client_clone
                .create_policy_store_snapshot(&store.policy_store_id, Some(format!("Snapshot {}", i)))
                .await
                .expect("Failed to create snapshot");

            // Cleanup
            client_clone
                .delete_policy_store(&store.policy_store_id)
                .await
                .expect("Failed to delete store");

            store.policy_store_id
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    println!("✅ Stress test completed: {} stores created and deleted", num_stores);
}
