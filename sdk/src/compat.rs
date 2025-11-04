//! Deprecated compatibility layer for v0.1.x API
//!
//! This module provides deprecated methods to help users migrate from v0.1.x
//! to v0.2.x. These methods will return errors guiding users to the CLI tool
//! or HodeiAdmin library for Control Plane operations.
//!
//! # Migration
//!
//! For Control Plane operations (policy store, schema, policy management),
//! use the CLI tool or HodeiAdmin library:
//!
//! ```bash
//! # CLI Tool
//! hodei init my-app
//! hodei schema apply --file=schema.json
//! hodei policy create --store-id=... --id=... --statement=...
//! ```
//!
//! ```rust
//! // HodeiAdmin Library
//! use hodei_cli::HodeiAdmin;
//!
//! let admin = HodeiAdmin::connect("http://localhost:50051").await?;
//! let store = admin.create_policy_store("MyApp", None).await?;
//! ```

use crate::error::{Result, SdkError};

use crate::proto::*;

/// Error message for deprecated Control Plane operations
const DEPRECATED_MSG: &str = r#"
This operation has been moved to the CLI tool or HodeiAdmin library.

For Control Plane operations (policy store, schema, policy management):

CLI Tool:
  hodei init my-app
  hodei schema apply --file=schema.json
  hodei policy create --store-id=... --id=... --statement=...

Library (programmatic):
  use hodei_cli::HodeiAdmin;

  let admin = HodeiAdmin::connect("http://localhost:50051").await?;
  let store = admin.create_policy_store("MyApp", None).await?;

See the migration guide:
  https://github.com/rubentxu/hodei-verified-permissions/blob/main/docs/MIGRATION_GUIDE_SDK.md
"#;

/// Result type for deprecated operations
pub type DeprecatedResult<T> = Result<T>;

/// Creates a deprecated error with guidance
fn deprecated_error(operation: &str) -> SdkError {
    SdkError::InvalidRequest(format!(
        "Operation '{}' is deprecated. {}\nOperation: {}",
        operation, DEPRECATED_MSG, operation
    ))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn create_policy_store_deprecated(
    _name: String,
    _description: Option<String>,
) -> DeprecatedResult<CreatePolicyStoreResponse> {
    Err(deprecated_error("create_policy_store"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn get_policy_store_deprecated(
    _policy_store_id: impl Into<String>,
) -> DeprecatedResult<GetPolicyStoreResponse> {
    Err(deprecated_error("get_policy_store"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn list_policy_stores_deprecated(
    _max_results: Option<i32>,
    _next_token: Option<String>,
) -> DeprecatedResult<ListPolicyStoresResponse> {
    Err(deprecated_error("list_policy_stores"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn delete_policy_store_deprecated(
    _policy_store_id: impl Into<String>,
) -> DeprecatedResult<DeletePolicyStoreResponse> {
    Err(deprecated_error("delete_policy_store"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn put_schema_deprecated(
    _policy_store_id: impl Into<String>,
    _schema: impl Into<String>,
) -> DeprecatedResult<PutSchemaResponse> {
    Err(deprecated_error("put_schema"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn get_schema_deprecated(
    _policy_store_id: impl Into<String>,
) -> DeprecatedResult<GetSchemaResponse> {
    Err(deprecated_error("get_schema"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn create_policy_deprecated(
    _policy_store_id: impl Into<String>,
    _policy_id: impl Into<String>,
    _statement: impl Into<String>,
    _description: Option<String>,
) -> DeprecatedResult<CreatePolicyResponse> {
    Err(deprecated_error("create_policy"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn get_policy_deprecated(
    _policy_store_id: impl Into<String>,
    _policy_id: impl Into<String>,
) -> DeprecatedResult<GetPolicyResponse> {
    Err(deprecated_error("get_policy"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn list_policies_deprecated(
    _policy_store_id: impl Into<String>,
) -> DeprecatedResult<ListPoliciesResponse> {
    Err(deprecated_error("list_policies"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn update_policy_deprecated(
    _policy_store_id: impl Into<String>,
    _policy_id: impl Into<String>,
    _statement: impl Into<String>,
    _description: Option<String>,
) -> DeprecatedResult<UpdatePolicyResponse> {
    Err(deprecated_error("update_policy"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn delete_policy_deprecated(
    _policy_store_id: impl Into<String>,
    _policy_id: impl Into<String>,
) -> DeprecatedResult<DeletePolicyResponse> {
    Err(deprecated_error("delete_policy"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn create_identity_source_deprecated(
    _policy_store_id: impl Into<String>,
    _configuration: IdentitySourceConfiguration,
    _claims_mapping: Option<ClaimsMappingConfiguration>,
    _description: Option<String>,
) -> DeprecatedResult<CreateIdentitySourceResponse> {
    Err(deprecated_error("create_identity_source"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn get_identity_source_deprecated(
    _policy_store_id: impl Into<String>,
    _identity_source_id: impl Into<String>,
) -> DeprecatedResult<GetIdentitySourceResponse> {
    Err(deprecated_error("get_identity_source"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn list_identity_sources_deprecated(
    _policy_store_id: impl Into<String>,
) -> DeprecatedResult<ListIdentitySourcesResponse> {
    Err(deprecated_error("list_identity_sources"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn delete_identity_source_deprecated(
    _policy_store_id: impl Into<String>,
    _identity_source_id: impl Into<String>,
) -> DeprecatedResult<DeleteIdentitySourceResponse> {
    Err(deprecated_error("delete_identity_source"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn create_policy_template_deprecated(
    _policy_store_id: impl Into<String>,
    _template_id: impl Into<String>,
    _statement: impl Into<String>,
    _description: Option<String>,
) -> DeprecatedResult<CreatePolicyTemplateResponse> {
    Err(deprecated_error("create_policy_template"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn get_policy_template_deprecated(
    _policy_store_id: impl Into<String>,
    _template_id: impl Into<String>,
) -> DeprecatedResult<GetPolicyTemplateResponse> {
    Err(deprecated_error("get_policy_template"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn list_policy_templates_deprecated(
    _policy_store_id: impl Into<String>,
) -> DeprecatedResult<ListPolicyTemplatesResponse> {
    Err(deprecated_error("list_policy_templates"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn delete_policy_template_deprecated(
    _policy_store_id: impl Into<String>,
    _template_id: impl Into<String>,
) -> DeprecatedResult<DeletePolicyTemplateResponse> {
    Err(deprecated_error("delete_policy_template"))
}

/// Deprecated: Use CLI tool or HodeiAdmin library instead
#[deprecated(since = "0.2.0", note = "Use CLI tool or HodeiAdmin library instead")]
pub async fn create_policy_from_template_deprecated(
    _policy_store_id: impl Into<String>,
    _policy_id: impl Into<String>,
    _template_id: impl Into<String>,
    _principal: impl Into<String>,
    _resource: impl Into<String>,
    _description: Option<String>,
) -> DeprecatedResult<CreatePolicyResponse> {
    Err(deprecated_error("create_policy_from_template"))
}
