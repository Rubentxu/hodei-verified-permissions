//! Entity identifier for Cedar entities

use std::fmt;
use serde::{Deserialize, Serialize};

/// Entity identifier for Cedar (type + id)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityIdentifier {
    /// Entity type (e.g., "User", "Document", "Action")
    pub entity_type: String,
    /// Entity identifier
    pub id: String,
}

impl EntityIdentifier {
    /// Create a new entity identifier
    pub fn new(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }
    
    /// Create from Cedar string format (e.g., "User::\"alice\"")
    pub fn from_cedar_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split("::").collect();
        if parts.len() != 2 {
            return Err(format!("Invalid Cedar format: {}", s));
        }
        
        let entity_type = parts[0].to_string();
        let id = parts[1]
            .trim_matches('"')
            .to_string();
            
        Ok(Self { entity_type, id })
    }
    
    /// Convert to Cedar string format
    pub fn to_cedar_string(&self) -> String {
        format!("{}::\"{}\"", self.entity_type, self.id)
    }
    
    /// Check if this is a wildcard identifier
    pub fn is_wildcard(&self) -> bool {
        self.id == "*"
    }
    
    /// Create a wildcard identifier for a type
    pub fn wildcard(entity_type: impl Into<String>) -> Self {
        Self {
            entity_type: entity_type.into(),
            id: "*".to_string(),
        }
    }
}

impl fmt::Display for EntityIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::\"{}\"", self.entity_type, self.id)
    }
}

impl From<&str> for EntityIdentifier {
    fn from(s: &str) -> Self {
        EntityIdentifier::from_cedar_string(s)
            .unwrap_or_else(|_| EntityIdentifier::new("Unknown", s))
    }
}

impl From<String> for EntityIdentifier {
    fn from(s: String) -> Self {
        EntityIdentifier::from(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_identifier() {
        let id = EntityIdentifier::new("User", "alice");
        assert_eq!(id.entity_type, "User");
        assert_eq!(id.id, "alice");
        assert_eq!(id.to_cedar_string(), "User::\"alice\"");
    }
    
    #[test]
    fn test_from_cedar_string() {
        let id = EntityIdentifier::from_cedar_string("Document::\"doc123\"").unwrap();
        assert_eq!(id.entity_type, "Document");
        assert_eq!(id.id, "doc123");
    }
    
    #[test]
    fn test_invalid_cedar_format() {
        let result = EntityIdentifier::from_cedar_string("invalid");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_wildcard() {
        let wildcard = EntityIdentifier::wildcard("Action");
        assert_eq!(wildcard.entity_type, "Action");
        assert_eq!(wildcard.id, "*");
        assert!(wildcard.is_wildcard());
    }
    
    #[test]
    fn test_display() {
        let id = EntityIdentifier::new("Resource", "test");
        assert_eq!(format!("{}", id), "Resource::\"test\"");
    }
}
