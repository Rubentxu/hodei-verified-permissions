//! Builder patterns for requests

use crate::proto::*;
use std::collections::HashMap;

/// Builder for IsAuthorizedRequest
pub struct IsAuthorizedRequestBuilder {
    policy_store_id: String,
    principal: Option<EntityIdentifier>,
    action: Option<EntityIdentifier>,
    resource: Option<EntityIdentifier>,
    context: Option<String>,
    entities: Vec<Entity>,
}

impl IsAuthorizedRequestBuilder {
    pub fn new(policy_store_id: impl Into<String>) -> Self {
        Self {
            policy_store_id: policy_store_id.into(),
            principal: None,
            action: None,
            resource: None,
            context: None,
            entities: Vec::new(),
        }
    }

    pub fn principal(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.principal = Some(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    pub fn action(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.action = Some(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    pub fn resource(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.resource = Some(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    pub fn context(mut self, context_json: impl Into<String>) -> Self {
        self.context = Some(context_json.into());
        self
    }

    pub fn add_entity(mut self, entity: Entity) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn build(self) -> IsAuthorizedRequest {
        IsAuthorizedRequest {
            policy_store_id: self.policy_store_id,
            principal: self.principal,
            action: self.action,
            resource: self.resource,
            context: self.context,
            entities: self.entities,
        }
    }
}

/// Builder for Entity
pub struct EntityBuilder {
    identifier: Option<EntityIdentifier>,
    attributes: HashMap<String, String>,
    parents: Vec<EntityIdentifier>,
}

impl EntityBuilder {
    pub fn new(entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self {
            identifier: Some(EntityIdentifier {
                entity_type: entity_type.into(),
                entity_id: entity_id.into(),
            }),
            attributes: HashMap::new(),
            parents: Vec::new(),
        }
    }

    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    pub fn parent(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.parents.push(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    pub fn build(self) -> Entity {
        Entity {
            identifier: self.identifier,
            attributes: self.attributes,
            parents: self.parents,
        }
    }
}

/// Builder for IsAuthorizedWithTokenRequest
pub struct IsAuthorizedWithTokenRequestBuilder {
    policy_store_id: String,
    identity_source_id: String,
    access_token: String,
    action: Option<EntityIdentifier>,
    resource: Option<EntityIdentifier>,
    context: Option<String>,
    entities: Vec<Entity>,
}

impl IsAuthorizedWithTokenRequestBuilder {
    /// Create a new builder for IsAuthorizedWithTokenRequest
    ///
    /// # Arguments
    ///
    /// * `policy_store_id` - The policy store ID
    /// * `identity_source_id` - The identity source ID for JWT validation
    /// * `access_token` - The JWT access token
    pub fn new(
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            access_token: access_token.into(),
            action: None,
            resource: None,
            context: None,
            entities: Vec::new(),
        }
    }

    /// Set the action for the authorization request
    pub fn action(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.action = Some(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    /// Set the resource for the authorization request
    pub fn resource(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.resource = Some(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    /// Set the context for the authorization request
    pub fn context(mut self, context_json: impl Into<String>) -> Self {
        self.context = Some(context_json.into());
        self
    }

    /// Add an entity to the authorization request
    pub fn add_entity(mut self, entity: Entity) -> Self {
        self.entities.push(entity);
        self
    }

    /// Build the IsAuthorizedWithTokenRequest
    pub fn build(self) -> IsAuthorizedWithTokenRequest {
        IsAuthorizedWithTokenRequest {
            policy_store_id: self.policy_store_id,
            identity_source_id: self.identity_source_id,
            access_token: self.access_token,
            action: self.action,
            resource: self.resource,
            context: self.context,
            entities: self.entities,
        }
    }
}
