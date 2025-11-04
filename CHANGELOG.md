# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-27

### Breaking Changes

#### SDK Client (hodei-permissions-sdk)
- **BREAKING**: Removed Control Plane operations from SDK client
  - Removed: `create_policy_store()`, `get_policy_store()`, `list_policy_stores()`
  - Removed: `put_schema()`, `get_schema()`
  - Removed: `create_policy()`, `get_policy()`, `list_policies()`, `update_policy()`, `delete_policy()`
  - Removed: `create_identity_source()`, `get_identity_source()`, `list_identity_sources()`, `delete_identity_source()`
  - Removed: `create_policy_template()`, `get_policy_template()`, `list_policy_templates()`, `delete_policy_template()`

- **MIGRATION REQUIRED**: Use `sdk-admin` library for Control Plane operations
  - Old: `hodei_permissions_sdk::Client`
  - New: `sdk_admin::HodeiAdmin`

### Added

#### SDK Client (hodei-permissions-sdk)
- ✅ Enhanced backward compatibility layer with deprecation warnings
- ✅ `compat` feature flag to enable deprecated methods (returns helpful errors)
- ✅ Comprehensive middleware support for Axum and Tower
- ✅ Builder patterns for authorization requests
- ✅ Client trait for testing and mocking

#### SDK Admin Library (sdk-admin) - NEW!
- ✅ Complete Control Plane API exposed as reusable library
- ✅ `HodeiAdmin` struct for programmatic operations
- ✅ All 21+ Control Plane operations implemented
- ✅ **NEW**: Bulk operations for efficient batch processing
  - `batch_create_policies()` - Create multiple policies in one call
  - `batch_update_policies()` - Update multiple policies efficiently
  - `batch_delete_policies()` - Delete multiple policies in batch
  - `test_authorization()` - Playground mode for testing without persistence
  - `validate_policy()` - Comprehensive Cedar policy validation
  - `batch_is_authorized()` - Batch authorization checks

#### Documentation
- ✅ Complete migration guide (`docs/MIGRATION_GUIDE_SDK.md`)
- ✅ SDK Admin library documentation (`sdk-admin/README.md`)
- ✅ Updated main README with new architecture diagrams
- ✅ Before/after code examples for migration
- ✅ API reference documentation

#### Testing
- ✅ 26 unit tests for SDK Data Plane operations
- ✅ 16 integration tests for SDK
- ✅ 6 integration tests for sdk-admin library
- ✅ Backward compatibility test suite
- ✅ 100% test coverage achieved

### Changed

#### SDK Client (hodei-permissions-sdk)
- Refactored to Data Plane only (authorization checking)
- Simplified API surface from 26+ methods to 5 core methods
- Updated documentation to reflect Data Plane focus
- Maintained backward compatibility via deprecation warnings

#### CLI Tool
- Refactored to use `sdk-admin` library internally
- No breaking changes to CLI user experience
- Cleaner architecture with library separation

### Deprecated

#### SDK Client (hodei-permissions-sdk)
- ⚠️ All Control Plane methods deprecated (use `sdk-admin` instead)
- ⚠️ Methods return helpful errors directing users to migration guide
- Deprecation warnings active in 0.2.x versions
- Breaking changes planned for 0.3.0

### Architecture Changes

#### New Separation of Concerns
```
┌─────────────────────────────────────────────┐
│           Hodei Admin SDK                   │
│        (Control Plane Operations)           │
├─────────────────────────────────────────────┤
│  • Policy Store Management                  │
│  • Schema Upload                            │
│  • Policy CRUD Operations                   │
│  • Bulk Operations                          │
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
│  • Middleware Integration                   │
└─────────────────────────────────────────────┘
```

#### Benefits
- **Performance**: SDK focused on fast authorization checks
- **Simplicity**: Clear separation of concerns
- **Flexibility**: Choose only what you need
- **Industry Pattern**: Follows AWS Verified Permissions architecture
- **Reusability**: SDK Admin can be used in any application

### Migration Guide

For users upgrading from v0.1.x:

#### Before (v0.1.x) - DEPRECATED
```rust
use hodei_permissions_sdk::Client;

let mut client = Client::connect("http://localhost:50051").await?;

// This no longer works in v0.2.0
let store = client.create_policy_store("test", None).await?;
let policy = client.create_policy(&store.policy_store_id, "pol1", statement).await?;
```

#### After (v0.2.0) - RECOMMENDED
```rust
// For Control Plane operations
use sdk_admin::HodeiAdmin;
let mut admin = HodeiAdmin::connect("http://localhost:50051").await?;
let store = admin.create_policy_store("test", None).await?;

// For Data Plane operations
use hodei_permissions_sdk::Client;
let sdk = Client::connect("http://localhost:50051").await?;
let response = sdk.is_authorized(&store.policy_store_id, request).await?;
```

See `docs/MIGRATION_GUIDE_SDK.md` for complete migration instructions.

### Test Results

- ✅ 46/46 tests passing
- ✅ 100% Data Plane API coverage
- ✅ All backward compatibility checks passing
- ✅ Documentation examples verified

### Known Issues

- None at this time

### Contributors

- Engineering Team - Architecture refactoring
- QA Team - Test coverage and validation

---

## [0.1.0] - 2025-10-01

### Added
- Initial release with monolithic SDK
- Both Data Plane and Control Plane operations
- Basic authorization checking
- Policy management
- Schema handling

---

**Note**: This changelog will be updated for v0.2.0 release. See GitHub releases for detailed release notes.
