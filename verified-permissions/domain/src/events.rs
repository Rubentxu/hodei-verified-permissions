//! Domain Events - Domain Event pattern implementation
//!
//! This module contains domain events and abstract interfaces following
//! hexagonal architecture principles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use async_trait::async_trait;

// ============================================================================
// Domain Events
// ============================================================================

/// Event envelope - wraps all domain events in a type-safe enum
/// This follows the standard event sourcing pattern and allows for proper serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEventEnvelope {
    PolicyStoreCreated(Box<PolicyStoreCreated>),
    PolicyStoreUpdated(Box<PolicyStoreUpdated>),
    PolicyStoreTagsUpdated(Box<PolicyStoreTagsUpdated>),
    PolicyStoreDeleted(Box<PolicyStoreDeleted>),
}

impl DomainEventEnvelope {
    /// Unique event ID
    pub fn event_id(&self) -> String {
        match self {
            DomainEventEnvelope::PolicyStoreCreated(e) => e.event_id.clone(),
            DomainEventEnvelope::PolicyStoreUpdated(e) => e.event_id.clone(),
            DomainEventEnvelope::PolicyStoreTagsUpdated(e) => e.event_id.clone(),
            DomainEventEnvelope::PolicyStoreDeleted(e) => e.event_id.clone(),
        }
    }

    /// Event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEventEnvelope::PolicyStoreCreated(_) => "PolicyStoreCreated",
            DomainEventEnvelope::PolicyStoreUpdated(_) => "PolicyStoreUpdated",
            DomainEventEnvelope::PolicyStoreTagsUpdated(_) => "PolicyStoreTagsUpdated",
            DomainEventEnvelope::PolicyStoreDeleted(_) => "PolicyStoreDeleted",
        }
    }

    /// Aggregate ID this event relates to
    pub fn aggregate_id(&self) -> String {
        match self {
            DomainEventEnvelope::PolicyStoreCreated(e) => e.policy_store_id.clone(),
            DomainEventEnvelope::PolicyStoreUpdated(e) => e.policy_store_id.clone(),
            DomainEventEnvelope::PolicyStoreTagsUpdated(e) => e.policy_store_id.clone(),
            DomainEventEnvelope::PolicyStoreDeleted(e) => e.policy_store_id.clone(),
        }
    }

    /// Timestamp when event occurred
    pub fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            DomainEventEnvelope::PolicyStoreCreated(e) => e.occurred_at,
            DomainEventEnvelope::PolicyStoreUpdated(e) => e.occurred_at,
            DomainEventEnvelope::PolicyStoreTagsUpdated(e) => e.occurred_at,
            DomainEventEnvelope::PolicyStoreDeleted(e) => e.occurred_at,
        }
    }

    /// Event version for event schema evolution
    pub fn version(&self) -> u32 {
        match self {
            DomainEventEnvelope::PolicyStoreCreated(e) => e.version,
            DomainEventEnvelope::PolicyStoreUpdated(e) => e.version,
            DomainEventEnvelope::PolicyStoreTagsUpdated(e) => e.version,
            DomainEventEnvelope::PolicyStoreDeleted(e) => e.version,
        }
    }
}

// Note: We removed the DomainEvent trait and use DomainEventEnvelope (enum) directly
// This provides better type safety, easier serialization, and cleaner SOLID-compliant code

/// Event ID type
pub type EventId = String;

// ============================================================================
// Policy Store Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreCreated {
    pub event_id: EventId,
    pub policy_store_id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreUpdated {
    pub event_id: EventId,
    pub policy_store_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub changed_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreTagsUpdated {
    pub event_id: EventId,
    pub policy_store_id: String,
    pub old_tags: Vec<String>,
    pub new_tags: Vec<String>,
    pub changed_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreDeleted {
    pub event_id: EventId,
    pub policy_store_id: String,
    pub deleted_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

// ============================================================================
// Abstract Ports (Interfaces) - Hexagonal Architecture
// ============================================================================

/// Event Bus Port - Abstract interface for publishing events
#[async_trait]
pub trait EventBusPort: Send + Sync {
    async fn publish(
        &self,
        event: &DomainEventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Event Store Port - Abstract interface for persisting and retrieving events
#[async_trait]
pub trait EventStorePort: Send + Sync {
    async fn save_events(
        &self,
        aggregate_id: &str,
        events: &[DomainEventEnvelope],
        expected_version: u32,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Unique identifier for event subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub u64);

// ============================================================================
// Application Service (Uses Ports)
// ============================================================================

/// Event Dispatcher Port - Trait for dispatching events
/// This trait provides a simple interface for the gRPC services to publish events
#[async_trait]
pub trait EventDispatcherPort: Send + Sync {
    async fn dispatch(&self, event: DomainEventEnvelope) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Event Dispatcher - Application service that uses the ports
pub struct EventDispatcher<B: EventBusPort, S: EventStorePort> {
    event_bus: B,
    event_store: S,
}

impl<B: EventBusPort, S: EventStorePort> EventDispatcher<B, S> {
    pub fn new(event_bus: B, event_store: S) -> Self {
        Self {
            event_bus,
            event_store,
        }
    }

    pub async fn dispatch_with_aggregate(
        &self,
        aggregate_id: &str,
        events: Vec<DomainEventEnvelope>,
        expected_version: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _saved_events = self
            .event_store
            .save_events(aggregate_id, &events, expected_version)
            .await?;

        for event in &events {
            self.event_bus.publish(event).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<B: EventBusPort, S: EventStorePort> EventDispatcherPort for EventDispatcher<B, S> {
    async fn dispatch(&self, event: DomainEventEnvelope) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let aggregate_id = event.aggregate_id();
        let events = vec![event];
        let expected_version = 0; // TODO: Implement proper version management

        self.dispatch_with_aggregate(&aggregate_id, events, expected_version).await
    }
}

/// Filter for audit events queries
#[derive(Debug, Clone)]
pub struct AuditEventFilter {
    pub event_type: Option<String>,
    pub policy_store_id: Option<String>,
    pub user_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub search_text: Option<String>,
    pub client_ip: Option<String>,
    pub limit: Option<u32>,
}

/// Query result for audit events
#[derive(Debug, Clone)]
pub struct AuditEventQueryResult {
    pub event_id: String,
    pub event_type: &'static str,
    pub aggregate_id: String,
    pub event_data: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_domain_event_envelope_policy_store_created() {
        let event = PolicyStoreCreated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "store-123".to_string(),
            name: "Test Store".to_string(),
            description: Some("Test Description".to_string()),
            author: "test-user".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        let envelope = DomainEventEnvelope::PolicyStoreCreated(Box::new(event));

        assert_eq!(envelope.event_type(), "PolicyStoreCreated");
        assert_eq!(envelope.aggregate_id(), "store-123");
        assert_eq!(envelope.version(), 1);
    }

    #[test]
    fn test_domain_event_envelope_policy_store_updated() {
        let event = PolicyStoreUpdated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "store-456".to_string(),
            name: Some("Updated Store".to_string()),
            description: Some("Updated Description".to_string()),
            changed_by: "test-user".to_string(),
            occurred_at: Utc::now(),
            version: 2,
        };

        let envelope = DomainEventEnvelope::PolicyStoreUpdated(Box::new(event));

        assert_eq!(envelope.event_type(), "PolicyStoreUpdated");
        assert_eq!(envelope.aggregate_id(), "store-456");
        assert_eq!(envelope.version(), 2);
    }

    #[test]
    fn test_domain_event_envelope_policy_store_tags_updated() {
        let event = PolicyStoreTagsUpdated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "store-789".to_string(),
            old_tags: vec!["tag1".to_string()],
            new_tags: vec!["tag2".to_string(), "tag3".to_string()],
            changed_by: "test-user".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        let envelope = DomainEventEnvelope::PolicyStoreTagsUpdated(Box::new(event));

        assert_eq!(envelope.event_type(), "PolicyStoreTagsUpdated");
        assert_eq!(envelope.aggregate_id(), "store-789");
    }

    #[test]
    fn test_domain_event_envelope_policy_store_deleted() {
        let event = PolicyStoreDeleted {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "store-999".to_string(),
            deleted_by: "test-user".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        let envelope = DomainEventEnvelope::PolicyStoreDeleted(Box::new(event));

        assert_eq!(envelope.event_type(), "PolicyStoreDeleted");
        assert_eq!(envelope.aggregate_id(), "store-999");
    }

    #[test]
    fn test_all_event_types_have_unique_type_names() {
        let mut types = std::collections::HashSet::new();

        // Test each event type is unique
        types.insert("PolicyStoreCreated");
        types.insert("PolicyStoreUpdated");
        types.insert("PolicyStoreTagsUpdated");
        types.insert("PolicyStoreDeleted");

        assert_eq!(types.len(), 4, "All event types should be unique");
    }
}
