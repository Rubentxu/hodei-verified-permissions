//! Domain Events - Domain Event pattern implementation
//!
//! This module contains domain events and abstract interfaces following
//! hexagonal architecture principles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Domain Events
// ============================================================================

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    /// Unique event ID
    fn event_id(&self) -> String;
    /// Event type name
    fn event_type(&self) -> &'static str;
    /// Aggregate ID this event relates to
    fn aggregate_id(&self) -> String;
    /// Timestamp when event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
    /// Event version for event schema evolution
    fn version(&self) -> u32;
}

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

impl DomainEvent for PolicyStoreCreated {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "PolicyStoreCreated"
    }

    fn aggregate_id(&self) -> String {
        self.policy_store_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
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

impl DomainEvent for PolicyStoreUpdated {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "PolicyStoreUpdated"
    }

    fn aggregate_id(&self) -> String {
        self.policy_store_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
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

impl DomainEvent for PolicyStoreTagsUpdated {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "PolicyStoreTagsUpdated"
    }

    fn aggregate_id(&self) -> String {
        self.policy_store_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreDeleted {
    pub event_id: EventId,
    pub policy_store_id: String,
    pub deleted_by: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent for PolicyStoreDeleted {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "PolicyStoreDeleted"
    }

    fn aggregate_id(&self) -> String {
        self.policy_store_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
}

// ============================================================================
// Audit Events (CloudTrail-like)
// ============================================================================

/// Fired when any API call is made to the service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCalled {
    pub event_id: EventId,
    pub service_name: String, // e.g., "AuthorizationControl"
    pub method_name: String,  // e.g., "CreatePolicyStore"
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: String,
    pub request_size_bytes: i64,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent for ApiCalled {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "ApiCalled"
    }

    fn aggregate_id(&self) -> String {
        self.request_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
}

/// Fired when API call completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCompleted {
    pub event_id: EventId,
    pub request_id: String,
    pub service_name: String,
    pub method_name: String,
    pub status_code: i32, // 0=success, 1=failure
    pub error_message: Option<String>,
    pub response_size_bytes: i64,
    pub duration_ms: u64,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent for ApiCompleted {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "ApiCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.request_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
}

/// Fired when policy store is accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreAccessed {
    pub event_id: EventId,
    pub policy_store_id: String,
    pub access_type: AccessType, // READ, WRITE, DELETE
    pub operation: String,       // e.g., "GetPolicyStore", "ListPolicies"
    pub user_id: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessType {
    READ,
    WRITE,
    DELETE,
}

impl DomainEvent for PolicyStoreAccessed {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "PolicyStoreAccessed"
    }

    fn aggregate_id(&self) -> String {
        self.policy_store_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
}

/// Fired when authorization decision is made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationPerformed {
    pub event_id: EventId,
    pub policy_store_id: String,
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub decision: AuthorizationDecision, // ALLOW/DENY
    pub determining_policies: Vec<String>,
    pub client_ip: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationDecision {
    ALLOW,
    DENY,
}

impl DomainEvent for AuthorizationPerformed {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn event_type(&self) -> &'static str {
        "AuthorizationPerformed"
    }

    fn aggregate_id(&self) -> String {
        self.policy_store_id.clone()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> u32 {
        self.version
    }
}

// ============================================================================
// Abstract Ports (Interfaces) - Hexagonal Architecture
// ============================================================================

/// Event Bus Port - Abstract interface for publishing and subscribing to events
pub trait EventBusPort: Send + Sync {
    async fn publish(
        &self,
        event: &dyn DomainEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn subscribe(
        &self,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, Box<dyn std::error::Error + Send + Sync>>;
}

/// Event Store Port - Abstract interface for persisting and retrieving events
pub trait EventStorePort: Send + Sync {
    async fn save_events(
        &self,
        aggregate_id: &str,
        events: &[Box<dyn DomainEvent>],
        expected_version: u32,
    ) -> Result<Vec<Box<dyn DomainEvent>>, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<Box<dyn DomainEvent>>, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<Box<dyn DomainEvent>>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Event Handler - Contract for handling domain events
/// Uses Box<dyn Future> to be dyn compatible
pub trait EventHandler: Send + Sync {
    fn handle(
        &self,
        event: &dyn DomainEvent,
    ) -> Box<
        dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync,
    >;
}

/// Unique identifier for event subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub u64);

// ============================================================================
// Application Service (Uses Ports)
// ============================================================================

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

    pub async fn dispatch(
        &self,
        aggregate_id: &str,
        events: Vec<Box<dyn DomainEvent>>,
        expected_version: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _saved_events = self
            .event_store
            .save_events(aggregate_id, &events, expected_version)
            .await?;

        for event in &events {
            self.event_bus.publish(event.as_ref()).await?;
        }

        Ok(())
    }
}
