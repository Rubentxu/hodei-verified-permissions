//! Tower Service implementation for Hodei Verified Permissions

use crate::AuthorizationClient;
use crate::middleware::{AuthorizationRequestExtractor, DefaultExtractor, SkippedEndpoint};
use crate::proto::Decision;
use http::{Request, Response};
use http_body::Body;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Service;

#[cfg(feature = "runtime-mapping")]
use crate::schema::SimpleRestMapping;
#[cfg(feature = "runtime-mapping")]
use form_urlencoded;

/// Tower Service for Hodei Verified Permissions authorization
///
/// This service intercepts requests, extracts JWT tokens, calls
/// IsAuthorizedWithToken, and either allows or denies the request.
#[derive(Clone)]
pub struct VerifiedPermissionsService<S> {
    inner: S,
    client: Arc<AuthorizationClient>,
    policy_store_id: String,
    identity_source_id: String,
    extractor: Arc<DefaultExtractor>,
    skipped_endpoints: Vec<SkippedEndpoint>,
    #[cfg(feature = "runtime-mapping")]
    simple_rest_mapping: Option<Arc<SimpleRestMapping>>,
}

impl<S> VerifiedPermissionsService<S> {
    /// Create a new VerifiedPermissionsService
    pub fn new(
        inner: S,
        client: Arc<AuthorizationClient>,
        policy_store_id: String,
        identity_source_id: String,
        extractor: Arc<DefaultExtractor>,
        skipped_endpoints: Vec<SkippedEndpoint>,
    ) -> Self {
        Self {
            inner,
            client,
            policy_store_id,
            identity_source_id,
            extractor,
            skipped_endpoints,
            #[cfg(feature = "runtime-mapping")]
            simple_rest_mapping: None,
        }
    }
    
    /// Create with SimpleRest mapping
    #[cfg(feature = "runtime-mapping")]
    pub fn with_mapping(
        inner: S,
        client: Arc<AuthorizationClient>,
        policy_store_id: String,
        identity_source_id: String,
        extractor: Arc<DefaultExtractor>,
        skipped_endpoints: Vec<SkippedEndpoint>,
        mapping: Option<Arc<SimpleRestMapping>>,
    ) -> Self {
        Self {
            inner,
            client,
            policy_store_id,
            identity_source_id,
            extractor,
            skipped_endpoints,
            simple_rest_mapping: mapping,
        }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for VerifiedPermissionsService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    ReqBody: Body + Send + Sync + 'static,
    ResBody: Body + Default + Send + 'static,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let client = self.client.clone();
        let policy_store_id = self.policy_store_id.clone();
        let identity_source_id = self.identity_source_id.clone();
        let extractor = self.extractor.clone();
        let skipped_endpoints = self.skipped_endpoints.clone();
        #[cfg(feature = "runtime-mapping")]
        let mapping = self.simple_rest_mapping.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Check if this endpoint should skip authorization
            let method = req.method();
            let path = req.uri().path();
            
            if skipped_endpoints.iter().any(|endpoint| endpoint.matches(method, path)) {
                // Skip authorization, forward directly to inner service
                return inner.call(req).await;
            }
            
            // Extract JWT token from Authorization header
            let token = match DefaultExtractor::extract_token(&req) {
                Ok(t) => t,
                Err(_) => {
                    // Return 401 Unauthorized response
                    let response = Response::builder()
                        .status(401)
                        .body(ResBody::default())
                        .unwrap();
                    return Ok(response);
                }
            };

            // Try to use SimpleRest mapping if available
            #[cfg(feature = "runtime-mapping")]
            let (action, resource, _context) = if let Some(ref mapping) = mapping {
                // Resolve route using mapping
                match mapping.resolve(method, path) {
                    Ok(resolved) => {
                        // Build context from path parameters and query string
                        let mut context_map = serde_json::Map::new();
                        
                        // Add path parameters
                        if !resolved.path_params.is_empty() {
                            let mut path_params = serde_json::Map::new();
                            for (key, value) in resolved.path_params {
                                path_params.insert(key, serde_json::Value::String(value));
                            }
                            context_map.insert(
                                "pathParameters".to_string(),
                                serde_json::Value::Object(path_params)
                            );
                        }
                        
                        // Add query parameters
                        if let Some(query) = req.uri().query() {
                            let mut query_params = serde_json::Map::new();
                            for (key, value) in form_urlencoded::parse(query.as_bytes()) {
                                query_params.insert(
                                    key.to_string(),
                                    serde_json::Value::String(value.to_string())
                                );
                            }
                            if !query_params.is_empty() {
                                context_map.insert(
                                    "queryStringParameters".to_string(),
                                    serde_json::Value::Object(query_params)
                                );
                            }
                        }
                        
                        let context_value = if !context_map.is_empty() {
                            Some(serde_json::Value::Object(context_map))
                        } else {
                            None
                        };
                        
                        // Use first resource type (or default to Application)
                        let resource_type = resolved.resource_types.first()
                            .cloned()
                            .unwrap_or_else(|| "Application".to_string());
                        
                        (resolved.action_name, resource_type, context_value)
                    }
                    Err(_) => {
                        // Fallback to extractor if mapping fails
                        let parts = match extractor.extract(&req).await {
                            Ok(p) => p,
                            Err(_) => {
                                // Return 400 Bad Request
                                let response = Response::builder()
                                    .status(400)
                                    .body(ResBody::default())
                                    .unwrap();
                                return Ok(response);
                            }
                        };
                        (parts.action, parts.resource, parts.context)
                    }
                }
            } else {
                // No mapping, use extractor
                let parts = match extractor.extract(&req).await {
                    Ok(p) => p,
                    Err(_) => {
                        // Return 400 Bad Request
                        let response = Response::builder()
                            .status(400)
                            .body(ResBody::default())
                            .unwrap();
                        return Ok(response);
                    }
                };
                (parts.action, parts.resource, parts.context)
            };
            
            #[cfg(not(feature = "runtime-mapping"))]
            let (action, resource, _context) = {
                // Extract authorization request parts
                let parts = match extractor.extract(&req).await {
                    Ok(p) => p,
                    Err(_) => {
                        // Return 400 Bad Request
                        let response = Response::builder()
                            .status(400)
                            .body(ResBody::default())
                            .unwrap();
                        return Ok(response);
                    }
                };
                (parts.action, parts.resource, parts.context)
            };

            // Call IsAuthorizedWithToken
            let auth_result = client
                .is_authorized_with_token(
                    &policy_store_id,
                    &identity_source_id,
                    &token,
                    &action,
                    &resource,
                )
                .await;

            match auth_result {
                Ok(response) => {
                    // Check decision
                    if response.decision == Decision::Allow as i32 {
                        // Allow: forward request to inner service
                        inner.call(req).await
                    } else {
                        // Deny: return 403 Forbidden
                        let response = Response::builder()
                            .status(403)
                            .body(ResBody::default())
                            .unwrap();
                        Ok(response)
                    }
                }
                Err(_) => {
                    // Authorization service error: return 500
                    let response = Response::builder()
                        .status(500)
                        .body(ResBody::default())
                        .unwrap();
                    Ok(response)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Method, Request, Response, StatusCode};
    use tower::ServiceExt;

    // Mock service that always returns 200 OK
    #[derive(Clone)]
    struct MockService;

    impl<B> Service<Request<B>> for MockService
    where
        B: Send + 'static,
    {
        type Response = Response<String>;
        type Error = Box<dyn std::error::Error + Send + Sync>;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: Request<B>) -> Self::Future {
            Box::pin(async {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body("Success".to_string())
                    .unwrap())
            })
        }
    }

    #[tokio::test]
    async fn test_service_missing_auth_header() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/test")
            .body(())
            .unwrap();

        // Note: This test would need a real client setup
        // Skipping for now as it requires integration testing
    }
}
