//! Background task for automatic cache reloading
//!
//! This module provides a background task that periodically reloads
//! the cache from the database to ensure fresh policy data.

use crate::cache::CacheManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

/// Configuration for the reload task
#[derive(Debug, Clone)]
pub struct ReloadConfig {
    /// Interval between reloads in seconds
    pub interval_secs: u64,
    
    /// Whether the reload task is enabled
    pub enabled: bool,
}

impl Default for ReloadConfig {
    fn default() -> Self {
        Self {
            interval_secs: 300, // 5 minutes
            enabled: true,
        }
    }
}

/// Background task for reloading cache
pub struct ReloadTask {
    cache_manager: Arc<CacheManager>,
    config: ReloadConfig,
}

impl ReloadTask {
    /// Create a new reload task
    pub fn new(cache_manager: Arc<CacheManager>, config: ReloadConfig) -> Self {
        Self {
            cache_manager,
            config,
        }
    }

    /// Start the reload task in the background
    ///
    /// Returns a handle to the spawned task
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.run().await;
        })
    }

    /// Run the reload loop
    async fn run(self) {
        if !self.config.enabled {
            info!("Cache reload task is disabled");
            return;
        }

        info!(
            "Starting cache reload task (interval: {}s)",
            self.config.interval_secs
        );

        let mut interval = time::interval(Duration::from_secs(self.config.interval_secs));
        
        // Skip the first tick (immediate)
        interval.tick().await;

        loop {
            interval.tick().await;
            
            info!("Running cache reload...");
            
            match self.cache_manager.reload_all().await {
                Ok(count) => {
                    info!("Successfully reloaded {} policy stores", count);
                }
                Err(e) => {
                    error!("Failed to reload cache: {}", e);
                    // Continue running even if reload fails
                }
            }
        }
    }
}

/// Extension trait for CacheManager to support reload operations
impl CacheManager {
    /// Reload all caches from the database
    ///
    /// Returns the number of policy stores reloaded
    pub async fn reload_all(&self) -> crate::error::Result<usize> {
        info!("Reloading all policy store caches...");
        
        // Simply reinitialize all caches
        self.initialize().await?;
        
        // Count the number of stores
        let stores = self.repository.list_policy_stores().await?;
        let count = stores.len();
        
        info!("Reloaded {} policy stores", count);
        
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Repository;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_reload_config_default() {
        let config = ReloadConfig::default();
        assert_eq!(config.interval_secs, 300);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_reload_task_creation() {
        let repo = Repository::new(":memory:").await.unwrap();
        let cache_manager = Arc::new(CacheManager::new(Arc::new(repo)));
        
        let config = ReloadConfig {
            interval_secs: 1,
            enabled: false, // Disabled for test
        };
        
        let task = ReloadTask::new(cache_manager, config);
        assert!(!task.config.enabled);
    }

    #[tokio::test]
    async fn test_reload_all() {
        let repo = Arc::new(Repository::new(":memory:").await.unwrap());
        
        // Create some policy stores
        repo.create_policy_store(Some("Store 1".to_string())).await.unwrap();
        repo.create_policy_store(Some("Store 2".to_string())).await.unwrap();
        
        let cache_manager = CacheManager::new(repo);
        cache_manager.initialize().await.unwrap();
        
        // Reload all
        let count = cache_manager.reload_all().await.unwrap();
        assert_eq!(count, 2);
    }
}
