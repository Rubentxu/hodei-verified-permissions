//! Batch operations example for Hodei Admin SDK
//!
//! This example demonstrates:
//! - Bulk creation of policies using batch operations
//! - Batch updates and deletions
//! - Authorization testing (playground mode)
//! - Policy validation against schema
//! - Batch authorization checks
//! - Handling partial failures
//! - Progress tracking

use std::error::Error;
use verified_permissions_sdk::proto::{EntityIdentifier, IsAuthorizedRequest};
use verified_permissions_sdk_admin::HodeiAdmin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Hodei Admin SDK - Batch Operations Example ===\n");

    let endpoint = "http://localhost:50051";

    // Connect to server
    let mut client = HodeiAdmin::connect(endpoint).await.map_err(|e| {
        eprintln!("Failed to connect: {}", e);
        eprintln!("Make sure server is running on {}", endpoint);
        e
    })?;

    println!("✓ Connected to server\n");

    // Create a policy store for batch operations
    let store = client
        .create_policy_store(
            "batch-operations-store",
            Some("Store for testing batch operations".to_string()),
        )
        .await?;

    let policy_store_id = store.policy_store_id;
    println!("✓ Created policy store: {}\n", policy_store_id);

    // Example 1: True BATCH policy creation using new bulk operations
    println!("Example 1: True Batch Policy Creation (NEW BULK API)\n");

    let policies = vec![
        (
            "read-doc",
            r#"permit(
                principal == User:user_id,
                action == Action:"read",
                resource == Resource:resource_id
            ) when {
                resource.type == "document"
            }"#,
            Some("Users can read documents".to_string()),
        ),
        (
            "write-doc",
            r#"permit(
                principal == User:user_id,
                action == Action:"write",
                resource == Resource:resource_id
            ) when {
                resource.type == "document" && principal.roles.has("editor")
            }"#,
            Some("Editors can write documents".to_string()),
        ),
        (
            "delete-doc",
            r#"permit(
                principal == User:user_id,
                action == Action:"delete",
                resource == Resource:resource_id
            ) when {
                resource.type == "document" && principal.roles.has("admin")
            }"#,
            Some("Admins can delete documents".to_string()),
        ),
        (
            "view-sensitive",
            r#"permit(
                principal == User:user_id,
                action == Action:"read",
                resource == Resource:resource_id
            ) when {
                resource.sensitive == true && principal.roles.has("security")
            }"#,
            Some("Security team can view sensitive documents".to_string()),
        ),
    ];

    // Use the NEW batch API - creates all policies in a single call!
    match client
        .batch_create_policies(&policy_store_id, policies.clone())
        .await
    {
        Ok(response) => {
            println!(
                "  ✓ Batch created {} policies successfully!",
                response.results.len()
            );
            for result in &response.results {
                if let Some(error) = &result.error {
                    println!("    ✗ Policy '{}' failed: {}", result.policy_id, error);
                } else {
                    println!(
                        "    ✓ Policy '{}': {} (created at: {})",
                        result.policy_id, result.created_at, result.created_at
                    );
                }
            }
            if !response.errors.is_empty() {
                println!("\n  Batch errors: {:?}", response.errors);
            }
        }
        Err(e) => {
            println!("  ✗ Batch creation failed: {}", e);
        }
    }

    println!("\nNote: The old method (individual policy creation) would require");
    println!(
        "{} separate API calls. The new batch API needs only 1!\n",
        policies.len()
    );

    // Example 2: Creating multiple policy stores
    println!("Example 2: Multiple Policy Stores Creation\n");

    let store_definitions = vec![
        ("dev-store", "Development environment policies"),
        ("staging-store", "Staging environment policies"),
        ("prod-store", "Production environment policies"),
    ];

    let mut created_stores = Vec::new();

    for &(ref name, ref description) in store_definitions.iter() {
        match client
            .create_policy_store(*name, Some(description.to_string()))
            .await
        {
            Ok(store) => {
                let store_id = store.policy_store_id;
                created_stores.push(store_id.clone());
                println!("  ✓ Created store '{}': {}", name, store_id);
            }
            Err(e) => {
                println!("  ✗ Failed to create store '{}': {}", name, e);
            }
        }
    }

    println!("\nCreated {} policy stores\n", created_stores.len());

    // Example 3: Batch schema upload to multiple stores
    println!("Example 3: Batch Schema Operations\n");

    let common_schema = r#"{
        "entities": {
            "User": {
                "id": "String",
                "roles": "Set<String>"
            },
            "Resource": {
                "id": "String",
                "owner_id": "String"
            }
        }
    }"#;

    for (index, store_id) in created_stores.iter().enumerate() {
        match client.put_schema(store_id, common_schema).await {
            Ok(_) => {
                println!(
                    "  ✓ [{}/{}] Uploaded schema to store: {}",
                    index + 1,
                    created_stores.len(),
                    store_id
                );
            }
            Err(e) => {
                println!(
                    "  ✗ [{}/{}] Failed to upload schema to {}: {}",
                    index + 1,
                    created_stores.len(),
                    store_id,
                    e
                );
            }
        }
    }

    // Example 4: Pagination with list operations
    println!("\nExample 4: Pagination with List Operations\n");

    // Create more policies to demonstrate pagination
    for i in 0..15 {
        let policy_id = format!("pagination-policy-{:02}", i);
        let statement = format!(
            r#"permit(
                principal == User:user_id,
                action == Action:"view",
                resource == Resource:resource_id
            ) when {{
                resource.id == "{}"
            }}"#,
            format!("res-{:02}", i)
        );

        let _ = client
            .create_policy(&policy_store_id, &policy_id, &statement, None)
            .await;
    }

    // List policies with pagination
    println!("Listing policies with pagination:");
    let mut next_token = None;
    let mut page_count = 0;
    let mut total_policies = 0;

    loop {
        page_count += 1;
        println!("\n  Page {}:", page_count);

        let response = client.list_policies(&policy_store_id).await.map_err(|e| {
            eprintln!("Failed to list policies: {}", e);
            e
        })?;

        for policy in &response.policies {
            println!("    - {}", policy.policy_id);
            total_policies += 1;
        }

        if let Some(token) = &response.next_token {
            next_token = Some(token.clone());
        } else {
            break;
        }
    }

    println!("\n  Total policies found: {}\n", total_policies);

    // Example 5: Batch policy updates
    println!("Example 5: Batch Policy Updates (NEW BULK API)\n");

    let updated_policies = vec![
        (
            "read-doc",
            r#"permit(
                principal == User:user_id,
                action == Action:"read",
                resource == Resource:resource_id
            ) when {
                resource.type == "document" && principal.verified == true
            }"#,
            Some("Users can read documents (updated: now requires verified)".to_string()),
        ),
        (
            "write-doc",
            r#"permit(
                principal == User:user_id,
                action == Action:"write",
                resource == Resource:resource_id
            ) when {
                resource.type == "document" &&
                (principal.roles.has("editor") || principal.roles.has("admin"))
            }"#,
            Some("Editors and admins can write documents (updated: admins included)".to_string()),
        ),
    ];

    match client
        .batch_update_policies(&policy_store_id, updated_policies)
        .await
    {
        Ok(response) => {
            println!("  ✓ Batch updated {} policies!", response.results.len());
            for result in &response.results {
                if let Some(error) = &result.error {
                    println!("    ✗ Policy '{}' failed: {}", result.policy_id, error);
                } else {
                    println!(
                        "    ✓ Policy '{}' updated at: {}",
                        result.policy_id, result.updated_at
                    );
                }
            }
        }
        Err(e) => {
            println!("  ✗ Batch update failed: {}", e);
        }
    }

    println!();

    // Example 6: Policy validation (NEW BULK API)
    println!("Example 6: Policy Validation (NEW BULK API)\n");

    let test_policy = r#"permit(
        principal == User:"alice",
        action == Action:"read",
        resource == Resource:"doc123"
    );"#;

    match client
        .validate_policy(None, None, test_policy.to_string())
        .await
    {
        Ok(response) => {
            if response.is_valid {
                println!("  ✓ Policy is valid!");
                if let Some(info) = &response.policy_info {
                    println!("    Effect: {}", info.effect);
                    println!("    Has conditions: {}", info.has_conditions);
                }
            } else {
                println!("  ✗ Policy validation failed!");
                for error in &response.errors {
                    println!("    Error: {} (type: {})", error.message, error.issue_type);
                }
            }
            if !response.warnings.is_empty() {
                for warning in &response.warnings {
                    println!(
                        "    Warning: {} (type: {})",
                        warning.message, warning.issue_type
                    );
                }
            }
        }
        Err(e) => {
            println!("  ✗ Validation request failed: {}", e);
        }
    }

    println!();

    // Example 7: Authorization Testing (Playground mode - NEW BULK API)
    println!("Example 7: Authorization Testing (Playground - NEW BULK API)\n");

    let test_policies = vec![
        r#"permit(
            principal == User:"alice",
            action == Action:"read",
            resource == Resource:"doc123"
        );"#
        .to_string(),
        r#"forbid(
            principal == User:"bob",
            action == Action:"read",
            resource == Resource:"doc123"
        );"#
        .to_string(),
    ];

    let principal = EntityIdentifier {
        entity_type: "User".to_string(),
        entity_id: "alice".to_string(),
    };

    let action = EntityIdentifier {
        entity_type: "Action".to_string(),
        entity_id: "read".to_string(),
    };

    let resource = EntityIdentifier {
        entity_type: "Resource".to_string(),
        entity_id: "doc123".to_string(),
    };

    match client
        .test_authorization(test_policies, principal, action, resource, None)
        .await
    {
        Ok(response) => {
            println!("  ✓ Test completed!");
            println!("    Decision: {:?}", response.decision);
            println!(
                "    Determining policies: {:?}",
                response.determining_policies
            );
            if !response.errors.is_empty() {
                println!("    Errors: {:?}", response.errors);
            }
            if !response.validation_warnings.is_empty() {
                println!("    Warnings: {:?}", response.validation_warnings);
            }
        }
        Err(e) => {
            println!("  ✗ Test failed: {}", e);
        }
    }

    println!();

    // Example 8: Batch authorization checks (NEW BULK API)
    println!("Example 8: Batch Authorization Checks (NEW BULK API)\n");

    let batch_requests = vec![
        {
            let mut req = IsAuthorizedRequest {
                policy_store_id: policy_store_id.clone(),
                principal: Some(EntityIdentifier {
                    entity_type: "User".to_string(),
                    entity_id: "alice".to_string(),
                }),
                action: Some(EntityIdentifier {
                    entity_type: "Action".to_string(),
                    entity_id: "read".to_string(),
                }),
                resource: Some(EntityIdentifier {
                    entity_type: "Resource".to_string(),
                    entity_id: "doc123".to_string(),
                }),
                context: None,
                entities: vec![],
            };
            req
        },
        {
            let mut req = IsAuthorizedRequest {
                policy_store_id: policy_store_id.clone(),
                principal: Some(EntityIdentifier {
                    entity_type: "User".to_string(),
                    entity_id: "bob".to_string(),
                }),
                action: Some(EntityIdentifier {
                    entity_type: "Action".to_string(),
                    entity_id: "read".to_string(),
                }),
                resource: Some(EntityIdentifier {
                    entity_type: "Resource".to_string(),
                    entity_id: "doc123".to_string(),
                }),
                context: None,
                entities: vec![],
            };
            req
        },
    ];

    match client
        .batch_is_authorized(&policy_store_id, batch_requests)
        .await
    {
        Ok(response) => {
            println!("  ✓ Batch authorization check completed!");
            println!("    Total responses: {}", response.responses.len());
            for (index, resp) in response.responses.iter().enumerate() {
                println!(
                    "    [{}/{}] Decision: {:?}",
                    index + 1,
                    response.responses.len(),
                    resp.decision
                );
            }
        }
        Err(e) => {
            println!("  ✗ Batch authorization check failed: {}", e);
        }
    }

    println!();

    // Example 9: Batch cleanup with error handling (NEW BULK API)
    println!("Example 9: Batch Cleanup (NEW BULK API)\n");

    println!("Cleaning up policies (batch delete)...");
    let delete_targets = vec![
        "read-doc",
        "write-doc",
        "delete-doc",
        "view-sensitive",
        "pagination-policy-00",
        "pagination-policy-01",
        "pagination-policy-02",
    ];

    match client
        .batch_delete_policies(&policy_store_id, delete_targets.clone())
        .await
    {
        Ok(response) => {
            println!(
                "  ✓ Batch delete completed! Processed {} policies",
                response.results.len()
            );
            let mut deleted = 0;
            let mut failed = 0;
            for result in &response.results {
                if let Some(error) = &result.error {
                    failed += 1;
                    println!("    ✗ Failed to delete '{}': {}", result.policy_id, error);
                } else {
                    deleted += 1;
                    println!("    ✓ Deleted policy: {}", result.policy_id);
                }
            }
            println!("\n  Summary: {} deleted, {} failed", deleted, failed);

            if !response.errors.is_empty() {
                println!("  Batch errors: {:?}", response.errors);
            }
        }
        Err(e) => {
            println!("  ✗ Batch delete failed: {}", e);
            println!("\n  Falling back to individual deletes...");

            let mut deleted = 0;
            let mut failed = 0;
            for policy_id in delete_targets {
                match client.delete_policy(&policy_store_id, policy_id).await {
                    Ok(_) => {
                        deleted += 1;
                        println!("  ✓ Deleted policy: {}", policy_id);
                    }
                    Err(e) => {
                        if e.to_string().contains("NotFound") {
                            println!("  - Policy not found (already deleted?): {}", policy_id);
                        } else {
                            failed += 1;
                            println!("  ✗ Failed to delete {}: {}", policy_id, e);
                        }
                    }
                }
            }
            println!(
                "\n  Individual delete summary: {} deleted, {} failed",
                deleted, failed
            );
        }
    }

    // Cleanup: Delete all created policy stores
    println!("\n\nCleaning up policy stores...");

    // Add the original store
    created_stores.push(policy_store_id);

    for store_id in created_stores {
        match client.delete_policy_store(&store_id).await {
            Ok(_) => {
                println!("  ✓ Deleted store: {}", store_id);
            }
            Err(e) => {
                println!("  ✗ Failed to delete store {}: {}", store_id, e);
            }
        }
    }

    println!("\n=== Batch Operations Example Completed! ===");
    println!("\nSummary:");
    println!("  - Created and managed multiple policy stores");
    println!("  - Demonstrated bulk policy operations");
    println!("  - Handled errors gracefully");
    println!("  - Used pagination for large result sets");

    Ok(())
}
