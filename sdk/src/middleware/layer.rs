// Tower Layer implementation for Hodei Verified Permissions

use crate::AuthorizationClient;
use crate::middleware::{VerifiedPermissionsService, DefaultExtractor};
use std::sync::Arc;
use tower_layer::Layer;

#[cfg(feature = "runtime-mapping")]
use crate::schema::SimpleRestMapping;

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
    skipped_endpoints: Vec<SkippedEndpoint>,
    #[cfg(feature = "runtime-mapping")]
    simple_rest_mapping: Option<Arc<SimpleRestMapping>>,
}

/// Endpoint to skip authorization
#[derive(Debug, Clone)]
pub struct SkippedEndpoint {
    /// HTTP verb (lowercase)
    pub http_verb: String,
    /// Path pattern
    pub path: String,
    /// Whether to use exact match or prefix match
    pub match_type: MatchType,
}

/// Type of path matching
#[derive(Debug, Clone)]
pub enum MatchType {
    /// Exact path match
    Exact,
    /// Prefix match (path starts with this)
    Prefix,
    /// Wildcard match
    Wildcard,
}

impl SkippedEndpoint {
    /// Create a new skipped endpoint with exact match
    pub fn new(http_verb: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            http_verb: http_verb.into().to_lowercase(),
            path: path.into(),
            match_type: MatchType::Exact,
        }
    }
    
    /// Create a new skipped endpoint with prefix match
    pub fn prefix(http_verb: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            http_verb: http_verb.into().to_lowercase(),
            path: path.into(),
            match_type: MatchType::Prefix,
        }
    }
    
    /// Create a new skipped endpoint with wildcard match
    pub fn wildcard(http_verb: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            http_verb: http_verb.into().to_lowercase(),
            path: pattern.into(),
            match_type: MatchType::Wildcard,
        }
    }
    
    /// Create from all HTTP verbs
    pub fn all_verbs(path: impl Into<String>) -> Self {
        Self {
            http_verb: "*".to_string(),
            path: path.into(),
            match_type: MatchType::Exact,
        }
    }
    
    /// Check if a request matches this skipped endpoint
    pub fn matches(&self, method: &http::Method, path: &str) -> bool {
        // Check HTTP verb
        if self.http_verb != "*" {
            let method_str = method.as_str().to_lowercase();
            if method_str != self.http_verb {
                return false;
            }
        }
        
        // Check path based on match type
        match &self.match_type {
            MatchType::Exact => path == self.path,
            MatchType::Prefix => path.starts_with(&self.path),
            MatchType::Wildcard => {
                // Simple wildcard support for now
                if self.path.ends_with("*") {
                    let prefix = &self.path[..self.path.len() - 1];
                    path.starts_with(prefix)
                } else {
                    path == self.path
                }
            }
        }
    }
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
            skipped_endpoints: Vec::new(),
            #[cfg(feature = "runtime-mapping")]
            simple_rest_mapping: None,
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
            skipped_endpoints: Vec::new(),
            #[cfg(feature = "runtime-mapping")]
            simple_rest_mapping: None,
        }
    }
    
    /// Load a SimpleRest mapping from a Cedar schema JSON
    #[cfg(feature = "runtime-mapping")]
    pub fn with_simple_rest_mapping(mut self, schema_json: &str) -> anyhow::Result<Self> {
        let mapping = SimpleRestMapping::from_schema_json(schema_json)?;
        self.simple_rest_mapping = Some(Arc::new(mapping));
        Ok(self)
    }
    
    /// Set a SimpleRest mapping directly
    #[cfg(feature = "runtime-mapping")]
    pub fn with_mapping(mut self, mapping: SimpleRestMapping) -> Self {
        self.simple_rest_mapping = Some(Arc::new(mapping));
        self
    }
    
    /// Skip authorization for a specific endpoint
    pub fn skip_endpoint(mut self, http_verb: &str, path: &str) -> Self {
        self.skipped_endpoints.push(SkippedEndpoint::new(http_verb, path));
        self
    }
    
    /// Skip authorization for a path prefix
    pub fn skip_prefix(mut self, http_verb: &str, path_prefix: &str) -> Self {
        self.skipped_endpoints.push(SkippedEndpoint::prefix(http_verb, path_prefix));
        self
    }
    
    /// Skip authorization for a wildcard path
    pub fn skip_wildcard(mut self, http_verb: &str, pattern: &str) -> Self {
        self.skipped_endpoints.push(SkippedEndpoint::wildcard(http_verb, pattern));
        self
    }
    
    /// Skip authorization for all HTTP verbs on a path
    pub fn skip_all_verbs(mut self, path: &str) -> Self {
        self.skipped_endpoints.push(SkippedEndpoint::all_verbs(path));
        self
    }
    
    /// Skip authorization for multiple endpoints
    pub fn skip_endpoints(mut self, endpoints: Vec<SkippedEndpoint>) -> Self {
        self.skipped_endpoints.extend(endpoints);
        self
    }
    
    /// Check if a request should skip authorization
    pub fn should_skip(&self, method: &http::Method, path: &str) -> bool {
        self.skipped_endpoints.iter()
            .any(|endpoint| endpoint.matches(method, path))
    }
    
    /// Get the list of skipped endpoints
    pub fn skipped_endpoints(&self) -> &[SkippedEndpoint] {
        &self.skipped_endpoints
    }
}

impl<S> Layer<S> for VerifiedPermissionsLayer {
    type Service = VerifiedPermissionsService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        let extractor = DefaultExtractor::new(
            self.policy_store_id.clone(),
            self.identity_source_id.clone(),
        );

        #[cfg(feature = "runtime-mapping")]
        {
            if self.simple_rest_mapping.is_some() {
                return VerifiedPermissionsService::with_mapping(
                    inner,
                    self.client.clone(),
                    self.policy_store_id.clone(),
                    self.identity_source_id.clone(),
                    std::sync::Arc::new(extractor),
                    self.skipped_endpoints.clone(),
                    self.simple_rest_mapping.clone(),
                );
            }
        }

        VerifiedPermissionsService::new(
            inner,
            self.client.clone(),
            self.policy_store_id.clone(),
            self.identity_source_id.clone(),
            std::sync::Arc::new(extractor),
            self.skipped_endpoints.clone(),
        )
    }
}
