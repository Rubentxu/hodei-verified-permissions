//! Factory for creating repository and event store instances

use crate::repository::RepositoryAdapter;
use crate::{
    SqliteEventStore,
    events::{EventStoreBox, EventStoreType, InMemoryEventBus},
};

/// Creates a repository instance based on configuration
pub async fn create_repository(database_url: &str) -> anyhow::Result<RepositoryAdapter> {
    RepositoryAdapter::new(database_url).await
}

/// Creates an event bus instance
pub fn create_event_bus() -> InMemoryEventBus {
    InMemoryEventBus::new()
}

/// Helper to create SQLite event store (for main.rs use)
pub async fn create_sqlite_event_store(
    database_url: &str,
) -> Result<SqliteEventStore, Box<dyn std::error::Error + Send + Sync>> {
    SqliteEventStore::new(database_url).await
}

/// Creates an event store instance with database selection based on configuration
/// This function supports both SQLite and PostgreSQL via feature flags
pub async fn create_event_store(
    database_url: &str,
) -> Result<EventStoreBox, Box<dyn std::error::Error + Send + Sync>> {
    let store = EventStoreBox::new(database_url).await?;
    let store_type = store.get_type();

    tracing::info!("📊 Event store initialized: {:?}", store_type);

    Ok(store)
}
