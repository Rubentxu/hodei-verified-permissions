# Plan de Implementación: Hodei AVP Clone

**Versión**: 1.0  
**Fecha**: 2025-10-19  
**Rama**: `feature/avp-clone-identity-middleware`  
**Base**: Estado actual con SDK 100% implementado

---

## Estado Actual (Punto de Partida)

### ✅ Completado
- **Servidor gRPC**: 100% (22/22 RPCs)
- **SDK Cliente**: 100% (22/22 RPCs + 1 helper)
- **Tests**: 95.5% (21/22 con cobertura)
- **Proto**: Consolidado en fuente única
- **Arquitectura**: Hexagonal implementada

### ⚠️ Implementado pero Requiere Mejoras
- **IsAuthorizedWithToken**: Servidor implementado, falta test e2e
- **Identity Sources**: CRUD completo en servidor y SDK, falta integración JWT real
- **Policy Templates**: CRUD completo, falta helper en SDK

### ❌ Pendiente de Implementación
- **Middleware Axum/Tower**: No existe
- **SDK Builders**: Patrón builder para requests
- **JWKS Caching**: Validación JWT sin caché
- **Claims Mapping avanzado**: Mapeo básico implementado

---

## Objetivos del Plan

Transformar Hodei Verified Permissions en un **clon funcional de AWS Verified Permissions** con:

1. **Integración profunda con IdPs** (OIDC/Cognito)
2. **SDK ergonómico** estilo AWS
3. **Middleware declarativo** estilo Express.js
4. **Experiencia de desarrollador superior**

---

## ÉPICA 18: Identity Sources - Integración Profunda con IdPs

**Objetivo**: Centralizar validación JWT y mapeo de identidades en el servidor

**Proveedores de Identidad Soportados**:
- ✅ **OIDC Genérico**: Cualquier proveedor compatible con OpenID Connect
- 🎯 **Keycloak**: Open-source IdP con soporte completo de roles y grupos
- 🎯 **Zitadel**: Modern cloud-native IdP con RBAC avanzado
- ✅ **AWS Cognito**: Integración nativa con User Pools
- ✅ **Okta**: Enterprise IdP
- ✅ **Auth0**: Developer-friendly IdP

### Fase 1: JWKS Caching y Validación Mejorada

**HU 18.1: Implementar JWKS Cache con Auto-refresh**

**Archivos a modificar**:
- `verified-permissions/infrastructure/src/jwt/validator.rs`
- Nuevo: `verified-permissions/infrastructure/src/jwt/jwks_cache.rs`

**Tareas**:
1. Crear `JwksCache` con TTL configurable
2. Implementar auto-discovery de `.well-known/openid-configuration`
3. Agregar refresh automático de claves
4. Métricas de hit/miss rate

**Criterios de aceptación**:
- [ ] Cache con TTL de 1 hora por defecto
- [ ] Auto-discovery funcional
- [ ] Refresh en background cada 30 min
- [ ] Tests unitarios con mock HTTP

**Estimación**: 1 día

---

**HU 18.2: Mejorar Claims Mapping Configuration**

**Archivos a modificar**:
- `verified-permissions/infrastructure/src/jwt/claims_mapper.rs`
- `proto/authorization.proto` (ya tiene ClaimsMappingConfiguration)

**Tareas**:
1. Implementar mapeo de arrays a parents (grupos → roles)
2. Soportar claims anidados (dot notation: `user.department`)
3. Transformaciones de valores (uppercase, lowercase)
4. Validación de claims requeridos

**Criterios de aceptación**:
- [ ] Mapeo `groups` claim → entidades padre
- [ ] Dot notation para claims anidados
- [ ] Transformaciones básicas
- [ ] Tests con JWTs reales

**Estimación**: 2 días

---

**HU 18.3: Test E2E IsAuthorizedWithToken**

**Archivos a crear**:
- `tests/e2e_jwt_authorization_tests.rs`
- `tests/fixtures/jwt_tokens.rs`

**Tareas**:
1. Crear helper para generar JWTs de prueba
2. Test con OIDC mock server
3. Test con diferentes claims mappings
4. Test de validación de errores

**Criterios de aceptación**:
- [ ] Test con JWT válido → Allow
- [ ] Test con JWT expirado → Error
- [ ] Test con firma inválida → Error
- [ ] Test con grupos mapeados a roles

**Estimación**: 1 día

**Total Fase 1**: 4 días

---

### Fase 2: Integración con IdPs Específicos

**HU 18.4: Implementar Keycloak Integration**

**Archivos a crear**:
- `verified-permissions/infrastructure/src/jwt/providers/keycloak.rs`
- `verified-permissions/infrastructure/src/jwt/providers/mod.rs`

**Tareas**:
1. Detectar issuer de Keycloak (formato: `https://{host}/realms/{realm}`)
2. Mapeo de claims específicos de Keycloak:
   - `realm_access.roles` → roles de realm
   - `resource_access.{client}.roles` → roles de cliente
   - `groups` → grupos de Keycloak
3. Soporte para token introspection endpoint (opcional)
4. Configuración de realm y client_id

**Criterios de aceptación**:
- [ ] Auto-detección de Keycloak por issuer
- [ ] Mapeo de roles de realm y cliente
- [ ] Mapeo de grupos
- [ ] Tests con Keycloak testcontainer
- [ ] Documentación de configuración

**Ejemplo de configuración**:
```rust
IdentitySourceConfiguration::Keycloak {
    issuer: "https://keycloak.example.com/realms/myapp",
    client_id: "verified-permissions",
    realm_roles_claim: "realm_access.roles",
    client_roles_claim: "resource_access.verified-permissions.roles",
    groups_claim: "groups",
}
```

**Estimación**: 2 días

---

**HU 18.5: Implementar Zitadel Integration**

**Archivos a crear**:
- `verified-permissions/infrastructure/src/jwt/providers/zitadel.rs`

**Tareas**:
1. Detectar issuer de Zitadel (formato: `https://{instance}.zitadel.cloud`)
2. Mapeo de claims específicos de Zitadel:
   - `urn:zitadel:iam:org:project:roles` → roles de proyecto
   - `urn:zitadel:iam:user:resourceowner:name` → organización
3. Soporte para project_id y organization_id
4. Validación de audience específica de Zitadel

**Criterios de aceptación**:
- [ ] Auto-detección de Zitadel por issuer
- [ ] Mapeo de roles de proyecto
- [ ] Mapeo de organización
- [ ] Tests con Zitadel mock
- [ ] Documentación de configuración

**Ejemplo de configuración**:
```rust
IdentitySourceConfiguration::Zitadel {
    issuer: "https://myinstance.zitadel.cloud",
    project_id: "123456789",
    client_id: "verified-permissions@project",
    roles_claim: "urn:zitadel:iam:org:project:roles",
}
```

**Estimación**: 2 días

---

**HU 18.6: Implementar Cognito User Pool Integration**

**Archivos a crear**:
- `verified-permissions/infrastructure/src/jwt/providers/cognito.rs`

**Tareas**:
1. Parsear `user_pool_arn` para extraer región y pool ID
2. Construir JWKS URI de Cognito
3. Validar claim `cognito:groups`
4. Soportar múltiples client_ids

**Criterios de aceptación**:
- [ ] Validación de ARN de Cognito
- [ ] JWKS URI correcto
- [ ] Grupos de Cognito mapeados
- [ ] Tests con tokens de Cognito mock

**Estimación**: 1 día

**Total Fase 2**: 5 días

---

### Fase 2.5: Tests E2E con IdPs Reales (NUEVO)

**HU 18.7: E2E Tests con Keycloak Testcontainer**

**Archivos a crear**:
- `tests/e2e_keycloak_integration_tests.rs`
- `tests/testcontainers/keycloak_container.rs`
- `tests/fixtures/keycloak_setup.rs`

**Tareas**:
1. Crear Keycloak testcontainer wrapper
2. Configurar realm, client, users y roles automáticamente
3. Generar tokens JWT reales desde Keycloak
4. Test flujo completo: token → validación → autorización
5. Test con realm roles y client roles
6. Test con grupos de Keycloak

**Criterios de aceptación**:
- [ ] Keycloak container inicia correctamente
- [ ] Realm y client se crean automáticamente
- [ ] Usuarios con roles se crean vía Admin API
- [ ] Tokens JWT reales se obtienen vía OAuth2
- [ ] IsAuthorizedWithToken funciona con token real
- [ ] Roles mapeados correctamente a Cedar entities
- [ ] Test completo de Allow y Deny

**Configuración Keycloak**:
```rust
KeycloakContainer::new()
    .with_realm("test-realm")
    .with_client("test-app", "secret")
    .with_user("admin", "password", vec!["admin", "user"])
    .with_user("viewer", "password", vec!["viewer"])
    .start()
```

**Estimación**: 1.5 días

---

**HU 18.8: E2E Tests con Zitadel Testcontainer**

**Archivos a crear**:
- `tests/e2e_zitadel_integration_tests.rs`
- `tests/testcontainers/zitadel_container.rs`
- `tests/fixtures/zitadel_setup.rs`

**Tareas**:
1. Crear Zitadel testcontainer wrapper
2. Configurar project, application y users
3. Generar tokens JWT reales desde Zitadel
4. Test flujo completo con URN claims
5. Test con project roles
6. Test con organization context

**Criterios de aceptación**:
- [ ] Zitadel container inicia correctamente
- [ ] Project y application se crean vía API
- [ ] Usuarios con roles se crean automáticamente
- [ ] Tokens JWT con URN claims se obtienen
- [ ] Project roles se mapean correctamente
- [ ] Organization attribute se extrae
- [ ] Test completo de autorización

**Configuración Zitadel**:
```rust
ZitadelContainer::new()
    .with_project("test-project")
    .with_application("test-app")
    .with_user("developer", vec!["developer", "admin"])
    .with_organization("test-org")
    .start()
```

**Estimación**: 1.5 días

**Total Fase 2.5**: 3 días

---

## ÉPICA 11: SDK Ergonómico Mejorado

**Objetivo**: SDK idiomático con builders y mejor UX

### Fase 3: Request Builders

**HU 11.1: Implementar Builder Pattern para Requests**

**Archivos a crear**:
- `sdk/src/builders/mod.rs`
- `sdk/src/builders/is_authorized.rs`
- `sdk/src/builders/is_authorized_with_token.rs`

**Tareas**:
1. Builder para `IsAuthorizedRequest`
2. Builder para `IsAuthorizedWithTokenRequest`
3. Builder para `CreatePolicyRequest`
4. Métodos fluent API

**Ejemplo de API objetivo**:
```rust
let request = IsAuthorizedRequest::builder()
    .policy_store_id("store-123")
    .principal("User", "alice")
    .action("Action", "read")
    .resource("Document", "doc-456")
    .context(json!({"ip": "192.168.1.1"}))
    .entity("User", "alice", |e| {
        e.attribute("department", "Engineering")
         .parent("Role", "Developer")
    })
    .build()?;
```

**Criterios de aceptación**:
- [ ] Builders para todos los request types
- [ ] API fluida y ergonómica
- [ ] Validación en build time
- [ ] Documentación completa

**Estimación**: 3 días

---

**HU 11.2: Mejorar Error Handling**

**Archivos a modificar**:
- `sdk/src/error.rs`

**Tareas**:
1. Errores específicos por tipo (AuthError, ValidationError, etc.)
2. Contexto rico en errores
3. Conversión desde tonic::Status
4. Display impl amigable

**Criterios de aceptación**:
- [ ] Tipos de error específicos
- [ ] Mensajes descriptivos
- [ ] Source chain completo
- [ ] Tests de error handling

**Estimación**: 1 día

**Total Fase 3**: 4 días

---

## ÉPICA 22: Middleware Axum/Tower

**Objetivo**: Middleware declarativo para integración web

### Fase 4: Core Middleware

**HU 22.1: Crear Trait AuthorizationRequestExtractor**

**Archivos a crear**:
- Nuevo crate: `crates/hodei-middleware/`
- `crates/hodei-middleware/src/extractor.rs`
- `crates/hodei-middleware/src/lib.rs`

**Estructura**:
```
crates/hodei-middleware/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── extractor.rs      # Trait público
│   ├── builder.rs        # Builder del middleware
│   ├── layer.rs          # Tower Layer impl
│   ├── service.rs        # Tower Service impl
│   └── error.rs          # Errores del middleware
└── examples/
    └── axum_app.rs       # Ejemplo completo
```

**Tareas**:
1. Definir trait `AuthorizationRequestExtractor`
2. Tipos `AuthorizationRequestParts`
3. Documentación y ejemplos

**Criterios de aceptación**:
- [ ] Trait async con error asociado
- [ ] Tipos bien documentados
- [ ] Ejemplo básico funcional

**Estimación**: 1 día

---

**HU 22.2: Implementar Middleware Builder**

**Archivos a crear**:
- `crates/hodei-middleware/src/builder.rs`

**Tareas**:
1. Builder con validación
2. Configuración de policy store
3. Inyección de cliente SDK
4. Inyección de extractor

**API objetivo**:
```rust
let middleware = VerifiedPermissionsMiddleware::builder()
    .client(auth_client)
    .policy_store_id("store-123")
    .extractor(MyExtractor)
    .build();
```

**Criterios de aceptación**:
- [ ] Builder type-safe
- [ ] Validación de configuración
- [ ] Documentación completa

**Estimación**: 1 día

---

**HU 22.3: Implementar Tower Layer y Service**

**Archivos a crear**:
- `crates/hodei-middleware/src/layer.rs`
- `crates/hodei-middleware/src/service.rs`

**Tareas**:
1. Implementar `tower::Layer`
2. Implementar `tower::Service`
3. Extracción de JWT del header
4. Llamada a `is_authorized_with_token`
5. Manejo de decisiones (Allow/Deny)

**Criterios de aceptación**:
- [ ] Compatible con Axum
- [ ] Extracción automática de Bearer token
- [ ] 403 en Deny, continuar en Allow
- [ ] Insertar decisión en extensions

**Estimación**: 2 días

---

**HU 22.4: Crear Ejemplo Completo Axum**

**Archivos a crear**:
- `crates/hodei-middleware/examples/axum_app.rs`
- `crates/hodei-middleware/examples/extractors/api_extractor.rs`

**Tareas**:
1. App Axum completa con rutas
2. Implementación de extractor
3. Handlers que usan decisión
4. README con instrucciones

**Criterios de aceptación**:
- [ ] Ejemplo ejecutable
- [ ] Múltiples rutas protegidas
- [ ] Logging de decisiones
- [ ] README detallado

**Estimación**: 1 día

**Total Fase 4**: 5 días

---

## Fase 5: Documentación y Ejemplos

**HU 23.1: Documentación Completa**

**Archivos a crear**:
- `docs/identity-sources.md`
- `docs/sdk-guide.md`
- `docs/middleware-guide.md`
- `examples/complete-app/`

**Tareas**:
1. Guía de Identity Sources
2. Guía del SDK con builders
3. Guía del middleware
4. Aplicación ejemplo completa

**Estimación**: 2 días

---

**HU 23.2: Tests de Integración Completos**

**Archivos a crear**:
- `tests/integration_jwt_flow.rs`
- `tests/integration_middleware.rs`

**Tareas**:
1. Test flujo completo JWT
2. Test middleware con Axum
3. Test con diferentes IdPs
4. Test de errores

**Estimación**: 2 días

**Total Fase 5**: 4 días

---

## Resumen de Estimaciones

| Épica | Fase | Días | Prioridad |
|-------|------|------|-----------|
| **18** | Fase 1: JWKS Cache | 4 | ALTA |
| **18** | Fase 2: IdP Integration (Keycloak, Zitadel, Cognito) | 5 | ALTA |
| **18** | Fase 2.5: E2E Tests con IdPs Reales | 3 | ALTA |
| **11** | Fase 3: SDK Builders | 4 | ALTA |
| **22** | Fase 4: Middleware | 5 | ALTA |
| - | Fase 5: Docs | 4 | MEDIA |
| **TOTAL** | | **25 días** | |

---

## Orden de Implementación Recomendado

### Sprint 1 (5 días) - Fundamentos
1. ✅ HU 18.1: JWKS Cache (1 día)
2. ✅ HU 18.2: Claims Mapping (2 días)
3. ✅ HU 18.3: Test E2E JWT (1 día)
4. ✅ HU 11.2: Error Handling (1 día)

### Sprint 2 (5 días) - IdP Integration
5. ✅ HU 18.4: Keycloak Integration (2 días)
6. ✅ HU 18.5: Zitadel Integration (2 días)
7. ✅ HU 18.6: Cognito Integration (1 día)

### Sprint 2.5 (3 días) - E2E Tests con IdPs Reales
8. 🔄 HU 18.7: E2E Keycloak Testcontainer (1.5 días)
9. 🔄 HU 18.8: E2E Zitadel Testcontainer (1.5 días)

### Sprint 3 (5 días) - Middleware
7. ✅ HU 22.1: Extractor Trait (1 día)
8. ✅ HU 22.2: Builder (1 día)
9. ✅ HU 22.3: Layer/Service (2 días)
10. ✅ HU 22.4: Ejemplo Axum (1 día)

### Sprint 4 (4 días) - Finalización
11. ✅ HU 23.1: Documentación (2 días)
12. ✅ HU 23.2: Tests Integración (2 días)

---

## Criterios de Éxito del Proyecto

### Funcionales
- [ ] Identity Sources OIDC completamente funcional
- [ ] Cognito User Pool integration funcional
- [ ] SDK con builders ergonómicos
- [ ] Middleware Axum listo para producción
- [ ] Documentación completa

### No Funcionales
- [ ] Latencia IsAuthorizedWithToken < 50ms (con cache)
- [ ] JWKS cache hit rate > 95%
- [ ] Cobertura de tests > 90%
- [ ] Ejemplos ejecutables y documentados
- [ ] API idiomática Rust

### Compatibilidad AWS AVP
- [ ] Misma estructura de Identity Sources
- [ ] Mismo formato de Claims Mapping
- [ ] API similar en SDK
- [ ] Comportamiento equivalente

---

## Dependencias y Riesgos

### Dependencias Técnicas
- `jsonwebtoken` para validación JWT
- `reqwest` para JWKS fetching
- `tower` y `axum` para middleware
- `moka` o `cached` para JWKS cache
- `testcontainers` para tests E2E
- `testcontainers-modules` para Keycloak
- Docker para ejecutar containers en tests

### Riesgos Identificados
1. **Complejidad JWKS**: Manejo de rotación de claves
   - Mitigación: Cache con refresh automático
2. **Performance JWT**: Validación puede ser lenta
   - Mitigación: Cache agresivo de claves públicas
3. **Compatibilidad Axum**: Cambios en Tower API
   - Mitigación: Tests de integración extensivos

---

## Próximos Pasos Inmediatos

1. **Crear estructura del middleware crate**
   ```bash
   cargo new --lib crates/hodei-middleware
   ```

2. **Actualizar Cargo.toml del workspace**
   ```toml
   members = [
       "crates/hodei-middleware",
       # ... otros
   ]
   ```

3. **Implementar HU 18.1** (JWKS Cache)
   - Comenzar con la funcionalidad más crítica
   - Base para todo el flujo JWT

4. **Tests primero**
   - TDD para JWKS cache
   - Mocks de OIDC provider

---

## Ejemplos de Integración con IdPs

### Keycloak
```yaml
# docker-compose.yml
services:
  keycloak:
    image: quay.io/keycloak/keycloak:latest
    environment:
      KEYCLOAK_ADMIN: admin
      KEYCLOAK_ADMIN_PASSWORD: admin
    ports:
      - "8080:8080"
    command: start-dev
```

```rust
// Configuración en Hodei
let identity_source = CreateIdentitySourceRequest {
    policy_store_id: "store-123",
    configuration: Some(IdentitySourceConfiguration::Keycloak {
        issuer: "http://localhost:8080/realms/myapp",
        client_id: "verified-permissions",
        realm_roles_claim: "realm_access.roles",
        client_roles_claim: "resource_access.verified-permissions.roles",
        groups_claim: "groups",
    }),
    claims_mapping: Some(ClaimsMappingConfiguration {
        principal_id_claim: "sub",
        group_claim: Some("groups"),
        attribute_mappings: vec![
            ("email", "email"),
            ("name", "name"),
            ("department", "department"),
        ],
    }),
};
```

### Zitadel
```rust
let identity_source = CreateIdentitySourceRequest {
    policy_store_id: "store-123",
    configuration: Some(IdentitySourceConfiguration::Zitadel {
        issuer: "https://myinstance.zitadel.cloud",
        project_id: "123456789",
        client_id: "verified-permissions@project",
        roles_claim: "urn:zitadel:iam:org:project:roles",
    }),
    claims_mapping: Some(ClaimsMappingConfiguration {
        principal_id_claim: "sub",
        group_claim: Some("urn:zitadel:iam:org:project:roles"),
        attribute_mappings: vec![
            ("email", "email"),
            ("org", "urn:zitadel:iam:user:resourceowner:name"),
        ],
    }),
};
```

### AWS Cognito
```rust
let identity_source = CreateIdentitySourceRequest {
    policy_store_id: "store-123",
    configuration: Some(IdentitySourceConfiguration::Cognito {
        user_pool_arn: "arn:aws:cognito-idp:us-east-1:123456789:userpool/us-east-1_ABC123",
        client_ids: vec!["app-client-id"],
    }),
    claims_mapping: Some(ClaimsMappingConfiguration {
        principal_id_claim: "sub",
        group_claim: Some("cognito:groups"),
        attribute_mappings: vec![
            ("email", "email"),
            ("department", "custom:department"),
        ],
    }),
};
```

---

## Notas de Implementación

### Patrón de Desarrollo
1. **Test First**: Escribir tests antes de implementar
2. **Iterativo**: Implementar por HU completas
3. **Documentar**: README por cada feature
4. **Ejemplos**: Código ejecutable por cada API
5. **Testcontainers**: Usar contenedores reales para tests de integración

### Estándares de Código
- Documentación Rustdoc completa
- Tests unitarios + integración
- Ejemplos en cada módulo público
- Error handling exhaustivo

### Review Checklist
- [ ] Tests pasan
- [ ] Documentación actualizada
- [ ] Ejemplos funcionan
- [ ] Sin warnings del compilador
- [ ] Clippy clean
- [ ] Formato con rustfmt
