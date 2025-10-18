//! PostgreSQL implementation of PolicyRepository

use crate::error::{AuthorizationError, Result};
use crate::storage::models::{IdentitySource, Policy, PolicyStore, Schema};
use crate::storage::repository_trait::{PolicyRepository, AuthorizationLog};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// PostgreSQL implementation of PolicyRepository
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    /// Create a new PostgreSQL repository
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Self::run_migrations(&pool).await?;
        Ok(Self { pool })
    }

    /// Run database migrations
    async fn run_migrations(pool: &PgPool) -> Result<()> {
        // Policy stores table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS policy_stores (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                description TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Schemas table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schemas (
                policy_store_id UUID PRIMARY KEY,
                schema_json TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Policies table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS policies (
                policy_store_id UUID NOT NULL,
                policy_id TEXT NOT NULL,
                statement TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (policy_store_id, policy_id),
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Identity sources table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS identity_sources (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                policy_store_id UUID NOT NULL,
                configuration_type TEXT NOT NULL,
                configuration_json TEXT NOT NULL,
                claims_mapping_json TEXT,
                description TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Authorization logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS authorization_logs (
                id SERIAL PRIMARY KEY,
                policy_store_id UUID NOT NULL,
                principal TEXT NOT NULL,
                action TEXT NOT NULL,
                resource TEXT NOT NULL,
                decision TEXT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl PolicyRepository for PostgresRepository {
    // Policy Store Operations
    async fn create_policy_store(&self, description: Option<String>) -> Result<PolicyStore> {
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO policy_stores (id, description) VALUES ($1, $2)",
        )
        .bind(id)
        .bind(&description)
        .execute(&self.pool)
        .await?;

        Ok(PolicyStore {
            id: id.to_string(),
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_policy_store(&self, id: &str) -> Result<PolicyStore> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", id)))?;

        let row = sqlx::query(
            "SELECT id, description, created_at, updated_at FROM policy_stores WHERE id = $1",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthorizationError::PolicyStoreNotFound(id.to_string()))?;

        Ok(PolicyStore {
            id: row.get::<Uuid, _>("id").to_string(),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_policy_stores(&self) -> Result<Vec<PolicyStore>> {
        let rows = sqlx::query(
            "SELECT id, description, created_at, updated_at FROM policy_stores ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| PolicyStore {
                id: row.get::<Uuid, _>("id").to_string(),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn delete_policy_store(&self, id: &str) -> Result<()> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", id)))?;

        let result = sqlx::query("DELETE FROM policy_stores WHERE id = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::PolicyStoreNotFound(id.to_string()));
        }

        Ok(())
    }

    // Schema Operations
    async fn put_schema(&self, policy_store_id: &str, schema: String) -> Result<()> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        sqlx::query(
            r#"
            INSERT INTO schemas (policy_store_id, schema_json, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (policy_store_id)
            DO UPDATE SET schema_json = $2, updated_at = NOW()
            "#,
        )
        .bind(store_uuid)
        .bind(schema)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_schema(&self, policy_store_id: &str) -> Result<Schema> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let row = sqlx::query(
            "SELECT policy_store_id, schema_json, created_at, updated_at FROM schemas WHERE policy_store_id = $1",
        )
        .bind(store_uuid)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthorizationError::SchemaNotFound(policy_store_id.to_string()))?;

        Ok(Schema {
            policy_store_id: row.get::<Uuid, _>("policy_store_id").to_string(),
            schema_json: row.get("schema_json"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn delete_schema(&self, policy_store_id: &str) -> Result<()> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let result = sqlx::query("DELETE FROM schemas WHERE policy_store_id = $1")
            .bind(store_uuid)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Authorization::SchemaNotFound(policy_store_id.to_string()));
        }

        Ok(())
    }

    // Policy Operations
    async fn create_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        sqlx::query(
            "INSERT INTO policies (policy_store_id, policy_id, statement, description) VALUES ($1, $2, $3, $4)",
        )
        .bind(store_uuid)
        .bind(policy_id)
        .bind(&statement)
        .bind(&description)
        .execute(&self.pool)
        .await?;

        Ok(Policy {
            policy_store_id: policy_store_id.to_string(),
            policy_id: policy_id.to_string(),
            statement,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<Policy> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let row = sqlx::query(
            "SELECT policy_store_id, policy_id, statement, description, created_at, updated_at FROM policies WHERE policy_store_id = $1 AND policy_id = $2",
        )
        .bind(store_uuid)
        .bind(policy_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthorizationError::PolicyNotFound(format!("{}:{}", policy_store_id, policy_id)))?;

        Ok(Policy {
            policy_store_id: row.get::<Uuid, _>("policy_store_id").to_string(),
            policy_id: row.get("policy_id"),
            statement: row.get("statement"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_policies(&self, policy_store_id: &str) -> Result<Vec<Policy>> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let rows = sqlx::query(
            "SELECT policy_store_id, policy_id, statement, description, created_at, updated_at FROM policies WHERE policy_store_id = $1 ORDER BY created_at",
        )
        .bind(store_uuid)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Policy {
                policy_store_id: row.get::<Uuid, _>("policy_store_id").to_string(),
                policy_id: row.get("policy_id"),
                statement: row.get("statement"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn update_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let result = sqlx::query(
            "UPDATE policies SET statement = $1, description = $2, updated_at = NOW() WHERE policy_store_id = $3 AND policy_id = $4",
        )
        .bind(&statement)
        .bind(&description)
        .bind(store_uuid)
        .bind(policy_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::PolicyNotFound(format!("{}:{}", policy_store_id, policy_id)));
        }

        Ok(Policy {
            policy_store_id: policy_store_id.to_string(),
            policy_id: policy_id.to_string(),
            statement,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn delete_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<()> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let result = sqlx::query("DELETE FROM policies WHERE policy_store_id = $1 AND policy_id = $2")
            .bind(store_uuid)
            .bind(policy_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::PolicyNotFound(format!("{}:{}", policy_store_id, policy_id)));
        }

        Ok(())
    }

    // Identity Source Operations
    async fn create_identity_source(
        &self,
        policy_store_id: &str,
        provider_type: &str,
        config: &str,
        claims_mapping: Option<&str>,
        description: Option<&str>,
    ) -> Result<IdentitySource> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO identity_sources (id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(store_uuid)
        .bind(provider_type)
        .bind(config)
        .bind(claims_mapping)
        .bind(description)
        .execute(&self.pool)
        .await?;

        Ok(IdentitySource {
            id: id.to_string(),
            policy_store_id: policy_store_id.to_string(),
            configuration_type: provider_type.to_string(),
            configuration_json: config.to_string(),
            claims_mapping_json: claims_mapping.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<IdentitySource> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;
        let source_uuid = Uuid::parse_str(identity_source_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid identity source ID: {}", identity_source_id)))?;

        let row = sqlx::query(
            "SELECT id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description, created_at, updated_at FROM identity_sources WHERE policy_store_id = $1 AND id = $2",
        )
        .bind(store_uuid)
        .bind(source_uuid)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthorizationError::NotFound(format!("Identity source not found: {}", identity_source_id)))?;

        Ok(IdentitySource {
            id: row.get::<Uuid, _>("id").to_string(),
            policy_store_id: row.get::<Uuid, _>("policy_store_id").to_string(),
            configuration_type: row.get("configuration_type"),
            configuration_json: row.get("configuration_json"),
            claims_mapping_json: row.get("claims_mapping_json"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_identity_sources(&self, policy_store_id: &str) -> Result<Vec<IdentitySource>> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;

        let rows = sqlx::query(
            "SELECT id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description, created_at, updated_at FROM identity_sources WHERE policy_store_id = $1 ORDER BY created_at",
        )
        .bind(store_uuid)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| IdentitySource {
                id: row.get::<Uuid, _>("id").to_string(),
                policy_store_id: row.get::<Uuid, _>("policy_store_id").to_string(),
                configuration_type: row.get("configuration_type"),
                configuration_json: row.get("configuration_json"),
                claims_mapping_json: row.get("claims_mapping_json"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn delete_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<()> {
        let store_uuid = Uuid::parse_str(policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", policy_store_id)))?;
        let source_uuid = Uuid::parse_str(identity_source_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid identity source ID: {}", identity_source_id)))?;

        let result = sqlx::query("DELETE FROM identity_sources WHERE policy_store_id = $1 AND id = $2")
            .bind(store_uuid)
            .bind(source_uuid)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::NotFound(format!("Identity source not found: {}", identity_source_id)));
        }

        Ok(())
    }

    // Audit Operations
    async fn log_authorization(&self, log: AuthorizationLog) -> Result<()> {
        let store_uuid = Uuid::parse_str(&log.policy_store_id)
            .map_err(|_| AuthorizationError::NotFound(format!("Invalid policy store ID: {}", log.policy_store_id)))?;

        sqlx::query(
            "INSERT INTO authorization_logs (policy_store_id, principal, action, resource, decision, timestamp) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(store_uuid)
        .bind(&log.principal)
        .bind(&log.action)
        .bind(&log.resource)
        .bind(&log.decision)
        .bind(log.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
