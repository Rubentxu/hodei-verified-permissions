//! Factory for creating repository instances

use crate::repository::Repository;

/// Creates a repository instance based on configuration
pub async fn create_repository(database_url: &str) -> anyhow::Result<Repository> {
    Repository::new(database_url).await
}
