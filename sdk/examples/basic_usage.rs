//! Basic SDK usage example

use hodei_permissions_sdk::{AuthorizationClient, EntityBuilder, IsAuthorizedRequestBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the service
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // Create a policy store
    let store = client
        .create_policy_store(Some("My Application".to_string()))
        .await?;
    println!("Created policy store: {}", store.policy_store_id);

    let policy_store_id = &store.policy_store_id;

    // Put a schema
    let schema = r#"{
        "MyApp": {
            "entityTypes": {
                "User": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "department": {"type": "String"}
                        }
                    }
                },
                "Document": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "owner": {"type": "Entity", "name": "User"}
                        }
                    }
                }
            },
            "actions": {
                "view": {
                    "appliesTo": {
                        "principalTypes": ["User"],
                        "resourceTypes": ["Document"]
                    }
                }
            }
        }
    }"#;

    client.put_schema(policy_store_id, schema).await?;
    println!("Schema uploaded");

    // Create a policy
    let policy = r#"permit(principal, action == Action::"view", resource) 
        when { resource.owner == principal };"#;

    client
        .create_policy(
            policy_store_id,
            "allow-owners",
            policy,
            Some("Allow owners to view their documents".to_string()),
        )
        .await?;
    println!("Policy created");

    // Simple authorization check
    let response = client
        .is_authorized(
            policy_store_id,
            "User::alice",
            "Action::view",
            "Document::doc123",
        )
        .await?;

    println!("Simple check - Decision: {:?}", response.decision());

    // Authorization with entities
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

    println!("With entities - Decision: {:?}", response.decision());
    println!("Determining policies: {:?}", response.determining_policies);

    // List policies
    let policies = client.list_policies(policy_store_id).await?;
    println!("Policies in store: {}", policies.policies.len());

    // Cleanup
    client.delete_policy_store(policy_store_id).await?;
    println!("Policy store deleted");

    Ok(())
}
