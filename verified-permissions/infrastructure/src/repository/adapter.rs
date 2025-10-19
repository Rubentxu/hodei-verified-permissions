//! Adapter that implements domain PolicyRepository trait using SQLite

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hodei_domain::{
    Action, AuthorizationDecision, AuthorizationEvaluator, AuthorizationLog, CedarPolicy,
    DomainError, DomainResult, IdentitySource, IdentitySourceType, Policy, PolicyId,
    PolicyRepository, PolicyStore, PolicyStoreId, PolicyTemplate, Principal, Resource, Schema,
};

use super::{models, SqliteRepository};

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
        Ok(PolicyStore {
            id,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

    fn to_model_authorization_log(log: AuthorizationLog) -> models::AuthorizationLog {
        models::AuthorizationLog {
            policy_store_id: log.policy_store_id.into_string(),
            principal: log.principal.to_string(),
            action: log.action.to_string(),
            resource: log.resource.to_string(),
            decision: log.decision.to_string(),
            timestamp: log.timestamp,
        }
    }

    fn to_domain_authorization_log(model: models::AuthorizationLog) -> DomainResult<AuthorizationLog> {
        Ok(AuthorizationLog {
            policy_store_id: PolicyStoreId::new(model.policy_store_id)?,
            principal: Principal::new(model.principal)?,
            action: Action::new(model.action)?,
            resource: Resource::new(model.resource)?,
            decision: match model.decision.as_str() {
                "ALLOW" => AuthorizationDecision::Allow,
                "DENY" => AuthorizationDecision::Deny,
                other => return Err(DomainError::Internal(format!("Invalid decision: {}", other))),
            },
            timestamp: model.timestamp,
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
    async fn create_policy_store(&self, description: Option<String>) -> DomainResult<PolicyStore> {
        let model = self
            .sqlite_repo
            .create_policy_store(description)
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

    async fn delete_policy_store(&self, id: &PolicyStoreId) -> DomainResult<()> {
        self
            .sqlite_repo
            .delete_policy_store(Self::policy_store_id_str(id))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn put_schema(&self, policy_store_id: &PolicyStoreId, schema: String) -> DomainResult<()> {
        self
            .sqlite_repo
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
        self
            .sqlite_repo
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
        self
            .sqlite_repo
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
            .get_policy_template(
                Self::policy_store_id_str(policy_store_id),
                template_id,
            )
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
        self
            .sqlite_repo
            .delete_policy_template(
                Self::policy_store_id_str(policy_store_id),
                template_id,
            )
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn log_authorization(&self, log: AuthorizationLog) -> DomainResult<()> {
        let model = Self::to_model_authorization_log(log);
        self
            .sqlite_repo
            .log_authorization(model)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
