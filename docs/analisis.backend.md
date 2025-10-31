# Análisis Completo del Backend - Hodei Verified Permissions Service

## 1. APIs gRPC IMPLEMENTADAS

### 1.1 AuthorizationData Service (Data Plane)
**Puerto 50051 - Para evaluaciones de autorización de alto rendimiento**

#### Métodos Disponibles:

1. **IsAuthorized**
    - **Parámetros:**
        - `policy_store_id`: string (ID del policy store)
        - `principal`: EntityIdentifier (ej: "User::\"alice\"")
        - `action`: EntityIdentifier (ej: "Action::\"viewDocument\"")
        - `resource`: EntityIdentifier (ej: "Document::\"doc123\"")
        - `context`: optional string (JSON con contexto adicional como IP, tiempo)
        - `entities`: repeated Entity (datos de entidades con atributos y jerarquías)
    - **Respuesta:**
        - `decision`: enum (ALLOW/DENY)
        - `determining_policies`: repeated string (IDs de políticas que determinaron la decisión)
        - `errors`: repeated string (errores durante evaluación)

2. **BatchIsAuthorized**
    - **Parámetros:**
        - `policy_store_id`: string
        - `requests`: repeated IsAuthorizedRequest
    - **Respuesta:**
        - `responses`: repeated IsAuthorizedResponse
    - **Capacidad**: Procesa múltiples evaluaciones en una sola llamada

3. **IsAuthorizedWithToken**
    - **Parámetros:**
        - `policy_store_id`: string
        - `identity_source_id`: string
        - `access_token`: string (JWT token)
        - `action`: EntityIdentifier
        - `resource`: EntityIdentifier
        - `context`: optional string
        - `entities`: repeated Entity
    - **Capacidad**: Valida JWT y extrae claims (sub, email, grupos) para crear entidades automáticamente

### 1.2 AuthorizationControl Service (Control Plane)
**Puerto 50051 - Para gestión de políticas y esquemas**

#### Gestión de Policy Stores:

1. **CreatePolicyStore**
    - Crea un nuevo policy store
    - Parámetros: `description`: optional string
    - Respuesta: `policy_store_id`, `created_at`

2. **GetPolicyStore**
    - Obtiene detalles de un policy store
    - Parámetros: `policy_store_id`: string
    - Respuesta: PolicyStore con description, timestamps

3. **ListPolicyStores**
    - Lista todos los policy stores con paginación
    - Parámetros: `max_results`: optional int32, `next_token`: optional string
    - Respuesta: `policy_stores`: repeated PolicyStoreItem, `next_token`: optional string

4. **DeletePolicyStore**
    - Elimina un policy store (con cascada)

#### Gestión de Esquemas Cedar:

5. **PutSchema**
    - Guarda o actualiza esquema Cedar para un policy store
    - Parámetros: `policy_store_id`: string, `schema`: string (JSON)
    - Valida formato de esquema
    - Respuesta: `policy_store_id`, `namespaces`: repeated string

6. **GetSchema**
    - Obtiene esquema Cedar de un policy store
    - Parámetros: `policy_store_id`: string
    - Respuesta: `policy_store_id`, `schema`, `created_at`, `updated_at`

#### Gestión de Políticas:

7. **CreatePolicy**
    - Crea nueva política estática o desde template
    - Parámetros: `policy_store_id`, `policy_id`, `definition` (StaticPolicy o TemplateLinkedPolicy), `description`:
      optional string
    - Valida sintaxis Cedar
    - Soporta placeholders `?principal` y `?resource` desde templates

8. **GetPolicy**
    - Obtiene política por ID

9. **UpdatePolicy**
    - Actualiza política existente

10. **DeletePolicy**
    - Elimina política

11. **ListPolicies**
    - Lista políticas de un policy store con paginación
    - Parámetros: `policy_store_id`, `max_results`, `next_token`
    - Respuesta: `policies`: repeated PolicyItem, `next_token`

#### Gestión de Identity Sources (Épica 4):

12. **CreateIdentitySource**
    - Crea fuente de identidad (OIDC o Cognito)
    - Soporta configuraciones:
        - **CognitoUserPoolConfiguration**: user_pool_arn, client_ids, group_configuration_group_claim
        - **OidcConfiguration**: issuer, client_ids, jwks_uri, group_claim
    - Claims mapping configurable: principal_id_claim, group_claim, attribute_mappings
    - Respuesta: `identity_source_id`, `created_at`

13. **GetIdentitySource**
    - Obtiene configuración de fuente de identidad

14. **ListIdentitySources**
    - Lista fuentes de identidad de un policy store

15. **DeleteIdentitySource**
    - Elimina fuente de identidad

#### Gestión de Policy Templates (Épica 6):

16. **CreatePolicyTemplate**
    - Crea template reutilizable con placeholders `?principal` y `?resource`
    - Parámetros: `policy_store_id`, `template_id`, `statement` (Cedar con placeholders), `description`: optional
      string
    - Respuesta: `template_id`, `created_at`

17. **GetPolicyTemplate**
    - Obtiene template

18. **ListPolicyTemplates**
    - Lista templates

19. **DeletePolicyTemplate**
    - Elimina template

#### Playground/Testing Endpoints:

20. **TestAuthorization**
    - Prueba autorización sin persistir políticas
    - Utilizable con o sin policy_store_id
    - Valida políticas contra esquema
    - Retorna: decision, determining_policies, errors, validation_warnings, validation_errors

21. **ValidatePolicy**
    - Valida sintaxis y semántica de política contra esquema
    - Parámetros: `policy_store_id` o `schema`, `policy_statement`
    - Respuesta: `is_valid`, `errors`, `warnings`, `policy_info` (effect, principal_scope, action_scope,
      resource_scope, has_conditions)

### 1.3 Esquemas de Request/Response Completos

#### Tipos de Datos Principales:

**EntityIdentifier:**
- `entity_type`: string (ej: "User", "Action", "Resource")
- `entity_id`: string (ej: "alice", "viewDocument")

**Entity:**
- `identifier`: EntityIdentifier
- `attributes`: map<string, string> (JSON-encoded attribute values)
- `parents`: repeated EntityIdentifier (jerarquía)

**PolicyDefinition (oneof):**
- `StaticPolicy`: statement (texto Cedar)
- `TemplateLinkedPolicy`: policy_template_id, principal, resource

**IdentitySourceConfiguration (oneof):**
- `CognitoUserPoolConfiguration`
- `OidcConfiguration`

**ClaimsMappingConfiguration:**
- `principal_id_claim`: string (default: "sub")
- `group_claim`: string (claim para grupos)
- `attribute_mappings`: map<string, string> (claims → Cedar attributes)

**ValidationIssue:**
- `severity`: enum (ERROR/WARNING/INFO)
- `message`: string
- `location`: optional string
- `issue_type`: string

### 1.4 Tipos de Datos Cedar Soportados

El backend utiliza Cedar Policy Engine v4.7.0 y soporta:

**Tipos de Entidades:**
- User (principales/identidades)
- Action (acciones)
- Resource (recursos)
- Role (grupos jerárquicos)
- Cualquier tipo definido en el esquema JSON

**Tipos de Atributos:**
- String ({"attr": "value"})
- Number ({"attr": 123})
- Boolean ({"attr": true})
- Array ({"attr": ["a", "b", "c"]})
- Object ({"attr": {"nested": "value"}})

**Políticas Cedar:**
- **permit/forbid**: Efecto de la política
- **principal**: Tipo y alcance del principal
- **action**: Tipo y alcance de la acción
- **resource**: Tipo y alcance del recurso
- **when/unless**: Condiciones
- **variables**: ?principal, ?resource, ?action

**Validación de Esquemas:**
- Validación sintáctica de políticas
- Validación semántica contra esquema JSON
- Verificación de tipos de entidades y atributos

   ---

## 2. ARQUITECTURA DEL BACKEND

### 2.1 Estructura Hexagonal (Puertos y Adaptadores)

   ```
   ┌─────────────────────────────────────────────────────────┐
   │                     FRONTEND (gRPC)                     │
   ├─────────────────────────────────────────────────────────┤
   │  API Layer (api/)                                       │
   │  - control_plane.rs  (AuthorizationControl gRPC)       │
   │  - data_plane.rs     (AuthorizationData gRPC)          │
   │  - CLI Interface                                        │
   └─────────────────────────────────────────────────────────┘
                               │
                               ▼
   ┌─────────────────────────────────────────────────────────┐
   │  Application Layer (application/)                       │
   │  - use_cases/        (Casos de uso coordinados)        │
   │  - services.rs       (Servicios de aplicación)         │
   │  - dto.rs           (Data Transfer Objects)            │
   └─────────────────────────────────────────────────────────┘
                               │
                               ▼
   ┌─────────────────────────────────────────────────────────┐
   │  Domain Layer (domain/)                                 │
   │  - entities/        (Entidades de dominio)             │
   │    * PolicyStore                                         │
   │    * Policy                                             │
   │    * Schema                                             │
   │    * IdentitySource                                     │
   │    * PolicyTemplate                                     │
   │  - value_objects/    (Value objects tipados)           │
   │    * PolicyStoreId                                      │
   │    * PolicyId                                           │
   │    * CedarPolicy                                        │
   │    * IdentitySourceType                                 │
   │    * AuthorizationDecision                              │
   │  - repository/      (Traits de repositorio)            │
   │    * PolicyRepository (trait principal)                 │
   │  - services.rs      (Servicios de dominio)             │
   │  - errors.rs        (Errores de dominio)               │
   └─────────────────────────────────────────────────────────┘
                               │
                               ▼
   ┌─────────────────────────────────────────────────────────┐
   │  Infrastructure Layer (infrastructure/)                 │
   │  - repository/      (Adaptadores de persistencia)      │
   │    * adapter.rs     (Adapter principal)                 │
   │    * sqlite_repository.rs (SQLite implementación)      │
   │    * postgres_repository.rs (PostgreSQL implementación)│
   │    * surreal_repository.rs (SurrealDB implementación)  │
   │  - jwt/             (Validación JWT)                   │
   │    * validator.rs   (Validador principal)               │
   │    * jwks_cache.rs  (Cache de claves públicas)         │
   │    * claims_mapper.rs (Mapeo de claims)                │
   │    * issuer_detection.rs                                │
   │  - cache/           (Capa de cache)                    │
   │    * policy_store_cache.rs                              │
   │    * cache_manager.rs                                   │
   │    * reload_task.rs                                     │
   │  - config.rs        (Configuración)                    │
   └─────────────────────────────────────────────────────────┘
   ```

### 2.2 Componentes Principales

#### Puertos (Interfaces):

**PolicyRepository (Dominio):**
   ```rust
   trait PolicyRepository: Send + Sync {
       // Policy Store Operations
       async fn create_policy_store(&self, description: Option<String>) -> DomainResult<PolicyStore>;
       async fn get_policy_store(&self, id: &PolicyStoreId) -> DomainResult<PolicyStore>;
       async fn list_policy_stores(&self) -> DomainResult<Vec<PolicyStore>>;
       async fn delete_policy_store(&self, id: &PolicyStoreId) -> DomainResult<()>;

       // Schema Operations
       async fn put_schema(&self, policy_store_id: &PolicyStoreId, schema: String) -> DomainResult<()>;
       async fn get_schema(&self, policy_store_id: &PolicyStoreId) -> DomainResult<Option<Schema>>;

       // Policy Operations
       async fn create_policy(...) -> DomainResult<Policy>;
       async fn get_policy(...) -> DomainResult<Policy>;
       async fn list_policies(...) -> DomainResult<Vec<Policy>>;
       async fn update_policy(...) -> DomainResult<Policy>;
       async fn delete_policy(...) -> DomainResult<()>;

       // Identity Source Operations
       async fn create_identity_source(...) -> DomainResult<IdentitySource>;
       async fn get_identity_source(...) -> DomainResult<IdentitySource>;
       async fn list_identity_sources(...) -> DomainResult<Vec<IdentitySource>>;
       async fn delete_identity_source(...) -> DomainResult<()>;

       // Policy Template Operations
       async fn create_policy_template(...) -> DomainResult<PolicyTemplate>;
       async fn get_policy_template(...) -> DomainResult<PolicyTemplate>;
       async fn list_policy_templates(...) -> DomainResult<Vec<PolicyTemplate>>;
       async fn delete_policy_template(...) -> DomainResult<()>;

       // Audit Operations
       async fn log_authorization(&self, log: AuthorizationLog) -> DomainResult<()>;
   }
   ```

#### Adaptadores (Implementaciones):

**RepositoryAdapter:**
- Puente entre trait de dominio y implementación SQLite
- Mapea modelos de base de datos a entidades de dominio
- Maneja conversión de tipos y validaciones

**JwtValidator:**
- Valida tokens JWT con firmas digitales
- Cache de claves públicas (JWKS) en memoria con RwLock
- Extracción de claims (sub, iss, aud, exp, iat)
- Soporte para múltiples issuers
- Configuración de audiences por identity source

**JwksCache:**
- Cache en memoria de claves públicas RSA
- Actualización automática de claves
- Persistencia durante la vida del proceso
- Thread-safe con RwLock

**ClaimsMapper:**
- Mapea claims JWT a atributos Cedar
- Configuración flexible de attribute_mappings
- Soporte para grupos como parent entities
- Soporte para roles jerárquicos

### 2.3 Manejo de Policy Stores

**Gestión Completa:**
- Creación de policy stores independientes
- Aislamiento de datos por policy store
- Múltiples identity sources por store
- Identity source por defecto configurable
- Eliminación en cascada (store → policies, schemas, templates, sources)

**Estado Interno:**
   ```rust
   struct PolicyStore {
       id: PolicyStoreId,
       description: Option<String>,
       identity_source_ids: Vec<String>,
       default_identity_source_id: Option<String>,
       created_at: DateTime<Utc>,
       updated_at: DateTime<Utc>,
   }
   ```

### 2.4 Gestión de Esquemas y Entidades

**Esquemas Cedar:**
- Formato JSON para definir tipos de entidades
- Validación sintáctica con Schema::from_str
- Versionado con timestamps
- Múltiples namespaces por store

**Entidades:**
- Soporte para jerarquías (parents)
- Atributos flexibles JSON-encoded
- Herencia de atributos vía parents
- EntityUIDs en formato "Type::\"id\""

### 2.5 Sistema de Políticas y Templates

**Políticas Estáticas:**
- Texto Cedar puro
- Validación sintáctica con CedarPolicy::from_str
- ID único por store
- Versionado con timestamps

**Policy Templates:**
- Placeholders ?principal y ?resource
- Instantiation automática con valores específicos
- Soporte para políticas linkadas a templates
- Validación de reemplazo completo de placeholders

**Validación Avanzada:**
- Verificación contra esquema JSON
- Validación semántica de tipos
- Warnings y errors categorizados
- Validación en tiempo de creación y testing

### 2.6 Identity Sources y JWT Validation

**Tipos Soportados:**
- **AWS Cognito User Pools**
    - Configuración: user_pool_arn, client_ids, group_claim
    - Extracción de grupos automática
    - Mapeo configurable de attributes

- **OIDC Genérico**
    - Configuración: issuer, client_ids[], jwks_uri, group_claim
    - Soporte multi-tenant
    - Cache de JWKS por kid
    - Validación de firma, issuer, audience, expiración

**Flujo de Validación JWT:**
1. Decode header para obtener kid
2. Buscar clave en cache (RwLock<HashMap<String, DecodingKey>>)
3. Si no está, fetch desde jwks_uri
4. Parsear JWK para extraer n y e
5. Crear DecodingKey desde RSA components
6. Cachear clave
7. Validar token (signature, issuer, audience, exp)
8. Extraer claims (sub, iss, aud, exp, iat, + custom)
9. Mapear claims a entidades Cedar
10. Crear entities con atributos y parents (grupos)

**Claims Mapping:**
- principal_id_claim: default "sub"
- group_claim: configurable (ej: "cognito:groups", "groups", "roles")
- attribute_mappings: HashMap<claim_name, cedar_attribute_name>
- Transformaciones automáticas (string, number, boolean, array, object)

### 2.7 Motor Cedar Policy Engine

**Versión:** Cedar Policy v4.7.0

**Capacidades:**
- Evaluación is_authorized con Authorizer
- Soporte completo para permit/forbid
- Context JSON para condiciones dinámicas
- Entities con atributos y jerarquías
- Validación con Validator
- Diagnostics con reason() y errors()

**Performance:**
- PolicySet en memoria para evaluación rápida
- Cache de PolicySets planificado
- Batch authorization para múltiples requests
- Reload task para sincronización automática

   ---

## 3. CAPACIDADES DEL SERVIDOR

### 3.1 Endpoints Disponibles

**Puerto:** 0.0.0.0:50051 (configurable vía código)

**Servicios gRPC:**
- **AuthorizationDataServer**: Maneja AuthorizationDataService
- **AuthorizationControlServer**: Maneja AuthorizationControlService

**Modo de Operación:**
- Servidor TCP síncrono con tokio runtime
- Multi-threaded async runtime
- Shutdown graceful en Ctrl+C

### 3.2 Validaciones Implementadas

**Validación de Entrada:**
1. **IDs de Policy Store y Policy**
    - No vacíos
    - Formato válido

2. **Sintaxis Cedar**
    - CedarPolicy::from_str() validation
    - Verificación de placeholders en templates

3. **Esquemas JSON**
    - Schema::from_str() validation
    - Estructura válida según especificación Cedar

4. **JWT Tokens**
    - Header con kid presente
    - Firma válida con clave pública
    - Issuer match
    - Audience match
    - Expiration time (exp)
    - Not before (iat)

5. **Entity Identifiers**
    - Formato "Type::\"id\""
    - Entity type no vacío
    - Entity ID no vacío

6. **Context JSON**
    - Parseable como JSON
    - Compatible con Context::from_json_value

**Validación Semántica:**
- Políticas contra esquema JSON (ValidatePolicy, TestAuthorization)
- Tipos de entidades correctos
- Atributos con tipos correctos
- Parent relationships válidos
- Principal/Action/Resource scopes válidos

### 3.3 Manejo de Errores

**Jerarquía de Errores:**

1. **DomainError (Dominio)**
    - InvalidPolicyStoreId
    - InvalidPolicyId
    - InvalidEntityIdentifier
    - InvalidPolicySyntax
    - InvalidSchemaFormat
    - PolicyValidationFailed
    - PolicyStoreNotFound
    - PolicyNotFound
    - SchemaNotFound
    - AuthorizationEvaluationFailed
    - BusinessRuleViolation
    - Internal

2. **AuthorizationError (Infraestructura)**
    - Internal (para errores de JWT, JWKS, HTTP)
    - InvalidToken
    - AuthenticationFailed

3. **Status (gRPC)**
    - OK
    - INVALID_ARGUMENT (validaciones fallidas)
    - NOT_FOUND (recursos no encontrados)
    - INTERNAL (errores del servidor)
    - UNAUTHENTICATED (tokens inválidos)

**Mapeo de Errores:**
   ```rust
   // Ejemplo del Data Plane
   .map_err(|e| Status::invalid_argument(format!("Invalid entity identifier: {}", e)))

   // Ejemplo del Repository
   .map_err(|e| Status::internal(format!("Failed to load policies: {}", e)))

   // Ejemplo de JWT
   .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))
   ```

**Logging:**
- Tracing con niveles (INFO, WARN, ERROR)
- Structured logging con policy_store_id, principal, action, resource
- No logging de tokens JWT completos (seguridad)

### 3.4 Performance y Optimizaciones

**JWT Validation:**
- Cache de claves públicas en memoria (HashMap + RwLock)
- Evita fetch repetido de JWKS
- TTL configurable implícito (vida del proceso)
- Actualización automática si clave no encontrada

**Policy Evaluation:**
- PolicySet construido dinámicamente por request
- Cache de PolicySets planificado (PolicyStoreCache)
- Batch authorization para múltiples requests
- Evaluación paralela posible

**Database:**
- Conexión pool con SQLx
- Soporte para SQLite (desarrollo)
- Soporte para PostgreSQL (producción)
- Soporte para SurrealDB (alternativa NoSQL)

**Async Runtime:**
- Tokio con multi-thread
- Non-blocking I/O para JWKS fetches
- Concurrent request handling

**Memory:**
- Arc<RepositoryAdapter> compartido entre servicios
- RwLock para cache thread-safe
- HashMap para caching de claves

### 3.5 Capacidades de Auditoría

**AuthorizationLog:**
   ```rust
   struct AuthorizationLog {
       policy_store_id: PolicyStoreId,
       principal: Principal,
       action: Action,
       resource: Resource,
       decision: AuthorizationDecision,
       timestamp: DateTime<Utc>,
   }
   ```

**Logging Implementado:**
- Log de cada autorización (planned)
- Almacenamiento en base de datos
- Trazabilidad completa
- Metadata para compliance

### 3.6 Configuración y Dependencias

**Configuración Base:**
- DATABASE_URL (default: sqlite:///app/data/hodei.db)
- Puerto: 50051 (hardcoded en main.rs)
- Variables de entorno para JWT y otros secrets

**Dependencias Principales:**
- **cedar-policy**: v4.7.0 (motor de evaluación)
- **tonic**: v0.14.2 (gRPC server/client)
- **tokio**: v1.40 (async runtime)
- **sqlx**: v0.8 (database con async/await)
- **jsonwebtoken**: v9 (JWT validation)
- **reqwest**: v0.12 (HTTP client para JWKS)
- **serde**: v1.0 (serialization)
- **tracing**: v0.1 (structured logging)
- **chrono**: v0.4 (timestamps)

**Features Opcionales:**
- SurrealDB para NoSQL
- SQLite para desarrollo local
- PostgreSQL para producción

   ---

## 4. PROTOCOL BUFFERS - ESTRUCTURA COMPLETA

### 4.1 Archivo authorization.proto

**Ubicación:** `/home/rubentxu/Proyectos/rust/hodei-verified-permissions/proto/authorization.proto`

**Tamaño:** 16,910 bytes (613 líneas)

### 4.2 Servicios Definidos

   ```protobuf
   service AuthorizationData {
     rpc IsAuthorized(IsAuthorizedRequest) returns (IsAuthorizedResponse);
     rpc BatchIsAuthorized(BatchIsAuthorizedRequest) returns (BatchIsAuthorizedResponse);
     rpc IsAuthorizedWithToken(IsAuthorizedWithTokenRequest) returns (IsAuthorizedResponse);
   }

   service AuthorizationControl {
     // 21 métodos para gestión completa
     // Ver sección 1.2 para lista completa
   }
   ```

### 4.3 Mensajes y Tipos de Datos

**Enums:**

1. **Decision**
    - DECISION_UNSPECIFIED = 0
    - ALLOW = 1
    - DENY = 2

2. **Severity (ValidationIssue)**
    - SEVERITY_UNSPECIFIED = 0
    - ERROR = 1
    - WARNING = 2
    - INFO = 3

**Mensajes Principales:**

1. **EntityIdentifier**
    - entity_type: string
    - entity_id: string

2. **Entity**
    - identifier: EntityIdentifier
    - attributes: map<string, string>
    - parents: repeated EntityIdentifier

3. **PolicyDefinition (oneof)**
    - StaticPolicy static
    - TemplateLinkedPolicy template_linked

4. **StaticPolicy**
    - statement: string

5. **TemplateLinkedPolicy**
    - policy_template_id: string
    - principal: EntityIdentifier
    - resource: EntityIdentifier

6. **IdentitySourceConfiguration (oneof)**
    - CognitoUserPoolConfiguration cognito_user_pool
    - OidcConfiguration oidc

7. **CognitoUserPoolConfiguration**
    - user_pool_arn: string
    - client_ids: string (comma-separated list)
    - group_configuration_group_claim: string

8. **OidcConfiguration**
    - issuer: string
    - client_ids: repeated string
    - jwks_uri: string
    - group_claim: string

9. **ClaimsMappingConfiguration**
    - principal_id_claim: string
    - group_claim: string
    - attribute_mappings: map<string, string>

10. **ValidationIssue**
    - severity: enum
    - message: string
    - location: optional string
    - issue_type: string

11. **PolicyInfo**
    - effect: string
    - principal_scope: optional string
    - action_scope: optional string
    - resource_scope: optional string
    - has_conditions: bool

**Mensajes de Request/Response:**
- 21 métodos = 42 mensajes (21 requests + 21 responses)
- Ver sección 1.2 y 1.3 para detalles completos

### 4.4 Relaciones Entre Entidades

   ```
   PolicyStore (1) -----> (N) Policy
   PolicyStore (1) -----> (1) Schema
   PolicyStore (1) -----> (N) IdentitySource
   PolicyStore (1) -----> (N) PolicyTemplate
   PolicyStore (1) -----> (N) AuthorizationLog

   PolicyTemplate (1) ----> (N) Policy (via TemplateLinkedPolicy)

   IdentitySource (1) ----> (N) JWT Token Validation

   Entity (N) <-----> (N) Entity (parents relationship)
   ```

### 4.5 Tipos de Datos Soportados en Protocol Buffers

**Escalares:**
- string: UTF-8 encoded
- int32: signed 32-bit
- bool: boolean
- bytes: arbitrary bytes

**Compuestos:**
- map<key, value>: dictionaries
- repeated T: lists/arrays
- oneof: union types (discriminated unions)

**Timestamps:**
- Representados como string RFC3339 (ej: "2023-10-30T10:00:00Z")
- No uso de google.protobuf.Timestamp (más simple)

**JSON Integration:**
- context: string (JSON-encoded)
- attributes: map<string, string> (JSON-encoded values)
- schema: string (JSON format for Cedar schema)
- configuration_json: string (JSON for identity sources)
- claims_mapping_json: string (JSON for claims mapping)

**Validation:**
- is_valid: bool
- errors/warnings: repeated ValidationIssue
- validation_warnings: repeated ValidationIssue
- validation_errors: repeated ValidationIssue

   ---

## RESUMEN EJECUTIVO

El backend de Hodei Verified Permissions es un servicio gRPC completo y robusto que implementa:

### ✅ Capacidades Implementadas

1. **21 APIs gRPC** (3 Data Plane + 18 Control Plane)
2. **Arquitectura Hexagonal** con separación clara de responsabilidades
3. **Soporte completo para Cedar Policy Engine v4.7.0**
4. **Validación JWT** con OIDC y Cognito
5. **Identity Sources** configurables (OIDC/Cognito)
6. **Policy Templates** con placeholders
7. **Playground/Testing** endpoints
8. **Batch Authorization** para performance
9. **Cache de claves JWT** en memoria
10. **Múltiples bases de datos** (SQLite, PostgreSQL, SurrealDB)
11. **Validación de esquemas** y políticas
12. **Manejo de errores** jerárquico
13. **Logging y trazabilidad**
14. **Audit logging** planificado

### 🎯 Funcionalidades Clave

- **Evaluación de autorización** con/sin JWT
- **Gestión completa de policy stores**
- **Validación sintáctica y semántica** de políticas
- **Mapeo flexible de claims** JWT a entidades Cedar
- **Soporte para jerarquías** de entidades
- **Templates reutilizables** para políticas
- **Testing sin persistencia**

### 📊 Métricas y Límites

- **Puerto:** 50051
- **Persistencia:** SQLite (dev), PostgreSQL (prod), SurrealDB (alt)
- **JWT Cache:** In-memory con RwLock
- **Concurrent requests:** Multi-threaded tokio
- **Batch size:** Ilimitado (dependiente de memoria)
- **Policy Store:** Ilimitados por instancia
- **Policies por Store:** Ilimitados
- **Entities por Request:** Ilimitados

### 🔒 Seguridad

- Validación de firma JWT con JWKS
- Verificación de issuer y audience
- Validación de expiración
- Sanitización de logs (no tokens completos)
- Aislamiento por policy store
- Claims mapping configurable

### 🚀 Extensibilidad

- Arquitectura hexagonal fácil de extender
- Traits para nuevos adaptadores de repositorio
- Soporte multi-tenant nativo
- Configuración flexible de identity sources
- Cache layer modular
- Validadores extensibles