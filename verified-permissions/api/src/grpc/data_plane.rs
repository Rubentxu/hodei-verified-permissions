//! Data Plane gRPC service implementation

use crate::audit::{AuditEvent, AuditLogger};
use crate::error::AuthorizationError;
use crate::proto::authorization_data_server::AuthorizationData;
use crate::proto::*;
use crate::storage::Repository;
use cedar_policy::{Authorizer, Context, Entities, EntityUid, PolicySet, Request as CedarRequest};
use chrono::Utc;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};
use uuid::Uuid;

pub struct AuthorizationDataService {
    repository: Repository,
    audit_logger: Arc<AuditLogger>,
}

impl AuthorizationDataService {
    pub fn new(repository: Repository) -> Self {
        Self {
            repository,
            audit_logger: Arc::new(AuditLogger::new()),
        }
    }
    
    pub fn with_audit_logger(repository: Repository, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            repository,
            audit_logger,
        }
    }

    fn build_entity_uid(identifier: &EntityIdentifier) -> Result<EntityUid, Status> {
        EntityUid::from_str(&format!("{}::\"{}\"", identifier.entity_type, identifier.entity_id))
            .map_err(|e| {
                error!("Failed to parse entity UID: {}", e);
                Status::invalid_argument(format!("Invalid entity identifier: {}", e))
            })
    }

    fn build_entities(entities: &[Entity]) -> Result<Entities, Status> {
        let mut entities_json = Vec::new();

        for entity in entities {
            let uid = Self::build_entity_uid(entity.identifier.as_ref().ok_or_else(|| {
                Status::invalid_argument("Entity identifier is required")
            })?)?;

            let mut entity_obj = serde_json::json!({
                "uid": uid.to_string(),
                "attrs": {},
                "parents": []
            });

            // Add attributes
            if !entity.attributes.is_empty() {
                let mut attrs = HashMap::new();
                for (key, value) in &entity.attributes {
                    // Parse JSON value
                    let parsed_value: serde_json::Value = serde_json::from_str(value)
                        .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
                    attrs.insert(key.clone(), parsed_value);
                }
                entity_obj["attrs"] = serde_json::json!(attrs);
            }

            // Add parents
            if !entity.parents.is_empty() {
                let parents: Result<Vec<String>, Status> = entity
                    .parents
                    .iter()
                    .map(|p| Ok(Self::build_entity_uid(p)?.to_string()))
                    .collect();
                entity_obj["parents"] = serde_json::json!(parents?);
            }

            entities_json.push(entity_obj);
        }

        Entities::from_json_value(serde_json::Value::Array(entities_json), None).map_err(|e| {
            error!("Failed to build entities: {}", e);
            Status::invalid_argument(format!("Invalid entities: {}", e))
        })
    }

    fn build_context(context_json: Option<&str>) -> Result<Context, Status> {
        if let Some(json_str) = context_json {
            let value: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
                error!("Failed to parse context JSON: {}", e);
                Status::invalid_argument(format!("Invalid context JSON: {}", e))
            })?;

            Context::from_json_value(value, None).map_err(|e| {
                error!("Failed to build context: {}", e);
                Status::invalid_argument(format!("Invalid context: {}", e))
            })
        } else {
            Ok(Context::empty())
        }
    }
}

#[tonic::async_trait]
impl AuthorizationData for AuthorizationDataService {
    async fn is_authorized(
        &self,
        request: Request<IsAuthorizedRequest>,
    ) -> Result<Response<IsAuthorizedResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Authorization request for policy store: {}",
            req.policy_store_id
        );

        // Load policies from database
        let policies = self
            .repository
            .list_policies(&req.policy_store_id)
            .await
            .map_err(Status::from)?;

        if policies.is_empty() {
            info!("No policies found for policy store: {}", req.policy_store_id);
            return Ok(Response::new(IsAuthorizedResponse {
                decision: Decision::Deny as i32,
                determining_policies: vec![],
                errors: vec!["No policies found in policy store".to_string()],
            }));
        }

        // Build policy set
        let mut policy_set_str = String::new();
        for policy in &policies {
            policy_set_str.push_str(&policy.statement);
            policy_set_str.push('\n');
        }

        let policy_set = PolicySet::from_str(&policy_set_str).map_err(|e| {
            error!("Failed to parse policy set: {}", e);
            Status::from(AuthorizationError::EvaluationError(e.to_string()))
        })?;

        // Build Cedar request
        let principal = Self::build_entity_uid(req.principal.as_ref().ok_or_else(|| {
            Status::invalid_argument("Principal is required")
        })?)?;

        let action = Self::build_entity_uid(req.action.as_ref().ok_or_else(|| {
            Status::invalid_argument("Action is required")
        })?)?;

        let resource = Self::build_entity_uid(req.resource.as_ref().ok_or_else(|| {
            Status::invalid_argument("Resource is required")
        })?)?;

        let context = Self::build_context(req.context.as_deref())?;
        let entities = Self::build_entities(&req.entities)?;

        let cedar_request = CedarRequest::new(principal, action, resource, context, None)
            .map_err(|e| {
                error!("Failed to create Cedar request: {}", e);
                Status::from(AuthorizationError::EvaluationError(e.to_string()))
            })?;

        // Evaluate
        let authorizer = Authorizer::new();
        let response = authorizer.is_authorized(&cedar_request, &policy_set, &entities);

        let decision = match response.decision() {
            cedar_policy::Decision::Allow => Decision::Allow,
            cedar_policy::Decision::Deny => Decision::Deny,
        };

        let determining_policies: Vec<String> = response
            .diagnostics()
            .reason()
            .map(|policy_id| policy_id.to_string())
            .collect();

        let errors: Vec<String> = response
            .diagnostics()
            .errors()
            .map(|err| err.to_string())
            .collect();

        info!(
            "Authorization decision: {:?}, determining policies: {:?}",
            decision, determining_policies
        );

        // Audit log the decision
        let audit_event = AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            policy_store_id: req.policy_store_id.clone(),
            principal: format!("{}::{}", req.principal.as_ref().unwrap().entity_type, req.principal.as_ref().unwrap().entity_id),
            action: format!("{}::{}", req.action.as_ref().unwrap().entity_type, req.action.as_ref().unwrap().entity_id),
            resource: format!("{}::{}", req.resource.as_ref().unwrap().entity_type, req.resource.as_ref().unwrap().entity_id),
            decision: format!("{:?}", decision),
            determining_policies: determining_policies.clone(),
            errors: errors.clone(),
            context: req.context.clone(),
            entity_count: req.entities.len(),
            request_type: "is_authorized".to_string(),
            identity_source_id: None,
        };
        
        self.audit_logger.log_decision(audit_event).await;

        Ok(Response::new(IsAuthorizedResponse {
            decision: decision as i32,
            determining_policies,
            errors,
        }))
    }

    async fn batch_is_authorized(
        &self,
        request: Request<BatchIsAuthorizedRequest>,
    ) -> Result<Response<BatchIsAuthorizedResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Batch authorization request with {} requests",
            req.requests.len()
        );

        let mut responses = Vec::new();

        for auth_request in req.requests {
            // Wrap in a Request for the is_authorized method
            let result = self
                .is_authorized(Request::new(auth_request))
                .await;

            match result {
                Ok(response) => responses.push(response.into_inner()),
                Err(status) => {
                    // Convert error to a deny response
                    responses.push(IsAuthorizedResponse {
                        decision: Decision::Deny as i32,
                        determining_policies: vec![],
                        errors: vec![status.message().to_string()],
                    });
                }
            }
        }

        Ok(Response::new(BatchIsAuthorizedResponse { responses }))
    }

    async fn is_authorized_with_token(
        &self,
        request: Request<IsAuthorizedWithTokenRequest>,
    ) -> Result<Response<IsAuthorizedResponse>, Status> {
        use crate::jwt::{ClaimsMapper, JwtValidator};
        use crate::jwt::claims_mapper::ClaimsMappingConfig;
        
        let req = request.into_inner();
        info!(
            "Authorization with token request for policy store: {}",
            req.policy_store_id
        );

        // 1. Get identity source from database
        let identity_source = self
            .repository
            .get_identity_source(&req.policy_store_id, &req.identity_source_id)
            .await
            .map_err(Status::from)?;

        // 2. Parse identity source configuration
        let config_json: serde_json::Value = serde_json::from_str(&identity_source.configuration_json)
            .map_err(|e| Status::internal(format!("Failed to parse identity source config: {}", e)))?;

        let (issuer, audiences, jwks_uri) = match identity_source.configuration_type.as_str() {
            "oidc" => {
                let issuer = config_json["issuer"].as_str()
                    .ok_or_else(|| Status::internal("Missing issuer in OIDC config"))?;
                let jwks_uri = config_json["jwks_uri"].as_str()
                    .ok_or_else(|| Status::internal("Missing jwks_uri in OIDC config"))?;
                let client_ids = config_json["client_ids"].as_array()
                    .ok_or_else(|| Status::internal("Missing client_ids in OIDC config"))?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>();
                (issuer.to_string(), client_ids, jwks_uri.to_string())
            }
            "cognito" => {
                // For Cognito, construct issuer and JWKS URI from user pool ARN
                let user_pool_arn = config_json["user_pool_arn"].as_str()
                    .ok_or_else(|| Status::internal("Missing user_pool_arn in Cognito config"))?;
                
                // Extract region and pool ID from ARN
                // Format: arn:aws:cognito-idp:REGION:ACCOUNT:userpool/POOL_ID
                let parts: Vec<&str> = user_pool_arn.split(':').collect();
                if parts.len() < 6 {
                    return Err(Status::internal("Invalid Cognito user pool ARN format"));
                }
                let region = parts[3];
                let pool_id = parts[5].split('/').last()
                    .ok_or_else(|| Status::internal("Invalid pool ID in ARN"))?;
                
                let issuer = format!("https://cognito-idp.{}.amazonaws.com/{}", region, pool_id);
                let jwks_uri = format!("{}/.well-known/jwks.json", issuer);
                let client_ids = config_json["client_ids"].as_str()
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                
                (issuer, client_ids, jwks_uri)
            }
            _ => return Err(Status::internal("Unknown identity source type")),
        };

        // 3. Validate JWT token
        let validator = JwtValidator::new();
        let claims = validator
            .validate_token(&req.access_token, &issuer, &audiences, &jwks_uri)
            .await
            .map_err(|e| Status::unauthenticated(format!("Token validation failed: {}", e)))?;

        // 4. Parse claims mapping configuration
        let claims_mapping_config = if let Some(mapping_json) = &identity_source.claims_mapping_json {
            let mapping_val: serde_json::Value = serde_json::from_str(mapping_json)
                .map_err(|e| Status::internal(format!("Failed to parse claims mapping: {}", e)))?;
            
            let mut attribute_mappings = std::collections::HashMap::new();
            if let Some(attrs) = mapping_val["attribute_mappings"].as_object() {
                for (k, v) in attrs {
                    if let Some(v_str) = v.as_str() {
                        attribute_mappings.insert(k.clone(), v_str.to_string());
                    }
                }
            }
            
            ClaimsMappingConfig {
                principal_id_claim: mapping_val["principal_id_claim"]
                    .as_str()
                    .unwrap_or("sub")
                    .to_string(),
                group_claim: mapping_val["group_claim"]
                    .as_str()
                    .map(String::from),
                attribute_mappings,
            }
        } else {
            ClaimsMappingConfig::default()
        };

        // 5. Map claims to Cedar principal
        let (principal, mut principal_entities) = ClaimsMapper::map_to_principal(
            &claims,
            &claims_mapping_config,
            "User", // Default principal type
        )
        .map_err(|e| Status::internal(format!("Failed to map claims: {}", e)))?;

        // 6. Combine with provided entities
        let mut all_entities = req.entities.clone();
        all_entities.append(&mut principal_entities);

        // 7. Call standard is_authorized
        let auth_request = IsAuthorizedRequest {
            policy_store_id: req.policy_store_id,
            principal: Some(principal),
            action: req.action,
            resource: req.resource,
            context: req.context,
            entities: all_entities,
        };

        self.is_authorized(Request::new(auth_request)).await
    }
}
