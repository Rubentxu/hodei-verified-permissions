//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStore {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String, // "active" or "inactive"
    pub version: String,
    pub author: String,
    pub tags: String,                // JSON serialized vector of strings
    pub identity_source_ids: String, // JSON serialized vector of strings
    pub default_identity_source_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub policy_store_id: String,
    pub schema_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub policy_store_id: String,
    pub policy_id: String,
    pub statement: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySource {
    pub id: String,
    pub policy_store_id: String,
    pub configuration_type: String,          // "cognito" or "oidc"
    pub configuration_json: String,          // JSON serialized configuration
    pub claims_mapping_json: Option<String>, // JSON serialized claims mapping
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTemplate {
    pub template_id: String,
    pub policy_store_id: String,
    pub statement: String, // Cedar policy with ?principal and/or ?resource placeholders
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthorizationLog {
    pub policy_store_id: String,
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub decision: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreAuditLog {
    pub id: i64,
    pub policy_store_id: String,
    pub action: String, // "CREATE", "UPDATE", "DELETE"
    pub user_id: String,
    pub changes: Option<String>, // JSON string of changes
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Snapshot database model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub policy_store_id: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub policy_count: i32,
    pub has_schema: bool,
    pub schema_json: Option<String>,
    pub policies: Vec<SnapshotPolicy>,
    pub size_bytes: i64,
}

/// Policy within a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPolicy {
    pub policy_id: String,
    pub description: Option<String>,
    pub statement: String,
}

/// Result of rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub policy_store_id: String,
    pub snapshot_id: String,
    pub rolled_back_at: DateTime<Utc>,
    pub policies_restored: i32,
    pub schema_restored: bool,
}

/// Audit log entry model for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub event_id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub event_data: String, // JSON serialized event data
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}
