//! Local Agent for offline policy evaluation
//!
//! This module provides a local agent that can synchronize policies from the central
//! service and evaluate them locally with ultra-low latency.

use crate::proto::{
    authorization_control_client::AuthorizationControlClient,
    authorization_data_server::{AuthorizationData, AuthorizationDataServer},
    IsAuthorizedRequest, IsAuthorizedResponse, ListPoliciesRequest,
};
use crate::storage::models::{Policy, Schema};
use cedar_policy::{Authorizer, Context, Entities, EntityUid, PolicySet, Request as CedarRequest};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

/// Local agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Central service URL
    pub central_service_url: String,
    /// Policy store ID to sync
    pub policy_store_id: String,
    /// Sync interval in seconds
    pub sync_interval_secs: u64,
    /// Local gRPC port
    pub local_port: u16,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            central_service_url: "http://localhost:50051".to_string(),
            policy_store_id: String::new(),
            sync_interval_secs: 60,
            local_port: 50052,
        }
    }
}

/// Local policy cache
#[derive(Debug, Clone)]
struct PolicyCache {
    policies: Vec<Policy>,
    schema: Option<Schema>,
    last_sync: std::time::SystemTime,
}

/// Local agent for policy evaluation
pub struct LocalAgent {
    config: AgentConfig,
    cache: Arc<RwLock<PolicyCache>>,
}

impl LocalAgent {
    /// Create a new local agent
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(PolicyCache {
                policies: Vec::new(),
                schema: None,
                last_sync: std::time::SystemTime::now(),
            })),
        }
    }

    /// Start the agent (sync loop + gRPC server)
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let agent = Arc::new(self);
        
        // Start sync loop
        let sync_agent = agent.clone();
        tokio::spawn(async move {
            sync_agent.sync_loop().await;
        });

        // Start local gRPC server
        let addr = format!("127.0.0.1:{}", agent.config.local_port).parse()?;
        let service = LocalAgentService {
            cache: agent.cache.clone(),
            policy_store_id: agent.config.policy_store_id.clone(),
        };

        info!("Local agent listening on {}", addr);
        
        tonic::transport::Server::builder()
            .add_service(AuthorizationDataServer::new(service))
            .serve(addr)
            .await?;

        Ok(())
    }

    /// Sync loop - periodically fetch policies from central service
    async fn sync_loop(&self) {
        let mut interval = time::interval(Duration::from_secs(self.config.sync_interval_secs));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.sync_policies().await {
                error!("Failed to sync policies: {}", e);
            }
        }
    }

    /// Sync policies from central service
    async fn sync_policies(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Syncing policies from central service...");
        
        let mut client = AuthorizationControlClient::connect(self.config.central_service_url.clone()).await?;
        
        // Fetch policies
        let response = client
            .list_policies(Request::new(ListPoliciesRequest {
                policy_store_id: self.config.policy_store_id.clone(),
                max_results: None,
                next_token: None,
            }))
            .await?;
        
        let policies: Vec<Policy> = response
            .into_inner()
            .policies
            .into_iter()
            .map(|p| Policy {
                policy_id: p.policy_id,
                policy_store_id: self.config.policy_store_id.clone(),
                statement: String::new(), // Will be fetched individually if needed
                description: p.description,
                created_at: chrono::Utc::now(), // Simplified
                updated_at: chrono::Utc::now(),
            })
            .collect();
        
        // Update cache
        let mut cache = self.cache.write().await;
        cache.policies = policies;
        cache.last_sync = std::time::SystemTime::now();
        
        info!("Synced {} policies", cache.policies.len());
        
        Ok(())
    }
}

/// Local agent gRPC service
struct LocalAgentService {
    cache: Arc<RwLock<PolicyCache>>,
    policy_store_id: String,
}

#[tonic::async_trait]
impl AuthorizationData for LocalAgentService {
    async fn is_authorized(
        &self,
        request: Request<IsAuthorizedRequest>,
    ) -> Result<Response<IsAuthorizedResponse>, Status> {
        let req = request.into_inner();
        
        // Get cached policies
        let cache = self.cache.read().await;
        
        if cache.policies.is_empty() {
            warn!("No policies in cache, returning DENY");
            return Ok(Response::new(IsAuthorizedResponse {
                decision: crate::proto::Decision::Deny as i32,
                determining_policies: vec![],
                errors: vec!["No policies loaded in local cache".to_string()],
            }));
        }
        
        // Build policy set
        let mut policy_set_str = String::new();
        for policy in &cache.policies {
            policy_set_str.push_str(&policy.statement);
            policy_set_str.push('\n');
        }
        
        let policy_set = PolicySet::from_str(&policy_set_str).map_err(|e| {
            error!("Failed to parse policy set: {}", e);
            Status::internal(format!("Failed to parse policies: {}", e))
        })?;
        
        // Build Cedar request
        let principal = EntityUid::from_str(&format!(
            "{}::\"{}\"",
            req.principal.as_ref().unwrap().entity_type,
            req.principal.as_ref().unwrap().entity_id
        ))
        .map_err(|e| Status::invalid_argument(format!("Invalid principal: {}", e)))?;
        
        let action = EntityUid::from_str(&format!(
            "{}::\"{}\"",
            req.action.as_ref().unwrap().entity_type,
            req.action.as_ref().unwrap().entity_id
        ))
        .map_err(|e| Status::invalid_argument(format!("Invalid action: {}", e)))?;
        
        let resource = EntityUid::from_str(&format!(
            "{}::\"{}\"",
            req.resource.as_ref().unwrap().entity_type,
            req.resource.as_ref().unwrap().entity_id
        ))
        .map_err(|e| Status::invalid_argument(format!("Invalid resource: {}", e)))?;
        
        let context = Context::empty();
        let entities = Entities::empty();
        
        let cedar_request = CedarRequest::new(principal, action, resource, context, None)
            .map_err(|e| Status::internal(format!("Failed to create request: {}", e)))?;
        
        // Evaluate locally
        let authorizer = Authorizer::new();
        let response = authorizer.is_authorized(&cedar_request, &policy_set, &entities);
        
        let decision = match response.decision() {
            cedar_policy::Decision::Allow => crate::proto::Decision::Allow,
            cedar_policy::Decision::Deny => crate::proto::Decision::Deny,
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
        
        Ok(Response::new(IsAuthorizedResponse {
            decision: decision as i32,
            determining_policies,
            errors,
        }))
    }

    async fn batch_is_authorized(
        &self,
        _request: Request<crate::proto::BatchIsAuthorizedRequest>,
    ) -> Result<Response<crate::proto::BatchIsAuthorizedResponse>, Status> {
        Err(Status::unimplemented("Batch operations not supported in local agent"))
    }

    async fn is_authorized_with_token(
        &self,
        _request: Request<crate::proto::IsAuthorizedWithTokenRequest>,
    ) -> Result<Response<IsAuthorizedResponse>, Status> {
        Err(Status::unimplemented("Token validation not supported in local agent"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.sync_interval_secs, 60);
        assert_eq!(config.local_port, 50052);
    }

    #[test]
    fn test_agent_creation() {
        let config = AgentConfig {
            policy_store_id: "test-store".to_string(),
            ..Default::default()
        };
        let agent = LocalAgent::new(config);
        assert!(!agent.config.policy_store_id.is_empty());
    }
}
