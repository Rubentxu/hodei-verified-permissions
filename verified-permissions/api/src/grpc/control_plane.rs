//! Control Plane gRPC service implementation

use crate::proto::authorization_control_server::AuthorizationControl;
use crate::proto::*;
use hodei_infrastructure::repository::RepositoryAdapter;
use hodei_domain::{PolicyStoreId, PolicyId, CedarPolicy, IdentitySourceType, PolicyRepository};
use tonic::{Request, Response, Status};
use tracing::{info, error};
use cedar_policy::{
    Policy as CedarPolicyType, Schema, Validator, PolicySet,
    Authorizer, Context, Entities, EntityUid,
    Request as CedarRequest, Decision,
};
use std::str::FromStr;
use std::sync::Arc;
use serde_json;

pub struct AuthorizationControlService {
    repository: Arc<RepositoryAdapter>,
}

impl AuthorizationControlService {
    pub fn new(repository: Arc<RepositoryAdapter>) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl AuthorizationControl for AuthorizationControlService {
    async fn create_policy_store(
        &self,
        request: Request<CreatePolicyStoreRequest>,
    ) -> Result<Response<CreatePolicyStoreResponse>, Status> {
        let req = request.into_inner();
        info!("Creating policy store with description: {:?}", req.description);

        let store = self
            .repository
            .create_policy_store(req.description)
            .await
            .map_err(|e| {
                error!("Failed to create policy store: {}", e);
                Status::internal(format!("Failed to create policy store: {}", e))
            })?;

        Ok(Response::new(CreatePolicyStoreResponse {
            policy_store_id: store.id.into_string(),
            created_at: store.created_at.to_rfc3339(),
        }))
    }

    async fn get_policy_store(
        &self,
        request: Request<GetPolicyStoreRequest>,
    ) -> Result<Response<GetPolicyStoreResponse>, Status> {
        let req = request.into_inner();
        info!("Getting policy store: {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let store = self
            .repository
            .get_policy_store(&policy_store_id)
            .await
            .map_err(|e| {
                error!("Failed to get policy store: {}", e);
                Status::not_found(format!("Policy store not found: {}", e))
            })?;

        Ok(Response::new(GetPolicyStoreResponse {
            policy_store_id: store.id.into_string(),
            description: store.description,
            created_at: store.created_at.to_rfc3339(),
            updated_at: store.updated_at.to_rfc3339(),
        }))
    }

    async fn list_policy_stores(
        &self,
        _request: Request<ListPolicyStoresRequest>,
    ) -> Result<Response<ListPolicyStoresResponse>, Status> {
        info!("Listing policy stores");

        let stores = self
            .repository
            .list_policy_stores()
            .await
            .map_err(|e| {
                error!("Failed to list policy stores: {}", e);
                Status::internal(format!("Failed to list policy stores: {}", e))
            })?;

        let items = stores
            .into_iter()
            .map(|store| PolicyStoreItem {
                policy_store_id: store.id.into_string(),
                description: store.description,
                created_at: store.created_at.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(ListPolicyStoresResponse {
            policy_stores: items,
            next_token: None,
        }))
    }

    async fn delete_policy_store(
        &self,
        request: Request<DeletePolicyStoreRequest>,
    ) -> Result<Response<DeletePolicyStoreResponse>, Status> {
        let req = request.into_inner();
        info!("Deleting policy store: {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        self.repository
            .delete_policy_store(&policy_store_id)
            .await
            .map_err(|e| {
                error!("Failed to delete policy store: {}", e);
                Status::internal(format!("Failed to delete policy store: {}", e))
            })?;

        Ok(Response::new(DeletePolicyStoreResponse {}))
    }

    async fn put_schema(
        &self,
        request: Request<PutSchemaRequest>,
    ) -> Result<Response<PutSchemaResponse>, Status> {
        let req = request.into_inner();
        info!("Putting schema for policy store: {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id.clone())
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        // Validate schema format
        Schema::from_str(&req.schema).map_err(|e| {
            error!("Invalid schema format: {}", e);
            Status::invalid_argument(format!("Invalid schema format: {}", e))
        })?;

        self.repository
            .put_schema(&policy_store_id, req.schema)
            .await
            .map_err(|e| {
                error!("Failed to put schema: {}", e);
                Status::internal(format!("Failed to put schema: {}", e))
            })?;

        Ok(Response::new(PutSchemaResponse {
            policy_store_id: req.policy_store_id,
            namespaces: vec![],
        }))
    }

    async fn get_schema(
        &self,
        request: Request<GetSchemaRequest>,
    ) -> Result<Response<GetSchemaResponse>, Status> {
        let req = request.into_inner();
        info!("Getting schema for policy store: {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let schema = self
            .repository
            .get_schema(&policy_store_id)
            .await
            .map_err(|e| {
                error!("Failed to get schema: {}", e);
                Status::internal(format!("Failed to get schema: {}", e))
            })?
            .ok_or_else(|| Status::not_found("Schema not found"))?;

        Ok(Response::new(GetSchemaResponse {
            policy_store_id: schema.policy_store_id.into_string(),
            schema: schema.schema_json,
            created_at: schema.created_at.to_rfc3339(),
            updated_at: schema.updated_at.to_rfc3339(),
        }))
    }

    async fn create_policy(
        &self,
        request: Request<CreatePolicyRequest>,
    ) -> Result<Response<CreatePolicyResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Creating policy {} in store {}",
            req.policy_id, req.policy_store_id
        );

        let policy_store_id = PolicyStoreId::new(req.policy_store_id.clone())
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let policy_id = PolicyId::new(req.policy_id.clone())
            .map_err(|e| Status::invalid_argument(format!("Invalid policy ID: {}", e)))?;

        let definition = req.definition.ok_or_else(|| {
            Status::invalid_argument("Policy definition is required")
        })?;

        let statement = match definition.policy_type {
            Some(policy_definition::PolicyType::Static(static_policy)) => {
                // Static policy - use as-is
                static_policy.statement
            }
            Some(policy_definition::PolicyType::TemplateLinked(template_linked)) => {
                // Template-linked policy - instantiate template with values
                info!("Creating template-linked policy from template: {}", template_linked.policy_template_id);
                
                // 1. Load the template
                let template = self
                    .repository
                    .get_policy_template(&policy_store_id, &template_linked.policy_template_id)
                    .await
                    .map_err(|e| {
                        error!("Failed to load policy template: {}", e);
                        Status::not_found(format!("Policy template not found: {}", e))
                    })?;
                
                // 2. Instantiate template with principal and resource
                let mut instantiated = template.statement.clone();
                
                // Replace ?principal placeholder
                if let Some(principal) = &template_linked.principal {
                    let principal_value = format!("{}::\"{}\"", principal.entity_type, principal.entity_id);
                    instantiated = instantiated.replace("?principal", &principal_value);
                    info!("Replaced ?principal with {}", principal_value);
                }
                
                // Replace ?resource placeholder
                if let Some(resource) = &template_linked.resource {
                    let resource_value = format!("{}::\"{}\"", resource.entity_type, resource.entity_id);
                    instantiated = instantiated.replace("?resource", &resource_value);
                    info!("Replaced ?resource with {}", resource_value);
                }
                
                // 3. Verify all placeholders were replaced
                if instantiated.contains("?principal") || instantiated.contains("?resource") {
                    return Err(Status::invalid_argument(
                        "Template contains placeholders that were not provided. \
                         Please provide principal and/or resource values."
                    ));
                }
                
                info!("Template instantiated successfully");
                instantiated
            }
            None => {
                return Err(Status::invalid_argument("Policy type is required"));
            }
        };

        // Validate Cedar policy syntax
        CedarPolicyType::from_str(&statement).map_err(|e| {
            error!("Invalid policy syntax: {}", e);
            Status::invalid_argument(format!("Invalid policy syntax: {}", e))
        })?;

        let cedar_policy = CedarPolicy::new(statement)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy: {}", e)))?;

        let policy = self
            .repository
            .create_policy(&policy_store_id, &policy_id, &cedar_policy, req.description)
            .await
            .map_err(|e| {
                error!("Failed to create policy: {}", e);
                Status::internal(format!("Failed to create policy: {}", e))
            })?;

        Ok(Response::new(CreatePolicyResponse {
            policy_store_id: policy.policy_store_id.into_string(),
            policy_id: policy.policy_id.into_string(),
            created_at: policy.created_at.to_rfc3339(),
        }))
    }

    async fn get_policy(
        &self,
        request: Request<GetPolicyRequest>,
    ) -> Result<Response<GetPolicyResponse>, Status> {
        let req = request.into_inner();
        info!("Getting policy {} from store {}", req.policy_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let policy_id = PolicyId::new(req.policy_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy ID: {}", e)))?;

        let policy = self
            .repository
            .get_policy(&policy_store_id, &policy_id)
            .await
            .map_err(|e| {
                error!("Failed to get policy: {}", e);
                Status::not_found(format!("Policy not found: {}", e))
            })?;

        Ok(Response::new(GetPolicyResponse {
            policy_store_id: policy.policy_store_id.into_string(),
            policy_id: policy.policy_id.into_string(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(StaticPolicy {
                    statement: policy.statement.into_string(),
                })),
            }),
            description: policy.description,
            created_at: policy.created_at.to_rfc3339(),
            updated_at: policy.updated_at.to_rfc3339(),
        }))
    }

    async fn update_policy(
        &self,
        request: Request<UpdatePolicyRequest>,
    ) -> Result<Response<UpdatePolicyResponse>, Status> {
        let req = request.into_inner();
        info!("Updating policy {} in store {}", req.policy_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let policy_id = PolicyId::new(req.policy_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy ID: {}", e)))?;

        let definition = req.definition.ok_or_else(|| {
            Status::invalid_argument("Policy definition is required")
        })?;

        let statement = match definition.policy_type {
            Some(policy_definition::PolicyType::Static(static_policy)) => {
                static_policy.statement
            }
            Some(policy_definition::PolicyType::TemplateLinked(template_linked)) => {
                // Template-linked policy - instantiate template with values
                info!("Updating with template-linked policy from template: {}", template_linked.policy_template_id);
                
                // Load and instantiate template
                let template = self
                    .repository
                    .get_policy_template(&policy_store_id, &template_linked.policy_template_id)
                    .await
                    .map_err(|e| {
                        error!("Failed to load policy template: {}", e);
                        Status::not_found(format!("Policy template not found: {}", e))
                    })?;
                
                let mut instantiated = template.statement.clone();
                
                if let Some(principal) = &template_linked.principal {
                    let principal_value = format!("{}::\"{}\"", principal.entity_type, principal.entity_id);
                    instantiated = instantiated.replace("?principal", &principal_value);
                }
                
                if let Some(resource) = &template_linked.resource {
                    let resource_value = format!("{}::\"{}\"", resource.entity_type, resource.entity_id);
                    instantiated = instantiated.replace("?resource", &resource_value);
                }
                
                if instantiated.contains("?principal") || instantiated.contains("?resource") {
                    return Err(Status::invalid_argument(
                        "Template contains placeholders that were not provided"
                    ));
                }
                
                instantiated
            }
            None => {
                return Err(Status::invalid_argument("Policy type is required"));
            }
        };

        // Validate Cedar policy syntax
        CedarPolicyType::from_str(&statement).map_err(|e| {
            error!("Invalid policy syntax: {}", e);
            Status::invalid_argument(format!("Invalid policy syntax: {}", e))
        })?;

        let cedar_policy = CedarPolicy::new(statement)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy: {}", e)))?;

        let policy = self
            .repository
            .update_policy(&policy_store_id, &policy_id, &cedar_policy, req.description)
            .await
            .map_err(|e| {
                error!("Failed to update policy: {}", e);
                Status::internal(format!("Failed to update policy: {}", e))
            })?;

        Ok(Response::new(UpdatePolicyResponse {
            policy_store_id: policy.policy_store_id.into_string(),
            policy_id: policy.policy_id.into_string(),
            updated_at: policy.updated_at.to_rfc3339(),
        }))
    }

    async fn delete_policy(
        &self,
        request: Request<DeletePolicyRequest>,
    ) -> Result<Response<DeletePolicyResponse>, Status> {
        let req = request.into_inner();
        info!("Deleting policy {} from store {}", req.policy_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let policy_id = PolicyId::new(req.policy_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy ID: {}", e)))?;

        self.repository
            .delete_policy(&policy_store_id, &policy_id)
            .await
            .map_err(|e| {
                error!("Failed to delete policy: {}", e);
                Status::internal(format!("Failed to delete policy: {}", e))
            })?;

        Ok(Response::new(DeletePolicyResponse {}))
    }

    async fn list_policies(
        &self,
        request: Request<ListPoliciesRequest>,
    ) -> Result<Response<ListPoliciesResponse>, Status> {
        let req = request.into_inner();
        info!("Listing policies for store {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let policies = self
            .repository
            .list_policies(&policy_store_id)
            .await
            .map_err(|e| {
                error!("Failed to list policies: {}", e);
                Status::internal(format!("Failed to list policies: {}", e))
            })?;

        let items = policies
            .into_iter()
            .map(|policy| PolicyItem {
                policy_id: policy.policy_id.into_string(),
                description: policy.description,
                created_at: policy.created_at.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(ListPoliciesResponse {
            policies: items,
            next_token: None,
        }))
    }

    // ========================================================================
    // Identity Source Management (Épica 4 - HU 4.1)
    // ========================================================================

    async fn create_identity_source(
        &self,
        request: Request<CreateIdentitySourceRequest>,
    ) -> Result<Response<CreateIdentitySourceResponse>, Status> {
        let req = request.into_inner();
        info!("Creating identity source for policy store: {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let config = req.configuration.ok_or_else(|| {
            Status::invalid_argument("Configuration is required")
        })?;

        let (config_type, config_json) = match config.configuration_type {
            Some(identity_source_configuration::ConfigurationType::Oidc(oidc)) => {
                let json = serde_json::json!({
                    "issuer": oidc.issuer,
                    "client_ids": oidc.client_ids,
                    "jwks_uri": oidc.jwks_uri,
                    "group_claim": oidc.group_claim,
                });
                (IdentitySourceType::Oidc, json.to_string())
            }
            Some(identity_source_configuration::ConfigurationType::CognitoUserPool(cognito)) => {
                let json = serde_json::json!({
                    "user_pool_arn": cognito.user_pool_arn,
                    "client_ids": cognito.client_ids,
                    "group_configuration_group_claim": cognito.group_configuration_group_claim,
                });
                (IdentitySourceType::Cognito, json.to_string())
            }
            None => {
                return Err(Status::invalid_argument("Configuration type is required"));
            }
        };

        let claims_mapping_json = req.claims_mapping.map(|mapping| {
            serde_json::json!({
                "principal_id_claim": mapping.principal_id_claim,
                "group_claim": mapping.group_claim,
                "attribute_mappings": mapping.attribute_mappings,
            }).to_string()
        });

        let identity_source = self
            .repository
            .create_identity_source(
                &policy_store_id,
                &config_type,
                config_json,
                claims_mapping_json,
                req.description,
            )
            .await
            .map_err(|e| {
                error!("Failed to create identity source: {}", e);
                Status::internal(format!("Failed to create identity source: {}", e))
            })?;

        Ok(Response::new(CreateIdentitySourceResponse {
            identity_source_id: identity_source.id,
            created_at: identity_source.created_at.to_rfc3339(),
        }))
    }

    async fn get_identity_source(
        &self,
        request: Request<GetIdentitySourceRequest>,
    ) -> Result<Response<GetIdentitySourceResponse>, Status> {
        let req = request.into_inner();
        info!("Getting identity source: {} from policy store: {}", req.identity_source_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let identity_source = self
            .repository
            .get_identity_source(&policy_store_id, &req.identity_source_id)
            .await
            .map_err(|e| {
                error!("Failed to get identity source: {}", e);
                Status::not_found(format!("Identity source not found: {}", e))
            })?;

        // Deserialize configuration
        let config_json: serde_json::Value = serde_json::from_str(&identity_source.configuration_json)
            .map_err(|e| Status::internal(format!("Failed to parse config: {}", e)))?;

        let configuration_type = match identity_source.configuration_type {
            IdentitySourceType::Oidc => {
                let oidc = OidcConfiguration {
                    issuer: config_json["issuer"].as_str().unwrap_or_default().to_string(),
                    client_ids: config_json["client_ids"].as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default(),
                    jwks_uri: config_json["jwks_uri"].as_str().unwrap_or_default().to_string(),
                    group_claim: config_json["group_claim"].as_str().unwrap_or_default().to_string(),
                };
                Some(identity_source_configuration::ConfigurationType::Oidc(oidc))
            }
            IdentitySourceType::Cognito => {
                let cognito = CognitoUserPoolConfiguration {
                    user_pool_arn: config_json["user_pool_arn"].as_str().unwrap_or_default().to_string(),
                    client_ids: config_json["client_ids"].as_str().unwrap_or_default().to_string(),
                    group_configuration_group_claim: config_json["group_configuration_group_claim"].as_str().unwrap_or_default().to_string(),
                };
                Some(identity_source_configuration::ConfigurationType::CognitoUserPool(cognito))
            }
        };

        let claims_mapping = identity_source.claims_mapping_json.as_ref().and_then(|json| {
            serde_json::from_str::<serde_json::Value>(json).ok().map(|val| {
                ClaimsMappingConfiguration {
                    principal_id_claim: val["principal_id_claim"].as_str().unwrap_or_default().to_string(),
                    group_claim: val["group_claim"].as_str().unwrap_or_default().to_string(),
                    attribute_mappings: val["attribute_mappings"].as_object()
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string())).collect())
                        .unwrap_or_default(),
                }
            })
        });

        Ok(Response::new(GetIdentitySourceResponse {
            identity_source_id: identity_source.id,
            policy_store_id: identity_source.policy_store_id.into_string(),
            configuration: Some(IdentitySourceConfiguration {
                configuration_type,
            }),
            claims_mapping,
            description: identity_source.description,
            created_at: identity_source.created_at.to_rfc3339(),
            updated_at: identity_source.updated_at.to_rfc3339(),
        }))
    }

    async fn list_identity_sources(
        &self,
        request: Request<ListIdentitySourcesRequest>,
    ) -> Result<Response<ListIdentitySourcesResponse>, Status> {
        let req = request.into_inner();
        info!("Listing identity sources for policy store: {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let identity_sources = self
            .repository
            .list_identity_sources(&policy_store_id)
            .await
            .map_err(|e| {
                error!("Failed to list identity sources: {}", e);
                Status::internal(format!("Failed to list identity sources: {}", e))
            })?;

        let items = identity_sources
            .into_iter()
            .map(|source| IdentitySourceItem {
                identity_source_id: source.id,
                policy_store_id: source.policy_store_id.into_string(),
                description: source.description,
                created_at: source.created_at.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(ListIdentitySourcesResponse {
            identity_sources: items,
            next_token: None,
        }))
    }

    async fn delete_identity_source(
        &self,
        request: Request<DeleteIdentitySourceRequest>,
    ) -> Result<Response<DeleteIdentitySourceResponse>, Status> {
        let req = request.into_inner();
        info!("Deleting identity source: {} from policy store: {}", req.identity_source_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        self.repository
            .delete_identity_source(&policy_store_id, &req.identity_source_id)
            .await
            .map_err(|e| {
                error!("Failed to delete identity source: {}", e);
                Status::internal(format!("Failed to delete identity source: {}", e))
            })?;

        Ok(Response::new(DeleteIdentitySourceResponse {
            identity_source_id: req.identity_source_id,
        }))
    }

    // ========================================================================
    // Policy Template Management (Épica 6 - HU 6.1)
    // ========================================================================

    async fn create_policy_template(
        &self,
        request: Request<CreatePolicyTemplateRequest>,
    ) -> Result<Response<CreatePolicyTemplateResponse>, Status> {
        let req = request.into_inner();
        info!("Creating policy template: {} for policy store: {}", req.template_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let template = self
            .repository
            .create_policy_template(
                &policy_store_id,
                req.template_id.clone(),
                req.statement,
                req.description,
            )
            .await
            .map_err(|e| {
                error!("Failed to create policy template: {}", e);
                Status::internal(format!("Failed to create policy template: {}", e))
            })?;

        Ok(Response::new(CreatePolicyTemplateResponse {
            template_id: template.template_id,
            created_at: template.created_at.to_rfc3339(),
        }))
    }

    async fn get_policy_template(
        &self,
        request: Request<GetPolicyTemplateRequest>,
    ) -> Result<Response<GetPolicyTemplateResponse>, Status> {
        let req = request.into_inner();
        info!("Getting policy template: {} from policy store: {}", req.template_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let template = self
            .repository
            .get_policy_template(&policy_store_id, &req.template_id)
            .await
            .map_err(|e| {
                error!("Failed to get policy template: {}", e);
                Status::not_found(format!("Policy template not found: {}", e))
            })?;

        Ok(Response::new(GetPolicyTemplateResponse {
            template_id: template.template_id,
            policy_store_id: template.policy_store_id.into_string(),
            statement: template.statement,
            description: template.description,
            created_at: template.created_at.to_rfc3339(),
            updated_at: template.updated_at.to_rfc3339(),
        }))
    }

    async fn list_policy_templates(
        &self,
        request: Request<ListPolicyTemplatesRequest>,
    ) -> Result<Response<ListPolicyTemplatesResponse>, Status> {
        let req = request.into_inner();
        info!("Listing policy templates for policy store: {}", req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let templates = self
            .repository
            .list_policy_templates(&policy_store_id)
            .await
            .map_err(|e| {
                error!("Failed to list policy templates: {}", e);
                Status::internal(format!("Failed to list policy templates: {}", e))
            })?;

        let items = templates
            .into_iter()
            .map(|template| PolicyTemplateItem {
                template_id: template.template_id,
                policy_store_id: template.policy_store_id.into_string(),
                description: template.description,
                created_at: template.created_at.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(ListPolicyTemplatesResponse {
            templates: items,
            next_token: None,
        }))
    }

    async fn delete_policy_template(
        &self,
        request: Request<DeletePolicyTemplateRequest>,
    ) -> Result<Response<DeletePolicyTemplateResponse>, Status> {
        let req = request.into_inner();
        info!("Deleting policy template: {} from policy store: {}", req.template_id, req.policy_store_id);

        let policy_store_id = PolicyStoreId::new(req.policy_store_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        self.repository
            .delete_policy_template(&policy_store_id, &req.template_id)
            .await
            .map_err(|e| {
                error!("Failed to delete policy template: {}", e);
                Status::internal(format!("Failed to delete policy template: {}", e))
            })?;

        Ok(Response::new(DeletePolicyTemplateResponse {
            template_id: req.template_id,
        }))
    }

    // ========================================================================
    // Playground / Testing Endpoints
    // ========================================================================

    async fn test_authorization(
        &self,
        request: Request<TestAuthorizationRequest>,
    ) -> Result<Response<TestAuthorizationResponse>, Status> {
        let req = request.into_inner();
        info!("Testing authorization in playground mode");

        // 1. Get or parse schema
        let schema = if let Some(policy_store_id) = &req.policy_store_id {
            // Load schema from policy store
            let store_id = PolicyStoreId::new(policy_store_id.clone())
                .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;
            
            let schema_model = self.repository
                .get_schema(&store_id)
                .await
                .map_err(|e| {
                    error!("Failed to load schema: {}", e);
                    Status::not_found(format!("Schema not found: {}", e))
                })?
                .ok_or_else(|| Status::not_found("Schema not found"))?;
            
            Some(Schema::from_str(&schema_model.schema_json).map_err(|e| {
                Status::invalid_argument(format!("Invalid schema: {}", e))
            })?)
        } else if let Some(schema_str) = &req.schema {
            // Parse provided schema
            Some(Schema::from_str(schema_str).map_err(|e| {
                Status::invalid_argument(format!("Invalid schema: {}", e))
            })?)
        } else {
            None
        };

        // 2. Parse and validate policies
        let mut policy_set_str = String::new();
        let mut validation_errors = Vec::new();
        let mut validation_warnings = Vec::new();

        for (idx, policy_str) in req.policies.iter().enumerate() {
            // Parse policy
            match CedarPolicyType::from_str(policy_str) {
                Ok(policy) => {
                    // Validate against schema if available
                    if let Some(ref schema) = schema {
                        let validator = Validator::new(schema.clone());
                        let policy_set = PolicySet::from_str(policy_str).unwrap_or_default();
                        let validation_result = validator.validate(&policy_set, cedar_policy::ValidationMode::default());
                        
                        // Collect validation issues
                        for error in validation_result.validation_errors() {
                            validation_errors.push(ValidationIssue {
                                severity: validation_issue::Severity::Error as i32,
                                message: error.to_string(),
                                location: Some(format!("Policy {}", idx)),
                                issue_type: "ValidationError".to_string(),
                            });
                        }
                        
                        for warning in validation_result.validation_warnings() {
                            validation_warnings.push(ValidationIssue {
                                severity: validation_issue::Severity::Warning as i32,
                                message: warning.to_string(),
                                location: Some(format!("Policy {}", idx)),
                                issue_type: "ValidationWarning".to_string(),
                            });
                        }
                    }
                    
                    // Add to policy set
                    policy_set_str.push_str(&format!("@id(\"test-policy-{}\")\n{}\n\n", idx, policy_str));
                }
                Err(e) => {
                    return Err(Status::invalid_argument(format!("Invalid policy {}: {}", idx, e)));
                }
            }
        }

        // 3. Build Cedar request
        let principal_uid = EntityUid::from_str(&format!(
            "{}::\"{}\"",
            req.principal.as_ref().ok_or_else(|| Status::invalid_argument("Principal required"))?.entity_type,
            req.principal.as_ref().unwrap().entity_id
        )).map_err(|e| Status::invalid_argument(format!("Invalid principal: {}", e)))?;

        let action_uid = EntityUid::from_str(&format!(
            "{}::\"{}\"",
            req.action.as_ref().ok_or_else(|| Status::invalid_argument("Action required"))?.entity_type,
            req.action.as_ref().unwrap().entity_id
        )).map_err(|e| Status::invalid_argument(format!("Invalid action: {}", e)))?;

        let resource_uid = EntityUid::from_str(&format!(
            "{}::\"{}\"",
            req.resource.as_ref().ok_or_else(|| Status::invalid_argument("Resource required"))?.entity_type,
            req.resource.as_ref().unwrap().entity_id
        )).map_err(|e| Status::invalid_argument(format!("Invalid resource: {}", e)))?;

        // 4. Build context
        let context = if let Some(context_str) = &req.context {
            Context::from_json_str(context_str, None).map_err(|e| {
                Status::invalid_argument(format!("Invalid context: {}", e))
            })?
        } else {
            Context::empty()
        };

        // 5. Build entities
        let mut entities_json = serde_json::json!([]);
        if let Some(entities_array) = entities_json.as_array_mut() {
            for entity in &req.entities {
                let entity_json = serde_json::json!({
                    "uid": {
                        "type": entity.identifier.as_ref().unwrap().entity_type,
                        "id": entity.identifier.as_ref().unwrap().entity_id
                    },
                    "attrs": entity.attributes,
                    "parents": entity.parents.iter().map(|p| {
                        serde_json::json!({
                            "type": p.entity_type,
                            "id": p.entity_id
                        })
                    }).collect::<Vec<_>>()
                });
                entities_array.push(entity_json);
            }
        }

        let entities = Entities::from_json_str(&entities_json.to_string(), None)
            .map_err(|e| Status::invalid_argument(format!("Invalid entities: {}", e)))?;

        // 6. Create Cedar request
        let cedar_request = CedarRequest::new(
            principal_uid,
            action_uid,
            resource_uid,
            context,
            schema.as_ref(),
        ).map_err(|e| Status::internal(format!("Failed to create request: {}", e)))?;

        // 7. Evaluate
        let policy_set = PolicySet::from_str(&policy_set_str)
            .map_err(|e| Status::internal(format!("Failed to create policy set: {}", e)))?;

        let authorizer = Authorizer::new();
        let response = authorizer.is_authorized(&cedar_request, &policy_set, &entities);

        // 8. Build response
        let decision = match response.decision() {
            Decision::Allow => Decision::Allow as i32,
            Decision::Deny => Decision::Deny as i32,
        };

        let determining_policies: Vec<String> = response
            .diagnostics()
            .reason()
            .map(|id| id.to_string())
            .collect();

        let errors: Vec<String> = response
            .diagnostics()
            .errors()
            .map(|e| e.to_string())
            .collect();

        Ok(Response::new(TestAuthorizationResponse {
            decision,
            determining_policies,
            errors,
            validation_warnings,
            validation_errors,
        }))
    }

    async fn validate_policy(
        &self,
        request: Request<ValidatePolicyRequest>,
    ) -> Result<Response<ValidatePolicyResponse>, Status> {
        let req = request.into_inner();
        info!("Validating policy in playground mode");

        // 1. Get or parse schema
        let schema = if let Some(policy_store_id) = &req.policy_store_id {
            // Load schema from policy store
            let store_id = PolicyStoreId::new(policy_store_id.clone())
                .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;
            
            let schema_model = self.repository
                .get_schema(&store_id)
                .await
                .map_err(|e| {
                    error!("Failed to load schema: {}", e);
                    Status::not_found(format!("Schema not found: {}", e))
                })?
                .ok_or_else(|| Status::not_found("Schema not found"))?;
            
            Schema::from_str(&schema_model.schema_json).map_err(|e| {
                Status::invalid_argument(format!("Invalid schema: {}", e))
            })?
        } else if let Some(schema_str) = &req.schema {
            // Parse provided schema
            Schema::from_str(schema_str).map_err(|e| {
                Status::invalid_argument(format!("Invalid schema: {}", e))
            })?
        } else {
            return Err(Status::invalid_argument("Either policy_store_id or schema must be provided"));
        };

        // 2. Parse policy
        let policy = match CedarPolicyType::from_str(&req.policy_statement) {
            Ok(p) => p,
            Err(e) => {
                return Ok(Response::new(ValidatePolicyResponse {
                    is_valid: false,
                    errors: vec![ValidationIssue {
                        severity: validation_issue::Severity::Error as i32,
                        message: format!("Syntax error: {}", e),
                        location: None,
                        issue_type: "SyntaxError".to_string(),
                    }],
                    warnings: vec![],
                    policy_info: None,
                }));
            }
        };

        // 3. Validate against schema
        let validator = Validator::new(schema);
        let policy_set = PolicySet::from_str(&req.policy_statement).unwrap_or_default();
        let validation_result = validator.validate(&policy_set, cedar_policy::ValidationMode::default());

        // 4. Collect errors and warnings
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for error in validation_result.validation_errors() {
            errors.push(ValidationIssue {
                severity: validation_issue::Severity::Error as i32,
                message: error.to_string(),
                location: None,
                issue_type: "ValidationError".to_string(),
            });
        }

        for warning in validation_result.validation_warnings() {
            warnings.push(ValidationIssue {
                severity: validation_issue::Severity::Warning as i32,
                message: warning.to_string(),
                location: None,
                issue_type: "ValidationWarning".to_string(),
            });
        }

        let is_valid = errors.is_empty();

        // 5. Extract policy info if valid
        let policy_info = if is_valid {
            let policy_str = policy.to_string();
            Some(PolicyInfo {
                effect: if policy_str.starts_with("permit") { "permit".to_string() } else { "forbid".to_string() },
                principal_scope: Some("principal".to_string()),
                action_scope: Some("action".to_string()),
                resource_scope: Some("resource".to_string()),
                has_conditions: policy_str.contains("when") || policy_str.contains("unless"),
            })
        } else {
            None
        };

        Ok(Response::new(ValidatePolicyResponse {
            is_valid,
            errors,
            warnings,
            policy_info,
        }))
    }
}
