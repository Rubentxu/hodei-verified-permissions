//! Hodei Infrastructure - External adapters and implementations
//!
//! This layer contains implementations of domain interfaces (repositories)
//! and external service integrations (database, cache, JWT, etc.).

pub mod error;
pub mod events;
pub mod factory;
pub mod jwt;
pub mod repository;

// TODO: These modules need updating to use new crate structure
// Temporarily commented out to allow compilation
// pub mod cache;
// pub mod config;

#[cfg(feature = "postgres")]
pub use events::PostgresEventStore;
pub use events::{EventStoreBox, EventStoreType, InMemoryEventBus, SqliteEventStore};
pub use factory::{create_event_bus, create_event_store, create_repository};
pub use repository::{RepositoryAdapter, SqliteRepository, models};
