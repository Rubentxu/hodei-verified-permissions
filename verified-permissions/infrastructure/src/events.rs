//! Event Infrastructure - Concrete implementations following Hexagonal Architecture
//!
//! This module provides infrastructure adapters (implementations) for the
//! abstract ports (interfaces) defined in the domain layer.

use async_trait::async_trait;

use hodei_domain::events::{DomainEventEnvelope, EventBusPort, EventStorePort};
use serde_json;
use sqlx::{Row, SqlitePool};

/// Event Store Type - for dynamic selection via configuration
#[derive(Debug, Clone, PartialEq)]
pub enum EventStoreType {
    SQLite,
    PostgreSQL,
}

impl EventStoreType {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sqlite" | "file" => Some(Self::SQLite),
            "postgres" | "postgresql" | "pg" => Some(Self::PostgreSQL),
            _ => None,
        }
    }

    /// Detect from database URL
    pub fn from_database_url(database_url: &str) -> Self {
        let url_lower = database_url.to_lowercase();
        if url_lower.starts_with("sqlite:") || url_lower.starts_with("file:") {
            Self::SQLite
        } else if url_lower.starts_with("postgres:")
            || url_lower.starts_with("postgresql:")
            || url_lower.starts_with("tcp:")
        {
            #[cfg(feature = "postgres")]
            {
                Self::PostgreSQL
            }
            #[cfg(not(feature = "postgres"))]
            {
                tracing::warn!(
                    "PostgreSQL selected but 'postgres' feature not enabled. Falling back to SQLite."
                );
                Self::SQLite
            }
        } else {
            tracing::warn!("Unknown database URL format, defaulting to SQLite");
            Self::SQLite
        }
    }
}

/// Dynamic Event Store wrapper - allows runtime selection
pub enum EventStoreBox {
    SQLite(SqliteEventStore),
    #[cfg(feature = "postgres")]
    PostgreSQL(PostgresEventStore),
}

impl EventStoreBox {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let store_type = EventStoreType::from_database_url(database_url);

        match store_type {
            EventStoreType::SQLite => {
                let store = SqliteEventStore::new(database_url).await?;
                Ok(Self::SQLite(store))
            }
            EventStoreType::PostgreSQL => {
                #[cfg(feature = "postgres")]
                {
                    let store = PostgresEventStore::new(database_url).await?;
                    Ok(Self::PostgreSQL(store))
                }
                #[cfg(not(feature = "postgres"))]
                {
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "PostgreSQL support not enabled. Enable with `--features postgres`",
                    ))
                        as Box<dyn std::error::Error + Send + Sync>)
                }
            }
        }
    }

    pub fn get_type(&self) -> EventStoreType {
        match self {
            Self::SQLite(_) => EventStoreType::SQLite,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(_) => EventStoreType::PostgreSQL,
        }
    }
}

#[async_trait]
impl EventStorePort for EventStoreBox {
    async fn save_events(
        &self,
        aggregate_id: &str,
        events: &[DomainEventEnvelope],
        expected_version: u32,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::SQLite(store) => {
                store
                    .save_events(aggregate_id, events, expected_version)
                    .await
            }
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(store) => {
                store
                    .save_events(aggregate_id, events, expected_version)
                    .await
            }
        }
    }

    async fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::SQLite(store) => store.get_events(aggregate_id).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(store) => store.get_events(aggregate_id).await,
        }
    }

    async fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::SQLite(store) => store.get_events_by_type(event_type).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(store) => store.get_events_by_type(event_type).await,
        }
    }
}

/// In-memory Event Bus Adapter (Infrastructure)
/// This adapter implements the EventBusPort interface from the domain
pub struct InMemoryEventBus {
    // No handlers needed - events are only published, not processed
    _placeholder: (),
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

#[async_trait]
impl EventBusPort for InMemoryEventBus {
    async fn publish(
        &self,
        _event: &DomainEventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // No-op: events are not processed in this implementation
        Ok(())
    }
}

/// SQLite Event Store Adapter (Infrastructure)
/// This adapter implements the EventStorePort interface from the domain
pub struct SqliteEventStore {
    pool: SqlitePool,
}

/// PostgreSQL Event Store Adapter (Infrastructure)
/// This adapter implements the EventStorePort interface from the domain
/// Only available with postgres feature flag
#[cfg(feature = "postgres")]
pub struct PostgresEventStore {
    pool: sqlx::PgPool,
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

        // Create indexes for optimal query performance on audit log queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_type
            ON domain_events(event_type)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_aggregate
            ON domain_events(aggregate_id)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_occurred_at
            ON domain_events(occurred_at)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_type_occurred_at
            ON domain_events(event_type, occurred_at)
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
        events: &[DomainEventEnvelope],
        expected_version: u32,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement optimistic concurrency control with expected_version
        // For now, just save the events

        for event in events {
            let event_data = serde_json::to_string(event)
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
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT event_data FROM domain_events WHERE aggregate_id = ? ORDER BY version ASC",
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut events = Vec::new();

        for row in rows {
            let event_data: String = row.try_get("event_data")?;

            let event: DomainEventEnvelope = serde_json::from_str(&event_data)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT event_data FROM domain_events WHERE event_type = ? ORDER BY occurred_at DESC",
        )
        .bind(event_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut events = Vec::new();

        for row in rows {
            let event_data: String = row.try_get("event_data")?;

            let event: DomainEventEnvelope = serde_json::from_str(&event_data)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            events.push(event);
        }

        Ok(events)
    }
}

#[cfg(feature = "postgres")]
impl PostgresEventStore {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pool = sqlx::PgPool::connect(database_url)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Create event store table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS domain_events (
                id SERIAL PRIMARY KEY,
                event_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                aggregate_id TEXT NOT NULL,
                event_data TEXT NOT NULL,
                occurred_at TIMESTAMPTZ NOT NULL,
                version INTEGER NOT NULL,
                UNIQUE(event_id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Create indexes for optimal query performance on audit log queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_type
            ON domain_events(event_type)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_aggregate
            ON domain_events(aggregate_id)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_occurred_at
            ON domain_events(occurred_at)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_domain_events_type_occurred_at
            ON domain_events(event_type, occurred_at)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(Self { pool })
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl EventStorePort for PostgresEventStore {
    async fn save_events(
        &self,
        aggregate_id: &str,
        events: &[DomainEventEnvelope],
        expected_version: u32,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement optimistic concurrency control with expected_version
        // For now, just save the events

        for event in events {
            let event_data = serde_json::to_string(event)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            sqlx::query(
                r#"
                INSERT INTO domain_events (event_id, event_type, aggregate_id, event_data, occurred_at, version)
                VALUES ($1, $2, $3, $4, $5, $6)
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
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT event_data FROM domain_events WHERE aggregate_id = $1 ORDER BY version ASC",
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut events = Vec::new();

        for row in rows {
            let event_data: String = row.try_get("event_data")?;

            let event: DomainEventEnvelope = serde_json::from_str(&event_data)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<DomainEventEnvelope>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT event_data FROM domain_events WHERE event_type = $1 ORDER BY occurred_at DESC",
        )
        .bind(event_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut events = Vec::new();

        for row in rows {
            let event_data: String = row.try_get("event_data")?;

            let event: DomainEventEnvelope = serde_json::from_str(&event_data)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            events.push(event);
        }

        Ok(events)
    }
}

// Note: With DomainEventEnvelope, we can serialize/deserialize directly using serde_json
// No need for separate helper functions. The enum carries all type information.

#[cfg(test)]
mod event_store_tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use hodei_domain::events::{PolicyStoreCreated, PolicyStoreUpdated};

    #[tokio::test]
    async fn test_event_store_type_from_sqlite_url() {
        let store_type = EventStoreType::from_database_url("sqlite:test.db");
        assert_eq!(store_type, EventStoreType::SQLite);
    }

    #[tokio::test]
    async fn test_event_store_type_from_postgres_url() {
        let store_type = EventStoreType::from_database_url("postgres://localhost/test");

        #[cfg(feature = "postgres")]
        assert_eq!(store_type, EventStoreType::PostgreSQL);

        #[cfg(not(feature = "postgres"))]
        assert_eq!(store_type, EventStoreType::SQLite);
    }

    #[tokio::test]
    async fn test_event_store_type_auto_detection() {
        // SQLite detection
        let sqlite_type = EventStoreType::from_database_url("sqlite:///path/to/db.db");
        assert_eq!(sqlite_type, EventStoreType::SQLite);

        let file_type = EventStoreType::from_database_url("file:test.db");
        assert_eq!(file_type, EventStoreType::SQLite);
    }

    #[tokio::test]
    async fn test_event_store_box_creation_sqlite() {
        // Use in-memory SQLite database for testing
        let database_url = "sqlite::memory:";

        let store = EventStoreBox::new(database_url).await.unwrap();
        assert!(matches!(store, EventStoreBox::SQLite(_)));

        let store_type = store.get_type();
        assert_eq!(store_type, EventStoreType::SQLite);
    }

    #[tokio::test]
    async fn test_event_store_box_save_and_retrieve_events() {
        // Use in-memory SQLite database for testing
        let database_url = "sqlite::memory:";

        let store = EventStoreBox::new(database_url).await.unwrap();

        // Create a test event
        let event = PolicyStoreCreated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "test-store-123".to_string(),
            name: "Test Store".to_string(),
            description: Some("Test Description".to_string()),
            author: "test-user".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        let envelope = DomainEventEnvelope::PolicyStoreCreated(Box::new(event));

        // Save the event
        let saved_events = store
            .save_events("test-store-123", &[envelope.clone()], 0)
            .await
            .unwrap();

        assert_eq!(saved_events.len(), 1);
        assert_eq!(saved_events[0].event_type(), "PolicyStoreCreated");

        // Retrieve the event
        let retrieved_events = store
            .get_events("test-store-123")
            .await
            .unwrap();

        assert_eq!(retrieved_events.len(), 1);
        assert_eq!(retrieved_events[0].event_type(), "PolicyStoreCreated");
        assert_eq!(retrieved_events[0].aggregate_id(), "test-store-123");
    }

    #[tokio::test]
    async fn test_event_store_box_get_events_by_type() {
        // Use in-memory SQLite database for testing
        let database_url = "sqlite::memory:";

        let store = EventStoreBox::new(database_url).await.unwrap();

        // Create multiple events of different types
        let event1 = PolicyStoreCreated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "store-1".to_string(),
            name: "Store 1".to_string(),
            description: None,
            author: "user1".to_string(),
            occurred_at: Utc::now(),
            version: 1,
        };

        let event2 = PolicyStoreUpdated {
            event_id: Uuid::new_v4().to_string(),
            policy_store_id: "store-1".to_string(),
            name: Some("Updated Store 1".to_string()),
            description: None,
            changed_by: "user1".to_string(),
            occurred_at: Utc::now(),
            version: 2,
        };

        let envelope1 = DomainEventEnvelope::PolicyStoreCreated(Box::new(event1));
        let envelope2 = DomainEventEnvelope::PolicyStoreUpdated(Box::new(event2));

        // Save both events
        store
            .save_events("store-1", &[envelope1, envelope2], 0)
            .await
            .unwrap();

        // Get events by type
        let created_events = store
            .get_events_by_type("PolicyStoreCreated")
            .await
            .unwrap();

        assert_eq!(created_events.len(), 1);
        assert_eq!(created_events[0].event_type(), "PolicyStoreCreated");

        let updated_events = store
            .get_events_by_type("PolicyStoreUpdated")
            .await
            .unwrap();

        assert_eq!(updated_events.len(), 1);
        assert_eq!(updated_events[0].event_type(), "PolicyStoreUpdated");
    }
}
