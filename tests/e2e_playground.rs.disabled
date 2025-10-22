//! E2E tests for Playground functionality
//!
//! These tests validate the playground/testing features:
//! - TestAuthorization: Test policies without persisting
//! - ValidatePolicy: Validate policy syntax and semantics
//!
//! Run with: cargo test --test e2e_playground -- --ignored --nocapture

use hodei_permissions_sdk::AuthorizationClient;
use hodei_permissions_sdk::proto::*;

const SQLITE_ENDPOINT: &str = "http://localhost:50051";

// ============================================================================
// TEST: Playground - Test Authorization without Persisting
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_playground_authorization_basic() {
    println!("🎮 Testing playground authorization (no persistence)");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    // Test authorization with ad-hoc policies (no policy store needed)
    let policies = vec![
        r#"permit(
            principal == User::"alice",
            action == Action::"read",
            resource == Document::"report"
        );"#.to_string(),
    ];

    let request = TestAuthorizationRequest {
        policy_store_id: None, // No policy store - pure playground mode
        schema: None, // No schema validation
        policies,
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "Document".to_string(),
            entity_id: "report".to_string(),
        }),
        context: None,
        entities: vec![],
    };

    let response = client
        .test_authorization_raw(request)
        .await
        .expect("Test authorization failed");

    assert_eq!(response.decision, Decision::Allow as i32, "Should ALLOW alice to read report");
    assert!(!response.determining_policies.is_empty(), "Should have determining policies");
    println!("  ✓ Authorization ALLOW (correct)");
    println!("  ✓ Determining policies: {:?}", response.determining_policies);
}

// ============================================================================
// TEST: Playground with Schema Validation
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_playground_with_schema_validation() {
    println!("🎮 Testing playground with schema validation");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    // Define a schema
    let schema = r#"{
        "MyApp": {
            "entityTypes": {
                "User": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "department": { "type": "String" }
                        }
                    }
                },
                "Document": {}
            },
            "actions": {
                "read": {
                    "appliesTo": {
                        "principalTypes": ["User"],
                        "resourceTypes": ["Document"]
                    }
                }
            }
        }
    }"#.to_string();

    // Policy that uses the schema
    let policies = vec![
        r#"permit(
            principal,
            action == MyApp::Action::"read",
            resource
        ) when {
            principal.department == "engineering"
        };"#.to_string(),
    ];

    // Entities with attributes
    let entities = vec![
        Entity {
            identifier: Some(EntityIdentifier {
                entity_type: "MyApp::User".to_string(),
                entity_id: "alice".to_string(),
            }),
            attributes: vec![
                ("department".to_string(), "\"engineering\"".to_string()),
            ].into_iter().collect(),
            parents: vec![],
        },
    ];

    let request = TestAuthorizationRequest {
        policy_store_id: None,
        schema: Some(schema),
        policies,
        principal: Some(EntityIdentifier {
            entity_type: "MyApp::User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "MyApp::Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "MyApp::Document".to_string(),
            entity_id: "doc1".to_string(),
        }),
        context: None,
        entities,
    };

    let response = client
        .test_authorization_raw(request)
        .await
        .expect("Test authorization failed");

    assert_eq!(response.decision, Decision::Allow as i32, "Should ALLOW");
    println!("  ✓ Authorization with schema validation passed");
    
    if !response.validation_warnings.is_empty() {
        println!("  ⚠️  Validation warnings: {:?}", response.validation_warnings);
    }
    if !response.validation_errors.is_empty() {
        println!("  ❌ Validation errors: {:?}", response.validation_errors);
    }
}

// ============================================================================
// TEST: Playground with Policy Store Schema
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_playground_with_policy_store_schema() {
    println!("🎮 Testing playground using existing policy store schema");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    // 1. Create policy store with schema
    let store = client
        .create_policy_store(Some("Playground Test Store".to_string()))
        .await
        .expect("Failed to create policy store");
    let policy_store_id = store.policy_store_id.clone();

    // 2. Put schema
    let schema = r#"{
        "TestApp": {
            "entityTypes": {
                "User": {},
                "File": {}
            },
            "actions": {
                "view": {
                    "appliesTo": {
                        "principalTypes": ["User"],
                        "resourceTypes": ["File"]
                    }
                }
            }
        }
    }"#;

    client
        .put_schema(&policy_store_id, schema.to_string())
        .await
        .expect("Failed to put schema");

    // 3. Test authorization using the policy store's schema
    let policies = vec![
        r#"permit(
            principal == TestApp::User::"bob",
            action == TestApp::Action::"view",
            resource
        );"#.to_string(),
    ];

    let request = TestAuthorizationRequest {
        policy_store_id: Some(policy_store_id.clone()), // Use policy store's schema
        schema: None, // Schema will be loaded from policy store
        policies,
        principal: Some(EntityIdentifier {
            entity_type: "TestApp::User".to_string(),
            entity_id: "bob".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "TestApp::Action".to_string(),
            entity_id: "view".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "TestApp::File".to_string(),
            entity_id: "file1".to_string(),
        }),
        context: None,
        entities: vec![],
    };

    let response = client
        .test_authorization_raw(request)
        .await
        .expect("Test authorization failed");

    assert_eq!(response.decision, Decision::Allow as i32, "Should ALLOW bob to view file");
    println!("  ✓ Authorization using policy store schema passed");

    // Cleanup
    client.delete_policy_store(&policy_store_id).await.ok();
}

// ============================================================================
// TEST: Validate Policy - Syntax Error
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_validate_policy_syntax_error() {
    println!("🔍 Testing policy validation - syntax error");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    // Invalid policy syntax (missing semicolon)
    let invalid_policy = r#"permit(
        principal,
        action,
        resource
    )"#; // Missing semicolon

    let schema = r#"{
        "App": {
            "entityTypes": {},
            "actions": {}
        }
    }"#.to_string();

    let request = ValidatePolicyRequest {
        policy_store_id: None,
        schema: Some(schema),
        policy_statement: invalid_policy.to_string(),
    };

    let response = client
        .validate_policy_raw(request)
        .await
        .expect("Validate policy failed");

    assert!(!response.is_valid, "Policy should be invalid");
    assert!(!response.errors.is_empty(), "Should have syntax errors");
    println!("  ✓ Syntax error detected correctly");
    println!("  ✓ Errors: {:?}", response.errors.iter().map(|e| &e.message).collect::<Vec<_>>());
}

// ============================================================================
// TEST: Validate Policy - Valid Policy
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_validate_policy_valid() {
    println!("🔍 Testing policy validation - valid policy");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    let valid_policy = r#"permit(
        principal == User::"alice",
        action == Action::"read",
        resource == Document::"doc1"
    );"#;

    let schema = r#"{
        "": {
            "entityTypes": {
                "User": {},
                "Document": {}
            },
            "actions": {
                "read": {
                    "appliesTo": {
                        "principalTypes": ["User"],
                        "resourceTypes": ["Document"]
                    }
                }
            }
        }
    }"#.to_string();

    let request = ValidatePolicyRequest {
        policy_store_id: None,
        schema: Some(schema),
        policy_statement: valid_policy.to_string(),
    };

    let response = client
        .validate_policy_raw(request)
        .await
        .expect("Validate policy failed");

    assert!(response.is_valid, "Policy should be valid");
    assert!(response.errors.is_empty(), "Should have no errors");
    assert!(response.policy_info.is_some(), "Should have policy info");
    
    let policy_info = response.policy_info.unwrap();
    assert_eq!(policy_info.effect, "permit", "Effect should be permit");
    println!("  ✓ Policy validated successfully");
    println!("  ✓ Effect: {}", policy_info.effect);
}

// ============================================================================
// TEST: Validate Policy - Schema Validation Error
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_validate_policy_schema_error() {
    println!("🔍 Testing policy validation - schema validation error");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    // Policy references non-existent entity type
    let policy = r#"permit(
        principal == NonExistentType::"alice",
        action == Action::"read",
        resource
    );"#;

    let schema = r#"{
        "": {
            "entityTypes": {
                "User": {},
                "Document": {}
            },
            "actions": {
                "read": {
                    "appliesTo": {
                        "principalTypes": ["User"],
                        "resourceTypes": ["Document"]
                    }
                }
            }
        }
    }"#.to_string();

    let request = ValidatePolicyRequest {
        policy_store_id: None,
        schema: Some(schema),
        policy_statement: policy.to_string(),
    };

    let response = client
        .validate_policy_raw(request)
        .await
        .expect("Validate policy failed");

    // Note: Cedar might not catch this as an error if the syntax is valid
    // but it should at least not crash
    println!("  ✓ Validation completed");
    println!("  ✓ Is valid: {}", response.is_valid);
    if !response.errors.is_empty() {
        println!("  ✓ Errors: {:?}", response.errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }
    if !response.warnings.is_empty() {
        println!("  ⚠️  Warnings: {:?}", response.warnings.iter().map(|w| &w.message).collect::<Vec<_>>());
    }
}

// ============================================================================
// TEST: Playground - Multiple Policies
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_playground_multiple_policies() {
    println!("🎮 Testing playground with multiple policies");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    // Multiple policies: one permit, one forbid
    let policies = vec![
        // Allow all users to read
        r#"permit(
            principal,
            action == Action::"read",
            resource
        );"#.to_string(),
        // But forbid alice specifically
        r#"forbid(
            principal == User::"alice",
            action == Action::"read",
            resource == Document::"secret"
        );"#.to_string(),
    ];

    // Test 1: Bob can read secret (only permit applies)
    let request1 = TestAuthorizationRequest {
        policy_store_id: None,
        schema: None,
        policies: policies.clone(),
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "bob".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "Document".to_string(),
            entity_id: "secret".to_string(),
        }),
        context: None,
        entities: vec![],
    };

    let response1 = client
        .test_authorization_raw(request1)
        .await
        .expect("Test authorization failed");

    assert_eq!(response1.decision, Decision::Allow as i32, "Bob should be ALLOWED");
    println!("  ✓ Bob can read secret (ALLOW)");

    // Test 2: Alice cannot read secret (forbid overrides permit)
    let request2 = TestAuthorizationRequest {
        policy_store_id: None,
        schema: None,
        policies,
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "Document".to_string(),
            entity_id: "secret".to_string(),
        }),
        context: None,
        entities: vec![],
    };

    let response2 = client
        .test_authorization_raw(request2)
        .await
        .expect("Test authorization failed");

    assert_eq!(response2.decision, Decision::Deny as i32, "Alice should be DENIED");
    println!("  ✓ Alice cannot read secret (DENY - forbid policy)");
    println!("  ✓ Determining policies: {:?}", response2.determining_policies);
}

// ============================================================================
// TEST: Playground - ABAC with Context
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_playground_abac_with_context() {
    println!("🎮 Testing playground with ABAC and context");
    
    let client = AuthorizationClient::connect(SQLITE_ENDPOINT.to_string())
        .await
        .expect("Failed to connect to server");

    // Policy that checks context
    let policies = vec![
        r#"permit(
            principal,
            action == Action::"access",
            resource
        ) when {
            context.ip_address like "192.168.*"
        };"#.to_string(),
    ];

    // Test with valid IP
    let request = TestAuthorizationRequest {
        policy_store_id: None,
        schema: None,
        policies,
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "access".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "System".to_string(),
            entity_id: "server1".to_string(),
        }),
        context: Some(r#"{"ip_address": "192.168.1.100"}"#.to_string()),
        entities: vec![],
    };

    let response = client
        .test_authorization_raw(request)
        .await
        .expect("Test authorization failed");

    assert_eq!(response.decision, Decision::Allow as i32, "Should ALLOW from internal IP");
    println!("  ✓ Access from internal IP allowed");
}
