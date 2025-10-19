//! Common result types

/// Standard result type used across the application
pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;
