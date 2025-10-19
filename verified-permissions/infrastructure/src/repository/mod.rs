//! Repository implementations for persistence layer

pub mod models;
mod sqlite_repository;
pub mod adapter;

// Optional DB implementations
#[cfg(feature = "postgres")]
pub mod postgres_repository;
#[cfg(feature = "surreal")]
pub mod surreal_repository;

pub use models::*;
pub use sqlite_repository::SqliteRepository;
pub use adapter::RepositoryAdapter;

#[cfg(feature = "postgres")]
pub use postgres_repository::PostgresRepository;
#[cfg(feature = "surreal")]
pub use surreal_repository::SurrealRepository;
