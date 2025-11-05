# ANEXO A: Implementación del Security Token Service (STS) en Rust

## 1. Descripción del STS y Responsabilidades

El **Security Token Service (STS)** es el componente crítico de Hodei-IAM que permite la **delegación temporal de permisos** mediante el mecanismo de **AssumeRole**. Es el pilar fundamental para implementar los siguientes casos de uso:

### Casos de Uso Principales:

1. **CI/CD con Credenciales Temporales**
   - Un pipeline de CI/CD no debe tener credenciales permanentes con permisos amplios.
   - El pipeline autentica con credenciales limitadas y llama a `AssumeRole` para obtener credenciales temporales con permisos específicos para desplegar.

2. **Acceso Cross-Account**
   - Un servicio en la cuenta A necesita acceder a recursos en la cuenta B.
   - El servicio asume un rol en la cuenta B que tiene una TrustPolicy permitiendo principals de la cuenta A.

3. **Escalado de Privilegios Controlado**
   - Un desarrollador tiene permisos básicos, pero para realizar operaciones críticas necesita asumir un rol con más permisos.
   - Cada asunción de rol queda registrada en logs de auditoría.

4. **Federación de Identidades**
   - Usuarios externos (ej: desde OIDC, SAML) pueden obtener credenciales temporales sin crear usuarios IAM permanentes.

---

## 2. Arquitectura del STS

### 2.1. Componentes Principales

```mermaid
graph TD
    A[Cliente con Credenciales] -->|1. AssumeRole Request| B[STS API]
    B -->|2. Validar Identidad| C[Identity Store]
    B -->|3. Consultar TrustPolicy del Rol| D[Role Store]
    D -->|4. TrustPolicy| B
    B -->|5. Evaluar TrustPolicy| E[Policy Engine]
    E -->|6. ALLOW/DENY| B
    B -->|7. Generar Tokens| F[Token Generator]
    F -->|8. Firmar Tokens| G[Crypto Service]
    G -->|9. Token Firmado| F
    F -->|10. Almacenar Session| H[Session Store Redis/DB]
    B -->|11. Temporary Credentials| A
    
    style B fill:#f96
    style E fill:#9cf
    style F fill:#fc9
    style H fill:#9f9
```

### 2.2. Flujo Detallado de AssumeRole

```rust
// Estructura de la petición
pub struct AssumeRoleRequest {
    pub role_hrn: String,                    // hrn:hodei-iam::account:role/ci-cd-deployer
    pub session_name: String,                // identificador de auditoría
    pub duration_seconds: u32,               // 900 a 43200 (15 min - 12 hrs)
    pub external_id: Option<String>,         // para mitigar "confused deputy"
    pub policy: Option<String>,              // política de sesión (reduce permisos)
    pub tags: Option<Vec<SessionTag>>,       // tags de sesión para ABAC
}

// Estructura de la respuesta
pub struct AssumeRoleResponse {
    pub credentials: TemporaryCredentials,
    pub assumed_role_user: AssumedRoleUser,
    pub packed_policy_size: u32,
}

pub struct TemporaryCredentials {
    pub access_key_id: String,               // ASIAXXX (prefijo temporal)
    pub secret_access_key: String,           // clave secreta temporal
    pub session_token: String,               // JWT con metadata
    pub expiration: DateTime<Utc>,           // cuándo expira
}
```

---

## 3. Especificaciones Técnicas de Alto Rendimiento

### 3.1. Generación de Tokens con JWT

El `session_token` debe ser un **JWT (JSON Web Token)** firmado que contiene:

```rust
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTokenClaims {
    // Claims estándar JWT
    pub iss: String,                    // "hodei-iam-sts"
    pub sub: String,                    // hrn del rol asumido
    pub aud: String,                    // "hodei-iam"
    pub exp: i64,                       // timestamp de expiración
    pub iat: i64,                       // timestamp de emisión
    pub jti: String,                    // ID único del token (UUID v4)
    
    // Claims personalizados
    pub principal_hrn: String,          // quién asumió el rol
    pub role_hrn: String,               // rol asumido
    pub session_name: String,           // nombre de sesión
    pub session_id: String,             // ID de sesión en store
    pub policy_list: Vec<String>,       // HRNs de políticas adjuntas
    pub session_policy: Option<String>, // política inline de sesión
    pub tags: Option<Vec<SessionTag>>,  // tags para ABAC
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTag {
    pub key: String,
    pub value: String,
}
```

**Ventajas de JWT:**
- ✅ **Stateless**: El token contiene toda la información necesaria
- ✅ **Performance**: No requiere lookup en DB en cada request (solo validación de firma)
- ✅ **Distribuido**: Cualquier instancia del servicio puede validar
- ✅ **Revocación**: Se puede implementar una blacklist con TTL corto en Redis

### 3.2. Algoritmos Criptográficos Recomendados

```rust
// Firma de tokens JWT
Algorithm::RS256  // RSA con SHA-256 (recomendado para producción)
// O alternativa con mejor performance:
Algorithm::EdDSA  // Ed25519 (más rápido, claves más cortas)

// Generación de claves de acceso temporales
use rand::Rng;
use rand::distributions::Alphanumeric;

pub fn generate_access_key_id() -> String {
    let random: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    format!("ASIA{}", random) // ASIA = Amazon Style Temporary
}

pub fn generate_secret_access_key() -> String {
    use base64::{Engine as _, engine::general_purpose};
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    general_purpose::STANDARD.encode(random_bytes)
}
```

---

## 4. Librerías y Servicios en Rust para Implementación

### 4.1. Librerías Core Recomendadas

#### A. Manejo de JWT - `jsonwebtoken`
```toml
[dependencies]
jsonwebtoken = "9.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Características:**
- ✅ Alto rendimiento (benchmarks: ~100K tokens/sec en validación)
- ✅ Soporte para RS256, RS384, RS512, ES256, ES384, EdDSA
- ✅ Validación completa de claims (exp, iat, nbf, etc.)
- ✅ Zero-copy parsing donde sea posible

**Ejemplo de uso:**
```rust
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

// Generar token
let claims = SessionTokenClaims {
    iss: "hodei-iam-sts".to_string(),
    sub: role_hrn.clone(),
    exp: (Utc::now() + Duration::hours(1)).timestamp(),
    // ... otros campos
};

let token = encode(
    &Header::new(Algorithm::RS256),
    &claims,
    &EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?,
)?;

// Validar token
let validation = Validation::new(Algorithm::RS256);
let token_data = decode::<SessionTokenClaims>(
    &token,
    &DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?,
    &validation,
)?;
```

#### B. Criptografía - `ring`
```toml
[dependencies]
ring = "0.17"
```

**Características:**
- ✅ Implementación de BoringSSL (usado por Google/AWS)
- ✅ Extremadamente rápido y seguro
- ✅ Auditorías de seguridad constantes
- ✅ Generación de números aleatorios criptográficamente seguros

#### C. Session Store - `redis` + `deadpool-redis`
```toml
[dependencies]
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
deadpool-redis = "0.15"
```

**Características:**
- ✅ Pool de conexiones de alto rendimiento
- ✅ Soporte async completo con Tokio
- ✅ Reconexión automática
- ✅ Pipeline para batch operations

**Estructura de datos en Redis:**
```rust
// Key: session:{session_id}
// Value: JSON con metadata de sesión
// TTL: mismo que duration_seconds del token

pub struct SessionMetadata {
    pub session_id: String,
    pub principal_hrn: String,
    pub role_hrn: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub is_revoked: bool,
}

// Revocación de token
pub async fn revoke_session(redis: &RedisPool, session_id: &str) -> Result<()> {
    let mut conn = redis.get().await?;
    let key = format!("session:{}", session_id);
    redis::cmd("HSET")
        .arg(&key)
        .arg("is_revoked")
        .arg("true")
        .query_async(&mut conn)
        .await?;
    Ok(())
}

// Verificación rápida (< 1ms)
pub async fn is_session_valid(redis: &RedisPool, session_id: &str) -> Result<bool> {
    let mut conn = redis.get().await?;
    let key = format!("session:{}", session_id);
    let is_revoked: Option<String> = redis::cmd("HGET")
        .arg(&key)
        .arg("is_revoked")
        .query_async(&mut conn)
        .await?;
    Ok(is_revoked != Some("true".to_string()))
}
```

#### D. Time Management - `chrono`
```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

**Uso para manejo preciso de expiración:**
```rust
use chrono::{DateTime, Utc, Duration};

pub fn calculate_expiration(duration_seconds: u32) -> DateTime<Utc> {
    Utc::now() + Duration::seconds(duration_seconds as i64)
}

pub fn is_expired(expiration: DateTime<Utc>) -> bool {
    Utc::now() > expiration
}
```

---

### 4.2. Integración con Cedar Policy Engine

El STS debe evaluar la **TrustPolicy** del rol antes de emitir credenciales:

```rust
use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request};

pub async fn evaluate_trust_policy(
    authorizer: &Authorizer,
    policy_set: &PolicySet,
    entities: &Entities,
    principal: &str,
    role: &str,
) -> Result<bool> {
    let request = Request::new(
        EntityUid::from_str(principal)?,  // ej: User::"dev-bob"
        EntityUid::from_str("Action::\"sts:AssumeRole\"")?,
        EntityUid::from_str(role)?,       // ej: Role::"ci-cd-deployer"
        Context::empty(),
    )?;
    
    let response = authorizer.is_authorized(&request, policy_set, entities);
    Ok(response.decision() == Decision::Allow)
}
```

**Ejemplo de TrustPolicy en Cedar:**
```cedar
// Permitir que usuarios del grupo DevOps asuman el rol
permit(
  principal in Group::"DevOps",
  action == Action::"sts:AssumeRole",
  resource == Role::"ci-cd-deployer"
);

// Permitir con External ID (mitigación de confused deputy)
permit(
  principal,
  action == Action::"sts:AssumeRole",
  resource == Role::"cross-account-role"
)
when {
  context.external_id == "expected-external-id-12345"
};
```

---

## 5. Arquitectura de Servicios y Módulos

```rust
// src/sts/mod.rs
pub mod api;           // Endpoints gRPC/REST
pub mod token;         // Generación y validación de tokens
pub mod session;       // Gestión de sesiones
pub mod crypto;        // Operaciones criptográficas
pub mod trust;         // Evaluación de TrustPolicy
pub mod revocation;    // Sistema de revocación

// src/sts/api.rs
use tonic::{Request, Response, Status};

pub struct StsService {
    token_generator: TokenGenerator,
    session_store: SessionStore,
    policy_engine: PolicyEngine,
    crypto_service: CryptoService,
}

impl StsService {
    pub async fn assume_role(
        &self,
        request: Request<AssumeRoleRequest>,
    ) -> Result<Response<AssumeRoleResponse>, Status> {
        let req = request.into_inner();
        
        // 1. Validar identidad del caller
        let principal = self.authenticate_caller(&request).await?;
        
        // 2. Obtener el rol y su TrustPolicy
        let role = self.get_role(&req.role_hrn).await?;
        
        // 3. Evaluar TrustPolicy
        let allowed = self.policy_engine
            .evaluate_trust_policy(&principal, &role, &req.external_id)
            .await?;
            
        if !allowed {
            return Err(Status::permission_denied("TrustPolicy denied"));
        }
        
        // 4. Generar credenciales temporales
        let credentials = self.token_generator
            .generate_temporary_credentials(&role, &principal, req.duration_seconds)
            .await?;
        
        // 5. Crear sesión
        let session = self.session_store
            .create_session(&credentials, &role, &principal)
            .await?;
        
        // 6. Retornar credenciales
        Ok(Response::new(AssumeRoleResponse {
            credentials,
            assumed_role_user: AssumedRoleUser {
                hrn: format!("{}/session/{}", role.hrn, req.session_name),
                assumed_role_id: format!("{}:{}", role.role_id, session.session_id),
            },
            packed_policy_size: calculate_policy_size(&role.policies),
        }))
    }
}
```

---

## 6. Optimizaciones de Alto Rendimiento

### 6.1. Caché de Claves Públicas
```rust
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

pub struct KeyCache {
    cache: Cache<String, Arc<DecodingKey>>,
}

impl KeyCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(3600))
                .build(),
        }
    }
    
    pub async fn get_or_load(&self, key_id: &str) -> Result<Arc<DecodingKey>> {
        self.cache
            .try_get_with(key_id.to_string(), async {
                // Cargar clave desde DB/Vault
                load_public_key(key_id).await
            })
            .await
    }
}
```

### 6.2. Pool de Conexiones Redis Optimizado
```rust
use deadpool_redis::{Config, Runtime, Pool};

pub fn create_redis_pool() -> Pool {
    let cfg = Config::from_url("redis://localhost:6379");
    cfg.builder()
        .unwrap()
        .max_size(50)                    // conexiones máximas
        .wait_timeout(Some(Duration::from_secs(2)))
        .create_timeout(Some(Duration::from_secs(2)))
        .recycle_timeout(Some(Duration::from_secs(30)))
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap()
}
```

### 6.3. Validación de Token con Caché de Revocación
```rust
use moka::future::Cache;

pub struct TokenValidator {
    key_cache: KeyCache,
    revocation_cache: Cache<String, bool>, // session_id -> is_revoked
    redis_pool: RedisPool,
}

impl TokenValidator {
    pub async fn validate_token(&self, token: &str) -> Result<SessionTokenClaims> {
        // 1. Decodificar y validar firma (muy rápido, sin I/O)
        let token_data = decode::<SessionTokenClaims>(
            token,
            &self.key_cache.get_or_load("current").await?,
            &Validation::new(Algorithm::RS256),
        )?;
        
        let claims = token_data.claims;
        
        // 2. Verificar expiración (sin I/O)
        if Utc::now().timestamp() > claims.exp {
            return Err(Error::TokenExpired);
        }
        
        // 3. Verificar revocación (con caché de 1 segundo)
        let is_revoked = self.revocation_cache
            .get_with(claims.session_id.clone(), async {
                self.check_revocation_redis(&claims.session_id).await.unwrap_or(false)
            })
            .await;
            
        if is_revoked {
            return Err(Error::TokenRevoked);
        }
        
        Ok(claims)
    }
}
```

---

## 7. Métricas de Rendimiento Esperadas

Con la arquitectura propuesta:

| Operación | Latencia p50 | Latencia p99 | Throughput |
|-----------|-------------|--------------|------------|
| AssumeRole (completo) | < 10ms | < 50ms | 5,000 req/s |
| Validación de Token (caché hit) | < 0.5ms | < 2ms | 50,000 req/s |
| Validación de Token (caché miss) | < 5ms | < 15ms | 10,000 req/s |
| Revocación de Sesión | < 5ms | < 20ms | 10,000 req/s |

**Hardware de referencia:** 4 vCPUs, 8GB RAM, Redis en la misma red.

---

## 8. Seguridad y Mejores Prácticas

### 8.1. Rotación de Claves de Firma
```rust
pub struct KeyRotationManager {
    current_key_id: String,
    previous_key_id: Option<String>,
    next_rotation: DateTime<Utc>,
}

impl KeyRotationManager {
    pub async fn rotate_keys(&mut self) -> Result<()> {
        // Generar nuevo par de claves
        let new_key = self.generate_key_pair().await?;
        
        // Almacenar en Vault/DB
        self.store_key(&new_key).await?;
        
        // Actualizar referencias
        self.previous_key_id = Some(self.current_key_id.clone());
        self.current_key_id = new_key.key_id;
        self.next_rotation = Utc::now() + Duration::days(90);
        
        // Mantener clave anterior por período de gracia (para tokens aún válidos)
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(43200)).await; // 12 horas
            // Eliminar clave antigua
        });
        
        Ok(())
    }
}
```

### 8.2. Rate Limiting por Principal
```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct StsRateLimiter {
    limiter: RateLimiter<String, governor::state::direct::NotKeyed, 
                         governor::clock::DefaultClock>,
}

impl StsRateLimiter {
    pub fn new() -> Self {
        Self {
            limiter: RateLimiter::direct(
                Quota::per_second(NonZeroU32::new(10).unwrap())
            ),
        }
    }
    
    pub async fn check_rate_limit(&self, principal: &str) -> Result<()> {
        self.limiter.check()
            .map_err(|_| Error::RateLimitExceeded)?;
        Ok(())
    }
}
```

---

## 9. Alternativas y Servicios Externos

### 9.1. Request Authentication (CRÍTICO - Requerido para Control Plane)

Antes de cualquier operación STS, debemos autenticar al caller. AWS usa Signature V4, nosotros implementaremos un esquema similar.

#### Estructura de Request Firmada

```rust
pub struct RequestSignature {
    pub algorithm: String,              // "HODEI-HMAC-SHA256"
    pub credential: Credential,
    pub signed_headers: Vec<String>,
    pub signature: String,
}

pub struct Credential {
    pub access_key_id: String,          // "AKIA..." (permanente) o "ASIA..." (temporal)
    pub date: String,                   // "20251103"
    pub region: String,                 // "global" para IAM/STS
    pub service: String,                // "iam" o "sts"
}

// Formato del header Authorization:
// HODEI-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20251103/global/sts,
//   SignedHeaders=host;x-hodei-date,
//   Signature=fe5f80f77d5fa3beca038a248ff027d0445342fe2855ddc963176630326f1024
```

#### Algoritmo de Firma

```rust
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};

type HmacSha256 = Hmac<Sha256>;

pub struct RequestSigner {
    access_key_id: String,
    secret_access_key: String,
}

impl RequestSigner {
    pub fn sign_request(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &HashMap<String, String>,
        payload: &[u8],
        timestamp: DateTime<Utc>,
    ) -> String {
        // 1. Crear canonical request
        let canonical_headers = self.canonical_headers(headers);
        let signed_headers = self.signed_headers_list(headers);
        let payload_hash = hex::encode(Sha256::digest(payload));
        
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method,
            uri,
            query_string,
            canonical_headers,
            signed_headers,
            payload_hash
        );
        
        // 2. Crear string to sign
        let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let credential_scope = format!(
            "{}/{}/{}/hodei_request",
            timestamp.format("%Y%m%d"),
            "global",
            "sts"
        );
        
        let string_to_sign = format!(
            "HODEI-HMAC-SHA256\n{}\n{}\n{}",
            timestamp.to_rfc3339(),
            credential_scope,
            canonical_request_hash
        );
        
        // 3. Calcular signing key (derivada de secret)
        let k_date = self.hmac_sha256(
            format!("HODEI{}", self.secret_access_key).as_bytes(),
            timestamp.format("%Y%m%d").to_string().as_bytes()
        );
        let k_region = self.hmac_sha256(&k_date, b"global");
        let k_service = self.hmac_sha256(&k_region, b"sts");
        let k_signing = self.hmac_sha256(&k_service, b"hodei_request");
        
        // 4. Calcular firma
        let signature = self.hmac_sha256(&k_signing, string_to_sign.as_bytes());
        hex::encode(signature)
    }
    
    fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }
    
    fn canonical_headers(&self, headers: &HashMap<String, String>) -> String {
        let mut sorted_headers: Vec<_> = headers.iter().collect();
        sorted_headers.sort_by_key(|(k, _)| k.to_lowercase());
        
        sorted_headers
            .iter()
            .map(|(k, v)| format!("{}:{}\n", k.to_lowercase(), v.trim()))
            .collect()
    }
    
    fn signed_headers_list(&self, headers: &HashMap<String, String>) -> String {
        let mut keys: Vec<_> = headers.keys().map(|k| k.to_lowercase()).collect();
        keys.sort();
        keys.join(";")
    }
}
```

#### Validación en el Servidor

```rust
pub struct RequestAuthenticator {
    credential_store: Arc<CredentialStore>,
    session_validator: Arc<TokenValidator>,
}

impl RequestAuthenticator {
    pub async fn authenticate_request(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        headers: &HashMap<String, String>,
        payload: &[u8],
    ) -> Result<AuthenticatedPrincipal> {
        // 1. Extraer Authorization header
        let auth_header = headers.get("authorization")
            .ok_or(Error::MissingAuthHeader)?;
        
        // 2. Parsear componentes
        let (access_key_id, signed_headers, signature) = 
            self.parse_auth_header(auth_header)?;
        
        // 3. Obtener credenciales
        let credentials = if access_key_id.starts_with("ASIA") {
            // Credencial temporal: validar SessionToken
            let session_token = headers.get("x-hodei-security-token")
                .ok_or(Error::MissingSessionToken)?;
            
            let claims = self.session_validator.validate_token(session_token).await?;
            
            // Obtener credenciales temporales de la sesión
            self.credential_store.get_session_credentials(&claims.session_id).await?
        } else {
            // Credencial permanente
            self.credential_store.get_permanent_credentials(access_key_id).await?
        };
        
        // 4. Re-computar firma esperada
        let timestamp = headers.get("x-hodei-date")
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .ok_or(Error::InvalidTimestamp)?;
        
        let signer = RequestSigner {
            access_key_id: credentials.access_key_id.clone(),
            secret_access_key: credentials.secret_access_key.clone(),
        };
        
        let expected_signature = signer.sign_request(
            method,
            uri,
            query_string,
            headers,
            payload,
            timestamp.into(),
        );
        
        // 5. Comparar firmas (timing-safe)
        if !constant_time_eq(signature.as_bytes(), expected_signature.as_bytes()) {
            return Err(Error::SignatureMismatch);
        }
        
        // 6. Verificar timestamp (prevenir replay attacks)
        let now = Utc::now();
        let age = now.signed_duration_since(timestamp);
        if age > Duration::minutes(15) || age < Duration::minutes(-15) {
            return Err(Error::RequestExpired);
        }
        
        Ok(AuthenticatedPrincipal {
            hrn: credentials.principal_hrn,
            access_key_id: credentials.access_key_id,
            is_temporary: access_key_id.starts_with("ASIA"),
        })
    }
}
```

#### Interceptor gRPC/tonic

```rust
use tonic::service::Interceptor;

#[derive(Clone)]
pub struct AuthInterceptor {
    authenticator: Arc<RequestAuthenticator>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        // Extraer metadata
        let metadata = req.metadata();
        
        // Convertir a HashMap
        let mut headers = HashMap::new();
        for key in metadata.keys() {
            if let Some(value) = metadata.get(key) {
                headers.insert(
                    key.as_str().to_string(),
                    value.to_str().unwrap_or("").to_string()
                );
            }
        }
        
        // Autenticar (síncronamente - usa block_on si es necesario)
        let principal = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                self.authenticator.authenticate_request(
                    &req.method().to_string(),
                    req.uri().path(),
                    req.uri().query().unwrap_or(""),
                    &headers,
                    &[], // Payload se maneja después
                )
            )
        }).map_err(|e| Status::unauthenticated(e.to_string()))?;
        
        // Inyectar principal en extensiones del request
        req.extensions_mut().insert(principal);
        
        Ok(req)
    }
}
```

---

### 9.2. Usar AWS STS Directamente (No Recomendado para este caso)

**Pros:**
- ✅ Implementación battle-tested
- ✅ Integración nativa con otros servicios AWS
- ✅ Cero mantenimiento

**Cons:**
- ❌ Vendor lock-in completo
- ❌ Requiere que toda la infraestructura esté en AWS
- ❌ Costos por API calls
- ❌ No es open-source
- ❌ No se integra con Cedar directamente

### 9.2. Keycloak con Extensiones (Alternativa Viable)

**Keycloak** es un sistema de gestión de identidad open-source que incluye funcionalidad similar a STS.

```toml
# Integración con Keycloak
[dependencies]
keycloak = "24.0"
oauth2 = "4.4"
```

**Pros:**
- ✅ Open-source y mature
- ✅ Soporta OIDC, OAuth2, SAML
- ✅ UI de administración completa
- ✅ Federación de identidades built-in
- ✅ Alta disponibilidad y clustering

**Cons:**
- ❌ Más pesado (Java, requires JVM)
- ❌ Curva de aprendizaje
- ❌ Requiere adaptador personalizado para integrarse con Cedar
- ❌ No soporta modelo IAM de AWS nativamente

**Recomendación:** Keycloak es excelente para **autenticación**, pero para el STS con modelo IAM completo, mejor implementación custom.

### 9.3. Vault de HashiCorp (Complementario)

**HashiCorp Vault** puede ser usado para:
- Almacenamiento seguro de claves de firma
- Generación de credenciales dinámicas
- Auditoría de accesos

```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

pub async fn get_signing_key_from_vault() -> Result<String> {
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address("https://vault.example.com")
            .token("token")
            .build()?,
    )?;
    
    let secret: String = vaultrs::kv2::read(&client, "secret", "sts-signing-key").await?;
    Ok(secret)
}
```

---

## 10. Roadmap de Implementación (Actualizado tras Análisis AWS)

> **NOTA IMPORTANTE:** Tras el análisis profundo de AWS Verified Permissions + IAM + Cognito, se han identificado gaps críticos que requieren una fase adicional (Fase 0). Ver [ANALISIS_AWS_INTEGRATION.md](./ANALISIS_AWS_INTEGRATION.md) para detalles completos.

### Fase 0: Fundamentos de Autenticación (NUEVA - 1 semana) ⚠️ CRÍTICO

**Esta fase es bloqueante para todo el sistema IAM/STS.**

- [ ] **HRN (Hodei Resource Name) System**
  - [ ] Parser y validator de HRNs
  - [ ] Estructura: `hrn:hodei:service:region:account:resource-type/resource-id`
  - [ ] Tests de parsing y validación

- [ ] **Credential Management**
  - [ ] Estructura de AccessKey (AccessKeyId, SecretAccessKey, Status)
  - [ ] Generación de AccessKeyId (AKIA... permanente, ASIA... temporal)
  - [ ] Generación segura de SecretAccessKey (32 bytes random)
  - [ ] Credential Store (DB para persistencia)

- [ ] **Request Signing Implementation (HODEI-HMAC-SHA256)**
  - [ ] Algoritmo inspirado en AWS Signature V4
  - [ ] RequestSigner client-side (para CLI/SDK)
  - [ ] RequestAuthenticator server-side (para validación)
  - [ ] Constant-time signature comparison (prevención timing attacks)
  - [ ] Timestamp validation (prevención replay attacks)

- [ ] **gRPC/HTTP Interceptors**
  - [ ] AuthInterceptor para tonic
  - [ ] Extracción de metadata (authorization, x-hodei-date, etc.)
  - [ ] Inyección de AuthenticatedPrincipal en request extensions

**Entregables:**
- Librería `hodei-auth` con signing y validation
- CLI tool para generar y probar credenciales
- Tests end-to-end de autenticación
- Documentación de protocolo de autenticación

### Fase 1: STS MVP (2-3 semanas)
- [ ] Implementar estructura básica de STS API (gRPC + REST endpoints)
- [ ] Generación y validación de JWT SessionToken
- [ ] Integración con Redis para sesiones (TTL automático)
- [ ] AssumeRole básico sin TrustPolicy compleja
- [ ] Gestión de expiración de sesiones
- [ ] Integración con RequestAuthenticator de Fase 0

### Fase 2: IAM Core (NUEVA - 2 semanas)

- [ ] **Gestión de Identidades**
  - [ ] CRUD de IAM Users, Groups, Roles
  - [ ] Gestión de membresía (AddUserToGroup, RemoveUserFromGroup)

- [ ] **Gestión de Credenciales**
  - [ ] CreateAccessKey, DeleteAccessKey, ListAccessKeys
  - [ ] UpdateAccessKey (cambiar estado: Active/Inactive)

- [ ] **Políticas IAM en Cedar**
  - [ ] Parser de políticas Cedar para IAM
  - [ ] AttachUserPolicy, AttachGroupPolicy, AttachRolePolicy
  - [ ] PutInlinePolicy, DeleteInlinePolicy

### Fase 3: Integración IAM + STS (2 semanas)
- [ ] Evaluación de TrustPolicy con Cedar
- [ ] Soporte para ExternalId (anti confused-deputy)
- [ ] Session policies (reducción de permisos)
- [ ] Motor de evaluación IAM completo
- [ ] Interceptor IAM para control plane

### Fase 4: Producción (2-3 semanas)
- [ ] Sistema de revocación con blacklist
- [ ] Rotación de claves de firma
- [ ] Rate limiting por principal
- [ ] Audit logging estructurado
- [ ] Métricas y observabilidad
- [ ] Tests de carga (objetivo: 5K AssumeRole/s, 50K validations/s)

### Fase 5: Features Avanzadas (opcional - 3-4 semanas)
- [ ] Permission Boundaries
- [ ] Service Control Policies
- [ ] Policy Simulator
- [ ] AssumeRoleWithWebIdentity (federación OIDC)
- [ ] AssumeRoleWithSAML (federación SAML)
- [ ] MFA para AssumeRole
- [ ] Cross-account access

**Tiempo Total:** 10-13 semanas (vs 6-8 original)

**Razón del aumento:** Request authentication (Fase 0) e IAM Core (Fase 2) son críticos y no estaban en estimación original.

---

## 11. Dependencias Completas Recomendadas

```toml
[dependencies]
# Core async runtime
tokio = { version = "1.38", features = ["full"] }

# gRPC/Web framework
tonic = "0.12"
axum = "0.7"

# JWT y crypto
jsonwebtoken = "9.3"
ring = "0.17"
rand = "0.8"
base64 = "0.22"

# Redis y caché
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
deadpool-redis = "0.15"
moka = { version = "0.12", features = ["future"] }

# Cedar policy
cedar-policy = "4.0"

# Tiempo
chrono = { version = "0.4", features = ["serde"] }

# Serialización
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# UUID para session IDs
uuid = { version = "1.10", features = ["v4", "serde"] }

# Rate limiting
governor = "0.6"

# Métricas
prometheus = "0.13"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Vault (opcional)
vaultrs = "0.7"
```

---

## Conclusión

### Actualización tras Análisis AWS (2025-11-03)

Tras realizar un análisis exhaustivo de la integración AWS Verified Permissions + IAM + Cognito (ver [ANALISIS_AWS_INTEGRATION.md](./ANALISIS_AWS_INTEGRATION.md)), se confirmó que:

**✅ La especificación Hodei-IAM + STS está bien alineada con AWS** con las siguientes consideraciones:

#### Alineación Confirmada (95%+):
- ✅ **Separación de planos** correcta (Control Plane IAM vs Data Plane AVP)
- ✅ **Modelo de identidades** correcto (Users, Groups, Roles)
- ✅ **STS con AssumeRole** correctamente especificado (JWT, Redis, TrustPolicy)
- ✅ **Integración OIDC** correcta (IsAuthorizedWithToken, Identity Sources)
- ✅ **HRN system** bien diseñado (equivalente a ARN de AWS)

#### Diferencias Intencionales (Mejoras):
- ⚠️ **Políticas IAM en Cedar** (en lugar de JSON AWS)
  - **Justificación:** Consistencia total con AVP, mayor poder expresivo, validación formal
  - **Trade-off:** No compatible 1:1 con AWS, pero superior técnicamente
  
- ⚠️ **SessionToken como JWT** (en lugar de token opaco AWS)
  - **Justificación:** Stateless, distribuible, estándar de industria
  - **Trade-off:** Revocación más compleja (mitigado con blacklist + caché)

#### Gaps Críticos Identificados:
1. ❌ **Request Authentication** - No estaba especificado (ahora agregado en sección 9.1)
2. ⚠️ **Permission Boundaries** - No estaba especificado (agregado a Fase 5)
3. ⚠️ **Audit Logging** - Parcialmente especificado (mejorado en Fase 4)

### Viabilidad Técnica

La implementación de un STS+IAM custom en Rust es **completamente viable y recomendada** para Hodei. Las librerías del ecosistema Rust proporcionan todas las primitivas necesarias:

#### Ventajas clave:
- ✅ **Alto rendimiento:** Sub-10ms AssumeRole, Sub-1ms validación (con caché)
- ✅ **Type safety:** Rust previene errores comunes de seguridad (memory safety, thread safety)
- ✅ **Zero-cost abstractions:** Performance comparable a C/C++ sin sacrificar ergonomía
- ✅ **Ecosistema maduro:** Librerías battle-tested (jsonwebtoken, ring, redis, cedar-policy)
- ✅ **Control total:** Personalización completa del comportamiento
- ✅ **No vendor lock-in:** Implementación open-source y portable
- ✅ **Cedar everywhere:** Consistencia total en motor de políticas

#### Stack Tecnológico Completo:

**Core:**
- `tokio` - async runtime
- `tonic` - gRPC framework
- `cedar-policy` - motor de políticas

**Seguridad:**
- `jsonwebtoken` - JWT signing/validation (100K tokens/sec)
- `ring` - criptografía (BoringSSL)
- `hmac` + `sha2` - request signing
- `rand` - generación segura de credenciales

**Persistencia:**
- `redis` + `deadpool-redis` - sesiones y caché
- `moka` - caché en memoria
- `sqlx` o `diesel` - credential store

**Observabilidad:**
- `tracing` + `tracing-subscriber` - logging estructurado
- `prometheus` - métricas
- `governor` - rate limiting

### Estimación de Costos

**Tiempo de desarrollo:** 10-13 semanas (vs 6-8 original)
- Fase 0 (crítica): 1 semana
- Fases 1-4 (core): 8-10 semanas  
- Fase 5 (opcional): 3-4 semanas

**ROI vs Alternativas:**

| Opción | Costo Inicial | Costo Mensual | Lock-in | Customización |
|--------|--------------|---------------|---------|---------------|
| **Hodei (custom)** | 10-13 semanas dev | $0 (self-hosted) | ❌ No | ✅ Total |
| **AWS IAM + AVP** | 0 (managed) | ~$500-5000+ | ✅ Alto | ❌ Limitada |
| **Keycloak + custom** | 4-6 semanas | ~$100-500 | ⚠️ Medio | ⚠️ Media |

**Conclusión:** El costo de desarrollo (10-13 semanas) se amortiza en 3-6 meses vs AWS, con las ventajas de:
- Control total del stack
- Sin límites arbitrarios
- Optimización para casos de uso específicos
- Open-source (contribución a la comunidad)

### Próximos Pasos Inmediatos

1. ✅ **Validar análisis AWS** con stakeholders
2. ⏭️ **Aprobar roadmap actualizado** (10-13 semanas)
3. ⏭️ **Iniciar Fase 0** (request authentication)
4. ⏭️ **Configurar infrastructure** (Redis, Vault opcional, DB)
5. ⏭️ **Setup CI/CD** para hodei-iam crate

### Referencias Completas

- [AWS Verified Permissions Documentation](https://docs.aws.amazon.com/verified-permissions/)
- [AWS IAM Documentation](https://docs.aws.amazon.com/IAM/)
- [AWS STS Documentation](https://docs.aws.amazon.com/STS/)
- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [AWS Signature Version 4](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html)
- [RFC 7519 - JWT](https://datatracker.ietf.org/doc/html/rfc7519)

### Documento de Análisis Completo

Para el análisis exhaustivo de la integración AWS y comparación con Hodei, consultar:

**📄 [ANALISIS_AWS_INTEGRATION.md](./ANALISIS_AWS_INTEGRATION.md)**

Incluye:
- Arquitectura detallada de AWS Verified Permissions + IAM + Cognito
- Flujos completos con diagramas Mermaid
- Comparación punto por punto con Hodei
- Gaps identificados con soluciones propuestas
- Decisiones de diseño justificadas
- Roadmap actualizado con priorización

