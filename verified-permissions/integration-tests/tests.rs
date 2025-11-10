//! Integration tests for verified-permissions
//!
//! These tests require a running database and gRPC server.

use chrono::Utc;
use hodei_domain::events::{InMemoryEventBus, PolicyStoreCreated, SqliteEventStore};
use hodei_domain::{
    AuditLogEntry, AuditLogFilters, CedarPolicy, PolicyStoreId, events::EventDispatcher,
};
use hodei_infrastructure::repository::RepositoryAdapter;
use std::sync::Arc;
use uuid::Uuid;

const TEST_DATABASE_URL: &str = "sqlite::memory:"; // Use in-memory database for tests

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repository_creation() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL).await;
        assert!(repository.is_ok());
    }

    #[tokio::test]
    async fn test_create_policy_store_integration() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store
        let store = repository
            .create_policy_store(
                "Test Store".to_string(),
                Some("Integration test store".to_string()),
            )
            .await
            .expect("Failed to create policy store");

        assert_eq!(store.name, "Test Store");
        assert_eq!(
            store.description,
            Some("Integration test store".to_string())
        );
        assert!(!store.id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_get_policy_store_integration() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store
        let created_store = repository
            .create_policy_store(
                "Get Test Store".to_string(),
                Some("Get integration test".to_string()),
            )
            .await
            .expect("Failed to create policy store");

        // Get the policy store
        let retrieved_store = repository
            .get_policy_store(&created_store.id)
            .await
            .expect("Failed to get policy store");

        assert_eq!(retrieved_store.id, created_store.id);
        assert_eq!(retrieved_store.name, created_store.name);
        assert_eq!(retrieved_store.description, created_store.description);
    }

    #[tokio::test]
    async fn test_list_policy_stores_integration() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create multiple policy stores
        for i in 0..5 {
            let _ = repository
                .create_policy_store(
                    format!("List Test Store {}", i),
                    Some(format!("List integration test {}", i)),
                )
                .await
                .expect("Failed to create policy store");
        }

        // List all policy stores
        let stores = repository
            .list_policy_stores()
            .await
            .expect("Failed to list policy stores");

        assert_eq!(stores.len(), 5);

        // Verify all stores have correct names
        for (i, store) in stores.iter().enumerate() {
            assert_eq!(store.name, format!("List Test Store {}", i));
        }
    }

    #[tokio::test]
    async fn test_update_policy_store_integration() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store
        let mut store = repository
            .create_policy_store(
                "Update Test Store".to_string(),
                Some("Original description".to_string()),
            )
            .await
            .expect("Failed to create policy store");

        // Update the policy store
        let updated_store = repository
            .update_policy_store(
                &store.id,
                Some("Updated Store Name".to_string()),
                Some("Updated description".to_string()),
            )
            .await
            .expect("Failed to update policy store");

        assert_eq!(updated_store.name, "Updated Store Name");
        assert_eq!(
            updated_store.description,
            Some("Updated description".to_string())
        );
    }

    #[tokio::test]
    async fn test_delete_policy_store_integration() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store
        let store = repository
            .create_policy_store(
                "Delete Test Store".to_string(),
                Some("Delete integration test".to_string()),
            )
            .await
            .expect("Failed to create policy store");

        // Delete the policy store
        let result = repository
            .delete_policy_store(&store.id)
            .await
            .expect("Failed to delete policy store");

        assert!(result.is_ok());

        // Try to get the deleted store (should fail)
        let retrieved = repository.get_policy_store(&store.id).await;

        assert!(retrieved.is_err());
    }

    #[tokio::test]
    async fn test_event_dispatcher_integration() {
        // Create event infrastructure
        let event_bus = InMemoryEventBus::new();
        let event_store = SqliteEventStore::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create event store");

        let dispatcher = Arc::new(EventDispatcher::new(event_bus, event_store));

        // Create a test event
        let event = PolicyStoreCreated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_event_test_123".to_string(),
            name: "Event Test Store".to_string(),
            description: Some("Event integration test".to_string()),
            author: "test_integration".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        // Dispatch the event
        let result = dispatcher
            .dispatch(
                &event.aggregate_id(),
                vec![Box::new(event) as Box<dyn hodei_domain::events::DomainEvent>],
                0,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_log_filters_integration() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store (this will generate an event)
        let store = repository
            .create_policy_store(
                "Audit Test Store".to_string(),
                Some("Audit integration test".to_string()),
            )
            .await
            .expect("Failed to create policy store");

        // Create audit log filters
        let filters = AuditLogFilters {
            event_types: Some(vec!["PolicyStoreCreated".to_string()]),
            service_name: None,
            policy_store_id: Some(store.id.into_string()),
            start_date: None,
            end_date: None,
            limit: Some(10),
        };

        // Get audit log with filters
        let audit_logs = repository
            .get_audit_log(filters)
            .await
            .expect("Failed to get audit log");

        // Should have at least one event (the create event)
        assert!(!audit_logs.is_empty());

        // Verify the event is the policy store created event
        let event = &audit_logs[0];
        assert_eq!(event.event_type, "PolicyStoreCreated");
        assert_eq!(event.aggregate_id, store.id.into_string());
    }

    #[tokio::test]
    async fn test_multiple_operations_audit_trail() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Perform multiple operations
        let store1 = repository
            .create_policy_store("Store 1".to_string(), Some("First store".to_string()))
            .await
            .expect("Failed to create store 1");

        let store2 = repository
            .create_policy_store("Store 2".to_string(), Some("Second store".to_string()))
            .await
            .expect("Failed to create store 2");

        // Update store 1
        let _ = repository
            .update_policy_store(&store1.id, Some("Updated Store 1".to_string()), None)
            .await
            .expect("Failed to update store 1");

        // Get all audit logs
        let filters = AuditLogFilters {
            event_types: None,
            service_name: None,
            policy_store_id: None,
            start_date: None,
            end_date: None,
            limit: Some(100),
        };

        let audit_logs = repository
            .get_audit_log(filters)
            .await
            .expect("Failed to get audit log");

        // Should have at least 3 events: 2 creates + 1 update
        assert!(audit_logs.len() >= 3);

        // Verify event types
        let event_types: Vec<&str> = audit_logs.iter().map(|e| e.event_type.as_str()).collect();

        assert!(event_types.contains(&"PolicyStoreCreated"));
        assert!(event_types.contains(&"PolicyStoreUpdated"));
    }

    #[tokio::test]
    async fn test_audit_log_date_filtering() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store
        let store = repository
            .create_policy_store("Date Filter Test".to_string(), None)
            .await
            .expect("Failed to create policy store");

        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);

        // Filter events from one hour ago
        let filters = AuditLogFilters {
            event_types: None,
            service_name: None,
            policy_store_id: Some(store.id.into_string()),
            start_date: Some(one_hour_ago),
            end_date: Some(now),
            limit: Some(10),
        };

        let audit_logs = repository
            .get_audit_log(filters)
            .await
            .expect("Failed to get filtered audit log");

        // Should have the create event within the time range
        assert!(!audit_logs.is_empty());

        // Verify all events are within the time range
        for event in &audit_logs {
            assert!(event.occurred_at >= one_hour_ago);
            assert!(event.occurred_at <= now);
        }
    }

    #[tokio::test]
    async fn test_cedar_policy_integration() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store
        let store = repository
            .create_policy_store(
                "Policy Test Store".to_string(),
                Some("Policy integration test".to_string()),
            )
            .await
            .expect("Failed to create policy store");

        // Create a Cedar policy
        let policy_statement = r#"
            permit(
                principal,
                action in [Action::"read", Action::"write"],
                resource
            );
        "#;

        let cedar_policy =
            CedarPolicy::new(policy_statement).expect("Failed to create Cedar policy");

        let policy_id =
            hodei_domain::PolicyId::new("policy_123").expect("Failed to create policy ID");

        // Create the policy in the store
        let policy = repository
            .create_policy(
                &store.id,
                &policy_id,
                &cedar_policy,
                Some("Test policy".to_string()),
            )
            .await
            .expect("Failed to create policy");

        assert_eq!(policy.policy_id, policy_id);
        assert_eq!(policy.statement.as_str(), policy_statement);
    }

    #[tokio::test]
    async fn test_schema_operations() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Create a policy store
        let store = repository
            .create_policy_store(
                "Schema Test Store".to_string(),
                Some("Schema integration test".to_string()),
            )
            .await
            .expect("Failed to create policy store");

        // Define a schema
        let schema_json = r#"
        {
            "entities": {
                "User": {
                    "shape": {
                        "type": "record",
                        "attributes": {
                            "id": {
                                "type": "String"
                            },
                            "role": {
                                "type": "String"
                            }
                        }
                    }
                }
            }
        }
        "#;

        // Put the schema
        let result = repository
            .put_schema(&store.id, schema_json.to_string())
            .await;
        assert!(result.is_ok());

        // Get the schema
        let retrieved_schema = repository
            .get_schema(&store.id)
            .await
            .expect("Failed to get schema")
            .expect("Schema not found");

        assert_eq!(retrieved_schema.schema_json, schema_json);
        assert_eq!(retrieved_schema.policy_store_id, store.id);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let repository = RepositoryAdapter::new(TEST_DATABASE_URL)
            .await
            .expect("Failed to create repository");

        // Try to get a non-existent policy store
        let fake_id = PolicyStoreId::new("ps_nonexistent_123").expect("Failed to create fake ID");

        let result = repository.get_policy_store(&fake_id).await;

        assert!(result.is_err());
    }
}
