//! Storage layer for policy stores, schemas, and policies

pub mod models;
pub mod repository;
pub mod repository_trait;
pub mod factory;

// Optional DB implementations
#[cfg(feature = "postgres")]
pub mod postgres_repository;
#[cfg(feature = "surreal")]
pub mod surreal_repository;

pub use repository::Repository;
pub use repository_trait::{PolicyRepository, AuthorizationLog};
pub use factory::create_repository;

#[cfg(feature = "postgres")]
pub use postgres_repository::PostgresRepository;
#[cfg(feature = "surreal")]
pub use surreal_repository::SurrealRepository;
