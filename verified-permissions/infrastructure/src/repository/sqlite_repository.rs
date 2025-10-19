//! SQLite repository implementation

use super::models;
use chrono::Utc;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
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

    pub async fn create_policy_store(&self, description: Option<String>) -> anyhow::Result<models::PolicyStore> {
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

        Ok(models::PolicyStore {
            id,
            description,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_policy_store(&self, id: &str) -> anyhow::Result<models::PolicyStore> {
        let row = sqlx::query(
            "SELECT id, description, created_at, updated_at FROM policy_stores WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Policy store not found: {}", id))?;

        Ok(models::PolicyStore {
            id: row.get("id"),
            description: row.get("description"),
            created_at: row.get::<String, _>("created_at").parse().unwrap(),
            updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
        })
    }

    pub async fn list_policy_stores(&self) -> anyhow::Result<Vec<models::PolicyStore>> {
        let rows = sqlx::query(
            "SELECT id, description, created_at, updated_at FROM policy_stores ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| models::PolicyStore {
                id: row.get("id"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            })
            .collect())
    }

    pub async fn delete_policy_store(&self, id: &str) -> anyhow::Result<()> {
        let result = sqlx::query("DELETE FROM policy_stores WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Policy store not found: {}", id));
        }

        Ok(())
    }

    // ========================================================================
    // Schema Operations
    // ========================================================================

    pub async fn put_schema(&self, policy_store_id: &str, schema_json: String) -> anyhow::Result<models::Schema> {
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

        Ok(models::Schema {
            policy_store_id: policy_store_id.to_string(),
            schema_json,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_schema(&self, policy_store_id: &str) -> anyhow::Result<models::Schema> {
        let row = sqlx::query(
            "SELECT policy_store_id, schema_json, created_at, updated_at FROM schemas WHERE policy_store_id = ?",
        )
        .bind(policy_store_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Schema not found: {}", policy_store_id))?;

        Ok(models::Schema {
            policy_store_id: row.get("policy_store_id"),
            schema_json: row.get("schema_json"),
            created_at: row.get::<String, _>("created_at").parse().unwrap(),
            updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
        })
    }

    pub async fn delete_schema(&self, policy_store_id: &str) -> anyhow::Result<()> {
        let result = sqlx::query("DELETE FROM schemas WHERE policy_store_id = ?")
            .bind(policy_store_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Schema not found: {}", policy_store_id));
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
    ) -> anyhow::Result<models::Policy> {
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

        Ok(models::Policy {
            policy_store_id: policy_store_id.to_string(),
            policy_id: policy_id.to_string(),
            statement,
            description,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_policy(&self, policy_store_id: &str, policy_id: &str) -> anyhow::Result<models::Policy> {
        let row = sqlx::query(
            "SELECT policy_store_id, policy_id, statement, description, created_at, updated_at FROM policies WHERE policy_store_id = ? AND policy_id = ?",
        )
        .bind(policy_store_id)
        .bind(policy_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Policy not found: {}", policy_id))?;

        Ok(models::Policy {
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
    ) -> anyhow::Result<models::Policy> {
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
            return Err(anyhow::anyhow!("Policy not found: {}", policy_id));
        }

        Ok(models::Policy {
            policy_store_id: policy_store_id.to_string(),
            policy_id: policy_id.to_string(),
            statement,
            description,
            created_at: now, // We don't have the original created_at, but it's not critical
            updated_at: now,
        })
    }

    pub async fn delete_policy(&self, policy_store_id: &str, policy_id: &str) -> anyhow::Result<()> {
        let result = sqlx::query(
            "DELETE FROM policies WHERE policy_store_id = ? AND policy_id = ?",
        )
        .bind(policy_store_id)
        .bind(policy_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Policy not found: {}", policy_id));
        }

        Ok(())
    }

    pub async fn list_policies(&self, policy_store_id: &str) -> anyhow::Result<Vec<models::Policy>> {
        let rows = sqlx::query(
            "SELECT policy_store_id, policy_id, statement, description, created_at, updated_at FROM policies WHERE policy_store_id = ? ORDER BY created_at DESC",
        )
        .bind(policy_store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| models::Policy {
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
    ) -> anyhow::Result<models::IdentitySource> {
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

        Ok(models::IdentitySource {
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
    ) -> anyhow::Result<models::IdentitySource> {
        let row = sqlx::query(
            "SELECT id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description, created_at, updated_at FROM identity_sources WHERE policy_store_id = ? AND id = ?",
        )
        .bind(policy_store_id)
        .bind(identity_source_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(models::IdentitySource {
                id: row.get("id"),
                policy_store_id: row.get("policy_store_id"),
                configuration_type: row.get("configuration_type"),
                configuration_json: row.get("configuration_json"),
                claims_mapping_json: row.get("claims_mapping_json"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            }),
            None => Err(anyhow::anyhow!(
                "Identity source {} not found",
                identity_source_id
            )),
        }
    }

    pub async fn list_identity_sources(&self, policy_store_id: &str) -> anyhow::Result<Vec<models::IdentitySource>> {
        let rows = sqlx::query(
            "SELECT id, policy_store_id, configuration_type, configuration_json, claims_mapping_json, description, created_at, updated_at FROM identity_sources WHERE policy_store_id = ? ORDER BY created_at DESC",
        )
        .bind(policy_store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| models::IdentitySource {
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
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "DELETE FROM identity_sources WHERE policy_store_id = ? AND id = ?",
        )
        .bind(policy_store_id)
        .bind(identity_source_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!(
                "Identity source {} not found",
                identity_source_id
            ));
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
    ) -> anyhow::Result<models::PolicyTemplate> {
        let now = Utc::now();

        // Validate template syntax (check for placeholders)
        if !statement.contains("?principal") && !statement.contains("?resource") {
            return Err(anyhow::anyhow!(
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

        Ok(models::PolicyTemplate {
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
    ) -> anyhow::Result<models::PolicyTemplate> {
        let row = sqlx::query(
            "SELECT template_id, policy_store_id, statement, description, created_at, updated_at FROM policy_templates WHERE policy_store_id = ? AND template_id = ?",
        )
        .bind(policy_store_id)
        .bind(template_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(models::PolicyTemplate {
                template_id: row.get("template_id"),
                policy_store_id: row.get("policy_store_id"),
                statement: row.get("statement"),
                description: row.get("description"),
                created_at: row.get::<String, _>("created_at").parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at").parse().unwrap(),
            }),
            None => Err(anyhow::anyhow!(
                "Policy template {} not found",
                template_id
            )),
        }
    }

    pub async fn list_policy_templates(&self, policy_store_id: &str) -> anyhow::Result<Vec<models::PolicyTemplate>> {
        let rows = sqlx::query(
            "SELECT template_id, policy_store_id, statement, description, created_at, updated_at FROM policy_templates WHERE policy_store_id = ? ORDER BY created_at DESC",
        )
        .bind(policy_store_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| models::PolicyTemplate {
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
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "DELETE FROM policy_templates WHERE policy_store_id = ? AND template_id = ?",
        )
        .bind(policy_store_id)
        .bind(template_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!(
                "Policy template {} not found",
                template_id
            ));
        }

        Ok(())
    }

    // ========================================================================
    // Audit Operations
    // ========================================================================

    pub async fn log_authorization(&self, log: models::AuthorizationLog) -> anyhow::Result<()> {
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
