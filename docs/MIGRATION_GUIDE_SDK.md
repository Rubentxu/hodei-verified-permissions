# Migration Guide: SDK v0.1.x to v0.2.0

## Overview

This guide helps you migrate from Hodei Permissions SDK v0.1.x to v0.2.0. The major change in v0.2.0 is the separation of **Data Plane** and **Control Plane** operations for a cleaner, more maintainable architecture.

## What's Changed

### v0.1.x: Monolithic SDK
- All operations (authorization + management) in one SDK
- `AuthorizationClient` included both Data Plane and Control Plane methods
- Heavy and over-engineered for typical use cases

### v0.2.0: Clean Separation

#### Data Plane (`hodei-permissions-sdk`)
**Purpose**: Authorization checking only
- `is_authorized()` - Check permissions
- `batch_is_authorized()` - Bulk authorization checks
- `is_authorized_with_context()` - Authorization with entities
- `is_authorized_with_token()` - JWT token authorization

**When to use**: In your application for authorization checks

#### Control Plane (`sdk-admin`)
**Purpose**: Policy and schema management
- Policy store CRUD operations
- Schema management
- Policy CRUD operations
- Identity source management

**When to use**: For setup, configuration, and management (can be done via CLI or programmatically)

## Migration Steps

### Step 1: Update Dependencies

Update your `Cargo.toml`:

```toml
# Before (v0.1.x)
hodei-permissions-sdk = "0.1"

# After (v0.2.x)
hodei-permissions-sdk = "0.2"
```

### Step 2: Identify Control Plane Operations

Search your codebase for these operations (they need to be moved):

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
- `update_policy()`
- `delete_policy()`

**Identity Sources:**
- `create_identity_source()`
- `get_identity_source()`
- `list_identity_sources()`
- `delete_identity_source()`

### Step 3: Choose Your Migration Path

#### Option A: Use CLI Tool (Recommended for DevOps)

Move all Control Plane operations to CLI commands:

```bash
# Before: In your Rust code
let store = client.create_policy_store("MyApp", None).await?;
client.put_schema(&store.policy_store_id, schema).await?;
client.create_policy(&store.policy_store_id, "policy1", statement).await?;

// After: CLI commands
hodei init my-app
hodei schema apply --file=schema.json --store-id=...
hodei policy create --store-id=... --id=policy1 --statement='...'
```

#### Option B: Use SDK Admin Library (Programmatic)

Add `sdk-admin` dependency:

```toml
[dependencies]
sdk-admin = "0.1"
hodei-permissions-sdk = "0.2"
```

Update your code:

```rust
// Before (v0.1.x): All in one client
let store = client.create_policy_store("MyApp", None).await?;
let response = client.is_authorized(&store.policy_store_id, ...).await?;

// After (v0.2.x): Separate clients
use sdk_admin::HodeiAdmin;
use hodei_permissions_sdk::AuthorizationClient;

// Setup (once, or in a separate process)
let mut admin = HodeiAdmin::connect("http://localhost:50051").await?;
let store = admin.create_policy_store("MyApp", None).await?;

// In your application: Data Plane only
let client = AuthorizationClient::connect("http://localhost:50051").await?;
let response = client.is_authorized(&store.policy_store_id, ...).await?;
```

### Step 4: Update Code Examples

#### Before (v0.1.x)

```rust
use hodei_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // Create policy store
    let store = client
        .create_policy_store(Some("My App".to_string()))
        .await?;

    // Upload schema
    client.put_schema(&store.policy_store_id, schema).await?;

    // Create policy
    client
        .create_policy(&store.policy_store_id, "policy1", statement, None)
        .await?;

    // Check authorization
    let response = client
        .is_authorized(&store.policy_store_id, "User::alice", "Action::view", "Document::doc1")
        .await?;

    Ok(())
}
```

#### After (v0.2.x) - CLI Approach

```rust
// setup.sh (or run these commands)
/*
hodei init my-app
hodei schema apply --file=schema.json
hodei policy create --store-id=... --id=policy1 --statement='...'
*/

use hodei_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In your application: Data Plane only
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // Check authorization
    let response = client
        .is_authorized("pre-configured-store-id", "User::alice", "Action::view", "Document::doc1")
        .await?;

    Ok(())
}
```

#### After (v0.2.x) - SDK Admin Approach

```rust
// Separate setup process (or admin module)
use sdk_admin::HodeiAdmin;

async fn setup_infrastructure() -> Result<String, Box<dyn std::error::Error>> {
    let mut admin = HodeiAdmin::connect("http://localhost:50051").await?;
    let store = admin.create_policy_store("MyApp", None).await?;
    admin.put_schema(&store.policy_store_id, schema).await?;
    admin.create_policy(&store.policy_store_id, "policy1", statement, None).await?;
    Ok(store.policy_store_id)
}

// In your application
use hodei_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store_id = setup_infrastructure().await?;

    let client = AuthorizationClient::connect("http://localhost:50051").await?;
    let response = client
        .is_authorized(&store_id, "User::alice", "Action::view", "Document::doc1")
        .await?;

    Ok(())
}
```

### Step 5: Handle JWT Token Authorization

If you're using JWT token authorization, note that identity source configuration is now a Control Plane operation:

```rust
// Before (v0.1.x): All in one place
let identity_source = client
    .create_identity_source(&store.policy_store_id, config, None, None)
    .await?;
let response = client
    .is_authorized_with_token(&store.policy_store_id, &identity_source.identity_source_id, jwt_token, "Action::view", "Document::doc1")
    .await?;

// After (v0.2.x): Separate concerns
// Setup (CLI or sdk-admin):
// hodei identity-source create --store-id=... --type=oidc --issuer=...

// In your application:
let response = client
    .is_authorized_with_token("store-id", "identity-source-id", jwt_token, "Action::view", "Document::doc1")
    .await?;
```

## Temporary Compatibility Layer

For gradual migration, you can enable the compatibility layer:

```toml
[dependencies]
hodei-permissions-sdk = { version = "0.2", features = ["compat"] }
```

This provides deprecated versions of old methods that return helpful error messages:

```rust
// This will compile but return an error with migration guidance
let store = client
    .create_policy_store(Some("MyApp".to_string()))
    .await;
// Error: Operation 'create_policy_store' is deprecated. Use CLI tool or sdk-admin library instead.
```

**Note**: The compatibility layer is temporary and will be removed in a future version.

## Common Migration Scenarios

### Scenario 1: Web Application with Middleware

**Before:**
```rust
// One client for everything
let client = AuthorizationClient::connect(...).await?;

// Setup
let store = client.create_policy_store(...).await?;

// Middleware
app.layer(VerifiedPermissionsLayer::new(client.clone(), store.policy_store_id));
```

**After:**
```rust
// Setup (CLI or separate process)
// hodei init my-app

// In your application: Data Plane only
let client = AuthorizationClient::connect(...).await?;

// Middleware
app.layer(VerifiedPermissionsLayer::new(client, "store-id-from-config"));
```

### Scenario 2: Microservice with Embedded Setup

**Before:**
```rust
// Service startup
let client = AuthorizationClient::connect(...).await?;
let store = client.create_policy_store(...).await?;
client.put_schema(...).await?;

// Check permissions
let authorized = client.is_authorized(...).await?;
```

**After:**
```rust
// Option A: External setup script
// run_setup.sh:
// hodei init my-app
// hodei schema apply ...
// export STORE_ID=...

// Service startup
let client = AuthorizationClient::connect(...).await?;

// Check permissions (no setup code in service)
let authorized = client.is_authorized(std::env::var("STORE_ID")?, ...).await?;

// Option B: Admin module
mod setup {
    use sdk_admin::HodeiAdmin;

    pub async fn ensure_infrastructure() -> Result<String> {
        let mut admin = HodeiAdmin::connect(...).await?;
        // Check if store exists, create if needed
        // Apply schema if needed
        Ok(store_id)
    }
}

let store_id = setup::ensure_infrastructure().await?;
let client = AuthorizationClient::connect(...).await?;
```

## Benefits of This Change

1. **Clearer Architecture**: Separation of concerns makes the codebase easier to understand and maintain
2. **Lighter SDK**: Applications only include what they need (authorization checking)
3. **Industry Standard**: Follows patterns from AWS Verified Permissions, Casbin, and Oso
4. **Better Tooling**: CLI tool for DevOps workflows
5. **Flexibility**: Choose between CLI or programmatic Control Plane access

## Troubleshooting

### Error: "operation is deprecated"

**Solution**: Move the operation to CLI or sdk-admin library

### Error: "No AuthorizationControlClient in proto"

**Solution**: Make sure you're using `sdk-admin` for Control Plane operations, not the main SDK

### Error: "Connection failed"

**Solution**: Verify your gRPC endpoint is correct and the service is running

### Compilation Errors

**Solution**: Check that you've updated all imports:
- Data Plane: `hodei_permissions_sdk`
- Control Plane: `sdk_admin`

## Getting Help

- Documentation: [docs.rs/hodei-permissions-sdk](https://docs.rs/hodei-permissions-sdk)
- SDK Admin: [docs.rs/sdk-admin](https://docs.rs/sdk-admin)
- CLI Help: Run `hodei --help`
- Issues: [GitHub Issues](https://github.com/rubentxu/hodei-verified-permissions/issues)

## Summary Checklist

- [ ] Update `Cargo.toml` to v0.2.0
- [ ] Identify Control Plane operations in your code
- [ ] Decide on CLI or sdk-admin approach
- [ ] Move Control Plane operations out of your application
- [ ] Update Data Plane usage (minimal changes needed)
- [ ] Test authorization checks still work
- [ ] Remove compatibility layer when ready
- [ ] Update documentation and runbooks

---

**Migration Support**: If you encounter issues during migration, please open an issue on GitHub with:
- Your current code (before migration)
- The error message you're seeing
- What you're trying to achieve

We'll help you through the migration!
