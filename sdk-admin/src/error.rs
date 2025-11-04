//! Error types for SDK Admin library

use thiserror::Error;
use tonic::Status;

/// Result type for SDK Admin operations
pub type Result<T> = std::result::Result<T, SdkAdminError>;

/// Error type for SDK Admin operations
#[derive(Error, Debug)]
pub enum SdkAdminError {
    /// Failed to connect to the service
    #[error("Failed to connect to service")]
    ConnectionFailed,

    /// gRPC status error
    #[error("gRPC error: {0}")]
    Status(#[from] Status),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<String> for SdkAdminError {
    fn from(msg: String) -> Self {
        SdkAdminError::Other(msg)
    }
}
