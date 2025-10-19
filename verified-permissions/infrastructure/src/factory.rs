//! Factory for creating repository instances

use crate::repository::RepositoryAdapter;

/// Creates a repository instance based on configuration
pub async fn create_repository(database_url: &str) -> anyhow::Result<RepositoryAdapter> {
    RepositoryAdapter::new(database_url).await
}
