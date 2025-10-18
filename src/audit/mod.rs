//! Audit logging module for authorization decisions
//!
//! This module provides structured logging of all authorization decisions
//! for forensic analysis, monitoring, and compliance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit event for an authorization decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub event_id: String,
    
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    
    /// Policy store ID
    pub policy_store_id: String,
    
    /// Principal making the request
    pub principal: String,
    
    /// Action being performed
    pub action: String,
    
    /// Resource being accessed
    pub resource: String,
    
    /// Authorization decision (ALLOW/DENY)
    pub decision: String,
    
    /// Policies that determined the decision
    pub determining_policies: Vec<String>,
    
    /// Any errors that occurred
    pub errors: Vec<String>,
    
    /// Optional context
    pub context: Option<String>,
    
    /// Number of entities provided
    pub entity_count: usize,
    
    /// Request source (e.g., "is_authorized", "is_authorized_with_token")
    pub request_type: String,
    
    /// Optional identity source ID (for JWT requests)
    pub identity_source_id: Option<String>,
}

/// Audit logger for recording authorization decisions
pub struct AuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an authorization decision
    pub async fn log_decision(&self, event: AuditEvent) {
        // Log to structured JSON
        let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        tracing::info!(target: "audit", "{}", json);
        
        // Store in memory for testing/retrieval
        let mut events = self.events.write().await;
        events.push(event);
    }

    /// Get all audit events (for testing/monitoring)
    pub async fn get_events(&self) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// Get events filtered by policy store
    pub async fn get_events_by_store(&self, policy_store_id: &str) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.policy_store_id == policy_store_id)
            .cloned()
            .collect()
    }

    /// Get events filtered by principal
    pub async fn get_events_by_principal(&self, principal: &str) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.principal == principal)
            .cloned()
            .collect()
    }

    /// Get events filtered by decision
    pub async fn get_events_by_decision(&self, decision: &str) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.decision == decision)
            .cloned()
            .collect()
    }

    /// Clear all events (for testing)
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        events.clear();
    }

    /// Get event count
    pub async fn count(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event() -> AuditEvent {
        AuditEvent {
            event_id: "evt-123".to_string(),
            timestamp: Utc::now(),
            policy_store_id: "store-123".to_string(),
            principal: "User::alice".to_string(),
            action: "Action::view".to_string(),
            resource: "Document::doc123".to_string(),
            decision: "ALLOW".to_string(),
            determining_policies: vec!["policy-1".to_string()],
            errors: vec![],
            context: None,
            entity_count: 0,
            request_type: "is_authorized".to_string(),
            identity_source_id: None,
        }
    }

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        assert_eq!(logger.count().await, 0);
    }

    #[tokio::test]
    async fn test_log_decision() {
        let logger = AuditLogger::new();
        let event = create_test_event();
        
        logger.log_decision(event.clone()).await;
        
        assert_eq!(logger.count().await, 1);
        let events = logger.get_events().await;
        assert_eq!(events[0].event_id, "evt-123");
    }

    #[tokio::test]
    async fn test_filter_by_store() {
        let logger = AuditLogger::new();
        
        let mut event1 = create_test_event();
        event1.policy_store_id = "store-1".to_string();
        logger.log_decision(event1).await;
        
        let mut event2 = create_test_event();
        event2.policy_store_id = "store-2".to_string();
        logger.log_decision(event2).await;
        
        let events = logger.get_events_by_store("store-1").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].policy_store_id, "store-1");
    }

    #[tokio::test]
    async fn test_filter_by_principal() {
        let logger = AuditLogger::new();
        
        let mut event1 = create_test_event();
        event1.principal = "User::alice".to_string();
        logger.log_decision(event1).await;
        
        let mut event2 = create_test_event();
        event2.principal = "User::bob".to_string();
        logger.log_decision(event2).await;
        
        let events = logger.get_events_by_principal("User::alice").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].principal, "User::alice");
    }

    #[tokio::test]
    async fn test_filter_by_decision() {
        let logger = AuditLogger::new();
        
        let mut event1 = create_test_event();
        event1.decision = "ALLOW".to_string();
        logger.log_decision(event1).await;
        
        let mut event2 = create_test_event();
        event2.decision = "DENY".to_string();
        logger.log_decision(event2).await;
        
        let allow_events = logger.get_events_by_decision("ALLOW").await;
        assert_eq!(allow_events.len(), 1);
        
        let deny_events = logger.get_events_by_decision("DENY").await;
        assert_eq!(deny_events.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_events() {
        let logger = AuditLogger::new();
        logger.log_decision(create_test_event()).await;
        assert_eq!(logger.count().await, 1);
        
        logger.clear().await;
        assert_eq!(logger.count().await, 0);
    }

    #[test]
    fn test_audit_event_serialization() {
        let event = create_test_event();
        let json = serde_json::to_string(&event).unwrap();
        
        assert!(json.contains("evt-123"));
        assert!(json.contains("User::alice"));
        assert!(json.contains("ALLOW"));
    }
}
