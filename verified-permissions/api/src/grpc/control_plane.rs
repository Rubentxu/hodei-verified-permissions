//! Control Plane gRPC service implementation

use crate::proto::authorization_control_server::AuthorizationControl;
use crate::proto::*;
use hodei_domain::{PolicyRepository, PolicyStoreId, PolicyId, CedarPolicy, IdentitySourceType};
use cedar_policy::Schema;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct AuthorizationControlService {
    repository: Arc<dyn PolicyRepository>,
}

impl AuthorizationControlService {
    pub fn new(repository: Arc<dyn PolicyRepository>) -> Self {
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
                Status::from(e)
            })?;

        Ok(Response::new(CreatePolicyStoreResponse {
            policy_store_id: store.id,
            created_at: store.created_at.to_rfc3339(),
        }))
    }

    async fn get_policy_store(
        &self,
        request: Request<GetPolicyStoreRequest>,
    ) -> Result<Response<GetPolicyStoreResponse>, Status> {
        let req = request.into_inner();
        info!("Getting policy store: {}", req.policy_store_id);

        let store = self
            .repository
            .get_policy_store(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(GetPolicyStoreResponse {
            policy_store_id: store.id,
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
            .map_err(Status::from)?;

        let items = stores
            .into_iter()
            .map(|store| PolicyStoreItem {
                policy_store_id: store.id,
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

        self.repository
            .delete_policy_store(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(DeletePolicyStoreResponse {}))
    }

    async fn put_schema(
        &self,
        request: Request<PutSchemaRequest>,
    ) -> Result<Response<PutSchemaResponse>, Status> {
        let req = request.into_inner();
        info!("Putting schema for policy store: {}", req.policy_store_id);

        // Validate schema format
        let _schema = Schema::from_str(&req.schema).map_err(|e| {
            error!("Invalid schema format: {}", e);
            Status::from(AuthorizationError::InvalidSchema(e.to_string()))
        })?;

        // For MVP, we'll return empty namespaces list
        let namespaces: Vec<String> = vec![];

        self.repository
            .put_schema(&req.policy_store_id, req.schema)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(PutSchemaResponse {
            policy_store_id: req.policy_store_id,
            namespaces,
        }))
    }

    async fn get_schema(
        &self,
        request: Request<GetSchemaRequest>,
    ) -> Result<Response<GetSchemaResponse>, Status> {
        let req = request.into_inner();
        info!("Getting schema for policy store: {}", req.policy_store_id);

        let schema = self
            .repository
            .get_schema(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(GetSchemaResponse {
            policy_store_id: schema.policy_store_id,
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

        let definition = req.definition.ok_or_else(|| {
            Status::invalid_argument("Policy definition is required")
        })?;

        let statement = match definition.policy_type {
            Some(policy_definition::PolicyType::Static(static_policy)) => static_policy.statement,
            Some(policy_definition::PolicyType::TemplateLinked(template_linked)) => {
                // HU 6.2: Template-linked policies
                info!("Creating template-linked policy from template: {}", template_linked.policy_template_id);
                
                // Get template
                let template = self.repository
                    .get_policy_template(&req.policy_store_id, &template_linked.policy_template_id)
                    .await
                    .map_err(Status::from)?;
                
                // Validate that principal and resource are provided if needed
                let principal = template_linked.principal
                    .ok_or_else(|| Status::invalid_argument("Principal is required for template-linked policy"))?;
                let resource = template_linked.resource
                    .ok_or_else(|| Status::invalid_argument("Resource is required for template-linked policy"))?;
                
                // Replace placeholders in template
                let mut statement = template.statement.clone();
                statement = statement.replace(
                    "?principal",
                    &format!("{}::\"{}\"", principal.entity_type, principal.entity_id)
                );
                statement = statement.replace(
                    "?resource",
                    &format!("{}::\"{}\"", resource.entity_type, resource.entity_id)
                );
                
                statement
            }
            None => {
                return Err(Status::invalid_argument("Policy type is required"));
            }
        };

        // Parse policy to validate syntax
        cedar_policy::Policy::from_str(&statement).map_err(|e| {
            error!("Invalid policy syntax: {}", e);
            Status::from(AuthorizationError::InvalidPolicy(e.to_string()))
        })?;

        // TODO: Implement full schema validation
        // For MVP, we validate syntax only. Full schema validation requires
        // converting between cedar_policy and cedar_policy_core types.
        if let Ok(_schema_model) = self.repository.get_schema(&req.policy_store_id).await {
            info!("Schema validation will be implemented in future iteration");
        }

        let policy = self
            .repository
            .create_policy(&req.policy_store_id, &req.policy_id, statement, req.description)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(CreatePolicyResponse {
            policy_store_id: policy.policy_store_id,
            policy_id: policy.policy_id,
            created_at: policy.created_at.to_rfc3339(),
        }))
    }

    async fn get_policy(
        &self,
        request: Request<GetPolicyRequest>,
    ) -> Result<Response<GetPolicyResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Getting policy {} from store {}",
            req.policy_id, req.policy_store_id
        );

        let policy = self
            .repository
            .get_policy(&req.policy_store_id, &req.policy_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(GetPolicyResponse {
            policy_store_id: policy.policy_store_id,
            policy_id: policy.policy_id,
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(StaticPolicy {
                    statement: policy.statement,
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
        info!(
            "Updating policy {} in store {}",
            req.policy_id, req.policy_store_id
        );

        let definition = req.definition.ok_or_else(|| {
            Status::invalid_argument("Policy definition is required")
        })?;

        let statement = match definition.policy_type {
            Some(policy_definition::PolicyType::Static(static_policy)) => static_policy.statement,
            Some(policy_definition::PolicyType::TemplateLinked(_)) => {
                return Err(Status::unimplemented("Template-linked policies not yet supported"));
            }
            None => {
                return Err(Status::invalid_argument("Policy type is required"));
            }
        };

        // Validate policy syntax and against schema (same as create)
        cedar_policy::Policy::from_str(&statement).map_err(|e| {
            error!("Invalid policy syntax: {}", e);
            Status::from(AuthorizationError::InvalidPolicy(e.to_string()))
        })?;

        // TODO: Implement full schema validation
        // For MVP, we validate syntax only
        if let Ok(_schema_model) = self.repository.get_schema(&req.policy_store_id).await {
            info!("Schema validation will be implemented in future iteration");
        }

        let policy = self
            .repository
            .update_policy(&req.policy_store_id, &req.policy_id, statement, req.description)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(UpdatePolicyResponse {
            policy_store_id: policy.policy_store_id,
            policy_id: policy.policy_id,
            updated_at: policy.updated_at.to_rfc3339(),
        }))
    }

    async fn delete_policy(
        &self,
        request: Request<DeletePolicyRequest>,
    ) -> Result<Response<DeletePolicyResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Deleting policy {} from store {}",
            req.policy_id, req.policy_store_id
        );

        self.repository
            .delete_policy(&req.policy_store_id, &req.policy_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(DeletePolicyResponse {}))
    }

    async fn list_policies(
        &self,
        request: Request<ListPoliciesRequest>,
    ) -> Result<Response<ListPoliciesResponse>, Status> {
        let req = request.into_inner();
        info!("Listing policies for store {}", req.policy_store_id);

        let policies = self
            .repository
            .list_policies(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        let items = policies
            .into_iter()
            .map(|policy| PolicyItem {
                policy_id: policy.policy_id,
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

        // Verify policy store exists
        self.repository
            .get_policy_store(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        // Extract configuration type and serialize to JSON
        let configuration = req.configuration.ok_or_else(|| {
            Status::invalid_argument("Configuration is required")
        })?;

        let (config_type, config_json) = match configuration.configuration_type {
            Some(config) => match config {
                crate::proto::identity_source_configuration::ConfigurationType::CognitoUserPool(cognito) => {
                    let json = format!(
                        r#"{{"user_pool_arn":"{}","client_ids":"{}","group_configuration_group_claim":"{}"}}"#,
                        cognito.user_pool_arn, cognito.client_ids, cognito.group_configuration_group_claim
                    );
                    ("cognito", json)
                },
                crate::proto::identity_source_configuration::ConfigurationType::Oidc(oidc) => {
                    let client_ids_json = oidc.client_ids.iter()
                        .map(|id| format!(r#""{}""#, id))
                        .collect::<Vec<_>>()
                        .join(",");
                    let json = format!(
                        r#"{{"issuer":"{}","client_ids":[{}],"jwks_uri":"{}","group_claim":"{}"}}"#,
                        oidc.issuer, client_ids_json, oidc.jwks_uri, oidc.group_claim
                    );
                    ("oidc", json)
                },
            },
            None => return Err(Status::invalid_argument("Configuration type is required")),
        };

        // Serialize claims mapping if present
        let claims_mapping_json = req.claims_mapping.map(|mapping| {
            format!(
                r#"{{"principal_id_claim":"{}","group_claim":"{}","attribute_mappings":{}}}"#,
                mapping.principal_id_claim,
                mapping.group_claim,
                serde_json::to_string(&mapping.attribute_mappings).unwrap_or_else(|_| "{}".to_string())
            )
        });

        // Create in database
        let identity_source = self.repository
            .create_identity_source(
                &req.policy_store_id,
                config_type,
                &config_json,
                claims_mapping_json.as_deref(),
                req.description.as_deref(),
            )
            .await
            .map_err(Status::from)?;

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

        let identity_source = self.repository
            .get_identity_source(&req.policy_store_id, &req.identity_source_id)
            .await
            .map_err(Status::from)?;

        // Deserialize configuration (simplified - just parse JSON values)
        let configuration_type = match identity_source.configuration_type.as_str() {
            "cognito" => {
                let json_val: serde_json::Value = serde_json::from_str(&identity_source.configuration_json)
                    .map_err(|e| Status::internal(format!("Failed to parse Cognito config: {}", e)))?;
                let cognito = CognitoUserPoolConfiguration {
                    user_pool_arn: json_val["user_pool_arn"].as_str().unwrap_or_default().to_string(),
                    client_ids: json_val["client_ids"].as_str().unwrap_or_default().to_string(),
                    group_configuration_group_claim: json_val["group_configuration_group_claim"].as_str().unwrap_or_default().to_string(),
                };
                Some(crate::proto::identity_source_configuration::ConfigurationType::CognitoUserPool(cognito))
            },
            "oidc" => {
                let json_val: serde_json::Value = serde_json::from_str(&identity_source.configuration_json)
                    .map_err(|e| Status::internal(format!("Failed to parse OIDC config: {}", e)))?;
                let client_ids = json_val["client_ids"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let oidc = OidcConfiguration {
                    issuer: json_val["issuer"].as_str().unwrap_or_default().to_string(),
                    client_ids,
                    jwks_uri: json_val["jwks_uri"].as_str().unwrap_or_default().to_string(),
                    group_claim: json_val["group_claim"].as_str().unwrap_or_default().to_string(),
                };
                Some(crate::proto::identity_source_configuration::ConfigurationType::Oidc(oidc))
            },
            _ => None,
        };

        // Deserialize claims mapping if present
        let claims_mapping = identity_source.claims_mapping_json
            .as_ref()
            .and_then(|json| {
                serde_json::from_str::<serde_json::Value>(json).ok().map(|json_val| {
                    ClaimsMappingConfiguration {
                        principal_id_claim: json_val["principal_id_claim"].as_str().unwrap_or_default().to_string(),
                        group_claim: json_val["group_claim"].as_str().unwrap_or_default().to_string(),
                        attribute_mappings: json_val["attribute_mappings"].as_object()
                            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string())).collect())
                            .unwrap_or_default(),
                    }
                })
            });

        Ok(Response::new(GetIdentitySourceResponse {
            identity_source_id: identity_source.id,
            policy_store_id: identity_source.policy_store_id,
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

        let identity_sources = self.repository
            .list_identity_sources(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        let items: Vec<IdentitySourceItem> = identity_sources
            .into_iter()
            .map(|source| IdentitySourceItem {
                identity_source_id: source.id,
                policy_store_id: source.policy_store_id,
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

        self.repository
            .delete_identity_source(&req.policy_store_id, &req.identity_source_id)
            .await
            .map_err(Status::from)?;

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

        // Verify policy store exists
        self.repository
            .get_policy_store(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        // Create template
        let template = self.repository
            .create_policy_template(
                &req.policy_store_id,
                &req.template_id,
                &req.statement,
                req.description.as_deref(),
            )
            .await
            .map_err(Status::from)?;

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

        let template = self.repository
            .get_policy_template(&req.policy_store_id, &req.template_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(GetPolicyTemplateResponse {
            template_id: template.template_id,
            policy_store_id: template.policy_store_id,
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

        let templates = self.repository
            .list_policy_templates(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        let items: Vec<PolicyTemplateItem> = templates
            .into_iter()
            .map(|template| PolicyTemplateItem {
                template_id: template.template_id,
                policy_store_id: template.policy_store_id,
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

        self.repository
            .delete_policy_template(&req.policy_store_id, &req.template_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(DeletePolicyTemplateResponse {
            template_id: req.template_id,
        }))
    }
}
