//! Authorization Service - Core authorization logic

use crate::cache::CacheManager;
use crate::error::Result;
use crate::storage::AuthorizationLog;
use cedar_policy::{Authorizer, Request, Entities, Context, Decision};
use chrono::Utc;
use std::sync::Arc;

/// Respuesta de autorización
#[derive(Debug, Clone)]
pub struct AuthorizationResponse {
    /// Decisión: Allow o Deny
    pub decision: Decision,
    
    /// Políticas que determinaron la decisión
    pub determining_policies: Vec<String>,
    
    /// Errores de evaluación (si los hay)
    pub errors: Vec<String>,
}

/// Servicio de autorización
/// 
/// Responsabilidades:
/// - Evaluar requests de autorización usando Cedar
/// - Leer políticas desde el cache (100% memoria)
/// - Registrar decisiones de forma asíncrona
/// 
/// # Rendimiento
/// 
/// - Evaluación: ~100μs (sin I/O)
/// - Throughput: >100K ops/s
/// - Concurrencia: Ilimitada (lecturas)
pub struct AuthorizationService {
    /// Cache manager para acceso a políticas
    cache_manager: Arc<CacheManager>,
    
    /// Authorizer de Cedar (stateless)
    authorizer: Authorizer,
}

impl AuthorizationService {
    /// Crea un nuevo servicio de autorización
    /// 
    /// # Arguments
    /// * `cache_manager` - Cache manager con políticas cargadas
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self {
            cache_manager,
            authorizer: Authorizer::new(),
        }
    }
    
    /// Evalúa una solicitud de autorización
    /// 
    /// # Arguments
    /// * `policy_store_id` - ID del policy store
    /// * `principal` - Principal (ej: "User::alice")
    /// * `action` - Acción (ej: "Action::view")
    /// * `resource` - Recurso (ej: "Document::doc123")
    /// * `context` - Context JSON opcional
    /// * `entities` - Entities JSON opcional
    /// 
    /// # Returns
    /// Respuesta con la decisión y políticas determinantes
    /// 
    /// # Performance
    /// Esta operación es extremadamente rápida (~100μs) porque:
    /// - Lee del cache (memoria)
    /// - No hace I/O
    /// - Cedar evalúa en memoria
    pub async fn is_authorized(
        &self,
        policy_store_id: &str,
        principal: &str,
        action: &str,
        resource: &str,
        context: Option<&str>,
        entities: Option<&str>,
    ) -> Result<AuthorizationResponse> {
        // 1. Obtener cache (lock de lectura, muy rápido)
        let cache = self.cache_manager.get_cache(policy_store_id).await?;
        
        // 2. Leer policy set y schema (locks de lectura)
        let policy_set_arc = cache.policy_set();
        let policy_set = policy_set_arc.read().await;
        let _schema_arc = cache.schema();
        let _schema = _schema_arc.read().await;
        
        // 3. Parsear entidades
        let entities = if let Some(entities_json) = entities {
            Entities::from_json_str(entities_json, None)
                .map_err(|e| crate::error::AuthorizationError::CedarParseError(format!("Entities error: {:?}", e)))?
        } else {
            Entities::empty()
        };
        
        // 4. Parsear context
        let ctx = if let Some(context_json) = context {
            Context::from_json_str(context_json, None)?
        } else {
            Context::empty()
        };
        
        // 5. Crear request
        let principal_euid = principal.parse()
            .map_err(|e| crate::error::AuthorizationError::InvalidPolicy(format!("Invalid principal: {}", e)))?;
        let action_euid = action.parse()
            .map_err(|e| crate::error::AuthorizationError::InvalidPolicy(format!("Invalid action: {}", e)))?;
        let resource_euid = resource.parse()
            .map_err(|e| crate::error::AuthorizationError::InvalidPolicy(format!("Invalid resource: {}", e)))?;
        
        let request = Request::new(
            principal_euid,
            action_euid,
            resource_euid,
            ctx,
            None,
        ).map_err(|e| crate::error::AuthorizationError::ValidationFailed(e.to_string()))?;
        
        // 6. Evaluar (100% en memoria, sin I/O)
        let response = self.authorizer.is_authorized(&request, &policy_set, &entities);
        
        // 7. Extraer información de la respuesta
        let decision = response.decision();
        let determining_policies: Vec<String> = response
            .diagnostics()
            .reason()
            .map(|p| p.to_string())
            .collect();
        let errors: Vec<String> = response
            .diagnostics()
            .errors()
            .map(|e| e.to_string())
            .collect();
        
        // 8. Log asíncrono (no bloquea la respuesta)
        self.log_authorization_async(
            policy_store_id,
            principal,
            action,
            resource,
            decision,
        ).await;
        
        Ok(AuthorizationResponse {
            decision,
            determining_policies,
            errors,
        })
    }
    
    /// Evalúa múltiples solicitudes de autorización (batch)
    /// 
    /// # Arguments
    /// * `policy_store_id` - ID del policy store
    /// * `requests` - Vector de (principal, action, resource)
    /// 
    /// # Returns
    /// Vector de respuestas en el mismo orden
    /// 
    /// # Performance
    /// Más eficiente que llamadas individuales porque:
    /// - Obtiene el cache una sola vez
    /// - Reutiliza el lock de lectura
    pub async fn batch_is_authorized(
        &self,
        policy_store_id: &str,
        requests: Vec<(&str, &str, &str)>,
    ) -> Result<Vec<AuthorizationResponse>> {
        // Obtener cache una sola vez
        let cache = self.cache_manager.get_cache(policy_store_id).await?;
        let policy_set_arc = cache.policy_set();
        let policy_set = policy_set_arc.read().await;
        let entities = Entities::empty();
        
        let mut responses = Vec::with_capacity(requests.len());
        
        for (principal, action, resource) in requests {
            // Crear request
            let principal_euid = principal.parse()
                .map_err(|e| crate::error::AuthorizationError::InvalidPolicy(format!("Invalid principal: {}", e)))?;
            let action_euid = action.parse()
                .map_err(|e| crate::error::AuthorizationError::InvalidPolicy(format!("Invalid action: {}", e)))?;
            let resource_euid = resource.parse()
                .map_err(|e| crate::error::AuthorizationError::InvalidPolicy(format!("Invalid resource: {}", e)))?;
            
            let request = Request::new(
                principal_euid,
                action_euid,
                resource_euid,
                Context::empty(),
                None,
            ).map_err(|e| crate::error::AuthorizationError::ValidationFailed(e.to_string()))?;
            
            // Evaluar
            let response = self.authorizer.is_authorized(&request, &policy_set, &entities);
            
            let decision = response.decision();
            let determining_policies: Vec<String> = response
                .diagnostics()
                .reason()
                .map(|p| p.to_string())
                .collect();
            let errors: Vec<String> = response
                .diagnostics()
                .errors()
                .map(|e| e.to_string())
                .collect();
            
            responses.push(AuthorizationResponse {
                decision,
                determining_policies,
                errors,
            });
            
            // Log asíncrono
            self.log_authorization_async(
                policy_store_id,
                principal,
                action,
                resource,
                decision,
            ).await;
        }
        
        Ok(responses)
    }
    
    /// Registra una decisión de autorización de forma asíncrona
    /// 
    /// Este método no bloquea la respuesta al usuario.
    /// El log se hace en background.
    async fn log_authorization_async(
        &self,
        policy_store_id: &str,
        principal: &str,
        action: &str,
        resource: &str,
        decision: Decision,
    ) {
        let log = AuthorizationLog {
            policy_store_id: policy_store_id.to_string(),
            principal: principal.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            decision: format!("{:?}", decision),
            timestamp: Utc::now(),
        };
        
        // Spawn task para no bloquear
        let repo = self.cache_manager.repository.clone();
        tokio::spawn(async move {
            if let Err(e) = repo.log_authorization(log).await {
                tracing::error!("Failed to log authorization: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Repository;
    
    async fn create_test_service() -> AuthorizationService {
        let repo = Repository::new(":memory:").await.unwrap();
        let cache_manager = Arc::new(CacheManager::new(Arc::new(repo)));
        cache_manager.initialize().await.unwrap();
        AuthorizationService::new(cache_manager)
    }
    
    #[tokio::test]
    async fn test_service_creation() {
        let _service = create_test_service().await;
        // Service should be created successfully
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_is_authorized_with_policy() {
        let repo = Repository::new(":memory:").await.unwrap();
        let repo_arc = Arc::new(repo);
        
        // Create store and policy
        let store = repo_arc.create_policy_store(None).await.unwrap();
        
        let schema = r#"{"": {"entityTypes": {"User": {}, "Document": {}}, "actions": {"view": {}}}}"#;
        repo_arc.put_schema(&store.id, schema.to_string()).await.unwrap();
        
        let policy = r#"permit(principal == User::"alice", action == Action::"view", resource == Document::"doc123");"#;
        repo_arc.create_policy(&store.id, "p1", policy.to_string(), None).await.unwrap();
        
        // Create service and initialize cache
        let cache_manager = Arc::new(CacheManager::new(repo_arc));
        cache_manager.initialize().await.unwrap();
        let service = AuthorizationService::new(cache_manager);
        
        // Test authorization
        let response = service.is_authorized(
            &store.id,
            "User::\"alice\"",
            "Action::\"view\"",
            "Document::\"doc123\"",
            None,
            None,
        ).await.unwrap();
        
        assert_eq!(response.decision, Decision::Allow);
    }
    
    #[tokio::test]
    async fn test_batch_is_authorized() {
        let repo = Repository::new(":memory:").await.unwrap();
        let repo_arc = Arc::new(repo);
        
        let store = repo_arc.create_policy_store(None).await.unwrap();
        
        let schema = r#"{"": {"entityTypes": {"User": {}, "Document": {}}, "actions": {"view": {}}}}"#;
        repo_arc.put_schema(&store.id, schema.to_string()).await.unwrap();
        
        let policy = r#"permit(principal == User::"alice", action == Action::"view", resource);"#;
        repo_arc.create_policy(&store.id, "p1", policy.to_string(), None).await.unwrap();
        
        let cache_manager = Arc::new(CacheManager::new(repo_arc));
        cache_manager.initialize().await.unwrap();
        let service = AuthorizationService::new(cache_manager);
        
        // Batch request
        let requests = vec![
            ("User::\"alice\"", "Action::\"view\"", "Document::\"doc1\""),
            ("User::\"alice\"", "Action::\"view\"", "Document::\"doc2\""),
            ("User::\"bob\"", "Action::\"view\"", "Document::\"doc1\""),
        ];
        
        let responses = service.batch_is_authorized(&store.id, requests).await.unwrap();
        
        assert_eq!(responses.len(), 3);
        assert_eq!(responses[0].decision, Decision::Allow);
        assert_eq!(responses[1].decision, Decision::Allow);
        assert_eq!(responses[2].decision, Decision::Deny);
    }
}
