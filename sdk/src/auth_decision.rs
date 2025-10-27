//! Authorization decision types and utilities

use serde::{Deserialize, Serialize};
use std::fmt;

/// Authorization decision result
///
/// Represents the outcome of an authorization check.
/// This type is used in both `IsAuthorizedResponse` and `IsAuthorizedWithTokenResponse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationDecision {
    /// Authorization was granted
    Allow,
    /// Authorization was denied
    Deny,
}

impl AuthorizationDecision {
    /// Check if the decision is Allow
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Check if the decision is Deny
    pub fn is_deny(&self) -> bool {
        matches!(self, Self::Deny)
    }
}

impl fmt::Display for AuthorizationDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "ALLOW"),
            Self::Deny => write!(f, "DENY"),
        }
    }
}

impl From<i32> for AuthorizationDecision {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Allow,
            1 => Self::Deny,
            _ => Self::Deny,
        }
    }
}

impl From<AuthorizationDecision> for i32 {
    fn from(decision: AuthorizationDecision) -> Self {
        match decision {
            AuthorizationDecision::Allow => 0,
            AuthorizationDecision::Deny => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_decision_display() {
        assert_eq!(AuthorizationDecision::Allow.to_string(), "ALLOW");
        assert_eq!(AuthorizationDecision::Deny.to_string(), "DENY");
    }

    #[test]
    fn test_authorization_decision_checks() {
        assert!(AuthorizationDecision::Allow.is_allow());
        assert!(!AuthorizationDecision::Allow.is_deny());
        assert!(!AuthorizationDecision::Deny.is_allow());
        assert!(AuthorizationDecision::Deny.is_deny());
    }

    #[test]
    fn test_authorization_decision_conversions() {
        assert_eq!(AuthorizationDecision::from(0), AuthorizationDecision::Allow);
        assert_eq!(AuthorizationDecision::from(1), AuthorizationDecision::Deny);
        assert_eq!(i32::from(AuthorizationDecision::Allow), 0);
        assert_eq!(i32::from(AuthorizationDecision::Deny), 1);
    }
}
