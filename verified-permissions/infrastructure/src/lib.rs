//! Hodei Infrastructure - External adapters and implementations
//!
//! This layer contains implementations of domain interfaces (repositories)
//! and external service integrations (database, cache, JWT, etc.).

pub mod repository;
pub mod factory;

// TODO: These modules need updating to use new crate structure
// Temporarily commented out to allow compilation
// pub mod cache;
// pub mod jwt;
// pub mod config;

pub use repository::{RepositoryAdapter, SqliteRepository, models};
pub use factory::create_repository;
