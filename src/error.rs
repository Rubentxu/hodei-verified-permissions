//! Error types for the authorization service

use thiserror::Error;

/// Unified database error that abstracts over different database backends
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Internal database error: {0}")]
    Internal(String),
}

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
    Database(#[from] DatabaseError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Cedar parse error: {0}")]
    CedarParseError(String),
    
    #[error("Cedar policy set error: {0}")]
    CedarPolicySetError(String),
    
    #[error("Cedar schema error: {0}")]
    CedarSchemaError(String),
}

pub type Result<T> = std::result::Result<T, AuthorizationError>;

// Cedar error conversions
impl From<cedar_policy::ParseErrors> for AuthorizationError {
    fn from(err: cedar_policy::ParseErrors) -> Self {
        AuthorizationError::CedarParseError(format!("{:?}", err))
    }
}

impl From<cedar_policy::PolicySetError> for AuthorizationError {
    fn from(err: cedar_policy::PolicySetError) -> Self {
        AuthorizationError::CedarPolicySetError(err.to_string())
    }
}

impl From<cedar_policy::SchemaError> for AuthorizationError {
    fn from(err: cedar_policy::SchemaError) -> Self {
        AuthorizationError::CedarSchemaError(err.to_string())
    }
}

impl From<cedar_policy::ContextJsonError> for AuthorizationError {
    fn from(err: cedar_policy::ContextJsonError) -> Self {
        AuthorizationError::CedarParseError(format!("Context error: {:?}", err))
    }
}

impl From<cedar_policy::RequestValidationError> for AuthorizationError {
    fn from(err: cedar_policy::RequestValidationError) -> Self {
        AuthorizationError::ValidationFailed(err.to_string())
    }
}

// SQLx error conversions
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound("Row not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    DatabaseError::ConstraintViolation(db_err.to_string())
                } else {
                    DatabaseError::Query(db_err.to_string())
                }
            }
            sqlx::Error::PoolTimedOut => DatabaseError::Connection("Pool timeout".to_string()),
            sqlx::Error::PoolClosed => DatabaseError::Connection("Pool closed".to_string()),
            _ => DatabaseError::Internal(err.to_string()),
        }
    }
}

impl From<sqlx::Error> for AuthorizationError {
    fn from(err: sqlx::Error) -> Self {
        AuthorizationError::Database(DatabaseError::from(err))
    }
}

// SurrealDB error conversions
#[cfg(feature = "surreal")]
impl From<surrealdb::Error> for DatabaseError {
    fn from(err: surrealdb::Error) -> Self {
        // Map SurrealDB errors to our unified DatabaseError
        let err_str = err.to_string();
        if err_str.contains("not found") || err_str.contains("no record") {
            DatabaseError::NotFound(err_str)
        } else if err_str.contains("connection") {
            DatabaseError::Connection(err_str)
        } else if err_str.contains("unique") || err_str.contains("duplicate") {
            DatabaseError::ConstraintViolation(err_str)
        } else {
            DatabaseError::Internal(err_str)
        }
    }
}

#[cfg(feature = "surreal")]
impl From<surrealdb::Error> for AuthorizationError {
    fn from(err: surrealdb::Error) -> Self {
        AuthorizationError::Database(DatabaseError::from(err))
    }
}

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
            AuthorizationError::Database(_) => {
                tonic::Status::internal(err.to_string())
            }
            AuthorizationError::SerializationError(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            AuthorizationError::Internal(_) => {
                tonic::Status::internal(err.to_string())
            }
            AuthorizationError::CedarParseError(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            AuthorizationError::CedarPolicySetError(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
            AuthorizationError::CedarSchemaError(_) => {
                tonic::Status::invalid_argument(err.to_string())
            }
        }
    }
}
