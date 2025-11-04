//! Authorization engine system

pub mod engine;

pub use engine::{AuthorizationEngine, AuthorizationRequest, AuthorizationResult};

/// Common authorization types
pub mod types {
    use super::*;
    use crate::entities::CedarEntity;
    
    /// Authorization decision
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Decision {
        Allow,
        Deny,
    }
    
    impl From<bool> for Decision {
        fn from(allowed: bool) -> Self {
            if allowed { Decision::Allow } else { Decision::Deny }
        }
    }
    
    impl fmt::Display for Decision {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Decision::Allow => write!(f, "Allow"),
                Decision::Deny => write!(f, "Deny"),
            }
        }
    }
    
    /// Authorization context
    #[derive(Debug, Clone, Default)]
    pub struct AuthorizationContext {
        pub entities: Vec<CedarEntity>,
        pub context: Option<serde_json::Value>,
    }
    
    impl AuthorizationContext {
        pub fn new() -> Self {
            Self::default()
        }
        
        pub fn with_entity(mut self, entity: CedarEntity) -> Self {
            self.entities.push(entity);
            self
        }
        
        pub fn with_context(mut self, context: serde_json::Value) -> Self {
            self.context = Some(context);
            self
        }
    }
}

use std::fmt;
pub use types::*;

/// Error types for authorization
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationError {
    #[error("Policy evaluation failed: {0}")]
    EvaluationFailed(String),
    
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
    
    #[error("Invalid entity: {0}")]
    InvalidEntity(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}
