//! gRPC service implementations

pub mod control_plane;
pub mod data_plane;

pub use control_plane::AuthorizationControlService;
pub use data_plane::AuthorizationDataService;

use hodei_domain::DomainError;
use tonic::Status;

/// Convert DomainError to gRPC Status
pub fn domain_error_to_status(error: DomainError) -> Status {
    match error {
        DomainError::PolicyStoreNotFound(msg) => Status::not_found(msg),
        DomainError::PolicyNotFound(msg) => Status::not_found(msg),
        DomainError::SchemaNotFound(msg) => Status::not_found(msg),
        DomainError::InvalidPolicyStoreId(msg) => Status::invalid_argument(msg),
        DomainError::InvalidPolicyId(msg) => Status::invalid_argument(msg),
        DomainError::InvalidEntityIdentifier(msg) => Status::invalid_argument(msg),
        DomainError::InvalidPolicySyntax(msg) => Status::invalid_argument(msg),
        DomainError::InvalidSchemaFormat(msg) => Status::invalid_argument(msg),
        DomainError::PolicyValidationFailed(msg) => Status::invalid_argument(msg),
        DomainError::AuthorizationEvaluationFailed(msg) => Status::internal(msg),
        DomainError::BusinessRuleViolation(msg) => Status::failed_precondition(msg),
        DomainError::Internal(msg) => Status::internal(msg),
    }
}
