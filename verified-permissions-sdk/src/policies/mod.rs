//! Policy Generation Module
//!
//! This module provides utilities for generating Cedar policies from OpenAPI specifications
//! with a focus on least privilege access control.

pub mod analyzer;
pub mod generators;
pub mod templates;
pub mod types;

pub use analyzer::*;
pub use generators::*;
pub use templates::*;
pub use types::*;

/// A complete bundle of generated authorization artifacts
#[derive(Debug, Clone)]
pub struct PolicyBundle {
    /// Generated Cedar schemas
    pub schemas: Vec<GeneratedSchema>,
    /// Generated Cedar policies
    pub policies: Vec<CedarPolicy>,
    /// Policy templates for different use cases
    pub templates: Vec<PolicyTemplate>,
    /// Security analysis report
    pub security_report: SecurityReport,
}

/// A generated Cedar schema with metadata
#[derive(Debug, Clone)]
pub struct GeneratedSchema {
    /// The namespace for this schema
    pub namespace: String,
    /// The schema content in JSON format
    pub content: String,
    /// The version of the schema (v2 or v4)
    pub version: SchemaVersion,
    /// Metadata about the schema generation
    pub metadata: SchemaMetadata,
}

/// Metadata about a generated schema
#[derive(Debug, Clone)]
pub struct SchemaMetadata {
    pub namespace: String,
    pub mapping_type: String,
    pub action_count: usize,
    pub entity_type_count: usize,
    pub base_path: Option<String>,
    pub openapi_version: String,
}

/// A Cedar policy with metadata
#[derive(Debug, Clone)]
pub struct CedarPolicy {
    /// Unique identifier for the policy
    pub id: String,
    /// The policy description/comment
    pub description: String,
    /// The policy content in Cedar syntax
    pub content: String,
    /// The effect of the policy
    pub effect: PolicyEffect,
    /// The scope/visibility of the policy
    pub scope: PolicyScope,
}

/// The effect of a Cedar policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEffect {
    /// Allow access
    Permit,
    /// Deny access
    Forbid,
}

/// The scope of a policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyScope {
    /// Global policy (applies to all resources)
    Global,
    /// Resource-specific policy
    Resource(String),
    /// Action-specific policy
    Action(String),
    /// Role-specific policy
    Role(String),
}

/// Schema version
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaVersion {
    /// Cedar Schema v2 (legacy)
    V2,
    /// Cedar Schema v4 (current)
    V4,
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaVersion::V2 => write!(f, "v2"),
            SchemaVersion::V4 => write!(f, "v4"),
        }
    }
}
