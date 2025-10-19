# Hodei Permissions SDK

Ergonomic Rust client SDK for Hodei Verified Permissions authorization service.

## 📚 Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Guide](#usage-guide)
  - [Basic Authorization](#basic-authorization)
  - [JWT Token Authorization](#jwt-token-authorization)
  - [Middleware (Axum/Tower)](#middleware-axumtower)
  - [Identity Sources](#identity-sources)
- [API Reference](#api-reference)
- [Examples](#examples)
- [For Developers](#for-developers)
- [Testing](#testing)

## ✨ Features

- 🚀 **Simple API**: Easy-to-use methods for all operations
- 🔧 **Builder Patterns**: Fluent API for complex requests
- ⚡ **Async/Await**: Built on Tokio for high performance
- 🛡️ **Type Safe**: Leverages Rust's type system
- 📝 **Well Documented**: Comprehensive examples and docs
- 🔐 **JWT Support**: Built-in JWT token validation with Identity Sources
- 🌐 **IdP Integration**: Keycloak, Zitadel, AWS Cognito support
- 🔌 **Middleware**: Optional Axum/Tower middleware (feature flag)
- 🎯 **Cedar Policies**: Full support for Cedar policy language

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hodei-permissions-sdk = "0.1"
tokio = { version = "1.40", features = ["full"] }
```

### With Middleware Support

```toml
[dependencies]
hodei-permissions-sdk = { version = "0.1", features = ["middleware"] }
axum = "0.7"
tower = "0.5"
```

## 🚀 Quick Start

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
        println!("✅ Access granted!");
    } else {
        println!("❌ Access denied!");
    }

    Ok(())
}
```

## 📖 Usage Guide

### Basic Authorization

#### 1. Create a Policy Store

```rust
let store = client
    .create_policy_store(Some("My Application".to_string()))
    .await?;

println!("Policy Store ID: {}", store.policy_store_id);
```

#### 2. Create Cedar Policies

```rust
let policy = r#"
permit(
    principal == User::"alice",
    action == Action::"view",
    resource == Document::"doc123"
);
"#;

client.create_policy(
    &store.policy_store_id,
    "allow-alice-view-doc123",
    policy,
    Some("Allow Alice to view document 123".to_string())
).await?;
```

#### 3. Check Authorization

```rust
let response = client
    .is_authorized(
        &store.policy_store_id,
        "User::alice",
        "Action::view",
        "Document::doc123"
    )
    .await?;

println!("Decision: {:?}", response.decision());
```

### JWT Token Authorization

#### 1. Create an Identity Source

```rust
use hodei_permissions_sdk::proto::{
    IdentitySourceConfiguration, OidcConfiguration,
    identity_source_configuration, ClaimsMappingConfiguration
};

// Configure OIDC (works with Keycloak, Zitadel, Cognito, etc.)
let oidc_config = OidcConfiguration {
    issuer: "https://your-idp.com".to_string(),
    client_ids: vec!["your-client-id".to_string()],
    jwks_uri: "https://your-idp.com/.well-known/jwks.json".to_string(),
    group_claim: "groups".to_string(), // or "realm_access.roles" for Keycloak
};

let config = IdentitySourceConfiguration {
    configuration_type: Some(
        identity_source_configuration::ConfigurationType::Oidc(oidc_config)
    ),
};

let claims_mapping = ClaimsMappingConfiguration {
    principal_id_claim: "sub".to_string(),
    group_claim: String::new(),
    attribute_mappings: HashMap::new(),
};

let identity_source = client
    .create_identity_source(
        &store.policy_store_id,
        config,
        Some(claims_mapping),
        Some("My IdP".to_string())
    )
    .await?;
```

#### 2. Authorize with JWT Token

```rust
// Get JWT token from your authentication flow
let jwt_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";

let response = client
    .is_authorized_with_token(
        &store.policy_store_id,
        &identity_source.identity_source_id,
        jwt_token,
        "Action::\"read\"",
        "Document::\"doc123\""
    )
    .await?;

if response.decision() == Decision::Allow {
    println!("✅ Token validated and access granted!");
}
```

### Middleware (Axum/Tower)

Protect your HTTP routes with automatic authorization checks.

#### 1. Setup Middleware

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
        "policy-store-123",      // Your policy store ID
        "identity-source-456"     // Your identity source ID
    );

    // Build your app with the middleware
    let app = Router::new()
        .route("/api/documents", get(list_documents))
        .route("/api/documents/:id", get(get_document))
        .layer(auth_layer);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}

// Your handlers - middleware handles authorization automatically
async fn list_documents() -> Json<Vec<String>> {
    Json(vec!["doc1".to_string(), "doc2".to_string()])
}

async fn get_document() -> Json<String> {
    Json("Document content".to_string())
}
```

#### 2. How It Works

The middleware automatically:
1. Extracts JWT token from `Authorization: Bearer <token>` header
2. Maps HTTP method to action (GET → read, POST → create, etc.)
3. Uses URI path as resource
4. Calls `IsAuthorizedWithToken`
5. Returns **403 Forbidden** on Deny
6. Forwards request to handler on Allow

#### 3. Custom Extractor (Advanced)

Implement your own extraction logic:

```rust
use hodei_permissions_sdk::middleware::{
    AuthorizationRequestExtractor, AuthorizationRequestParts
};
use async_trait::async_trait;
use http::Request;

struct MyCustomExtractor;

#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for MyCustomExtractor
where
    B: Send + Sync,
{
    type Error = hodei_permissions_sdk::middleware::MiddlewareError;

    async fn extract(&self, req: &Request<B>) 
        -> Result<AuthorizationRequestParts, Self::Error> 
    {
        // Custom logic to extract principal, action, resource
        let principal = req.headers()
            .get("x-user-id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("anonymous");

        let action = match req.method().as_str() {
            "GET" => "read",
            "POST" => "create",
            _ => "unknown",
        };

        let resource = req.uri().path();

        Ok(AuthorizationRequestParts {
            principal: principal.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            context: None,
        })
    }
}
```

### Identity Sources

#### Keycloak

```rust
let oidc_config = OidcConfiguration {
    issuer: "http://localhost:8080/realms/myrealm".to_string(),
    client_ids: vec!["my-app".to_string()],
    jwks_uri: "http://localhost:8080/realms/myrealm/protocol/openid-connect/certs".to_string(),
    group_claim: "realm_access.roles".to_string(), // Realm roles
    // or "resource_access.my-app.roles" for client roles
};
```

#### Zitadel

```rust
let project_id = "123456789";
let oidc_config = OidcConfiguration {
    issuer: "https://myinstance.zitadel.cloud".to_string(),
    client_ids: vec!["my-app@project".to_string()],
    jwks_uri: "https://myinstance.zitadel.cloud/oauth/v2/keys".to_string(),
    group_claim: format!("urn:zitadel:iam:org:project:{}:roles", project_id),
};
```

#### AWS Cognito

```rust
let oidc_config = OidcConfiguration {
    issuer: "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123".to_string(),
    client_ids: vec!["your-app-client-id".to_string()],
    jwks_uri: "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123/.well-known/jwks.json".to_string(),
    group_claim: "cognito:groups".to_string(),
};
```

## 📚 API Reference

### Data Plane (Authorization)

| Method | Description |
|--------|-------------|
| `is_authorized()` | Simple authorization check |
| `is_authorized_with_context()` | Authorization with entities and context |
| `is_authorized_with_token()` | Authorization with JWT token |
| `batch_is_authorized()` | Multiple authorization checks |

### Control Plane (Management)

**Policy Stores:**
- `create_policy_store()` - Create a new policy store
- `get_policy_store()` - Get policy store details
- `list_policy_stores()` - List all policy stores
- `delete_policy_store()` - Delete a policy store

**Schemas:**
- `put_schema()` - Upload or update schema
- `get_schema()` - Get current schema

**Policies:**
- `create_policy()` - Create a new policy
- `get_policy()` - Get policy details
- `list_policies()` - List all policies
- `delete_policy()` - Delete a policy

**Identity Sources:**
- `create_identity_source()` - Create identity source (OIDC/Cognito)
- `get_identity_source()` - Get identity source details
- `list_identity_sources()` - List all identity sources
- `delete_identity_source()` - Delete identity source

### Builder Patterns

#### IsAuthorizedRequestBuilder

```rust
use hodei_permissions_sdk::IsAuthorizedRequestBuilder;

let request = IsAuthorizedRequestBuilder::new(&policy_store_id)
    .principal("User", "alice")
    .action("Action", "view")
    .resource("Document", "doc123")
    .context(r#"{"ip": "192.168.1.1"}"#)
    .add_entity(entity1)
    .add_entity(entity2)
    .build();
```

#### EntityBuilder

```rust
use hodei_permissions_sdk::EntityBuilder;

let entity = EntityBuilder::new("User", "alice")
    .attribute("department", r#""engineering""#)
    .attribute("level", "5")
    .parent("Group", "admins")
    .build();
```

## 💡 Examples

### Complete Example: Document Management

```rust
use hodei_permissions_sdk::{AuthorizationClient, EntityBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // 1. Create policy store
    let store = client.create_policy_store(Some("DocApp".to_string())).await?;

    // 2. Create policy
    let policy = r#"
        permit(
            principal,
            action == Action::"view",
            resource
        ) when {
            resource.owner == principal ||
            principal in resource.viewers
        };
    "#;

    client.create_policy(
        &store.policy_store_id,
        "allow-owner-and-viewers",
        policy,
        Some("Allow owners and viewers to view documents".to_string())
    ).await?;

    // 3. Create entities
    let alice = EntityBuilder::new("User", "alice").build();
    
    let doc = EntityBuilder::new("Document", "doc123")
        .attribute("owner", r#"{"__entity": {"type": "User", "id": "alice"}}"#)
        .build();

    // 4. Check authorization
    let request = IsAuthorizedRequestBuilder::new(&store.policy_store_id)
        .principal("User", "alice")
        .action("Action", "view")
        .resource("Document", "doc123")
        .add_entity(alice)
        .add_entity(doc)
        .build();

    let response = client.is_authorized_with_context(request).await?;

    println!("Decision: {:?}", response.decision());

    Ok(())
}
```

## 🔧 For Developers

### Extending the SDK

#### Custom Error Types

```rust
use hodei_permissions_sdk::SdkError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyAppError {
    #[error("SDK error: {0}")]
    Sdk(#[from] SdkError),
    
    #[error("Custom error: {0}")]
    Custom(String),
}
```

#### Custom Middleware Extractor

See [Custom Extractor](#3-custom-extractor-advanced) section above.

#### Adding New Methods

The SDK is built on top of the generated gRPC client. You can extend it:

```rust
use hodei_permissions_sdk::AuthorizationClient;

impl AuthorizationClient {
    pub async fn my_custom_method(&self) -> Result<(), SdkError> {
        // Your custom logic
        Ok(())
    }
}
```

### Project Structure

```
sdk/
├── src/
│   ├── lib.rs              # Main entry point
│   ├── client.rs           # Authorization client
│   ├── builders.rs         # Builder patterns
│   ├── error.rs            # Error types
│   └── middleware/         # Optional middleware (feature gated)
│       ├── mod.rs
│       ├── extractor.rs    # Request extraction trait
│       ├── layer.rs        # Tower Layer
│       ├── service.rs      # Tower Service
│       └── error.rs        # Middleware errors
├── examples/               # Usage examples
├── tests/                  # Integration tests
└── Cargo.toml
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires running server)
cargo test --features integration-tests

# Middleware tests
cargo test --features middleware
```

### Building Documentation

```bash
# Generate and open docs
cargo doc --open --features middleware

# Generate docs for all features
cargo doc --all-features
```

## 🐛 Error Handling

```rust
use hodei_permissions_sdk::SdkError;

match client.is_authorized(...).await {
    Ok(response) => {
        println!("Decision: {:?}", response.decision());
    },
    Err(SdkError::ConnectionError(e)) => {
        eprintln!("Failed to connect to server: {}", e);
    },
    Err(SdkError::StatusError(status)) => {
        eprintln!("gRPC error: {} - {}", status.code(), status.message());
    },
    Err(SdkError::InvalidRequest(msg)) => {
        eprintln!("Invalid request: {}", msg);
    },
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    },
}
```

## 📄 License

MIT

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

## 📞 Support

- Documentation: [docs.rs/hodei-permissions-sdk](https://docs.rs/hodei-permissions-sdk)
- Issues: [GitHub Issues](https://github.com/your-org/hodei-verified-permissions/issues)
- Discussions: [GitHub Discussions](https://github.com/your-org/hodei-verified-permissions/discussions)
