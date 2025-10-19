//! Factory for creating repository instances based on configuration

use crate::config::{DatabaseConfig, DatabaseProvider};
use crate::error::Result;
use crate::storage::{PolicyRepository, Repository};
#[cfg(feature = "postgres")]
use crate::storage::PostgresRepository;
#[cfg(feature = "surreal")]
use crate::storage::SurrealRepository;
use std::sync::Arc;

/// Creates a repository instance based on the database configuration
/// 
/// # Arguments
/// * `config` - Database configuration
/// 
/// # Returns
/// Arc-wrapped repository implementing PolicyRepository trait
/// 
/// # Examples
/// ```no_run
/// use hodei_verified_permissions::config::{DatabaseConfig, DatabaseProvider};
/// use hodei_verified_permissions::storage::create_repository;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseConfig {
///     provider: DatabaseProvider::Sqlite,
///     url: "sqlite::memory:".to_string(),
///     max_connections: 10,
/// };
/// 
/// let repo = create_repository(&config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_repository(config: &DatabaseConfig) -> Result<Arc<dyn PolicyRepository>> {
    match config.provider {
        DatabaseProvider::Sqlite => {
            tracing::info!("Creating SQLite repository: {}", config.url);
            let repo = Repository::new(&config.url).await?;
            Ok(Arc::new(repo))
        }
        DatabaseProvider::Postgres => {
            tracing::info!("Creating PostgreSQL repository: {}", config.url);
            #[cfg(feature = "postgres")]
            {
                let repo = PostgresRepository::new(&config.url).await?;
                return Ok(Arc::new(repo));
            }
            #[cfg(not(feature = "postgres"))]
            {
                return Err(crate::error::AuthorizationError::Internal(
                    "PostgreSQL feature not enabled. Build with --features postgres".to_string(),
                ));
            }
        }
        DatabaseProvider::Surreal => {
            tracing::info!("Creating SurrealDB repository: {}", config.url);
            #[cfg(feature = "surreal")]
            {
                let repo = SurrealRepository::new(&config.url).await?;
                return Ok(Arc::new(repo));
            }
            #[cfg(not(feature = "surreal"))]
            {
                return Err(crate::error::AuthorizationError::Internal(
                    "SurrealDB feature not enabled. Build with --features surreal".to_string(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseProvider;
    
    #[tokio::test]
    async fn test_create_sqlite_repository() {
        let config = DatabaseConfig {
            provider: DatabaseProvider::Sqlite,
            url: "sqlite::memory:".to_string(),
            max_connections: 10,
        };
        
        let repo = create_repository(&config).await;
        assert!(repo.is_ok());
    }
    
    #[tokio::test]
    async fn test_postgres_not_implemented() {
        let config = DatabaseConfig {
            provider: DatabaseProvider::Postgres,
            url: "postgresql://localhost/test".to_string(),
            max_connections: 10,
        };
        
        let repo = create_repository(&config).await;
        assert!(repo.is_err());
    }
    
    #[tokio::test]
    async fn test_surreal_not_implemented() {
        let config = DatabaseConfig {
            provider: DatabaseProvider::Surreal,
            url: "ws://localhost:8000".to_string(),
            max_connections: 10,
        };
        
        let repo = create_repository(&config).await;
        assert!(repo.is_err());
    }
}
