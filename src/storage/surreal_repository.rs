//! SurrealDB implementation of PolicyRepository

use crate::error::{AuthorizationError, Result};
use crate::storage::models::{IdentitySource, Policy, PolicyStore, Schema};
use crate::storage::repository_trait::{PolicyRepository, AuthorizationLog};
use async_trait::async_trait;
use chrono::Utc;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

/// SurrealDB implementation of PolicyRepository
pub struct SurrealRepository {
    db: Surreal<surrealdb::engine::remote::ws::Client>,
}

impl SurrealRepository {
    /// Create a new SurrealDB repository
    pub async fn new(connection_string: &str) -> Result<Self> {
        let db = Surreal::new::<Ws>(connection_string).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        // Sign in as root (for development - in production use proper auth)
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        // Select namespace and database
        db.use_ns("hodei").use_db("permissions").await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Self::run_migrations(&db).await?;

        Ok(Self { db })
    }

    /// Run database migrations
    async fn run_migrations(db: &Surreal<surrealdb::engine::remote::ws::Client>) -> Result<()> {
        // Define schemas using SurrealQL
        let migrations = vec![
            // Policy stores table
            r#"
            DEFINE TABLE policy_stores SCHEMAFULL;
            DEFINE FIELD id ON TABLE policy_stores TYPE string;
            DEFINE FIELD description ON TABLE policy_stores TYPE option<string>;
            DEFINE FIELD created_at ON TABLE policy_stores TYPE datetime VALUE time::now();
            DEFINE FIELD updated_at ON TABLE policy_stores TYPE datetime VALUE time::now();
            "#,

            // Schemas table
            r#"
            DEFINE TABLE schemas SCHEMAFULL;
            DEFINE FIELD policy_store_id ON TABLE schemas TYPE string;
            DEFINE FIELD schema_json ON TABLE schemas TYPE string;
            DEFINE FIELD created_at ON TABLE schemas TYPE datetime VALUE time::now();
            DEFINE FIELD updated_at ON TABLE schemas TYPE datetime VALUE time::now();
            "#,

            // Policies table
            r#"
            DEFINE TABLE policies SCHEMAFULL;
            DEFINE FIELD policy_store_id ON TABLE policies TYPE string;
            DEFINE FIELD policy_id ON TABLE policies TYPE string;
            DEFINE FIELD statement ON TABLE policies TYPE string;
            DEFINE FIELD description ON TABLE policies TYPE option<string>;
            DEFINE FIELD created_at ON TABLE policies TYPE datetime VALUE time::now();
            DEFINE FIELD updated_at ON TABLE policies TYPE datetime VALUE time::now();
            "#,

            // Identity sources table
            r#"
            DEFINE TABLE identity_sources SCHEMAFULL;
            DEFINE FIELD id ON TABLE identity_sources TYPE string;
            DEFINE FIELD policy_store_id ON TABLE identity_sources TYPE string;
            DEFINE FIELD configuration_type ON TABLE identity_sources TYPE string;
            DEFINE FIELD configuration_json ON TABLE identity_sources TYPE string;
            DEFINE FIELD claims_mapping_json ON TABLE identity_sources TYPE option<string>;
            DEFINE FIELD description ON TABLE identity_sources TYPE option<string>;
            DEFINE FIELD created_at ON TABLE identity_sources TYPE datetime VALUE time::now();
            DEFINE FIELD updated_at ON TABLE identity_sources TYPE datetime VALUE time::now();
            "#,

            // Authorization logs table
            r#"
            DEFINE TABLE authorization_logs SCHEMAFULL;
            DEFINE FIELD policy_store_id ON TABLE authorization_logs TYPE string;
            DEFINE FIELD principal ON TABLE authorization_logs TYPE string;
            DEFINE FIELD action ON TABLE authorization_logs TYPE string;
            DEFINE FIELD resource ON TABLE authorization_logs TYPE string;
            DEFINE FIELD decision ON TABLE authorization_logs TYPE string;
            DEFINE FIELD timestamp ON TABLE authorization_logs TYPE datetime VALUE time::now();
            "#,
        ];

        for migration in migrations {
            db.query(migration).await
                .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;
        }

        Ok(())
    }

    /// Generate unique ID for SurrealDB
    fn generate_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:x}", timestamp)
    }
}

#[async_trait]
impl PolicyRepository for SurrealRepository {
    // Policy Store Operations
    async fn create_policy_store(&self, description: Option<String>) -> Result<PolicyStore> {
        let id = self.generate_id();

        let sql = format!(
            "CREATE policy_stores:{} SET id = '{}', description = {}",
            id,
            id,
            description.as_ref()
                .map(|d| format!("'{}'", d))
                .unwrap_or("NONE".to_string())
        );

        self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Ok(PolicyStore {
            id,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_policy_store(&self, id: &str) -> Result<PolicyStore> {
        let sql = format!("SELECT * FROM policy_stores:{}", id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let store: Option<PolicyStore> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        store.ok_or_else(|| AuthorizationError::PolicyStoreNotFound(id.to_string()))
    }

    async fn list_policy_stores(&self) -> Result<Vec<PolicyStore>> {
        let mut result = self.db.query("SELECT * FROM policy_stores ORDER BY created_at DESC").await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let stores: Vec<PolicyStore> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Ok(stores)
    }

    async fn delete_policy_store(&self, id: &str) -> Result<()> {
        let sql = format!("DELETE policy_stores:{}", id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let deleted: Option<PolicyStore> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        if deleted.is_none() {
            return Err(AuthorizationError::PolicyStoreNotFound(id.to_string()));
        }

        Ok(())
    }

    // Schema Operations
    async fn put_schema(&self, policy_store_id: &str, schema: String) -> Result<()> {
        let id = format!("schemas:{}", policy_store_id);

        let sql = format!(
            "UPSERT {} SET policy_store_id = '{}', schema_json = '{}'",
            id, policy_store_id, schema.replace("'", "\\'")
        );

        self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Ok(())
    }

    async fn get_schema(&self, policy_store_id: &str) -> Result<Schema> {
        let sql = format!("SELECT * FROM schemas:{}", policy_store_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let schema: Option<Schema> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        schema.ok_or_else(|| AuthorizationError::SchemaNotFound(policy_store_id.to_string()))
    }

    async fn delete_schema(&self, policy_store_id: &str) -> Result<()> {
        let sql = format!("DELETE schemas:{}", policy_store_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let deleted: Option<Schema> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        if deleted.is_none() {
            return Err(AuthorizationError::SchemaNotFound(policy_store_id.to_string()));
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
        let record_id = format!("policies:{}:{}", policy_store_id, policy_id);

        let sql = format!(
            "CREATE {} SET policy_store_id = '{}', policy_id = '{}', statement = '{}', description = {}",
            record_id, policy_store_id, policy_id,
            statement.replace("'", "\\'"),
            description.as_ref()
                .map(|d| format!("'{}'", d.replace("'", "\\'")))
                .unwrap_or("NONE".to_string())
        );

        self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

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
        let record_id = format!("policies:{}:{}", policy_store_id, policy_id);
        let sql = format!("SELECT * FROM {}", record_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let policy: Option<Policy> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        policy.ok_or_else(|| AuthorizationError::PolicyNotFound(format!("{}:{}", policy_store_id, policy_id)))
    }

    async fn list_policies(&self, policy_store_id: &str) -> Result<Vec<Policy>> {
        let sql = format!("SELECT * FROM policies WHERE policy_store_id = '{}' ORDER BY created_at", policy_store_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let policies: Vec<Policy> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Ok(policies)
    }

    async fn update_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        let record_id = format!("policies:{}:{}", policy_store_id, policy_id);

        let sql = format!(
            "UPDATE {} SET statement = '{}', description = {}, updated_at = time::now()",
            record_id,
            statement.replace("'", "\\'"),
            description.as_ref()
                .map(|d| format!("'{}'", d.replace("'", "\\'")))
                .unwrap_or("NONE".to_string())
        );

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let updated: Option<Policy> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        updated.ok_or_else(|| AuthorizationError::PolicyNotFound(format!("{}:{}", policy_store_id, policy_id)))
    }

    async fn delete_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<()> {
        let record_id = format!("policies:{}:{}", policy_store_id, policy_id);
        let sql = format!("DELETE {}", record_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let deleted: Option<Policy> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        if deleted.is_none() {
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
        let id = self.generate_id();
        let record_id = format!("identity_sources:{}", id);

        let sql = format!(
            "CREATE {} SET id = '{}', policy_store_id = '{}', configuration_type = '{}', configuration_json = '{}', claims_mapping_json = {}, description = {}",
            record_id, id, policy_store_id, provider_type,
            config.replace("'", "\\'"),
            claims_mapping.map(|c| format!("'{}'", c.replace("'", "\\'"))).unwrap_or("NONE".to_string()),
            description.map(|d| format!("'{}'", d.replace("'", "\\'"))).unwrap_or("NONE".to_string())
        );

        self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Ok(IdentitySource {
            id,
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
        let record_id = format!("identity_sources:{}", identity_source_id);
        let sql = format!("SELECT * FROM {} WHERE policy_store_id = '{}'", record_id, policy_store_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let source: Option<IdentitySource> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        source.ok_or_else(|| AuthorizationError::NotFound(format!("Identity source not found: {}", identity_source_id)))
    }

    async fn list_identity_sources(&self, policy_store_id: &str) -> Result<Vec<IdentitySource>> {
        let sql = format!("SELECT * FROM identity_sources WHERE policy_store_id = '{}' ORDER BY created_at", policy_store_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let sources: Vec<IdentitySource> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Ok(sources)
    }

    async fn delete_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<()> {
        let record_id = format!("identity_sources:{}", identity_source_id);
        let sql = format!("DELETE {} WHERE policy_store_id = '{}'", record_id, policy_store_id);

        let mut result = self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        let deleted: Option<IdentitySource> = result.take(0)
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        if deleted.is_none() {
            return Err(AuthorizationError::NotFound(format!("Identity source not found: {}", identity_source_id)));
        }

        Ok(())
    }

    // Audit Operations
    async fn log_authorization(&self, log: AuthorizationLog) -> Result<()> {
        let sql = format!(
            "CREATE authorization_logs SET policy_store_id = '{}', principal = '{}', action = '{}', resource = '{}', decision = '{}', timestamp = '{}'",
            log.policy_store_id,
            log.principal.replace("'", "\\'"),
            log.action.replace("'", "\\'"),
            log.resource.replace("'", "\\'"),
            log.decision,
            log.timestamp.to_rfc3339()
        );

        self.db.query(&sql).await
            .map_err(|e| AuthorizationError::DatabaseError(sqlx::Error::from(e)))?;

        Ok(())
    }
}
