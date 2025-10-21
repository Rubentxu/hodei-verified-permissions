//! Cedar entity system for Hodei authorization

pub mod identifier;
pub mod builder;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub use identifier::EntityIdentifier;
pub use builder::CedarEntityBuilder;

/// Cedar entity with attributes and hierarchy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CedarEntity {
    /// Entity identifier (type and id)
    pub uid: EntityIdentifier,
    /// Entity attributes
    pub attrs: HashMap<String, serde_json::Value>,
    /// Parent entities (for hierarchy)
    pub parents: Vec<EntityIdentifier>,
}

impl CedarEntity {
    /// Create a new entity builder
    pub fn builder(entity_type: impl Into<String>, id: impl Into<String>) -> CedarEntityBuilder {
        CedarEntityBuilder::new(entity_type, id)
    }
    
    /// Convert to Cedar string format
    pub fn to_cedar_string(&self) -> String {
        self.uid.to_cedar_string()
    }
    
    /// Check if this entity has a specific parent
    pub fn has_parent(&self, parent: &EntityIdentifier) -> bool {
        self.parents.contains(parent)
    }
    
    /// Add a parent entity
    pub fn add_parent(&mut self, parent: EntityIdentifier) {
        if !self.parents.contains(&parent) {
            self.parents.push(parent);
        }
    }
    
    /// Get attribute value
    pub fn get_attr<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.attrs.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Set attribute value
    pub fn set_attr<T: serde::Serialize>(&mut self, key: impl Into<String>, value: T) {
        self.attrs.insert(key.into(), serde_json::to_value(value).unwrap_or_default());
    }
}

impl std::fmt::Display for CedarEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CedarEntity {{ uid: {}, attrs: {}, parents: {} }}", 
               self.uid, 
               serde_json::to_string(&self.attrs).unwrap_or_default(),
               self.parents.len())
    }
}

/// Helper functions for common entity types
pub mod helpers {
    use super::*;
    
    /// Create a user entity
    pub fn user_entity(id: impl Into<String>) -> CedarEntityBuilder {
        CedarEntity::builder("User", id)
    }
    
    /// Create a resource entity
    pub fn resource_entity(resource_type: impl Into<String>, id: impl Into<String>) -> CedarEntityBuilder {
        CedarEntity::builder(resource_type, id)
    }
    
    /// Create a group entity
    pub fn group_entity(id: impl Into<String>) -> CedarEntityBuilder {
        CedarEntity::builder("UserGroup", id)
    }
    
    /// Create an action entity
    pub fn action_entity(id: impl Into<String>) -> CedarEntityBuilder {
        CedarEntity::builder("Action", id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_builder() {
        let entity = CedarEntity::builder("User", "alice")
            .attribute("email", "alice@example.com")
            .attribute("role", "admin")
            .parent("UserGroup", "admins")
            .build();
            
        assert_eq!(entity.uid.entity_type, "User");
        assert_eq!(entity.uid.id, "alice");
        assert_eq!(entity.attrs["email"], "alice@example.com");
        assert_eq!(entity.parents.len(), 1);
    }
    
    #[test]
    fn test_entity_helpers() {
        let user = helpers::user_entity("bob")
            .attribute("email", "bob@example.com")
            .build();
            
        assert_eq!(user.uid.entity_type, "User");
        assert_eq!(user.uid.id, "bob");
    }
    
    #[test]
    fn test_entity_display() {
        let entity = CedarEntity::builder("Document", "doc123")
            .attribute("title", "Important Doc")
            .build();
            
        let display = format!("{}", entity);
        assert!(display.contains("Document::\"doc123\""));
    }
}
