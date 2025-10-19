# Hodei Permissions SDK

SDK cliente ergonómico en Rust para el servicio de autorización Hodei Verified Permissions.

## 📚 Tabla de Contenidos

- [Características](#características)
- [Instalación](#instalación)
- [Inicio Rápido](#inicio-rápido)
- [Guía de Uso](#guía-de-uso)
  - [Autorización Básica](#autorización-básica)
  - [Autorización con Tokens JWT](#autorización-con-tokens-jwt)
  - [Middleware (Axum/Tower)](#middleware-axumtower)
  - [Identity Sources](#identity-sources)
- [Referencia de API](#referencia-de-api)
- [Ejemplos](#ejemplos)
- [Para Desarrolladores](#para-desarrolladores)
- [Pruebas](#pruebas)

## ✨ Características

- 🚀 **API Sencilla**: Métodos fáciles para todas las operaciones
- 🔧 **Patrones Builder**: API fluida para peticiones complejas
- ⚡ **Async/Await**: Construido sobre Tokio para alto rendimiento
- 🛡️ **Type Safe**: Aprovecha el sistema de tipos de Rust
- 📝 **Bien Documentado**: Ejemplos y documentación completos
- 🔐 **Soporte JWT**: Validación incorporada de tokens con Identity Sources
- 🌐 **Integración IdP**: Soporte para Keycloak, Zitadel y AWS Cognito
- 🔌 **Middleware**: Middleware Axum/Tower opcional (feature flag)
- 🎯 **Políticas Cedar**: Soporte completo para el lenguaje Cedar

## 📦 Instalación

Añade a tu `Cargo.toml`:

```toml
[dependencies]
hodei-permissions-sdk = "0.1"
tokio = { version = "1.40", features = ["full"] }
```

### Con soporte de Middleware

```toml
[dependencies]
hodei-permissions-sdk = { version = "0.1", features = ["middleware"] }
axum = "0.7"
tower = "0.5"
```

## 🚀 Inicio Rápido

```rust
use hodei_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Conectar al servicio
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    // Verificar autorización
    let response = client
        .is_authorized(
            "policy-store-id",
            "User::alice",
            "Action::view",
            "Document::doc123"
        )
        .await?;

    if response.decision() == hodei_permissions_sdk::Decision::Allow {
        println!("✅ ¡Acceso concedido!");
    } else {
        println!("❌ ¡Acceso denegado!");
    }

    Ok(())
}
```

## 📖 Guía de Uso

### Autorización Básica

#### 1. Crear un Policy Store

```rust
let store = client
    .create_policy_store(Some("Mi Aplicación".to_string()))
    .await?;

println!("Policy Store ID: {}", store.policy_store_id);
```

#### 2. Crear Políticas Cedar

```rust
let policy = r#"
permit(
    principal == User::"alice",
    action == Action::"view",
    resource == Document::"doc123"
);
"#;

client.create_policy(
    &store.policy_store_id,
    "permitir-alice",
    policy,
    Some("Permite a Alice ver el documento 123".to_string())
).await?;
```

#### 3. Verificar Autorización

```rust
let response = client
    .is_authorized(
        &store.policy_store_id,
        "User::alice",
        "Action::view",
        "Document::doc123"
    )
    .await?;

println!("Decisión: {:?}", response.decision());
```

### Autorización con Tokens JWT

#### 1. Crear un Identity Source

```rust
use hodei_permissions_sdk::proto::{
    IdentitySourceConfiguration, OidcConfiguration,
    identity_source_configuration, ClaimsMappingConfiguration
};
use std::collections::HashMap;

let oidc_config = OidcConfiguration {
    issuer: "https://tu-idp.com".to_string(),
    client_ids: vec!["tu-client-id".to_string()],
    jwks_uri: "https://tu-idp.com/.well-known/jwks.json".to_string(),
    group_claim: "groups".to_string(),
};

let config = IdentitySourceConfiguration {
    configuration_type: Some(
        identity_source_configuration::ConfigurationType::Oidc(oidc_config)
    ),
};

let mut attribute_mappings = HashMap::new();
attribute_mappings.insert("email".to_string(), "email".to_string());
attribute_mappings.insert("name".to_string(), "name".to_string());

let claims_mapping = ClaimsMappingConfiguration {
    principal_id_claim: "sub".to_string(),
    group_claim: String::new(),
    attribute_mappings,
};

let identity_source = client
    .create_identity_source(
        &store.policy_store_id,
        config,
        Some(claims_mapping),
        Some("Mi IdP".to_string())
    )
    .await?;
```

#### 2. Autorizar con un Token JWT

```rust
let jwt_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";

let response = client
    .is_authorized_with_token(
        &store.policy_store_id,
        &identity_source.identity_source_id,
        jwt_token,
        "Action::\"read\"",
        "Document::\"doc123\""
    )
    .await?;

if response.decision() == Decision::Allow {
    println!("✅ Token válido y acceso concedido");
}
```

### Middleware (Axum/Tower)

Protege tus rutas HTTP con autorizaciones automáticas.

```rust
use hodei_permissions_sdk::{AuthorizationClient, middleware::VerifiedPermissionsLayer};
use axum::{Router, routing::get, Json};

#[tokio::main]
async fn main() {
    let client = AuthorizationClient::connect("http://localhost:50051")
        .await
        .unwrap();

    let auth_layer = VerifiedPermissionsLayer::new(
        client,
        "policy-store-123",
        "identity-source-456"
    );

    let app = Router::new()
        .route("/api/documentos", get(listar_documentos))
        .layer(auth_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### Identity Sources

Consulta la [Guía de Identity Sources](docs/IDENTITY_SOURCES.es.md) para ver configuraciones detalladas de Keycloak, Zitadel y AWS Cognito.

## 📚 Referencia de API

### Plano de Datos (Autorización)

| Método | Descripción |
|--------|-------------|
| `is_authorized()` | Verificación simple de autorización |
| `is_authorized_with_context()` | Autorización con entidades y contexto |
| `is_authorized_with_token()` | Autorización con JWT |
| `batch_is_authorized()` | Verificación de múltiples peticiones |

### Plano de Control

**Policy Stores:**
- `create_policy_store()`
- `get_policy_store()`
- `list_policy_stores()`
- `delete_policy_store()`

**Schemas:**
- `put_schema()`
- `get_schema()`

**Policies:**
- `create_policy()`
- `get_policy()`
- `list_policies()`
- `delete_policy()`

**Identity Sources:**
- `create_identity_source()`
- `get_identity_source()`
- `list_identity_sources()`
- `delete_identity_source()`

## 💡 Ejemplos

### Ejemplo Completo: Documentos

```rust
use hodei_permissions_sdk::{AuthorizationClient, EntityBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuthorizationClient::connect("http://localhost:50051").await?;

    let store = client.create_policy_store(Some("DocApp".to_string())).await?;

    let policy = r#"
        permit(
            principal,
            action == Action::"view",
            resource
        ) when {
            resource.owner == principal ||
            principal in resource.viewers
        };
    "#;

    client.create_policy(
        &store.policy_store_id,
        "permitir-visor",
        policy,
        Some("Permite a dueños y visores ver documentos".to_string())
    ).await?;

    let alice = EntityBuilder::new("User", "alice").build();
    
    let doc = EntityBuilder::new("Document", "doc123")
        .attribute("owner", r#"{"__entity": {"type": "User", "id": "alice"}}"#)
        .build();

    let request = IsAuthorizedRequestBuilder::new(&store.policy_store_id)
        .principal("User", "alice")
        .action("Action", "view")
        .resource("Document", "doc123")
        .add_entity(alice)
        .add_entity(doc)
        .build();

    let response = client.is_authorized_with_context(request).await?;

    println!("Decisión: {:?}", response.decision());

    Ok(())
}
```

## 🔧 Para Desarrolladores

### Extender el SDK

```rust
use hodei_permissions_sdk::AuthorizationClient;

impl AuthorizationClient {
    pub async fn my_custom_method(&self) -> Result<(), SdkError> {
        // Tu lógica personalizada
        Ok(())
    }
}
```

### Estructura del Proyecto

```
sdk/
├── src/
│   ├── lib.rs
│   ├── client.rs
│   ├── builders.rs
│   ├── error.rs
│   └── middleware/
├── docs/
│   ├── README.md
│   ├── MIDDLEWARE_GUIDE.md
│   └── IDENTITY_SOURCES.md
├── examples/
├── tests/
└── Cargo.toml
```

## 🧪 Pruebas

```bash
# Tests unitarios
cargo test

# Tests de integración
cargo test --features integration-tests

# Tests de middleware
cargo test --features middleware
```

## 🐛 Manejo de Errores

```rust
use hodei_permissions_sdk::SdkError;

match client.is_authorized(...).await {
    Ok(response) => println!("Decisión: {:?}", response.decision()),
    Err(SdkError::ConnectionError(e)) => eprintln!("Error de conexión: {}", e),
    Err(SdkError::StatusError(status)) => eprintln!("Error gRPC: {}", status),
    Err(e) => eprintln!("Error: {}", e),
}
```

## 📄 Licencia

MIT

## 🤝 Contribuciones

¡Las contribuciones son bienvenidas! Ver [README.es.md](../README.es.md) para más detalles.
