//! Domain errors

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("Invalid policy store ID: {0}")]
    InvalidPolicyStoreId(String),

    #[error("Invalid policy ID: {0}")]
    InvalidPolicyId(String),

    #[error("Invalid entity identifier: {0}")]
    InvalidEntityIdentifier(String),

    #[error("Invalid policy syntax: {0}")]
    InvalidPolicySyntax(String),

    #[error("Invalid schema format: {0}")]
    InvalidSchemaFormat(String),

    #[error("Policy validation failed: {0}")]
    PolicyValidationFailed(String),

    #[error("Policy store not found: {0}")]
    PolicyStoreNotFound(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Authorization evaluation failed: {0}")]
    AuthorizationEvaluationFailed(String),

    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
