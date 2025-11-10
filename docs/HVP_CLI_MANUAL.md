# 📘 Manual de Usuario: HVP CLI

**Herramienta de línea de comandos para Hodei Verified Permissions**

---

## 📋 Tabla de Contenidos

1. [Introducción](#introducción)
2. [Instalación](#instalación)
3. [Comandos Principales](#comandos-principales)
4. [Flujo de Trabajo Completo](#flujo-de-trabajo-completo)
5. [Contextos de Uso](#contextos-de-uso)
6. [Integración con SDK](#integración-con-sdk)
7. [Ejemplos Prácticos](#ejemplos-prácticos)
8. [Mejores Prácticas](#mejores-prácticas)
9. [Arquitectura Interna](#arquitectura-interna)
10. [Solución de Problemas](#solución-de-problemas)

---

## 🎯 Introducción

`hvp-cli` es la herramienta oficial de línea de comandos para **Hodei Verified Permissions** que automatiza la generación de configuración de autorización desde especificaciones OpenAPI.

### ¿Qué hace?
- Genera **schemas Cedar** desde OpenAPI
- Crea **policies de autorización** automáticamente
- Genera **configuración completa** para tu aplicación
- Integra con **Keycloak** para autenticación
- Asegura **principio de mínimo privilegio** por defecto

### ¿Por qué usarlo?
- **Ahorra tiempo**: De semanas a minutos en setup de autorización
- **Reduce errores**: Generación automática basada en estándares
- **Seguridad incorporada**: Policies seguras por defecto
- **CI/CD ready**: Integración perfecta con pipelines

---

## ⚡ Instalación

### Desde código fuente

```bash
# Clonar el repositorio
git clone https://github.com/rubentxu/hodei-verified-permissions.git
cd hodei-verified-permissions

# Construir el CLI
cargo build --release --bin hvp

# Instalar en tu PATH
cp target/release/hvp /usr/local/bin/
```

### Verificar instalación

```bash
hvp --version
hvp --help
```

---

## 🔧 Comandos Principales

### 1. `generate-schema` - Generar Schema Cedar

Genera un schema Cedar v4 desde una especificación OpenAPI.

```bash
hvp generate-schema \
  --api-spec openapi.json \
  --namespace PetStore \
  --base-path /api/v1 \
  --output ./config
```

**Parámetros:**
- `--api-spec`: Ruta al archivo OpenAPI (JSON)
- `--namespace`: Namespace Cedar para tu app
- `--base-path`: (Opcional) Base path de tu API
- `--output`: Directorio de salida

**Salida:**
```
config/
├── v4.cedarschema.json  # Schema para Cedar 4.x
└── v2.cedarschema.json  # Schema para Cedar 2.x (si aplica)
```

**Ejemplo de schema generado:**
```json
{
  "PetStore": {
    "entityTypes": {
      "User": { ... },
      "Pet": { ... }
    },
    "actions": {
      "listPets": {
        "appliesTo": {
          "principalTypes": ["User"],
          "resourceTypes": ["Pet"]
        }
      }
    }
  }
}
```

---

### 2. `generate-policies` - Generar Policies de Ejemplo

Crea policies de ejemplo desde un schema existente.

```bash
hvp generate-policies \
  --schema config/v4.cedarschema.json \
  --output ./config/policies
```

**Salida:**
```
config/policies/
├── policy_1.cedar  # Policy de admin
└── policy_2.cedar  # Policy de rol base
```

---

### 3. `generate-least-privilege` - Generar con Análisis de Seguridad

Genera policies con análisis de seguridad y reporte.

```bash
hvp generate-least-privilege \
  --spec openapi.json \
  --namespace PetStore \
  --roles admin,vet,customer \
  --mode strict \
  --output ./authorization
```

**Parámetros:**
- `--spec`: OpenAPI spec
- `--namespace`: Namespace Cedar
- `--roles`: Roles separados por comas
- `--mode`: `strict`, `moderate`, o `permissive`
- `--output`: Directorio de salida

**Salida:**
```
authorization/
├── v4.cedarschema.json
├── policies/
│   ├── policy_1.cedar
│   ├── policy_2.cedar
│   └── policy_3.cedar
└── security_report.md
```

**Reporte de seguridad incluye:**
- Puntuación de seguridad (0-100)
- Vulnerabilidades detectadas
- Recomendaciones de mejora
- Cobertura de policies

---

### 4. `generate-setup` - Generar Configuración Completa ⭐

**COMANDO RECOMENDADO** - Genera TODO en un solo paso.

```bash
hvp generate-setup \
  --spec openapi.json \
  --namespace PetStore \
  --app-name petstore \
  --keycloak-issuer http://localhost:8080/realms/demo \
  --keycloak-client-id petstore-client \
  --roles admin,vet,customer \
  --output ./config
```

**Parámetros:**
- `--spec`: OpenAPI spec
- `--namespace`: Namespace Cedar
- `--app-name`: Nombre de tu aplicación
- `--keycloak-issuer`: URL del issuer de Keycloak
- `--keycloak-client-id`: Client ID de Keycloak
- `--avp-endpoint`: Endpoint de AVP (default: localhost:50051)
- `--roles`: Roles a generar
- `--output`: Directorio de salida

**Salida completa:**
```
config/
├── schema/
│   └── v4.cedarschema.json
├── policies/
│   ├── admin.cedar
│   ├── vet.cedar
│   └── customer.cedar
├── setup.sh              # Script ejecutable
└── .env.example          # Configuración
```

---

## 🔄 Flujo de Trabajo Completo

### Escenario 1: Nueva Aplicación desde Cero

```bash
# Paso 1: Tener OpenAPI spec (crear o exportar)
# openapi.json debe tener x-cedar extensions

# Paso 2: Generar configuración completa
hvp generate-setup \
  --spec openapi.json \
  --namespace MyApp \
  --app-name myapp \
  --keycloak-issuer http://keycloak:8080/realms/myapp \
  --roles admin,user,viewer

# Paso 3: Revisar policies generadas
cat config/policies/admin.cedar
cat config/policies/user.cedar

# Paso 4: Ajustar si es necesario
# Editar policies manualmente para casos especiales

# Paso 5: Ejecutar setup en AVP
bash config/setup.sh

# Paso 6: Configurar aplicación
cp config/.env.example .env
# Editar .env con tus credenciales

# Paso 7: Integrar SDK en tu código
# Ver sección "Integración con SDK"

# Paso 8: Iniciar aplicación
cargo run
```

### Escenario 2: Añadir Nuevos Endpoints

```bash
# 1. Actualizar openapi.json con nuevos endpoints

# 2. Regenerar solo schema
hvp generate-schema \
  --api-spec openapi.json \
  --namespace MyApp \
  --output ./config/schema

# 3. Subir nuevo schema (sin sobreescribir policies)
bash -c "grpcurl -plaintext -d @ localhost:50051 \
  hodei.permissions.v1.AuthorizationControl/PutSchema" < config/schema/v4.cedarschema.json

# 4. Verificar que todo funciona
```

### Escenario 3: CI/CD Pipeline

```yaml
# .github/workflows/deploy.yml
name: Deploy Authorization

on:
  push:
    branches: [main]
    paths: ['openapi.json', 'config/**']

jobs:
  deploy-authorization:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install hvp-cli
        run: |
          cargo install --path hvp-cli

      - name: Generate authorization bundle
        run: |
          hvp generate-setup \
            --spec openapi.json \
            --namespace MyApp \
            --app-name myapp \
            --output ./deploy

      - name: Deploy to AVP
        env:
          AVP_ENDPOINT: ${{ secrets.AVP_ENDPOINT }}
        run: |
          bash deploy/setup.sh
```

---

## 🎨 Contextos de Uso

### Contexto 1: Startup Rápida (MVP)

**Problema**: Necesitas autorización funcional en 1 día.

**Solución**:
```bash
# Generar todo en 1 comando
hvp generate-setup --spec openapi.json --namespace App --app-name myapp

# Setup automático
bash config/setup.sh

# Listo para codificar
```

**Tiempo**: 15 minutos vs 1 semana manual.

---

### Contexto 2: Equipo Grande con Múltiples Roles

**Problema**: 5 roles (admin, manager, developer, tester, viewer) con permisos complejos.

**Solución**:
```bash
hvp generate-least-privilege \
  --spec openapi.json \
  --namespace EnterpriseApp \
  --roles admin,manager,developer,tester,viewer \
  --mode strict \
  --output ./auth

# Revisar reporte de seguridad
cat auth/security_report.md

# Ajustar policies manualmente para casos edge
vim auth/policies/policy_3.cedar
```

**Beneficio**: Policies consistentes y seguras por defecto.

---

### Contexto 3: Migración desde Otro Sistema

**Problema**: Migrando de Auth0/AWS Cognito a Hodei.

**Solución**:
```bash
# 1. Exportar roles y permisos actuales
# 2. Crear OpenAPI spec con x-cedar extensions
# 3. Generar configuración
hvp generate-setup \
  --spec legacy-api.json \
  --namespace LegacyApp \
  --app-name legacy \
  --roles role1,role2,role3

# 4. Mapear roles antiguos a nuevos
# 5. Probar con SDK antes de migrar
```

---

### Contexto 4: Microservicios

**Problema**: 10 microservicios, cada uno necesita su propia autorización.

**Solución**:
```bash
# Para cada servicio
cd services/user-service
hvp generate-setup \
  --spec src/openapi.json \
  --namespace UserService \
  --app-name user-service \
  --output ./config

cd services/order-service
hvp generate-setup \
  --spec src/openapi.json \
  --namespace OrderService \
  --app-name order-service \
  --output ./config
```

**Beneficio**: Consistencia y seguridad en toda la arquitectura.

---

## 🔌 Integración con SDK

### Paso 1: Añadir Dependencias

```toml
# Cargo.toml
[dependencies]
# Para verificar permisos (Data Plane)
verified-permissions-sdk = { version = "0.2", features = ["middleware"] }

# Para gestionar policies (Control Plane)
verified-permissions-sdk-admin = { version = "0.2" }

# Async runtime
tokio = { version = "1", features = ["full"] }
```

### Paso 2: Configurar Cliente

```rust
use verified_permissions_sdk::AuthorizationClient;
use verified_permissions_sdk_admin::HodeiAdmin;

#[tokio::main]
async fn main() -> Result<()> {
    // Cliente para verificar permisos (Data Plane)
    let auth_client = AuthorizationClient::connect("http://localhost:50051").await?;
    
    // Cliente para gestionar policies (Control Plane)
    let admin_client = HodeiAdmin::connect("http://localhost:50051").await?;
    
    Ok(())
}
```

### Paso 3: Verificar Permisos en tu API

```rust
use verified_permissions_sdk::{IsAuthorizedRequest, Entity};
use axum::{extract::Extension, http::StatusCode};

async fn create_pet_handler(
    Extension(client): Extension<AuthorizationClient>,
    Extension(user): Extension<User>,
) -> Result<Json<Pet>, StatusCode> {
    // Verificar permiso antes de ejecutar lógica
    let request = IsAuthorizedRequest {
        principal: Some(Entity::new("User", &user.id)),
        action: "Action::\"createPet\"".to_string(),
        resource: "Resource::\"PetStore\"".to_string(),
        context: serde_json::json!({
            "role": user.role,
        }),
    };
    
    let response = client
        .is_authorized("petstore-store", request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if response.decision != Decision::Allow {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // ... lógica de crear pet
    Ok(Json(pet))
}
```

### Paso 4: Usar Middleware (Axum)

```rust
use verified_permissions_sdk::middleware::VerifiedPermissionsLayer;

let layer = VerifiedPermissionsLayer::new(
    client,
    "petstore-store".to_string(),
    "petstore-identity".to_string(),
);

let app = Router::new()
    .route("/pets", post(create_pet))
    .layer(layer);
```

---

## 💡 Ejemplos Prácticos

### Ejemplo 1: Pet Store Completo

```bash
# 1. Estructura del proyecto
pet-store/
├── openapi.json
├── src/
│   └── main.rs
└── Cargo.toml

# 2. OpenAPI spec con x-cedar
# openapi.json
{
  "paths": {
    "/pets": {
      "post": {
        "operationId": "createPet",
        "x-cedar": {
          "appliesToResourceTypes": ["Pet"]
        }
      }
    }
  }
}

# 3. Generar configuración
hvp generate-setup \
  --spec openapi.json \
  --namespace PetStore \
  --app-name petstore \
  --roles admin,vet,customer

# 4. Salida generada
tree config/
# config/
# ├── schema/v4.cedarschema.json
# ├── policies/
# │   ├── admin.cedar
# │   ├── vet.cedar
# │   └── customer.cedar
# ├── setup.sh
# └── .env.example

# 5. Revisar policy de admin
cat config/policies/admin.cedar
# // POLÍTICA: ADMIN Access
# // Acceso completo para administradores
# permit(
#     principal in Group::"admin",
#     action in [
#         Action::"createPet",
#         Action::"listPets",
#         ...
#     ],
#     resource
# )
# when {
#     principal.role == "admin"
# };

# 6. Ejecutar setup
bash config/setup.sh

# 7. Integrar en código
# src/main.rs
use verified_permissions_sdk::AuthorizationClient;

#[tokio::main]
async fn main() {
    let client = AuthorizationClient::connect("http://localhost:50051")
        .await
        .unwrap();
    
    // Usar en handlers...
}
```

### Ejemplo 2: API REST con Múltiples Roles

```bash
# OpenAPI con múltiples endpoints
# - GET /users (listUsers)
# - POST /users (createUser)
# - DELETE /users/{id} (deleteUser)
# - GET /reports (generateReport)

# Generar con modo estricto
hvp generate-least-privilege \
  --spec api.json \
  --namespace MyApi \
  --roles admin,manager,employee \
  --mode strict \
  --output ./auth

# Resultado:
# - admin: Todas las acciones
# - manager: listUsers, createUser, generateReport
# - employee: Solo listUsers
# - Reporte de seguridad con puntuación
```

### Ejemplo 3: Integración en Makefile

```makefile
# Makefile
.PHONY: auth-generate auth-deploy auth-validate

auth-generate:
    hvp generate-setup \
        --spec openapi.json \
        --namespace MyApp \
        --app-name myapp \
        --output ./config

auth-deploy:
    bash config/setup.sh

auth-validate:
    @echo "Validando policies..."
    @grpcurl -plaintext localhost:50051 list
```

---

## 🏆 Mejores Prácticas

### Para Usuarios

1. **Siempre usar `generate-setup` para nuevos proyectos**
   - Es el comando más completo y seguro

2. **Revisar el reporte de seguridad**
   ```bash
   hvp generate-least-privilege ... --mode strict
   cat auth/security_report.md
   ```

3. **Versionar la configuración generada**
   ```bash
   git add config/
   git commit -m "feat: update authorization for new endpoints"
   ```

4. **Usar CI/CD para despliegue**
   - Automatizar `bash config/setup.sh` en pipelines

5. **Probar policies antes de deploy**
   ```bash
   # Usar SDK para probar permisos
   cargo test --test authorization_tests
   ```

### Para Developers del CLI

Si contribuyes al código de `hvp-cli`:

1. **Mantener la arquitectura de funciones reutilizables**
   - `read_and_validate_openapi()` → parsea una vez
   - `extract_actions_from_openapi()` → extrae con HashSet
   - `generate_policies_from_openapi()` → genera en paralelo

2. **Usar estructuras de datos eficientes**
   - `HashSet` para búsquedas O(1)
   - `HashMap` para caching
   - Evitar `Vec` con búsquedas lineales

3. **Precomputar reglas de mapeo**
   ```rust
   // Definir reglas una vez, reusar siempre
   let role_rules: HashMap<&str, Box<dyn Fn(&str) -> bool>> = [
       ("admin", Box::new(|_: &str| true)),
       ("vet", Box::new(|action: &str| {
           action.contains("list") || action.contains("view")
       })),
   ].iter().cloned().collect();
   ```

4. **Generación paralela de policies**
   - Usar `tokio::spawn` para tareas independientes
   - Reducir tiempo de ejecución en ~3x

5. **Ordenar outputs para consistencia**
   - Siempre ordenar actions y roles antes de generar
   - Facilita diff en version control

---

## 🔧 Arquitectura Interna

### Flujo de Datos

```
OpenAPI.json → Parser → HashSet<Actions> → Policy Generator → .cedar files
     ↓
Schema Generator → v4.cedarschema.json
     ↓
Setup Script Generator → setup.sh
```

### Componentes Principales

#### 1. **OpenAPI Parser** (`read_and_validate_openapi`)
- **Input**: Ruta a archivo OpenAPI
- **Output**: String validado + HashMap de paths
- **Complejidad**: O(n) donde n = número de paths
- **Optimización**: Valida JSON una vez, reusar resultado

#### 2. **Action Extractor** (`extract_actions_from_openapi`)
- **Input**: OpenApiSpec parseado
- **Output**: `HashSet<String>` de actions
- **Complejidad**: O(n) donde n = número de operaciones
- **Optimización**: Usa HashSet para eliminar duplicados y búsquedas O(1)

#### 3. **Role Mapper** (`precompute_actions_by_role`)
- **Input**: HashSet de actions + Vec de roles
- **Output**: `HashMap<String, HashSet<String>>` (rol → actions permitidas)
- **Complejidad**: O(r × n) donde r = roles, n = actions
- **Optimización**: Precomputa reglas de mapeo una vez

#### 4. **Policy Generator** (`generate_policies_from_openapi`)
- **Input**: Actions precomputadas por rol
- **Output**: Archivos .cedar en disco
- **Complejidad**: O(r) donde r = roles
- **Optimización**: Generación paralela con `tokio::spawn`

#### 5. **Schema Generator** (`generate_schema`)
- **Input**: OpenAPI spec + namespace
- **Output**: v4.cedarschema.json
- **Reutiliza**: Mismo parser que policy generator

### Algoritmos y Estructuras de Datos

#### HashSet vs Vec
```rust
// ❌ Ineficiente: O(n) por búsqueda
let actions: Vec<String> = vec![...];
actions.contains("createPet") // O(n)

// ✅ Eficiente: O(1) por búsqueda
let actions: HashSet<String> = HashSet::from([...]);
actions.contains("createPet") // O(1)
```

#### Precomputación de Reglas
```rust
// Reglas definidas una vez, ejecutadas múltiples veces
let role_rules = [
    ("admin", |action| true),
    ("vet", |action| action.contains("list") || action.contains("view")),
    ("customer", |action| action.contains("list")),
];

// Aplicar reglas a todas las actions
for (role, rule) in &role_rules {
    let permitted: HashSet<_> = all_actions
        .iter()
        .filter(|action| rule(action))
        .cloned()
        .collect();
}
```

#### Generación Paralela
```rust
// Generar todas las policies concurrentemente
let mut tasks = Vec::new();
for role in roles {
    let actions = actions_by_role.get(&role).unwrap().clone();
    let task = tokio::spawn(async move {
        generate_policy(&role, &actions)
    });
    tasks.push(task);
}

// Esperar a todas
for task in tasks {
    task.await??;
}
```

### Performance Characteristics

| Operación | Complejidad | Tiempo (1000 actions) | Memoria |
|-----------|-------------|----------------------|---------|
| Parse OpenAPI | O(n) | ~10ms | O(n) |
| Extraer actions | O(n) | ~5ms | O(n) |
| Precomputar roles | O(r × n) | ~50ms | O(r × n) |
| Generar policies (secuencial) | O(r) | ~100ms | O(r) |
| Generar policies (paralelo) | O(r) | ~30ms | O(r) |
| **Total** | **O(n + r×n)** | **~95ms** | **O(r×n)** |

Donde:
- n = número de actions en OpenAPI
- r = número de roles (typical: 3-10)

### Caching Interno

El CLI usa `lazy_static` para caching global:

```rust
lazy_static! {
    static ref ROLE_ACTION_CACHE: Mutex<HashMap<String, HashSet<String>>> =
        Mutex::new(HashMap::new());
}
```

Esto permite:
- Múltiples llamadas sin recalcular
- Mejora de performance en tests y CI/CD
- Memoria compartida segura entre threads

---

## 📊 Benchmarks

### Comparación: Antes vs Después

| Métrica | Antes | Después | Mejora |
|---------|-------|---------|--------|
| Parse OpenAPI | 3 veces | 1 vez | **66% menos** |
| Búsqueda de actions | O(n) | O(1) | **Instantáneo** |
| Generación policies | Secuencial | Paralela | **~3x más rápido** |
| Uso de memoria | Vec duplicados | HashSet único | **50% menos** |
| Tiempo total (ej: Pet Store) | 250ms | 95ms | **62% más rápido** |

### Ejemplo Real: Pet Store

```bash
# Con 15 endpoints y 3 roles
time hvp generate-setup --spec openapi.json --namespace PetStore --app-name petstore

# Antes: ~250ms
# Después: ~95ms
# Mejora: 2.6x más rápido
```

---

## 🔍 Debugging y Profiling

### Habilitar logs detallados

```bash
RUST_LOG=debug hvp generate-setup --spec openapi.json ...
```

Verás:
- Tiempo de parseo
- Número de actions extraídas
- Reglas aplicadas por rol
- Tiempo de generación de cada policy

### Perfilado con `cargo flamegraph`

```bash
# Instalar
cargo install flamegraph

# Ejecutar
cargo flamegraph --bin hvp -- generate-setup --spec openapi.json ...

# Ver resultado
open flamegraph.svg
```

---

## 🛠️ Extendiendo el CLI

### Añadir un Nuevo Comando

1. **Añadir a enum `Commands`**:
```rust
#[derive(Subcommand)]
enum Commands {
    // ... comandos existentes
    MyNewCommand {
        #[arg(long)]
        param: String,
    },
}
```

2. **Añadir case en `match`**:
```rust
match cli.command {
    // ... casos existentes
    Commands::MyNewCommand { param } => {
        my_new_function(param).await?;
    }
}
```

3. **Implementar función**:
```rust
async fn my_new_function(param: String) -> Result<()> {
    // Reutiliza helpers existentes
    let openapi = read_and_validate_openapi(...).await?;
    let actions = extract_actions_from_openapi(&openapi, ...);
    // ... lógica específica
    Ok(())
}
```

### Añadir un Nuevo Rol

Edita `precompute_actions_by_role()`:
```rust
let role_rules: HashMap<&str, Box<dyn Fn(&str) -> bool>> = [
    // ... roles existentes
    ("new-role", Box::new(|action: &str| {
        action.contains("list") && !action.contains("delete")
    })),
].iter().cloned().collect();
```

---

## 🚀 Próximas Mejoras

### En el roadmap

1. **Generación incremental**: Solo regenerar policies para endpoints modificados
2. **Validación de policies**: Verificar sintaxis Cedar antes de generar
3. **Soporte para OpenAPI 3.1**: JSON Schema compatibility
4. **Plugins**: Sistema de plugins para custom roles/generators
5. **Watch mode**: Auto-regenerar al cambiar openapi.json

---

## 🏆 Resumen de Mejores Prácticas para Developers

1. **Parsea una vez, reutiliza siempre**
2. **Usa HashSet para búsquedas frecuentes**
3. **Precomputa reglas de mapeo**
4. **Genera en paralelo cuando sea posible**
5. **Ordena outputs para consistencia**
6. **Cache resultados costosos**
7. **Profile antes de optimizar**
8. **Mantén funciones puras y testeables**

---

## 📚 Referencias

- **Código fuente**: [`hvp-cli/src/main.rs`](hvp-cli/src/main.rs:1)
- **OpenAPI Spec**: [examples/app/openapi.json](examples/app/openapi.json:1)
- **SDK Schema**: [`verified-permissions-sdk/src/schema/`](verified-permissions-sdk/src/schema/)
- **Cedar Policy Language**: https://cedar-policy.github.io/

---

<div align="center">

**¿Preguntas?** Abre un issue en GitHub o consulta la documentación completa en https://hodei.dev/docs

</div>

### ✅ Hacer

1. **Siempre usar `generate-setup` para nuevos proyectos**
   - Es el comando más completo y seguro

2. **Revisar el reporte de seguridad**
   ```bash
   hvp generate-least-privilege ... --mode strict
   cat auth/security_report.md
   ```

3. **Versionar la configuración generada**
   ```bash
   git add config/
   git commit -m "feat: update authorization for new endpoints"
   ```

4. **Usar CI/CD para despliegue**
   - Automatizar `bash config/setup.sh` en pipelines

5. **Probar policies antes de deploy**
   ```bash
   # Usar SDK para probar permisos
   cargo test --test authorization_tests
   ```

### ❌ No Hacer

1. **No editar manualmente el schema generado**
   - Regenera desde OpenAPI si necesitas cambios

2. **No commit secrets en `.env`**
   - Usar `.env.example` y variables de entorno

3. **No usar modo `permissive` en producción**
   - Usar `strict` para máxima seguridad

4. **No olvidar actualizar OpenAPI**
   - Siempre mantener OpenAPI sync con código

5. **No deployar sin revisar policies**
   - Siempre revisar policies generadas

---

## 🔍 Solución de Problemas

### Problema 1: "Invalid JSON in OpenAPI spec"

**Solución**:
```bash
# Validar OpenAPI
npx @apidevtools/swagger-cli validate openapi.json

# O usar jq
cat openapi.json | jq .
```

### Problema 2: "Base path not found in servers"

**Solución**:
```bash
# Especificar base path manualmente
hvp generate-schema \
  --api-spec openapi.json \
  --namespace MyApp \
  --base-path /api/v1  # <-- Añadir esto
```

### Problema 3: Setup falla con "Policy Store already exists"

**Solución**:
```bash
# El script ya maneja esto con "|| true"
# Para forzar recreación:
grpcurl -plaintext -d '{"policy_store_id": "my-store"}' \
  localhost:50051 hodei.permissions.v1.AuthorizationControl/DeletePolicyStore

bash config/setup.sh
```

### Problema 4: Permisos no funcionan como esperado

**Debug**:
```bash
# 1. Verificar policies cargadas
grpcurl -plaintext -d '{"policy_store_id": "my-store"}' \
  localhost:50051 hodei.permissions.v1.AuthorizationControl/ListPolicies

# 2. Probar autorización manual
grpcurl -plaintext -d '{
  "policy_store_id": "my-store",
  "principal": "User::\"alice\"",
  "action": "Action::\"createPet\"",
  "resource": "Resource::\"PetStore\""
}' localhost:50051 hodei.permissions.v1.AuthorizationData/IsAuthorized
```

---

## 📚 Referencias Rápidas

### Comandos más usados

```bash
# Setup rápido
hvp generate-setup --spec openapi.json --namespace App --app-name myapp

# Solo schema
hvp generate-schema --api-spec openapi.json --namespace App

# Solo policies
hvp generate-policies --schema schema.json --output ./policies

# Con análisis de seguridad
hvp generate-least-privilege --spec openapi.json --namespace App --mode strict
```

### Variables de entorno

```bash
# Para setup.sh
export AVP_HOST=localhost
export AVP_PORT=50051
export KEYCLOAK_ISSUER=http://keycloak:8080/realms/demo
export KEYCLOAK_CLIENT_ID=myapp-client
```

### Estructura de archivos recomendada

```
my-app/
├── openapi.json
├── config/
│   ├── schema/
│   │   └── v4.cedarschema.json
│   ├── policies/
│   │   ├── admin.cedar
│   │   └── user.cedar
│   ├── setup.sh
│   └── .env.example
├── src/
│   └── main.rs
├── Cargo.toml
└── Makefile
```

---

## 🎓 Resumen

`hvp-cli` transforma la configuración de autorización de un proceso manual de **días/semanas** a **minutos**, con:

- ✅ **Generación automática** desde OpenAPI
- ✅ **Seguridad incorporada** (principio de mínimo privilegio)
- ✅ **Integración con Keycloak**
- ✅ **Reportes de seguridad**
- ✅ **CI/CD ready**
- ✅ **Mejores prácticas** por defecto

**Flujo recomendado**: `generate-setup` → `setup.sh` → Integrar SDK → Deploy

¡Empieza ahora y olvídate de configurar autorización manualmente!