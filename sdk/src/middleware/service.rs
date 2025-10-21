//! Tower Service implementation for Hodei Verified Permissions

use crate::AuthorizationClient;
use crate::middleware::{AuthorizationRequestExtractor, DefaultExtractor, MiddlewareError, SkippedEndpoint};
use crate::proto::Decision;
use http::{Request, Response};
use http_body::Body;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Service;

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
        }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for VerifiedPermissionsService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    ReqBody: Body + Send + Sync + 'static,
    ResBody: Body + Send + 'static,
    ResBody::Data: Send,
    ResBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response<ResBody>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let client = self.client.clone();
        let policy_store_id = self.policy_store_id.clone();
        let identity_source_id = self.identity_source_id.clone();
        let extractor = self.extractor.clone();
        let skipped_endpoints = self.skipped_endpoints.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Check if this endpoint should skip authorization
            let method = req.method();
            let path = req.uri().path();
            
            if skipped_endpoints.iter().any(|endpoint| endpoint.matches(method, path)) {
                // Skip authorization, forward directly to inner service
                return match inner.call(req).await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(e.into()),
                };
            }
            
            // Extract JWT token from Authorization header
            let token = match DefaultExtractor::extract_token(&req) {
                Ok(t) => t,
                Err(e) => {
                    let error = MiddlewareError::AuthorizationHeader(e.to_string());
                    return Err(Box::new(error) as Box<dyn std::error::Error + Send + Sync>);
                }
            };

            // Extract authorization request parts
            let parts = match extractor.extract(&req).await {
                Ok(p) => p,
                Err(e) => {
                    let error = MiddlewareError::ExtractionFailed(e.to_string());
                    return Err(Box::new(error) as Box<dyn std::error::Error + Send + Sync>);
                }
            };

            // Call IsAuthorizedWithToken
            let auth_result = client
                .is_authorized_with_token(
                    &policy_store_id,
                    &identity_source_id,
                    &token,
                    &parts.action,
                    &parts.resource,
                )
                .await;

            match auth_result {
                Ok(response) => {
                    // Check decision
                    if response.decision == Decision::Allow as i32 {
                        // Allow: forward request to inner service
                        match inner.call(req).await {
                            Ok(response) => Ok(response),
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        // Deny: return error
                        let error = MiddlewareError::AccessDenied(format!(
                            "Access denied for action '{}' on resource '{}'",
                            parts.action, parts.resource
                        ));
                        Err(Box::new(error) as Box<dyn std::error::Error + Send + Sync>)
                    }
                }
                Err(e) => {
                    let error = MiddlewareError::AuthorizationFailed(e.to_string());
                    Err(Box::new(error) as Box<dyn std::error::Error + Send + Sync>)
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
