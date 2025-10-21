# Plan de Expansión de Funcionalidad: OpenAPI → Cedar (SimpleRest), CLI y Metaprogramación Axum

## Objetivo
- Alinear el SDK Rust con la librería Cedar para Express.js en la generación de esquemas Cedar a partir de OpenAPI y su uso en runtime con mapeo SimpleRest.
- Añadir una opción productiva y mantenible para generar el esquema desde OpenAPI (o anotaciones), un CLI, y una integración fluida con `VerifiedPermissionsLayer`/`VerifiedPermissionsService`.
- Respetar arquitectura hexagonal, buenas prácticas en Rust y apoyar Axum.

## Alcance
- Generador de esquema Cedar (v4, opcional v2) desde OpenAPI 3.x.
- Creador de mapeos SimpleRest por método HTTP y template de ruta.
- CLI para `generate-schema` y `generate-policies`.
- Opción de metaprogramación con anotaciones en Axum para generar el esquema sin OpenAPI.
- Integración con middleware existente y ejemplos end-to-end.

---

## Epics, Historias de Usuario, Criterios de Aceptación y Tareas

### EPIC 1 — Generación de Schema Cedar desde OpenAPI
- **Historia**: Como desarrollador, quiero generar un esquema Cedar v4 desde un documento OpenAPI para que mis rutas HTTP queden mapeadas a acciones Cedar siguiendo el patrón SimpleRest.
- **Criterios de aceptación**:
  - Entrada: OpenAPI v3 JSON, `namespace`, `base_path` opcional.
  - Salida: `v4.cedarschema.json` válido con anotación `mappingType: "SimpleRest"`, acciones con `httpVerb` y `httpPathTemplate`.
  - Validaciones: namespace válido, verbos soportados, coherencia con `servers`/`base_path`.
- **Tareas**:
  - **[dominio]** Definir puerto `SchemaGenerationUseCase` en `sdk/src/schema/mod.rs`.
  - **[aplicación]** Implementar `SimpleRestSchemaGenerator` en `sdk/src/schema/service.rs`.
  - **[infra/openapi]** Parser con `openapiv3` en `sdk/src/schema/openapi_mapper.rs`.
  - **[infra/cedar]** (Opcional) Validador con `cedar-policy` en `sdk/src/schema/cedar_validation.rs`.
  - **[serialización]** Emisor de v4 (y v2 opcional) en `sdk/src/schema/serialization.rs`.

Características clave del patrón SimpleRest:
Mapeo automático de rutas HTTP:

Las rutas de OpenAPI (ej: /v1/files/{id}) se convierten en entidades tipo recurso en Cedar.

Los métodos HTTP (GET, POST, PUT, DELETE) se mapean a acciones (ej: readFile, createFile).

Estructura jerárquica:

Los recursos se organizan en una jerarquía lógica (ej: File::"v1/files/123").

Permite usar el control de acceso basado en atributos de Cedar para relaciones como pertenencia o herencia.

Ejemplo de mapeo:

OpenAPI:

yaml
GET /v1/files/{id}
POST /v1/files
Esquema Cedar generado:

cedar
actions:
[
{
name: "readFile",
appliesTo: Resource::"File"
},
{
name: "createFile",
appliesTo: Resource::"File"
}
]
entities:
[
{
name: "File",
shape: Record,
...
}
]
Ventajas:

Consistencia: Garantiza que todas las rutas sigan un mismo estándar de autorización.

Productividad: Elimina la necesidad de definir manualmente cada acción/resource en Cedar.

Integración con AWS: Se alinea con servicios como Amazon Verified Permissions.

Flujo de trabajo descrito en la frase:
Como desarrollador, tienes un documento OpenAPI que define tu API.

Usas una herramienta para generar automáticamente un esquema Cedar v4.

El patrón SimpleRest se aplica para:

Convertir GET /files → Acción readFile en el recurso File.

Convertir POST /files → Acción createFile.

Resultado: Las políticas Cedar pueden controlar acceso a las rutas usando este esquema generado.

Herramientas relacionadas:
AWS sugiere el uso de generadores de esquemas desde OpenAPI, aunque la implementación específica puede variar. Ejemplo conceptual:

bash
openapi-generator --input openapi.yaml --output cedar-schema.json --pattern simplerest
Si trabajas con Cedar, investiga en la documentación oficial de AWS o herramientas como Cedar Policy Builder para detalles técnicos exactos. 😊

### EPIC 2 — Mapeo SimpleRest y Emparejador en Runtime
- **Historia**: Como desarrollador, quiero cargar un mapeo SimpleRest por método y path para resolver rápidamente la acción Cedar asociada a cada request.
- **Criterios de aceptación**:
  - Construye un índice por método (`GET/POST/PUT/PATCH/DELETE`).
  - Empareja paths en O(log N)/O(1) por método con `matchit` (similar a path-to-regexp).
  - Expone `resolve(req) -> { action_name, resource_type, template }`.
- **Tareas**:
  - `sdk/src/schema/runtime_mapping.rs`: `SimpleRestMapping`, routers por método, normalización de plantillas (`/users/:id`).
  - Utilidades de normalización de paths/base_path.

### EPIC 3 — CLI de Soporte
- **Historias**:
  - Como desarrollador, quiero ejecutar `hodei-cli generate-schema` para obtener el esquema Cedar desde OpenAPI.
  - Como desarrollador, quiero ejecutar `hodei-cli generate-policies` para scaffold de políticas.
- **Criterios de aceptación**:
  - Comandos: `generate-schema --api-spec <file> --namespace <ns> [--base-path <path>]`.
  - Comando: `generate-policies --schema <file>` generando archivos en `policies/`.
- **Tareas**:
  - Crear binario `cli/` o `src/bin/hodei-cli.rs` con `clap`.
  - Invocar puerto `SchemaGenerationUseCase` y (opcional) generador de políticas.

### EPIC 4 — Anotaciones y Metaprogramación con Axum
- **Historias**:
  - Como desarrollador, quiero anotar rutas Axum `#[cedar_endpoint(...)]` para generar el mapeo sin mantener OpenAPI a mano.
  - Alternativa: usar `utoipa` para producir OpenAPI y reutilizar EPIC 1.
- **Criterios de aceptación**:
  - Captura método, ruta y metadatos (acción, resourceTypes opcional, contexto).
  - Build-time: generar artefacto (`OUT_DIR`) consumido por el generador de esquema.
- **Tareas**:
  - Crate proc-macro `cedar_axum_macros` con `#[cedar_endpoint(...)]`.
  - Registro en `inventory`/`linkme` de `RouteMeta`.
  - `build.rs` que transforma `RouteMeta[]` → OpenAPI mínimo o DTO directo → EPIC 1.
  - Alternativa Utoipa: documentar flujo y ejemplo.

### EPIC 5 — Integración con Middleware y Servicios
- **Historia**: Como desarrollador, quiero que `VerifiedPermissionsLayer` use el mapeo para construir `AuthorizationRequest` de forma consistente.
- **Criterios de aceptación**:
  - Soporta endpoints saltados (`skipped_endpoints`) por método y ruta.
  - Extrae contexto de `path/query/headers` similar a Express.
  - Integra con `sdk/src/authorization/engine.rs` y entidades (`sdk/src/entities/`).
- **Tareas**:
  - Extender `sdk/src/middleware/layer.rs` para cargar `SimpleRestMapping` al iniciar.
  - Añadir extracción de contexto y `AuthorizationRequestParts` actualizados.
  - Mantener compatibilidad con `AuthorizationRequestExtractor` personalizado.

### EPIC 6 — Validación, Pruebas y Ejemplos
- **Historias**:
  - Como mantenedor, quiero pruebas unitarias/integ para asegurar estabilidad del generador y del matcher.
  - Como usuario, quiero un ejemplo Axum end-to-end con políticas de muestra.
- **Criterios de aceptación**:
  - Cobertura de casos: parámetros path/query, base_path, métodos, skips, colisiones.
  - Ejemplo funcional en `examples/axum-simple-rest/` con README.
- **Tareas**:
  - Tests `sdk/src/schema/tests/` para mapeo y validaciones.
  - Ejemplo Axum con integración del Layer y CLI.

### EPIC 7 — Documentación
- **Historias**:
  - Como usuario, quiero guías claras para: generar esquema, integrar el middleware, usar CLI, y anotar rutas.
- **Criterios de aceptación**:
  - Documentos en `docs/` con pasos y referencias a archivos exactos.
- **Tareas**:
  - Actualizar `docs/RUST_ARCHITECTURE_IMPLEMENTATION.md`.
  - Nueva guía `docs/USO_CLI_Y_GENERACION_SCHEMA.md`.
  - Ejemplos de `utoipa` y `#[cedar_endpoint]`.

---

## Plan de Entregas (Sprints)

### ✅ Sprint 1 (Generador + CLI) - COMPLETADO
- **EPIC 1**: Generación de Schema Cedar desde OpenAPI ✅
- **EPIC 3**: CLI de Soporte ✅
- **Artefactos entregados**:
  - `sdk/src/schema/` - Módulo completo de generación
  - `cli/` - Binario `hodei-cli` con comandos `generate-schema` y `generate-policies`
  - `examples/openapi-sample.json` - Ejemplo funcional
  - `v4.cedarschema.json` - Schema generado y validado
  - `docs/SPRINT1_IMPLEMENTACION_COMPLETADA.md` - Documentación completa
- **Estado**: ✅ Funcional, testeado E2E, listo para producción

### 🔄 Sprint 2 (Runtime Mapping + Middleware) - PENDIENTE
- **EPIC 2**: Mapeo SimpleRest y Emparejador en Runtime
- **EPIC 5**: Integración con Middleware y Servicios (mínimo viable)
- **Artefactos a entregar**:
  - `sdk/src/schema/runtime_mapping.rs`
  - Extensiones en `sdk/src/middleware/layer.rs`
  - Extracción de contexto desde HTTP requests

### 📋 Sprint 3 (Metaprogramación Axum) - PENDIENTE
- **EPIC 4**: Anotaciones y Metaprogramación con Axum
- **Artefactos a entregar**:
  - POC macro `#[cedar_endpoint]` o integración Utoipa
  - `build.rs` para generación en compile-time
  - Registro de rutas con `inventory`/`linkme`

### 📋 Sprint 4 (Pruebas + Docs + Ejemplos) - PENDIENTE
- **EPIC 6**: Validación, Pruebas y Ejemplos
- **EPIC 7**: Documentación
- **Artefactos a entregar**:
  - Tests unitarios e integración
  - Ejemplo Axum end-to-end
  - Guías de uso completas

## Dependencias y Decisiones
- `openapiv3` para parseo.
- `matchit` para matcher de rutas.
- `clap` para CLI.
- `utoipa` (opcional) o proc-macro propia `cedar_axum_macros`.
- `cedar-policy` (opcional) para validación/format; si no hay paridad con WASM, validar estructuralmente y delegar errores al servidor de autorización.

## Riesgos y Mitigaciones
- **Paridad de validación Cedar**: empezar con validación estructural → añadir validación completa cuando esté disponible.
- **Complejidad de macros**: empezar con ruta Utoipa; macro propia en Sprint 3.
- **Compatibilidad esquemas v2**: priorizar v4; v2 sólo si requerido.

## Hecho vs. No Hecho (DoD)
- Generador produce v4 válido y tests cubren: verbos, base_path, path/query params.
- Runtime mapping resuelve acción correcta con latencia constante por método.
- CLI usable con documentación.
- Ejemplo Axum funcional con políticas de ejemplo.

## Métricas de Éxito
- Tiempo de generación del esquema (<1s para specs medianos).
- Tiempo de resolución de acción en runtime (~O(1) por método).
- Cobertura de tests >80% para generador/matcher.
- Ejemplo Axum validado end-to-end.
