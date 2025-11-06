//! Policy store use cases

use tracing::info;

use crate::dto::{CreatePolicyStoreRequest, PolicyStoreResponse};
use crate::errors::{ApplicationError, ApplicationResult};
use hodei_domain::{PolicyRepository, PolicyStoreId};

/// Create policy store use case
pub struct CreatePolicyStoreUseCase<R: PolicyRepository> {
    repository: R,
}

impl<R: PolicyRepository> CreatePolicyStoreUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        request: CreatePolicyStoreRequest,
    ) -> ApplicationResult<PolicyStoreResponse> {
        info!("Creating policy store");

        let policy_store = self
            .repository
            .create_policy_store(
                request.name.clone(),
                request.description,
                request.tags,
                request.user,
            )
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(PolicyStoreResponse {
            id: policy_store.id.into_string(),
            name: policy_store.name,
            description: policy_store.description,
            status: policy_store.status.to_string(),
            version: policy_store.version,
            author: policy_store.author,
            tags: policy_store.tags,
            identity_source_ids: policy_store.identity_source_ids,
            default_identity_source_id: policy_store.default_identity_source_id,
            created_at: policy_store.created_at,
            updated_at: policy_store.updated_at,
        })
    }
}

/// Get policy store use case
pub struct GetPolicyStoreUseCase<R: PolicyRepository> {
    repository: R,
}

impl<R: PolicyRepository> GetPolicyStoreUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: String) -> ApplicationResult<PolicyStoreResponse> {
        info!("Getting policy store: {}", id);

        let policy_store_id =
            PolicyStoreId::new(id).map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let policy_store = self
            .repository
            .get_policy_store(&policy_store_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(PolicyStoreResponse {
            id: policy_store.id.into_string(),
            name: policy_store.name,
            description: policy_store.description,
            status: policy_store.status.to_string(),
            version: policy_store.version,
            author: policy_store.author,
            tags: policy_store.tags,
            identity_source_ids: policy_store.identity_source_ids,
            default_identity_source_id: policy_store.default_identity_source_id,
            created_at: policy_store.created_at,
            updated_at: policy_store.updated_at,
        })
    }
}

/// List policy stores use case
pub struct ListPolicyStoresUseCase<R: PolicyRepository> {
    repository: R,
}

impl<R: PolicyRepository> ListPolicyStoresUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> ApplicationResult<Vec<PolicyStoreResponse>> {
        info!("Listing policy stores");

        let policy_stores = self
            .repository
            .list_policy_stores()
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(policy_stores
            .into_iter()
            .map(|ps| PolicyStoreResponse {
                id: ps.id.into_string(),
                name: ps.name,
                description: ps.description,
                status: ps.status.to_string(),
                version: ps.version,
                author: ps.author,
                tags: ps.tags,
                identity_source_ids: ps.identity_source_ids,
                default_identity_source_id: ps.default_identity_source_id,
                created_at: ps.created_at,
                updated_at: ps.updated_at,
            })
            .collect())
    }
}

/// Delete policy store use case
pub struct DeletePolicyStoreUseCase<R: PolicyRepository> {
    repository: R,
}

impl<R: PolicyRepository> DeletePolicyStoreUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: String) -> ApplicationResult<()> {
        info!("Deleting policy store: {}", id);

        let policy_store_id =
            PolicyStoreId::new(id).map_err(|e| ApplicationError::Validation(e.to_string()))?;

        self.repository
            .delete_policy_store(&policy_store_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }
}
