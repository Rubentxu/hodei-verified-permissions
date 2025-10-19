//! Hodei Domain - Core business logic
//!
//! This layer contains the pure business logic with no dependencies on external frameworks.
//! It defines entities, value objects, domain services, and repository traits.

pub mod entities;
pub mod value_objects;
pub mod services;
pub mod errors;
pub mod repository;

pub use entities::*;
pub use value_objects::*;
pub use services::{AuthorizationEvaluator, PolicyValidator};
pub use errors::{DomainError, DomainResult};
pub use repository::*;
