# 📚 Guía Completa del Desarrollador - Hodei Verified Permissions

**Versión:** 2.0  
**Última actualización:** 2025-11-10  
**Nivel:** Desarrollador Senior

---

## 📋 Índice

1. [Introducción y Arquitectura](#1-introducción-y-arquitectura)
2. [Instalación y Levantamiento con Docker Compose](#2-instalación-y-levantamiento-con-docker-compose)
3. [Integración con Keycloak](#3-integración-con-keycloak)
4. [SDKs y APIs](#4-sdks-y-apis)
5. [Conversión OpenAPI a Schema Cedar](#5-conversión-openapi-a-schema-cedar)
6. [Ejemplos Prácticos de Integración](#6-ejemplos-prácticos-de-integración)
7. [Testing y Debugging](#7-testing-y-debugging)
8. [Mejores Prácticas](#8-mejores-prácticas)
9. [Solución de Problemas](#9-solución-de-problemas)

---

## 1. Introducción y Arquitectura

### 1.1 ¿Qué es Hodei Verified Permissions?

Hodei Verified Permissions es un servicio de autorización de grado empresarial basado en el motor de políticas Cedar, que proporciona:

- **Control de acceso basado en atributos (ABAC)** con lenguaje Cedar
- **Separación de planos**: Data Plane (autorización) y Control Plane (gestión)
- **Arquitectura hexagonal** para máxima mantenibilidad
- **Tres componentes SDK**: Cliente ligero, Admin SDK y CLI
- **Integración con proveedores de identidad** (Keycloak, Zitadel, etc.)
- **Conversión automática OpenAPI → Schema Cedar**

### 1.2 Arquitectura de Componentes

```mermaid
graph TD
    subgraph "Capa de Aplicación"
        A1[App Web/API]
        A2[Microservicio]
        A3[CLI Tool]
    end

    subgraph "SDKs"
        B1[verified-permissions-sdk<br/>Data Plane]
        B2[verified-permissions-sdk-admin<br/>Control Plane]
        B3[hvp CLI]
    end

    subgraph "Backend gRPC"
        C1[AuthorizationDataService]
        C2[AuthorizationControlService]
        C3[Audit Service]
    end

    subgraph "Infraestructura"
        D1[Cedar Engine]
        D2[PostgreSQL/SQLite]
        D3[Keycloak/OIDC]
    end

    A1 --> B1
    A2 --> B1
    A3 --> B2
    B1 --> C1
    B2 --> C2
    C1 --> D1
    C2 --> D2
    C1 --> D3
```

### 1.3 Flujo de Autorización Completo

```mermaid
sequenceDiagram
    participant Client as Client App
    participant SDK as verified-permissions-sdk
    participant AVP as Hodei Server
    participant Keycloak as Keycloak
    participant Cedar as Cedar Engine
    
    Client->>SDK: is_authorized(principal, action, resource)
    SDK->>AVP: gRPC Request
    AVP->>Keycloak: Validate JWT + get claims
    Keycloak-->>AVP: User roles/groups
    AVP->>AVP: Load policies from DB
    AVP->>Cedar: Evaluate request
    Cedar-->>AVP: Decision (Allow/Deny)
    AVP-->>SDK: AuthorizationResponse
    SDK-->>Client: Result
```

---

## 2. Instalación y Levantamiento con Docker Compose

### 2.1 Requisitos Previos

```bash
# Docker y Docker Compose
docker --version          # >= 20.10.0
docker compose version    # >= 2.0.0

# Opcional: Rust toolchain para desarrollo local
rustc --version          # >= 1.70.0
```

### 2.2 Levantamiento Completo con Docker Compose

```bash
# 1. Clonar el repositorio
git clone https://github.com/rubentxu/hodei-verified-permissions.git
cd hodei-verified-permissions/examples

# 2. Configurar variables de entorno
cp .env.example .env

# 3. Levantar toda la infraestructura
make up

# 4. Bootstrap de Keycloak (en terminal separada)
make keycloak

# 5. Seed de datos en AVP
make seed

# 6. Verificar que todo funciona
make demo-run
```

### 2.3 Servicios Disponibles

| Servicio | URL | Descripción |
|----------|-----|-------------|
| **Keycloak** | http://localhost:8080 | Servidor de autenticación |
| **Hodei AVP** | localhost:50051 | gRPC API |
| **Demo App** | http://localhost:3000 | Aplicación de ejemplo |
| **PostgreSQL** | localhost:5432 | Base de datos |

### 2.4 Estructura del docker-compose.yml

```yaml
# examples/docker-compose.yml
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: avp
      POSTGRES_PASSWORD: avp-password
      POSTGRES_DB: avp
    ports:
      - "5432:5432"
    volumes:
      - postgres-data:/var/lib/postgresql/data

  keycloak:
    image: quay.io/keycloak/keycloak:24.0.5
    command: start-dev
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      KEYCLOAK_ADMIN: admin
      KEYCLOAK_ADMIN_PASSWORD: admin
      KC_DB: postgres
      KC_DB_URL_HOST: postgres
    ports:
      - "8080:8080"

  avp-server:
    build:
      context: ..
      dockerfile: Dockerfile.test
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      DATABASE_PROVIDER: postgres
      DATABASE_URL: postgresql://avp:avp-password@postgres:5432/avp
    ports:
      - "50051:50051"

  demo-app:
    build:
      context: ./app
    depends_on:
      avp-server:
        condition: service_started
      keycloak:
        condition: service_healthy
    environment:
      AVP_ENDPOINT: http://avp-server:50051
      POLICY_STORE_ID: demo-policy-store
      IDENTITY_SOURCE_ID: demo-identity-source
      KEYCLOAK_ISSUER: http://keycloak:8080/realms/demo
    ports:
      - "3000:3000"
```

---

## 3. Integración con Keycloak

### 3.1 Configuración de Keycloak

El script [`bootstrap_keycloak.sh`](examples/scripts/bootstrap_keycloak.sh) configura automáticamente:

```bash
#!/bin/bash
# Configuración automática de Keycloak

# 1. Esperar a que Keycloak esté disponible
wait_for_keycloak() {
  until curl -sf "http://localhost:8080/realms/master" >/dev/null; do
    sleep 3
  done
}

# 2. Crear realm "demo"
create_realm() {
  curl -X POST "http://localhost:8080/admin/realms" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -d '{"realm": "demo", "enabled": true}'
}

# 3. Crear roles
create_roles() {
  local roles=("pet-admin" "pet-vet" "pet-customer")
  for role in "${roles[@]}"; do
    curl -X POST "http://localhost:8080/admin/realms/demo/roles" \
      -d "{\"name\": \"$role\"}"
  done
}

# 4. Crear client "demo-app"
create_client() {
  curl -X POST "http://localhost:8080/admin/realms/demo/clients" \
    -d '{
      "clientId": "demo-app",
      "enabled": true,
      "publicClient": false,
      "secret": "demo-secret",
      "directAccessGrantsEnabled": true
    }'
}

# 5. Crear usuarios con roles
create_users() {
  # pet_admin (role: pet-admin)
  # pet_vet (role: pet-vet)
  # pet_customer (role: pet-customer)
}
```

### 3.2 Configuración Manual en Keycloak UI

Si prefieres configurar manualmente:

1. **Acceder a Keycloak**: http://localhost:8080
   - Usuario: `admin`
   - Contraseña: `admin`

2. **Crear Realm**:
   - Click en dropdown "master" → "Create realm"
   - Name: `demo`
   - Enabled: ON

3. **Crear Roles**:
   - Realm roles → Add role
   - `pet-admin`, `pet-vet`, `pet-customer`

4. **Crear Client**:
   - Clients → Create client
   - Client ID: `demo-app`
   - Client authentication: ON
   - Authentication flow: Direct access grants
   - Credentials → Client secret: `demo-secret`

5. **Crear Usuarios**:
   - Users → Add user
   - Username: `pet_admin`
   - Credentials → Set password: `Password123!`
   - Role mapping → Assign `pet-admin`

### 3.3 Configuración de Identity Source en AVP

El script [`seed_avp.sh`](examples/scripts/seed_avp.sh) configura la integración:

```bash
# Crear Identity Source en AVP
create_identity_source() {
  grpcurl -plaintext -d "{
    \"policy_store_id\": \"demo-policy-store\",
    \"identity_source_id\": \"demo-identity-source\",
    \"oidc_configuration\": {
      \"issuer\": \"http://keycloak:8080/realms/demo\",
      \"client_ids\": [\"demo-app\"],
      \"jwks_uri\": \"http://keycloak:8080/realms/demo/protocol/openid-connect/certs\",
      \"group_claim\": \"realm_access.roles\"
    },
    \"claims_mapping\": {
      \"principal_id_claim\": \"sub\",
      \"group_claim\": \"realm_access.roles\"
    }
  }" localhost:50051 hodei.permissions.v1.Authorization/CreateIdentitySource
}
```

### 3.4 Obtener Tokens de Keycloak

```bash
# Obtener token para pet_admin
curl -X POST "http://localhost:8080/realms/demo/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=demo-app" \
  -d "client_secret=demo-secret" \
  -d "username=pet_admin" \
  -d "password=Password123!" | jq -r '.access_token'

# Decodificar JWT para ver claims
echo $TOKEN | jwt decode
```

**Claims en el JWT:**
```json
{
  "sub": "uuid-del-usuario",
  "realm_access": {
    "roles": ["pet-admin", "default-roles-demo"]
  },
  "preferred_username": "pet_admin"
}
```

---

## 4. SDKs y APIs

### 4.1 verified-permissions-sdk (Data Plane)

**Instalación:**
```toml
# Cargo.toml
[dependencies]
verified-permissions-sdk = "0.2"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

**Uso Básico:**
```rust
use verified_permissions_sdk::{
    Client, IsAuthorizedRequestBuilder, EntityBuilder, Decision
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Conectar al servidor
    let mut client = Client::connect("http://localhost:50051").await?;
    
    // 2. Construir request de autorización
    let request = IsAuthorizedRequestBuilder::new("read", "document:123")
        .with_principal(EntityBuilder::new("User", "alice").build())
        .with_context(json!({
            "department": "engineering",
            "ip": "192.168.1.1"
        }))
        .build();
    
    // 3. Ejecutar autorización
    let response = client.is_authorized("demo-policy-store", request).await?;
    
    // 4. Procesar decisión
    match response.decision {
        Decision::Allow => println!("✅ Acceso permitido"),
        Decision::Deny => println!("❌ Acceso denegado"),
    }
    
    Ok(())
}
```

**Integración con JWT de Keycloak:**
```rust
use verified_permissions_sdk::middleware::VerifiedPermissionsLayer;

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar cliente
    let client = AuthorizationClient::connect("http://localhost:50051").await?;
    
    // Crear middleware
    let layer = VerifiedPermissionsLayer::new(
        client,
        "demo-policy-store".to_string(),
        "demo-identity-source".to_string(),
    );
    
    // Usar en Axum
    let app = Router::new()
        .route("/pets", get(list_pets))
        .layer(layer);
    
    Ok(())
}
```

### 4.2 verified-permissions-sdk-admin (Control Plane)

**Instalación:**
```toml
[dependencies]
verified-permissions-sdk-admin = "0.2"
```

**Gestión de Policy Stores:**
```rust
use verified_permissions_sdk_admin::HodeiAdmin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut admin = HodeiAdmin::connect("http://localhost:50051").await?;
    
    // Crear policy store
    let store = admin.create_policy_store(
        "production-store",
        Some("Store para producción".to_string())
    ).await?;
    
    println!("Creado: {}", store.policy_store_id);
    
    // Subir schema
    let schema = r#"
    {
      "entityTypes": {
        "User": {
          "memberOfTypes": ["Group"],
          "shape": { "type": "Record", "attributes": {} }
        },
        "Document": {
          "memberOfTypes": [],
          "shape": { "type": "Record", "attributes": {} }
        }
      },
      "actions": {
        "read": { "appliesTo": { "resourceTypes": ["Document"] } }
      }
    }
    "#;
    
    admin.put_schema(&store.policy_store_id, schema).await?;
    
    // Crear política
    admin.create_policy(
        &store.policy_store_id,
        "allow-read",
        r#"permit(principal, action == Action::"read", resource);"#,
        Some("Permitir lectura".to_string())
    ).await?;
    
    Ok(())
}
```

### 4.3 hvp CLI

**Instalación:**
```bash
cargo install --path hvp-cli
```

**Comandos principales:**
```bash
# Crear policy store
hvp create policy-store --name production --description "Production environment"

# Subir schema
hvp put schema ps_abc123 schema.json

# Crear política desde archivo
hvp create policy ps_abc123 admin-policy --file admin.cedar

# Verificar autorización
hvp check ps_abc123 User::"alice" read Document::"doc123"

# Listar políticas
hvp list policies ps_abc123

# Obtener audit log
hvp get audit-log ps_abc123 --max-results 100
```

**Ejemplo de archivo .cedar:**
```cedar
// policies/admin.cedar
permit(
    principal in Group::"admins",
    action in [
        Action::"read",
        Action::"write",
        Action::"delete"
    ],
    resource in Document::*
)
when {
    principal.department == resource.department
};
```

### 4.4 API gRPC

**Servicios disponibles:**

```protobuf
// proto/authorization.proto
service AuthorizationControl {
  rpc CreatePolicyStore(CreatePolicyStoreRequest) returns (PolicyStore);
  rpc GetPolicyStore(GetPolicyStoreRequest) returns (PolicyStore);
  rpc ListPolicyStores(ListPolicyStoresRequest) returns (ListPolicyStoresResponse);
  rpc DeletePolicyStore(DeletePolicyStoreRequest) returns (google.protobuf.Empty);
  
  rpc PutSchema(PutSchemaRequest) returns (google.protobuf.Empty);
  rpc GetSchema(GetSchemaRequest) returns (Schema);
  
  rpc CreatePolicy(CreatePolicyRequest) returns (Policy);
  rpc GetPolicy(GetPolicyRequest) returns (Policy);
  rpc ListPolicies(ListPoliciesRequest) returns (ListPoliciesResponse);
  rpc UpdatePolicy(UpdatePolicyRequest) returns (Policy);
  rpc DeletePolicy(DeletePolicyRequest) returns (google.protobuf.Empty);
  
  rpc CreateIdentitySource(CreateIdentitySourceRequest) returns (IdentitySource);
}

service AuthorizationData {
  rpc IsAuthorized(IsAuthorizedRequest) returns (IsAuthorizedResponse);
  rpc BatchIsAuthorized(BatchIsAuthorizedRequest) returns (BatchIsAuthorizedResponse);
}
```

**Ejemplo con grpcurl:**
```bash
# Crear policy store
grpcurl -plaintext -d '{
  "name": "Production Store",
  "description": "Main production policy store"
}' localhost:50051 hodei.permissions.v1.AuthorizationControl/CreatePolicyStore

# Verificar autorización
grpcurl -plaintext -d '{
  "policy_store_id": "ps_01HABC123DEFG456HIJ7",
  "principal": "User::\"alice\"",
  "action": "Action::\"viewDocument\"",
  "resource": "Document::\"doc123\""
}' localhost:50051 hodei.permissions.v1.AuthorizationData/IsAuthorized
```

---

## 5. Conversión OpenAPI a Schema Cedar

### 5.1 Funcionalidad Actual

El SDK incluye conversión automática de OpenAPI a schema Cedar:

```rust
use verified_permissions_sdk::schema::{SimpleRestSchemaGenerator, OpenApiParser};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Cargar OpenAPI spec
    let openapi_spec = std::fs::read_to_string("openapi.json")?;
    let spec = OpenApiParser::parse(&openapi_spec)?;
    
    // 2. Generar schema Cedar
    let generator = SimpleRestSchemaGenerator::new("PetStore");
    let cedar_schema = generator.generate(&spec, Some("/api/v1"))?;
    
    // 3. Serializar a JSON
    let schema_json = serde_json::to_string_pretty(&cedar_schema)?;
    std::fs::write("schema.cedar.json", schema_json)?;
    
    Ok(())
}
```

### 5.2 CLI para Conversión

```bash
# Generar schema desde OpenAPI
hvp generate schema \
  --spec openapi.json \
  --namespace PetStore \
  --base-path /api/v1 \
  --output schema.cedar.json

# Generar políticas por defecto
hvp generate policies \
  --spec openapi.json \
  --namespace PetStore \
  --roles admin,vet,customer \
  --output policies/
```

### 5.3 Mapeo OpenAPI → Cedar

| OpenAPI | Cedar | Ejemplo |
|---------|-------|---------|
| `GET /users/{id}` | `Action::"GET::/users/{id}"` | `permit(principal, action == Action::"GET::/users/{id}", resource);` |
| `POST /users` | `Action::"POST::/users"` | `permit(principal, action == Action::"POST::/users", resource);` |
| Path parameters | `context.pathParameters.id` | `when { context.pathParameters.id == principal.id }` |
| Query parameters | `context.queryStringParameters` | `when { context.queryStringParameters.limit < 100 }` |

### 5.4 Extensiones OpenAPI (x-cedar)

Puedes añadir metadatos Cedar a tu OpenAPI:

```yaml
# openapi.yaml
paths:
  /pets/{id}:
    get:
      operationId: getPet
      x-cedar:
        appliesToResourceTypes: ["Pet"]
        requiredAttributes: ["principal.id"]
      parameters:
        - name: id
          in: path
          schema:
            type: string
      responses:
        '200':
          description: Success
```

---

## 6. Ejemplos Prácticos de Integración

### 6.1 Aplicación Axum Completa

```rust
// src/main.rs
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router, middleware,
};
use hodei_permissions_sdk::{
    middleware::VerifiedPermissionsLayer, AuthorizationClient, Decision
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    // Tu estado de aplicación
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Conectar a AVP
    let client = AuthorizationClient::connect("http://localhost:50051").await?;
    
    // 2. Crear middleware
    let layer = VerifiedPermissionsLayer::new(
        client,
        "demo-policy-store".to_string(),
        "demo-identity-source".to_string(),
    );
    
    // 3. Estado de la app
    let state = Arc::new(AppState { /* ... */ });
    
    // 4. Configurar router
    let app = Router::new()
        // Rutas públicas
        .route("/health", get(health_check))
        // Rutas protegidas
        .route("/api/pets", get(list_pets).post(create_pet))
        .route("/api/pets/:id", get(get_pet))
        .route("/api/pets/:id/appointments", 
               get(list_appointments).post(create_appointment))
        .with_state(state)
        // Aplicar middleware de autorización
        .layer(layer);
    
    // 5. Iniciar servidor
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Handlers
async fn health_check() -> &'static str {
    "OK"
}

async fn list_pets(
    State(state): State<Arc<AppState>>,
    claims: Claims,  // Extraído del JWT por el middleware
) -> Json<Vec<Pet>> {
    // El middleware ya verificó la autorización
    // Solo llegamos aquí si el usuario tiene permiso
    
    let pets = get_pets_from_db(&state).await;
    Json(pets)
}

async fn create_pet(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(payload): Json<CreatePetRequest>,
) -> Result<Json<Pet>, StatusCode> {
    // Verificación adicional si es necesario
    if !claims.roles.contains(&"pet-admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let pet = create_pet_in_db(&state, payload).await;
    Ok(Json(pet))
}
```

### 6.2 Middleware Personalizado

```rust
// src/middleware/auth.rs
use hodei_permissions_sdk::{
    AuthorizationClient, IsAuthorizedRequest, Entity, Decision
};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    async_trait,
};

#[derive(Debug, Clone)]
pub struct Claims {
    pub sub: String,
    pub roles: Vec<String>,
    pub username: String,
}

#[async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = (StatusCode, String);
    
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Extraer token del header
        let auth_header = parts
            .headers
            .get("authorization")
            .ok_or((StatusCode::UNAUTHORIZED, "Missing token".to_string()))?
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid token".to_string()))?;
        
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::BAD_REQUEST, "Invalid token format".to_string()))?;
        
        // 2. Verificar con AVP
        let client = &state.avp_client;
        let request = IsAuthorizedRequest {
            principal: Some(Entity::new("User", &claims.sub)),
            action: "Action::\"access\"".to_string(),
            resource: "Resource::\"api\"".to_string(),
            context: serde_json::json!({
                "token": token,
                "path": parts.uri.path(),
                "method": parts.method.as_str()
            }),
        };
        
        let response = client
            .is_authorized("demo-policy-store", request)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        // 3. Verificar decisión
        match response.decision {
            Decision::Allow => {
                // Extraer claims del contexto
                let claims = extract_claims_from_token(token)?;
                Ok(claims)
            }
            Decision::Deny => Err((
                StatusCode::FORBIDDEN,
                "Access denied".to_string()
            )),
        }
    }
}
```

### 6.3 Testing con Tokens

```rust
// tests/integration_test.rs
use hodei_permissions_sdk::Client;

#[tokio::test]
async fn test_pet_admin_access() {
    // 1. Obtener token de Keycloak
    let token = get_keycloak_token("pet_admin", "Password123!").await;
    
    // 2. Crear cliente AVP
    let mut client = Client::connect("http://localhost:50051").await.unwrap();
    
    // 3. Testear acceso a GET /pets
    let request = IsAuthorizedRequestBuilder::new("GET::/pets", "PetStore::\"api\"")
        .with_principal(EntityBuilder::new("User", "pet_admin").build())
        .with_context(json!({
            "jwt": token,
            "realm_access": {
                "roles": ["pet-admin"]
            }
        }))
        .build();
    
    let response = client
        .is_authorized("demo-policy-store", request)
        .await
        .unwrap();
    
    assert_eq!(response.decision, Decision::Allow);
}

#[tokio::test]
async fn test_pet_customer_cannot_create() {
    let token = get_keycloak_token("pet_customer", "Password123!").await;
    
    let mut client = Client::connect("http://localhost:50051").await.unwrap();
    
    let request = IsAuthorizedRequestBuilder::new("POST::/pets", "PetStore::\"api\"")
        .with_principal(EntityBuilder::new("User", "pet_customer").build())
        .with_context(json!({
            "jwt": token,
            "realm_access": {
                "roles": ["pet-customer"]
            }
        }))
        .build();
    
    let response = client
        .is_authorized("demo-policy-store", request)
        .await
        .unwrap();
    
    assert_eq!(response.decision, Decision::Deny);
}

async fn get_keycloak_token(username: &str, password: &str) -> String {
    let response = reqwest::Client::new()
        .post("http://localhost:8080/realms/demo/protocol/openid-connect/token")
        .form(&[
            ("grant_type", "password"),
            ("client_id", "demo-app"),
            ("client_secret", "demo-secret"),
            ("username", username),
            ("password", password),
        ])
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    
    response["access_token"].as_str().unwrap().to_string()
}
```

---

## 7. Testing y Debugging

### 7.1 End-to-End Testing

```bash
# Ejecutar tests completos
cd examples
./scripts/check_end_to_end.sh

# Salida esperada:
# ===> GET http://localhost:3000/pets (esperado: 200)
#     status=200
#     ✅
# ===> POST http://localhost:3000/pets (esperado: 200)
#     status=200
#     ✅
# ===> Verificación end-to-end completada
```

### 7.2 Debugging con Web Console

1. **Acceder a la web console**: http://localhost:3000
2. **Navegar a Policy Store**: `demo-policy-store`
3. **Ver Audit Log**: Todas las decisiones de autorización
4. **Testear políticas**: Usar el playground

### 7.3 Logs y Tracing

```bash
# Ver logs de AVP
docker logs -f examples-avp-server-1

# Ver logs con tracing
RUST_LOG=debug cargo run

# Filtrar por componente
RUST_LOG=hodei_permissions_sdk=debug,avp_server=info cargo run
```

### 7.4 grpcurl para Debugging

```bash
# Listar servicios
grpcurl -plaintext localhost:50051 list

# Ver método concreto
grpcurl -plaintext localhost:50051 describe hodei.permissions.v1.AuthorizationControl

# Testear health check
grpcurl -plaintext -d '{}' localhost:50051 grpc.health.v1.Health/Check
```

---

## 8. Mejores Prácticas

### 8.1 Seguridad

- ✅ **Principio de mínimo privilegio**: Siempre usar `default-deny`
- ✅ **Validar inputs**: Sanitizar todos los parámetros antes de usarlos en políticas
- ✅ **Rotación de tokens**: Implementar refresh tokens en producción
- ✅ **TLS en producción**: Nunca usar `plaintext` en gRPC
- ✅ **Secrets management**: Usar vaults, no variables de entorno

### 8.2 Performance

- ✅ **Cache de decisiones**: Implementar TTL para decisiones frecuentes
- ✅ **Batch requests**: Usar `BatchIsAuthorized` para múltiples recursos
- ✅ **Connection pooling**: Reutilizar conexiones gRPC
- ✅ **Índices en DB**: Asegurar índices en policy_store_id, created_at

### 8.3 Mantenibilidad

- ✅ **Versionado de políticas**: Usar git para versionar archivos .cedar
- ✅ **CI/CD**: Automatizar despliegue de políticas
- ✅ **Testing**: 100% de cobertura en lógica de autorización
- ✅ **Documentación**: Comentar políticas complejas

### 8.4 Estructura de Proyecto

```
my-app/
├── policies/
│   ├── admin.cedar
│   ├── vet.cedar
│   └── customer.cedar
├── schema/
│   └── pet_store.json
├── src/
│   ├── main.rs
│   ├── middleware/
│   │   └── auth.rs
│   └── handlers/
│       ├── pets.rs
│       └── appointments.rs
├── tests/
│   └── integration_test.rs
├── .env
├── Cargo.toml
└── docker-compose.yml
```

---

## 9. Solución de Problemas

### 9.1 Keycloak no responde

```bash
# Verificar logs
docker logs examples-keycloak-1

# Reiniciar servicio
docker compose restart keycloak

# Verificar puerto
netstat -tlnp | grep 8080
```

### 9.2 AVP no puede conectar a Keycloak

**Síntoma**: `Identity source validation failed`

**Solución**:
```bash
# 1. Verificar network de Docker
docker network ls
docker network inspect examples_default

# 2. Verificar que Keycloak está healthy
docker ps | grep keycloak

# 3. Probar conexión desde contenedor AVP
docker exec -it examples-avp-server-1 curl http://keycloak:8080/realms/demo

# 4. Re-crear identity source
./scripts/seed_avp.sh
```

### 9.3 Decisiones de autorización incorrectas

**Debugging paso a paso**:

```bash
# 1. Verificar políticas cargadas
hvp list policies demo-policy-store

# 2. Verificar schema
hvp get schema demo-policy-store

# 3. Testear manualmente con grpcurl
grpcurl -plaintext -d '{
  "policy_store_id": "demo-policy-store",
  "principal": "User::\"pet_admin\"",
  "action": "Action::\"POST::/pets\"",
  "resource": "PetStore::\"api\""
}' localhost:50051 hodei.permissions.v1.AuthorizationData/IsAuthorized

# 4. Ver audit log
hvp get audit-log demo-policy-store --max-results 10
```

### 9.4 Performance lenta

**Diagnóstico**:
```bash
# Benchmark de autorización
hyperfine --warmup 3 \
  'grpcurl -plaintext -d "{\"policy_store_id\":\"demo-policy-store\",...}" localhost:50051 hodei.permissions.v1.AuthorizationData/IsAuthorized'

# Ver métricas
curl http://localhost:3000/api/metrics

# Profile con perf
perf record -g cargo run --release
perf report
```

### 9.5 Errores comunes

| Error | Causa | Solución |
|-------|-------|----------|
| `Invalid JWT` | Token expirado o mal firmado | Renovar token o verificar JWKS URI |
| `Policy not found` | Policy ID incorrecto | Listar políticas con `hvp list` |
| `Schema validation failed` | Schema no cargado | Ejecutar `hvp put schema` |
| `Identity source not found` | Identity source ID incorrecto | Re-crear con `seed_avp.sh` |
| `Connection refused` | Servidor no está corriendo | Verificar con `docker ps` |

---

## 📚 Recursos Adicionales

- **[Documentación completa](docs/)**: Documentación detallada del proyecto
- **[Ejemplos](examples/)**: Ejemplos funcionales completos
- **[Cedar Policy Language](https://docs.cedarpolicy.com/)**: Referencia del lenguaje Cedar
- **[AWS Verified Permissions](https://docs.aws.amazon.com/verified-permissions/)**: Guía oficial de AWS
- **[Keycloak Documentation](https://www.keycloak.org/documentation)**: Documentación de Keycloak

---

## 🆘 Soporte

- **GitHub Issues**: [Reportar bugs o solicitar features](https://github.com/rubentxu/hodei-verified-permissions/issues)
- **GitHub Discussions**: [Preguntas y discusiones](https://github.com/rubentxu/hodei-verified-permissions/discussions)
- **Email**: support@hodei.dev

---

<div align="center">
<strong>Happy coding! 🚀</strong>
</div>