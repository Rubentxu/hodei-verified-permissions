# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - Sprint 1 (2025-10-21)

#### Schema Generation Module
- **OpenAPI → Cedar Schema Generator**: New module `sdk/src/schema/` implementing SimpleRest mapping pattern
  - `SchemaGenerationUseCase` trait (hexagonal architecture port)
  - `SimpleRestSchemaGenerator` service with full OpenAPI 3.x support
  - OpenAPI parser adapter using `openapiv3` crate
  - Support for path and query parameters → Cedar context
  - Automatic entity type discovery (User, UserGroup, Application, + custom)
  - Schema validation (namespace format, base path, HTTP methods)
  - v4 Cedar schema JSON output

#### CLI Tool
- **`hodei-cli` binary**: New command-line tool for Cedar schema operations
  - `generate-schema` command: Generate Cedar v4 schema from OpenAPI spec
    - Arguments: `--api-spec`, `--namespace`, `--base-path`, `--output`
    - Validates namespace format and reserved words
    - Supports multiple servers with base path disambiguation
  - `generate-policies` command: Generate sample Cedar policies from schema
    - Admin policy (permit all)
    - Role-based policy template
  - Comprehensive logging with `tracing`
  - Error handling with `anyhow`

#### Documentation
- `docs/SPRINT1_IMPLEMENTACION_COMPLETADA.md`: Complete Sprint 1 implementation report
- `docs/GUIA_USO_CLI_GENERACION_SCHEMA.md`: Comprehensive CLI usage guide
- `docs/PLAN_EXPANSION_FUNCIONALIDAD_CEDAR_RUST.md`: Updated with Sprint 1 completion status
- `cli/README.md`: CLI-specific documentation
- `examples/openapi-sample.json`: Sample OpenAPI spec for testing

#### Features
- **SimpleRest Mapping Pattern**:
  - HTTP method + path → Cedar action
  - `operationId` → action name (preferred)
  - Path parameters → `context.pathParameters`
  - Query parameters → `context.queryStringParameters`
  - Annotations: `httpVerb`, `httpPathTemplate`, `mappingType: "SimpleRest"`
- **OpenAPI Extensions**:
  - `x-cedar.appliesToResourceTypes`: Custom resource types per operation
- **Supported HTTP Methods**: GET, POST, PUT, PATCH, DELETE
- **Entity Types**: User, UserGroup, Application, + auto-discovered custom types

### Changed
- `sdk/Cargo.toml`: Added `schema` feature with `openapiv3` and `url` dependencies
- `sdk/src/lib.rs`: Export new `schema` module (feature-gated)
- `Cargo.toml` (workspace): Added `cli` member
- `sdk/src/authorization/mod.rs`: Removed `cedar_inline` module (not required)

### Removed
- `sdk/src/authorization/cedar_inline.rs`: Eliminated inline Cedar engine (not needed for current architecture)

## [0.1.0] - Previous Work

### Existing Features
- gRPC-based authorization client SDK
- Entity management (CedarEntity, EntityIdentifier, builders)
- Authorization engine abstraction (AuthorizationEngine trait)
- Middleware for Axum (VerifiedPermissionsLayer, VerifiedPermissionsService)
- Principal extraction and configuration
- Multi-tenancy support
- JWT validation
- Policy store management
- PostgreSQL and SurrealDB adapters

---

## Sprint Roadmap

### ✅ Sprint 1 - Schema Generation & CLI (Completed)
- OpenAPI → Cedar schema generator
- CLI with `generate-schema` and `generate-policies` commands
- Hexagonal architecture implementation
- E2E validation with examples

### 🔄 Sprint 2 - Runtime Mapping & Middleware (Next)
- SimpleRest runtime mapping with route matcher
- Integration with VerifiedPermissionsLayer
- Context extraction from HTTP requests
- Support for skipped endpoints

### 📋 Sprint 3 - Axum Metaprogramming (Planned)
- `#[cedar_endpoint]` attribute macro or Utoipa integration
- Compile-time route registry
- Automatic OpenAPI generation from annotations
- Build pipeline integration

### 📋 Sprint 4 - Tests, Docs & Examples (Planned)
- Unit and integration tests
- Complete Axum example with middleware
- Usage guides and best practices
