//! Domain Layer - Core business logic
//!
//! This layer contains the pure business logic with no dependencies on external frameworks.
//! It defines entities, value objects, and domain services.

pub mod entities;
pub mod value_objects;
pub mod services;
pub mod errors;

pub use entities::*;
pub use value_objects::*;
pub use errors::DomainError;
