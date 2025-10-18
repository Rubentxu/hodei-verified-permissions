//! Error types for the authorization service

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthorizationError {
    #[error("Policy store not found: {0}")]
    PolicyStoreNotFound(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    #[error("Schema not found for policy store: {0}")]
    SchemaNotFound(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid policy: {0}")]
    InvalidPolicy(String),

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Policy validation failed: {0}")]
    ValidationFailed(String),

    #[error("Cedar evaluation error: {0}")]
    EvaluationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AuthorizationError>;

impl From<AuthorizationError> for tonic::Status {
    fn from(err: AuthorizationError) -> Self {
        match err {
            AuthorizationError::PolicyStoreNotFound(_) => {
                tonic::Status::not_found(err.to_string())
            }
            AuthorizationError::PolicyNotFound(_) => {
                tonic::Status::not_found(err.to_string())
            }
            AuthorizationError::SchemaNotFound(_) => {
                tonic::Status::not_found(err.to_string())
            }
            AuthorizationError::NotFound(_) => {
                tonic::Status::not_found(err.to_string())
            }
            AuthorizationError::InvalidPolicy(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            AuthorizationError::InvalidSchema(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            AuthorizationError::ValidationFailed(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            AuthorizationError::EvaluationError(_) => {
                tonic::Status::internal(err.to_string())
            }
            AuthorizationError::DatabaseError(_) => {
                tonic::Status::internal(err.to_string())
            }
            AuthorizationError::SerializationError(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            AuthorizationError::Internal(_) => {
                tonic::Status::internal(err.to_string())
            }
        }
    }
}
