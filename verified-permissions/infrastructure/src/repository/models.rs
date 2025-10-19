//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStore {
    pub id: String,
    pub description: Option<String>,
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
    pub configuration_type: String, // "cognito" or "oidc"
    pub configuration_json: String, // JSON serialized configuration
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
