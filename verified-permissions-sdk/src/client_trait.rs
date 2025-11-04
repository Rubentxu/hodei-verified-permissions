//! Authorization client trait for testability and mocking
//!
//! This trait abstracts the authorization client interface (Data Plane only), allowing for
//! easy mocking and testing of authorization logic without requiring
//! a real connection to the authorization service.
//!
//! For policy and schema management (Control Plane), use the CLI tool or HodeiAdmin library.
//!
//! # Example
//!
//! ```rust,ignore
//! use verified_permissions_sdk::client_trait::AuthorizationClientTrait;
//! use async_trait::async_trait;
//! use verified_permissions_sdk::proto::{IsAuthorizedResponse, Decision};
//!
//! struct MockClient;
//!
//! #[async_trait]
//! impl AuthorizationClientTrait for MockClient {
//!     async fn is_authorized(
//!         &self,
//!         policy_store_id: &str,
//!         principal: &str,
//!         action: &str,
//!         resource: &str,
//!     ) -> Result<IsAuthorizedResponse> {
//!         Ok(IsAuthorizedResponse {
//!             decision: Decision::Allow as i32,
//!             determining_policies: vec!["mock-policy".to_string()],
//!             errors: vec![],
//!         })
//!     }
//!
//!     // ... implement other Data Plane methods
//! }
//! ```

use crate::error::Result;
use crate::proto::*;
use async_trait::async_trait;

/// Trait for authorization client operations (Data Plane only)
///
/// This trait provides an abstraction for authorization checking operations.
/// It's designed for testability and mocking, allowing you to replace the
/// real client with a mock implementation in tests.
///
/// For operations like creating policy stores, managing policies, or schemas,
/// use the CLI tool or HodeiAdmin library instead.
///
/// # Example
///
/// ```rust,ignore
/// use verified_permissions_sdk::client_trait::AuthorizationClientTrait;
/// use async_trait::async_trait;
///
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///
///     struct TestAuthClient;
///
///     #[async_trait]
///     impl AuthorizationClientTrait for TestAuthClient {
///         async fn is_authorized(&self, ...) -> Result<IsAuthorizedResponse> {
///             // Mock implementation
///             Ok(IsAuthorizedResponse { ... })
///         }
///         // ... implement other methods
///     }
/// }
/// ```
#[async_trait]
pub trait AuthorizationClientTrait: Send + Sync {
    /// Check if an action is authorized
    ///
    /// Given a principal, action, and resource, determines whether the action is allowed.
    ///
    /// # Arguments
    ///
    /// * `policy_store_id` - The ID of the policy store to evaluate against
    /// * `principal` - The principal making the request (format: "Type::id")
    /// * `action` - The action being performed (format: "Type::id")
    /// * `resource` - The resource being accessed (format: "Type::id")
    ///
    /// # Returns
    ///
    /// Returns `Result<IsAuthorizedResponse>` with the authorization decision.
    async fn is_authorized(
        &self,
        policy_store_id: &str,
        principal: &str,
        action: &str,
        resource: &str,
    ) -> Result<IsAuthorizedResponse>;

    /// Check authorization with entities and context
    ///
    /// Performs authorization check with additional context and entity data.
    /// This allows for more complex authorization scenarios using Cedar policies.
    ///
    /// # Arguments
    ///
    /// * `request` - A pre-built `IsAuthorizedRequest` with entities and context
    ///
    /// # Returns
    ///
    /// Returns `Result<IsAuthorizedResponse>` with the authorization decision.
    async fn is_authorized_with_context(
        &self,
        request: IsAuthorizedRequest,
    ) -> Result<IsAuthorizedResponse>;

    /// Batch authorization check
    ///
    /// Performs multiple authorization checks in a single call.
    /// More efficient than individual calls when checking multiple permissions.
    ///
    /// # Arguments
    ///
    /// * `policy_store_id` - The ID of the policy store to evaluate against
    /// * `requests` - A vector of `IsAuthorizedRequest` to evaluate
    ///
    /// # Returns
    ///
    /// Returns `Result<BatchIsAuthorizedResponse>` with all authorization decisions.
    async fn batch_is_authorized(
        &self,
        policy_store_id: &str,
        requests: Vec<IsAuthorizedRequest>,
    ) -> Result<BatchIsAuthorizedResponse>;

    /// Check authorization with JWT token
    ///
    /// Validates a JWT token against an identity source and performs authorization
    /// using the token's claims.
    ///
    /// # Arguments
    ///
    /// * `policy_store_id` - The ID of the policy store to evaluate against
    /// * `identity_source_id` - The ID of the identity source for token validation
    /// * `access_token` - The JWT access token
    /// * `action` - The action being performed (format: "Type::id")
    /// * `resource` - The resource being accessed (format: "Type::id")
    ///
    /// # Returns
    ///
    /// Returns `Result<IsAuthorizedResponse>` with the authorization decision.
    async fn is_authorized_with_token(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
        access_token: &str,
        action: &str,
        resource: &str,
    ) -> Result<IsAuthorizedResponse>;

    /// Check authorization with JWT token and context
    ///
    /// Validates a JWT token and performs authorization with additional
    /// context and entity data.
    ///
    /// # Arguments
    ///
    /// * `request` - A pre-built `IsAuthorizedWithTokenRequest` with entities and context
    ///
    /// # Returns
    ///
    /// Returns `Result<IsAuthorizedResponse>` with the authorization decision.
    async fn is_authorized_with_token_and_context(
        &self,
        request: IsAuthorizedWithTokenRequest,
    ) -> Result<IsAuthorizedResponse>;
}
