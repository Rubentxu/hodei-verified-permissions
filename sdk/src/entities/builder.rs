//! Builder pattern for Cedar entities

use std::collections::HashMap;
use super::{CedarEntity, EntityIdentifier};

/// Builder for Cedar entities with fluent API
pub struct CedarEntityBuilder {
    uid: EntityIdentifier,
    attrs: HashMap<String, serde_json::Value>,
    parents: Vec<EntityIdentifier>,
}

impl CedarEntityBuilder {
    /// Create a new entity builder
    pub fn new(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            uid: EntityIdentifier::new(entity_type, id),
            attrs: HashMap::new(),
            parents: Vec::new(),
        }
    }
    
    /// Add a single attribute
    pub fn attribute<T: serde::Serialize>(mut self, key: impl Into<String>, value: T) -> Self {
        self.attrs.insert(key.into(), serde_json::to_value(value).unwrap_or_default());
        self
    }
    
    /// Add multiple attributes at once
    pub fn attributes<T: serde::Serialize>(mut self, attrs: impl IntoIterator<Item = (impl Into<String>, T)>) -> Self {
        for (key, value) in attrs {
            self.attrs.insert(key.into(), serde_json::to_value(value).unwrap_or_default());
        }
        self
    }
    
    /// Add a single parent entity
    pub fn parent(mut self, entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        self.parents.push(EntityIdentifier::new(entity_type, id));
        self
    }
    
    /// Add multiple parent entities
    pub fn parents(mut self, parents: impl IntoIterator<Item = EntityIdentifier>) -> Self {
        self.parents.extend(parents);
        self
    }
    
    /// Add parent entities from strings
    pub fn parent_strings(mut self, parents: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        for (entity_type, id) in parents {
            self.parents.push(EntityIdentifier::new(entity_type, id));
        }
        self
    }
    
    /// Set the entity ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.uid = EntityIdentifier::new(self.uid.entity_type.clone(), id);
        self
    }
    
    /// Set the entity type
    pub fn entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.uid = EntityIdentifier::new(entity_type, self.uid.id.clone());
        self
    }
    
    /// Build the final entity
    pub fn build(self) -> CedarEntity {
        CedarEntity {
            uid: self.uid,
            attrs: self.attrs,
            parents: self.parents,
        }
    }
    
    /// Build and validate the entity
    pub fn build_validated(self) -> Result<CedarEntity, String> {
        let entity = self.build();
        
        // Basic validation
        if entity.uid.entity_type.is_empty() {
            return Err("Entity type cannot be empty".to_string());
        }
        
        if entity.uid.id.is_empty() {
            return Err("Entity ID cannot be empty".to_string());
        }
        
        Ok(entity)
    }
}

impl Default for CedarEntityBuilder {
    fn default() -> Self {
        Self::new("Unknown", "unknown")
    }
}

/// Convenience methods for common entity types
impl CedarEntityBuilder {
    /// Create a user entity builder
    pub fn user(id: impl Into<String>) -> Self {
        Self::new("User", id)
    }
    
    /// Create a document entity builder
    pub fn document(id: impl Into<String>) -> Self {
        Self::new("Document", id)
    }
    
    /// Create an action entity builder
    pub fn action(id: impl Into<String>) -> Self {
        Self::new("Action", id)
    }
    
    /// Create a resource entity builder
    pub fn resource(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::new(resource_type, id)
    }
    
    /// Create a group entity builder
    pub fn group(id: impl Into<String>) -> Self {
        Self::new("UserGroup", id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_builder() {
        let entity = CedarEntityBuilder::new("User", "alice")
            .attribute("email", "alice@example.com")
            .attribute("role", "admin")
            .parent("UserGroup", "admins")
            .build();
            
        assert_eq!(entity.uid.entity_type, "User");
        assert_eq!(entity.uid.id, "alice");
        assert_eq!(entity.attrs.len(), 2);
        assert_eq!(entity.parents.len(), 1);
    }
    
    #[test]
    fn test_convenience_builders() {
        let user = CedarEntityBuilder::user("bob")
            .attribute("email", "bob@example.com")
            .build();
            
        assert_eq!(user.uid.entity_type, "User");
        assert_eq!(user.uid.id, "bob");
        
        let doc = CedarEntityBuilder::document("doc123")
            .attribute("title", "Important Document")
            .build();
            
        assert_eq!(doc.uid.entity_type, "Document");
        assert_eq!(doc.uid.id, "doc123");
    }
    
    #[test]
    fn test_multiple_attributes() {
        let entity = CedarEntityBuilder::new("User", "charlie")
            .attributes([
                ("email", "charlie@example.com"),
                ("role", "user"),
                ("department", "engineering"),
            ])
            .build();
            
        assert_eq!(entity.attrs.len(), 3);
        assert_eq!(entity.attrs["email"], "charlie@example.com");
    }
    
    #[test]
    fn test_multiple_parents() {
        let entity = CedarEntityBuilder::new("User", "dave")
            .parents([
                EntityIdentifier::new("UserGroup", "admins"),
                EntityIdentifier::new("UserGroup", "developers"),
            ])
            .build();
            
        assert_eq!(entity.parents.len(), 2);
    }
    
    #[test]
    fn test_validation() {
        let result = CedarEntityBuilder::new("", "valid")
            .build_validated();
        assert!(result.is_err());
        
        let result = CedarEntityBuilder::new("User", "")
            .build_validated();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_chaining() {
        let entity = CedarEntityBuilder::user("eve")
            .id("eve.smith")
            .attribute("email", "eve@example.com")
            .parent("UserGroup", "users")
            .build();
            
        assert_eq!(entity.uid.id, "eve.smith");
    }
}
