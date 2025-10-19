//! Tower Layer implementation for Hodei Verified Permissions

use crate::AuthorizationClient;
use crate::middleware::{VerifiedPermissionsService, DefaultExtractor};
use std::sync::Arc;
use tower_layer::Layer;

/// Tower Layer for Hodei Verified Permissions authorization
///
/// This layer wraps services to add authorization checks using
/// Hodei Verified Permissions with JWT tokens.
///
/// # Example
///
/// ```rust,ignore
/// use hodei_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
/// use axum::{Router, routing::get};
///
/// #[tokio::main]
/// async fn main() {
///     let client = AuthorizationClient::connect("http://localhost:50051")
///         .await
///         .unwrap();
///
///     let layer = VerifiedPermissionsLayer::new(
///         client,
///         "policy-store-123",
///         "identity-source-456"
///     );
///
///     let app = Router::new()
///         .route("/api/documents", get(list_documents))
///         .layer(layer);
/// }
/// ```
#[derive(Clone)]
pub struct VerifiedPermissionsLayer {
    client: Arc<AuthorizationClient>,
    policy_store_id: String,
    identity_source_id: String,
}

impl VerifiedPermissionsLayer {
    /// Create a new VerifiedPermissionsLayer
    ///
    /// # Arguments
    ///
    /// * `client` - The authorization client
    /// * `policy_store_id` - The policy store ID to use for authorization
    /// * `identity_source_id` - The identity source ID for JWT validation
    pub fn new(
        client: AuthorizationClient,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
    ) -> Self {
        Self {
            client: Arc::new(client),
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
        }
    }

    /// Create a new layer from an Arc client (for sharing)
    pub fn from_arc(
        client: Arc<AuthorizationClient>,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
        }
    }
}

impl<S> Layer<S> for VerifiedPermissionsLayer {
    type Service = VerifiedPermissionsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let extractor = DefaultExtractor::new(
            self.policy_store_id.clone(),
            self.identity_source_id.clone(),
        );

        VerifiedPermissionsService::new(
            inner,
            self.client.clone(),
            self.policy_store_id.clone(),
            self.identity_source_id.clone(),
            Arc::new(extractor),
        )
    }
}
