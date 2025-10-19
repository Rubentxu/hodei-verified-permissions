//! SQLite repository implementation

use hodei_domain::{
    PolicyRepository, AuthorizationLog,
    PolicyStore, Policy, Schema, IdentitySource, PolicyTemplate,
    PolicyStoreId, PolicyId, CedarPolicy, IdentitySourceType,
    Principal, Action, Resource, AuthorizationDecision,
    DomainError, DomainResult,
};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl Repository {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Run migrations
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS policy_stores (
                id TEXT PRIMARY KEY,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schemas (
                policy_store_id TEXT PRIMARY KEY,
                schema_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS policies (
                policy_store_id TEXT NOT NULL,
                policy_id TEXT NOT NULL,
                statement TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (policy_store_id, policy_id),
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS identity_sources (
                id TEXT PRIMARY KEY,
                policy_store_id TEXT NOT NULL,
                configuration_type TEXT NOT NULL,
                configuration_json TEXT NOT NULL,
                claims_mapping_json TEXT,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS policy_templates (
                template_id TEXT NOT NULL,
                policy_store_id TEXT NOT NULL,
                statement TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (policy_store_id, template_id),
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS authorization_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                policy_store_id TEXT NOT NULL,
                principal TEXT NOT NULL,
                action TEXT NOT NULL,
                resource TEXT NOT NULL,
                decision TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (policy_store_id) REFERENCES policy_stores(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    // ========================================================================
    // Policy Store Operations
    // ========================================================================

    pub async fn create_policy_store(&self, description: Option<String>) -> Result<PolicyStore> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO policy_stores (id, description, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&description)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(PolicyStore {
            id,
            description,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_policy_store(&self, id: &str) -> Result<PolicyStore> {
        let row = sqlx::query(
            "SELECT id, description, created_at, updated_at FROM policy_stores WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthorizationError::PolicyStoreNotFound(id.to_string()))?;

        Ok(PolicyStore {
            id: row.get("id"),
            description: row.get("description"),
            created_at: row.get::<String, _>("created_at").parse().unwrap(),
            updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
        })
    }

    pub async fn list_policy_stores(&self) -> Result<Vec<PolicyStore>> {
        let rows = sqlx::query(
            "SELECT id, description, created_at, updated_at FROM policy_stores ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| PolicyStore {
                id: row.get("id"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            })
            .collect())
    }

    pub async fn delete_policy_store(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM policy_stores WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::PolicyStoreNotFound(id.to_string()));
        }

        Ok(())
    }

    // ========================================================================
    // Schema Operations
    // ========================================================================

    pub async fn put_schema(&self, policy_store_id: &str, schema_json: String) -> Result<Schema> {
        // Verify policy store exists
        self.get_policy_store(policy_store_id).await?;

        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO schemas (policy_store_id, schema_json, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(policy_store_id) DO UPDATE SET
                schema_json = excluded.schema_json,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(policy_store_id)
        .bind(&schema_json)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(Schema {
            policy_store_id: policy_store_id.to_string(),
            schema_json,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_schema(&self, policy_store_id: &str) -> Result<Schema> {
        let row = sqlx::query(
            "SELECT policy_store_id, schema_json, created_at, updated_at FROM schemas WHERE policy_store_id = ?",
        )
        .bind(policy_store_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthorizationError::SchemaNotFound(policy_store_id.to_string()))?;

        Ok(Schema {
            policy_store_id: row.get("policy_store_id"),
            schema_json: row.get("schema_json"),
            created_at: row.get::<String, _>("created_at").parse().unwrap(),
            updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
        })
    }

    pub async fn delete_schema(&self, policy_store_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM schemas WHERE policy_store_id = ?")
            .bind(policy_store_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::SchemaNotFound(policy_store_id.to_string()));
        }

        Ok(())
    }

    // ========================================================================
    // Policy Operations
    // ========================================================================

    pub async fn create_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        // Verify policy store exists
        self.get_policy_store(policy_store_id).await?;

        let now = Utc::now();

        sqlx::query(
            "INSERT INTO policies (policy_store_id, policy_id, statement, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(policy_store_id)
        .bind(policy_id)
        .bind(&statement)
        .bind(&description)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(Policy {
            policy_store_id: policy_store_id.to_string(),
            policy_id: policy_id.to_string(),
            statement,
            description,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<Policy> {
        let row = sqlx::query(
            "SELECT policy_store_id, policy_id, statement, description, created_at, updated_at FROM policies WHERE policy_store_id = ? AND policy_id = ?",
        )
        .bind(policy_store_id)
        .bind(policy_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AuthorizationError::PolicyNotFound(policy_id.to_string()))?;

        Ok(Policy {
            policy_store_id: row.get("policy_store_id"),
            policy_id: row.get("policy_id"),
            statement: row.get("statement"),
            description: row.get("description"),
            created_at: row.get::<String, _>("created_at").parse().unwrap(),
            updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
        })
    }

    pub async fn update_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE policies SET statement = ?, description = ?, updated_at = ? WHERE policy_store_id = ? AND policy_id = ?",
        )
        .bind(&statement)
        .bind(&description)
        .bind(now.to_rfc3339())
        .bind(policy_store_id)
        .bind(policy_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::PolicyNotFound(policy_id.to_string()));
        }

        Ok(Policy {
            policy_store_id: policy_store_id.to_string(),
            policy_id: policy_id.to_string(),
            statement,
            description,
            created_at: now, // We don't have the original created_at, but it's not critical
            updated_at: now,
        })
    }

    pub async fn delete_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM policies WHERE policy_store_id = ? AND policy_id = ?",
        )
        .bind(policy_store_id)
        .bind(policy_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::PolicyNotFound(policy_id.to_string()));
        }

        Ok(())
    }

    pub async fn list_policies(&self, policy_store_id: &str) -> Result<Vec<Policy>> {
        let rows = sqlx::query(
            "SELECT policy_store_id, policy_id, statement, description, created_at, updated_at FROM policies WHERE policy_store_id = ? ORDER BY created_at DESC",
        )
        .bind(policy_store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Policy {
                policy_store_id: row.get("policy_store_id"),
                policy_id: row.get("policy_id"),
                statement: row.get("statement"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            })
            .collect())
    }

    // ========================================================================
    // Identity Source Operations (Épica 4 - HU 4.1)
    // ========================================================================

    pub async fn create_identity_source(
        &self,
        policy_store_id: &str,
        configuration_type: &str,
        configuration_json: &str,
        claims_mapping_json: Option<&str>,
        description: Option<&str>,
    ) -> Result<IdentitySource> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO identity_sources (id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(policy_store_id)
        .bind(configuration_type)
        .bind(configuration_json)
        .bind(claims_mapping_json)
        .bind(description)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(IdentitySource {
            id,
            policy_store_id: policy_store_id.to_string(),
            configuration_type: configuration_type.to_string(),
            configuration_json: configuration_json.to_string(),
            claims_mapping_json: claims_mapping_json.map(String::from),
            description: description.map(String::from),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<IdentitySource> {
        let row = sqlx::query(
            "SELECT id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description, created_at, updated_at FROM identity_sources WHERE policy_store_id = ? AND id = ?",
        )
        .bind(policy_store_id)
        .bind(identity_source_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(IdentitySource {
                id: row.get("id"),
                policy_store_id: row.get("policy_store_id"),
                configuration_type: row.get("configuration_type"),
                configuration_json: row.get("configuration_json"),
                claims_mapping_json: row.get("claims_mapping_json"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            }),
            None => Err(AuthorizationError::NotFound(format!(
                "Identity source {} not found",
                identity_source_id
            ))),
        }
    }

    pub async fn list_identity_sources(&self, policy_store_id: &str) -> Result<Vec<IdentitySource>> {
        let rows = sqlx::query(
            "SELECT id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description, created_at, updated_at FROM identity_sources WHERE policy_store_id = ? ORDER BY created_at DESC",
        )
        .bind(policy_store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| IdentitySource {
                id: row.get("id"),
                policy_store_id: row.get("policy_store_id"),
                configuration_type: row.get("configuration_type"),
                configuration_json: row.get("configuration_json"),
                claims_mapping_json: row.get("claims_mapping_json"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            })
            .collect())
    }

    pub async fn delete_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM identity_sources WHERE policy_store_id = ? AND id = ?",
        )
        .bind(policy_store_id)
        .bind(identity_source_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::NotFound(format!(
                "Identity source {} not found",
                identity_source_id
            )));
        }

        Ok(())
    }

    // ========================================================================
    // Policy Template Operations (Épica 6 - HU 6.1)
    // ========================================================================

    pub async fn create_policy_template(
        &self,
        policy_store_id: &str,
        template_id: &str,
        statement: &str,
        description: Option<&str>,
    ) -> Result<PolicyTemplate> {
        let now = Utc::now();

        // Validate template syntax (check for placeholders)
        if !statement.contains("?principal") && !statement.contains("?resource") {
            return Err(AuthorizationError::InvalidPolicy(
                "Template must contain at least one placeholder (?principal or ?resource)".to_string()
            ));
        }

        sqlx::query(
            "INSERT INTO policy_templates (template_id, policy_store_id, statement, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(template_id)
        .bind(policy_store_id)
        .bind(statement)
        .bind(description)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(PolicyTemplate {
            template_id: template_id.to_string(),
            policy_store_id: policy_store_id.to_string(),
            statement: statement.to_string(),
            description: description.map(String::from),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_policy_template(
        &self,
        policy_store_id: &str,
        template_id: &str,
    ) -> Result<PolicyTemplate> {
        let row = sqlx::query(
            "SELECT template_id, policy_store_id, statement, description, created_at, updated_at FROM policy_templates WHERE policy_store_id = ? AND template_id = ?",
        )
        .bind(policy_store_id)
        .bind(template_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(PolicyTemplate {
                template_id: row.get("template_id"),
                policy_store_id: row.get("policy_store_id"),
                statement: row.get("statement"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            }),
            None => Err(AuthorizationError::NotFound(format!(
                "Policy template {} not found",
                template_id
            ))),
        }
    }

    pub async fn list_policy_templates(&self, policy_store_id: &str) -> Result<Vec<PolicyTemplate>> {
        let rows = sqlx::query(
            "SELECT template_id, policy_store_id, statement, description, created_at, updated_at FROM policy_templates WHERE policy_store_id = ? ORDER BY created_at DESC",
        )
        .bind(policy_store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| PolicyTemplate {
                template_id: row.get("template_id"),
                policy_store_id: row.get("policy_store_id"),
                statement: row.get("statement"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            })
            .collect())
    }

    pub async fn delete_policy_template(
        &self,
        policy_store_id: &str,
        template_id: &str,
    ) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM policy_templates WHERE policy_store_id = ? AND template_id = ?",
        )
        .bind(policy_store_id)
        .bind(template_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthorizationError::NotFound(format!(
                "Policy template {} not found",
                template_id
            )));
        }

        Ok(())
    }

    // ========================================================================
    // Audit Operations
    // ========================================================================

    pub async fn log_authorization(&self, log: AuthorizationLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO authorization_logs (
                policy_store_id, principal, action, resource, decision, timestamp
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&log.policy_store_id)
        .bind(&log.principal)
        .bind(&log.action)
        .bind(&log.resource)
        .bind(&log.decision)
        .bind(log.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ============================================================================
// PolicyRepository Trait Implementation
// ============================================================================

#[async_trait]
impl PolicyRepository for Repository {
    // Policy Store Operations
    async fn create_policy_store(&self, description: Option<String>) -> Result<PolicyStore> {
        self.create_policy_store(description).await
    }
    
    async fn get_policy_store(&self, id: &str) -> Result<PolicyStore> {
        self.get_policy_store(id).await
    }
    
    async fn list_policy_stores(&self) -> Result<Vec<PolicyStore>> {
        self.list_policy_stores().await
    }
    
    async fn delete_policy_store(&self, id: &str) -> Result<()> {
        self.delete_policy_store(id).await
    }
    
    // Schema Operations
    async fn put_schema(&self, policy_store_id: &str, schema: String) -> Result<()> {
        self.put_schema(policy_store_id, schema).await?;
        Ok(())
    }
    
    async fn get_schema(&self, policy_store_id: &str) -> Result<Schema> {
        self.get_schema(policy_store_id).await
    }
    
    async fn delete_schema(&self, policy_store_id: &str) -> Result<()> {
        self.delete_schema(policy_store_id).await
    }
    
    // Policy Operations
    async fn create_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        self.create_policy(policy_store_id, policy_id, statement, description).await
    }
    
    async fn get_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<Policy> {
        self.get_policy(policy_store_id, policy_id).await
    }
    
    async fn list_policies(&self, policy_store_id: &str) -> Result<Vec<Policy>> {
        self.list_policies(policy_store_id).await
    }
    
    async fn update_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy> {
        self.update_policy(policy_store_id, policy_id, statement, description).await
    }
    
    async fn delete_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<()> {
        self.delete_policy(policy_store_id, policy_id).await
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
        self.create_identity_source(
            policy_store_id,
            provider_type,
            config,
            claims_mapping,
            description,
        ).await
    }
    
    async fn get_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<IdentitySource> {
        self.get_identity_source(policy_store_id, identity_source_id).await
    }
    
    async fn list_identity_sources(&self, policy_store_id: &str) -> Result<Vec<IdentitySource>> {
        self.list_identity_sources(policy_store_id).await
    }
    
    async fn delete_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<()> {
        self.delete_identity_source(policy_store_id, identity_source_id).await
    }
    
    // Audit Operations
    async fn log_authorization(&self, log: AuthorizationLog) -> Result<()> {
        self.log_authorization(log).await
    }
}
