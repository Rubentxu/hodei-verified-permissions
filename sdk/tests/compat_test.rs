//! Test for compatibility layer with v0.1.x API
//! This test verifies that deprecated methods return appropriate errors

#[cfg(feature = "compat")]
use hodei_permissions_sdk::compat::create_policy_store_deprecated;

#[tokio::test]
#[cfg(feature = "compat")]
async fn test_compatibility_layer_returns_error() {
    let result = create_policy_store_deprecated(
        "test-store".to_string(),
        Some("Test description".to_string()),
    )
    .await;

    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Verify error message contains migration guidance
    assert!(error_msg.contains("deprecated"));
    assert!(error_msg.contains("CLI tool"));
    assert!(error_msg.contains("HodeiAdmin library"));
}

#[tokio::test]
#[cfg(not(feature = "compat"))]
async fn test_compat_feature_disabled() {
    // Test that the compat feature can be disabled
    // This test passes when the feature is not enabled
    assert!(true);
}
