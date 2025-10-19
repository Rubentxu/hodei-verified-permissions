//! Middleware error types

use thiserror::Error;

/// Errors that can occur in the middleware
#[derive(Debug, Error)]
pub enum MiddlewareError {
    /// Missing or invalid Authorization header
    #[error("Authorization header error: {0}")]
    AuthorizationHeader(String),

    /// Failed to extract authorization request parts
    #[error("Failed to extract authorization request: {0}")]
    ExtractionFailed(String),

    /// Authorization check failed
    #[error("Authorization check failed: {0}")]
    AuthorizationFailed(String),

    /// Access denied by policy
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Internal error
    #[error("Internal middleware error: {0}")]
    Internal(String),
}

impl MiddlewareError {
    /// Convert to HTTP status code
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            MiddlewareError::AuthorizationHeader(_) => http::StatusCode::UNAUTHORIZED,
            MiddlewareError::ExtractionFailed(_) => http::StatusCode::BAD_REQUEST,
            MiddlewareError::AuthorizationFailed(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            MiddlewareError::AccessDenied(_) => http::StatusCode::FORBIDDEN,
            MiddlewareError::Internal(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convert to HTTP response body
    pub fn to_response_body(&self) -> String {
        serde_json::json!({
            "error": self.to_string(),
            "status": self.status_code().as_u16(),
        })
        .to_string()
    }
}
