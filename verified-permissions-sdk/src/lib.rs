//! Hodei Permissions SDK - Lightweight Data Plane Client
//!
//! Ergonomic client SDK for Hodei Verified Permissions authorization service.
//! This SDK focuses exclusively on authorization checking (Data Plane).
//!
//! For policy store, schema, and policy management (Control Plane), use:
//! - CLI Tool: `hodei` command
//! - Library: `hodei_cli` crate with `HodeiAdmin` struct
//!
//! # Quick Start
//!
//! ```no_run
//! use verified_permissions_sdk::AuthorizationClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = AuthorizationClient::connect("http://localhost:50051").await?;
//!
//!     let response = client
//!         .is_authorized(
//!
//! "policy-store-id",
//!             "User::alice",
//!             "Action::view",
//!             "Document::doc123"
//!         )
//!         .await?;
//!
//!     println!("Decision: {:?}", response.decision());
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - **Data Plane Only**: Authorization checks, JWT validation, batch operations
//! - **Lightweight**: Simple API focused on authorization
//! - **Middleware**: Optional Axum/Tower integration
//! - **Builders**: Fluent API for complex requests
//! - **Testable**: Trait-based design for easy mocking
//!
//! # Migration from v0.1.x
//!
//! Control Plane operations (create_policy_store, put_schema, etc.) have been moved.
//! See the [migration guide](https://github.com/rubentxu/hodei-verified-permissions/blob/main/docs/MIGRATION_GUIDE_SDK.md)
//! for details on how to update your code.

pub mod auth_decision;
pub mod authorization;
pub mod builders;
pub mod client;
pub mod client_trait;
pub mod entities;
pub mod error;
pub mod validation;

// Schema generation module (optional, requires "schema" feature)
#[cfg(feature = "schema")]
pub mod schema;

// Middleware module (optional, requires "middleware" feature)
#[cfg(feature = "middleware")]
pub mod middleware;

// Compatibility layer for v0.1.x users (optional, requires "compat" feature)
#[cfg(feature = "compat")]
pub mod compat;

// Re-export generated proto types (Data Plane only)
pub mod proto {
    tonic::include_proto!("authorization");
}

pub use auth_decision::AuthorizationDecision;
pub use builders::*;
pub use client::AuthorizationClient;
pub use client_trait::AuthorizationClientTrait;
pub use error::{Result, SdkError};
pub use validation::OidcConfigValidator;

// Re-export Data Plane types (most commonly used)
pub use proto::{
    BatchIsAuthorizedRequest, BatchIsAuthorizedResponse, Decision, EntityIdentifier,
    IsAuthorizedRequest, IsAuthorizedResponse, IsAuthorizedWithTokenRequest,
};

// Re-export compatibility layer (if enabled)
#[cfg(feature = "compat")]
pub use compat::{
    create_identity_source_deprecated, create_policy_deprecated,
    create_policy_from_template_deprecated, create_policy_store_deprecated,
    create_policy_template_deprecated, delete_identity_source_deprecated, delete_policy_deprecated,
    delete_policy_store_deprecated, delete_policy_template_deprecated,
    get_identity_source_deprecated, get_policy_deprecated, get_policy_store_deprecated,
    get_policy_template_deprecated, get_schema_deprecated, list_identity_sources_deprecated,
    list_policies_deprecated, list_policy_stores_deprecated, list_policy_templates_deprecated,
    put_schema_deprecated, update_policy_deprecated,
};
