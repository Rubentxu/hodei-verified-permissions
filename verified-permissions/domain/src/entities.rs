//! Domain entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::*;

/// Policy Store entity - Represents a container for policies and schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStore {
    pub id: PolicyStoreId,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PolicyStore {
    pub fn new(id: PolicyStoreId, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Schema entity - Represents a Cedar schema for a policy store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub policy_store_id: PolicyStoreId,
    pub schema_json: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Schema {
    pub fn new(policy_store_id: PolicyStoreId, schema_json: String) -> Self {
        let now = Utc::now();
        Self {
            policy_store_id,
            schema_json,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Policy entity - Represents a Cedar policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub policy_store_id: PolicyStoreId,
    pub policy_id: PolicyId,
    pub statement: CedarPolicy,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Policy {
    pub fn new(
        policy_store_id: PolicyStoreId,
        policy_id: PolicyId,
        statement: CedarPolicy,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            policy_store_id,
            policy_id,
            statement,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Identity Source entity - Represents a source of identity information (Cognito, OIDC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySource {
    pub id: String,
    pub policy_store_id: PolicyStoreId,
    pub configuration_type: IdentitySourceType,
    pub configuration_json: String,
    pub claims_mapping_json: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IdentitySource {
    pub fn new(
        id: String,
        policy_store_id: PolicyStoreId,
        configuration_type: IdentitySourceType,
        configuration_json: String,
        claims_mapping_json: Option<String>,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            policy_store_id,
            configuration_type,
            configuration_json,
            claims_mapping_json,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Policy Template entity - Represents a reusable policy template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTemplate {
    pub template_id: String,
    pub policy_store_id: PolicyStoreId,
    pub statement: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PolicyTemplate {
    pub fn new(
        template_id: String,
        policy_store_id: PolicyStoreId,
        statement: String,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            template_id,
            policy_store_id,
            statement,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}
