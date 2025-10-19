# Middleware Guide

Complete guide for using Hodei Permissions middleware with Axum and Tower.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Basic Setup](#basic-setup)
- [Advanced Configuration](#advanced-configuration)
- [Custom Extractors](#custom-extractors)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Overview

The Hodei Permissions middleware provides automatic authorization checks for HTTP services built with Axum and Tower. It:

1. Extracts JWT tokens from requests
2. Maps HTTP methods to actions
3. Uses URI paths as resources
4. Calls `IsAuthorizedWithToken` automatically
5. Returns 403 Forbidden on denial
6. Forwards requests to handlers on approval

## Installation

Add the middleware feature to your `Cargo.toml`:

```toml
[dependencies]
hodei-permissions-sdk = { version = "0.1", features = ["middleware"] }
axum = "0.7"
tower = "0.5"
tokio = { version = "1.40", features = ["full"] }
```

## Basic Setup

### 1. Create the Middleware Layer

```rust
use hodei_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    // Connect to authorization service
    let client = AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Failed to connect");

    // Create middleware
    let auth_layer = VerifiedPermissionsLayer::new(
        client,
        "your-policy-store-id",
        "your-identity-source-id"
    );

    // Apply to routes
    let app = Router::new()
        .route("/api/documents", get(list_documents))
        .route("/api/documents/:id", get(get_document))
        .layer(auth_layer);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### 2. Implement Your Handlers

```rust
use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Document {
    id: String,
    title: String,
    content: String,
}

async fn list_documents() -> Json<Vec<Document>> {
    // Middleware has already checked authorization
    // This code only runs if access is granted
    Json(vec![
        Document {
            id: "doc1".to_string(),
            title: "Document 1".to_string(),
            content: "Content 1".to_string(),
        },
    ])
}

async fn get_document(Path(id): Path<String>) -> Json<Document> {
    // Authorization already checked by middleware
    Json(Document {
        id,
        title: "Document".to_string(),
        content: "Content".to_string(),
    })
}
```

### 3. Send Requests with JWT

```bash
curl -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIs..." \
     http://localhost:3000/api/documents
```

## Advanced Configuration

### Selective Route Protection

Apply middleware only to specific routes:

```rust
let app = Router::new()
    // Public routes (no auth)
    .route("/health", get(health_check))
    .route("/public/info", get(public_info))
    
    // Protected routes (with auth)
    .route("/api/documents", get(list_documents))
    .route("/api/documents/:id", get(get_document).delete(delete_document))
    .layer(auth_layer);
```

### Multiple Policy Stores

Use different policy stores for different route groups:

```rust
// Admin routes
let admin_layer = VerifiedPermissionsLayer::new(
    client.clone(),
    "admin-policy-store",
    "admin-identity-source"
);

let admin_routes = Router::new()
    .route("/admin/users", get(list_users))
    .layer(admin_layer);

// User routes
let user_layer = VerifiedPermissionsLayer::new(
    client.clone(),
    "user-policy-store",
    "user-identity-source"
);

let user_routes = Router::new()
    .route("/api/documents", get(list_documents))
    .layer(user_layer);

// Combine
let app = Router::new()
    .nest("/", user_routes)
    .nest("/", admin_routes);
```

### Shared Client

Share a single authorization client across multiple middleware instances:

```rust
use std::sync::Arc;

let client = Arc::new(
    AuthorizationClient::connect("http://localhost:50051").await?
);

let layer1 = VerifiedPermissionsLayer::from_arc(
    client.clone(),
    "store-1",
    "identity-1"
);

let layer2 = VerifiedPermissionsLayer::from_arc(
    client.clone(),
    "store-2",
    "identity-2"
);
```

## Custom Extractors

### Implementing a Custom Extractor

Create your own logic for extracting authorization parameters:

```rust
use hodei_permissions_sdk::middleware::{
    AuthorizationRequestExtractor, AuthorizationRequestParts, MiddlewareError
};
use async_trait::async_trait;
use http::Request;

struct ApiKeyExtractor {
    policy_store_id: String,
    identity_source_id: String,
}

#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for ApiKeyExtractor
where
    B: Send + Sync,
{
    type Error = MiddlewareError;

    async fn extract(&self, req: &Request<B>) 
        -> Result<AuthorizationRequestParts, Self::Error> 
    {
        // Extract API key from header
        let api_key = req.headers()
            .get("x-api-key")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| MiddlewareError::AuthorizationHeader(
                "Missing x-api-key header".to_string()
            ))?;

        // Map to principal
        let principal = format!("ApiKey::{}", api_key);

        // Extract action from method
        let action = match req.method().as_str() {
            "GET" => "Action::\"read\"",
            "POST" => "Action::\"create\"",
            "PUT" | "PATCH" => "Action::\"update\"",
            "DELETE" => "Action::\"delete\"",
            _ => return Err(MiddlewareError::ExtractionFailed(
                "Unsupported method".to_string()
            )),
        };

        // Extract resource from path
        let path = req.uri().path();
        let resource = format!("Resource::\"{}\"", path.trim_start_matches('/'));

        Ok(AuthorizationRequestParts {
            principal,
            action: action.to_string(),
            resource,
            context: None,
        })
    }
}
```

### Using Custom Extractor

```rust
// Note: Custom extractors require building the service manually
// This is an advanced use case

use tower::ServiceBuilder;

let extractor = ApiKeyExtractor {
    policy_store_id: "store-123".to_string(),
    identity_source_id: "identity-456".to_string(),
};

// Build service with custom extractor
// (Implementation depends on your specific needs)
```

## Error Handling

### Default Error Responses

The middleware returns these HTTP status codes:

| Error | Status Code | Description |
|-------|-------------|-------------|
| Missing Authorization header | 401 Unauthorized | No Bearer token provided |
| Invalid token format | 401 Unauthorized | Token is malformed |
| Token validation failed | 401 Unauthorized | Token signature invalid |
| Access denied | 403 Forbidden | Policy evaluation returned Deny |
| Authorization service error | 500 Internal Server Error | Failed to reach authorization service |

### Custom Error Handler

Handle middleware errors in your application:

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// Axum will catch errors from middleware
// You can implement custom error responses

async fn handle_error(err: BoxError) -> Response {
    if err.is::<MiddlewareError>() {
        let middleware_err = err.downcast_ref::<MiddlewareError>().unwrap();
        
        let (status, message) = match middleware_err {
            MiddlewareError::AuthorizationHeader(_) => {
                (StatusCode::UNAUTHORIZED, "Invalid or missing authorization")
            },
            MiddlewareError::AccessDenied(_) => {
                (StatusCode::FORBIDDEN, "Access denied")
            },
            _ => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Authorization error")
            },
        };

        (status, Json(json!({
            "error": message,
            "details": middleware_err.to_string()
        }))).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
    }
}
```

## Best Practices

### 1. Use Shared Clients

Create one authorization client and share it:

```rust
let client = Arc::new(AuthorizationClient::connect("...").await?);
```

### 2. Configure JWKS Caching

The server automatically caches JWKS keys. Configure TTL in server settings:

```yaml
# server config
jwks_cache_ttl: 3600  # 1 hour
```

### 3. Policy Design

Design policies that work well with HTTP patterns:

```cedar
// Good: Uses HTTP-mapped actions
permit(
    principal in Group::"developers",
    action in [Action::"read", Action::"create"],
    resource like Resource::"api/documents/*"
);

// Better: More specific
permit(
    principal in Group::"developers",
    action == Action::"read",
    resource like Resource::"api/documents/*"
) when {
    context.method == "GET"
};
```

### 4. Testing

Test your protected endpoints:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_protected_endpoint() {
        let app = create_app().await;

        // Request without token
        let response = app
            .oneshot(Request::builder()
                .uri("/api/documents")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Request with valid token
        let response = app
            .oneshot(Request::builder()
                .uri("/api/documents")
                .header("Authorization", "Bearer valid_token")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

### 5. Logging

Add logging to track authorization decisions:

```rust
use tracing::{info, warn};

// In your handler
async fn list_documents() -> Json<Vec<Document>> {
    info!("Listing documents - authorization passed");
    // ...
}
```

## Examples

### Complete Example: Multi-Tenant API

```rust
use hodei_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    auth_client: Arc<AuthorizationClient>,
}

#[tokio::main]
async fn main() {
    // Setup
    let auth_client = Arc::new(
        AuthorizationClient::connect("http://localhost:50051")
            .await
            .unwrap()
    );

    let state = AppState { auth_client: auth_client.clone() };

    // Middleware
    let auth_layer = VerifiedPermissionsLayer::from_arc(
        auth_client,
        "multi-tenant-store",
        "oidc-identity-source"
    );

    // Routes
    let app = Router::new()
        // Public
        .route("/health", get(health_check))
        
        // Protected
        .route("/api/tenants/:tenant_id/documents", 
            get(list_documents).post(create_document))
        .route("/api/tenants/:tenant_id/documents/:doc_id",
            get(get_document).delete(delete_document))
        .layer(auth_layer)
        .with_state(state);

    // Serve
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_documents(
    Path(tenant_id): Path<String>,
) -> Json<Vec<String>> {
    // Authorization already checked
    Json(vec![format!("doc1 in {}", tenant_id)])
}

async fn create_document(
    Path(tenant_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<String> {
    Json(format!("Created in {}", tenant_id))
}

async fn get_document(
    Path((tenant_id, doc_id)): Path<(String, String)>,
) -> Json<String> {
    Json(format!("Document {} in {}", doc_id, tenant_id))
}

async fn delete_document(
    Path((tenant_id, doc_id)): Path<(String, String)>,
) -> StatusCode {
    StatusCode::NO_CONTENT
}
```

### Example: Rate Limiting + Authorization

Combine with other middleware:

```rust
use tower::ServiceBuilder;
use tower_http::limit::RateLimitLayer;
use std::time::Duration;

let app = Router::new()
    .route("/api/documents", get(list_documents))
    .layer(
        ServiceBuilder::new()
            // Rate limit first
            .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
            // Then authorize
            .layer(auth_layer)
    );
```

## Troubleshooting

### Common Issues

**1. "Missing Authorization header"**
- Ensure requests include `Authorization: Bearer <token>` header
- Check that token is not empty

**2. "Token validation failed"**
- Verify JWKS URI is accessible
- Check token hasn't expired
- Ensure issuer matches identity source configuration

**3. "Access denied"**
- Review Cedar policies
- Check that JWT claims map correctly to entities
- Verify action and resource match policy patterns

**4. Middleware not applying**
- Ensure `.layer()` is called after routes
- Check feature flag is enabled: `features = ["middleware"]`

### Debug Logging

Enable debug logs:

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

## Further Reading

- [SDK README](../README.md)
- [Identity Sources Guide](./IDENTITY_SOURCES.md)
- [Cedar Policy Language](https://docs.cedarpolicy.com/)
- [Axum Documentation](https://docs.rs/axum)
- [Tower Documentation](https://docs.rs/tower)
