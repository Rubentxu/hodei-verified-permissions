//! Basic SDK usage example - Data Plane operations only
//!
//! This example demonstrates the lightweight SDK focused on authorization checking.
//! For Control Plane operations (policy store, schema, policy management), use:
//! - CLI Tool: `hodei` command
//! - Library: `hodei_cli` crate with `HodeiAdmin` struct

use hodei_permissions_sdk::{AuthorizationClient, EntityBuilder, IsAuthorizedRequestBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the service
    // Make sure you have a running Hodei service and configured policy store
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // You need a pre-configured policy store ID (created via CLI or HodeiAdmin)
    let policy_store_id = "your-policy-store-id";

    // ===== Data Plane Operations =====

    // Simple authorization check
    println!("1. Simple authorization check:");
    let response = client
        .is_authorized(
            policy_store_id,
            "User::alice",
            "Action::view",
            "Document::doc123",
        )
        .await?;

    println!("   Decision: {:?}", response.decision());

    // Authorization with entities (context)
    println!("\n2. Authorization with entities (context):");
    let alice = EntityBuilder::new("User", "alice")
        .attribute("department", r#""engineering""#)
        .build();

    let doc = EntityBuilder::new("Document", "doc123")
        .attribute("owner", r#"{"__entity": {"type": "User", "id": "alice"}}"#)
        .build();

    let request = IsAuthorizedRequestBuilder::new(policy_store_id)
        .principal("User", "alice")
        .action("Action", "view")
        .resource("Document", "doc123")
        .add_entity(alice)
        .add_entity(doc)
        .build();

    let response = client.is_authorized_with_context(request).await?;

    println!("   Decision: {:?}", response.decision());
    println!(
        "   Determining policies: {:?}",
        response.determining_policies
    );

    // Batch authorization checks
    println!("\n3. Batch authorization checks:");
    let batch_request = vec![
        ("User::alice", "Action::view", "Document::doc123"),
        ("User::bob", "Action::edit", "Document::doc123"),
        ("User::alice", "Action::delete", "Document::doc456"),
    ];

    let batch_response = client
        .batch_is_authorized(policy_store_id, &batch_request)
        .await?;

    println!("   Batch results:");
    for (i, result) in batch_response.results.iter().enumerate() {
        println!("   - Check {}: {:?}", i + 1, result.decision());
    }

    // JWT token authorization
    println!("\n4. JWT token authorization:");
    println!("   (Requires pre-configured identity source and valid JWT token)");

    let jwt_token = "your-jwt-token-here";

    // Note: This requires the policy store to have an identity source configured
    // Use the CLI tool or HodeiAdmin library to set up identity sources
    #[cfg(feature = "compat")]
    {
        println!("   Note: For identity source configuration, use:");
        println!(
            "   CLI: hodei identity-source create --store-id={} --type=cognito ...",
            policy_store_id
        );
        println!("   Or use the hodei_cli::HodeiAdmin library programmatically");
    }

    // This is a simple example - in production you would extract the JWT from the request
    // and the middleware would handle the authorization automatically

    println!("\n✅ All authorization checks completed!");
    println!("\n📚 For Control Plane operations:");
    println!("   - Create policy stores: CLI 'hodei init' or HodeiAdmin::create_policy_store()");
    println!("   - Upload schemas: CLI 'hodei schema apply' or HodeiAdmin::put_schema()");
    println!("   - Manage policies: CLI 'hodei policy create' or HodeiAdmin::create_policy()");
    println!("   - Configure identity sources: CLI 'hodei identity-source create'");

    Ok(())
}
