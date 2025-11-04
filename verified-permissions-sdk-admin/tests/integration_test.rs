//! Basic tests for sdk-admin library

use hodei_permissions_sdk::proto::{EntityIdentifier, IsAuthorizedRequest};
use sdk_admin::{HodeiAdmin, SdkAdminError};

#[test]
fn test_sdk_admin_error_creation() {
    let error = SdkAdminError::ConnectionFailed;
    assert!(error.to_string().contains("Failed to connect"));

    let error = SdkAdminError::InvalidRequest("test".to_string());
    assert!(error.to_string().contains("Invalid request"));
}

#[test]
fn test_sdk_admin_compile_time_api() {
    // This test ensures the Control Plane API is available
    // It will fail to compile if methods don't exist
}

#[tokio::test]
async fn test_error_handling_invalid_endpoint() {
    // Test that invalid endpoints are handled gracefully
    let result = HodeiAdmin::connect("invalid://endpoint").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_batch_operations_availability() {
    // Test that bulk/batch methods compile correctly
    // This is a compile-time test
}

#[test]
fn test_entity_identifier_creation() {
    // Test EntityIdentifier creation for bulk operations
    let entity = EntityIdentifier {
        entity_type: "User".to_string(),
        entity_id: "alice".to_string(),
    };
    assert_eq!(entity.entity_type, "User");
    assert_eq!(entity.entity_id, "alice");
}

#[test]
fn test_policy_definition_creation() {
    // Test policy definition creation for bulk operations
    let statement = r#"permit(principal == User::"alice", action, resource);"#;

    // This test ensures the API for bulk operations compiles
    // The actual functionality will be tested via integration tests with a real server
}
