//! Domain value objects

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::errors::{DomainError, DomainResult};

/// Status of a Policy Store
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyStoreStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
}

impl fmt::Display for PolicyStoreStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyStoreStatus::Active => write!(f, "active"),
            PolicyStoreStatus::Inactive => write!(f, "inactive"),
        }
    }
}

/// Policy Store identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyStoreId(String);

impl PolicyStoreId {
    pub fn new(id: String) -> DomainResult<Self> {
        if id.is_empty() {
            return Err(DomainError::InvalidPolicyStoreId(
                "Policy store ID cannot be empty".to_string(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for PolicyStoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for PolicyStoreId {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Policy identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(String);

impl PolicyId {
    pub fn new(id: String) -> DomainResult<Self> {
        if id.is_empty() {
            return Err(DomainError::InvalidPolicyId(
                "Policy ID cannot be empty".to_string(),
            ));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for PolicyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for PolicyId {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Cedar policy statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CedarPolicy(String);

impl CedarPolicy {
    pub fn new(statement: String) -> DomainResult<Self> {
        if statement.is_empty() {
            return Err(DomainError::InvalidPolicySyntax(
                "Policy statement cannot be empty".to_string(),
            ));
        }
        // TODO: Add Cedar syntax validation
        Ok(Self(statement))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for CedarPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for CedarPolicy {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Principal in authorization request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal(String);

impl Principal {
    pub fn new(principal: String) -> DomainResult<Self> {
        if principal.is_empty() {
            return Err(DomainError::InvalidEntityIdentifier(
                "Principal cannot be empty".to_string(),
            ));
        }
        Ok(Self(principal))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Principal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Action in authorization request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action(String);

impl Action {
    pub fn new(action: String) -> DomainResult<Self> {
        if action.is_empty() {
            return Err(DomainError::InvalidEntityIdentifier(
                "Action cannot be empty".to_string(),
            ));
        }
        Ok(Self(action))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Resource in authorization request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource(String);

impl Resource {
    pub fn new(resource: String) -> DomainResult<Self> {
        if resource.is_empty() {
            return Err(DomainError::InvalidEntityIdentifier(
                "Resource cannot be empty".to_string(),
            ));
        }
        Ok(Self(resource))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Authorization decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
}

impl fmt::Display for AuthorizationDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "ALLOW"),
            Self::Deny => write!(f, "DENY"),
        }
    }
}

/// Identity source type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentitySourceType {
    Cognito,
    Oidc,
}

impl fmt::Display for IdentitySourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cognito => write!(f, "cognito"),
            Self::Oidc => write!(f, "oidc"),
        }
    }
}

impl TryFrom<String> for IdentitySourceType {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "cognito" => Ok(Self::Cognito),
            "oidc" => Ok(Self::Oidc),
            _ => Err(DomainError::InvalidEntityIdentifier(format!(
                "Invalid identity source type: {}",
                value
            ))),
        }
    }
}
