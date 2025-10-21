# Plan de Implementación: Alineación con Cedar Authorization for Express.js

## 🎯 Objetivo

Cerrar los gaps identificados entre nuestro SDK de Rust y las especificaciones de Cedar Authorization for Express.js, manteniendo las ventajas arquitecturales de Hodei.

---

## 📋 Gaps Identificados

### 🔴 Críticos
1. **Extracción de Entidades Completas** - Entidades con atributos y jerarquía
2. **Endpoints Excluidos** - Rutas públicas sin autorización

### 🟡 Importantes
3. **Configuración de Contexto** - API clara para contexto personalizado
4. **Herramientas CLI** - Generación de schema y políticas

---

## 🚀 Sprint 1: Extracción de Entidades Completas

### Objetivo
Permitir que el middleware extraiga y pase entidades completas con atributos y jerarquía de padres.

### Tareas

#### 1.1 Extender `AuthorizationRequestParts`
**Archivo:** `sdk/src/middleware/extractor.rs`

```rust
use std::collections::HashMap;

/// Entity with attributes and parents
#[derive(Debug, Clone)]
pub struct PrincipalEntity {
    /// Entity identifier (type and id)
    pub uid: EntityIdentifier,
    /// Entity attributes
    pub attrs: HashMap<String, serde_json::Value>,
    /// Parent entities (for hierarchy)
    pub parents: Vec<EntityIdentifier>,
}

/// Entity identifier
#[derive(Debug, Clone)]
pub struct EntityIdentifier {
    pub entity_type: String,
    pub id: String,
}

impl EntityIdentifier {
    pub fn new(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }
    
    pub fn to_cedar_string(&self) -> String {
        format!("{}::\"{}\"", self.entity_type, self.id)
    }
}

/// Parts of an authorization request extracted from an HTTP request
#[derive(Debug, Clone)]
pub struct AuthorizationRequestParts {
    /// The principal (user/entity) making the request
    pub principal: String,
    
    /// The action being performed
    pub action: String,
    
    /// The resource being accessed
    pub resource: String,
    
    /// Optional context for the authorization decision
    pub context: Option<serde_json::Value>,
    
    /// NEW: Principal entity with attributes (optional)
    pub principal_entity: Option<PrincipalEntity>,
    
    /// NEW: Additional entities for evaluation (optional)
    pub entities: Vec<Entity>,
}

/// Entity for Cedar evaluation
#[derive(Debug, Clone)]
pub struct Entity {
    pub uid: EntityIdentifier,
    pub attrs: HashMap<String, serde_json::Value>,
    pub parents: Vec<EntityIdentifier>,
}
```

#### 1.2 Actualizar Trait `AuthorizationRequestExtractor`

```rust
#[async_trait]
pub trait AuthorizationRequestExtractor<B>: Send + Sync
where
    B: Send,
{
    type Error: std::error::Error + Send + Sync + 'static;

    /// Extract authorization request parts from an HTTP request
    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error>;
    
    /// NEW: Extract principal entity with attributes (optional)
    /// 
    /// Override this method to provide principal entity with attributes
    /// for policies that need entity attributes in conditions.
    async fn extract_principal_entity(&self, req: &Request<B>) 
        -> Result<Option<PrincipalEntity>, Self::Error> 
    {
        Ok(None) // Default: no entity attributes
    }
    
    /// NEW: Extract additional entities for evaluation (optional)
    /// 
    /// Override this method to provide additional entities that may be
    /// referenced in policy conditions.
    async fn extract_entities(&self, req: &Request<B>) 
        -> Result<Vec<Entity>, Self::Error> 
    {
        Ok(Vec::new()) // Default: no additional entities
    }
}
```

#### 1.3 Actualizar `DefaultExtractor`

```rust
#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for DefaultExtractor
where
    B: Send + Sync,
{
    type Error = crate::middleware::MiddlewareError;

    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
        let token = Self::extract_token(req)?;
        
        let action = match req.method().as_str() {
            "GET" | "HEAD" => "Action::\"read\"",
            "POST" => "Action::\"create\"",
            "PUT" | "PATCH" => "Action::\"update\"",
            "DELETE" => "Action::\"delete\"",
            method => return Err(crate::middleware::MiddlewareError::ExtractionFailed(
                format!("Unsupported HTTP method: {}", method)
            )),
        };

        let path = req.uri().path();
        let resource = if path.is_empty() || path == "/" {
            "Resource::\"root\"".to_string()
        } else {
            format!("Resource::\"{}\"", path.trim_start_matches('/'))
        };

        let principal = String::new(); // Not used with IsAuthorizedWithToken

        Ok(AuthorizationRequestParts {
            principal,
            action: action.to_string(),
            resource,
            context: None,
            principal_entity: None, // Default extractor doesn't extract entities
            entities: Vec::new(),
        })
    }
}
```

#### 1.4 Crear `CustomExtractor` de Ejemplo

**Archivo:** `sdk/examples/custom_extractor.rs`

```rust
use hodei_permissions_sdk::middleware::{
    AuthorizationRequestExtractor, AuthorizationRequestParts,
    PrincipalEntity, EntityIdentifier, Entity,
};
use async_trait::async_trait;
use http::Request;
use std::collections::HashMap;

/// Custom extractor that extracts user entity with attributes
pub struct CustomUserExtractor {
    policy_store_id: String,
}

#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for CustomUserExtractor
where
    B: Send + Sync,
{
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
        // Extract JWT token
        let token = extract_jwt(req)?;
        let claims = decode_jwt(&token)?;
        
        // Extract action from method
        let action = format!("Action::\"{}\"", map_method_to_action(req.method()));
        
        // Extract resource from path
        let resource = format!("Resource::\"{}\"", req.uri().path().trim_start_matches('/'));
        
        // Principal identifier
        let principal = format!("User::\"{}\"", claims.sub);
        
        Ok(AuthorizationRequestParts {
            principal,
            action,
            resource,
            context: None,
            principal_entity: None, // Will be set by extract_principal_entity
            entities: Vec::new(),
        })
    }
    
    async fn extract_principal_entity(&self, req: &Request<B>) 
        -> Result<Option<PrincipalEntity>, Self::Error> 
    {
        let token = extract_jwt(req)?;
        let claims = decode_jwt(&token)?;
        
        // Build user entity with attributes
        let mut attrs = HashMap::new();
        attrs.insert("email".to_string(), serde_json::json!(claims.email));
        attrs.insert("department".to_string(), serde_json::json!(claims.department));
        attrs.insert("role".to_string(), serde_json::json!(claims.role));
        
        // Build parent entities (user groups)
        let parents = claims.groups.iter()
            .map(|group| EntityIdentifier::new("UserGroup", group))
            .collect();
        
        Ok(Some(PrincipalEntity {
            uid: EntityIdentifier::new("User", claims.sub),
            attrs,
            parents,
        }))
    }
    
    async fn extract_entities(&self, req: &Request<B>) 
        -> Result<Vec<Entity>, Self::Error> 
    {
        // Optionally extract resource entity with attributes
        // For example, if accessing /documents/123, fetch document attributes
        Ok(Vec::new())
    }
}
```

#### 1.5 Actualizar `VerifiedPermissionsService`

**Archivo:** `sdk/src/middleware/service.rs`

```rust
impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for VerifiedPermissionsService<S>
where
    // ... bounds
{
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let client = self.client.clone();
        let policy_store_id = self.policy_store_id.clone();
        let identity_source_id = self.identity_source_id.clone();
        let extractor = self.extractor.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract JWT token
            let token = match DefaultExtractor::extract_token(&req) {
                Ok(t) => t,
                Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            };

            // Extract authorization request parts
            let mut parts = match extractor.extract(&req).await {
                Ok(p) => p,
                Err(e) => return Err(Box::new(MiddlewareError::ExtractionFailed(e.to_string()))),
            };
            
            // NEW: Extract principal entity if available
            if let Ok(Some(principal_entity)) = extractor.extract_principal_entity(&req).await {
                parts.principal_entity = Some(principal_entity);
            }
            
            // NEW: Extract additional entities if available
            if let Ok(entities) = extractor.extract_entities(&req).await {
                parts.entities = entities;
            }

            // Call IsAuthorizedWithToken
            let auth_result = if parts.principal_entity.is_some() || !parts.entities.is_empty() {
                // Use IsAuthorized with entities
                client.is_authorized_with_entities(
                    &policy_store_id,
                    &parts.principal,
                    &parts.action,
                    &parts.resource,
                    parts.entities,
                    parts.context,
                ).await
            } else {
                // Use IsAuthorizedWithToken (simpler, faster)
                client.is_authorized_with_token(
                    &policy_store_id,
                    &identity_source_id,
                    &token,
                    &parts.action,
                    &parts.resource,
                ).await
            };

            match auth_result {
                Ok(response) => {
                    if response.decision == Decision::Allow as i32 {
                        match inner.call(req).await {
                            Ok(response) => Ok(response),
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        let error = MiddlewareError::AccessDenied(format!(
                            "Access denied for action '{}' on resource '{}'",
                            parts.action, parts.resource
                        ));
                        Err(Box::new(error))
                    }
                }
                Err(e) => {
                    let error = MiddlewareError::AuthorizationFailed(e.to_string());
                    Err(Box::new(error))
                }
            }
        })
    }
}
```

### Tests

```rust
#[tokio::test]
async fn test_custom_extractor_with_entities() {
    let extractor = CustomUserExtractor {
        policy_store_id: "store-123".to_string(),
    };
    
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/documents/123")
        .header("Authorization", "Bearer eyJ...")
        .body(())
        .unwrap();
    
    let parts = extractor.extract(&req).await.unwrap();
    assert_eq!(parts.principal, "User::\"alice\"");
    
    let entity = extractor.extract_principal_entity(&req).await.unwrap();
    assert!(entity.is_some());
    
    let entity = entity.unwrap();
    assert_eq!(entity.uid.entity_type, "User");
    assert_eq!(entity.uid.id, "alice");
    assert!(entity.attrs.contains_key("email"));
    assert!(entity.attrs.contains_key("department"));
}
```

---

## 🚀 Sprint 2: Endpoints Excluidos

### Objetivo
Permitir excluir rutas específicas de la autorización (ej: `/login`, `/health`, `/metrics`).

### Tareas

#### 2.1 Definir `SkippedEndpoint`

**Archivo:** `sdk/src/middleware/layer.rs`

```rust
/// Endpoint to skip authorization
#[derive(Debug, Clone)]
pub struct SkippedEndpoint {
    /// HTTP verb (lowercase)
    pub http_verb: String,
    /// Path pattern (exact match or wildcard)
    pub path: String,
}

impl SkippedEndpoint {
    pub fn new(http_verb: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            http_verb: http_verb.into().to_lowercase(),
            path: path.into(),
        }
    }
    
    /// Check if a request matches this skipped endpoint
    pub fn matches(&self, method: &http::Method, path: &str) -> bool {
        let method_str = method.as_str().to_lowercase();
        
        // Check method
        if self.http_verb != "*" && self.http_verb != method_str {
            return false;
        }
        
        // Check path (exact match for now, could add wildcard support)
        self.path == path
    }
}
```

#### 2.2 Actualizar `VerifiedPermissionsLayer`

```rust
#[derive(Clone)]
pub struct VerifiedPermissionsLayer {
    client: Arc<AuthorizationClient>,
    policy_store_id: String,
    identity_source_id: String,
    skipped_endpoints: Vec<SkippedEndpoint>,
}

impl VerifiedPermissionsLayer {
    pub fn new(
        client: AuthorizationClient,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
    ) -> Self {
        Self {
            client: Arc::new(client),
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            skipped_endpoints: Vec::new(),
        }
    }
    
    /// Skip authorization for a specific endpoint
    pub fn skip_endpoint(mut self, http_verb: &str, path: &str) -> Self {
        self.skipped_endpoints.push(SkippedEndpoint::new(http_verb, path));
        self
    }
    
    /// Skip authorization for multiple endpoints
    pub fn skip_endpoints(mut self, endpoints: Vec<SkippedEndpoint>) -> Self {
        self.skipped_endpoints.extend(endpoints);
        self
    }
    
    /// Check if a request should skip authorization
    pub fn should_skip(&self, method: &http::Method, path: &str) -> bool {
        self.skipped_endpoints.iter()
            .any(|endpoint| endpoint.matches(method, path))
    }
}
```

#### 2.3 Actualizar `VerifiedPermissionsService`

```rust
impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for VerifiedPermissionsService<S>
where
    // ... bounds
{
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let client = self.client.clone();
        let policy_store_id = self.policy_store_id.clone();
        let identity_source_id = self.identity_source_id.clone();
        let extractor = self.extractor.clone();
        let skipped_endpoints = self.skipped_endpoints.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // NEW: Check if endpoint should be skipped
            if skipped_endpoints.iter().any(|e| e.matches(req.method(), req.uri().path())) {
                // Skip authorization, forward directly to inner service
                return match inner.call(req).await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(e.into()),
                };
            }
            
            // ... rest of authorization logic
        })
    }
}
```

### Ejemplo de Uso

```rust
use hodei_permissions_sdk::middleware::{VerifiedPermissionsLayer, SkippedEndpoint};

let layer = VerifiedPermissionsLayer::new(client, "store-123", "identity-456")
    .skip_endpoint("get", "/login")
    .skip_endpoint("get", "/health")
    .skip_endpoint("get", "/metrics")
    .skip_endpoint("*", "/public/*"); // Wildcard support

let app = Router::new()
    .route("/login", get(login_handler))
    .route("/health", get(health_check))
    .route("/api/documents", get(list_documents))
    .layer(layer);
```

### Tests

```rust
#[test]
fn test_skipped_endpoint_matches() {
    let endpoint = SkippedEndpoint::new("get", "/login");
    
    assert!(endpoint.matches(&Method::GET, "/login"));
    assert!(!endpoint.matches(&Method::POST, "/login"));
    assert!(!endpoint.matches(&Method::GET, "/api/login"));
}

#[test]
fn test_wildcard_method() {
    let endpoint = SkippedEndpoint::new("*", "/health");
    
    assert!(endpoint.matches(&Method::GET, "/health"));
    assert!(endpoint.matches(&Method::POST, "/health"));
    assert!(endpoint.matches(&Method::DELETE, "/health"));
}
```

---

## 🚀 Sprint 3: Configuración de Contexto

### Objetivo
Proporcionar API clara para extraer contexto personalizado de las requests.

### Tareas

#### 3.1 Definir `ContextExtractor` Trait

**Archivo:** `sdk/src/middleware/context.rs`

```rust
use async_trait::async_trait;
use http::Request;
use serde_json::Value;

/// Trait for extracting context from HTTP requests
#[async_trait]
pub trait ContextExtractor<B>: Send + Sync
where
    B: Send,
{
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Extract context from request
    async fn extract_context(&self, req: &Request<B>) 
        -> Result<Option<Value>, Self::Error>;
}

/// Default context extractor (no context)
#[derive(Debug, Clone, Default)]
pub struct NoContextExtractor;

#[async_trait]
impl<B> ContextExtractor<B> for NoContextExtractor
where
    B: Send,
{
    type Error = std::convert::Infallible;
    
    async fn extract_context(&self, _req: &Request<B>) 
        -> Result<Option<Value>, Self::Error> 
    {
        Ok(None)
    }
}

/// Example: Extract IP address as context
#[derive(Debug, Clone)]
pub struct IpAddressContextExtractor;

#[async_trait]
impl<B> ContextExtractor<B> for IpAddressContextExtractor
where
    B: Send,
{
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn extract_context(&self, req: &Request<B>) 
        -> Result<Option<Value>, Self::Error> 
    {
        // Extract IP from headers or connection info
        let ip = req.headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");
        
        Ok(Some(serde_json::json!({
            "ip_address": ip,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })))
    }
}
```

#### 3.2 Integrar en `VerifiedPermissionsLayer`

```rust
pub struct VerifiedPermissionsLayer<C = NoContextExtractor> {
    client: Arc<AuthorizationClient>,
    policy_store_id: String,
    identity_source_id: String,
    skipped_endpoints: Vec<SkippedEndpoint>,
    context_extractor: Arc<C>,
}

impl VerifiedPermissionsLayer<NoContextExtractor> {
    pub fn new(
        client: AuthorizationClient,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
    ) -> Self {
        Self {
            client: Arc::new(client),
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            skipped_endpoints: Vec::new(),
            context_extractor: Arc::new(NoContextExtractor),
        }
    }
    
    /// Set custom context extractor
    pub fn with_context_extractor<C>(
        self, 
        extractor: C
    ) -> VerifiedPermissionsLayer<C> 
    where
        C: ContextExtractor<B>,
    {
        VerifiedPermissionsLayer {
            client: self.client,
            policy_store_id: self.policy_store_id,
            identity_source_id: self.identity_source_id,
            skipped_endpoints: self.skipped_endpoints,
            context_extractor: Arc::new(extractor),
        }
    }
}
```

### Ejemplo de Uso

```rust
use hodei_permissions_sdk::middleware::{
    VerifiedPermissionsLayer,
    IpAddressContextExtractor,
};

let layer = VerifiedPermissionsLayer::new(client, "store-123", "identity-456")
    .with_context_extractor(IpAddressContextExtractor)
    .skip_endpoint("get", "/login");

// Custom context extractor
struct CustomContextExtractor;

#[async_trait]
impl<B: Send> ContextExtractor<B> for CustomContextExtractor {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn extract_context(&self, req: &Request<B>) 
        -> Result<Option<Value>, Self::Error> 
    {
        Ok(Some(serde_json::json!({
            "user_agent": req.headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown"),
            "request_id": uuid::Uuid::new_v4().to_string(),
        })))
    }
}
```

---

## 🚀 Sprint 4: Herramientas CLI

### Objetivo
Proporcionar herramientas CLI para generar schemas y políticas desde OpenAPI specs.

### Tareas

#### 4.1 Crear CLI Tool

**Archivo:** `sdk/bin/hodei-cli.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hodei-cli")]
#[command(about = "Hodei Verified Permissions CLI Tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Cedar schema from OpenAPI specification
    GenerateSchema {
        /// Path to OpenAPI specification file
        #[arg(long)]
        api_spec: String,
        
        /// Cedar namespace
        #[arg(long)]
        namespace: String,
        
        /// Mapping type (SimpleRest, Custom)
        #[arg(long, default_value = "SimpleRest")]
        mapping_type: String,
        
        /// Output file
        #[arg(long, default_value = "v4.cedarschema.json")]
        output: String,
    },
    
    /// Generate sample Cedar policies
    GeneratePolicies {
        /// Path to Cedar schema file
        #[arg(long)]
        schema: String,
        
        /// Output directory
        #[arg(long, default_value = "policies")]
        output_dir: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::GenerateSchema { api_spec, namespace, mapping_type, output } => {
            generate_schema(&api_spec, &namespace, &mapping_type, &output)?;
        }
        Commands::GeneratePolicies { schema, output_dir } => {
            generate_policies(&schema, &output_dir)?;
        }
    }
    
    Ok(())
}
```

#### 4.2 Implementar Generación de Schema

```rust
fn generate_schema(
    api_spec_path: &str,
    namespace: &str,
    mapping_type: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse OpenAPI spec
    let spec = parse_openapi_spec(api_spec_path)?;
    
    // Generate Cedar schema
    let schema = match mapping_type {
        "SimpleRest" => generate_simple_rest_schema(&spec, namespace)?,
        _ => return Err("Unsupported mapping type".into()),
    };
    
    // Write to file
    std::fs::write(output_path, schema)?;
    
    println!("✅ Generated schema: {}", output_path);
    Ok(())
}
```

### Ejemplo de Uso

```bash
# Generate schema from OpenAPI
hodei-cli generate-schema \
    --api-spec openapi.json \
    --namespace MyApp \
    --mapping-type SimpleRest \
    --output v4.cedarschema.json

# Generate sample policies
hodei-cli generate-policies \
    --schema v4.cedarschema.json \
    --output-dir policies
```

---

## 📊 Resumen de Cambios

### Archivos Nuevos
- `sdk/src/middleware/context.rs` - Context extractors
- `sdk/bin/hodei-cli.rs` - CLI tool
- `sdk/examples/custom_extractor.rs` - Example custom extractor

### Archivos Modificados
- `sdk/src/middleware/extractor.rs` - Entity extraction
- `sdk/src/middleware/layer.rs` - Skipped endpoints
- `sdk/src/middleware/service.rs` - Integration
- `sdk/src/middleware/mod.rs` - Re-exports

### Tests Nuevos
- `test_custom_extractor_with_entities`
- `test_skipped_endpoint_matches`
- `test_wildcard_method`
- `test_context_extraction`

---

## 🎯 Métricas de Éxito

### Sprint 1
- [ ] Trait `AuthorizationRequestExtractor` extendido
- [ ] `PrincipalEntity` implementado
- [ ] Ejemplo `custom_extractor.rs` funcionando
- [ ] Tests pasando

### Sprint 2
- [ ] `SkippedEndpoint` implementado
- [ ] API fluida `.skip_endpoint()` funcionando
- [ ] Tests de matching pasando
- [ ] Documentación actualizada

### Sprint 3
- [ ] `ContextExtractor` trait implementado
- [ ] Ejemplo `IpAddressContextExtractor` funcionando
- [ ] Integración con service completa
- [ ] Tests pasando

### Sprint 4
- [ ] CLI tool funcionando
- [ ] Generación de schema desde OpenAPI
- [ ] Generación de políticas de ejemplo
- [ ] Documentación de uso

---

## 📅 Timeline

| Sprint | Duración | Entregables |
|--------|----------|-------------|
| Sprint 1 | 2 semanas | Entity extraction completa |
| Sprint 2 | 1 semana | Skipped endpoints |
| Sprint 3 | 1 semana | Context extractors |
| Sprint 4 | 2 semanas | CLI tools |

**Total:** 6 semanas

---

## 🔗 Referencias

- [Análisis de Alineación](./ALIGNMENT_EXPRESSJS_CEDAR.md)
- [Middleware Guide](../sdk/docs/MIDDLEWARE_GUIDE.md)
- [Cedar Policy Language](https://www.cedarpolicy.com/)

---

**Última actualización:** 2025-10-21
