//! Hodei Permissions SDK
//!
//! Ergonomic client SDK for Hodei Verified Permissions authorization service.
//!
//! # Example
//!
//! ```no_run
//! use hodei_permissions_sdk::{AuthorizationClient, IsAuthorizedRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = AuthorizationClient::connect("http://localhost:50051").await?;
//!     
//!     let response = client
//!         .is_authorized(
//!             "policy-store-id",
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

pub mod client;
pub mod client_trait;
pub mod builders;
pub mod error;
pub mod entities;
pub mod auth_decision;
pub mod validation;
pub mod authorization;

// Schema generation module (optional, requires "schema" feature)
#[cfg(feature = "schema")]
pub mod schema;

// Middleware module (optional, requires "middleware" feature)
#[cfg(feature = "middleware")]
pub mod middleware;

// Re-export generated proto types
pub mod proto {
    tonic::include_proto!("authorization");
}

pub use client::AuthorizationClient;
pub use client_trait::AuthorizationClientTrait;
pub use builders::*;
pub use error::{SdkError, Result};
pub use auth_decision::AuthorizationDecision;
pub use validation::OidcConfigValidator;

// Re-export common types
pub use proto::{
    Decision,
    EntityIdentifier,
    IsAuthorizedRequest,
    IsAuthorizedResponse,
    IsAuthorizedWithTokenRequest,
};
