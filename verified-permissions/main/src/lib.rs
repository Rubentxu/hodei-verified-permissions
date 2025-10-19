//! Hodei Verified Permissions - Main application
//!
//! This is the main entry point that wires together all the crates.

pub use hodei_shared as shared;
pub use hodei_domain as domain;
pub use hodei_application as application;
pub use hodei_infrastructure as infrastructure;
pub use hodei_api as api;

// Re-export commonly used types
pub use api::proto;
