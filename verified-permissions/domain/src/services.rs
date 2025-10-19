//! Domain services - Pure business logic

use cedar_policy::{Authorizer, Context, Entities, PolicySet, Request};
use serde_json::Value as JsonValue;

use crate::entities::*;
use crate::value_objects::*;
use crate::errors::{DomainError, DomainResult};

/// Authorization evaluator service
pub struct AuthorizationEvaluator {
    authorizer: Authorizer,
}

impl AuthorizationEvaluator {
    pub fn new() -> Self {
        Self {
            authorizer: Authorizer::new(),
        }
    }

    /// Evaluates an authorization request
    pub fn evaluate(
        &self,
        policies: &[Policy],
        principal: &Principal,
        action: &Action,
        resource: &Resource,
        context: Option<JsonValue>,
        entities: Option<JsonValue>,
    ) -> DomainResult<AuthorizationDecision> {
        // Convert policies to Cedar PolicySet
        let policy_set = self.build_policy_set(policies)?;
        
        // Build Cedar request
        let request = self.build_request(principal, action, resource, context)?;
        
        // Build entities
        let entities = self.build_entities(entities)?;
        
        // Evaluate
        let response = self.authorizer.is_authorized(&request, &policy_set, &entities);
        
        Ok(match response.decision() {
            cedar_policy::Decision::Allow => AuthorizationDecision::Allow,
            cedar_policy::Decision::Deny => AuthorizationDecision::Deny,
        })
    }

    fn build_policy_set(&self, policies: &[Policy]) -> DomainResult<PolicySet> {
        use std::str::FromStr;
        
        let mut policy_set = PolicySet::new();
        
        for policy in policies {
            let policy_str = policy.statement.as_str();
            let cedar_policy = cedar_policy::Policy::from_str(policy_str)
                .map_err(|e| DomainError::InvalidPolicySyntax(e.to_string()))?;
            
            policy_set.add(cedar_policy)
                .map_err(|e| DomainError::PolicyValidationFailed(e.to_string()))?;
        }
        
        Ok(policy_set)
    }

    fn build_request(
        &self,
        principal: &Principal,
        action: &Action,
        resource: &Resource,
        context: Option<JsonValue>,
    ) -> DomainResult<Request> {
        let principal_euid = principal.as_str().parse()
            .map_err(|e| DomainError::InvalidEntityIdentifier(format!("Invalid principal: {}", e)))?;
        
        let action_euid = action.as_str().parse()
            .map_err(|e| DomainError::InvalidEntityIdentifier(format!("Invalid action: {}", e)))?;
        
        let resource_euid = resource.as_str().parse()
            .map_err(|e| DomainError::InvalidEntityIdentifier(format!("Invalid resource: {}", e)))?;
        
        let context = if let Some(ctx) = context {
            Context::from_json_value(ctx, None)
                .map_err(|e| DomainError::AuthorizationEvaluationFailed(format!("Invalid context: {}", e)))?
        } else {
            Context::empty()
        };
        
        Ok(Request::new(principal_euid, action_euid, resource_euid, context, None)
            .map_err(|e| DomainError::AuthorizationEvaluationFailed(e.to_string()))?)
    }

    fn build_entities(&self, entities: Option<JsonValue>) -> DomainResult<Entities> {
        if let Some(ents) = entities {
            Entities::from_json_value(ents, None)
                .map_err(|e| DomainError::AuthorizationEvaluationFailed(format!("Invalid entities: {}", e)))
        } else {
            Ok(Entities::empty())
        }
    }
}

impl Default for AuthorizationEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy validator service
pub struct PolicyValidator;

impl PolicyValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validates a Cedar policy syntax
    pub fn validate_policy(&self, statement: &str) -> DomainResult<()> {
        cedar_policy::Policy::parse(None, statement)
            .map_err(|e| DomainError::InvalidPolicySyntax(e.to_string()))?;
        Ok(())
    }

    /// Validates a Cedar schema
    pub fn validate_schema(&self, schema_json: &str) -> DomainResult<()> {
        // Parse JSON to validate format
        serde_json::from_str::<JsonValue>(schema_json)
            .map_err(|e| DomainError::InvalidSchemaFormat(e.to_string()))?;
        
        // TODO: Add Cedar schema validation
        Ok(())
    }
}

impl Default for PolicyValidator {
    fn default() -> Self {
        Self::new()
    }
}
