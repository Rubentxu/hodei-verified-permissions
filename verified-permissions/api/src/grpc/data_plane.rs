//! Data Plane gRPC service implementation

use crate::proto::authorization_data_server::AuthorizationData;
use crate::proto::*;
use cedar_policy::{Authorizer, Context, Entities, EntityUid, PolicySet, Request as CedarRequest};
use hodei_domain::{
    DomainEventEnvelope, EventBusPort, EventDispatcher, EventStorePort, PolicyRepository, PolicyStoreId,
};
use hodei_infrastructure::jwt::JwtValidator;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info};
use async_trait::async_trait;

pub struct AuthorizationDataService<R> {
    repository: Arc<R>,
    jwt_validator: JwtValidator,
}

impl<R> AuthorizationDataService<R>
where
    R: PolicyRepository + Send + Sync + 'static,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            jwt_validator: JwtValidator::new(),
        }
    }

    async fn publish_event(&self, _event: DomainEventEnvelope) {
        // Event publishing not implemented
        info!("Event would be published here");
    }

    fn build_entity_uid(identifier: &EntityIdentifier) -> Result<EntityUid, Status> {
        EntityUid::from_str(&format!(
            "{}::\"{}\"",
            identifier.entity_type, identifier.entity_id
        ))
        .map_err(|e| {
            error!("Failed to parse entity UID: {}", e);
            Status::invalid_argument(format!("Invalid entity identifier: {}", e))
        })
    }

    fn build_entities(entities: &[Entity]) -> Result<Entities, Status> {
        let mut entities_json = Vec::new();

        for entity in entities {
            let uid = Self::build_entity_uid(
                entity
                    .identifier
                    .as_ref()
                    .ok_or_else(|| Status::invalid_argument("Entity identifier is required"))?,
            )?;

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

#[async_trait]
impl<R> AuthorizationData for AuthorizationDataService<R>
where
    R: PolicyRepository + Send + Sync + 'static,
{
    async fn is_authorized(
        &self,
        request: Request<IsAuthorizedRequest>,
    ) -> Result<Response<IsAuthorizedResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Authorization request for policy store: {}",
            req.policy_store_id
        );

        // 1. Parse policy store ID
        let policy_store_id = PolicyStoreId::new(req.policy_store_id.clone())
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        // 2. Load policies from database
        let policies = self
            .repository
            .list_policies(&policy_store_id)
            .await
            .map_err(|e| {
                error!("Failed to load policies: {}", e);
                Status::internal(format!("Failed to load policies: {}", e))
            })?;

        if policies.is_empty() {
            info!(
                "No policies found for policy store: {}",
                req.policy_store_id
            );
            return Ok(Response::new(IsAuthorizedResponse {
                decision: Decision::Deny as i32,
                determining_policies: vec![],
                errors: vec!["No policies found in policy store".to_string()],
            }));
        }

        // 3. Build Cedar PolicySet
        let mut policy_set_str = String::new();
        for policy in &policies {
            policy_set_str.push_str(policy.statement.as_str());
            policy_set_str.push('\n');
        }

        let policy_set = PolicySet::from_str(&policy_set_str).map_err(|e| {
            error!("Failed to parse policy set: {}", e);
            Status::internal(format!("Failed to parse policy set: {}", e))
        })?;

        // 4. Build Cedar entities
        let principal = Self::build_entity_uid(
            req.principal
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("Principal is required"))?,
        )?;

        let action = Self::build_entity_uid(
            req.action
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("Action is required"))?,
        )?;

        let resource = Self::build_entity_uid(
            req.resource
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("Resource is required"))?,
        )?;

        // 5. Build context
        let context = Self::build_context(req.context.as_deref())?;

        // 6. Build entities slice
        let entities = Self::build_entities(&req.entities)?;

        // 7. Create Cedar request
        let cedar_request =
            CedarRequest::new(principal, action, resource, context, None).map_err(|e| {
                error!("Failed to create Cedar request: {}", e);
                Status::internal(format!("Failed to create Cedar request: {}", e))
            })?;

        // 8. Evaluate with Cedar Authorizer
        let authorizer = Authorizer::new();
        let response = authorizer.is_authorized(&cedar_request, &policy_set, &entities);

        // 9. Convert decision
        let decision = match response.decision() {
            cedar_policy::Decision::Allow => Decision::Allow,
            cedar_policy::Decision::Deny => Decision::Deny,
        };

        // 10. Extract determining policies
        let determining_policies: Vec<String> = response
            .diagnostics()
            .reason()
            .map(|policy_id| policy_id.to_string())
            .collect();

        // 11. Extract errors
        let errors: Vec<String> = response
            .diagnostics()
            .errors()
            .map(|err| err.to_string())
            .collect();

        info!(
            "Authorization decision: {:?}, determining policies: {:?}",
            decision, determining_policies
        );

        // 12. Return response
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
            let result = self.is_authorized(Request::new(auth_request)).await;

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
        let req = request.into_inner();
        info!(
            "Authorization with token request for policy store: {} and identity source: {}",
            req.policy_store_id, req.identity_source_id
        );

        // 1. Load Identity Source configuration
        let policy_store_id = PolicyStoreId::new(req.policy_store_id.clone())
            .map_err(|e| Status::invalid_argument(format!("Invalid policy store ID: {}", e)))?;

        let identity_source = self
            .repository
            .get_identity_source(&policy_store_id, &req.identity_source_id)
            .await
            .map_err(|e| {
                error!("Failed to load identity source: {}", e);
                Status::not_found(format!("Identity source not found: {}", e))
            })?;

        // 2. Parse Identity Source configuration
        let config_json: serde_json::Value =
            serde_json::from_str(&identity_source.configuration_json)
                .map_err(|e| Status::internal(format!("Invalid identity source config: {}", e)))?;

        let issuer = config_json["issuer"]
            .as_str()
            .ok_or_else(|| Status::internal("Identity source missing 'issuer'"))?;

        let client_ids = config_json["client_ids"]
            .as_array()
            .ok_or_else(|| Status::internal("Identity source missing 'client_ids'"))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<String>>();

        let jwks_uri = config_json["jwks_uri"]
            .as_str()
            .ok_or_else(|| Status::internal("Identity source missing 'jwks_uri'"))?;

        info!(
            "Validating token with issuer: {}, jwks_uri: {}",
            issuer, jwks_uri
        );

        // 3. Validate JWT token (signature, issuer, audience, expiration)
        let validated_claims = self
            .jwt_validator
            .validate_token(&req.access_token, issuer, &client_ids, jwks_uri)
            .await
            .map_err(|e| {
                error!("JWT validation failed: {}", e);
                Status::unauthenticated(format!("Invalid token: {}", e))
            })?;

        info!(
            "Token validated successfully for subject: {}",
            validated_claims.sub
        );

        // 4. Extract principal ID from claims
        let principal_id = validated_claims.sub.clone();
        let principal = EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: principal_id.clone(),
        };

        info!("Mapped principal: User::{}", principal_id);

        // 5. Extract groups from claims and create entities
        let mut entities = Vec::new();

        // Create principal entity
        let mut principal_attrs = HashMap::new();

        // Add email if present
        if let Some(email) = validated_claims.additional_claims.get("email") {
            if let Some(email_str) = email.as_str() {
                principal_attrs.insert("email".to_string(), format!("\"{}\"", email_str));
            }
        }

        // Extract groups as parent entities
        let mut parents = Vec::new();
        if let Some(groups_value) = validated_claims.additional_claims.get("groups") {
            if let Some(groups_array) = groups_value.as_array() {
                for group in groups_array {
                    if let Some(group_str) = group.as_str() {
                        parents.push(EntityIdentifier {
                            entity_type: "Role".to_string(),
                            entity_id: group_str.to_string(),
                        });
                    }
                }
            }
        }

        // Create principal entity with attributes and parents
        entities.push(Entity {
            identifier: Some(principal.clone()),
            attributes: principal_attrs,
            parents,
        });

        // 6. Merge with any additional entities from request
        entities.extend(req.entities);

        // 7. Create authorization request
        let auth_request = IsAuthorizedRequest {
            policy_store_id: req.policy_store_id,
            principal: Some(principal),
            action: req.action,
            resource: req.resource,
            context: req.context,
            entities,
        };

        // 8. Evaluate with Cedar (real authorization)
        self.is_authorized(Request::new(auth_request)).await
    }
}
