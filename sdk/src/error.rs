//! SDK error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("gRPC transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    StatusError(#[from] tonic::Status),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

pub type Result<T> = std::result::Result<T, SdkError>;
