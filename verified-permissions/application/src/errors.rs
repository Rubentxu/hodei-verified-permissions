//! Application errors

use thiserror::Error;
use hodei_domain::DomainError;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Repository error: {0}")]
    Repository(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;
