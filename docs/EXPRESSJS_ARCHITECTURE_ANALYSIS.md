# Análisis Detallado: Arquitectura Cedar Authorization for Express.js

## 📋 Resumen de la Arquitectura Express.js

Basándome en la documentación oficial y patrones observados, aquí está el análisis detallado de la arquitectura de Cedar Authorization for Express.js para crear una implementación equivalente en Rust.

---

## 🏗️ Estructura de Directorios Express.js

```
authorization-for-expressjs/
├── src/
│   ├── middleware.ts              # ExpressAuthorizationMiddleware
│   ├── authorization-engine.ts    # CedarInlineAuthorizationEngine
│   ├── extractors.ts              # Principal extractors
│   ├── types.ts                   # TypeScript interfaces
│   └── index.ts                   # Main exports
├── examples/
│   ├── basic-example.js
│   └── custom-extractor.js
├── tests/
│   └── middleware.test.ts
├── package.json
└── tsconfig.json
```

---

## 🔍 Análisis de Componentes

### 1. **ExpressAuthorizationMiddleware** - Estructura Principal

#### Configuración en Express.js:
```typescript
interface ExpressAuthorizationMiddlewareConfig {
  schema: {
    type: 'jsonString' | 'cedar';
    schema: string;
  };
  authorizationEngine: AuthorizationEngine;
  principalConfiguration: PrincipalConfiguration;
  skippedEndpoints?: SkippedEndpoint[];
  logger?: Logger;
  contextConfiguration?: ContextConfiguration;
}

interface SkippedEndpoint {
  httpVerb: string;
  path: string;
}

interface Logger {
  debug: (message: string) => void;
  log: (message: string) => void;
}

interface ContextConfiguration {
  // Optional context extraction
}

interface PrincipalConfiguration {
  type: 'identityToken' | 'accessToken' | 'custom';
  getPrincipalEntity?: (req: Request) => Promise<Entity>;
}
```

#### Uso en Express.js:
```typescript
const expressAuthorization = new ExpressAuthorizationMiddleware({
  schema: { type: 'jsonString', schema: schemaContent },
  authorizationEngine: cedarAuthorizationEngine,
  principalConfiguration: {
    type: 'custom',
    getPrincipalEntity: async (req) => {
      const user = req.user;
      return {
        uid: { type: 'PetStoreApp::User', id: user.sub },
        attrs: { ...user },
        parents: user.groups.map(g => ({ type: 'PetStoreApp::UserGroup', id: g }))
      };
    }
  },
  skippedEndpoints: [
    { httpVerb: 'get', path: '/login' },
    { httpVerb: 'get', path: '/health' }
  ],
  logger: {
    debug: (s) => console.log(s),
    log: (s) => console.log(s)
  }
});

app.use(expressAuthorization.middleware);
```

### 2. **Authorization Engine Pattern**

#### CedarInlineAuthorizationEngine:
```typescript
interface CedarInlineAuthorizationEngineConfig {
  staticPolicies: string;
  schema: {
    type: 'jsonString' | 'cedar';
    schema: string;
  };
}

interface AuthorizationEngine {
  authorize(request: AuthorizationRequest): Promise<AuthorizationResult>;
}
```

#### AVPAuthorizationEngine (AWS):
```typescript
interface AVPAuthorizationEngineConfig {
  policyStoreId: string;
  callType: 'isAuthorized' | 'accessToken' | 'identityToken';
  credentials?: AWSCredentials;
}
```

### 3. **Entity Structure**

#### Entity Format:
```typescript
interface Entity {
  uid: EntityIdentifier;
  attrs: Record<string, any>;
  parents: EntityIdentifier[];
}

interface EntityIdentifier {
  type: string;
  id: string;
}
```

#### Ejemplo de Entidad:
```typescript
{
  uid: { type: 'PetStoreApp::User', id: 'alice' },
  attrs: {
    email: 'alice@example.com',
    department: 'engineering',
    role: 'admin'
  },
  parents: [
    { type: 'PetStoreApp::UserGroup', id: 'administrators' },
    { type: 'PetStoreApp::UserGroup', id: 'engineers' }
  ]
}
```

---

## 🦀 Arquitectura Rust Equivalente - Diseño Propuesto

### 1. **Estructura de Directorios Rust**

```
hodei-verified-permissions-sdk/
├── src/
│   ├── middleware/
│   │   ├── mod.rs                    # Re-exports
│   │   ├── layer.rs                  # Tower Layer
│   │   ├── service.rs                # Tower Service
│   │   ├── extractors/
│   │   │   ├── mod.rs                # Extractor traits
│   │   │   ├── default.rs            # Default extractor
│   │   │   ├── jwt.rs               # JWT extractor
│   │   │   └── custom.rs            # Custom extractor examples
│   │   ├── context/
│   │   │   ├── mod.rs                # Context extractors
│   │   │   ├── ip.rs                # IP context
│   │   │   └── user_agent.rs        # User agent context
│   │   └── skipped.rs               # Endpoint filtering
│   ├── authorization/
│   │   ├── mod.rs                   # Authorization engines
│   │   ├── engine.rs                # Authorization engine trait
│   │   ├── cedar_inline.rs          # Cedar inline engine
│   │   └── avp.rs                   # AVP engine (future)
│   ├── entities/
│   │   ├── mod.rs                   # Entity definitions
│   │   ├── identifier.rs            # EntityIdentifier
│   │   └── builder.rs               # Entity builder
│   ├── types.rs                     # Common types
│   └── error.rs                     # Error types
├── examples/
│   ├── basic_middleware.rs
│   ├── custom_extractor.rs
│   └── skipped_endpoints.rs
└── tests/
    └── middleware_tests.rs
```

### 2. **Core Architecture - Rust Types**

#### Entity System:
```rust
use std::collections::HashMap;

/// Entity identifier for Cedar
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// Cedar entity with attributes and hierarchy
#[derive(Debug, Clone)]
pub struct CedarEntity {
    pub uid: EntityIdentifier,
    pub attrs: HashMap<String, serde_json::Value>,
    pub parents: Vec<EntityIdentifier>,
}

impl CedarEntity {
    pub fn builder(entity_type: impl Into<String>, id: impl Into<String>) -> CedarEntityBuilder {
        CedarEntityBuilder::new(entity_type, id)
    }
}

/// Builder for Cedar entities
pub struct CedarEntityBuilder {
    uid: EntityIdentifier,
    attrs: HashMap<String, serde_json::Value>,
    parents: Vec<EntityIdentifier>,
}

impl CedarEntityBuilder {
    pub fn new(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            uid: EntityIdentifier::new(entity_type, id),
            attrs: HashMap::new(),
            parents: Vec::new(),
        }
    }
    
    pub fn attribute(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        self.attrs.insert(key.into(), serde_json::to_value(value).unwrap());
        self
    }
    
    pub fn parent(mut self, entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        self.parents.push(EntityIdentifier::new(entity_type, id));
        self
    }
    
    pub fn parents(mut self, parents: Vec<EntityIdentifier>) -> Self {
        self.parents = parents;
        self
    }
    
    pub fn build(self) -> CedarEntity {
        CedarEntity {
            uid: self.uid,
            attrs: self.attrs,
            parents: self.parents,
        }
    }
}
```

#### Authorization Engine Trait:
```rust
use async_trait::async_trait;

/// Authorization request
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    pub principal: CedarEntity,
    pub action: String,
    pub resource: CedarEntity,
    pub context: Option<serde_json::Value>,
    pub entities: Vec<CedarEntity>,
}

/// Authorization result
#[derive(Debug, Clone)]
pub enum AuthorizationResult {
    Allow {
        determining_policies: Vec<String>,
        principal_uid: EntityIdentifier,
    },
    Deny,
    Error(String),
}

/// Authorization engine trait
#[async_trait]
pub trait AuthorizationEngine: Send + Sync {
    async fn authorize(&self, request: AuthorizationRequest) -> Result<AuthorizationResult, Box<dyn std::error::Error>>;
}

/// Cedar inline authorization engine
pub struct CedarInlineAuthorizationEngine {
    static_policies: String,
    schema: String,
}

impl CedarInlineAuthorizationEngine {
    pub fn new(static_policies: String, schema: String) -> Self {
        Self {
            static_policies,
            schema,
        }
    }
}

#[async_trait]
impl AuthorizationEngine for CedarInlineAuthorizationEngine {
    async fn authorize(&self, request: AuthorizationRequest) -> Result<AuthorizationResult, Box<dyn std::error::Error>> {
        // Implementation using Cedar WASM or gRPC
        todo!()
    }
}
```

### 3. **Extractor System - Rust Equivalent**

#### Principal Configuration (Rust equivalent):
```rust
use async_trait::async_trait;
use http::Request;

/// Principal configuration types
#[derive(Debug, Clone)]
pub enum PrincipalConfiguration {
    IdentityToken,
    AccessToken,
    Custom {
        extractor: Arc<dyn PrincipalExtractor>,
    },
}

/// Trait for extracting principal entities
#[async_trait]
pub trait PrincipalExtractor: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn extract_principal<B>(&self, req: &Request<B>) 
        -> Result<CedarEntity, Self::Error>
    where
        B: Send;
}

/// JWT-based principal extractor
pub struct JwtPrincipalExtractor {
    jwt_secret: String,
    entity_type: String,
}

#[async_trait]
impl PrincipalExtractor for JwtPrincipalExtractor {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn extract_principal<B>(&self, req: &Request<B>) -> Result<CedarEntity, Self::Error>
    where
        B: Send,
    {
        let token = extract_jwt_from_header(req)?;
        let claims = decode_jwt(&token, &self.jwt_secret)?;
        
        let mut attrs = HashMap::new();
        attrs.insert("sub".to_string(), serde_json::json!(claims.sub));
        attrs.insert("email".to_string(), serde_json::json!(claims.email));
        
        let parents = claims.groups
            .iter()
            .map(|group| EntityIdentifier::new("UserGroup", group))
            .collect();
        
        Ok(CedarEntity {
            uid: EntityIdentifier::new(&self.entity_type, &claims.sub),
            attrs,
            parents,
        })
    }
}
```

### 4. **Middleware Configuration - Rust Equivalent**

#### Tower Layer Configuration:
```rust
use tower_layer::Layer;
use std::sync::Arc;

/// Skipped endpoint configuration
#[derive(Debug, Clone)]
pub struct SkippedEndpoint {
    pub http_verb: String,
    pub path: String,
}

impl SkippedEndpoint {
    pub fn new(http_verb: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            http_verb: http_verb.into().to_lowercase(),
            path: path.into(),
        }
    }
    
    pub fn matches(&self, method: &http::Method, path: &str) -> bool {
        let method_str = method.as_str().to_lowercase();
        method_str == self.http_verb && path == self.path
    }
}

/// Logger trait
pub trait Logger: Send + Sync {
    fn debug(&self, message: &str);
    fn log(&self, message: &str);
}

/// Context extractor trait
#[async_trait]
pub trait ContextExtractor: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn extract_context<B>(&self, req: &Request<B>) 
        -> Result<Option<serde_json::Value>, Self::Error>
    where
        B: Send;
}

/// Middleware configuration
#[derive(Clone)]
pub struct HodeiAuthorizationLayer {
    authorization_engine: Arc<dyn AuthorizationEngine>,
    principal_configuration: PrincipalConfiguration,
    skipped_endpoints: Vec<SkippedEndpoint>,
    logger: Arc<dyn Logger>,
    context_extractor: Arc<dyn ContextExtractor>,
}

impl HodeiAuthorizationLayer {
    pub fn new(authorization_engine: Arc<dyn AuthorizationEngine>) -> Self {
        Self {
            authorization_engine,
            principal_configuration: PrincipalConfiguration::IdentityToken,
            skipped_endpoints: Vec::new(),
            logger: Arc::new(ConsoleLogger),
            context_extractor: Arc::new(NoContextExtractor),
        }
    }
    
    pub fn with_principal_configuration(mut self, config: PrincipalConfiguration) -> Self {
        self.principal_configuration = config;
        self
    }
    
    pub fn skip_endpoint(mut self, http_verb: &str, path: &str) -> Self {
        self.skipped_endpoints.push(SkippedEndpoint::new(http_verb, path));
        self
    }
    
    pub fn with_logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = logger;
        self
    }
    
    pub fn with_context_extractor(mut self, extractor: Arc<dyn ContextExtractor>) -> Self {
        self.context_extractor = extractor;
        self
    }
}

impl<S> Layer<S> for HodeiAuthorizationLayer {
    type Service = HodeiAuthorizationService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        HodeiAuthorizationService::new(
            inner,
            self.authorization_engine.clone(),
            self.principal_configuration.clone(),
            self.skipped_endpoints.clone(),
            self.logger.clone(),
            self.context_extractor.clone(),
        )
    }
}
```

### 5. **Service Implementation**

#### Tower Service:
```rust
use tower::Service;
use http::{Request, Response, StatusCode};
use std::future::Future;
use std::pin::Pin;

pub struct HodeiAuthorizationService<S> {
    inner: S,
    authorization_engine: Arc<dyn AuthorizationEngine>,
    principal_configuration: PrincipalConfiguration,
    skipped_endpoints: Vec<SkippedEndpoint>,
    logger: Arc<dyn Logger>,
    context_extractor: Arc<dyn ContextExtractor>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for HodeiAuthorizationService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }
    
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Implementation similar to Express.js middleware
        todo!()
    }
}
```

---

## 🎯 Comparación de Arquitecturas

### Express.js vs Rust - Mapeo de Conceptos

| Concepto Express.js | Equivalente Rust | Ventaja Rust |
|---------------------|------------------|--------------|
| `ExpressAuthorizationMiddleware` | `HodeiAuthorizationLayer` | Type safety |
| `CedarInlineAuthorizationEngine` | `CedarInlineAuthorizationEngine` | Zero-cost abstractions |
| `getPrincipalEntity(req)` | `PrincipalExtractor` trait | Trait-based polymorphism |
| `skippedEndpoints` | `Vec<SkippedEndpoint>` | Compile-time optimization |
| `logger.debug/log` | `Logger` trait | Structured logging |
| `contextConfiguration` | `ContextExtractor` trait | Composability |
| `Entity` object | `CedarEntity` struct | Memory safety |

### Mejoras Propuestas para Rust

1. **Type Safety**: Uso de enums y traits en lugar de strings
2. **Zero-cost Abstractions**: Traits en lugar de objetos dinámicos
3. **Memory Safety**: Sin null pointers o runtime errors
4. **Async/Await**: Soporte nativo para async
5. **Error Handling**: Result types en lugar de excepciones
6. **Composability**: Builder pattern para configuración
7. **Testing**: Traits mockables para tests

---

## 📊 Resumen de Implementación

### Fases de Desarrollo

1. **Fase 1**: Entity system y builders
2. **Fase 2**: Authorization engine trait
3. **Fase 3**: Principal extractors
4. **Fase 4**: Tower layer y service
5. **Fase 5**: Context extractors
6. **Fase 6**: Integration tests
7. **Fase 7**: Documentation y examples

### Beneficios de la Arquitectura Rust

- **100% type safety** en compile-time
- **Zero-cost abstractions** para máximo rendimiento
- **Memory safety** sin garbage collection
- **Async/await nativo** para alta concurrencia
- **Trait-based design** para extensibilidad
- **Builder pattern** para API fluida
- **Comprehensive testing** con mocks

---

## 🚀 Próximos Pasos

1. Implementar `CedarEntity` y `EntityIdentifier`
2. Crear `AuthorizationEngine` trait
3. Desarrollar extractors system
4. Implementar Tower layer/service
5. Crear ejemplos y tests
6. Documentar API completa

**Esta arquitectura proporciona una base sólida para crear una implementación Rust equivalente con las mejores prácticas del lenguaje.**
