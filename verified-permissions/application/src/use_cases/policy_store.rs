//! Policy store use cases

use tracing::info;

use hodei_domain::{PolicyRepository, PolicyStoreId};
use crate::dto::{CreatePolicyStoreRequest, PolicyStoreResponse};
use crate::errors::{ApplicationError, ApplicationResult};

/// Create policy store use case
pub struct CreatePolicyStoreUseCase<R: PolicyRepository> {
    repository: R,
}

impl<R: PolicyRepository> CreatePolicyStoreUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, request: CreatePolicyStoreRequest) -> ApplicationResult<PolicyStoreResponse> {
        info!("Creating policy store");

        let policy_store = self.repository.create_policy_store(request.description).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(PolicyStoreResponse {
            id: policy_store.id.into_string(),
            description: policy_store.description,
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

        let policy_store_id = PolicyStoreId::new(id)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let policy_store = self.repository.get_policy_store(&policy_store_id).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(PolicyStoreResponse {
            id: policy_store.id.into_string(),
            description: policy_store.description,
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

        let policy_stores = self.repository.list_policy_stores().await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(policy_stores.into_iter().map(|ps| PolicyStoreResponse {
            id: ps.id.into_string(),
            description: ps.description,
            created_at: ps.created_at,
            updated_at: ps.updated_at,
        }).collect())
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

        let policy_store_id = PolicyStoreId::new(id)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        self.repository.delete_policy_store(&policy_store_id).await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }
}
