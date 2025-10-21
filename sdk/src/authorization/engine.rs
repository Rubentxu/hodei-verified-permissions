//! Authorization engine trait and request/response types

use async_trait::async_trait;
use std::fmt;
use crate::entities::{CedarEntity, EntityIdentifier};

/// Authorization request
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    /// Principal entity making the request
    pub principal: CedarEntity,
    /// Action being performed
    pub action: String,
    /// Resource being accessed
    pub resource: CedarEntity,
    /// Additional context for authorization
    pub context: Option<serde_json::Value>,
    /// Additional entities for evaluation
    pub entities: Vec<CedarEntity>,
}

impl AuthorizationRequest {
    /// Create a new authorization request
    pub fn new(
        principal: CedarEntity,
        action: impl Into<String>,
        resource: CedarEntity,
    ) -> Self {
        Self {
            principal,
            action: action.into(),
            resource,
            context: None,
            entities: Vec::new(),
        }
    }
    
    /// Add context to the request
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Add additional entities
    pub fn with_entities(mut self, entities: Vec<CedarEntity>) -> Self {
        self.entities = entities;
        self
    }
    
    /// Add a single entity
    pub fn add_entity(mut self, entity: CedarEntity) -> Self {
        self.entities.push(entity);
        self
    }
    
    /// Convert to Cedar string format
    pub fn to_cedar_strings(&self) -> (String, String, String) {
        (
            self.principal.uid.to_cedar_string(),
            self.action.clone(),
            self.resource.uid.to_cedar_string(),
        )
    }
}

/// Authorization result
#[derive(Debug, Clone)]
pub enum AuthorizationResult {
    /// Access granted
    Allow {
        /// Policies that determined the result
        determining_policies: Vec<String>,
        /// Principal UID
        principal_uid: EntityIdentifier,
    },
    /// Access denied
    Deny {
        /// Optional reason for denial
        reason: Option<String>,
    },
    /// Authorization error
    Error {
        /// Error message
        message: String,
    },
}

impl AuthorizationResult {
    /// Check if access is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, AuthorizationResult::Allow { .. })
    }
    
    /// Check if access is denied
    pub fn is_denied(&self) -> bool {
        matches!(self, AuthorizationResult::Deny { .. })
    }
    
    /// Check if there was an error
    pub fn is_error(&self) -> bool {
        matches!(self, AuthorizationResult::Error { .. })
    }
    
    /// Get the decision as a boolean
    pub fn decision(&self) -> bool {
        self.is_allowed()
    }
    
    /// Get determining policies (if any)
    pub fn determining_policies(&self) -> Option<&[String]> {
        match self {
            AuthorizationResult::Allow { determining_policies, .. } => Some(determining_policies),
            _ => None,
        }
    }
}

impl fmt::Display for AuthorizationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthorizationResult::Allow { determining_policies, .. } => {
                write!(f, "Allow (policies: {:?})", determining_policies)
            }
            AuthorizationResult::Deny { reason } => {
                if let Some(reason) = reason {
                    write!(f, "Deny: {}", reason)
                } else {
                    write!(f, "Deny")
                }
            }
            AuthorizationResult::Error { message } => {
                write!(f, "Error: {}", message)
            }
        }
    }
}

/// Authorization engine trait
#[async_trait]
pub trait AuthorizationEngine: Send + Sync {
    /// Authorize a request
    async fn authorize(&self, request: AuthorizationRequest) -> Result<AuthorizationResult, Box<dyn std::error::Error>>;
    
    /// Authorize with token (for token-based auth)
    async fn authorize_with_token(
        &self,
        token: &str,
        action: &str,
        resource: &str,
        context: Option<&serde_json::Value>,
    ) -> Result<AuthorizationResult, Box<dyn std::error::Error>>;
    
    /// Batch authorize multiple requests
    async fn batch_authorize(
        &self,
        requests: Vec<AuthorizationRequest>,
    ) -> Result<Vec<AuthorizationResult>, Box<dyn std::error::Error>>;
}

/// Helper functions for creating authorization requests
pub mod request_helpers {
    use super::*;
    use crate::entities::CedarEntity;
    
    /// Create a simple authorization request
    pub fn simple_request(
        principal_type: &str,
        principal_id: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
    ) -> AuthorizationRequest {
        let principal = CedarEntity::builder(principal_type, principal_id).build();
        let resource = CedarEntity::builder(resource_type, resource_id).build();
        
        AuthorizationRequest::new(principal, action, resource)
    }
    
    /// Create a request with context
    pub fn request_with_context(
        principal: CedarEntity,
        action: &str,
        resource: CedarEntity,
        context: serde_json::Value,
    ) -> AuthorizationRequest {
        AuthorizationRequest::new(principal, action, resource)
            .with_context(context)
    }
    
    /// Create a request with additional entities
    pub fn request_with_entities(
        principal: CedarEntity,
        action: &str,
        resource: CedarEntity,
        entities: Vec<CedarEntity>,
    ) -> AuthorizationRequest {
        AuthorizationRequest::new(principal, action, resource)
            .with_entities(entities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::CedarEntity;
    
    #[test]
    fn test_authorization_request() {
        let principal = CedarEntity::builder("User", "alice").build();
        let resource = CedarEntity::builder("Document", "doc123").build();
        
        let request = AuthorizationRequest::new(principal, "view", resource);
        
        assert_eq!(request.action, "view");
        assert_eq!(request.context, None);
        assert!(request.entities.is_empty());
    }
    
    #[test]
    fn test_authorization_request_with_context() {
        let principal = CedarEntity::builder("User", "alice").build();
        let resource = CedarEntity::builder("Document", "doc123").build();
        let context = serde_json::json!({"ip": "192.168.1.1"});
        
        let request = AuthorizationRequest::new(principal, "view", resource)
            .with_context(context.clone());
            
        assert_eq!(request.context, Some(context));
    }
    
    #[test]
    fn test_authorization_result() {
        let result = AuthorizationResult::Allow {
            determining_policies: vec!["policy1".to_string()],
            principal_uid: EntityIdentifier::new("User", "alice"),
        };
        
        assert!(result.is_allowed());
        assert!(!result.is_denied());
        assert!(!result.is_error());
        assert_eq!(result.decision(), true);
    }
    
    #[test]
    fn test_authorization_result_deny() {
        let result = AuthorizationResult::Deny { reason: Some("Not authorized".to_string()) };
        
        assert!(!result.is_allowed());
        assert!(result.is_denied());
        assert!(!result.is_error());
        assert_eq!(result.decision(), false);
    }
    
    #[test]
    fn test_request_helpers() {
        let request = request_helpers::simple_request(
            "User", "alice", "view", "Document", "doc123"
        );
        
        assert_eq!(request.principal.uid.entity_type, "User");
        assert_eq!(request.principal.uid.id, "alice");
        assert_eq!(request.resource.uid.entity_type, "Document");
        assert_eq!(request.resource.uid.id, "doc123");
        assert_eq!(request.action, "view");
    }
    
    #[test]
    fn test_display() {
        let result = AuthorizationResult::Allow {
            determining_policies: vec!["policy1".to_string()],
            principal_uid: EntityIdentifier::new("User", "alice"),
        };
        
        let display = format!("{}", result);
        assert!(display.contains("Allow"));
        assert!(display.contains("policy1"));
    }
}
