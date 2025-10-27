//! Authorization request extractor trait
//!
//! This module defines the trait that allows users to customize how
//! authorization requests are extracted from HTTP requests.

use async_trait::async_trait;
use http::Request;

/// Parts of an authorization request extracted from an HTTP request
#[derive(Debug, Clone)]
pub struct AuthorizationRequestParts {
    /// The principal (user/entity) making the request
    pub principal: String,
    
    /// The action being performed
    pub action: String,
    
    /// The resource being accessed
    pub resource: String,
    
    /// Optional context for the authorization decision
    pub context: Option<serde_json::Value>,
}

/// Trait for extracting authorization request parts from HTTP requests
///
/// Implement this trait to customize how your middleware extracts
/// the principal, action, and resource from incoming HTTP requests.
///
/// # Example
///
/// ```rust,ignore
/// use hodei_permissions_sdk::middleware::{AuthorizationRequestExtractor, AuthorizationRequestParts};
/// use async_trait::async_trait;
/// use http::Request;
///
/// struct MyExtractor;
///
/// #[async_trait]
/// impl<B> AuthorizationRequestExtractor<B> for MyExtractor
/// where
///     B: Send,
/// {
///     type Error = Box<dyn std::error::Error + Send + Sync>;
///
///     async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
///         // Extract user from header
///         let principal = req.headers()
///             .get("x-user-id")
///             .and_then(|h| h.to_str().ok())
///             .ok_or("Missing x-user-id header")?
///             .to_string();
///
///         // Extract action from method
///         let action = match req.method().as_str() {
///             "GET" => "read",
///             "POST" => "create",
///             "PUT" | "PATCH" => "update",
///             "DELETE" => "delete",
///             _ => "unknown",
///         }.to_string();
///
///         // Extract resource from path
///         let resource = req.uri().path().to_string();
///
///         Ok(AuthorizationRequestParts {
///             principal,
///             action,
///             resource,
///             context: None,
///         })
///     }
/// }
/// ```
#[async_trait]
pub trait AuthorizationRequestExtractor<B>: Send + Sync
where
    B: Send,
{
    /// Error type returned by the extractor
    type Error: std::error::Error + Send + Sync + 'static;

    /// Extract authorization request parts from an HTTP request
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to extract from
    ///
    /// # Returns
    ///
    /// Returns the extracted authorization request parts or an error
    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error>;
}

/// Default extractor that extracts JWT from Authorization header
///
/// This extractor:
/// - Extracts JWT token from `Authorization: Bearer <token>` header
/// - Maps HTTP methods to actions (GET -> read, POST -> create, etc.)
/// - Uses the URI path as the resource
/// - Passes the JWT token to IsAuthorizedWithToken
#[derive(Debug, Clone, Default)]
pub struct DefaultExtractor {
    /// Policy store ID
    pub policy_store_id: String,
    /// Identity source ID
    pub identity_source_id: String,
}

impl DefaultExtractor {
    /// Create a new default extractor
    pub fn new(policy_store_id: impl Into<String>, identity_source_id: impl Into<String>) -> Self {
        Self {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
        }
    }

    /// Extract JWT token from Authorization header
    pub fn extract_token<B>(req: &Request<B>) -> Result<String, crate::middleware::MiddlewareError> {
        let auth_header = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| crate::middleware::MiddlewareError::AuthorizationHeader(
                "Missing Authorization header".to_string()
            ))?;

        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            Ok(token.to_string())
        } else {
            Err(crate::middleware::MiddlewareError::AuthorizationHeader(
                "Invalid Authorization header format, expected 'Bearer <token>'".to_string()
            ))
        }
    }
}

#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for DefaultExtractor
where
    B: Send + Sync,
{
    type Error = crate::middleware::MiddlewareError;

    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
        // Extract JWT token from Authorization header
        let _token = Self::extract_token(req)?;

        // Map HTTP method to action
        let action = match req.method().as_str() {
            "GET" | "HEAD" => "Action::\"read\"",
            "POST" => "Action::\"create\"",
            "PUT" | "PATCH" => "Action::\"update\"",
            "DELETE" => "Action::\"delete\"",
            method => return Err(crate::middleware::MiddlewareError::ExtractionFailed(
                format!("Unsupported HTTP method: {}", method)
            )),
        };

        // Use URI path as resource
        let path = req.uri().path();
        let resource = if path.is_empty() || path == "/" {
            "Resource::\"root\"".to_string()
        } else {
            format!("Resource::\"{}\"", path.trim_start_matches('/'))
        };

        // Principal will be extracted from JWT by the server
        let principal = String::new(); // Not used with IsAuthorizedWithToken

        Ok(AuthorizationRequestParts {
            principal,
            action: action.to_string(),
            resource,
            context: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Method, Request};

    #[tokio::test]
    async fn test_default_extractor_get_request() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/documents/123")
            .header("Authorization", "Bearer test_token_abc123")
            .body(())
            .unwrap();

        let extractor = DefaultExtractor::new("store-123", "identity-456");
        let parts = extractor.extract(&req).await.unwrap();

        assert_eq!(parts.action, "Action::\"read\"");
        assert_eq!(parts.resource, "Resource::\"api/documents/123\"");
    }

    #[tokio::test]
    async fn test_default_extractor_post_request() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/documents")
            .header("Authorization", "Bearer token123")
            .body(())
            .unwrap();

        let extractor = DefaultExtractor::new("store-123", "identity-456");
        let parts = extractor.extract(&req).await.unwrap();

        assert_eq!(parts.action, "Action::\"create\"");
        assert_eq!(parts.resource, "Resource::\"api/documents\"");
    }

    #[tokio::test]
    async fn test_extract_token() {
        let req = Request::builder()
            .header("Authorization", "Bearer my_jwt_token_here")
            .body(())
            .unwrap();

        let token = DefaultExtractor::extract_token(&req).unwrap();
        assert_eq!(token, "my_jwt_token_here");
    }

    #[tokio::test]
    async fn test_extract_token_missing() {
        let req = Request::builder()
            .body(())
            .unwrap();

        let result = DefaultExtractor::extract_token(&req);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing Authorization"));
    }

    #[tokio::test]
    async fn test_extract_token_invalid_format() {
        let req = Request::builder()
            .header("Authorization", "Basic dXNlcjpwYXNz")
            .body(())
            .unwrap();

        let result = DefaultExtractor::extract_token(&req);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected 'Bearer"));
    }
}

/// Parameterized extractor that supports path parameters
///
/// This extractor allows you to define path parameter patterns and extract
/// them from the URI path. It's useful for RESTful APIs where resource IDs
/// are part of the path.
///
/// # Example
///
/// ```rust,ignore
/// use hodei_permissions_sdk::middleware::ParameterizedExtractor;
/// use std::collections::HashMap;
///
/// let mut params = HashMap::new();
/// params.insert("resource_id".to_string(), "123".to_string());
/// params.insert("action".to_string(), "read".to_string());
///
/// let extractor = ParameterizedExtractor::new(
///     "policy-store-id",
///     "identity-source-id",
///     params,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct ParameterizedExtractor {
    /// Policy store ID
    pub policy_store_id: String,
    /// Identity source ID
    pub identity_source_id: String,
    /// Path parameters extracted from the URI
    pub path_parameters: std::collections::HashMap<String, String>,
}

impl ParameterizedExtractor {
    /// Create a new parameterized extractor
    pub fn new(
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
        path_parameters: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            path_parameters,
        }
    }

    /// Get a path parameter value
    pub fn get_parameter(&self, key: &str) -> Option<&str> {
        self.path_parameters.get(key).map(|s| s.as_str())
    }

    /// Set a path parameter
    pub fn set_parameter(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.path_parameters.insert(key.into(), value.into());
    }
}

#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for ParameterizedExtractor
where
    B: Send + Sync,
{
    type Error = crate::middleware::MiddlewareError;

    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
        // Extract JWT token from Authorization header
        let _token = DefaultExtractor::extract_token(req)?;

        // Map HTTP method to action
        let action = match req.method().as_str() {
            "GET" | "HEAD" => "Action::\"read\"",
            "POST" => "Action::\"create\"",
            "PUT" | "PATCH" => "Action::\"update\"",
            "DELETE" => "Action::\"delete\"",
            method => return Err(crate::middleware::MiddlewareError::ExtractionFailed(
                format!("Unsupported HTTP method: {}", method)
            )),
        };

        // Build resource from path and parameters
        let path = req.uri().path();
        let mut resource = if path.is_empty() || path == "/" {
            "Resource::\"root\"".to_string()
        } else {
            format!("Resource::\"{}\"", path.trim_start_matches('/'))
        };

        // Append path parameters to resource if available
        if !self.path_parameters.is_empty() {
            let params_str = self
                .path_parameters
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            resource = format!("{}?{}", resource, params_str);
        }

        // Principal will be extracted from JWT by the server
        let principal = String::new();

        Ok(AuthorizationRequestParts {
            principal,
            action: action.to_string(),
            resource,
            context: None,
        })
    }
}

#[cfg(test)]
mod parameterized_tests {
    use super::*;
    use http::{Method, Request};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_parameterized_extractor_with_params() {
        let mut params = HashMap::new();
        params.insert("resource_id".to_string(), "doc-123".to_string());
        params.insert("version".to_string(), "2".to_string());

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/documents/123")
            .header("Authorization", "Bearer test_token")
            .body(())
            .unwrap();

        let extractor = ParameterizedExtractor::new("store-123", "identity-456", params);
        let parts = extractor.extract(&req).await.unwrap();

        assert_eq!(parts.action, "Action::\"read\"");
        assert!(parts.resource.contains("resource_id=doc-123"));
        assert!(parts.resource.contains("version=2"));
    }

    #[test]
    fn test_parameterized_extractor_get_parameter() {
        let mut params = HashMap::new();
        params.insert("user_id".to_string(), "user-456".to_string());

        let extractor = ParameterizedExtractor::new("store-123", "identity-456", params);
        assert_eq!(extractor.get_parameter("user_id"), Some("user-456"));
        assert_eq!(extractor.get_parameter("nonexistent"), None);
    }

    #[test]
    fn test_parameterized_extractor_set_parameter() {
        let params = HashMap::new();
        let mut extractor = ParameterizedExtractor::new("store-123", "identity-456", params);
        
        extractor.set_parameter("resource_id", "res-789");
        assert_eq!(extractor.get_parameter("resource_id"), Some("res-789"));
    }
}
