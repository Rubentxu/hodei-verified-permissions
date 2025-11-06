//! Adapter that implements domain PolicyRepository trait using SQLite

use async_trait::async_trait;
use hodei_domain::{
    CedarPolicy, DomainError, DomainResult, IdentitySource, IdentitySourceType,
    Policy, PolicyId, PolicyRepository, PolicyStore, PolicyStoreId, PolicyTemplate,
    RollbackResult, Schema, Snapshot, SnapshotPolicy,
};
use serde_json;

use super::{SqliteRepository, models};

/// Adapter that bridges domain repository trait with infrastructure implementation
pub struct RepositoryAdapter {
    sqlite_repo: SqliteRepository,
}

impl RepositoryAdapter {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let sqlite_repo = SqliteRepository::new(database_url).await?;
        Ok(Self { sqlite_repo })
    }

    fn map_policy_store(model: models::PolicyStore) -> DomainResult<PolicyStore> {
        let id = PolicyStoreId::new(model.id)?;
        let tags: Vec<String> = serde_json::from_str(&model.tags).unwrap_or_else(|_| Vec::new());
        let status = match model.status.as_str() {
            "inactive" => hodei_domain::PolicyStoreStatus::Inactive,
            _ => hodei_domain::PolicyStoreStatus::Active,
        };
        Ok(PolicyStore {
            id,
            name: model.name,
            description: model.description,
            status,
            version: model.version,
            author: model.author,
            tags,
            created_at: model.created_at,
            updated_at: model.updated_at,
            default_identity_source_id: None,
            identity_source_ids: Vec::new(),
        })
    }

    fn map_schema(model: models::Schema) -> DomainResult<Schema> {
        let policy_store_id = PolicyStoreId::new(model.policy_store_id)?;
        Ok(Schema {
            policy_store_id,
            schema_json: model.schema_json,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    fn map_policy(model: models::Policy) -> DomainResult<Policy> {
        let policy_store_id = PolicyStoreId::new(model.policy_store_id)?;
        let policy_id = PolicyId::new(model.policy_id)?;
        let statement = CedarPolicy::new(model.statement)?;
        Ok(Policy {
            policy_store_id,
            policy_id,
            statement,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    fn map_identity_source(model: models::IdentitySource) -> DomainResult<IdentitySource> {
        let policy_store_id = PolicyStoreId::new(model.policy_store_id)?;
        let configuration_type = IdentitySourceType::try_from(model.configuration_type)?;
        Ok(IdentitySource {
            id: model.id,
            policy_store_id,
            configuration_type,
            configuration_json: model.configuration_json,
            claims_mapping_json: model.claims_mapping_json,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    fn map_policy_template(model: models::PolicyTemplate) -> DomainResult<PolicyTemplate> {
        let policy_store_id = PolicyStoreId::new(model.policy_store_id)?;
        Ok(PolicyTemplate {
            template_id: model.template_id,
            policy_store_id,
            statement: model.statement,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }

    fn cedar_statement(statement: &CedarPolicy) -> String {
        statement.as_str().to_string()
    }

    fn policy_store_id_str(id: &PolicyStoreId) -> &str {
        id.as_str()
    }

    fn policy_id_str(id: &PolicyId) -> &str {
        id.as_str()
    }

    fn identity_source_type_str(ty: &IdentitySourceType) -> &'static str {
        match ty {
            IdentitySourceType::Cognito => "cognito",
            IdentitySourceType::Oidc => "oidc",
        }
    }
}

#[async_trait]
impl PolicyRepository for RepositoryAdapter {
    async fn create_policy_store(
        &self,
        name: String,
        description: Option<String>,
    ) -> DomainResult<PolicyStore> {
        let model = self
            .sqlite_repo
            .create_policy_store(name, description)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy_store(model)
    }

    async fn get_policy_store(&self, id: &PolicyStoreId) -> DomainResult<PolicyStore> {
        let model = self
            .sqlite_repo
            .get_policy_store(Self::policy_store_id_str(id))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy_store(model)
    }

    async fn list_policy_stores(&self) -> DomainResult<Vec<PolicyStore>> {
        let models = self
            .sqlite_repo
            .list_policy_stores()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        models.into_iter().map(Self::map_policy_store).collect()
    }

    async fn update_policy_store(
        &self,
        id: &PolicyStoreId,
        name: Option<String>,
        description: Option<String>,
    ) -> DomainResult<PolicyStore> {
        let model = self
            .sqlite_repo
            .update_policy_store(Self::policy_store_id_str(id), name, description)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy_store(model)
    }

    async fn update_policy_store_tags(
        &self,
        id: &PolicyStoreId,
        tags_json: String,
    ) -> DomainResult<PolicyStore> {
        let model = self
            .sqlite_repo
            .update_policy_store_tags(Self::policy_store_id_str(id), tags_json)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy_store(model)
    }

    async fn delete_policy_store(&self, id: &PolicyStoreId) -> DomainResult<()> {
        self.sqlite_repo
            .delete_policy_store(Self::policy_store_id_str(id))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn put_schema(
        &self,
        policy_store_id: &PolicyStoreId,
        schema: String,
    ) -> DomainResult<()> {
        self.sqlite_repo
            .put_schema(Self::policy_store_id_str(policy_store_id), schema)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn get_schema(&self, policy_store_id: &PolicyStoreId) -> DomainResult<Option<Schema>> {
        match self
            .sqlite_repo
            .get_schema(Self::policy_store_id_str(policy_store_id))
            .await
        {
            Ok(model) => Self::map_schema(model).map(Some),
            Err(err) if err.to_string().contains("not found") => Ok(None),
            Err(err) => Err(DomainError::Internal(err.to_string())),
        }
    }

    async fn create_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
        statement: &CedarPolicy,
        description: Option<String>,
    ) -> DomainResult<Policy> {
        let model = self
            .sqlite_repo
            .create_policy(
                Self::policy_store_id_str(policy_store_id),
                Self::policy_id_str(policy_id),
                Self::cedar_statement(statement),
                description,
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy(model)
    }

    async fn get_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
    ) -> DomainResult<Policy> {
        let model = self
            .sqlite_repo
            .get_policy(
                Self::policy_store_id_str(policy_store_id),
                Self::policy_id_str(policy_id),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy(model)
    }

    async fn list_policies(&self, policy_store_id: &PolicyStoreId) -> DomainResult<Vec<Policy>> {
        let models = self
            .sqlite_repo
            .list_policies(Self::policy_store_id_str(policy_store_id))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        models.into_iter().map(Self::map_policy).collect()
    }

    async fn update_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
        statement: &CedarPolicy,
        description: Option<String>,
    ) -> DomainResult<Policy> {
        let model = self
            .sqlite_repo
            .update_policy(
                Self::policy_store_id_str(policy_store_id),
                Self::policy_id_str(policy_id),
                Self::cedar_statement(statement),
                description,
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy(model)
    }

    async fn delete_policy(
        &self,
        policy_store_id: &PolicyStoreId,
        policy_id: &PolicyId,
    ) -> DomainResult<()> {
        self.sqlite_repo
            .delete_policy(
                Self::policy_store_id_str(policy_store_id),
                Self::policy_id_str(policy_id),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn create_identity_source(
        &self,
        policy_store_id: &PolicyStoreId,
        configuration_type: &IdentitySourceType,
        configuration_json: String,
        claims_mapping_json: Option<String>,
        description: Option<String>,
    ) -> DomainResult<IdentitySource> {
        let model = self
            .sqlite_repo
            .create_identity_source(
                Self::policy_store_id_str(policy_store_id),
                Self::identity_source_type_str(configuration_type),
                &configuration_json,
                claims_mapping_json.as_deref(),
                description.as_deref(),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_identity_source(model)
    }

    async fn get_identity_source(
        &self,
        policy_store_id: &PolicyStoreId,
        identity_source_id: &str,
    ) -> DomainResult<IdentitySource> {
        let model = self
            .sqlite_repo
            .get_identity_source(
                Self::policy_store_id_str(policy_store_id),
                identity_source_id,
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_identity_source(model)
    }

    async fn list_identity_sources(
        &self,
        policy_store_id: &PolicyStoreId,
    ) -> DomainResult<Vec<IdentitySource>> {
        let models = self
            .sqlite_repo
            .list_identity_sources(Self::policy_store_id_str(policy_store_id))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        models.into_iter().map(Self::map_identity_source).collect()
    }

    async fn delete_identity_source(
        &self,
        policy_store_id: &PolicyStoreId,
        identity_source_id: &str,
    ) -> DomainResult<()> {
        self.sqlite_repo
            .delete_identity_source(
                Self::policy_store_id_str(policy_store_id),
                identity_source_id,
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn create_policy_template(
        &self,
        policy_store_id: &PolicyStoreId,
        template_id: String,
        statement: String,
        description: Option<String>,
    ) -> DomainResult<PolicyTemplate> {
        let model = self
            .sqlite_repo
            .create_policy_template(
                Self::policy_store_id_str(policy_store_id),
                &template_id,
                &statement,
                description.as_deref(),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy_template(model)
    }

    async fn get_policy_template(
        &self,
        policy_store_id: &PolicyStoreId,
        template_id: &str,
    ) -> DomainResult<PolicyTemplate> {
        let model = self
            .sqlite_repo
            .get_policy_template(Self::policy_store_id_str(policy_store_id), template_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Self::map_policy_template(model)
    }

    async fn list_policy_templates(
        &self,
        policy_store_id: &PolicyStoreId,
    ) -> DomainResult<Vec<PolicyTemplate>> {
        let models = self
            .sqlite_repo
            .list_policy_templates(Self::policy_store_id_str(policy_store_id))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        models.into_iter().map(Self::map_policy_template).collect()
    }

    async fn delete_policy_template(
        &self,
        policy_store_id: &PolicyStoreId,
        template_id: &str,
    ) -> DomainResult<()> {
        self.sqlite_repo
            .delete_policy_template(Self::policy_store_id_str(policy_store_id), template_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    // Snapshot / Version Control Operations
    async fn create_policy_store_snapshot(
        &self,
        policy_store_id: &PolicyStoreId,
        description: Option<String>,
    ) -> DomainResult<Snapshot> {
        let model = self
            .sqlite_repo
            .create_policy_store_snapshot(
                Self::policy_store_id_str(policy_store_id),
                description.as_deref(),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Snapshot {
            snapshot_id: model.snapshot_id,
            policy_store_id: PolicyStoreId::new(model.policy_store_id)?,
            description: model.description,
            created_at: model.created_at,
            policy_count: model.policy_count,
            has_schema: model.has_schema,
            schema_json: model.schema_json,
            policies: model
                .policies
                .into_iter()
                .map(|p| SnapshotPolicy {
                    policy_id: p.policy_id,
                    description: p.description,
                    statement: p.statement,
                })
                .collect(),
            size_bytes: model.size_bytes,
        })
    }

    async fn get_policy_store_snapshot(
        &self,
        policy_store_id: &PolicyStoreId,
        snapshot_id: &str,
    ) -> DomainResult<Snapshot> {
        let model = self
            .sqlite_repo
            .get_policy_store_snapshot(Self::policy_store_id_str(policy_store_id), snapshot_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Snapshot {
            snapshot_id: model.snapshot_id,
            policy_store_id: PolicyStoreId::new(model.policy_store_id)?,
            description: model.description,
            created_at: model.created_at,
            policy_count: model.policy_count,
            has_schema: model.has_schema,
            schema_json: model.schema_json,
            policies: model
                .policies
                .into_iter()
                .map(|p| SnapshotPolicy {
                    policy_id: p.policy_id,
                    description: p.description,
                    statement: p.statement,
                })
                .collect(),
            size_bytes: model.size_bytes,
        })
    }

    async fn list_policy_store_snapshots(
        &self,
        policy_store_id: &PolicyStoreId,
    ) -> DomainResult<Vec<Snapshot>> {
        let models = self
            .sqlite_repo
            .list_policy_store_snapshots(Self::policy_store_id_str(policy_store_id))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        models
            .into_iter()
            .map(|model| {
                Ok(Snapshot {
                    snapshot_id: model.snapshot_id,
                    policy_store_id: PolicyStoreId::new(model.policy_store_id)?,
                    description: model.description,
                    created_at: model.created_at,
                    policy_count: model.policy_count,
                    has_schema: model.has_schema,
                    schema_json: model.schema_json,
                    policies: Vec::new(), // List view doesn't include policies
                    size_bytes: model.size_bytes,
                })
            })
            .collect()
    }

    async fn rollback_to_snapshot(
        &self,
        policy_store_id: &PolicyStoreId,
        snapshot_id: &str,
        description: Option<String>,
    ) -> DomainResult<RollbackResult> {
        let model = self
            .sqlite_repo
            .rollback_to_snapshot(
                Self::policy_store_id_str(policy_store_id),
                snapshot_id,
                description.as_deref(),
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(RollbackResult {
            policy_store_id: PolicyStoreId::new(model.policy_store_id)?,
            snapshot_id: model.snapshot_id,
            rolled_back_at: model.rolled_back_at,
            policies_restored: model.policies_restored,
            schema_restored: model.schema_restored,
        })
    }

    async fn delete_snapshot(
        &self,
        policy_store_id: &PolicyStoreId,
        snapshot_id: &str,
    ) -> DomainResult<()> {
        self.sqlite_repo
            .delete_snapshot(Self::policy_store_id_str(policy_store_id), snapshot_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
