//! Authorization client

use crate::error::{Result, SdkError};
use crate::proto::authorization_control_client::AuthorizationControlClient;
use crate::proto::authorization_data_client::AuthorizationDataClient;
use crate::proto::*;
use tonic::transport::Channel;

/// Main client for Hodei Verified Permissions
#[derive(Clone)]
pub struct AuthorizationClient {
    data_client: AuthorizationDataClient<Channel>,
    control_client: AuthorizationControlClient<Channel>,
}

impl AuthorizationClient {
    /// Connect to the authorization service
    pub async fn connect(addr: impl Into<String>) -> Result<Self> {
        let addr = addr.into();
        let channel = Channel::from_shared(addr.clone())
            .map_err(|e| SdkError::ConnectionError(e.to_string()))?
            .connect()
            .await?;

        Ok(Self {
            data_client: AuthorizationDataClient::new(channel.clone()),
            control_client: AuthorizationControlClient::new(channel),
        })
    }

    // ========================================================================
    // Data Plane - Authorization
    // ========================================================================

    /// Check if an action is authorized
    pub async fn is_authorized(
        &self,
        policy_store_id: impl Into<String>,
        principal: impl Into<String>,
        action: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<IsAuthorizedResponse> {
        let request = IsAuthorizedRequest {
            policy_store_id: policy_store_id.into(),
            principal: Some(parse_entity_id(principal.into())?),
            action: Some(parse_entity_id(action.into())?),
            resource: Some(parse_entity_id(resource.into())?),
            context: None,
            entities: vec![],
        };

        let response = self
            .data_client
            .clone()
            .is_authorized(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Check authorization with entities and context
    pub async fn is_authorized_with_context(
        &self,
        request: IsAuthorizedRequest,
    ) -> Result<IsAuthorizedResponse> {
        let response = self
            .data_client
            .clone()
            .is_authorized(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Batch authorization check
    pub async fn batch_is_authorized(
        &self,
        policy_store_id: impl Into<String>,
        requests: Vec<IsAuthorizedRequest>,
    ) -> Result<BatchIsAuthorizedResponse> {
        let request = BatchIsAuthorizedRequest {
            policy_store_id: policy_store_id.into(),
            requests,
        };

        let response = self
            .data_client
            .clone()
            .batch_is_authorized(request)
            .await?
            .into_inner();

        Ok(response)
    }

    // ========================================================================
    // Control Plane - Policy Store
    // ========================================================================

    /// Create a new policy store
    pub async fn create_policy_store(
        &self,
        description: Option<String>,
    ) -> Result<CreatePolicyStoreResponse> {
        let request = CreatePolicyStoreRequest { description };

        let response = self
            .control_client
            .clone()
            .create_policy_store(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Get a policy store
    pub async fn get_policy_store(
        &self,
        policy_store_id: impl Into<String>,
    ) -> Result<GetPolicyStoreResponse> {
        let request = GetPolicyStoreRequest {
            policy_store_id: policy_store_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .get_policy_store(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// List policy stores
    pub async fn list_policy_stores(
        &self,
        max_results: Option<i32>,
        next_token: Option<String>,
    ) -> Result<ListPolicyStoresResponse> {
        let request = ListPolicyStoresRequest {
            max_results,
            next_token,
        };

        let response = self
            .control_client
            .clone()
            .list_policy_stores(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Delete a policy store
    pub async fn delete_policy_store(
        &self,
        policy_store_id: impl Into<String>,
    ) -> Result<DeletePolicyStoreResponse> {
        let request = DeletePolicyStoreRequest {
            policy_store_id: policy_store_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .delete_policy_store(request)
            .await?
            .into_inner();

        Ok(response)
    }

    // ========================================================================
    // Control Plane - Schema
    // ========================================================================

    /// Put a schema
    pub async fn put_schema(
        &self,
        policy_store_id: impl Into<String>,
        schema: impl Into<String>,
    ) -> Result<PutSchemaResponse> {
        let request = PutSchemaRequest {
            policy_store_id: policy_store_id.into(),
            schema: schema.into(),
        };

        let response = self
            .control_client
            .clone()
            .put_schema(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Get a schema
    pub async fn get_schema(
        &self,
        policy_store_id: impl Into<String>,
    ) -> Result<GetSchemaResponse> {
        let request = GetSchemaRequest {
            policy_store_id: policy_store_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .get_schema(request)
            .await?
            .into_inner();

        Ok(response)
    }

    // ========================================================================
    // Control Plane - Policy
    // ========================================================================

    /// Create a policy
    pub async fn create_policy(
        &self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
        statement: impl Into<String>,
        description: Option<String>,
    ) -> Result<CreatePolicyResponse> {
        let request = CreatePolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(StaticPolicy {
                    statement: statement.into(),
                })),
            }),
            description,
        };

        let response = self
            .control_client
            .clone()
            .create_policy(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Get a policy
    pub async fn get_policy(
        &self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
    ) -> Result<GetPolicyResponse> {
        let request = GetPolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .get_policy(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// List policies
    pub async fn list_policies(
        &self,
        policy_store_id: impl Into<String>,
    ) -> Result<ListPoliciesResponse> {
        let request = ListPoliciesRequest {
            policy_store_id: policy_store_id.into(),
            max_results: None,
            next_token: None,
        };

        let response = self
            .control_client
            .clone()
            .list_policies(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Update a policy
    pub async fn update_policy(
        &self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
        statement: impl Into<String>,
        description: Option<String>,
    ) -> Result<UpdatePolicyResponse> {
        let request = UpdatePolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(StaticPolicy {
                    statement: statement.into(),
                })),
            }),
            description,
        };

        let response = self
            .control_client
            .clone()
            .update_policy(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Delete a policy
    pub async fn delete_policy(
        &self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
    ) -> Result<DeletePolicyResponse> {
        let request = DeletePolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .delete_policy(request)
            .await?
            .into_inner();

        Ok(response)
    }

    // ========================================================================
    // Control Plane - Identity Source
    // ========================================================================

    /// Create an identity source
    pub async fn create_identity_source(
        &self,
        policy_store_id: impl Into<String>,
        configuration: IdentitySourceConfiguration,
        claims_mapping: Option<ClaimsMappingConfiguration>,
        description: Option<String>,
    ) -> Result<CreateIdentitySourceResponse> {
        let request = CreateIdentitySourceRequest {
            policy_store_id: policy_store_id.into(),
            configuration: Some(configuration),
            claims_mapping,
            description,
        };

        let response = self
            .control_client
            .clone()
            .create_identity_source(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Get an identity source
    pub async fn get_identity_source(
        &self,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
    ) -> Result<GetIdentitySourceResponse> {
        let request = GetIdentitySourceRequest {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .get_identity_source(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// List identity sources
    pub async fn list_identity_sources(
        &self,
        policy_store_id: impl Into<String>,
    ) -> Result<ListIdentitySourcesResponse> {
        let request = ListIdentitySourcesRequest {
            policy_store_id: policy_store_id.into(),
            max_results: None,
            next_token: None,
        };

        let response = self
            .control_client
            .clone()
            .list_identity_sources(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Delete an identity source
    pub async fn delete_identity_source(
        &self,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
    ) -> Result<DeleteIdentitySourceResponse> {
        let request = DeleteIdentitySourceRequest {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .delete_identity_source(request)
            .await?
            .into_inner();

        Ok(response)
    }

    // ========================================================================
    // Control Plane - Policy Template
    // ========================================================================

    /// Create a policy template
    pub async fn create_policy_template(
        &self,
        policy_store_id: impl Into<String>,
        template_id: impl Into<String>,
        statement: impl Into<String>,
        description: Option<String>,
    ) -> Result<CreatePolicyTemplateResponse> {
        let request = CreatePolicyTemplateRequest {
            policy_store_id: policy_store_id.into(),
            template_id: template_id.into(),
            statement: statement.into(),
            description,
        };

        let response = self
            .control_client
            .clone()
            .create_policy_template(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Get a policy template
    pub async fn get_policy_template(
        &self,
        policy_store_id: impl Into<String>,
        template_id: impl Into<String>,
    ) -> Result<GetPolicyTemplateResponse> {
        let request = GetPolicyTemplateRequest {
            policy_store_id: policy_store_id.into(),
            template_id: template_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .get_policy_template(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// List policy templates
    pub async fn list_policy_templates(
        &self,
        policy_store_id: impl Into<String>,
    ) -> Result<ListPolicyTemplatesResponse> {
        let request = ListPolicyTemplatesRequest {
            policy_store_id: policy_store_id.into(),
            max_results: None,
            next_token: None,
        };

        let response = self
            .control_client
            .clone()
            .list_policy_templates(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Delete a policy template
    pub async fn delete_policy_template(
        &self,
        policy_store_id: impl Into<String>,
        template_id: impl Into<String>,
    ) -> Result<DeletePolicyTemplateResponse> {
        let request = DeletePolicyTemplateRequest {
            policy_store_id: policy_store_id.into(),
            template_id: template_id.into(),
        };

        let response = self
            .control_client
            .clone()
            .delete_policy_template(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Create a template-linked policy
    pub async fn create_policy_from_template(
        &self,
        policy_store_id: impl Into<String>,
        policy_id: impl Into<String>,
        template_id: impl Into<String>,
        principal: impl Into<String>,
        resource: impl Into<String>,
        description: Option<String>,
    ) -> Result<CreatePolicyResponse> {
        let request = CreatePolicyRequest {
            policy_store_id: policy_store_id.into(),
            policy_id: policy_id.into(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::TemplateLinked(
                    TemplateLinkedPolicy {
                        policy_template_id: template_id.into(),
                        principal: Some(parse_entity_id(principal.into())?),
                        resource: Some(parse_entity_id(resource.into())?),
                    },
                )),
            }),
            description,
        };

        let response = self
            .control_client
            .clone()
            .create_policy(request)
            .await?
            .into_inner();

        Ok(response)
    }

    // ========================================================================
    // Data Plane - Authorization with Token
    // ========================================================================

    /// Check authorization with JWT token
    pub async fn is_authorized_with_token(
        &self,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
        access_token: impl Into<String>,
        action: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<IsAuthorizedResponse> {
        let request = IsAuthorizedWithTokenRequest {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            access_token: access_token.into(),
            action: Some(parse_entity_id(action.into())?),
            resource: Some(parse_entity_id(resource.into())?),
            context: None,
            entities: vec![],
        };

        let response = self
            .data_client
            .clone()
            .is_authorized_with_token(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Check authorization with JWT token and context
    pub async fn is_authorized_with_token_and_context(
        &self,
        request: IsAuthorizedWithTokenRequest,
    ) -> Result<IsAuthorizedResponse> {
        let response = self
            .data_client
            .clone()
            .is_authorized_with_token(request)
            .await?
            .into_inner();

        Ok(response)
    }
}

/// Parse entity ID from string format "Type::id"
fn parse_entity_id(s: String) -> Result<EntityIdentifier> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() != 2 {
        return Err(SdkError::InvalidRequest(format!(
            "Invalid entity format: {}. Expected 'Type::id'",
            s
        )));
    }

    Ok(EntityIdentifier {
        entity_type: parts[0].trim_matches('"').to_string(),
        entity_id: parts[1].trim_matches('"').to_string(),
    })
}
