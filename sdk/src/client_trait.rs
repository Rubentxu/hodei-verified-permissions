//! Authorization client trait for testability and mocking

use async_trait::async_trait;
use crate::error::Result;
use crate::proto::*;

/// Trait for authorization client operations
///
/// This trait abstracts the authorization client interface, allowing for
/// easy mocking and testing of authorization logic without requiring
/// a real connection to the authorization service.
///
/// # Example
///
/// ```rust,ignore
/// use hodei_permissions_sdk::client_trait::AuthorizationClientTrait;
/// use async_trait::async_trait;
///
/// struct MockClient;
///
/// #[async_trait]
/// impl AuthorizationClientTrait for MockClient {
///     async fn is_authorized(
///         &self,
///         policy_store_id: &str,
///         principal: &str,
///         action: &str,
///         resource: &str,
///     ) -> Result<IsAuthorizedResponse> {
///         Ok(IsAuthorizedResponse {
///             decision: 0, // Allow
///             reason: vec![],
///         })
///     }
///
///     // ... implement other methods
/// }
/// ```
#[async_trait]
pub trait AuthorizationClientTrait: Send + Sync {
    /// Check if an action is authorized
    async fn is_authorized(
        &self,
        policy_store_id: &str,
        principal: &str,
        action: &str,
        resource: &str,
    ) -> Result<IsAuthorizedResponse>;

    /// Check authorization with entities and context
    async fn is_authorized_with_context(
        &self,
        request: IsAuthorizedRequest,
    ) -> Result<IsAuthorizedResponse>;

    /// Batch authorization check
    async fn batch_is_authorized(
        &self,
        policy_store_id: &str,
        requests: Vec<IsAuthorizedRequest>,
    ) -> Result<BatchIsAuthorizedResponse>;

    /// Create a new policy store
    async fn create_policy_store(
        &self,
        description: Option<String>,
    ) -> Result<CreatePolicyStoreResponse>;

    /// Get a policy store
    async fn get_policy_store(&self, policy_store_id: &str) -> Result<GetPolicyStoreResponse>;

    /// List policy stores
    async fn list_policy_stores(
        &self,
        max_results: Option<i32>,
        next_token: Option<String>,
    ) -> Result<ListPolicyStoresResponse>;

    /// Delete a policy store
    async fn delete_policy_store(&self, policy_store_id: &str) -> Result<DeletePolicyStoreResponse>;

    /// Put a schema
    async fn put_schema(
        &self,
        policy_store_id: &str,
        schema: &str,
    ) -> Result<PutSchemaResponse>;

    /// Get a schema
    async fn get_schema(&self, policy_store_id: &str) -> Result<GetSchemaResponse>;

    /// Create a policy
    async fn create_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: &str,
        description: Option<String>,
    ) -> Result<CreatePolicyResponse>;

    /// Get a policy
    async fn get_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
    ) -> Result<GetPolicyResponse>;

    /// List policies
    async fn list_policies(&self, policy_store_id: &str) -> Result<ListPoliciesResponse>;

    /// Update a policy
    async fn update_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: &str,
        description: Option<String>,
    ) -> Result<UpdatePolicyResponse>;

    /// Delete a policy
    async fn delete_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
    ) -> Result<DeletePolicyResponse>;

    /// Create an identity source
    async fn create_identity_source(
        &self,
        policy_store_id: &str,
        configuration: IdentitySourceConfiguration,
        claims_mapping: Option<ClaimsMappingConfiguration>,
        description: Option<String>,
    ) -> Result<CreateIdentitySourceResponse>;

    /// Get an identity source
    async fn get_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<GetIdentitySourceResponse>;

    /// List identity sources
    async fn list_identity_sources(
        &self,
        policy_store_id: &str,
    ) -> Result<ListIdentitySourcesResponse>;

    /// Delete an identity source
    async fn delete_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<DeleteIdentitySourceResponse>;

    /// Create a policy template
    async fn create_policy_template(
        &self,
        policy_store_id: &str,
        template_id: &str,
        statement: &str,
        description: Option<String>,
    ) -> Result<CreatePolicyTemplateResponse>;

    /// Get a policy template
    async fn get_policy_template(
        &self,
        policy_store_id: &str,
        template_id: &str,
    ) -> Result<GetPolicyTemplateResponse>;

    /// List policy templates
    async fn list_policy_templates(
        &self,
        policy_store_id: &str,
    ) -> Result<ListPolicyTemplatesResponse>;

    /// Delete a policy template
    async fn delete_policy_template(
        &self,
        policy_store_id: &str,
        template_id: &str,
    ) -> Result<DeletePolicyTemplateResponse>;

    /// Create a template-linked policy
    async fn create_policy_from_template(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        template_id: &str,
        principal: &str,
        resource: &str,
        description: Option<String>,
    ) -> Result<CreatePolicyResponse>;

    /// Check authorization with JWT token
    async fn is_authorized_with_token(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
        access_token: &str,
        action: &str,
        resource: &str,
    ) -> Result<IsAuthorizedResponse>;

    /// Check authorization with JWT token and context
    async fn is_authorized_with_token_and_context(
        &self,
        request: IsAuthorizedWithTokenRequest,
    ) -> Result<IsAuthorizedResponse>;
}
