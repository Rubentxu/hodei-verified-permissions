# Guía de Uso: CLI y Generación de Schemas Cedar

Esta guía explica cómo usar el CLI `hodei-cli` para generar esquemas Cedar desde especificaciones OpenAPI y crear políticas de ejemplo.

## Tabla de Contenidos

- [Instalación](#instalación)
- [Generación de Schema desde OpenAPI](#generación-de-schema-desde-openapi)
- [Generación de Políticas de Ejemplo](#generación-de-políticas-de-ejemplo)
- [Patrón SimpleRest](#patrón-simplerest)
- [Extensiones OpenAPI](#extensiones-openapi)
- [Casos de Uso Comunes](#casos-de-uso-comunes)
- [Troubleshooting](#troubleshooting)

## Instalación

### Compilar desde el código fuente

```bash
cd hodei-verified-permissions/cli
cargo build --release
```

El binario estará disponible en `target/release/hodei-cli`.

### Añadir al PATH (opcional)

```bash
# Linux/macOS
export PATH="$PATH:$(pwd)/target/release"

# O instalar globalmente
cargo install --path .
```

## Generación de Schema desde OpenAPI

### Comando Básico

```bash
hodei-cli generate-schema \
  --api-spec path/to/openapi.json \
  --namespace MyApp \
  --output ./schemas
```

### Parámetros

| Parámetro | Requerido | Descripción | Ejemplo |
|-----------|-----------|-------------|---------|
| `--api-spec` | ✅ | Ruta al archivo OpenAPI v3 (JSON) | `openapi.json` |
| `--namespace` | ✅ | Namespace Cedar para tu aplicación | `DocumentApp` |
| `--base-path` | ⚠️ | Base path de la API (requerido si hay múltiples servers) | `/v1` |
| `--output` / `-o` | ❌ | Directorio de salida (default: `.`) | `./schemas` |

### Ejemplo Completo

```bash
hodei-cli generate-schema \
  --api-spec examples/openapi-sample.json \
  --namespace DocumentApp \
  --base-path /v1 \
  --output ./generated
```

**Salida:**
```
INFO Reading OpenAPI spec from: examples/openapi-sample.json
INFO Generating Cedar schema with namespace: DocumentApp
INFO ✓ Cedar schema v4 generated: ./generated/v4.cedarschema.json
INFO   Namespace: DocumentApp
INFO   Mapping type: SimpleRest
INFO   Actions: 6
INFO   Entity types: 4
INFO   Base path: /v1
```

### Archivos Generados

- **`v4.cedarschema.json`**: Schema Cedar en formato v4 (compatible con Cedar 4.x)
- Contiene:
  - Entity types (User, UserGroup, Application, + custom types)
  - Actions con anotaciones `httpVerb` y `httpPathTemplate`
  - Context definitions (pathParameters, queryStringParameters)

## Generación de Políticas de Ejemplo

### Comando Básico

```bash
hodei-cli generate-policies \
  --schema v4.cedarschema.json \
  --output ./policies
```

### Parámetros

| Parámetro | Requerido | Descripción | Ejemplo |
|-----------|-----------|-------------|---------|
| `--schema` | ✅ | Ruta al schema Cedar v4 | `v4.cedarschema.json` |
| `--output` / `-o` | ❌ | Directorio de salida (default: `./policies`) | `./policies` |

### Ejemplo Completo

```bash
hodei-cli generate-policies \
  --schema ./generated/v4.cedarschema.json \
  --output ./policies
```

**Salida:**
```
INFO Reading Cedar schema from: ./generated/v4.cedarschema.json
INFO Found 6 actions in schema
INFO ✓ Cedar policy generated: ./policies/policy_1.cedar
INFO ✓ Cedar policy generated: ./policies/policy_2.cedar
```

### Políticas Generadas

#### `policy_1.cedar` - Política Admin
```cedar
// Allows admin usergroup access to everything
permit(
    principal in DocumentApp::UserGroup::"admin",
    action,
    resource
);
```

#### `policy_2.cedar` - Política Basada en Roles
```cedar
// Allows more granular user group control, change actions as needed
permit(
    principal in DocumentApp::UserGroup::"ENTER_THE_USER_GROUP_HERE",
    action in [
        DocumentApp::Action::"createDocument",
        DocumentApp::Action::"deleteDocument",
        DocumentApp::Action::"getDocument",
        DocumentApp::Action::"listDocuments",
        DocumentApp::Action::"shareDocument",
        DocumentApp::Action::"updateDocument"
    ],
    resource
);
```

## Patrón SimpleRest

El generador implementa el patrón **SimpleRest** que mapea automáticamente rutas HTTP a entidades Cedar.

### Mapeo de Conceptos

| OpenAPI | Cedar | Ejemplo |
|---------|-------|---------|
| **Método HTTP + Path** | Action | `GET /documents/{id}` → `getDocument` |
| **operationId** | Action name | `getDocument` (preferido sobre método+path) |
| **Path template** | Annotation `httpPathTemplate` | `/v1/documents/{id}` |
| **Método HTTP** | Annotation `httpVerb` | `get` |
| **Path parameters** | `context.pathParameters` | `{id: String}` |
| **Query parameters** | `context.queryStringParameters` | `{limit: Long}` |
| **Resource (default)** | Entity `Application` | Para endpoints sin recurso específico |
| **Resource (custom)** | Entity custom | `Document`, `File`, etc. |

### Ejemplo de Transformación

**OpenAPI:**
```json
{
  "paths": {
    "/documents/{id}": {
      "get": {
        "operationId": "getDocument",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "required": true,
            "schema": { "type": "string" }
          }
        ]
      }
    }
  }
}
```

**Cedar Schema Generado:**
```json
{
  "DocumentApp": {
    "actions": {
      "getDocument": {
        "annotations": {
          "httpVerb": "get",
          "httpPathTemplate": "/v1/documents/{id}"
        },
        "appliesTo": {
          "principalTypes": ["User"],
          "resourceTypes": ["Application"],
          "context": {
            "type": "Record",
            "attributes": {
              "pathParameters": {
                "type": "Record",
                "attributes": {
                  "id": { "type": "String", "required": true }
                }
              }
            }
          }
        }
      }
    }
  }
}
```

## Extensiones OpenAPI

### `x-cedar.appliesToResourceTypes`

Permite especificar tipos de recursos personalizados para una operación.

**Ejemplo:**
```json
{
  "paths": {
    "/documents/{id}": {
      "get": {
        "operationId": "getDocument",
        "x-cedar": {
          "appliesToResourceTypes": ["Document"]
        }
      }
    }
  }
}
```

**Resultado:**
- La acción `getDocument` aplicará a recursos de tipo `Document` en lugar de `Application`
- El entity type `Document` se añadirá automáticamente al schema

## Casos de Uso Comunes

### 1. API con Base Path

Si tu API tiene un base path (ej: `/api/v1`):

```bash
hodei-cli generate-schema \
  --api-spec openapi.json \
  --namespace MyApp \
  --base-path /api/v1
```

### 2. API sin Base Path

Si tu API no tiene base path o está en la raíz:

```bash
hodei-cli generate-schema \
  --api-spec openapi.json \
  --namespace MyApp
```

### 3. Múltiples Servers en OpenAPI

Si tu OpenAPI tiene múltiples servers, **debes** especificar `--base-path`:

```json
{
  "servers": [
    { "url": "https://api.example.com/v1" },
    { "url": "https://api.example.com/v2" }
  ]
}
```

```bash
hodei-cli generate-schema \
  --api-spec openapi.json \
  --namespace MyApp \
  --base-path /v1
```

### 4. Workflow Completo

```bash
# 1. Generar schema
hodei-cli generate-schema \
  --api-spec openapi.json \
  --namespace MyApp \
  --output ./cedar

# 2. Generar políticas de ejemplo
hodei-cli generate-policies \
  --schema ./cedar/v4.cedarschema.json \
  --output ./cedar/policies

# 3. Revisar y personalizar políticas
vim ./cedar/policies/policy_2.cedar

# 4. Usar el schema con tu servicio de autorización
# (copiar a tu proyecto, cargar en AVP, etc.)
```

## Troubleshooting

### Error: "Invalid namespace format"

**Causa:** El namespace no cumple con el formato Cedar.

**Solución:** El namespace debe:
- Empezar con letra o `_`
- Contener solo letras, números y `_`
- Usar `::` para separar componentes
- No usar palabras reservadas: `if`, `in`, `is`, `__cedar`

**Ejemplos válidos:**
- `MyApp`
- `Company::ProductName`
- `_internal::Service`

**Ejemplos inválidos:**
- `123App` (empieza con número)
- `My-App` (contiene `-`)
- `if` (palabra reservada)

### Error: "Base path does not match any server"

**Causa:** El `--base-path` especificado no coincide con ningún server en el OpenAPI spec.

**Solución:** Verifica los servers en tu OpenAPI:
```json
{
  "servers": [
    { "url": "https://api.example.com/v1" }
  ]
}
```

El base path debe ser `/v1` en este caso.

### Error: "API spec has multiple servers. Base path parameter required"

**Causa:** Tu OpenAPI tiene múltiples servers y no especificaste `--base-path`.

**Solución:** Añade `--base-path` con el path correcto:
```bash
hodei-cli generate-schema \
  --api-spec openapi.json \
  --namespace MyApp \
  --base-path /v1
```

### Error: "Failed to parse OpenAPI spec"

**Causa:** El archivo OpenAPI no es JSON válido o no cumple con OpenAPI 3.x.

**Solución:**
1. Valida tu OpenAPI con herramientas online (Swagger Editor)
2. Asegúrate de que sea formato JSON (no YAML)
3. Verifica que sea OpenAPI 3.x (no 2.x/Swagger)

### Warning: "No actions found in schema"

**Causa:** Tu OpenAPI no tiene paths o todas las operaciones fueron ignoradas.

**Solución:**
- Verifica que tu OpenAPI tenga paths definidos
- Asegúrate de usar métodos HTTP soportados: GET, POST, PUT, PATCH, DELETE

## Logging Avanzado

Para ver más detalles durante la ejecución:

```bash
# Debug completo
RUST_LOG=debug hodei-cli generate-schema --api-spec openapi.json --namespace MyApp

# Solo info
RUST_LOG=info hodei-cli generate-schema --api-spec openapi.json --namespace MyApp

# Sin logs
RUST_LOG=error hodei-cli generate-schema --api-spec openapi.json --namespace MyApp
```

## Integración con CI/CD

### GitHub Actions

```yaml
name: Generate Cedar Schema

on:
  push:
    paths:
      - 'api/openapi.json'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build hodei-cli
        run: cargo build --release --manifest-path cli/Cargo.toml
      
      - name: Generate Cedar schema
        run: |
          ./target/release/hodei-cli generate-schema \
            --api-spec api/openapi.json \
            --namespace MyApp \
            --output ./cedar
      
      - name: Commit schema
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add cedar/
          git commit -m "Update Cedar schema" || true
          git push
```

## Referencias

- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [OpenAPI Specification](https://swagger.io/specification/)
- [AWS Verified Permissions](https://aws.amazon.com/verified-permissions/)
- [Documentación del Proyecto](../README.md)
