//! Unit tests for repository operations

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use hodei_domain::{AuditLogEntry, AuditLogFilters};
    use serde_json;

    #[test]
    fn test_audit_log_filters_default() {
        let filters = AuditLogFilters {
            event_types: None,
            service_name: None,
            policy_store_id: None,
            start_date: None,
            end_date: None,
            limit: None,
        };

        assert_eq!(filters.event_types, None);
        assert_eq!(filters.policy_store_id, None);
        assert_eq!(filters.limit, None);
    }

    #[test]
    fn test_audit_log_filters_with_event_types() {
        let filters = AuditLogFilters {
            event_types: Some(vec![
                "ApiCalled".to_string(),
                "PolicyStoreCreated".to_string(),
            ]),
            service_name: None,
            policy_store_id: None,
            start_date: None,
            end_date: None,
            limit: None,
        };

        assert_eq!(
            filters.event_types,
            Some(vec![
                "ApiCalled".to_string(),
                "PolicyStoreCreated".to_string()
            ])
        );
        assert_eq!(filters.event_types.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_audit_log_filters_with_date_range() {
        let now = Utc::now();
        let start_date = now - Duration::hours(1);
        let end_date = now;

        let filters = AuditLogFilters {
            event_types: None,
            service_name: None,
            policy_store_id: None,
            start_date: Some(start_date),
            end_date: Some(end_date),
            limit: Some(100),
        };

        assert!(filters.start_date.is_some());
        assert!(filters.end_date.is_some());
        assert_eq!(filters.limit, Some(100));
    }

    #[test]
    fn test_audit_log_filters_serialization() {
        let filters = AuditLogFilters {
            event_types: Some(vec!["ApiCalled".to_string()]),
            service_name: Some("TestService".to_string()),
            policy_store_id: Some("ps_test_123".to_string()),
            start_date: Some(Utc::now() - Duration::hours(1)),
            end_date: Some(Utc::now()),
            limit: Some(50),
        };

        let json = serde_json::to_string(&filters).unwrap();
        assert!(!json.is_empty());

        let deserialized: AuditLogFilters = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.service_name, Some("TestService".to_string()));
        assert_eq!(
            deserialized.policy_store_id,
            Some("ps_test_123".to_string())
        );
    }

    #[test]
    fn test_audit_log_entry() {
        let event_data = serde_json::json!({
            "service": "TestService",
            "action": "test_action"
        });

        let entry = AuditLogEntry {
            event_id: "evt_123".to_string(),
            event_type: "TestEvent".to_string(),
            aggregate_id: "agg_456".to_string(),
            event_data: event_data.clone(),
            occurred_at: Utc::now(),
            version: 1,
        };

        assert_eq!(entry.event_id, "evt_123");
        assert_eq!(entry.event_type, "TestEvent");
        assert_eq!(entry.aggregate_id, "agg_456");
        assert_eq!(entry.version, 1);
        assert_eq!(entry.event_data, event_data);
    }

    #[test]
    fn test_audit_log_entry_serialization() {
        let event_data = serde_json::json!({
            "field1": "value1",
            "field2": 42
        });

        let entry = AuditLogEntry {
            event_id: "evt_serialize_123".to_string(),
            event_type: "SerializeEvent".to_string(),
            aggregate_id: "agg_serialize_456".to_string(),
            event_data: event_data.clone(),
            occurred_at: Utc::now(),
            version: 1,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.is_empty());

        let deserialized: AuditLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_id, entry.event_id);
        assert_eq!(deserialized.event_type, entry.event_type);
        assert_eq!(deserialized.aggregate_id, entry.aggregate_id);
        assert_eq!(deserialized.version, entry.version);
    }

    #[test]
    fn test_complex_event_data() {
        let complex_data = serde_json::json!({
            "user": {
                "id": "user_123",
                "name": "Test User",
                "roles": ["admin", "developer"]
            },
            "request": {
                "method": "POST",
                "path": "/api/test",
                "headers": {
                    "content-type": "application/json"
                }
            },
            "metadata": {
                "timestamp": Utc::now().to_rfc3339(),
                "source": "unit_test"
            }
        });

        let entry = AuditLogEntry {
            event_id: "evt_complex_123".to_string(),
            event_type: "ComplexEvent".to_string(),
            aggregate_id: "agg_complex_456".to_string(),
            event_data: complex_data,
            occurred_at: Utc::now(),
            version: 1,
        };

        let json_value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&entry.event_data).unwrap()).unwrap();

        assert!(json_value.get("user").is_some());
        assert!(json_value.get("request").is_some());
        assert!(json_value.get("metadata").is_some());
    }
}
