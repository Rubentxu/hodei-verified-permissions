//! Hodei Infrastructure - External adapters and implementations
//!
//! This layer contains implementations of domain interfaces (repositories)
//! and external service integrations (database, cache, JWT, etc.).

pub mod repository;
pub mod factory;

// TODO: Update these modules to use new structure
// pub mod cache;
// pub mod jwt;
// pub mod config;

pub use repository::{RepositoryAdapter, SqliteRepository};
pub use factory::create_repository;
