//! Unit tests for domain events

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use hodei_domain::events::{
        AccessType, ApiCalled, ApiCompleted, AuthorizationDecision, AuthorizationPerformed,
        DomainEvent, PolicyStoreCreated, PolicyStoreDeleted, PolicyStoreTagsUpdated,
        PolicyStoreUpdated,
    };
    use uuid::Uuid;

    #[test]
    fn test_policy_store_created_event() {
        let event = PolicyStoreCreated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_test_123".to_string(),
            name: "Test Store".to_string(),
            description: Some("Test description".to_string()),
            author: "test_user".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "PolicyStoreCreated");
        assert_eq!(event.aggregate_id(), "ps_test_123");
        assert_eq!(event.version(), 1);
        assert!(!event.event_id().is_empty());
        assert_eq!(event.name, "Test Store");
        assert_eq!(event.author, "test_user");
    }

    #[test]
    fn test_policy_store_updated_event() {
        let event = PolicyStoreUpdated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_test_456".to_string(),
            name: Some("Updated Store".to_string()),
            description: Some("Updated description".to_string()),
            changed_by: "admin_user".to_string(),
            occurred_at: Utc::now(),
            version: 2,
        };

        assert_eq!(event.event_type(), "PolicyStoreUpdated");
        assert_eq!(event.aggregate_id(), "ps_test_456");
        assert_eq!(event.version(), 2);
        assert_eq!(event.name, Some("Updated Store".to_string()));
        assert_eq!(event.changed_by, "admin_user");
    }

    #[test]
    fn test_policy_store_tags_updated_event() {
        let old_tags = vec!["tag1".to_string(), "tag2".to_string()];
        let new_tags = vec!["tag1".to_string(), "tag3".to_string()];

        let event = PolicyStoreTagsUpdated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_test_789".to_string(),
            old_tags: old_tags.clone(),
            new_tags: new_tags.clone(),
            changed_by: "user123".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "PolicyStoreTagsUpdated");
        assert_eq!(event.aggregate_id(), "ps_test_789");
        assert_eq!(event.old_tags, old_tags);
        assert_eq!(event.new_tags, new_tags);
        assert_eq!(event.changed_by, "user123");
    }

    #[test]
    fn test_policy_store_deleted_event() {
        let event = PolicyStoreDeleted {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_test_delete".to_string(),
            deleted_by: "admin".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "PolicyStoreDeleted");
        assert_eq!(event.aggregate_id(), "ps_test_delete");
        assert_eq!(event.deleted_by, "admin");
    }

    #[test]
    fn test_api_called_event() {
        let event = ApiCalled {
            event_id: Uuid::new_v4().to_string(),
            service_name: "HodeiVerifiedPermissions".to_string(),
            method_name: "CreatePolicyStore".to_string(),
            client_ip: Some("192.168.1.100".to_string()),
            user_agent: Some("grpc-go/1.54.0".to_string()),
            request_id: Uuid::new_v4().to_string(),
            request_size_bytes: 1024,
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "ApiCalled");
        assert_eq!(event.service_name, "HodeiVerifiedPermissions");
        assert_eq!(event.method_name, "CreatePolicyStore");
        assert_eq!(event.client_ip, Some("192.168.1.100".to_string()));
        assert_eq!(event.request_size_bytes, 1024);
    }

    #[test]
    fn test_api_completed_event() {
        let event = ApiCompleted {
            event_id: Uuid::new_v4().to_string(),
            request_id: "req_123".to_string(),
            service_name: "HodeiVerifiedPermissions".to_string(),
            method_name: "CreatePolicyStore".to_string(),
            status_code: 0,
            error_message: None,
            response_size_bytes: 512,
            duration_ms: 150,
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "ApiCompleted");
        assert_eq!(event.request_id, "req_123");
        assert_eq!(event.status_code, 0);
        assert_eq!(event.error_message, None);
        assert_eq!(event.duration_ms, 150);
    }

    #[test]
    fn test_api_completed_with_error() {
        let event = ApiCompleted {
            event_id: Uuid::new_v4().to_string(),
            request_id: "req_error_456".to_string(),
            service_name: "HodeiVerifiedPermissions".to_string(),
            method_name: "GetPolicyStore".to_string(),
            status_code: 1,
            error_message: Some("Policy store not found".to_string()),
            response_size_bytes: 0,
            duration_ms: 200,
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "ApiCompleted");
        assert_eq!(event.status_code, 1);
        assert_eq!(
            event.error_message,
            Some("Policy store not found".to_string())
        );
        assert!(event.response_size_bytes == 0);
    }

    #[test]
    fn test_policy_store_accessed_event() {
        let event = PolicyStoreAccessed {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_access_123".to_string(),
            access_type: AccessType::READ,
            operation: "GetPolicyStore".to_string(),
            user_id: "user_readonly".to_string(),
            client_ip: Some("10.0.0.50".to_string()),
            user_agent: Some("grpc-go/1.54.0".to_string()),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "PolicyStoreAccessed");
        assert_eq!(event.aggregate_id(), "ps_access_123");
        assert_eq!(event.access_type, AccessType::READ);
        assert_eq!(event.operation, "GetPolicyStore");
        assert_eq!(event.user_id, "user_readonly");
    }

    #[test]
    fn test_policy_store_accessed_write() {
        let event = PolicyStoreAccessed {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_write_123".to_string(),
            access_type: AccessType::WRITE,
            operation: "UpdatePolicyStore".to_string(),
            user_id: "user_admin".to_string(),
            client_ip: Some("10.0.0.100".to_string()),
            user_agent: Some("grpc-go/1.54.0".to_string()),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.access_type, AccessType::WRITE);
        assert_eq!(event.user_id, "user_admin");
    }

    #[test]
    fn test_authorization_permitted_allow() {
        let event = AuthorizationPerformed {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_auth_123".to_string(),
            principal: "User::\"alice\"".to_string(),
            action: "Action::\"viewDocument\"".to_string(),
            resource: "Document::\"doc123\"".to_string(),
            decision: AuthorizationDecision::ALLOW,
            determining_policies: vec!["policy_1".to_string(), "policy_2".to_string()],
            client_ip: Some("192.168.1.50".to_string()),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.event_type(), "AuthorizationPerformed");
        assert_eq!(event.aggregate_id(), "ps_auth_123");
        assert_eq!(event.principal, "User::\"alice\"");
        assert_eq!(event.action, "Action::\"viewDocument\"");
        assert_eq!(event.resource, "Document::\"doc123\"");
        assert_eq!(event.decision, AuthorizationDecision::ALLOW);
        assert_eq!(event.determining_policies.len(), 2);
    }

    #[test]
    fn test_authorization_permitted_deny() {
        let event = AuthorizationPerformed {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_auth_456".to_string(),
            principal: "User::\"bob\"".to_string(),
            action: "Action::\"deleteDocument\"".to_string(),
            resource: "Document::\"doc456\"".to_string(),
            decision: AuthorizationDecision::DENY,
            determining_policies: vec!["policy_deny".to_string()],
            client_ip: Some("192.168.1.60".to_string()),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(event.decision, AuthorizationDecision::DENY);
        assert_eq!(event.determining_policies.len(), 1);
        assert_eq!(event.determining_policies[0], "policy_deny");
    }

    #[test]
    fn test_domain_event_common_properties() {
        let event = PolicyStoreCreated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_common_123".to_string(),
            name: "Common Test".to_string(),
            description: None,
            author: "tester".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        // Test that all domain events implement the same interface
        assert!(!event.event_id().is_empty());
        assert!(!event.event_type().is_empty());
        assert!(!event.aggregate_id().is_empty());
        assert!(event.occurred_at() <= Utc::now());
        assert_eq!(event.version(), 1);
    }

    #[test]
    fn test_event_serialization() {
        use serde_json;

        let event = PolicyStoreCreated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "ps_serialize_123".to_string(),
            name: "Serialize Test".to_string(),
            description: Some("JSON serialization test".to_string()),
            author: "tester".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: PolicyStoreCreated = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.policy_store_id, event.policy_store_id);
        assert_eq!(deserialized.name, event.name);
        assert_eq!(deserialized.description, event.description);
    }

    #[test]
    fn test_access_type_enum() {
        assert_eq!(format!("{:?}", AccessType::READ), "READ");
        assert_eq!(format!("{:?}", AccessType::WRITE), "WRITE");
        assert_eq!(format!("{:?}", AccessType::DELETE), "DELETE");
    }

    #[test]
    fn test_authorization_decision_enum() {
        assert_eq!(format!("{:?}", AuthorizationDecision::ALLOW), "ALLOW");
        assert_eq!(format!("{:?}", AuthorizationDecision::DENY), "DENY");
    }
}
