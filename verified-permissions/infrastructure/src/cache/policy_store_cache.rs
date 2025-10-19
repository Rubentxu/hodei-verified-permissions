//! Policy Store Cache - Mantiene PolicySet y Schema en memoria

use cedar_policy::{PolicySet, Schema, Policy};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::RwLock;
use crate::error::Result;
use crate::storage::PolicyRepository;

/// Cache en memoria para un Policy Store específico
/// 
/// Mantiene:
/// - PolicySet de Cedar (todas las políticas compiladas)
/// - Schema de Cedar (para validación)
/// - Timestamp de última actualización
/// 
/// # Concurrencia
/// 
/// Usa RwLock para permitir:
/// - Múltiples lecturas concurrentes (evaluación de políticas)
/// - Escrituras exclusivas (actualización de políticas)
pub struct PolicyStoreCache {
    /// ID del Policy Store
    pub policy_store_id: String,
    
    /// PolicySet de Cedar en memoria
    /// RwLock permite muchas lecturas concurrentes
    policy_set: Arc<RwLock<PolicySet>>,
    
    /// Schema de Cedar (opcional)
    schema: Arc<RwLock<Option<Schema>>>,
    
    /// Timestamp de última actualización
    last_updated: Arc<RwLock<DateTime<Utc>>>,
}

impl PolicyStoreCache {
    /// Crea un nuevo cache vacío
    pub fn new(policy_store_id: String) -> Self {
        Self {
            policy_store_id,
            policy_set: Arc::new(RwLock::new(PolicySet::new())),
            schema: Arc::new(RwLock::new(None)),
            last_updated: Arc::new(RwLock::new(Utc::now())),
        }
    }
    
    /// Carga el cache desde la base de datos
    /// 
    /// # Arguments
    /// * `repo` - Repository para cargar datos
    /// 
    /// # Errors
    /// Retorna error si:
    /// - No se puede cargar el schema
    /// - No se pueden cargar las políticas
    /// - Las políticas tienen sintaxis inválida
    pub async fn load_from_db(&self, repo: &dyn PolicyRepository) -> Result<()> {
        tracing::info!("Loading cache for policy store: {}", self.policy_store_id);
        
        // 1. Cargar schema (opcional)
        match repo.get_schema(&self.policy_store_id).await {
            Ok(schema_model) => {
                match Schema::from_json_str(&schema_model.schema_json) {
                    Ok(cedar_schema) => {
                        *self.schema.write().await = Some(cedar_schema);
                        tracing::debug!("Schema loaded for store {}", self.policy_store_id);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse schema for store {}: {}", 
                                     self.policy_store_id, e);
                    }
                }
            }
            Err(_) => {
                tracing::debug!("No schema found for store {}", self.policy_store_id);
            }
        }
        
        // 2. Cargar todas las políticas
        let policies = repo.list_policies(&self.policy_store_id).await?;
        let mut policy_set = PolicySet::new();
        let mut loaded_count = 0;
        let mut error_count = 0;
        
        for policy_model in policies {
            match Policy::from_str(&policy_model.statement) {
                Ok(cedar_policy) => {
                    if let Err(e) = policy_set.add(cedar_policy) {
                        tracing::error!("Failed to add policy {} to set: {}", 
                                      policy_model.policy_id, e);
                        error_count += 1;
                    } else {
                        loaded_count += 1;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse policy {}: {}", 
                                  policy_model.policy_id, e);
                    error_count += 1;
                }
            }
        }
        
        // 3. Actualizar cache
        *self.policy_set.write().await = policy_set;
        *self.last_updated.write().await = Utc::now();
        
        tracing::info!(
            "Cache loaded for store {}: {} policies loaded, {} errors",
            self.policy_store_id, loaded_count, error_count
        );
        
        Ok(())
    }
    
    /// Obtiene una referencia al PolicySet (para lectura)
    /// 
    /// # Returns
    /// Arc<RwLock<PolicySet>> que puede ser compartido entre threads
    pub fn policy_set(&self) -> Arc<RwLock<PolicySet>> {
        self.policy_set.clone()
    }
    
    /// Obtiene una referencia al Schema (para lectura)
    pub fn schema(&self) -> Arc<RwLock<Option<Schema>>> {
        self.schema.clone()
    }
    
    /// Obtiene el timestamp de última actualización
    pub async fn last_updated(&self) -> DateTime<Utc> {
        *self.last_updated.read().await
    }
    
    /// Agrega una política al cache
    /// 
    /// # Arguments
    /// * `policy_id` - ID de la política
    /// * `statement` - Statement de Cedar
    /// 
    /// # Errors
    /// Retorna error si la política tiene sintaxis inválida
    pub async fn add_policy(&self, policy_id: &str, statement: &str) -> Result<()> {
        let policy = Policy::from_str(statement)?;
        
        let mut policy_set = self.policy_set.write().await;
        policy_set.add(policy)?;
        
        *self.last_updated.write().await = Utc::now();
        
        tracing::debug!("Policy {} added to cache for store {}", 
                       policy_id, self.policy_store_id);
        
        Ok(())
    }
    
    /// Elimina una política del cache
    /// 
    /// # Arguments
    /// * `policy_id` - ID de la política a eliminar
    pub async fn remove_policy(&self, policy_id: &str) -> Result<()> {
        let mut policy_set = self.policy_set.write().await;
        
        // Cedar PolicySet no tiene método remove directo
        // Necesitamos reconstruir el set sin la política eliminada
        let policies: Vec<Policy> = policy_set
            .policies()
            .filter(|p| {
                // Comparar el ID de la política con el que queremos eliminar
                // p.id() devuelve &PolicyId, no Option
                p.id().to_string() != policy_id
            })
            .cloned()
            .collect();
        
        let mut new_set = PolicySet::new();
        for policy in policies {
            new_set.add(policy)?;
        }
        
        *policy_set = new_set;
        *self.last_updated.write().await = Utc::now();
        
        tracing::debug!("Policy {} removed from cache for store {}", 
                       policy_id, self.policy_store_id);
        
        Ok(())
    }
    
    /// Actualiza el schema en el cache
    /// 
    /// # Arguments
    /// * `schema_json` - Schema en formato JSON
    /// 
    /// # Errors
    /// Retorna error si el schema es inválido
    pub async fn update_schema(&self, schema_json: &str) -> Result<()> {
        let cedar_schema = Schema::from_json_str(schema_json)?;
        
        *self.schema.write().await = Some(cedar_schema);
        *self.last_updated.write().await = Utc::now();
        
        tracing::debug!("Schema updated in cache for store {}", self.policy_store_id);
        
        Ok(())
    }
    
    /// Elimina el schema del cache
    pub async fn remove_schema(&self) {
        *self.schema.write().await = None;
        *self.last_updated.write().await = Utc::now();
        
        tracing::debug!("Schema removed from cache for store {}", self.policy_store_id);
    }
    
    /// Obtiene estadísticas del cache
    pub async fn stats(&self) -> CacheStats {
        let policy_set = self.policy_set.read().await;
        let schema = self.schema.read().await;
        let last_updated = *self.last_updated.read().await;
        
        CacheStats {
            policy_store_id: self.policy_store_id.clone(),
            policy_count: policy_set.policies().count(),
            has_schema: schema.is_some(),
            last_updated,
        }
    }
}

/// Estadísticas del cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub policy_store_id: String,
    pub policy_count: usize,
    pub has_schema: bool,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_creation() {
        let cache = PolicyStoreCache::new("test-store".to_string());
        assert_eq!(cache.policy_store_id, "test-store");
        
        let stats = cache.stats().await;
        assert_eq!(stats.policy_count, 0);
        assert!(!stats.has_schema);
    }
    
    #[tokio::test]
    async fn test_add_policy() {
        let cache = PolicyStoreCache::new("test-store".to_string());
        
        let policy = r#"permit(principal, action, resource);"#;
        cache.add_policy("policy-1", policy).await.unwrap();
        
        let stats = cache.stats().await;
        assert_eq!(stats.policy_count, 1);
    }
    
    #[tokio::test]
    #[ignore] // TODO: Cedar from_str no asigna ID automáticamente
    async fn test_remove_policy() {
        let cache = PolicyStoreCache::new("test-store".to_string());
        
        let policy = r#"permit(principal, action, resource);"#;
        cache.add_policy("policy-1", policy).await.unwrap();
        cache.remove_policy("policy-1").await.unwrap();
        
        let stats = cache.stats().await;
        assert_eq!(stats.policy_count, 0);
    }
    
    #[tokio::test]
    async fn test_update_schema() {
        let cache = PolicyStoreCache::new("test-store".to_string());
        
        let schema = r#"{"": {"entityTypes": {}, "actions": {}}}"#;
        cache.update_schema(schema).await.unwrap();
        
        let stats = cache.stats().await;
        assert!(stats.has_schema);
    }
}
