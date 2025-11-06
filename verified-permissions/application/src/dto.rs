//! Data Transfer Objects (DTOs)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Policy Store DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyStoreRequest {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStoreResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub author: String,
    pub tags: Vec<String>,
    pub identity_source_ids: Vec<String>,
    pub default_identity_source_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Policy DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyRequest {
    pub policy_store_id: String,
    pub policy_id: String,
    pub statement: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicyRequest {
    pub statement: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub policy_store_id: String,
    pub policy_id: String,
    pub statement: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Schema DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutSchemaRequest {
    pub policy_store_id: String,
    pub schema_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaResponse {
    pub policy_store_id: String,
    pub schema_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Authorization DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub policy_store_id: String,
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub context: Option<serde_json::Value>,
    pub entities: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub decision: String, // "ALLOW" or "DENY"
    pub determining_policies: Vec<String>,
    pub errors: Vec<String>,
}

// ============================================================================
// Identity Source DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentitySourceRequest {
    pub policy_store_id: String,
    pub configuration_type: String, // "cognito" or "oidc"
    pub configuration_json: String,
    pub claims_mapping_json: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySourceResponse {
    pub id: String,
    pub policy_store_id: String,
    pub configuration_type: String,
    pub configuration_json: String,
    pub claims_mapping_json: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
