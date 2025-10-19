//! Authorization use case

use async_trait::async_trait;
use tracing::{info, warn};

use hodei_domain::{
    PolicyRepository, AuthorizationEvaluator, Principal, Action, Resource,
    PolicyStoreId, AuthorizationDecision,
};
use crate::dto::{AuthorizationRequest, AuthorizationResponse};
use crate::errors::{ApplicationError, ApplicationResult};

/// Authorization use case
pub struct AuthorizeUseCase<R: PolicyRepository> {
    repository: R,
    evaluator: AuthorizationEvaluator,
}

impl<R: PolicyRepository> AuthorizeUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            evaluator: AuthorizationEvaluator::new(),
        }
    }

    pub async fn execute(&self, request: AuthorizationRequest) -> ApplicationResult<AuthorizationResponse> {
        info!("Evaluating authorization request for policy store: {}", request.policy_store_id);

        // Parse policy store ID
        let policy_store_id = PolicyStoreId::new(request.policy_store_id.clone())
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        // Get all policies for the policy store
        let policies = self.repository.list_policies(&policy_store_id).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        if policies.is_empty() {
            warn!("No policies found for policy store: {}", request.policy_store_id);
        }

        // Parse principal, action, resource
        let principal = Principal::new(request.principal)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        let action = Action::new(request.action)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        let resource = Resource::new(request.resource)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        // Evaluate authorization
        let decision = self.evaluator.evaluate(
            &policies,
            &principal,
            &action,
            &resource,
            request.context,
            request.entities,
        )?;

        let decision_str = match decision {
            AuthorizationDecision::Allow => "ALLOW",
            AuthorizationDecision::Deny => "DENY",
        };

        info!("Authorization decision: {}", decision_str);

        Ok(AuthorizationResponse {
            decision: decision_str.to_string(),
            determining_policies: vec![], // TODO: Extract from Cedar response
            errors: vec![],
        })
    }
}
