# Guía de Middleware

Guía completa para usar el middleware de Hodei Permissions con Axum y Tower.

## Tabla de Contenidos

- [Visión General](#visión-general)
- [Instalación](#instalación)
- [Configuración Básica](#configuración-básica)
- [Configuración Avanzada](#configuración-avanzada)
- [Extractores Personalizados](#extractores-personalizados)
- [Manejo de Errores](#manejo-de-errores)
- [Mejores Prácticas](#mejores-prácticas)
- [Ejemplos](#ejemplos)

## Visión General

El middleware de Hodei Permissions proporciona verificaciones de autorización automáticas para servicios HTTP construidos con Axum y Tower.

### Flujo del Middleware

```mermaid
sequenceDiagram
    participant Client as Cliente
    participant Middleware
    participant Extractor
    participant AuthService as Servicio Autorización
    participant Handler as Handler

    Client->>Middleware: Petición HTTP + JWT
    Middleware->>Extractor: Extraer Token
    Extractor-->>Middleware: Token
    Middleware->>Extractor: Extraer Acción y Recurso
    Extractor-->>Middleware: AuthorizationParts
    
    Middleware->>AuthService: IsAuthorizedWithToken
    
    alt Allow
        AuthService-->>Middleware: Decisión: Allow
        Middleware->>Handler: Encaminar Petición
        Handler-->>Middleware: Respuesta
        Middleware-->>Client: 200 OK + Respuesta
    else Deny
        AuthService-->>Middleware: Decisión: Deny
        Middleware-->>Client: 403 Forbidden
    end
```

### Cómo Funciona

1. Extrae tokens JWT de las peticiones
2. Mapea los métodos HTTP a acciones
3. Usa la ruta URI como recurso
4. Llama a `IsAuthorizedWithToken` automáticamente
5. Devuelve 403 Forbidden en caso de Deny
6. Continúa al handler si es Allow

## Instalación

Añade la feature `middleware` en tu `Cargo.toml`:

```toml
[dependencies]
hodei-permissions-sdk = { version = "0.1", features = ["middleware"] }
axum = "0.7"
tower = "0.5"
tokio = { version = "1.40", features = ["full"] }
```

## Configuración Básica

### 1. Crear la Capa de Middleware

```rust
use hodei_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    // Conectar al servicio de autorización
    let client = AuthorizationClient::connect("http://localhost:50051")
        .await
        .expect("Error de conexión");

    // Crear middleware
    let auth_layer = VerifiedPermissionsLayer::new(
        client,
        "policy-store-id",
        "identity-source-id"
    );

    // Aplicar a rutas
    let app = Router::new()
        .route("/api/documentos", get(listar_documentos))
        .route("/api/documentos/:id", get(obtener_documento))
        .layer(auth_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### 2. Handlers

```rust
use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Documento {
    id: String,
    titulo: String,
    contenido: String,
}

async fn listar_documentos() -> Json<Vec<Documento>> {
    Json(vec![Documento {
        id: "doc1".to_string(),
        titulo: "Documento 1".to_string(),
        contenido: "Contenido 1".to_string(),
    }])
}

async fn obtener_documento(Path(id): Path<String>) -> Json<Documento> {
    Json(Documento {
        id,
        titulo: "Documento".to_string(),
        contenido: "Contenido".to_string(),
    })
}
```

### 3. Peticiones con JWT

```bash
curl -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIs..." \
     http://localhost:3000/api/documentos
```

## Configuración Avanzada

### Rutas Protegidas Selectivamente

```rust
let app = Router::new()
    .route("/health", get(health_check))  // Pública
    .route("/public/info", get(info_publica))
    
    .route("/api/documentos", get(listar_documentos)) // Protegida
    .route("/api/documentos/:id", get(obtener_documento))
    .layer(auth_layer);
```

### Múltiples Policy Stores

```rust
let admin_layer = VerifiedPermissionsLayer::new(
    client.clone(),
    "admin-policy-store",
    "admin-identity-source"
);

let user_layer = VerifiedPermissionsLayer::new(
    client.clone(),
    "user-policy-store",
    "user-identity-source"
);

let app = Router::new()
    .nest("/admin", admin_routes.layer(admin_layer))
    .nest("/api", user_routes.layer(user_layer));
```

### Cliente Compartido

```rust
use std::sync::Arc;

let client = Arc::new(
    AuthorizationClient::connect("http://localhost:50051").await?
);

let layer1 = VerifiedPermissionsLayer::from_arc(
    client.clone(),
    "store-1",
    "identity-1"
);

let layer2 = VerifiedPermissionsLayer::from_arc(
    client.clone(),
    "store-2",
    "identity-2"
);
```

## Extractores Personalizados

### Implementación

```rust
use hodei_permissions_sdk::middleware::{
    AuthorizationRequestExtractor, AuthorizationRequestParts, MiddlewareError
};
use async_trait::async_trait;
use http::Request;

struct ApiKeyExtractor;

#[async_trait]
impl<B> AuthorizationRequestExtractor<B> for ApiKeyExtractor
where
    B: Send + Sync,
{
    type Error = MiddlewareError;

    async fn extract(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
        let api_key = req.headers()
            .get("x-api-key")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| MiddlewareError::AuthorizationHeader(
                "Falta cabecera x-api-key".to_string()
            ))?;

        let principal = format!("ApiKey::{}", api_key);

        let action = match req.method().as_str() {
            "GET" => "Action::\"read\"",
            "POST" => "Action::\"create\"",
            "PUT" | "PATCH" => "Action::\"update\"",
            "DELETE" => "Action::\"delete\"",
            _ => return Err(MiddlewareError::ExtractionFailed(
                "Método HTTP no soportado".to_string()
            )),
        };

        let path = req.uri().path();
        let resource = format!("Resource::\"{}\"", path.trim_start_matches('/'));

        Ok(AuthorizationRequestParts {
            principal,
            action: action.to_string(),
            resource,
            context: None,
        })
    }
}
```

## Manejo de Errores

### Códigos por Defecto

| Error | Código | Descripción |
|-------|--------|-------------|
| Cabecera Authorization inválida | 401 Unauthorized | Falta o formato incorrecto |
| Token inválido | 401 Unauthorized | Firma inválida / expirado |
| Acceso denegado | 403 Forbidden | Política devuelve Deny |
| Error de servicio | 500 Internal Server Error | Fallo en llamado gRPC |

### Respuestas Personalizadas

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

async fn handle_error(err: BoxError) -> Response {
    if err.is::<MiddlewareError>() {
        let middleware_err = err.downcast_ref::<MiddlewareError>().unwrap();
        
        let (status, message) = match middleware_err {
            MiddlewareError::AuthorizationHeader(_) => (
                StatusCode::UNAUTHORIZED,
                "Autorización inválida"
            ),
            MiddlewareError::AccessDenied(_) => (
                StatusCode::FORBIDDEN,
                "Acceso denegado"
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error de autorización"
            ),
        };

        (status, Json(json!({
            "error": message,
            "detalles": middleware_err.to_string()
        }))).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Error interno").into_response()
    }
}
```

## Mejores Prácticas

- **Cliente Compartido**: Usa un `AuthorizationClient` compartido (`Arc`)
- **Configurar TTL JWKS**: Ajusta `JWKS_CACHE_TTL` según tu IdP
- **Políticas Claras**: Diseña políticas específicas para rutas HTTP
- **Pruebas E2E**: Probar con tokens reales del IdP
- **Logs**: Registra decisiones de autorización para auditoría

## Ejemplos

### API Multi-Tenant

```rust
use hodei_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
use axum::{Router, routing::{get, post, delete}, extract::{Path, State}, Json};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    auth_client: Arc<AuthorizationClient>,
}

#[tokio::main]
async fn main() {
    let auth_client = Arc::new(
        AuthorizationClient::connect("http://localhost:50051")
            .await
            .unwrap()
    );

    let state = AppState { auth_client: auth_client.clone() };

    let auth_layer = VerifiedPermissionsLayer::from_arc(
        auth_client,
        "multi-tenant-store",
        "oidc-identity-source"
    );

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/tenants/:tenant_id/documentos", get(listar_documentos).post(crear_documento))
        .route("/api/tenants/:tenant_id/documentos/:doc_id", get(obtener_documento).delete(borrar_documento))
        .layer(auth_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### Rate Limiting + Autorización

```rust
use tower::ServiceBuilder;
use tower_http::limit::RateLimitLayer;
use std::time::Duration;

let app = Router::new()
    .route("/api/documentos", get(listar_documentos))
    .layer(
        ServiceBuilder::new()
            .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
            .layer(auth_layer)
    );
```

## Resolución de Problemas

- **"Missing Authorization header"**: Asegúrate de enviar `Authorization: Bearer <token>`
- **"Token validation failed"**: Verifica issuer, JWKS y expiración
- **"Access denied"**: Revisa tus políticas Cedar
- **Middleware no aplica**: Confirmar que `.layer()` envuelve las rutas correctas
- **Logs**: Habilita `RUST_LOG=debug` para más detalle

## Lecturas Recomendadas

- [README.es.md](../../README.es.md)
- [Guía Identity Sources](IDENTITY_SOURCES.es.md)
- [Documentación Axum](https://docs.rs/axum)
- [Documentación Tower](https://docs.rs/tower)
- [Cedar Policy](https://docs.cedarpolicy.com/)
