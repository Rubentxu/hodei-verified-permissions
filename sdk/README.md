# Hodei Permissions SDK

Ergonomic Rust client SDK for [Hodei Verified Permissions](../README.md) authorization service.

## Features

- 🚀 **Simple API**: Easy-to-use methods for all operations
- 🔧 **Builder Patterns**: Fluent API for complex requests
- ⚡ **Async/Await**: Built on Tokio for high performance
- 🛡️ **Type Safe**: Leverages Rust's type system
- 📝 **Well Documented**: Comprehensive examples and docs

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hodei-permissions-sdk = { path = "../sdk" }
tokio = { version = "1.40", features = ["full"] }
```

## Quick Start

```rust
use hodei_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the service
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // Check authorization
    let response = client
        .is_authorized(
            "policy-store-id",
            "User::alice",
            "Action::view",
            "Document::doc123"
        )
        .await?;

    if response.decision() == hodei_permissions_sdk::Decision::Allow {
        println!("Access granted!");
    } else {
        println!("Access denied!");
    }

    Ok(())
}
```

## Usage Examples

### Create a Policy Store

```rust
let store = client
    .create_policy_store(Some("My Application".to_string()))
    .await?;

println!("Policy Store ID: {}", store.policy_store_id);
```

### Upload a Schema

```rust
let schema = r#"{
    "MyApp": {
        "entityTypes": {
            "User": {},
            "Document": {}
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

client.put_schema(&policy_store_id, schema).await?;
```

### Create a Policy

```rust
let policy = r#"
permit(principal, action == Action::"view", resource) 
when { resource.owner == principal };
"#;

client.create_policy(
    &policy_store_id,
    "allow-owners",
    policy,
    Some("Allow owners to view their documents".to_string())
).await?;
```

### Authorization with Entities

```rust
use hodei_permissions_sdk::{EntityBuilder, IsAuthorizedRequestBuilder};

// Build entities
let user = EntityBuilder::new("User", "alice")
    .attribute("department", r#""engineering""#)
    .build();

let doc = EntityBuilder::new("Document", "doc123")
    .attribute("owner", r#"{"__entity": {"type": "User", "id": "alice"}}"#)
    .build();

// Build request
let request = IsAuthorizedRequestBuilder::new(&policy_store_id)
    .principal("User", "alice")
    .action("Action", "view")
    .resource("Document", "doc123")
    .add_entity(user)
    .add_entity(doc)
    .build();

let response = client.is_authorized_with_context(request).await?;
```

### Batch Authorization

```rust
let requests = vec![
    IsAuthorizedRequestBuilder::new(&policy_store_id)
        .principal("User", "alice")
        .action("Action", "view")
        .resource("Document", "doc1")
        .build(),
    IsAuthorizedRequestBuilder::new(&policy_store_id)
        .principal("User", "alice")
        .action("Action", "edit")
        .resource("Document", "doc1")
        .build(),
];

let responses = client.batch_is_authorized(&policy_store_id, requests).await?;

for response in responses.responses {
    println!("Decision: {:?}", response.decision());
}
```

## API Overview

### Data Plane (Authorization)

- `is_authorized()` - Simple authorization check
- `is_authorized_with_context()` - Authorization with entities and context
- `batch_is_authorized()` - Multiple authorization checks

### Control Plane (Management)

**Policy Stores:**
- `create_policy_store()`
- `get_policy_store()`
- `list_policy_stores()`
- `delete_policy_store()`

**Schemas:**
- `put_schema()`
- `get_schema()`

**Policies:**
- `create_policy()`
- `get_policy()`
- `list_policies()`
- `delete_policy()`

## Builder Patterns

### IsAuthorizedRequestBuilder

```rust
let request = IsAuthorizedRequestBuilder::new(policy_store_id)
    .principal("User", "alice")
    .action("Action", "view")
    .resource("Document", "doc123")
    .context(r#"{"ip": "192.168.1.1"}"#)
    .add_entity(entity1)
    .add_entity(entity2)
    .build();
```

### EntityBuilder

```rust
let entity = EntityBuilder::new("User", "alice")
    .attribute("department", r#""engineering""#)
    .attribute("level", "5")
    .parent("Group", "admins")
    .build();
```

## Error Handling

```rust
use hodei_permissions_sdk::SdkError;

match client.is_authorized(...).await {
    Ok(response) => println!("Decision: {:?}", response.decision()),
    Err(SdkError::ConnectionError(e)) => eprintln!("Connection failed: {}", e),
    Err(SdkError::StatusError(status)) => eprintln!("gRPC error: {}", status),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Examples

Run the complete example:

```bash
# Start the server first
cd .. && cargo run --release

# In another terminal, run the example
cd sdk && cargo run --example basic_usage
```

## Documentation

Generate and view the API documentation:

```bash
cargo doc --open
```

## Testing

```bash
cargo test
```

## License

MIT
