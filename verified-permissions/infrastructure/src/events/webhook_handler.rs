//! Webhook Handler - Publishes audit events to external databases
//!
//! This module implements a webhook delivery system that subscribes to
//! domain events and publishes them to external databases and services.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hodei_domain::events::{DomainEvent, EventHandler, SubscriptionId};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Webhook configuration
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub id: String,
    pub url: String,
    pub secret: Option<String>,
    pub event_types: Vec<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Webhook delivery status
#[derive(Debug, Clone)]
pub struct WebhookDelivery {
    pub webhook_id: String,
    pub event_id: String,
    pub status: DeliveryStatus,
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum DeliveryStatus {
    PENDING,
    SUCCESS,
    FAILED,
}

/// Webhook Handler
/// Subscribes to domain events and publishes them to external systems
pub struct WebhookEventHandler {
    client: reqwest::Client,
    config: WebhookConfig,
}

impl WebhookEventHandler {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    /// Determine if this webhook should handle the given event
    fn should_handle_event(&self, event: &dyn DomainEvent) -> bool {
        if !self.config.active {
            return false;
        }

        if self.config.event_types.is_empty() {
            return true; // Handle all events if no filter specified
        }

        self.config
            .event_types
            .contains(&event.event_type().to_string())
    }
}

impl WebhookEventHandler {
    /// Determine if this webhook should handle the given event
    fn should_handle_event(&self, event: &dyn DomainEvent) -> bool {
        if !self.config.active {
            return false;
        }

        if self.config.event_types.is_empty() {
            return true; // Handle all events if no filter specified
        }

        self.config
            .event_types
            .contains(&event.event_type().to_string())
    }
}

impl EventHandler for WebhookEventHandler {
    fn handle(
        &self,
        event: &dyn DomainEvent,
    ) -> Box<
        dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync,
    > {
        let event_clone = event as *const dyn DomainEvent;
        let event_ref = unsafe { &*event_clone } as &dyn DomainEvent;
        let should_handle = self.should_handle_event(event_ref);
        let config = self.config.clone();
        let client = self.client.clone();

        Box::new(async move {
            if !should_handle {
                return Ok(());
            }

            // Prepare webhook payload
            let payload = WebhookPayload {
                event_id: event_ref.event_id(),
                event_type: event_ref.event_type().to_string(),
                aggregate_id: event_ref.aggregate_id(),
                timestamp: event_ref.occurred_at().to_rfc3339(),
                version: event_ref.version(),
                data: event_ref,
            };

            let payload_json = serde_json::to_string(&payload)
                .map_err(|e| format!("Failed to serialize event: {}", e))?;

            // Create request
            let mut request = client
                .post(&config.url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "Hodei-VerifiedPermissions/1.0")
                .header("X-Event-Type", event_ref.event_type())
                .header("X-Event-Id", &event_ref.event_id())
                .header("X-Signature", "sha256") // Placeholder for HMAC signature
                .body(payload_json);

            // Add HMAC signature if secret is configured
            if let Some(secret) = &config.secret {
                use hmac::{Hmac, Mac};
                use sha2::Sha256;

                type HmacSha256 = Hmac<Sha256>;
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .map_err(|e| format!("Failed to create HMAC: {}", e))?;
                mac.update(payload_json.as_bytes());

                let signature = hex::encode(mac.finalize().into_bytes());
                request = request.header("X-Signature-SHA256", signature);
            }

            // Send webhook
            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!(
                            "Webhook delivered successfully for event: {}",
                            event_ref.event_id()
                        );
                    } else {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        error!(
                            "Webhook delivery failed for event: {} with status {}: {}",
                            event_ref.event_id(),
                            status,
                            body
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to send webhook for event: {}: {}",
                        event_ref.event_id(),
                        e
                    );
                }
            }

            Ok(())
        })
    }
}

/// Webhook payload structure
#[derive(Debug, Serialize, Deserialize)]
struct WebhookPayload<'a> {
    event_id: &'a str,
    event_type: String,
    aggregate_id: String,
    timestamp: String,
    version: u32,
    data: &'a dyn DomainEvent,
}

/// Webhook Registry
/// Manages webhook configurations and subscriptions
pub struct WebhookRegistry {
    webhooks: HashMap<String, WebhookConfig>,
    handlers: HashMap<String, Box<dyn EventHandler<dyn DomainEvent + Send + Sync>>>,
}

impl WebhookRegistry {
    pub fn new() -> Self {
        Self {
            webhooks: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// Register a new webhook
    pub fn register_webhook(&mut self, config: WebhookConfig) {
        info!(
            "Registering webhook: {} for events: {:?}",
            config.id, config.event_types
        );

        self.webhooks.insert(config.id.clone(), config.clone());

        // Create and register handler for all event types
        let handler = WebhookEventHandler::new(config);
        let handler: Box<dyn EventHandler<dyn DomainEvent + Send + Sync>> = Box::new(handler);
        self.handlers.insert(config.id, handler);
    }

    /// Unregister a webhook
    pub fn unregister_webhook(&mut self, webhook_id: &str) {
        info!("Unregistering webhook: {}", webhook_id);
        self.webhooks.remove(webhook_id);
        self.handlers.remove(webhook_id);
    }

    /// Get all active webhooks
    pub fn get_active_webhooks(&self) -> Vec<&WebhookConfig> {
        self.webhooks.values().filter(|w| w.active).collect()
    }
}

/// Webhook Database Adapter
/// Manages webhook persistence
pub struct WebhookDatabase {
    pool: sqlx::SqlitePool,
}

impl WebhookDatabase {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pool = sqlx::SqlitePool::connect(database_url)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Create webhooks table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webhooks (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                secret TEXT,
                event_types TEXT NOT NULL, -- JSON array
                active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Create webhook deliveries table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webhook_deliveries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                webhook_id TEXT NOT NULL,
                event_id TEXT NOT NULL,
                status TEXT NOT NULL,
                attempt_count INTEGER NOT NULL DEFAULT 1,
                last_error TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (webhook_id) REFERENCES webhooks (id)
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(Self { pool })
    }

    /// Create a new webhook
    pub async fn create_webhook(
        &self,
        config: &WebhookConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_types_json = serde_json::to_string(&config.event_types)?;

        sqlx::query(
            r#"
            INSERT INTO webhooks (id, url, secret, event_types, active, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&config.id)
        .bind(&config.url)
        .bind(&config.secret)
        .bind(&event_types_json)
        .bind(config.active)
        .bind(config.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    /// Get all webhooks
    pub async fn get_webhooks(
        &self,
    ) -> Result<Vec<WebhookConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query("SELECT * FROM webhooks")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let mut webhooks = Vec::new();
        for row in rows {
            let event_types_json: String = row.get("event_types");
            let event_types: Vec<String> =
                serde_json::from_str(&event_types_json).unwrap_or_else(|_| Vec::new());

            let webhook = WebhookConfig {
                id: row.get("id"),
                url: row.get("url"),
                secret: row.get("secret"),
                event_types,
                active: row.get("active"),
                created_at: row.get("created_at"),
            };
            webhooks.push(webhook);
        }

        Ok(webhooks)
    }
}
