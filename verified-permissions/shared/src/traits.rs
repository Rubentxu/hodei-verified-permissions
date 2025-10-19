//! Common traits used across the application

use async_trait::async_trait;

/// Trait for entities that can be identified
pub trait Identifiable {
    type Id;
    fn id(&self) -> &Self::Id;
}

/// Trait for entities that have timestamps
pub trait Timestamped {
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;
}
