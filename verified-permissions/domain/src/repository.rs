//! Repository traits - Abstraction for persistence layer

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::entities::*;
use crate::value_objects::*;
use crate::errors::DomainResult;

/// Authorization log for auditing
#[derive(Debug, Clone)]
pub struct AuthorizationLog {
    pub policy_store_id: PolicyStoreId,
    pub principal: Principal,
    pub action: Action,
    pub resource: Resource,
    pub decision: AuthorizationDecision,
    pub timestamp: DateTime<Utc>,
}

/// Repository trait for policy store operations
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    // ============================================================================
    // Policy Store Operations
    // ============================================================================
    
    /// Creates a new Policy Store
    async fn create_policy_store(&self, description: Option<String>) -> DomainResult<PolicyStore>;
    
    /// Gets a Policy Store by ID
    async fn get_policy_store(&self, id: &PolicyStoreId) -> DomainResult<PolicyStore>;
    
    /// Lists all Policy Stores
    async fn list_policy_stores(&self) -> DomainResult<Vec<PolicyStore>>;
    
    /// Deletes a Policy Store and all its content (cascade)
    async fn delete_policy_store(&self, id: &PolicyStoreId) -> DomainResult<()>;
    
    // ============================================================================
    // Schema Operations
    // ============================================================================
    
    /// Saves or updates the schema for a Policy Store
    async fn put_schema(&self, policy_store_id: &PolicyStoreId, schema: String) -> DomainResult<()>;
    
    /// Gets the schema for a Policy Store
    async fn get_schema(&self, policy_store_id: &PolicyStoreId) -> DomainResult<Option<Schema>>;
    
    // ============================================================================
    // Policy Operations
    // ============================================================================
    
    /// Creates a new policy
    async fn create_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
        statement: &CedarPolicy,
        description: Option<String>,
    ) -> DomainResult<Policy>;
    
    /// Gets a policy by ID
    async fn get_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
    ) -> DomainResult<Policy>;
    
    /// Lists all policies for a Policy Store
    async fn list_policies(&self, policy_store_id: &PolicyStoreId) -> DomainResult<Vec<Policy>>;
    
    /// Updates a policy
    async fn update_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
        statement: &CedarPolicy,
        description: Option<String>,
    ) -> DomainResult<Policy>;
    
    /// Deletes a policy
    async fn delete_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
    ) -> DomainResult<()>;
    
    // ============================================================================
    // Identity Source Operations
    // ============================================================================
    
    /// Creates a new identity source
    async fn create_identity_source(
        &self,
        policy_store_id: &PolicyStoreId,
        configuration_type: &IdentitySourceType,
        configuration_json: String,
        claims_mapping_json: Option<String>,
        description: Option<String>,
    ) -> DomainResult<IdentitySource>;
    
    /// Gets an identity source by ID
    async fn get_identity_source(
        &self,
        policy_store_id: &PolicyStoreId,
        identity_source_id: &str,
    ) -> DomainResult<IdentitySource>;
    
    /// Lists all identity sources for a Policy Store
    async fn list_identity_sources(
        &self,
        policy_store_id: &PolicyStoreId,
    ) -> DomainResult<Vec<IdentitySource>>;
    
    /// Deletes an identity source
    async fn delete_identity_source(
        &self,
        policy_store_id: &PolicyStoreId,
        identity_source_id: &str,
    ) -> DomainResult<()>;
    
    // ============================================================================
    // Policy Template Operations
    // ============================================================================
    
    /// Creates a new policy template
    async fn create_policy_template(
        &self,
        policy_store_id: &PolicyStoreId,
        template_id: String,
        statement: String,
        description: Option<String>,
    ) -> DomainResult<PolicyTemplate>;
    
    /// Gets a policy template by ID
    async fn get_policy_template(
        &self,
        policy_store_id: &PolicyStoreId,
        template_id: &str,
    ) -> DomainResult<PolicyTemplate>;
    
    /// Lists all policy templates for a Policy Store
    async fn list_policy_templates(
        &self,
        policy_store_id: &PolicyStoreId,
    ) -> DomainResult<Vec<PolicyTemplate>>;
    
    /// Deletes a policy template
    async fn delete_policy_template(
        &self,
        policy_store_id: &PolicyStoreId,
        template_id: &str,
    ) -> DomainResult<()>;
    
    // ============================================================================
    // Audit Operations
    // ============================================================================
    
    /// Logs an authorization decision for auditing
    async fn log_authorization(&self, log: AuthorizationLog) -> DomainResult<()>;
}
