//! Error types for infrastructure layer

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AuthorizationError>;

#[derive(Error, Debug)]
pub enum AuthorizationError {
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

// Note: Conversion to tonic::Status is done in the API layer
