//! Cache Manager - Gestiona todos los caches de Policy Stores

use crate::cache::PolicyStoreCache;
use crate::error::Result;
use crate::storage::{PolicyRepository, models::{PolicyStore, Policy, Schema}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Gestor global de caches para todos los Policy Stores
/// 
/// Responsabilidades:
/// - Mantener un cache por cada Policy Store
/// - Sincronizar cambios entre DB y cache
/// - Proporcionar acceso thread-safe a los caches
/// 
/// # Arquitectura
/// 
/// ```text
/// CacheManager
///   ├─ Store1 Cache (PolicySet + Schema)
///   ├─ Store2 Cache (PolicySet + Schema)
///   └─ Store3 Cache (PolicySet + Schema)
/// ```
pub struct CacheManager {
    /// Mapa de caches por policy_store_id
    caches: Arc<RwLock<HashMap<String, Arc<PolicyStoreCache>>>>,
    
    /// Repository para persistencia
    repository: Arc<dyn PolicyRepository>,
}

impl CacheManager {
    /// Crea un nuevo CacheManager
    /// 
    /// # Arguments
    /// * `repository` - Repository para acceso a la base de datos
    pub fn new(repository: Arc<dyn PolicyRepository>) -> Self {
        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
            repository,
        }
    }
    
    /// Inicializa el cache manager cargando todos los policy stores desde DB
    /// 
    /// Este método debe ser llamado al inicio del servidor para cargar
    /// todos los datos en memoria.
    /// 
    /// # Errors
    /// Retorna error si no se pueden cargar los stores desde DB
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing cache manager...");
        
        let stores = self.repository.list_policy_stores().await?;
        let mut loaded = 0;
        let mut errors = 0;
        
        for store in stores {
            match self.load_store_cache(&store.id).await {
                Ok(_) => loaded += 1,
                Err(e) => {
                    tracing::error!("Failed to load cache for store {}: {}", store.id, e);
                    errors += 1;
                }
            }
        }
        
        tracing::info!(
            "Cache manager initialized: {} stores loaded, {} errors",
            loaded, errors
        );
        
        Ok(())
    }
    
    /// Carga el cache para un policy store específico
    async fn load_store_cache(&self, policy_store_id: &str) -> Result<()> {
        let cache = Arc::new(PolicyStoreCache::new(policy_store_id.to_string()));
        cache.load_from_db(self.repository.as_ref()).await?;
        
        self.caches.write().await.insert(
            policy_store_id.to_string(),
            cache
        );
        
        Ok(())
    }
    
    /// Obtiene el cache de un policy store
    /// 
    /// # Arguments
    /// * `policy_store_id` - ID del policy store
    /// 
    /// # Returns
    /// Arc al cache del store
    /// 
    /// # Errors
    /// Retorna error si el store no existe
    pub async fn get_cache(&self, policy_store_id: &str) -> Result<Arc<PolicyStoreCache>> {
        let caches = self.caches.read().await;
        
        caches
            .get(policy_store_id)
            .cloned()
            .ok_or_else(|| {
                crate::error::AuthorizationError::NotFound(
                    format!("Policy store not found: {}", policy_store_id)
                )
            })
    }
    
    // ========================================================================
    // Policy Store Operations (DB + Cache)
    // ========================================================================
    
    /// Crea un nuevo policy store (DB + Cache)
    /// 
    /// # Arguments
    /// * `description` - Descripción opcional
    /// 
    /// # Returns
    /// El policy store creado
    pub async fn create_policy_store(&self, description: Option<String>) -> Result<PolicyStore> {
        // 1. Crear en DB
        let store = self.repository.create_policy_store(description).await?;
        
        // 2. Crear cache vacío
        let cache = Arc::new(PolicyStoreCache::new(store.id.clone()));
        self.caches.write().await.insert(store.id.clone(), cache);
        
        tracing::info!("Policy store created: {}", store.id);
        
        Ok(store)
    }
    
    /// Elimina un policy store (DB + Cache)
    /// 
    /// # Arguments
    /// * `policy_store_id` - ID del policy store
    pub async fn delete_policy_store(&self, policy_store_id: &str) -> Result<()> {
        // 1. Eliminar de DB (cascade)
        self.repository.delete_policy_store(policy_store_id).await?;
        
        // 2. Eliminar cache
        self.caches.write().await.remove(policy_store_id);
        
        tracing::info!("Policy store deleted: {}", policy_store_id);
        
        Ok(())
    }
    
    // ========================================================================
    // Schema Operations (DB + Cache)
    // ========================================================================
    
    /// Guarda o actualiza el schema (DB + Cache)
    /// 
    /// # Arguments
    /// * `policy_store_id` - ID del policy store
    /// * `schema` - Schema en formato JSON
    pub async fn put_schema(&self, policy_store_id: &str, schema: String) -> Result<()> {
        // 1. Guardar en DB
        self.repository.put_schema(policy_store_id, schema.clone()).await?;
        
        // 2. Actualizar cache
        let cache = self.get_cache(policy_store_id).await?;
        cache.update_schema(&schema).await?;
        
        tracing::info!("Schema updated for store: {}", policy_store_id);
        
        Ok(())
    }
    
    /// Elimina el schema (DB + Cache)
    pub async fn delete_schema(&self, policy_store_id: &str) -> Result<()> {
        // 1. Eliminar de DB
        self.repository.delete_schema(policy_store_id).await?;
        
        // 2. Eliminar de cache
        let cache = self.get_cache(policy_store_id).await?;
        cache.remove_schema().await;
        
        tracing::info!("Schema deleted for store: {}", policy_store_id);
        
        Ok(())
    }
    
    // ========================================================================
    // Policy Operations (DB + Cache)
    // ========================================================================
    
    /// Crea una política (DB + Cache)
    /// 
    /// # Arguments
    /// * `policy_store_id` - ID del policy store
    /// * `policy_id` - ID de la política
    /// * `statement` - Statement de Cedar
    /// * `description` - Descripción opcional
    /// 
    /// # Returns
    /// La política creada
    pub async fn create_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        // 1. Guardar en DB
        let policy = self.repository.create_policy(
            policy_store_id,
            policy_id,
            statement.clone(),
            description,
        ).await?;
        
        // 2. Actualizar cache
        let cache = self.get_cache(policy_store_id).await?;
        cache.add_policy(policy_id, &statement).await?;
        
        tracing::info!("Policy created: {} in store {}", policy_id, policy_store_id);
        
        Ok(policy)
    }
    
    /// Actualiza una política (DB + Cache)
    pub async fn update_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        // 1. Actualizar en DB
        let policy = self.repository.update_policy(
            policy_store_id,
            policy_id,
            statement.clone(),
            description,
        ).await?;
        
        // 2. Actualizar cache (remove + add)
        let cache = self.get_cache(policy_store_id).await?;
        cache.remove_policy(policy_id).await?;
        cache.add_policy(policy_id, &statement).await?;
        
        tracing::info!("Policy updated: {} in store {}", policy_id, policy_store_id);
        
        Ok(policy)
    }
    
    /// Elimina una política (DB + Cache)
    pub async fn delete_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<()> {
        // 1. Eliminar de DB
        self.repository.delete_policy(policy_store_id, policy_id).await?;
        
        // 2. Eliminar de cache
        let cache = self.get_cache(policy_store_id).await?;
        cache.remove_policy(policy_id).await?;
        
        tracing::info!("Policy deleted: {} from store {}", policy_id, policy_store_id);
        
        Ok(())
    }
    
    // ========================================================================
    // Synchronization
    // ========================================================================
    
    /// Recarga el cache de un policy store desde la DB
    /// 
    /// Útil para sincronizar cambios realizados externamente.
    /// 
    /// # Arguments
    /// * `policy_store_id` - ID del policy store
    pub async fn reload_cache(&self, policy_store_id: &str) -> Result<()> {
        let cache = self.get_cache(policy_store_id).await?;
        cache.load_from_db(self.repository.as_ref()).await?;
        
        tracing::info!("Cache reloaded for store: {}", policy_store_id);
        
        Ok(())
    }
    
    /// Recarga todos los caches desde la DB
    pub async fn reload_all_caches(&self) -> Result<()> {
        tracing::info!("Reloading all caches...");
        
        let cache_ids: Vec<String> = {
            let caches = self.caches.read().await;
            caches.keys().cloned().collect()
        };
        
        let mut reloaded = 0;
        let mut errors = 0;
        
        for policy_store_id in cache_ids {
            match self.reload_cache(&policy_store_id).await {
                Ok(_) => reloaded += 1,
                Err(e) => {
                    tracing::error!("Failed to reload cache for {}: {}", policy_store_id, e);
                    errors += 1;
                }
            }
        }
        
        tracing::info!("All caches reloaded: {} success, {} errors", reloaded, errors);
        
        Ok(())
    }
    
    // ========================================================================
    // Statistics
    // ========================================================================
    
    /// Obtiene estadísticas de todos los caches
    pub async fn stats(&self) -> Vec<crate::cache::policy_store_cache::CacheStats> {
        let caches = self.caches.read().await;
        let mut stats = Vec::new();
        
        for cache in caches.values() {
            stats.push(cache.stats().await);
        }
        
        stats
    }
    
    /// Obtiene el número de policy stores en cache
    pub async fn cache_count(&self) -> usize {
        self.caches.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Repository;
    
    async fn create_test_manager() -> CacheManager {
        let repo = Repository::new(":memory:").await.unwrap();
        CacheManager::new(Arc::new(repo))
    }
    
    #[tokio::test]
    async fn test_cache_manager_creation() {
        let manager = create_test_manager().await;
        assert_eq!(manager.cache_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_create_policy_store() {
        let manager = create_test_manager().await;
        
        let store = manager.create_policy_store(Some("Test".to_string())).await.unwrap();
        assert!(!store.id.is_empty());
        assert_eq!(manager.cache_count().await, 1);
    }
    
    #[tokio::test]
    async fn test_create_and_get_policy() {
        let manager = create_test_manager().await;
        
        let store = manager.create_policy_store(None).await.unwrap();
        
        let policy = r#"permit(principal, action, resource);"#;
        manager.create_policy(&store.id, "p1", policy.to_string(), None)
            .await
            .unwrap();
        
        let cache = manager.get_cache(&store.id).await.unwrap();
        let stats = cache.stats().await;
        assert_eq!(stats.policy_count, 1);
    }
    
    #[tokio::test]
    async fn test_initialize() {
        let repo = Repository::new(":memory:").await.unwrap();
        let repo_arc = Arc::new(repo);
        
        // Crear algunos stores directamente en el repo
        repo_arc.create_policy_store(Some("Store 1".to_string())).await.unwrap();
        repo_arc.create_policy_store(Some("Store 2".to_string())).await.unwrap();
        
        // Crear manager e inicializar
        let manager = CacheManager::new(repo_arc);
        manager.initialize().await.unwrap();
        
        assert_eq!(manager.cache_count().await, 2);
    }
}
