//! Unit tests for lightweight SDK (Data Plane operations only)

use verified_permissions_sdk::AuthorizationClient;
use verified_permissions_sdk::AuthorizationDecision;
use verified_permissions_sdk::IsAuthorizedRequestBuilder;
use verified_permissions_sdk::SdkError;
use verified_permissions_sdk::entities::CedarEntityBuilder;
use verified_permissions_sdk::proto::{Decision, EntityIdentifier};

#[test]
fn test_entity_builder() {
    let user = CedarEntityBuilder::new("User".to_string(), "alice".to_string())
        .attribute("department".to_string(), "engineering".to_string())
        .build();

    assert_eq!(user.uid.entity_type, "User");
    assert_eq!(user.uid.id, "alice");
    assert_eq!(user.attrs.len(), 1);
}

#[test]
fn test_is_authorized_request_builder() {
    let request = IsAuthorizedRequestBuilder::new("store123")
        .principal("User".to_string(), "alice".to_string())
        .action("Action".to_string(), "read".to_string())
        .resource("Document".to_string(), "doc123".to_string())
        .build();

    assert_eq!(request.policy_store_id, "store123");
    assert!(request.principal.is_some());
}

#[test]
fn test_is_authorized_request_builder_with_context() {
    let request = IsAuthorizedRequestBuilder::new("store123")
        .principal("User".to_string(), "alice".to_string())
        .context("{\"department\": \"engineering\"}".to_string())
        .build();

    assert!(request.context.is_some());
}

#[test]
fn test_entity_identifier_format() {
    let user = EntityIdentifier {
        entity_type: "User".to_string(),
        entity_id: "alice".to_string(),
    };

    assert_eq!(user.entity_type, "User");
    assert_eq!(user.entity_id, "alice");
}

#[test]
fn test_decision_types() {
    let decision = Decision::Allow;
    assert_eq!(decision.as_str_name(), "ALLOW");

    let decision = Decision::Deny;
    assert_eq!(decision.as_str_name(), "DENY");
}

#[test]
fn test_entity_builder_chaining() {
    let entity = CedarEntityBuilder::new("User".to_string(), "alice".to_string())
        .attribute("role".to_string(), "admin".to_string())
        .attribute("department".to_string(), "engineering".to_string())
        .build();

    assert_eq!(entity.attrs.len(), 2);
}

#[test]
fn test_request_builder_default_values() {
    let request = IsAuthorizedRequestBuilder::new("store123").build();

    assert_eq!(request.policy_store_id, "store123");
    assert!(request.entities.is_empty());
}

#[test]
fn test_error_propagation() {
    let error = SdkError::ConnectionError("test".to_string());
    assert!(error.to_string().contains("Connection"));

    let error = SdkError::InvalidRequest("test".to_string());
    assert!(error.to_string().contains("Invalid"));
}

#[test]
fn test_request_with_context_attributes() {
    let request = IsAuthorizedRequestBuilder::new("store123")
        .context("{\"project\": \"alpha\"}".to_string())
        .build();

    assert!(request.context.is_some());
}

#[test]
fn test_authorization_decision_allow() {
    let decision = AuthorizationDecision::Allow;
    assert!(decision.is_allow());
    assert!(!decision.is_deny());
}

#[test]
fn test_authorization_decision_deny() {
    let decision = AuthorizationDecision::Deny;
    assert!(decision.is_deny());
    assert!(!decision.is_allow());
}

#[test]
fn test_entity_builder_with_parents() {
    let entity = CedarEntityBuilder::new("User".to_string(), "alice".to_string())
        .parent("Role".to_string(), "admin".to_string())
        .build();

    assert_eq!(entity.parents.len(), 1);
}

#[test]
fn test_entity_builder_multiple_parents() {
    let entity = CedarEntityBuilder::new("User".to_string(), "alice".to_string())
        .parent("Role".to_string(), "admin".to_string())
        .parent("Group".to_string(), "engineering".to_string())
        .build();

    assert_eq!(entity.parents.len(), 2);
}

#[test]
fn test_is_authorized_request_builder_minimal() {
    let request = IsAuthorizedRequestBuilder::new("minimal-store")
        .principal("User".to_string(), "user1".to_string())
        .action("Action".to_string(), "read".to_string())
        .resource("Resource".to_string(), "res1".to_string())
        .build();

    assert_eq!(request.policy_store_id, "minimal-store");
    assert!(request.principal.is_some());
    assert!(request.action.is_some());
    assert!(request.resource.is_some());
}

#[test]
fn test_sdk_error_variants() {
    let errors = vec![
        SdkError::ConnectionError("conn".to_string()),
        SdkError::InvalidRequest("invalid".to_string()),
    ];

    for error in errors {
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
    }
}

#[test]
fn test_data_plane_only_operations() {
    // Verify compile-time API availability
}
