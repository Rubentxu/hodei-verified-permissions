# Plan de Implementación: Gaps de Historias de Usuario 4
## Parte 1: Gaps 1-5

**Fecha:** 27 de Octubre de 2025  
**Versión:** 1.0

---

## Gap 1: IsAuthorizedWithTokenRequestBuilder

**Prioridad:** 🔴 CRÍTICO | **Tiempo:** 1-2h | **Complejidad:** Baja

### Solución

**Archivo:** `sdk/src/builders.rs`

```rust
pub struct IsAuthorizedWithTokenRequestBuilder {
    policy_store_id: String,
    identity_source_id: String,
    access_token: String,
    action: Option<EntityIdentifier>,
    resource: Option<EntityIdentifier>,
    context: Option<String>,
    entities: Vec<Entity>,
}

impl IsAuthorizedWithTokenRequestBuilder {
    pub fn new(
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            access_token: access_token.into(),
            action: None,
            resource: None,
            context: None,
            entities: Vec::new(),
        }
    }

    pub fn action(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.action = Some(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    pub fn resource(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.resource = Some(EntityIdentifier {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        });
        self
    }

    pub fn context(mut self, context_json: impl Into<String>) -> Self {
        self.context = Some(context_json.into());
        self
    }

    pub fn add_entity(mut self, entity: Entity) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn build(self) -> IsAuthorizedWithTokenRequest {
        IsAuthorizedWithTokenRequest {
            policy_store_id: self.policy_store_id,
            identity_source_id: self.identity_source_id,
            access_token: self.access_token,
            action: self.action,
            resource: self.resource,
            context: self.context,
            entities: self.entities,
        }
    }
}
```

### Checklist
- [ ] Agregar a `sdk/src/builders.rs`
- [ ] Tests unitarios
- [ ] Actualizar `sdk/src/lib.rs`
- [ ] Documentación

---

## Gap 2: Extracción de Path Parameters

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 4-6h | **Complejidad:** Media

### Solución

**Archivo:** `sdk/src/middleware/extractor.rs`

```rust
use matchit::Router;

pub struct ParameterizedExtractor {
    policy_store_id: String,
    identity_source_id: String,
    route_mappings: Vec<(String, String, String)>,
}

impl ParameterizedExtractor {
    pub fn new(policy_store_id: impl Into<String>, identity_source_id: impl Into<String>) -> Self {
        Self {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            route_mappings: Vec::new(),
        }
    }

    pub fn add_route_mapping(
        mut self,
        pattern: impl Into<String>,
        resource_type: impl Into<String>,
        param_name: impl Into<String>,
    ) -> Self {
        self.route_mappings.push((
            pattern.into(),
            resource_type.into(),
            param_name.into(),
        ));
        self
    }

    fn extract_param(path: &str, pattern: &str) -> Option<String> {
        let mut router = Router::new();
        router.insert(pattern, ()).ok()?;
        let m = router.at(path).ok()?;
        m.params.get("docId").map(|s| s.to_string())
    }
}

#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for ParameterizedExtractor
where
    B: Send + Sync,
{
    type Error = crate::middleware::MiddlewareError;

    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
        let _token = DefaultExtractor::extract_token(req)?;
        let path = req.uri().path();
        let method = req.method().as_str();

        let action = match method {
            "GET" | "HEAD" => "Action::\"read\"",
            "POST" => "Action::\"create\"",
            "PUT" | "PATCH" => "Action::\"update\"",
            "DELETE" => "Action::\"delete\"",
            _ => return Err(crate::middleware::MiddlewareError::ExtractionFailed(
                format!("Unsupported HTTP method: {}", method)
            )),
        };

        let mut resource = format!("Resource::\"{}\"", path);
        
        for (pattern, resource_type, _param_name) in &self.route_mappings {
            if let Some(param_value) = Self::extract_param(path, pattern) {
                resource = format!("{}::\"{}\"", resource_type, param_value);
                break;
            }
        }

        Ok(AuthorizationRequestParts {
            principal: String::new(),
            action: action.to_string(),
            resource,
            context: None,
        })
    }
}
```

### Checklist
- [ ] Crear `ParameterizedExtractor`
- [ ] Actualizar `VerifiedPermissionsLayer`
- [ ] Tests con diferentes patrones
- [ ] Documentación

---

## Gap 3: Rotación Automática de JWKS

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 6-8h | **Complejidad:** Media

### Solución

**Archivo:** `verified-permissions/infrastructure/src/jwt/validator.rs`

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct JwksConfig {
    pub cache_ttl: Duration,
    pub refresh_interval: Duration,
    pub max_retries: u32,
}

impl Default for JwksConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(24 * 3600),
            refresh_interval: Duration::from_secs(12 * 3600),
            max_retries: 3,
        }
    }
}

struct JwksCacheEntry {
    keys: JwkSet,
    cached_at: Instant,
    last_refresh_attempt: Instant,
}

impl JwksCacheEntry {
    fn is_expired(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() > ttl
    }

    fn should_refresh(&self, interval: Duration) -> bool {
        self.last_refresh_attempt.elapsed() > interval
    }
}

pub struct JwtValidator {
    http_client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, JwksCacheEntry>>>,
    config: JwksConfig,
}

impl JwtValidator {
    pub fn new(config: JwksConfig) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    async fn get_jwks(&self, jwks_uri: &str) -> Result<JwkSet> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(jwks_uri) {
            if !entry.is_expired(self.config.cache_ttl) {
                if entry.should_refresh(self.config.refresh_interval) {
                    let uri = jwks_uri.to_string();
                    let client = self.http_client.clone();
                    tokio::spawn(async move {
                        let _ = Self::fetch_jwks(&client, &uri).await;
                    });
                }
                return Ok(entry.keys.clone());
            }
        }

        let keys = Self::fetch_jwks(&self.http_client, jwks_uri).await?;
        
        cache.insert(
            jwks_uri.to_string(),
            JwksCacheEntry {
                keys: keys.clone(),
                cached_at: Instant::now(),
                last_refresh_attempt: Instant::now(),
            },
        );

        Ok(keys)
    }

    async fn fetch_jwks(client: &reqwest::Client, jwks_uri: &str) -> Result<JwkSet> {
        let response = client
            .get(jwks_uri)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        response.json().await
    }

    pub async fn cleanup_expired_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, entry| !entry.is_expired(self.config.cache_ttl));
    }

    pub async fn invalidate_cache(&self, jwks_uri: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(jwks_uri);
    }
}
```

### Checklist
- [ ] Agregar `JwksConfig`
- [ ] Implementar caché con TTL
- [ ] Refresh proactivo
- [ ] Tests de expiración
- [ ] Cleanup periódico

---

## Gap 4: Transformaciones Avanzadas de Claims

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 8-12h | **Complejidad:** Alta

### Solución

**Archivo:** `verified-permissions/infrastructure/src/jwt/transforms.rs`

```rust
use regex::Regex;

#[derive(Debug, Clone)]
pub enum ValueTransform {
    None,
    SplitLast(String),
    RegexCapture { pattern: String, group: usize },
    RegexReplace { pattern: String, replacement: String },
    Prefix(String),
    Suffix(String),
    Lowercase,
    Uppercase,
    Trim,
    Chain(Vec<ValueTransform>),
}

impl ValueTransform {
    pub fn apply(&self, value: &str) -> Result<String, String> {
        match self {
            ValueTransform::None => Ok(value.to_string()),
            ValueTransform::SplitLast(sep) => {
                Ok(value.split(sep).last().unwrap_or(value).to_string())
            }
            ValueTransform::RegexCapture { pattern, group } => {
                let re = Regex::new(pattern)
                    .map_err(|e| format!("Invalid regex: {}", e))?;
                re.captures(value)
                    .and_then(|caps| caps.get(*group).map(|m| m.as_str().to_string()))
                    .ok_or_else(|| format!("No match for pattern: {}", pattern))
            }
            ValueTransform::RegexReplace { pattern, replacement } => {
                let re = Regex::new(pattern)
                    .map_err(|e| format!("Invalid regex: {}", e))?;
                Ok(re.replace_all(value, replacement).to_string())
            }
            ValueTransform::Prefix(prefix) => Ok(format!("{}{}", prefix, value)),
            ValueTransform::Suffix(suffix) => Ok(format!("{}{}", value, suffix)),
            ValueTransform::Lowercase => Ok(value.to_lowercase()),
            ValueTransform::Uppercase => Ok(value.to_uppercase()),
            ValueTransform::Trim => Ok(value.trim().to_string()),
            ValueTransform::Chain(transforms) => {
                let mut result = value.to_string();
                for transform in transforms {
                    result = transform.apply(&result)?;
                }
                Ok(result)
            }
        }
    }
}
```

### Checklist
- [ ] Crear `transforms.rs`
- [ ] Implementar `apply()`
- [ ] Agregar dependencia `regex`
- [ ] Tests exhaustivos
- [ ] Documentación

---

## Gap 5: Trait Explícito para AuthorizationClient

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 2-3h | **Complejidad:** Baja

### Solución

**Archivo:** `sdk/src/client.rs`

```rust
#[async_trait]
pub trait AuthorizationClientTrait: Send + Sync {
    async fn is_authorized(
        &self,
        policy_store_id: impl Into<String> + Send,
        principal: impl Into<String> + Send,
        action: impl Into<String> + Send,
        resource: impl Into<String> + Send,
    ) -> Result<IsAuthorizedResponse>;

    async fn is_authorized_with_token(
        &self,
        policy_store_id: impl Into<String> + Send,
        identity_source_id: impl Into<String> + Send,
        access_token: impl Into<String> + Send,
        action: impl Into<String> + Send,
        resource: impl Into<String> + Send,
    ) -> Result<IsAuthorizedResponse>;

    // ... más métodos
}

#[async_trait]
impl AuthorizationClientTrait for AuthorizationClient {
    // Implementar todos los métodos
}
```

### Checklist
- [ ] Crear trait `AuthorizationClientTrait`
- [ ] Implementar para `AuthorizationClient`
- [ ] Re-exportar desde `lib.rs`
- [ ] Ejemplos de mocking
- [ ] Tests con mocks

