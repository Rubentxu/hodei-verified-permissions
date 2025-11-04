# Hodei Permissions SDK

Lightweight Rust client SDK for Hodei Verified Permissions authorization service.

**This SDK focuses exclusively on authorization checking (Data Plane).** For policy store, schema, and policy management (Control Plane), use the CLI tool or HodeiAdmin library.

## 📚 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Guide](#usage-guide)
  - [Basic Authorization](#basic-authorization)
  - [Authorization with Context](#authorization-with-context)
  - [Batch Authorization](#batch-authorization)
  - [JWT Token Authorization](#jwt-token-authorization)
  - [Middleware (Axum/Tower)](#middleware-axumtower)
- [API Reference](#api-reference)
- [Migration from v0.1.x](#migration-from-v01x)
- [For Developers](#for-developers)
- [Testing](#testing)

## 🎯 Overview

### Architecture

**Data Plane vs Control Plane Separation:**

- **Data Plane (This SDK)**: Authorization checking, JWT validation, batch operations
  - Lightweight and focused on authorization decisions
  - Optimized for high-frequency authorization checks
  - Perfect for middleware integration

- **Control Plane (CLI/HodeiAdmin)**: Policy management, schema configuration, identity sources
  - Used for one-time setup and configuration
  - Full management of policy stores, schemas, policies
  - Available via CLI tool or HodeiAdmin library

### When to Use This SDK

✅ **Use this SDK when:**
- You need to check permissions/authorization
- You're integrating authorization into your application
- You need JWT token validation
- You want to protect HTTP routes with middleware

❌ **Use CLI/HodeiAdmin when:**
- Creating or managing policy stores
- Uploading or updating schemas
- Creating or managing policies
- Configuring identity sources (OIDC, Cognito, etc.)

## ✨ Features

- 🚀 **Lightweight**: Data Plane only, no bloat
- 🔐 **Authorization Checks**: Simple and context-aware authorization
- ⚡ **Batch Operations**: Check multiple authorizations efficiently
- 🔑 **JWT Support**: Built-in JWT token validation with Identity Sources
- 🏗️ **Builder Patterns**: Fluent API for complex requests
- ⚡ **Async/Await**: Built on Tokio for high performance
- 🛡️ **Type Safe**: Leverages Rust's type system
- 📝 **Well Documented**: Comprehensive examples and docs
- 🌐 **IdP Integration**: Works with Keycloak, Zitadel, AWS Cognito
- 🔌 **Middleware**: Optional Axum/Tower middleware (feature flag)
- 🎯 **Cedar Policies**: Full support for Cedar policy language
- 🔄 **Migration Support**: Compatibility layer for v0.1.x users

## 📦 Installation

### Basic Installation

```toml
[dependencies]
hodei-permissions-sdk = "0.2"
tokio = { version = "1.40", features = ["full"] }
```

### With Middleware Support

```toml
[dependencies]
hodei-permissions-sdk = { version = "0.2", features = ["middleware"] }
axum = "0.7"
tower = "0.5"
```

### For Migration from v0.1.x

If you're migrating from v0.1.x and need temporary compatibility:

```toml
[dependencies]
hodei-permissions-sdk = { version = "0.2", features = ["compat"] }
```

**Note**: The compatibility layer is deprecated. See [Migration Guide](#migration-from-v01x) for details.

## 🚀 Quick Start

```rust
use hodei_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the service
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // Check authorization (you need a pre-configured policy store)
    let response = client
        .is_authorized(
            "your-policy-store-id",
            "User::alice",
            "Action::view",
            "Document::doc123"
        )
        .await?;

    if response.decision() == hodei_permissions_sdk::Decision::Allow {
        println!("✅ Access granted!");
    } else {
        println!("❌ Access denied!");
    }

    Ok(())
}
```

**Note**: Before using the SDK, you need to:
1. Set up a policy store using CLI: `hodei init my-app`
2. Create schemas using CLI: `hodei schema apply --file=schema.json`
3. Create policies using CLI: `hodei policy create --store-id=... --statement=...`

## 📖 Usage Guide

### Basic Authorization

The simplest form of authorization check:

```rust
let response = client
    .is_authorized(
        "policy-store-id",
        "User::alice",
        "Action::view",
        "Document::doc123"
    )
    .await?;

match response.decision() {
    Decision::Allow => println!("Access granted!"),
    Decision::Deny => println!("Access denied!"),
    Decision::NotApplicable => println!("No policy applies"),
    Decision::Indeterminate => println!("Unable to determine"),
}
```

### Authorization with Context

For complex authorization with entity attributes:

```rust
use hodei_permissions_sdk::{EntityBuilder, IsAuthorizedRequestBuilder};

let alice = EntityBuilder::new("User", "alice")
    .attribute("department", r#""engineering""#)
    .attribute("level", "5")
    .build();

let doc = EntityBuilder::new("Document", "doc123")
    .attribute("owner", r#"{"__entity": {"type": "User", "id": "alice"}}"#)
    .attribute("visibility", r#""public""#)
    .build();

let request = IsAuthorizedRequestBuilder::new("policy-store-id")
    .principal("User", "alice")
    .action("Action", "view")
    .resource("Document", "doc123")
    .add_entity(alice)
    .add_entity(doc)
    .context(r#"{"ip": "192.168.1.1", "time": "2024-01-01T00:00:00Z"}"#)
    .build();

let response = client.is_authorized_with_context(request).await?;
```

### Batch Authorization

Check multiple authorizations efficiently:

```rust
let checks = vec![
    ("User::alice", "Action::view", "Document::doc1"),
    ("User::bob", "Action::edit", "Document::doc1"),
    ("User::alice", "Action::delete", "Document::doc2"),
];

let batch_response = client
    .batch_is_authorized("policy-store-id", &checks)
    .await?;

for (i, result) in batch_response.results.iter().enumerate() {
    println!("Check {}: {:?}", i + 1, result.decision());
}
```

### JWT Token Authorization

**Prerequisites**: Configure identity sources using CLI or HodeiAdmin:

```bash
hodei identity-source create \
  --store-id=your-store-id \
  --type=oidc \
  --issuer=https://your-idp.com \
  --client-id=your-client-id \
  --jwks-uri=https://your-idp.com/.well-known/jwks.json
```

Then authorize with JWT tokens:

```rust
// Extract JWT from request (e.g., Authorization: Bearer header)
let jwt_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";

let response = client
    .is_authorized_with_token(
        "policy-store-id",
        "identity-source-id",  // From CLI: hodei identity-source list
        jwt_token,
        "Action::view",
        "Document::doc123"
    )
    .await?;
```

### Middleware (Axum/Tower)

Protect your HTTP routes with automatic authorization:

```rust
use hodei_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
use axum::{Router, routing::get, Json};

#[tokio::main]
async fn main() {
    // Create authorization client
    let client = AuthorizationClient::connect("http://localhost:50051")
        .await
        .unwrap();

    // Create middleware layer
    let auth_layer = VerifiedPermissionsLayer::new(
        client,
        "policy-store-id",
        "identity-source-id"
    );

    // Build your app with middleware
    let app = Router::new()
        .route("/api/documents", get(list_documents))
        .route("/api/documents/:id", get(get_document))
        .layer(auth_layer);

    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        app
    ).await.unwrap();
}

async fn list_documents() -> Json<Vec<String>> {
    // Middleware handles authorization automatically
    // Only authorized requests reach here
    Json(vec!["doc1".to_string(), "doc2".to_string()])
}
```

The middleware:
1. Extracts JWT from `Authorization: Bearer` header
2. Maps HTTP method to action (GET → read, POST → create, etc.)
3. Uses URI path as resource
4. Calls authorization service
5. Returns **403 Forbidden** on Deny
6. Forwards request to handler on Allow

## 📚 API Reference

### Core Operations

| Method | Description | Use Case |
|--------|-------------|----------|
| `is_authorized()` | Simple authorization check | Basic permission checks |
| `is_authorized_with_context()` | Authorization with entities and context | Complex authorization with attributes |
| `is_authorized_with_token()` | Authorization with JWT token | JWT token validation |
| `batch_is_authorized()` | Multiple authorization checks | Efficient bulk checking |

### Builder Patterns

#### IsAuthorizedRequestBuilder

```rust
IsAuthorizedRequestBuilder::new(policy_store_id)
    .principal("User", "alice")
    .action("Action", "view")
    .resource("Document", "doc123")
    .context(json_string)
    .add_entity(entity)
    .build()
```

#### EntityBuilder

```rust
EntityBuilder::new("User", "alice")
    .attribute("key", "value")
    .parent("Group", "admins")
    .build()
```

### Decision Types

```rust
pub enum Decision {
    Allow,           // Permission granted
    Deny,            // Permission denied
    NotApplicable,   // No policy applies
    Indeterminate,   // Unable to determine (missing context)
}
```

## 🔄 Migration from v0.1.x

### What's Changed

v0.2.0 introduces a **Data Plane only** architecture:
- ✅ **Kept**: Authorization checking methods (`is_authorized`, `batch_is_authorized`, etc.)
- ❌ **Removed**: Control Plane operations (`create_policy_store`, `put_schema`, `create_policy`, etc.)
- 🔄 **Replaced**: Control Plane moved to CLI tool and HodeiAdmin library

### Step 1: Use Compatibility Layer (Temporary)

Enable the `compat` feature flag:

```toml
hodei-permissions-sdk = { version = "0.2", features = ["compat"] }
```

This provides deprecated methods that return helpful error messages:

```rust
// This will compile but return an error with migration guidance
let store = client
    .create_policy_store(Some("MyApp".to_string()))
    .await;
```

### Step 2: Move Control Plane to CLI

For all Control Plane operations, use the CLI tool:

```bash
# Create policy store
hodei init my-app

# Upload schema
hodei schema apply --file=schema.json --store-id=...

# Create policy
hodei policy create \
  --store-id=... \
  --id=allow-alice \
  --statement='permit(principal == User::"alice", ...)'

# Configure identity source
hodei identity-source create \
  --store-id=... \
  --type=oidc \
  --issuer=https://auth.example.com \
  --client-id=myapp
```

### Step 3: Update SDK Usage

Remove Control Plane operations from your application code:

**Before (v0.1.x):**
```rust
// ❌ Don't do this in v0.2+
let store = client.create_policy_store(...).await?;
client.put_schema(&store.policy_store_id, schema).await?;
client.create_policy(&store.policy_store_id, "policy1", statement).await?;

// Only keep authorization checks
let response = client.is_authorized(...).await?;
```

**After (v0.2.x):**
```rust
// ✅ Authorization checks only
let response = client.is_authorized(
    "pre-configured-store-id",
    "User::alice",
    "Action::view",
    "Document::doc123"
).await?;
```

### Step 4: Programmatic Control Plane (Optional)

If you need programmatic Control Plane access, use the HodeiAdmin library:

```toml
hodei-cli = { version = "0.2", features = ["library"] }
```

```rust
use hodei_cli::HodeiAdmin;

let admin = HodeiAdmin::connect("http://localhost:50051").await?;

// Create policy store programmatically
let store = admin.create_policy_store("MyApp", None).await?;

// Upload schema programmatically
admin.put_schema(&store.policy_store_id, schema).await?;

// Create policy programmatically
admin.create_policy(
    &store.policy_store_id,
    "allow-alice",
    statement,
    None
).await?;
```

### Complete Migration Example

**Old approach (v0.1.x):**
```rust
let client = AuthorizationClient::connect(...).await?;

// Everything in one place
let store = client.create_policy_store(...).await?;
client.put_schema(&store.policy_store_id, schema).await?;
client.create_policy(&store.policy_store_id, "policy1", stmt).await?;

let response = client.is_authorized(&store.policy_store_id, ...).await?;
```

**New approach (v0.2.x):**
```rust
// 1. Setup (once, using CLI or HodeiAdmin):
// hodei init my-app
// hodei schema apply --file=schema.json
// hodei policy create --id=policy1 --statement='...'

// 2. In your application (SDK):
let client = AuthorizationClient::connect(...).await?;
let response = client.is_authorized("store-id-from-cli-setup", ...).await?;
```

### Migration Checklist

- [ ] Update `Cargo.toml` dependencies to v0.2
- [ ] Enable `compat` feature for temporary compatibility
- [ ] Run application and identify Control Plane calls
- [ ] Move Control Plane operations to CLI scripts or HodeiAdmin
- [ ] Document policy store IDs in configuration
- [ ] Remove `compat` feature flag
- [ ] Clean up application code

## 🔧 For Developers

### Project Structure

```
sdk/
├── src/
│   ├── lib.rs              # Main entry point
│   ├── client.rs           # Authorization client
│   ├── client_trait.rs     # Client trait for testing
│   ├── builders.rs         # Builder patterns
│   ├── entities/           # Entity builders
│   ├── error.rs            # Error types
│   ├── middleware/         # Optional middleware (feature gated)
│   │   ├── mod.rs
│   │   ├── extractor.rs    # Request extraction
│   │   ├── layer.rs        # Tower Layer
│   │   └── service.rs      # Tower Service
│   ├── schema/             # Schema generation (optional)
│   └── validation.rs       # OIDC validation utilities
├── examples/               # Usage examples
│   ├── basic_usage.rs
│   └── middleware.rs
├── tests/                  # Tests
└── Cargo.toml
```

### Adding Custom Logic

```rust
use hodei_permissions_sdk::AuthorizationClient;

impl AuthorizationClient {
    pub async fn my_custom_check(
        &self,
        policy_store_id: &str,
        user: &str,
        action: &str,
        resource: &str,
    ) -> Result<bool, SdkError> {
        let response = self
            .is_authorized(policy_store_id, user, action, resource)
            .await?;

        Ok(response.decision() == Decision::Allow)
    }
}
```

### Custom Error Handling

```rust
use hodei_permissions_sdk::SdkError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyAppError {
    #[error("Authorization error: {0}")]
    Auth(#[from] SdkError),

    #[error("Business logic error: {0}")]
    Business(String),
}

impl From<SdkError> for MyAppError {
    fn from(error: SdkError) -> Self {
        MyAppError::Auth(error)
    }
}
```

### Testing

```rust
use hodei_permissions_sdk::{AuthorizationClientTrait, Decision};
use async_trait::async_trait;

struct MockClient;

#[async_trait]
impl AuthorizationClientTrait for MockClient {
    async fn is_authorized(
        &self,
        _policy_store_id: &str,
        _principal: &str,
        _action: &str,
        _resource: &str,
    ) -> Result<hodei_permissions_sdk::IsAuthorizedResponse, SdkError> {
        Ok(IsAuthorizedResponse {
            decision: Decision::Allow as i32,
            ..Default::default()
        })
    }
}

#[tokio::test]
async fn test_my_logic() {
    let client = MockClient;
    // Test with mock client
}
```

### Running Tests

```bash
# Unit tests
cargo test

# Compatibility layer tests
cargo test --features compat

# Middleware tests
cargo test --features middleware

# All features
cargo test --all-features
```

### Building Documentation

```bash
# Generate docs
cargo doc --open

# With all features
cargo doc --all-features --open
```

## 🐛 Error Handling

```rust
use hodei_permissions_sdk::SdkError;

match client.is_authorized(...).await {
    Ok(response) => {
        match response.decision() {
            Decision::Allow => println!("✅ Access granted!"),
            Decision::Deny => println!("❌ Access denied!"),
            Decision::NotApplicable => println!("⚠️ No policy applies"),
            Decision::Indeterminate => println!("❓ Unable to determine"),
        }
    },
    Err(SdkError::ConnectionError(e)) => {
        eprintln!("🔌 Connection error: {}", e);
    },
    Err(SdkError::StatusError(status)) => {
        eprintln!("📡 gRPC error: {} - {}", status.code(), status.message());
    },
    Err(SdkError::InvalidRequest(msg)) => {
        eprintln!("❌ Invalid request: {}", msg);
    },
    Err(e) => {
        eprintln!("💥 Unexpected error: {}", e);
    },
}
```

## 📦 Related Packages

- **CLI Tool**: `hodei` command-line tool for Control Plane operations
  - Installation: `cargo install hodei-cli`
  - Documentation: Run `hodei --help`

- **HodeiAdmin Library**: Programmatic Control Plane access
  - Crate: `hodei-cli` with `library` feature
  - Documentation: Run `cargo doc -p hodei-cli --lib`

## 📄 License

MIT

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

## 📞 Support

- Documentation: [docs.rs/hodei-permissions-sdk](https://docs.rs/hodei-permissions-sdk)
- Migration Guide: [MIGRATION_GUIDE_SDK.md](../docs/MIGRATION_GUIDE_SDK.md)
- Issues: [GitHub Issues](https://github.com/rubentxu/hodei-verified-permissions/issues)
- Discussions: [GitHub Discussions](https://github.com/rubentxu/hodei-verified-permissions/discussions)
