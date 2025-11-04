//! Axum/Tower middleware for Hodei Verified Permissions
//!
//! This module provides middleware for protecting HTTP routes with
//! authorization checks using Hodei Verified Permissions.
//!
//! # Features
//!
//! This module is only available when the `middleware` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! hodei-permissions-sdk = { version = "0.1", features = ["middleware"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use verified_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
//! use axum::{Router, routing::get};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = AuthorizationClient::connect("http://localhost:50051")
//!         .await
//!         .unwrap();
//!
//!     let middleware = VerifiedPermissionsLayer::new(client, "policy-store-123");
//!
//!     let app = Router::new()
//!         .route("/api/documents", get(list_documents))
//!         .layer(middleware);
//!
//!     // Run server...
//! }
//! ```

pub mod extractor;
pub mod layer;
pub mod service;
pub mod error;

pub use extractor::{AuthorizationRequestExtractor, AuthorizationRequestParts, DefaultExtractor, ParameterizedExtractor};
pub use layer::{VerifiedPermissionsLayer, SkippedEndpoint, MatchType};
pub use service::VerifiedPermissionsService;
pub use error::MiddlewareError;
