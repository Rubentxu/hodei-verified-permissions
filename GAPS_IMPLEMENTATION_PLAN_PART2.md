# Plan de Implementación: Gaps de Historias de Usuario 4
## Parte 2: Gaps 6-10

**Fecha:** 27 de Octubre de 2025  
**Versión:** 1.0

---

## Gap 6: Exposición de AuthorizationDecision

**Prioridad:** 🔴 CRÍTICO | **Tiempo:** 1-2h | **Complejidad:** Baja

### Solución

**Archivo:** `sdk/src/authorization.rs` (nuevo)

```rust
use serde::{Deserialize, Serialize};
use crate::proto::Decision;

/// Decisión de autorización con contexto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationDecision {
    /// Decisión: Allow o Deny
    pub decision: Decision,
    
    /// Políticas que determinaron la decisión
    pub determining_policies: Vec<String>,
    
    /// Razón de la decisión
    pub reason: Option<String>,
    
    /// Errores durante la evaluación
    pub errors: Vec<String>,
    
    /// Timestamp de la decisión
    pub timestamp: i64,
}

impl AuthorizationDecision {
    pub fn allow(determining_policies: Vec<String>) -> Self {
        Self {
            decision: Decision::Allow,
            determining_policies,
            reason: None,
            errors: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    pub fn deny(reason: Option<String>) -> Self {
        Self {
            decision: Decision::Deny,
            determining_policies: vec![],
            reason,
            errors: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    pub fn is_allowed(&self) -> bool {
        self.decision == Decision::Allow
    }

    pub fn is_denied(&self) -> bool {
        self.decision == Decision::Deny
    }
}
```

**Archivo:** `sdk/src/lib.rs`

```rust
pub mod authorization;
pub use authorization::AuthorizationDecision;
```

### Uso en Handlers

```rust
use axum::Extension;
use hodei_permissions_sdk::AuthorizationDecision;

async fn get_document(
    Extension(decision): Extension<AuthorizationDecision>,
) -> impl IntoResponse {
    tracing::info!(
        "Access granted by policies: {:?}",
        decision.determining_policies
    );
    "Document content"
}
```

### Checklist
- [ ] Crear `authorization.rs`
- [ ] Re-exportar desde `lib.rs`
- [ ] Actualizar middleware
- [ ] Ejemplos de uso
- [ ] Tests

---

## Gap 7: Validación de Configuración OIDC

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 2-3h | **Complejidad:** Baja

### Solución

**Archivo:** `verified-permissions/application/src/services/identity_source_validator.rs` (nuevo)

```rust
use url::Url;
use std::time::Duration;

pub struct IdentitySourceValidator;

impl IdentitySourceValidator {
    /// Validar configuración OIDC
    pub async fn validate_oidc_config(
        issuer: &str,
        client_ids: &[String],
    ) -> Result<()> {
        // 1. Validar formato de URL
        let url = Url::parse(issuer)
            .map_err(|_| AuthorizationError::InvalidConfiguration(
                format!("Invalid issuer URL: {}", issuer)
            ))?;

        // 2. Verificar HTTPS (excepto localhost)
        if url.scheme() != "https" && !issuer.contains("localhost") {
            return Err(AuthorizationError::InvalidConfiguration(
                "Issuer must use HTTPS".to_string()
            ));
        }

        // 3. Obtener .well-known/openid-configuration
        let well_known_url = format!(
            "{}/.well-known/openid-configuration",
            issuer.trim_end_matches('/')
        );
        
        let client = reqwest::Client::new();
        let response = client
            .get(&well_known_url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AuthorizationError::InvalidConfiguration(
                format!("Failed to fetch OIDC configuration: {}", e)
            ))?;

        if !response.status().is_success() {
            return Err(AuthorizationError::InvalidConfiguration(
                format!("OIDC configuration not found at {}", well_known_url)
            ));
        }

        let config: serde_json::Value = response.json().await
            .map_err(|e| AuthorizationError::InvalidConfiguration(
                format!("Invalid OIDC configuration JSON: {}", e)
            ))?;

        // 4. Validar que tenga jwks_uri
        if config.get("jwks_uri").is_none() {
            return Err(AuthorizationError::InvalidConfiguration(
                "OIDC configuration missing 'jwks_uri'".to_string()
            ));
        }

        // 5. Validar JWKS
        let jwks_uri = config["jwks_uri"].as_str().unwrap();
        let jwks_response = client
            .get(jwks_uri)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AuthorizationError::InvalidConfiguration(
                format!("Failed to fetch JWKS: {}", e)
            ))?;

        if !jwks_response.status().is_success() {
            return Err(AuthorizationError::InvalidConfiguration(
                format!("JWKS not accessible at {}", jwks_uri)
            ));
        }

        // 6. Validar que JWKS sea JSON válido con keys
        let jwks: serde_json::Value = jwks_response.json().await
            .map_err(|e| AuthorizationError::InvalidConfiguration(
                format!("Invalid JWKS JSON: {}", e)
            ))?;

        if jwks.get("keys").is_none() {
            return Err(AuthorizationError::InvalidConfiguration(
                "JWKS missing 'keys' array".to_string()
            ));
        }

        Ok(())
    }
}
```

### Checklist
- [ ] Crear `identity_source_validator.rs`
- [ ] Agregar validación en `create_identity_source()`
- [ ] Tests de validación
- [ ] Manejo de errores

---

## Gap 8: Soporte para Múltiples Issuers

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 12-16h | **Complejidad:** Alta

### Solución (Arquitectura)

**Cambios en Modelo de Datos:**

```rust
// Antes: 1 Identity Source por PolicyStore
pub struct PolicyStore {
    id: PolicyStoreId,
    identity_source_id: String, // Solo uno
}

// Después: Múltiples Identity Sources por PolicyStore
pub struct PolicyStore {
    id: PolicyStoreId,
    identity_sources: Vec<IdentitySourceId>,
    default_identity_source: Option<IdentitySourceId>,
}
```

**Lógica de Selección en Data Plane:**

```rust
// verified-permissions/api/src/grpc/data_plane.rs

async fn is_authorized_with_token(
    &self,
    req: IsAuthorizedWithTokenRequest,
) -> Result<IsAuthorizedResponse> {
    // 1. Obtener PolicyStore
    let policy_store = self.repository.get_policy_store(&req.policy_store_id).await?;

    // 2. Si identity_source_id especificado, usarlo
    let identity_source_id = if !req.identity_source_id.is_empty() {
        req.identity_source_id.clone()
    } else {
        // 3. Si no, usar default o detectar del token
        if let Some(default_id) = &policy_store.default_identity_source {
            default_id.clone()
        } else {
            // 4. Detectar issuer del token sin validar
            let issuer = extract_issuer_from_token(&req.access_token)?;
            self.find_identity_source_by_issuer(&policy_store.id, &issuer).await?
        }
    };

    // 5. Continuar con validación normal
    let identity_source = self.repository
        .get_identity_source(&policy_store.id, &identity_source_id)
        .await?;

    // ... resto de la lógica
}

fn extract_issuer_from_token(token: &str) -> Result<String> {
    // Decodificar JWT sin validar firma
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthorizationError::InvalidToken("Invalid JWT format".to_string()));
    }

    let payload = base64_decode(parts[1])?;
    let claims: serde_json::Value = serde_json::from_slice(&payload)?;
    
    claims["iss"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AuthorizationError::InvalidToken("Missing 'iss' claim".to_string()))
}
```

### Checklist
- [ ] Actualizar modelo de PolicyStore
- [ ] Agregar `default_identity_source`
- [ ] Implementar detección de issuer
- [ ] Actualizar API gRPC
- [ ] Migrations de BD
- [ ] Tests de múltiples issuers

---

## Gap 9: Circuit Breaker y Retry Logic

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 10-14h | **Complejidad:** Alta

### Solución

**Archivo:** `sdk/src/client.rs`

```rust
use backoff::{ExponentialBackoff, backoff::Backoff};
use std::time::Duration;

pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }
}

pub struct AuthorizationClient {
    data_client: AuthorizationDataClient<Channel>,
    control_client: AuthorizationControlClient<Channel>,
    retry_config: RetryConfig,
}

impl AuthorizationClient {
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    async fn with_retry<F, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>>>>,
    {
        let mut backoff = ExponentialBackoff {
            current_interval: self.retry_config.initial_backoff,
            initial_interval: self.retry_config.initial_backoff,
            max_interval: self.retry_config.max_backoff,
            multiplier: self.retry_config.multiplier,
            ..Default::default()
        };

        let mut attempts = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.retry_config.max_retries {
                        return Err(e);
                    }

                    if let Some(duration) = backoff.next_backoff() {
                        tokio::time::sleep(duration).await;
                    }
                }
            }
        }
    }

    pub async fn is_authorized(
        &self,
        policy_store_id: impl Into<String>,
        principal: impl Into<String>,
        action: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<IsAuthorizedResponse> {
        let policy_store_id = policy_store_id.into();
        let principal = principal.into();
        let action = action.into();
        let resource = resource.into();

        self.with_retry(|| {
            let policy_store_id = policy_store_id.clone();
            let principal = principal.clone();
            let action = action.clone();
            let resource = resource.clone();
            let mut client = self.data_client.clone();

            Box::pin(async move {
                let request = IsAuthorizedRequest {
                    policy_store_id,
                    principal: Some(parse_entity_id(principal)?),
                    action: Some(parse_entity_id(action)?),
                    resource: Some(parse_entity_id(resource)?),
                    context: None,
                    entities: vec![],
                };

                client.is_authorized(request).await.map(|r| r.into_inner())
            })
        }).await
    }
}
```

### Checklist
- [ ] Agregar dependencia `backoff`
- [ ] Crear `RetryConfig`
- [ ] Implementar `with_retry()`
- [ ] Aplicar a todos los métodos
- [ ] Tests de retry
- [ ] Tests de circuit breaker

---

## Gap 10: Métricas y Observabilidad

**Prioridad:** ⚠️ IMPORTANTE | **Tiempo:** 12-16h | **Complejidad:** Alta

### Solución

**Archivo:** `sdk/src/middleware/metrics.rs` (nuevo)

```rust
use prometheus::{Counter, Histogram, Registry};
use std::sync::Arc;

pub struct AuthorizationMetrics {
    pub decisions_allow: Counter,
    pub decisions_deny: Counter,
    pub decision_latency: Histogram,
    pub token_validation_errors: Counter,
    pub extraction_errors: Counter,
}

impl AuthorizationMetrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        Ok(Self {
            decisions_allow: Counter::new(
                "authorization_decisions_allow_total",
                "Total authorization decisions: ALLOW"
            )?,
            decisions_deny: Counter::new(
                "authorization_decisions_deny_total",
                "Total authorization decisions: DENY"
            )?,
            decision_latency: Histogram::new(
                "authorization_decision_latency_seconds",
                "Authorization decision latency in seconds"
            )?,
            token_validation_errors: Counter::new(
                "authorization_token_validation_errors_total",
                "Total token validation errors"
            )?,
            extraction_errors: Counter::new(
                "authorization_extraction_errors_total",
                "Total extraction errors"
            )?,
        })
    }

    pub fn record_allow(&self) {
        self.decisions_allow.inc();
    }

    pub fn record_deny(&self) {
        self.decisions_deny.inc();
    }

    pub fn record_latency(&self, duration: Duration) {
        self.decision_latency.observe(duration.as_secs_f64());
    }

    pub fn record_token_error(&self) {
        self.token_validation_errors.inc();
    }

    pub fn record_extraction_error(&self) {
        self.extraction_errors.inc();
    }
}
```

**Integración en Middleware:**

```rust
// sdk/src/middleware/service.rs

pub struct VerifiedPermissionsService<S> {
    inner: S,
    client: Arc<AuthorizationClient>,
    metrics: Option<Arc<AuthorizationMetrics>>,
    // ... otros campos
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for VerifiedPermissionsService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
{
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let metrics = self.metrics.clone();

        Box::pin(async move {
            match decision {
                Decision::Allow => {
                    if let Some(m) = &metrics {
                        m.record_allow();
                        m.record_latency(start.elapsed());
                    }
                    // Continuar
                }
                Decision::Deny => {
                    if let Some(m) = &metrics {
                        m.record_deny();
                        m.record_latency(start.elapsed());
                    }
                    // Rechazar
                }
            }
        })
    }
}
```

### Checklist
- [ ] Agregar dependencia `prometheus`
- [ ] Crear `AuthorizationMetrics`
- [ ] Integrar en middleware
- [ ] Integrar en cliente
- [ ] Exponer endpoint `/metrics`
- [ ] Tests de métricas
- [ ] Documentación

---

## Resumen de Esfuerzo Total

| Gap | Prioridad | Tiempo | Complejidad |
|-----|-----------|--------|-------------|
| 1. IsAuthorizedWithTokenRequestBuilder | 🔴 | 1-2h | Baja |
| 2. Path Parameters | ⚠️ | 4-6h | Media |
| 3. JWKS Rotation | ⚠️ | 6-8h | Media |
| 4. Advanced Transforms | ⚠️ | 8-12h | Alta |
| 5. Trait Explícito | ⚠️ | 2-3h | Baja |
| 6. AuthorizationDecision | 🔴 | 1-2h | Baja |
| 7. OIDC Validation | ⚠️ | 2-3h | Baja |
| 8. Multiple Issuers | ⚠️ | 12-16h | Alta |
| 9. Circuit Breaker | ⚠️ | 10-14h | Alta |
| 10. Métricas | ⚠️ | 12-16h | Alta |
| **TOTAL** | | **59-82h** | |

**Recomendación:** Implementar en fases:
- **Fase 1 (Crítico):** Gaps 1, 6 (3-4h)
- **Fase 2 (Sprint 1):** Gaps 2, 5, 7 (8-12h)
- **Fase 3 (Sprint 2):** Gaps 3, 4 (14-20h)
- **Fase 4 (Sprint 3+):** Gaps 8, 9, 10 (34-46h)

