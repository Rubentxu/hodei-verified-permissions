# 🔐 Hodei Verified Permissions

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Cedar](https://img.shields.io/badge/cedar-4.7.0-blue.svg)](https://www.cedarpolicy.com/)
[![Tests](https://img.shields.io/badge/tests-66%20passing-brightgreen.svg)](#)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Servicio de Autorización Listo para Producción basado en Cedar** con Soporte Multi-Base de Datos, Caché en Memoria y Latencia Ultra-Baja (~100μs).

## ✨ Características

- 🚀 **Autorización Ultra-Rápida** - Latencia de ~100μs con caché en memoria
- 🗄️ **Soporte Multi-Base de Datos** - SQLite, PostgreSQL, SurrealDB
- 📊 **Métricas Integradas** - Aciertos de caché, latencias, rendimiento
- 🔄 **Recarga Automática** - Actualización de caché en segundo plano cada 5 minutos
- 🎯 **Motor de Políticas Cedar** - Lenguaje de políticas compatible con AWS
- 🔌 **API gRPC** - Comunicación de baja latencia
- 🔐 **Soporte JWT** - Autorización basada en tokens con Identity Sources
- 🌐 **Integración IdP** - Soporte para Keycloak, Zitadel, AWS Cognito
- 🔌 **Middleware** - Middleware Axum/Tower para servicios HTTP
- 📝 **Registro de Auditoría** - Rastro forense completo
- 🎨 **Plantillas de Políticas** - Patrones de políticas reutilizables
- 🏢 **Multi-Tenant** - Almacenes de políticas aislados
- 📚 **Documentación Completa** - Guías para usuarios y desarrolladores

## 📚 Documentación

- **[Guía del SDK](sdk/README.md)** - Documentación completa del SDK para usuarios ([Español](sdk/README.es.md))
- **[Guía de Middleware](sdk/docs/MIDDLEWARE_GUIDE.md)** - Integración middleware Axum/Tower ([Español](sdk/docs/MIDDLEWARE_GUIDE.es.md))
- **[Guía de Identity Sources](sdk/docs/IDENTITY_SOURCES.md)** - Integración Keycloak, Zitadel, AWS Cognito ([Español](sdk/docs/IDENTITY_SOURCES.es.md))
- **[English](README.md)** - English documentation

## 📊 Rendimiento


| Operación                            | Latencia | Rendimiento  |
| ------------------------------------- | -------- | ------------ |
| **IsAuthorized** (en caché)          | ~100μs  | >100K ops/s  |
| **BatchIsAuthorized** (30 peticiones) | ~3ms     | >10K batch/s |
| **CreatePolicy**                      | ~1-2ms   | ~1K ops/s    |

## 🏗️ Arquitectura

```mermaid
graph TD
    subgraph "Capa Frontend"
        A[Aplicación Web Next.js]
        A1[Dashboard]
        A2[Playground]
        A3[Visor de Auditoría]
    end

    subgraph "Capa API"
        B[Rutas API de Next.js]
    end

    subgraph "Capa Backend"
        C[Servidor gRPC Tonic]
        D[AuthorizationControlService<br/>Operaciones CRUD]
        E[AuthorizationDataService<br/>Verificaciones de Autorización]
        F[Interceptor de Auditoría<br/>Publicación de Eventos]
    end

    subgraph "Capa Infraestructura"
        G[Capa de Repositorio<br/>SQLite/PostgreSQL]
        H[Infraestructura de Eventos<br/>Bus de Eventos & Almacén]
        I[Motor de Políticas Cedar<br/>Evaluación de Políticas]
    end

    A --> B
    B --> C
    C --> D
    C --> E
    C --> F
    D --> G
    E --> G
    F --> H
    G --> I
    H --> I

    style A fill:#e1f5fe
    style B fill:#f3e5f5
    style C fill:#e8f5e8
    style D fill:#fff3e0
    style E fill:#fff3e0
    style F fill:#fff3e0
    style G fill:#fce4ec
    style H fill:#fce4ec
    style I fill:#fce4ec
```

### Componentes Clave

- **Frontend (Next.js)**: Interfaz web con dashboard, playground y visor de auditoría
- **Rutas API**: Funciones serverless que manejan peticiones HTTP y proxy a gRPC
- **Servidor gRPC (Tonic)**: Backend de alto rendimiento con tres servicios principales
- **Capa de Repositorio**: Persistencia de datos usando SQLx con soporte SQLite/PostgreSQL
- **Infraestructura de Eventos**: Registro de auditoría y publicación de webhooks
- **Motor de Políticas Cedar**: Lógica de autorización y evaluación de políticas

## 🚀 Inicio Rápido

### Prerequisitos

- Rust 1.83+ (Edición 2024)
- Una de: SQLite, PostgreSQL, o SurrealDB

### Instalación

```bash
git clone https://github.com/Rubentxu/hodei-verified-permissions.git
cd hodei-verified-permissions
```

### Ejecutar con SQLite (Por Defecto)

```bash
# Compilar con soporte SQLite
cargo build --release --features sqlite

# Establecer variables de entorno
export DATABASE_PROVIDER=sqlite
export DATABASE_URL=sqlite://hodei.db

# Ejecutar el servidor
./target/release/hodei-server
```

### Ejecutar con PostgreSQL

```bash
# Compilar con soporte PostgreSQL
cargo build --release --features postgres

# Establecer variables de entorno
export DATABASE_PROVIDER=postgres
export DATABASE_URL=postgresql://user:pass@localhost:5432/hodei

# Ejecutar el servidor
./target/release/hodei-server
```

### Ejecutar con SurrealDB

```bash
# Compilar con soporte SurrealDB
cargo build --release --features surreal

# Establecer variables de entorno
export DATABASE_PROVIDER=surreal
export DATABASE_URL=ws://localhost:8000

# Ejecutar el servidor
./target/release/hodei-server
```

## 🔄 Flujo de Autorización

```mermaid
sequenceDiagram
    participant Client as Cliente
    participant SDK
    participant Server as Servidor
    participant Cache as Caché
    participant Cedar
    participant DB

    Client->>SDK: is_authorized(principal, action, resource)
    SDK->>Server: Petición gRPC
    Server->>Cache: Verificar Caché PolicyStore
  
    alt Acierto de Caché
        Cache-->>Server: Devolver Políticas
    else Fallo de Caché
        Server->>DB: Cargar Políticas
        DB-->>Server: Devolver Políticas
        Server->>Cache: Actualizar Caché
    end
  
    Server->>Cedar: Evaluar(políticas, petición)
    Cedar-->>Server: Decisión (Allow/Deny)
    Server-->>SDK: Respuesta gRPC
    SDK-->>Client: Decisión
  
    Note over Server,Cache: ~100μs con caché
    Note over Server,DB: ~1-2ms sin caché
```

## 📖 Ejemplos de Uso

### Usando el SDK Cliente (Recomendado)

La forma más fácil de integrar Hodei Verified Permissions en tu aplicación es usando el SDK cliente gRPC.

#### Instalación

Añade a tu `Cargo.toml`:

```toml
[dependencies]
hodei-permissions-sdk = { git = "https://github.com/Rubentxu/hodei-verified-permissions", branch = "feature/hybrid-architecture" }
tokio = { version = "1.40", features = ["full"] }
```

#### Inicio Rápido - Verificación de Autorización

```rust
use hodei_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Conectar al servidor Hodei
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

## 🔧 Configuración

### Variables de Entorno

```bash
# Proveedor de base de datos (sqlite, postgres, surreal)
DATABASE_PROVIDER=sqlite

# URL de conexión a la base de datos
DATABASE_URL=sqlite://hodei.db

# Puerto del servidor gRPC (por defecto: 50051)
GRPC_PORT=50051

# Nivel de log (trace, debug, info, warn, error)
RUST_LOG=info

# TTL de caché JWKS en segundos (por defecto: 3600)
JWKS_CACHE_TTL=3600

# Intervalo de recarga de caché en segundos (por defecto: 300)
CACHE_RELOAD_INTERVAL=300
```

### Archivo de Configuración

Crea un archivo `config.toml`:

```toml
[server]
grpc_port = 50051
log_level = "info"

[database]
provider = "sqlite"
url = "sqlite://hodei.db"

[cache]
reload_interval = 300  # 5 minutos
jwks_ttl = 3600        # 1 hora

[metrics]
enabled = true
```

## 🧪 Testing

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar tests de integración
cargo test --features integration-tests

# Ejecutar tests E2E con contenedores (requiere Docker)
cargo test --features containers --test e2e_keycloak_integration_tests -- --ignored
cargo test --features containers --test e2e_zitadel_integration_tests -- --ignored
```

## 📊 Métricas

El servidor expone métricas de rendimiento:

```rust
// Obtener métricas actuales
let metrics = server.metrics();

println!("Aciertos de caché: {}", metrics.cache_hits());
println!("Fallos de caché: {}", metrics.cache_misses());
println!("Latencia promedio: {:?}", metrics.average_latency());
println!("Peticiones por segundo: {}", metrics.requests_per_second());
```

## 🏗️ Arquitectura del Proyecto

```
hodei-verified-permissions/
├── src/                    # Código fuente del servidor
│   ├── grpc/              # Implementación gRPC
│   ├── cache/             # Sistema de caché
│   ├── storage/           # Capa de almacenamiento
│   └── jwt/               # Validación JWT
├── sdk/                    # SDK cliente
│   ├── src/
│   │   ├── client.rs      # Cliente de autorización
│   │   ├── builders.rs    # Patrones builder
│   │   └── middleware/    # Middleware Axum/Tower
│   └── docs/              # Documentación del SDK
├── proto/                  # Definiciones Protocol Buffers
├── tests/                  # Tests E2E
│   ├── testcontainers/    # Wrappers de contenedores
│   ├── e2e_keycloak_*     # Tests Keycloak
│   └── e2e_zitadel_*      # Tests Zitadel
└── docs/                   # Documentación adicional
```

## 🤝 Contribuir

¡Las contribuciones son bienvenidas! Por favor:

1. Haz fork del repositorio
2. Crea tu rama de feature (`git checkout -b feature/caracteristica-increible`)
3. Haz commit de tus cambios (`git commit -m 'Añadir característica increíble'`)
4. Push a la rama (`git push origin feature/caracteristica-increible`)
5. Abre un Pull Request

## 📄 Licencia

Este proyecto está licenciado bajo la Licencia MIT - ver el archivo [LICENSE](LICENSE) para más detalles.

Copyright (c) 2025 Hodei Team

## 🙏 Agradecimientos

- [Cedar Policy Language](https://www.cedarpolicy.com/) - Lenguaje de políticas de autorización open-source de AWS
- [AWS Verified Permissions](https://aws.amazon.com/verified-permissions/) - Inspiración para la arquitectura
- [Rust Community](https://www.rust-lang.org/community) - Por las increíbles herramientas y librerías

## 📧 Contacto

- GitHub: [@Rubentxu](https://github.com/Rubentxu)
- Enlace del Proyecto: [https://github.com/Rubentxu/hodei-verified-permissions](https://github.com/Rubentxu/hodei-verified-permissions)

---

Para iniciar el entorno completo:
$ make dev-start              # Inicia Rust server + Next.js frontend

Para detener servicios:
$ make dev-stop               # Detiene todos los procesos

Para ver logs:
$ make dev-logs               # Logs del servidor Rust
$ make dev-logs-frontend      # Logs de Next.js

Para probar conectividad:
$ make dev-test               # Test de conexión gRPC

Para ejecutar tests:
$ cargo test --lib            # Tests unitarios
$ cargo test --test '*'       # Tests de integración

**Construido con ❤️ usando Rust y Cedar**
