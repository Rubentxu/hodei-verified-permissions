//! Basic usage example for Hodei Admin SDK
//!
//! This example demonstrates:
//! - Connecting to Hodei server
//! - Creating a policy store
//! - Uploading a schema
//! - Creating, reading, updating, and deleting policies

use std::error::Error;
use verified_permissions_sdk_admin::HodeiAdmin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure logging
    tracing_subscriber::fmt::init();

    println!("=== Hodei Admin SDK - Basic Usage Example ===\n");

    // 1. Connect to the Hodei server
    let endpoint = "http://localhost:50051";
    println!("Connecting to: {}", endpoint);

    let mut client = HodeiAdmin::connect(endpoint).await.map_err(|e| {
        eprintln!("Failed to connect to server: {}", e);
        eprintln!("Make sure the Hodei server is running on {}", endpoint);
        e
    })?;

    println!("✓ Connected successfully\n");

    // 2. Create a policy store
    println!("Creating policy store...");
    let store_name = "example-store";
    let store_description = Some("Example policy store".to_string());

    let store = client
        .create_policy_store(store_name, store_description.clone())
        .await
        .map_err(|e| {
            eprintln!("Failed to create policy store: {}", e);
            e
        })?;

    let policy_store_id = store.policy_store_id;
    println!("✓ Created policy store: {}", policy_store_id);
    println!("  Name: {}", store_name);
    println!("  Description: {}\n", store_description.unwrap_or_default());

    // 3. Upload schema
    println!("Uploading schema...");
    let schema = r#"{
        "entities": {
            "User": {
                "id": "String",
                "email": "String",
                "roles": "Set<String>",
                "department": "String"
            },
            "Resource": {
                "id": "String",
                "owner_id": "String",
                "type": "String",
                "sensitive": "Boolean"
            },
            "Action": {
                "read": "Boolean",
                "write": "Boolean",
                "delete": "Boolean"
            }
        }
    }"#;

    client
        .put_schema(&policy_store_id, schema)
        .await
        .map_err(|e| {
            eprintln!("Failed to upload schema: {}", e);
            e
        })?;

    println!("✓ Schema uploaded successfully\n");

    // 4. Create policies
    println!("Creating policies...");

    // Policy 1: Users can read their own resources
    let policy_1_id = "user-read-own-policy";
    let policy_1_statement = r#"permit(
    principal == User:user_id,
    action == Action:"read",
    resource == Resource:resource_id
) when {
    principal.id == resource.owner_id
}"#;

    let policy_1 = client
        .create_policy(
            &policy_store_id,
            policy_1_id,
            policy_1_statement,
            Some("Users can read resources they own".to_string()),
        )
        .await?;

    println!("✓ Created policy 1: {}", policy_1.policy_id);

    // Policy 2: Admins can do anything
    let policy_2_id = "admin-all-policy";
    let policy_2_statement = r#"permit(
    principal == User:user_id,
    action == Action:action_id,
    resource == Resource:resource_id
) when {
    principal.roles.contains("admin")
}"#;

    let policy_2 = client
        .create_policy(
            &policy_store_id,
            policy_2_id,
            policy_2_statement,
            Some("Admins can perform any action".to_string()),
        )
        .await?;

    println!("✓ Created policy 2: {}", policy_2.policy_id);

    // Policy 3: Engineering department can write resources
    let policy_3_id = "eng-write-policy";
    let policy_3_statement = r#"permit(
    principal == User:user_id,
    action == Action:"write",
    resource == Resource:resource_id
) when {
    principal.department == "engineering"
}"#;

    let policy_3 = client
        .create_policy(
            &policy_store_id,
            policy_3_id,
            policy_3_statement,
            Some("Engineering can write resources".to_string()),
        )
        .await?;

    println!("✓ Created policy 3: {}\n", policy_3.policy_id);

    // 5. List all policies
    println!("Listing all policies...");
    let policies_response = client.list_policies(&policy_store_id).await?;

    println!("Found {} policies:", policies_response.policies.len());
    for policy in &policies_response.policies {
        println!("  - {} (ID: {})", policy.policy_id, policy.policy_id);
        if let Some(desc) = &policy.description {
            println!("    Description: {}", desc);
        }
    }
    println!();

    // 6. Get a specific policy
    println!("Getting specific policy: {}", policy_1_id);
    let retrieved_policy = client.get_policy(&policy_store_id, policy_1_id).await?;

    println!("✓ Retrieved policy: {}", retrieved_policy.policy_id);
    println!("  Description: {:?}\n", retrieved_policy.description);

    // 7. Update a policy
    println!("Updating policy: {}", policy_2_id);
    let updated_statement = r#"permit(
    principal == User:user_id,
    action == Action:action_id,
    resource == Resource:resource_id
) when {
    principal.roles.has("admin") || principal.roles.has("owner")
}"#;

    client
        .update_policy(
            &policy_store_id,
            policy_2_id,
            updated_statement,
            Some("Admins and owners can perform any action".to_string()),
        )
        .await?;

    println!("✓ Policy updated successfully\n");

    // 8. Demonstrate cleanup (delete policies)
    println!("Cleaning up...");
    client
        .delete_policy(&policy_store_id, policy_1_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to delete policy 1: {}", e);
            e
        })?;
    println!("✓ Deleted policy 1");

    client
        .delete_policy(&policy_store_id, policy_2_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to delete policy 2: {}", e);
            e
        })?;
    println!("✓ Deleted policy 2");

    client
        .delete_policy(&policy_store_id, policy_3_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to delete policy 3: {}", e);
            e
        })?;
    println!("✓ Deleted policy 3");

    // 9. Delete the policy store
    println!("\nDeleting policy store...");
    client
        .delete_policy_store(&policy_store_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to delete policy store: {}", e);
            e
        })?;

    println!("✓ Policy store deleted successfully");

    println!("\n=== Example completed successfully! ===");

    Ok(())
}
