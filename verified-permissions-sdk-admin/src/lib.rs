//! SDK Admin Library - Programmatic Control Plane API

pub mod error;

use tonic::transport::{Channel, Endpoint};
use tracing::info;
use verified_permissions_sdk::proto::{
    BatchCreatePoliciesRequest, BatchCreatePoliciesResponse, BatchDeletePoliciesRequest,
    BatchDeletePoliciesResponse, BatchIsAuthorizedRequest, BatchIsAuthorizedResponse,
    BatchPolicyItem, BatchUpdatePoliciesRequest, BatchUpdatePoliciesResponse, CreatePolicyRequest,
    CreatePolicyResponse, CreatePolicyStoreRequest, CreatePolicyStoreResponse, DeletePolicyRequest,
    DeletePolicyResponse, DeletePolicyStoreRequest, DeletePolicyStoreResponse, EntityIdentifier,
    GetPolicyRequest, GetPolicyResponse, GetPolicyStoreRequest, GetPolicyStoreResponse,
    IsAuthorizedRequest, ListPoliciesRequest, ListPoliciesResponse, ListPolicyStoresRequest,
    ListPolicyStoresResponse, PolicyDefinition, PutSchemaRequest, PutSchemaResponse, StaticPolicy,
    TestAuthorizationRequest, TestAuthorizationResponse, UpdatePolicyRequest, UpdatePolicyResponse,
    ValidatePolicyRequest, ValidatePolicyResponse,
    authorization_control_client::AuthorizationControlClient,
    authorization_data_client::AuthorizationDataClient,
};

pub use error::{Result, SdkAdminError};

/// Programmatic client for Hodei Verified Permissions Control Plane
#[derive(Clone)]
pub struct HodeiAdmin {
    control_client: AuthorizationControlClient<Channel>,
    data_client: AuthorizationDataClient<Channel>,
}

impl HodeiAdmin {
    /// Connect to Hodei Verified Permissions service
    pub async fn connect(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint_str = endpoint.into();
        let channel = Endpoint::from_shared(endpoint_str)
            .map_err(|e| SdkAdminError::InvalidRequest(format!("Invalid endpoint: {}", e)))?
            .connect()
            .await
            .map_err(|_| SdkAdminError::ConnectionFailed)?;

        Ok(HodeiAdmin {
            control_client: AuthorizationControlClient::new(channel.clone()),
            data_client: AuthorizationDataClient::new(channel),
        })
    }

    /// Create a policy definition from a statement
    fn create_policy_definition(statement: impl Into<String>) -> PolicyDefinition {
        let statement_str = statement.into();
        let static_policy = StaticPolicy {
            statement: statement_str,
        };
        PolicyDefinition {
            policy_type: Some(
                hodei_permissions_sdk::proto::policy_definition::PolicyType::Static(static_policy),
            ),
        }
    }

    /// Create a new policy store
    pub async fn create_policy_store(
        &mut self,
        name: impl Into<String>,
        description: Option<String>,
    ) -> Result<CreatePolicyStoreResponse> {
        let request = CreatePolicyStoreRequest {
            name: name.into(),
            description,
        };

        info!("Creating policy store");

        let response = self
            .control_client
            .create_policy_store(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Get a policy store by ID
    pub async fn get_policy_store(
        &mut self,
        policy_store_id: impl Into<String>,
    ) -> Result<GetPolicyStoreResponse> {
        let request = GetPolicyStoreRequest {
            policy_store_id: policy_store_id.into(),
        };

        let response = self
            .control_client
            .get_policy_store(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// List all policy stores
    pub async fn list_policy_stores(
        &mut self,
        max_results: Option<i32>,
        next_token: Option<String>,
    ) -> Result<ListPolicyStoresResponse> {
        let request = ListPolicyStoresRequest {
            max_results,
            next_token,
        };

        let response = self
            .control_client
            .list_policy_stores(request)
            .await
            .map_err(SdkAdminError::from)?;

        let inner = response.into_inner();
        info!("Found {} policy stores", inner.policy_stores.len());

        Ok(inner)
    }

    /// Delete a policy store
    pub async fn delete_policy_store(
        &mut self,
        policy_store_id: impl Into<String>,
    ) -> Result<DeletePolicyStoreResponse> {
        let request = DeletePolicyStoreRequest {
            policy_store_id: policy_store_id.into(),
        };

        info!("Deleting policy store");

        let response = self
            .control_client
            .delete_policy_store(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Upload or update a schema
    pub async fn put_schema(
        &mut self,
        policy_store_id: impl Into<String>,
        schema: impl Into<String>,
    ) -> Result<PutSchemaResponse> {
        let request = PutSchemaRequest {
            policy_store_id: policy_store_id.into(),
            schema: schema.into(),
        };

        info!("Uploading schema");

        let response = self
            .control_client
            .put_schema(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Create a new policy
    pub async fn create_policy(
        &mut self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
        statement: impl Into<String>,
        description: Option<String>,
    ) -> Result<CreatePolicyResponse> {
        let statement_str = statement.into();

        // Create PolicyDefinition with StaticPolicy
        let static_policy = hodei_permissions_sdk::proto::StaticPolicy {
            statement: statement_str,
        };

        let definition = hodei_permissions_sdk::proto::PolicyDefinition {
            policy_type: Some(
                hodei_permissions_sdk::proto::policy_definition::PolicyType::Static(static_policy),
            ),
        };

        let request = CreatePolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
            definition: Some(definition),
            description,
        };

        info!("Creating policy");

        let response = self
            .control_client
            .create_policy(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Get a policy by ID
    pub async fn get_policy(
        &mut self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
    ) -> Result<GetPolicyResponse> {
        let request = GetPolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
        };

        let response = self
            .control_client
            .get_policy(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// List all policies in a policy store
    pub async fn list_policies(
        &mut self,
        policy_store_id: impl Into<String>,
    ) -> Result<ListPoliciesResponse> {
        let request = ListPoliciesRequest {
            policy_store_id: policy_store_id.into(),
            max_results: None,
            next_token: None,
        };

        let response = self
            .control_client
            .list_policies(request)
            .await
            .map_err(SdkAdminError::from)?;

        let inner = response.into_inner();
        info!("Found {} policies", inner.policies.len());

        Ok(inner)
    }

    /// Update an existing policy
    pub async fn update_policy(
        &mut self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
        statement: impl Into<String>,
        description: Option<String>,
    ) -> Result<UpdatePolicyResponse> {
        let statement_str = statement.into();

        // Create PolicyDefinition with StaticPolicy
        let static_policy = hodei_permissions_sdk::proto::StaticPolicy {
            statement: statement_str,
        };

        let definition = hodei_permissions_sdk::proto::PolicyDefinition {
            policy_type: Some(
                hodei_permissions_sdk::proto::policy_definition::PolicyType::Static(static_policy),
            ),
        };

        let request = UpdatePolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
            definition: Some(definition),
            description,
        };

        info!("Updating policy");

        let response = self
            .control_client
            .update_policy(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Delete a policy
    pub async fn delete_policy(
        &mut self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
    ) -> Result<DeletePolicyResponse> {
        let request = DeletePolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
        };

        info!("Deleting policy");

        let response = self
            .control_client
            .delete_policy(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    // =========================================================================
    // Bulk Operations
    // =========================================================================

    /// Create multiple policies in a single batch operation
    pub async fn batch_create_policies(
        &mut self,
        policy_store_id: impl Into<String>,
        policies: Vec<(impl Into<String>, impl Into<String>, Option<String>)>,
    ) -> Result<BatchCreatePoliciesResponse> {
        let policy_store_id = policy_store_id.into();

        let batch_items: Vec<BatchPolicyItem> = policies
            .into_iter()
            .map(|(policy_id, statement, description)| BatchPolicyItem {
                policy_id: policy_id.into(),
                definition: Some(Self::create_policy_definition(statement)),
                description,
            })
            .collect();

        let request = BatchCreatePoliciesRequest {
            policy_store_id,
            policies: batch_items,
        };

        info!("Batch creating {} policies", request.policies.len());

        let response = self
            .control_client
            .batch_create_policies(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Update multiple policies in a single batch operation
    pub async fn batch_update_policies(
        &mut self,
        policy_store_id: impl Into<String>,
        policies: Vec<(impl Into<String>, impl Into<String>, Option<String>)>,
    ) -> Result<BatchUpdatePoliciesResponse> {
        let policy_store_id = policy_store_id.into();

        let batch_items: Vec<BatchPolicyItem> = policies
            .into_iter()
            .map(|(policy_id, statement, description)| BatchPolicyItem {
                policy_id: policy_id.into(),
                definition: Some(Self::create_policy_definition(statement)),
                description,
            })
            .collect();

        let request = BatchUpdatePoliciesRequest {
            policy_store_id,
            policies: batch_items,
        };

        info!("Batch updating {} policies", request.policies.len());

        let response = self
            .control_client
            .batch_update_policies(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Delete multiple policies in a single batch operation
    pub async fn batch_delete_policies(
        &mut self,
        policy_store_id: impl Into<String>,
        policy_ids: Vec<impl Into<String>>,
    ) -> Result<BatchDeletePoliciesResponse> {
        let policy_store_id = policy_store_id.into();
        let policy_ids: Vec<String> = policy_ids.into_iter().map(|id| id.into()).collect();

        let request = BatchDeletePoliciesRequest {
            policy_store_id,
            policy_ids,
        };

        info!("Batch deleting {} policies", request.policy_ids.len());

        let response = self
            .control_client
            .batch_delete_policies(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Test authorization without persisting policies (Playground mode)
    pub async fn test_authorization(
        &mut self,
        policies: Vec<String>,
        principal: EntityIdentifier,
        action: EntityIdentifier,
        resource: EntityIdentifier,
        context: Option<String>,
    ) -> Result<TestAuthorizationResponse> {
        let request = TestAuthorizationRequest {
            policy_store_id: None,
            schema: None,
            policies,
            principal: Some(principal),
            action: Some(action),
            resource: Some(resource),
            context,
            entities: vec![],
        };

        info!(
            "Testing authorization with {} policies",
            request.policies.len()
        );

        let response = self
            .control_client
            .test_authorization(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Validate a policy statement against a schema
    pub async fn validate_policy(
        &mut self,
        policy_store_id: Option<String>,
        schema: Option<String>,
        policy_statement: String,
    ) -> Result<ValidatePolicyResponse> {
        let request = ValidatePolicyRequest {
            policy_store_id,
            schema,
            policy_statement,
        };

        info!("Validating policy statement");

        let response = self
            .control_client
            .validate_policy(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }

    /// Check authorization for multiple requests in a single batch operation
    pub async fn batch_is_authorized(
        &mut self,
        policy_store_id: impl Into<String>,
        requests: Vec<IsAuthorizedRequest>,
    ) -> Result<BatchIsAuthorizedResponse> {
        let request = BatchIsAuthorizedRequest {
            policy_store_id: policy_store_id.into(),
            requests,
        };

        info!(
            "Batch checking authorization for {} requests",
            request.requests.len()
        );

        let response = self
            .data_client
            .batch_is_authorized(request)
            .await
            .map_err(SdkAdminError::from)?;

        Ok(response.into_inner())
    }
}
