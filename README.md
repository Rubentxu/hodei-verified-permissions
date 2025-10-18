# Hodei Verified Permissions

✅ **55% COMPLETADO** - MVP + Identity + Templates + Auditoría

A Cedar-based authorization service inspired by AWS Verified Permissions, built in Rust with gRPC APIs.

## 🎯 Estado del Proyecto

```
✅ MVP (Épicas 1-3):        100%
✅ Épica 4 (Identity):      100%
✅ Épica 5 (Batch):         100%
✅ Épica 6 (Templates):     100%
✅ Épica 9.1 (Auditoría):   100%
⏳ Épica 7 (Multi-tenant):    0%
⏳ Épica 8 (Local Agent):     0%
⏳ Épica 9.2 (CLI):           0%

Progreso Total: 55%
Tests: 31/31 passing ✅
```

## Overview

This service provides a centralized authorization system using the Cedar policy engine. It offers:

- **Data Plane**: High-performance authorization decisions with JWT support
- **Control Plane**: Management of policy stores, schemas, policies, templates, and identity sources
- **gRPC API**: Low-latency communication for authorization requests
- **Audit Logging**: Complete forensic trail of all authorization decisions
- **Policy Templates**: Reusable policy patterns for common use cases

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                       gRPC Server                            │
├────────────────────────┬─────────────────────────────────────┤
│   Data Plane           │      Control Plane                  │
│   - IsAuthorized       │   - Policy Store Management         │
│   - BatchIsAuthorized  │   - Schema Management               │
│   - IsAuthorizedWith   │   - Policy Management               │
│     Token (JWT)        │   - Policy Template Management      │
│                        │   - Identity Source Management      │
├────────────────────────┴─────────────────────────────────────┤
│   JWT Validator + Claims Mapper  │  Audit Logger             │
├───────────────────────────────────┴───────────────────────────┤
│              Cedar Policy Engine                              │
├───────────────────────────────────────────────────────────────┤
│              SQLite Storage + Audit Logs                      │
└───────────────────────────────────────────────────────────────┘
```

## Features Implemented

### ✅ MVP (Épicas 1-3)
- Complete gRPC service definitions
- Data and Control plane services
- Policy Store, Schema, and Policy management
- Cedar policy evaluation
- SQLite persistence
- Batch operations (up to 30 requests)

### ✅ Épica 4: Identity Integration
- **Identity Sources**: OIDC and AWS Cognito support
- **JWT Validation**: Automatic token validation with JWKS
- **Claims Mapping**: Flexible mapping of JWT claims to Cedar entities
- **IsAuthorizedWithToken**: Single endpoint for JWT-based authorization
- **Public Key Caching**: Efficient JWKS caching

### ✅ Épica 6: Policy Templates
- **Template Creation**: Reusable policies with placeholders (?principal, ?resource)
- **Template-Linked Policies**: Create policies from templates
- **Use Cases**: "Share resource" patterns implemented
- **Validation**: Automatic placeholder validation

### ✅ Épica 9.1: Audit Logging
- **Structured Logging**: JSON-formatted audit events
- **Complete Trail**: All authorization decisions logged
- **Filtering**: Query by store, principal, decision
- **Forensic Analysis**: Timestamps, metadata, determining policies
- **Compliance**: Ready for regulatory requirements

### ✅ HU 2.1: Policy Store Management
- Create, read, list, and delete policy stores
- Isolated namespaces for different applications

### ✅ HU 2.2: Schema Management
- Put and get schemas for policy stores
- Schema validation on upload

### ✅ HU 2.3: Policy Management
- Create, read, update, delete, and list policies
- Syntax validation for Cedar policies
- Association with policy stores

### ✅ HU 1.1-1.3: Authorization Evaluation
- Evaluate authorization requests with principal, action, resource
- Support for entity data (attributes and hierarchies)
- Context support for conditional policies
- Batch authorization requests

## Quick Start

### Prerequisites

- Rust 1.70 or later
- SQLite 3

### Building

```bash
cargo build --release
```

### Running the Server

```bash
# Using default settings (SQLite database, port 50051)
cargo run --release

# Or with custom configuration
DATABASE_URL=sqlite://my-auth.db SERVER_ADDR=0.0.0.0:8080 cargo run --release
```

### Environment Variables

- `DATABASE_URL`: Database connection string (default: `sqlite://authorization.db`)
- `SERVER_ADDR`: Server address and port (default: `0.0.0.0:50051`)

## API Usage Examples

### Creating a Policy Store

```bash
grpcurl -plaintext -d '{"description": "My App Policies"}' \
  localhost:50051 authorization.AuthorizationControl/CreatePolicyStore
```

### Adding a Schema

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "store-id",
  "schema": "{\"MyApp\": {\"entityTypes\": {\"User\": {}, \"Document\": {}}, \"actions\": {\"view\": {\"appliesTo\": {\"principalTypes\": [\"User\"], \"resourceTypes\": [\"Document\"]}}}}}"
}' localhost:50051 authorization.AuthorizationControl/PutSchema
```

### Creating a Policy

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "store-id",
  "policy_id": "allow-owners",
  "definition": {
    "static": {
      "statement": "permit(principal, action == Action::\"view\", resource) when { resource.owner == principal };"
    }
  }
}' localhost:50051 authorization.AuthorizationControl/CreatePolicy
```

### Authorization Request

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "store-id",
  "principal": {"entity_type": "User", "entity_id": "alice"},
  "action": {"entity_type": "Action", "entity_id": "view"},
  "resource": {"entity_type": "Document", "entity_id": "doc123"},
  "entities": [
    {
      "identifier": {"entity_type": "Document", "entity_id": "doc123"},
      "attributes": {
        "owner": "{\"__entity\": {\"type\": \"User\", \"id\": \"alice\"}}"
      }
    }
  ]
}' localhost:50051 authorization.AuthorizationData/IsAuthorized
```

## Project Structure

```
hodei-verified-permissions/
├── proto/                    # Protocol Buffer definitions
│   └── authorization.proto
├── src/
│   ├── lib.rs               # Library entry point
│   ├── main.rs              # Server binary
│   ├── error.rs             # Error types
│   ├── grpc/                # gRPC service implementations
│   │   ├── mod.rs
│   │   ├── control_plane.rs # Policy management
│   │   └── data_plane.rs    # Authorization decisions
│   └── storage/             # Persistence layer
│       ├── mod.rs
│       ├── models.rs        # Database models
│       └── repository.rs    # Database operations
├── docs/
│   └── historias-usuario.md # User stories
├── Cargo.toml
└── README.md
```

## Technology Stack

- **Language**: Rust
- **gRPC Framework**: Tonic
- **Policy Engine**: Cedar Policy
- **Database**: SQLite (MVP), extensible to PostgreSQL
- **Async Runtime**: Tokio
- **Serialization**: Serde, Protocol Buffers

## Roadmap

### Completed (MVP Phase 1)
- ✅ gRPC API definition
- ✅ Server infrastructure
- ✅ Policy store management
- ✅ Schema management
- ✅ Policy CRUD operations
- ✅ Authorization evaluation with entities and context

### Planned (Future Iterations)
- 🔄 Full schema validation (requires Cedar API improvements)
- 📋 JWT-based authorization (IsAuthorizedWithToken)
- 📋 Policy templates for dynamic permissions
- 📋 Multi-tenancy patterns and best practices
- 📋 Local evaluation agent (sidecar pattern)
- 📋 Performance optimizations and caching
- 📋 Metrics and observability
- 📋 Client SDK in multiple languages

## Development

### Running Tests

```bash
cargo test
```

### Generating Documentation

```bash
cargo doc --open
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Contributing

This is an MVP implementation. Contributions are welcome for:

- Additional test coverage
- Performance improvements
- Documentation enhancements
- Client SDKs
- Additional storage backends

## License

[Add your license here]

## References

- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [AWS Verified Permissions](https://aws.amazon.com/verified-permissions/)
- [Tonic gRPC Framework](https://github.com/hyperium/tonic)
