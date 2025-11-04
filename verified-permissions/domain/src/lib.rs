//! Hodei Domain - Core business logic
//!
//! This layer contains the pure business logic with no dependencies on external frameworks.
//! It defines entities, value objects, domain services, and repository traits.

pub mod entities;
pub mod errors;
pub mod events;
pub mod repository;
pub mod services;
pub mod value_objects;

pub use entities::*;
pub use errors::{DomainError, DomainResult};
pub use repository::*;
pub use services::{AuthorizationEvaluator, PolicyValidator};
pub use value_objects::*;
