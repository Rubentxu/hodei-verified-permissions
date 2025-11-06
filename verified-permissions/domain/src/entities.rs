//! Domain entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::*;

/// Policy Store entity - Represents a container for policies and schemas
/// Supports multiple identity sources for flexible authentication scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStore {
    pub id: PolicyStoreId,
    pub name: String,
    pub description: Option<String>,
    /// Current status of the policy store (active/inactive)
    pub status: PolicyStoreStatus,
    /// Version number for versioning support
    pub version: String,
    /// Author/owner of the policy store
    pub author: String,
    /// List of tags for categorization
    pub tags: Vec<String>,
    /// List of identity source IDs associated with this policy store
    pub identity_source_ids: Vec<String>,
    /// Default identity source ID to use when not explicitly specified
    pub default_identity_source_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PolicyStore {
    pub fn new(
        id: PolicyStoreId,
        name: String,
        description: Option<String>,
        tags: Vec<String>,
        user: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            status: PolicyStoreStatus::Active,
            version: "1.0".to_string(),
            author: user,
            tags,
            identity_source_ids: Vec::new(),
            default_identity_source_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add an identity source to this policy store
    pub fn add_identity_source(&mut self, identity_source_id: String) {
        if !self.identity_source_ids.contains(&identity_source_id) {
            self.identity_source_ids.push(identity_source_id);
        }
    }

    /// Remove an identity source from this policy store
    pub fn remove_identity_source(&mut self, identity_source_id: &str) {
        self.identity_source_ids
            .retain(|id| id != identity_source_id);

        // If the removed source was the default, clear the default
        if self.default_identity_source_id.as_deref() == Some(identity_source_id) {
            self.default_identity_source_id = None;
        }
    }

    /// Set the default identity source
    pub fn set_default_identity_source(&mut self, identity_source_id: String) {
        if self.identity_source_ids.contains(&identity_source_id) {
            self.default_identity_source_id = Some(identity_source_id);
        }
    }

    /// Get the default identity source ID, or the first one if no default is set
    pub fn get_default_identity_source(&self) -> Option<&str> {
        self.default_identity_source_id
            .as_deref()
            .or_else(|| self.identity_source_ids.first().map(|s| s.as_str()))
    }

    /// Check if an identity source is associated with this policy store
    pub fn has_identity_source(&self, identity_source_id: &str) -> bool {
        self.identity_source_ids
            .contains(&identity_source_id.to_string())
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

/// Snapshot entity - Represents a point-in-time snapshot of a policy store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub policy_store_id: PolicyStoreId,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub policy_count: i32,
    pub has_schema: bool,
    pub schema_json: Option<String>,
    pub policies: Vec<SnapshotPolicy>,
    pub size_bytes: i64,
}

impl Snapshot {
    pub fn new(
        snapshot_id: String,
        policy_store_id: PolicyStoreId,
        description: Option<String>,
        policy_count: i32,
        has_schema: bool,
        schema_json: Option<String>,
        policies: Vec<SnapshotPolicy>,
        size_bytes: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            snapshot_id,
            policy_store_id,
            description,
            created_at: now,
            policy_count,
            has_schema,
            schema_json,
            policies,
            size_bytes,
        }
    }
}

/// Policy summary within a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPolicy {
    pub policy_id: String,
    pub description: Option<String>,
    pub statement: String,
}

impl SnapshotPolicy {
    pub fn new(policy_id: String, description: Option<String>, statement: String) -> Self {
        Self {
            policy_id,
            description,
            statement,
        }
    }
}

/// Result of a rollback operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub policy_store_id: PolicyStoreId,
    pub snapshot_id: String,
    pub rolled_back_at: DateTime<Utc>,
    pub policies_restored: i32,
    pub schema_restored: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_store_add_identity_source() {
        let mut store = PolicyStore::new(
            PolicyStoreId::new("store-1".to_string()).unwrap(),
            "Test Store".to_string(),
            None,
            vec![],
            "test_user".to_string(),
        );

        store.add_identity_source("identity-1".to_string());
        assert!(store.has_identity_source("identity-1"));
        assert_eq!(store.identity_source_ids.len(), 1);
    }

    #[test]
    fn test_policy_store_remove_identity_source() {
        let mut store = PolicyStore::new(
            PolicyStoreId::new("store-1".to_string()).unwrap(),
            "Test Store".to_string(),
            None,
            vec![],
            "test_user".to_string(),
        );

        store.add_identity_source("identity-1".to_string());
        store.remove_identity_source("identity-1");
        assert!(!store.has_identity_source("identity-1"));
        assert_eq!(store.identity_source_ids.len(), 0);
    }

    #[test]
    fn test_policy_store_set_default_identity_source() {
        let mut store = PolicyStore::new(
            PolicyStoreId::new("store-1".to_string()).unwrap(),
            "Test Store".to_string(),
            None,
            vec![],
            "test_user".to_string(),
        );

        store.add_identity_source("identity-1".to_string());
        store.set_default_identity_source("identity-1".to_string());

        assert_eq!(store.get_default_identity_source(), Some("identity-1"));
    }

    #[test]
    fn test_policy_store_get_default_first_if_not_set() {
        let mut store = PolicyStore::new(
            PolicyStoreId::new("store-1".to_string()).unwrap(),
            "Test Store".to_string(),
            None,
            vec![],
            "test_user".to_string(),
        );

        store.add_identity_source("identity-1".to_string());
        store.add_identity_source("identity-2".to_string());

        // Should return first one if no default is set
        assert_eq!(store.get_default_identity_source(), Some("identity-1"));
    }

    #[test]
    fn test_policy_store_remove_default_clears_it() {
        let mut store = PolicyStore::new(
            PolicyStoreId::new("store-1".to_string()).unwrap(),
            "Test Store".to_string(),
            None,
            vec![],
            "test_user".to_string(),
        );

        store.add_identity_source("identity-1".to_string());
        store.set_default_identity_source("identity-1".to_string());
        store.remove_identity_source("identity-1");

        assert_eq!(store.default_identity_source_id, None);
    }
}
