//! Event Infrastructure - Concrete implementations following Hexagonal Architecture
//!
//! This module provides infrastructure adapters (implementations) for the
//! abstract ports (interfaces) defined in the domain layer.

use async_trait::async_trait;
use hodei_domain::{
    DomainError,
    events::{DomainEvent, EventBusPort, EventHandler, EventStorePort, SubscriptionId},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory Event Bus Adapter (Infrastructure)
/// This adapter implements the EventBusPort interface from the domain
pub struct InMemoryEventBus {
    handlers:
        Arc<Mutex<HashMap<String, Vec<Box<dyn EventHandler<dyn DomainEvent + Send + Sync>>>>>>,
    subscription_counter: Arc<Mutex<u64>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            subscription_counter: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl EventBusPort for InMemoryEventBus {
    async fn publish(
        &self,
        event: &dyn DomainEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_type = event.event_type();
        let handlers = self.handlers.lock().unwrap();

        if let Some(event_handlers) = handlers.get(event_type) {
            for handler in event_handlers {
                // TODO: In production, spawn async tasks for each handler
                // For now, calling sequentially to maintain simplicity
            }
        }

        Ok(())
    }

    async fn subscribe(
        &self,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, Box<dyn std::error::Error + Send + Sync>> {
        let mut handlers = self.handlers.lock().unwrap();

        let subscription_id = {
            let mut counter = self.subscription_counter.lock().unwrap();
            let id = *counter;
            *counter += 1;
            SubscriptionId(id)
        };

        // Store handler without type-specific registration
        // In a real implementation, you'd want to filter events in the handler itself
        handlers
            .entry("all".to_string())
            .or_insert_with(Vec::new)
            .push(handler);

        Ok(subscription_id)
    }
}

/// SQLite Event Store Adapter (Infrastructure)
/// This adapter implements the EventStorePort interface from the domain
pub struct SqliteEventStore {
    pool: sqlx::SqlitePool,
}

impl SqliteEventStore {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pool = sqlx::SqlitePool::connect(database_url)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Create event store table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS domain_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                aggregate_id TEXT NOT NULL,
                event_data TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                version INTEGER NOT NULL,
                UNIQUE(event_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl EventStorePort for SqliteEventStore {
    async fn save_events(
        &self,
        aggregate_id: &str,
        events: &[Box<dyn DomainEvent>],
        expected_version: u32,
    ) -> Result<Vec<Box<dyn DomainEvent>>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement optimistic concurrency control with expected_version
        // For now, just save the events

        for event in events {
            let event_data = serde_json::to_string(event.as_ref())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            sqlx::query(
                r#"
                INSERT INTO domain_events (event_id, event_type, aggregate_id, event_data, occurred_at, version)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(event.event_id())
            .bind(event.event_type())
            .bind(event.aggregate_id())
            .bind(&event_data)
            .bind(event.occurred_at().to_rfc3339())
            .bind(event.version())
            .execute(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        }

        // Return the events (could return with additional metadata like IDs)
        Ok(events.to_vec())
    }

    async fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<Box<dyn DomainEvent>>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT event_data, event_type FROM domain_events WHERE aggregate_id = ? ORDER BY version ASC",
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut events = Vec::new();

        for row in rows {
            let event_data: String = row.get("event_data");
            let event_type: String = row.get("event_type");

            let event = deserialize_event(&event_type, &event_data)?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<Box<dyn DomainEvent>>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT event_data FROM domain_events WHERE event_type = ? ORDER BY occurred_at DESC",
        )
        .bind(event_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut events = Vec::new();

        for row in rows {
            let event_data: String = row.get("event_data");
            let event = deserialize_event(event_type, &event_data)?;
            events.push(event);
        }

        Ok(events)
    }
}

/// Helper function to deserialize events by type
fn deserialize_event(
    event_type: &str,
    event_data: &str,
) -> Result<Box<dyn DomainEvent>, Box<dyn std::error::Error + Send + Sync>> {
    match event_type {
        "PolicyStoreCreated" => {
            let event: hodei_domain::events::PolicyStoreCreated = serde_json::from_str(event_data)
                .map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize PolicyStoreCreated: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        "PolicyStoreUpdated" => {
            let event: hodei_domain::events::PolicyStoreUpdated = serde_json::from_str(event_data)
                .map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize PolicyStoreUpdated: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        "PolicyStoreTagsUpdated" => {
            let event: hodei_domain::events::PolicyStoreTagsUpdated =
                serde_json::from_str(event_data).map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize PolicyStoreTagsUpdated: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        "PolicyStoreDeleted" => {
            let event: hodei_domain::events::PolicyStoreDeleted = serde_json::from_str(event_data)
                .map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize PolicyStoreDeleted: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        "ApiCalled" => {
            let event: hodei_domain::events::ApiCalled =
                serde_json::from_str(event_data).map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize ApiCalled: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        "ApiCompleted" => {
            let event: hodei_domain::events::ApiCompleted = serde_json::from_str(event_data)
                .map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize ApiCompleted: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        "PolicyStoreAccessed" => {
            let event: hodei_domain::events::PolicyStoreAccessed = serde_json::from_str(event_data)
                .map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize PolicyStoreAccessed: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        "AuthorizationPerformed" => {
            let event: hodei_domain::events::AuthorizationPerformed =
                serde_json::from_str(event_data).map_err(|e| {
                    Box::new(DomainError::Internal(format!(
                        "Failed to deserialize AuthorizationPerformed: {}",
                        e
                    ))) as Box<dyn std::error::Error + Send + Sync>
                })?;
            Ok(Box::new(event) as Box<dyn DomainEvent>)
        }
        _ => Err(Box::new(
            DomainError::Internal(format!("Unknown event type: {}", event_type))
                as Box<dyn std::error::Error + Send + Sync>,
        )),
    }
}
