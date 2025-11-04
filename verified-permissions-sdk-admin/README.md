# Hodei Admin SDK

Programmatic Control Plane client for Hodei Verified Permissions.

## Overview

The **Hodei Admin SDK** (`verified-permissions-sdk-admin`) provides a high-level Rust library for programmatically managing Hodei Verified Permissions resources. It enables full Control Plane operations including policy stores, schemas, and policies management.

This library is part of the **Hodei SDK v0.2.0** refactoring that separates Data Plane (authorization checking) from Control Plane (policy management) following industry best practices.

## Architecture

```
┌─────────────────────────────────────────────┐
│           Hodei Admin SDK                   │
│        (Control Plane Operations)           │
├─────────────────────────────────────────────┤
│  • Policy Store Management                  │
│  • Schema Upload                            │
│  • Policy CRUD Operations                   │
└─────────────────────────────────────────────┘
                    │
                    │ uses
                    ▼
┌─────────────────────────────────────────────┐
│        Hodei Permissions SDK                │
│         (Data Plane Only)                   │
├─────────────────────────────────────────────┤
│  • Authorization Checks                     │
│  • Token Validation                         │
│  • Batch Operations                         │
└─────────────────────────────────────────────┘
                    │
                    │ gRPC
                    ▼
┌─────────────────────────────────────────────┐
│      Hodei Verified Permissions             │
│           gRPC Server                        │
└─────────────────────────────────────────────┘
```

### Data Plane vs Control Plane

- **Data Plane** (`hodei-permissions-sdk`): Authorization checking, permission validation, token verification
- **Control Plane** (`verified-permissions-sdk-admin`): Policy store creation, schema management, policy lifecycle

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
verified-permissions-sdk-admin = { path = "path/to/hodei-verified-permissions/verified-permissions-sdk-admin" }
tokio = { version = "1.40", features = ["macros", "rt-multi-thread"] }
```

### Basic Usage

```rust
use sdk_admin::HodeiAdmin;
use hodei_permissions_sdk::proto::{CreatePolicyStoreResponse, CreatePolicyResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Hodei server
    let mut client = HodeiAdmin::connect("http://localhost:50051").await?;

    // Create a policy store
    let store: CreatePolicyStoreResponse = client
        .create_policy_store("my-store", Some("Production policy store".to_string()))
        .await?;

    let policy_store_id = store.policy_store_id;
    println!("Created policy store: {}", policy_store_id);

    // Upload schema
    let schema = r#"
entity User = {
    id: String,
    roles: Set<String>,
}

entity Resource = {
    id: String,
    owner_id: String,
}

entity Role {
    name: String,
    permissions: Set<String>
}
"#;

    client.put_schema(&policy_store_id, schema).await?;
    println!("Schema uploaded successfully");

    // Create a policy
    let policy: CreatePolicyResponse = client
        .create_policy(
            &policy_store_id,
            "admin-policy",
            r#"
permit(
    principal == User:user_id,
    action == "admin:*",
    resource == Resource:resource_id
) when {
    principal.roles.contains("admin")
}
"#,
            Some("Administrators can perform any action".to_string()),
        )
        .await?;

    println!("Created policy: {}", policy.policy_id);

    Ok(())
}
```

## API Reference

### HodeiAdmin

The main client struct for programmatic access to Control Plane operations.

```rust
pub struct HodeiAdmin {
    control_client: AuthorizationControlClient<Channel>,
}
```

#### Connection

```rust
// Connect to server
impl HodeiAdmin {
    pub async fn connect(endpoint: impl Into<String>) -> Result<Self, SdkAdminError>
}
```

**Parameters:**
- `endpoint`: gRPC server endpoint (e.g., "http://localhost:50051")

**Returns:** `Result<HodeiAdmin, SdkAdminError>`

### Policy Store Operations

#### Create Policy Store

```rust
impl HodeiAdmin {
    pub async fn create_policy_store(
        &mut self,
        name: impl Into<String>,
        description: Option<String>,
    ) -> Result<CreatePolicyStoreResponse, SdkAdminError>
}
```

**Parameters:**
- `name`: Policy store name (unique identifier)
- `description`: Optional description

**Returns:** `CreatePolicyStoreResponse` containing:
- `policy_store_id`: Unique identifier for the created store

**Example:**
```rust
let store = client
    .create_policy_store("production", Some("Production environment".to_string()))
    .await?;
```

#### Get Policy Store

```rust
pub async fn get_policy_store(
    &mut self,
    policy_store_id: impl Into<String>,
) -> Result<GetPolicyStoreResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: ID of policy store to retrieve

**Returns:** `GetPolicyStoreResponse` containing policy store details

#### List Policy Stores

```rust
pub async fn list_policy_stores(
    &mut self,
    max_results: Option<i32>,
    next_token: Option<String>,
) -> Result<ListPolicyStoresResponse, SdkAdminError>
```

**Parameters:**
- `max_results`: Optional maximum number of results to return
- `next_token`: Optional pagination token for fetching next page

**Returns:** `ListPolicyStoresResponse` containing:
- `policy_stores`: Vec of policy store summaries
- `next_token`: Token for next page (if pagination is used)

#### Delete Policy Store

```rust
pub async fn delete_policy_store(
    &mut self,
    policy_store_id: impl Into<String>,
) -> Result<DeletePolicyStoreResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: ID of policy store to delete

**Warning:** This operation permanently deletes the policy store and all its resources!

### Schema Operations

#### Upload Schema

```rust
pub async fn put_schema(
    &mut self,
    policy_store_id: impl Into<String>,
    schema: impl Into<String>,
) -> Result<PutSchemaResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Target policy store ID
- schema: CEDAR schema as JSON string

**Returns:** `PutSchemaResponse`

**Example:**
```rust
let schema = r#"{
    "entities": {
        "User": {
            "id": "String",
            "roles": "Set<String>"
        }
    }
}"#;

client.put_schema("store-123", schema).await?;
```

### Policy Operations

#### Create Policy

```rust
pub async fn create_policy(
    &mut self,
    policy_store_id: impl Into<String>,
    policy_id: impl Into<String>,
    statement: impl Into<String>,
    description: Option<String>,
) -> Result<CreatePolicyResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Target policy store ID
- `policy_id`: Unique policy identifier
- `statement`: CEDAR policy statement
- `description`: Optional policy description

**Returns:** `CreatePolicyResponse` containing created policy metadata

**Example:**
```rust
let policy = client
    .create_policy(
        &store_id,
        "user-access-policy",
        r#"
permit(
    principal == User:user_id,
    action == Action:"read",
    resource == Resource:resource_id
) when {
    principal.roles.has("read")
}
"#,
        Some("Users with read role can read resources".to_string()),
    )
    .await?;
```

#### Get Policy

```rust
pub async fn get_policy(
    &mut self,
    policy_store_id: impl Into<String>,
    policy_id: impl Into<String>,
) -> Result<GetPolicyResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Policy store ID
- `policy_id`: Policy ID to retrieve

**Returns:** `GetPolicyResponse` containing policy details

#### List Policies

```rust
pub async fn list_policies(
    &mut self,
    policy_store_id: impl Into<String>,
) -> Result<ListPoliciesResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Policy store ID

**Returns:** `ListPoliciesResponse` containing:
- `policies`: Vec of policy summaries

#### Update Policy

```rust
pub async fn update_policy(
    &mut self,
    policy_store_id: impl Into<String>,
    policy_id: impl Into<String>,
    statement: impl Into<String>,
    description: Option<String>,
) -> Result<UpdatePolicyResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Policy store ID
- `policy_id`: Policy ID to update
- `statement`: New CEDAR policy statement
- `description`: Optional updated description

**Returns:** `UpdatePolicyResponse`

#### Delete Policy

```rust
pub async fn delete_policy(
    &mut self,
    policy_store_id: impl Into<String>,
    policy_id: impl Into<String>,
) -> Result<DeletePolicyResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Policy store ID
- `policy_id`: Policy ID to delete

**Warning:** This operation permanently deletes the policy!

### Bulk Operations (v0.2.0)

The Admin SDK provides bulk operations for efficient batch processing of multiple policies. These operations significantly reduce network overhead by consolidating multiple operations into a single gRPC call.

#### Batch Create Policies

```rust
pub async fn batch_create_policies(
    &mut self,
    policy_store_id: impl Into<String>,
    policies: Vec<(impl Into<String>, impl Into<String>, Option<String>)>,
) -> Result<BatchCreatePoliciesResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Target policy store ID
- `policies`: Vec of tuples containing (policy_id, statement, description)

**Returns:** `BatchCreatePoliciesResponse` containing individual operation results

**Example:**
```rust
let policies = vec![
    ("policy-1", "permit(...);", Some("First policy".to_string())),
    ("policy-2", "forbid(...);", Some("Second policy".to_string())),
    ("policy-3", "permit(...);", None),
];

let response = client.batch_create_policies(&store_id, policies).await?;

for result in response.results {
    if let Some(error) = result.error {
        println!("Failed to create policy '{}': {}", result.policy_id, error);
    } else {
        println!("Created policy '{}' at {}", result.policy_id, result.created_at);
    }
}
```

**Benefits:**
- Reduces from N API calls to 1
- Atomic per-operation (partial failures don't block successful operations)
- Detailed error reporting per policy

#### Batch Update Policies

```rust
pub async fn batch_update_policies(
    &mut self,
    policy_store_id: impl Into<String>,
    policies: Vec<(impl Into<String>, impl Into<String>, Option<String>)>,
) -> Result<BatchUpdatePoliciesResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Target policy store ID
- `policies`: Vec of tuples containing (policy_id, statement, description)

**Returns:** `BatchUpdatePoliciesResponse` with individual update results

**Example:**
```rust
let updates = vec![
    ("policy-1", "updated permit(...);", Some("Updated policy".to_string())),
];

let response = client.batch_update_policies(&store_id, updates).await?;

for result in response.results {
    if let Some(error) = result.error {
        println!("Update failed for '{}': {}", result.policy_id, error);
    } else {
        println!("Updated policy '{}' at {}", result.policy_id, result.updated_at);
    }
}
```

#### Batch Delete Policies

```rust
pub async fn batch_delete_policies(
    &mut self,
    policy_store_id: impl Into<String>,
    policy_ids: Vec<impl Into<String>>,
) -> Result<BatchDeletePoliciesResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Target policy store ID
- `policy_ids`: Vec of policy IDs to delete

**Returns:** `BatchDeletePoliciesResponse` with individual deletion results

**Example:**
```rust
let to_delete = vec!["policy-1", "policy-2", "policy-3"];

let response = client.batch_delete_policies(&store_id, to_delete).await?;

let mut deleted = 0;
let mut failed = 0;

for result in response.results {
    if let Some(error) = &result.error {
        failed += 1;
        println!("Delete failed for '{}': {}", result.policy_id, error);
    } else {
        deleted += 1;
        println!("Deleted policy: {}", result.policy_id);
    }
}

println!("Successfully deleted {} policies, {} failed", deleted, failed);
```

#### Test Authorization (Playground Mode)

```rust
pub async fn test_authorization(
    &mut self,
    policies: Vec<String>,
    principal: EntityIdentifier,
    action: EntityIdentifier,
    resource: EntityIdentifier,
    context: Option<String>,
) -> Result<TestAuthorizationResponse, SdkAdminError>
```

**Parameters:**
- `policies`: Vec of CEDAR policy statements (temporary, not persisted)
- `principal`: Entity identifier making the request
- `action`: Action being performed
- `resource`: Resource being accessed
- `context`: Optional context as JSON string

**Returns:** `TestAuthorizationResponse` with decision and diagnostics

**Example:**
```rust
let test_policies = vec![
    r#"permit(principal == User:"alice", action, resource);"#.to_string(),
    r#"forbid(principal == User:"bob", action, resource);"#.to_string(),
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

let response = client
    .test_authorization(test_policies, principal, action, resource, None)
    .await?;

println!("Decision: {:?}", response.decision);
println!("Determining policies: {:?}", response.determining_policies);
```

**Use Cases:**
- Test policies before deploying to production
- Validate policy behavior with different scenarios
- Debug authorization logic
- Integration testing

#### Validate Policy

```rust
pub async fn validate_policy(
    &mut self,
    policy_store_id: Option<String>,
    schema: Option<String>,
    policy_statement: String,
) -> Result<ValidatePolicyResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Optional policy store ID to get schema from
- `schema`: Optional schema string (if not using policy_store_id)
- `policy_statement`: CEDAR policy statement to validate

**Returns:** `ValidatePolicyResponse` with validation results

**Example:**
```rust
let policy = r#"permit(principal == User:"alice", action == Action:"read", resource);"#;

let response = client
    .validate_policy(Some(store_id), None, policy.to_string())
    .await?;

if response.is_valid {
    println!("Policy is valid!");
    if let Some(info) = &response.policy_info {
        println!("Effect: {}", info.effect);
        println!("Has conditions: {}", info.has_conditions);
    }
} else {
    for error in &response.errors {
        println!("Validation error: {} ({})", error.message, error.issue_type);
    }
}
```

**Use Cases:**
- Validate policy syntax before deployment
- Check policy compatibility with schema
- Linting and code review of policies

#### Batch Authorization Checks

```rust
pub async fn batch_is_authorized(
    &mut self,
    policy_store_id: impl Into<String>,
    requests: Vec<IsAuthorizedRequest>,
) -> Result<BatchIsAuthorizedResponse, SdkAdminError>
```

**Parameters:**
- `policy_store_id`: Target policy store ID
- `requests`: Vec of `IsAuthorizedRequest` structures

**Returns:** `BatchIsAuthorizedResponse` with decisions for all requests

**Example:**
```rust
let requests = vec![
    IsAuthorizedRequest {
        policy_store_id: store_id.clone(),
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
    },
    // ... more requests
];

let response = client.batch_is_authorized(&store_id, requests).await?;

for (index, resp) in response.responses.iter().enumerate() {
    println!("Request {}: {:?}", index + 1, resp.decision);
}
```

**Benefits:**
- Check multiple authorizations in parallel
- Efficient for user permission checks
- Consistent with Data Plane SDK's batch operation

### Error Handling

All methods return `Result<T, SdkAdminError>` where `SdkAdminError` is defined in `sdk_admin::error`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SdkAdminError {
    #[error("Connection failed: {0}")]
    ConnectionFailed,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("Unexpected error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
```

**Example error handling:**
```rust
match client.create_policy_store("test", None).await {
    Ok(store) => println!("Created: {}", store.policy_store_id),
    Err(SdkAdminError::ConnectionFailed) => {
        eprintln!("Failed to connect to server");
        std::process::exit(1);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        return Err(e.into());
    }
}
```

## Usage Patterns

### Complete Setup Flow

```rust
async fn setup_policy_store(
    client: &mut HodeiAdmin,
    store_name: &str,
) -> Result<String, SdkAdminError> {
    // Create policy store
    let store = client
        .create_policy_store(store_name, Some("Production store".to_string()))
        .await?;
    let store_id = store.policy_store_id;

    // Upload schema
    let schema = r#"{"entities": {...}}"#;
    client.put_schema(&store_id, schema).await?;

    // Create initial policies
    client
        .create_policy(
            &store_id,
            "default-policy",
            "permit(...)",
            Some("Default access policy".to_string()),
        )
        .await?;

    Ok(store_id)
}
```

### Batch Operations

```rust
async fn import_policies(
    client: &mut HodeiAdmin,
    store_id: &str,
    policies: Vec<(&str, &str)>,
) -> Result<(), SdkAdminError> {
    for (policy_id, statement) in policies {
        client
            .create_policy(store_id, policy_id, statement, None)
            .await
            .map(|p| println!("Imported policy: {}", p.policy_id))?;
    }
    Ok(())
}
```

### Policy Store Management

```rust
async fn list_all_stores(client: &mut HodeiAdmin) -> Result<(), SdkAdminError> {
    let mut next_token = None;
    loop {
        let response = client.list_policy_stores(None, next_token).await?;
        for store in &response.policy_stores {
            println!("Store: {} - {}", store.name, store.policy_store_id);
        }

        if let Some(token) = &response.next_token {
            next_token = Some(token.clone());
        } else {
            break;
        }
    }
    Ok(())
}
```

## Integration Examples

### With Authorization SDK

```rust
use hodei_permissions_sdk::{Client, IsAuthorizedRequestBuilder, EntityBuilder};

// Setup admin client for management
let mut admin = HodeiAdmin::connect("http://localhost:50051").await?;

// Setup SDK client for authorization checks
let sdk = Client::connect("http://localhost:50051").await?;

// Create and configure policy store
let store = admin
    .create_policy_store("my-app", Some("My Application".to_string()))
    .await?;

// ... upload schema and create policies ...

// Now use SDK for authorization
let request = IsAuthorizedRequestBuilder::new("read", "document:123")
    .with_principal(EntityBuilder::new("User", "user-456").build())
    .with_context(serde_json::json!({"department": "engineering"}))
    .build();

let response = sdk.is_authorized(&store.policy_store_id, request).await?;

if response.decision == "Allow" {
    println!("Access granted!");
}
```

### Web Server Integration (Axum)

```rust
use axum::{extract::State, post, Json};
use std::sync::Arc;

struct AppState {
    admin: Arc<tokio::sync::Mutex<HodeiAdmin>>,
}

#[post("/policy-stores")]
async fn create_store(
    State(state): State<AppState>,
    Json(payload): Json<CreateStoreRequest>,
) -> Result<Json<CreateStoreResponse>, String> {
    let mut admin = state.admin.lock().await;
    match admin.create_policy_store(&payload.name, payload.description).await {
        Ok(store) => Ok(Json(CreateStoreResponse {
            store_id: store.policy_store_id,
        })),
        Err(e) => Err(e.to_string()),
    }
}
```

### Background Job Processing

```rust
use tokio::time::{sleep, Duration};

async fn policy_rotation_job(client: &mut HodeiAdmin) -> Result<(), SdkAdminError> {
    loop {
        // List all policy stores
        let stores = client.list_policy_stores(None, None).await?;
        for store in stores.policy_stores {
            // Check if rotation is needed (e.g., based on timestamp in description)
            // ...

            // Update policies if needed
            client
                .update_policy(
                    &store.policy_store_id,
                    "rotating-policy",
                    "new policy statement",
                    None,
                )
                .await?;
        }

        // Wait before next rotation cycle
        sleep(Duration::from_secs(3600)).await;
    }
}
```

## Migration from v0.1.x

If you were using the monolithic SDK in v0.1.x with Control Plane operations:

### Before (v0.1.x - Deprecated)

```rust
use hodei_permissions_sdk::Client;

let mut client = Client::connect("http://localhost:50051").await?;

// This no longer works in v0.2.0
let _store = client.create_policy_store("test", None).await?;
```

### After (v0.2.0)

```rust
use sdk_admin::HodeiAdmin;

// For Control Plane operations
let mut admin = HodeiAdmin::connect("http://localhost:50051").await?;
let store = admin.create_policy_store("test", None).await?;

// For Data Plane operations
use hodei_permissions_sdk::Client;
let sdk = Client::connect("http://localhost:50051").await?;

// Use SDK for authorization checks only
let _response = sdk.is_authorized(&store.policy_store_id, request).await?;
```

For detailed migration instructions, see [MIGRATION_GUIDE_SDK.md].

## CLI Integration

The Hodei CLI binary (`hodei`) is built on top of this library. If you need programmatic control with CLI-like functionality:

```rust
// The CLI uses verified-permissions-sdk-admin internally
// All CLI commands can be replicated using HodeiAdmin API
```

For CLI usage, see: `hodei --help`

## Version Compatibility

| SDK Version | Admin SDK | Protocol Buffer |
|-------------|-----------|-----------------|
| 0.2.0       | 0.1.0     | v1              |
| 0.1.x       | N/A       | v1              |

## Best Practices

### 1. Connection Management

```rust
// Reuse connection
let mut admin = HodeiAdmin::connect(endpoint).await?;

// Don't create new connection for each operation
for policy in policies {
    admin.create_policy(&store_id, policy.id, statement, None).await?;
}
```

### 2. Error Handling

```rust
match client.create_policy(store_id, policy_id, statement, None).await {
    Ok(policy) => {
        println!("Created policy: {}", policy.policy_id);
    }
    Err(SdkAdminError::Grpc(status)) => {
        // Handle specific gRPC errors
        match status.code() {
            tonic::Code::AlreadyExists => {
                println!("Policy already exists");
            }
            _ => return Err(e),
        }
    }
    Err(e) => return Err(e.into()),
}
```

### 3. Schema Validation

```rust
// Validate schema before uploading
fn validate_schema(schema: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _: Schema = serde_json::from_str(schema)?;
    Ok(())
}

// Then upload
client.put_schema(store_id, schema).await?;
```

### 4. Policy Statements

```rust
// Use raw string literals for multi-line CEDAR statements
let policy_statement = r#"
permit(
    principal == User:user_id,
    action == Action:"read",
    resource == Resource:resource_id
) when {
    principal.roles.has("read")
}
"#;

client
    .create_policy(store_id, "policy-1", policy_statement, None)
    .await?;
```

## Troubleshooting

### Connection Issues

```rust
// Test connection
let admin = HodeiAdmin::connect("http://localhost:50051").await
    .map_err(|e| {
        eprintln!("Connection failed. Is the server running?");
        eprintln!("Error: {}", e);
        e
    })?;
```

### Authentication

```rust
// If using authentication, configure endpoint accordingly
let endpoint = "https://api.example.com:443";
let mut admin = HodeiAdmin::connect(endpoint).await?;

// Ensure TLS is configured if required
```

### Debug Logging

```rust
use tracing_subscriber;

tracing_subscriber::fmt::init();

// Operations will now log to stdout
let admin = HodeiAdmin::connect("http://localhost:50051").await?;
```

## License

MIT

## Support

- Documentation: [Hodei Docs](https://hodei.dev)
- GitHub Issues: [Report bugs](https://github.com/hodei/verified-permissions/issues)
- Changelog: See [CHANGELOG.md]

## Related

- **Hodei Permissions SDK** (`hodei-permissions-sdk`): Data Plane operations for authorization
- **Hodei CLI** (`hodei`): Command-line interface built on top of verified-permissions-sdk-admin
- **Migration Guide**: [MIGRATION_GUIDE_SDK.md]
