# Axum SimpleRest Example

This example demonstrates how to use the Hodei Verified Permissions SDK with Axum to implement automatic Cedar authorization using the SimpleRest mapping pattern.

## Features

- **Automatic Action Resolution**: HTTP method + path automatically mapped to Cedar actions
- **Context Extraction**: Path parameters and query strings extracted as Cedar context
- **Skipped Endpoints**: Health checks and public endpoints bypass authorization
- **Type-Safe Handlers**: Axum extractors with proper error handling

## Architecture

```
HTTP Request (GET /documents/123?limit=10)
    ↓
VerifiedPermissionsLayer (middleware)
    ↓
SimpleRestMapping.resolve()
    ↓
Action: "getDocument"
Resource: "Document"  
Context: {"pathParameters": {"id": "123"}, "queryStringParameters": {"limit": "10"}}
    ↓
AuthorizationClient.is_authorized_with_token()
    ↓
Cedar Policy Evaluation
    ↓
Allow → Forward to handler
Deny → Return 403
```

## Prerequisites

1. **Authorization Service**: Running Hodei authorization service
   ```bash
   # Start the authorization service (from project root)
   cargo run --bin hodei-verified-permissions
   ```

2. **Cedar Schema**: Generated from OpenAPI spec
   ```bash
   # Already included in this example as v4.cedarschema.json
   # To regenerate:
   hodei-cli generate-schema \
     --api-spec ../../openapi-sample.json \
     --namespace DocumentApp \
     --base-path /v1
   ```

3. **Cedar Policies**: Sample policies for testing
   ```bash
   # Generate sample policies
   hodei-cli generate-policies \
     --schema v4.cedarschema.json \
     --output ./policies
   ```

## Running the Example

### 1. Set Environment Variables (Optional)

```bash
export AUTH_ENDPOINT=http://localhost:50051
export RUST_LOG=info
```

### 2. Run the Server

```bash
cargo run --example axum-simple-rest
```

Or from the example directory:

```bash
cd examples/axum-simple-rest
cargo run
```

### 3. Test the Endpoints

#### Health Check (No Auth Required)

```bash
curl http://localhost:3000/health
# Expected: OK
```

#### List Documents (Auth Required)

```bash
curl -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/documents
```

**What happens:**
- Middleware resolves: action=`"listDocuments"`, resource=`"Application"`
- Context: `{"queryStringParameters": {}}`
- Cedar evaluates policy for `User::<from-token>` → `listDocuments` → `Application`

#### Get Document (Auth Required)

```bash
curl -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/documents/123
```

**What happens:**
- Middleware resolves: action=`"getDocument"`, resource=`"Document"`
- Context: `{"pathParameters": {"id": "123"}}`
- Cedar evaluates policy for `User::<from-token>` → `getDocument` → `Document`

#### Create Document (Auth Required)

```bash
curl -X POST \
  -H "Authorization: Bearer <your-jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{"title": "New Doc", "content": "Content"}' \
  http://localhost:3000/documents
```

#### Update Document (Auth Required)

```bash
curl -X PUT \
  -H "Authorization: Bearer <your-jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{"title": "Updated", "content": "New content"}' \
  http://localhost:3000/documents/123
```

#### Delete Document (Auth Required)

```bash
curl -X DELETE \
  -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/documents/123
```

#### Share Document (Auth Required)

```bash
curl -X POST \
  -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/documents/123/share?userId=alice
```

**What happens:**
- Middleware resolves: action=`"shareDocument"`, resource=`"Document"`
- Context: `{"pathParameters": {"id": "123"}, "queryStringParameters": {"userId": "alice"}}`

## Code Walkthrough

### 1. Load Cedar Schema

```rust
let schema_json = include_str!("../v4.cedarschema.json");
```

The schema is generated from OpenAPI and contains SimpleRest annotations.

### 2. Configure Authorization Layer

```rust
let auth_layer = VerifiedPermissionsLayer::new(
    client,
    "policy-store-id",
    "identity-source-id",
)
.with_simple_rest_mapping(schema_json)?
.skip_endpoint("get", "/health")
.skip_prefix("get", "/public/");
```

- `with_simple_rest_mapping()`: Loads the schema and builds route matchers
- `skip_endpoint()`: Specific endpoints to bypass authorization
- `skip_prefix()`: All paths with a prefix bypass authorization

### 3. Apply to Router

```rust
let app = Router::new()
    .route("/documents", get(list_documents).post(create_document))
    .route("/documents/:id", get(get_document))
    .layer(auth_layer);
```

The layer intercepts all requests before they reach handlers.

## Schema Mapping

The `v4.cedarschema.json` defines how HTTP operations map to Cedar:

```json
{
  "DocumentApp": {
    "actions": {
      "getDocument": {
        "annotations": {
          "httpVerb": "get",
          "httpPathTemplate": "/v1/documents/{id}"
        },
        "appliesTo": {
          "principalTypes": ["User"],
          "resourceTypes": ["Document"],
          "context": {
            "type": "Record",
            "attributes": {
              "pathParameters": {
                "type": "Record",
                "attributes": {
                  "id": {"type": "String", "required": true}
                }
              }
            }
          }
        }
      }
    }
  }
}
```

## Sample Policies

Create policies in `policies/` directory:

### Admin Policy (`policy_1.cedar`)

```cedar
// Allow admin group full access
permit(
    principal in DocumentApp::UserGroup::"admin",
    action,
    resource
);
```

### Read-Only Policy (`policy_2.cedar`)

```cedar
// Allow readers to view documents
permit(
    principal in DocumentApp::UserGroup::"readers",
    action in [
        DocumentApp::Action::"listDocuments",
        DocumentApp::Action::"getDocument"
    ],
    resource
);
```

### Owner Policy (`policy_3.cedar`)

```cedar
// Allow users to manage their own documents
permit(
    principal,
    action in [
        DocumentApp::Action::"getDocument",
        DocumentApp::Action::"updateDocument",
        DocumentApp::Action::"deleteDocument"
    ],
    resource
)
when {
    resource.owner == principal
};
```

## Troubleshooting

### "Connection refused" Error

Ensure the authorization service is running:
```bash
cargo run --bin hodei-verified-permissions
```

### "Access Denied" Error

Check:
1. JWT token is valid and not expired
2. User exists in the policy store
3. Policies allow the action for the user
4. Resource type matches the policy

### "No mapping found" Error

The HTTP method + path doesn't match any action in the schema. Check:
1. Schema is loaded correctly
2. Path template matches (e.g., `/documents/:id` vs `/documents/{id}`)
3. HTTP method is supported

## Next Steps

- Add more handlers for your domain
- Customize policies for your use case
- Add resource-specific authorization (check ownership, etc.)
- Integrate with your JWT provider
- Add request/response logging
- Implement proper error responses

## Related Documentation

- [Hodei SDK Documentation](../../sdk/README.md)
- [CLI Usage Guide](../../docs/GUIA_USO_CLI_GENERACION_SCHEMA.md)
- [Sprint 1 Implementation](../../docs/SPRINT1_IMPLEMENTACION_COMPLETADA.md)
- [Cedar Policy Language](https://www.cedarpolicy.com/)
