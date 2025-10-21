//! Principal extraction system

use async_trait::async_trait;
use http::Request;
use std::sync::Arc;
use crate::entities::{CedarEntity, EntityIdentifier};

/// Principal configuration types (equivalent to Express.js)
#[derive(Debug, Clone)]
pub enum PrincipalConfiguration {
    /// Use identity token from JWT
    IdentityToken,
    /// Use access token from JWT
    AccessToken,
    /// Custom principal extraction
    Custom {
        extractor: Arc<dyn PrincipalExtractor>,
    },
}

/// Trait for extracting principal entities (equivalent to getPrincipalEntity)
#[async_trait]
pub trait PrincipalExtractor: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Extract principal entity from HTTP request
    async fn extract_principal<B>(&self, req: &Request<B>) 
        -> Result<CedarEntity, Self::Error>
    where
        B: Send;
}

/// JWT-based principal extractor
#[derive(Debug, Clone)]
pub struct JwtPrincipalExtractor {
    jwt_secret: String,
    entity_type: String,
    claims_field: Option<String>,
}

impl JwtPrincipalExtractor {
    pub fn new(jwt_secret: impl Into<String>) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
            entity_type: "User".to_string(),
            claims_field: None,
        }
    }
    
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = entity_type.into();
        self
    }
    
    pub fn with_claims_field(mut self, field: impl Into<String>) -> Self {
        self.claims_field = Some(field.into());
        self
    }
}

#[async_trait]
impl PrincipalExtractor for JwtPrincipalExtractor {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn extract_principal<B>(&self, req: &Request<B>) -> Result<CedarEntity, Self::Error>
    where
        B: Send,
    {
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
        
        // Extract JWT token from Authorization header
        let auth_header = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or("Missing Authorization header")?;
            
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or("Invalid Authorization header format")?;
            
        // Decode JWT
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<serde_json::Value>(token, &decoding_key, &validation)?;
        
        let claims = token_data.claims;
        
        // Extract user ID
        let user_id = if let Some(field) = &self.claims_field {
            claims.get(field)
                .and_then(|v| v.as_str())
                .ok_or(format!("Missing {} claim", field))? 
        } else {
            claims.get("sub")
                .and_then(|v| v.as_str())
                .ok_or("Missing sub claim")?
        };
        
        // Build entity attributes
        let mut attrs = HashMap::new();
        for (key, value) in claims.as_object().unwrap_or(&serde_json::Map::new()) {
            attrs.insert(key.clone(), value.clone());
        }
        
        // Extract groups for parent relationships
        let parents = if let Some(groups) = claims.get("groups").and_then(|v| v.as_array()) {
            groups.iter()
                .filter_map(|g| g.as_str())
                .map(|group| EntityIdentifier::new("UserGroup", group))
                .collect()
        } else if let Some(roles) = claims.get("roles").and_then(|v| v.as_array()) {
            roles.iter()
                .filter_map(|r| r.as_str())
                .map(|role| EntityIdentifier::new("Role", role))
                .collect()
        } else {
            Vec::new()
        };
        
        Ok(CedarEntity {
            uid: EntityIdentifier::new(&self.entity_type, user_id),
            attrs,
            parents,
        })
    }
}

/// Simple user ID extractor
#[derive(Debug, Clone)]
pub struct UserIdExtractor {
    header_name: String,
    entity_type: String,
}

impl UserIdExtractor {
    pub fn new(header_name: impl Into<String>) -> Self {
        Self {
            header_name: header_name.into(),
            entity_type: "User".to_string(),
        }
    }
    
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = entity_type.into();
        self
    }
}

#[async_trait]
impl PrincipalExtractor for UserIdExtractor {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn extract_principal<B>(&self, req: &Request<B>) -> Result<CedarEntity, Self::Error>
    where
        B: Send,
    {
        let user_id = req.headers()
            .get(&self.header_name)
            .and_then(|h| h.to_str().ok())
            .ok_or(format!("Missing {} header", self.header_name))?;
            
        Ok(CedarEntity::builder(&self.entity_type, user_id).build())
    }
}

/// Session-based extractor
#[derive(Debug, Clone)]
pub struct SessionExtractor {
    session_key: String,
    entity_type: String,
}

impl SessionExtractor {
    pub fn new(session_key: impl Into<String>) -> Self {
        Self {
            session_key: session_key.into(),
            entity_type: "User".to_string(),
        }
    }
    
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = entity_type.into();
        self
    }
}

#[async_trait]
impl PrincipalExtractor for SessionExtractor {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn extract_principal<B>(&self, req: &Request<B>) -> Result<CedarEntity, Self::Error>
    where
        B: Send,
    {
        // This would typically extract from session storage
        // For now, we'll use a header as a placeholder
        let session_id = req.headers()
            .get(&self.session_key)
            .and_then(|h| h.to_str().ok())
            .ok_or(format!("Missing {} session header", self.session_key))?;
            
        Ok(CedarEntity::builder(&self.entity_type, session_id).build())
    }
}

/// Composite extractor that tries multiple extractors
#[derive(Debug, Clone)]
pub struct CompositeExtractor {
    extractors: Vec<Arc<dyn PrincipalExtractor<Error = Box<dyn std::error::Error + Send + Sync>>>>,
}

impl CompositeExtractor {
    pub fn new() -> Self {
        Self { extractors: Vec::new() }
    }
    
    pub fn add_extractor(mut self, extractor: Arc<dyn PrincipalExtractor<Error = Box<dyn std::error::Error + Send + Sync>>>) -> Self {
        self.extractors.push(extractor);
        self
    }
}

#[async_trait]
impl PrincipalExtractor for CompositeExtractor {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn extract_principal<B>(&self, req: &Request<B>) -> Result<CedarEntity, Self::Error>
    where
        B: Send,
    {
        for extractor in &self.extractors {
            match extractor.extract_principal(req).await {
                Ok(entity) => return Ok(entity),
                Err(_) => continue, // Try next extractor
            }
        }
        
        Err("No extractor could extract principal".into())
    }
}

/// Factory for creating common extractors
pub mod extractor_factory {
    use super::*;
    
    /// Create a JWT extractor with common settings
    pub fn jwt_extractor(secret: impl Into<String>) -> JwtPrincipalExtractor {
        JwtPrincipalExtractor::new(secret)
    }
    
    /// Create a user ID extractor from header
    pub fn user_id_extractor(header_name: impl Into<String>) -> UserIdExtractor {
        UserIdExtractor::new(header_name)
    }
    
    /// Create a session extractor
    pub fn session_extractor(session_key: impl Into<String>) -> SessionExtractor {
        SessionExtractor::new(session_key)
    }
    
    /// Create a composite extractor with JWT and fallback
    pub fn jwt_with_fallback(
        secret: impl Into<String>,
        fallback_header: impl Into<String>,
    ) -> CompositeExtractor {
        CompositeExtractor::new()
            .add_extractor(Arc::new(JwtPrincipalExtractor::new(secret)))
            .add_extractor(Arc::new(UserIdExtractor::new(fallback_header)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Request, Method};
    
    #[tokio::test]
    async fn test_jwt_extractor() {
        // Note: This test would require a valid JWT token
        // For now, we'll test the structure
        let extractor = JwtPrincipalExtractor::new("secret_key")
            .with_entity_type("User");
            
        assert_eq!(extractor.entity_type, "User");
    }
    
    #[tokio::test]
    async fn test_user_id_extractor() {
        let extractor = UserIdExtractor::new("X-User-ID")
            .with_entity_type("User");
            
        let req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("X-User-ID", "alice")
            .body(())
            .unwrap();
            
        let result = extractor.extract_principal(&req).await;
        assert!(result.is_ok());
        
        let entity = result.unwrap();
        assert_eq!(entity.uid.entity_type, "User");
        assert_eq!(entity.uid.id, "alice");
    }
    
    #[tokio::test]
    async fn test_session_extractor() {
        let extractor = SessionExtractor::new("session-id")
            .with_entity_type("User");
            
        let req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("session-id", "session_123")
            .body(())
            .unwrap();
            
        let result = extractor.extract_principal(&req).await;
        assert!(result.is_ok());
        
        let entity = result.unwrap();
        assert_eq!(entity.uid.entity_type, "User");
        assert_eq!(entity.uid.id, "session_123");
    }
    
    #[test]
    fn test_principal_configuration() {
        let config = PrincipalConfiguration::IdentityToken;
        assert!(matches!(config, PrincipalConfiguration::IdentityToken));
        
        let custom_config = PrincipalConfiguration::Custom {
            extractor: Arc::new(UserIdExtractor::new("test")),
        };
        assert!(matches!(custom_config, PrincipalConfiguration::Custom { .. }));
    }
}
