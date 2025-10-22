# 🏗️ CONTROL PLANE REFACTORING PLAN

**Fecha:** 22 de Octubre de 2025  
**Objetivo:** Refactorizar `control_plane.rs` (1062 líneas) siguiendo principios SOLID y Clean Code

---

## 📊 PROBLEMA ACTUAL

**Archivo:** `verified-permissions/api/src/grpc/control_plane.rs`
- **Líneas:** 1062
- **Responsabilidades:** 6 dominios diferentes
- **Violaciones SOLID:**
  - ❌ SRP: Múltiples responsabilidades en un solo archivo
  - ❌ OCP: Difícil de extender sin modificar
  - ❌ ISP: Interfaz monolítica

---

## 🎯 ESTRUCTURA PROPUESTA

```
api/src/grpc/control_plane/
├── mod.rs                      # Orchestrator (facade pattern)
├── policy_store_service.rs    # CRUD Policy Stores (~150 líneas)
├── schema_service.rs           # Schema management (~100 líneas)
├── policy_service.rs           # CRUD Policies (~300 líneas)
├── identity_source_service.rs # CRUD Identity Sources (~200 líneas)
├── policy_template_service.rs # CRUD Policy Templates (~150 líneas)
└── playground_service.rs       # Testing & Validation (~400 líneas)
```

**Total:** ~1300 líneas (distribuidas en 7 archivos)

---

## ✅ PRINCIPIOS APLICADOS

### Single Responsibility Principle (SRP)
Cada servicio tiene una única responsabilidad:
- `PolicyStoreService`: Gestión de Policy Stores
- `SchemaService`: Gestión de Schemas
- `PolicyService`: Gestión de Políticas (static + template-linked)
- `IdentitySourceService`: Gestión de Identity Sources
- `PolicyTemplateService`: Gestión de Policy Templates
- `PlaygroundService`: Testing y Validación

### Open/Closed Principle (OCP)
- Extensible: Nuevos servicios se añaden sin modificar existentes
- Cerrado: Cada servicio es independiente

### Liskov Substitution Principle (LSP)
- Todos los servicios implementan el mismo patrón
- Intercambiables en tests

### Interface Segregation Principle (ISP)
- Interfaces segregadas por dominio
- Clientes solo dependen de lo que necesitan

### Dependency Inversion Principle (DIP)
- Todos dependen de `RepositoryAdapter` (abstracción)
- No dependen de implementaciones concretas

---

## 📋 PLAN DE IMPLEMENTACIÓN

### Fase 1: Preparación (1h)
- [x] Analizar estructura actual
- [x] Diseñar nueva arquitectura
- [x] Crear prototipos de servicios
- [ ] Validar con proto definitions

### Fase 2: Implementación Gradual (4-6h)

#### Paso 1: PolicyStoreService
```rust
pub struct PolicyStoreService {
    repository: Arc<RepositoryAdapter>,
}

impl PolicyStoreService {
    pub async fn create(...) -> Result<...>
    pub async fn get(...) -> Result<...>
    pub async fn list(...) -> Result<...>
    pub async fn delete(...) -> Result<...>
}
```

#### Paso 2: SchemaService
```rust
pub struct SchemaService {
    repository: Arc<RepositoryAdapter>,
}

impl SchemaService {
    pub async fn put(...) -> Result<...>  // Con validación Cedar
    pub async fn get(...) -> Result<...>
}
```

#### Paso 3: PolicyService
```rust
pub struct PolicyService {
    repository: Arc<RepositoryAdapter>,
}

impl PolicyService {
    pub async fn create(...) -> Result<...>  // Static + Template-linked
    pub async fn get(...) -> Result<...>
    pub async fn update(...) -> Result<...>
    pub async fn delete(...) -> Result<...>
    pub async fn list(...) -> Result<...>
    
    // Private helpers
    fn instantiate_template(...) -> Result<...>
    fn validate_policy(...) -> Result<...>
}
```

#### Paso 4: IdentitySourceService
```rust
pub struct IdentitySourceService {
    repository: Arc<RepositoryAdapter>,
}

impl IdentitySourceService {
    pub async fn create(...) -> Result<...>
    pub async fn get(...) -> Result<...>
    pub async fn list(...) -> Result<...>
    pub async fn delete(...) -> Result<...>
}
```

#### Paso 5: PolicyTemplateService
```rust
pub struct PolicyTemplateService {
    repository: Arc<RepositoryAdapter>,
}

impl PolicyTemplateService {
    pub async fn create(...) -> Result<...>
    pub async fn get(...) -> Result<...>
    pub async fn list(...) -> Result<...>
    pub async fn delete(...) -> Result<...>
}
```

#### Paso 6: PlaygroundService
```rust
pub struct PlaygroundService {
    repository: Arc<RepositoryAdapter>,
}

impl PlaygroundService {
    pub async fn test_authorization(...) -> Result<...>
    pub async fn validate_policy(...) -> Result<...>
    
    // Private helpers
    fn load_schema(...) -> Result<...>
    fn parse_and_validate_policies(...) -> Result<...>
    fn parse_entity_uid(...) -> Result<...>
    fn parse_context(...) -> Result<...>
    fn parse_entities(...) -> Result<...>
}
```

#### Paso 7: Orchestrator (mod.rs)
```rust
pub struct AuthorizationControlService {
    policy_store_service: PolicyStoreService,
    schema_service: SchemaService,
    policy_service: PolicyService,
    identity_source_service: IdentitySourceService,
    policy_template_service: PolicyTemplateService,
    playground_service: PlaygroundService,
}

#[tonic::async_trait]
impl AuthorizationControl for AuthorizationControlService {
    // Delega a servicios especializados
    async fn create_policy_store(...) {
        self.policy_store_service.create(request).await
    }
    // ... etc
}
```

### Fase 3: Testing (2h)
- [ ] Verificar compilación
- [ ] Ejecutar tests E2E existentes
- [ ] Añadir unit tests por servicio
- [ ] Verificar no hay regresiones

### Fase 4: Documentación (1h)
- [ ] Documentar cada servicio
- [ ] Actualizar README
- [ ] Crear ejemplos de uso

---

## 🔧 BENEFICIOS

### Mantenibilidad
- ✅ Archivos más pequeños (<400 líneas)
- ✅ Responsabilidades claras
- ✅ Fácil de navegar

### Testabilidad
- ✅ Unit tests por servicio
- ✅ Mocking más fácil
- ✅ Tests independientes

### Extensibilidad
- ✅ Nuevos servicios sin modificar existentes
- ✅ Fácil añadir features
- ✅ Menos conflictos en git

### Legibilidad
- ✅ Nombres descriptivos
- ✅ Funciones pequeñas
- ✅ Lógica clara

---

## ⚠️ CONSIDERACIONES

### Compatibilidad
- ✅ API pública no cambia
- ✅ gRPC endpoints iguales
- ✅ Tests E2E siguen funcionando

### Performance
- ✅ Sin overhead adicional
- ✅ Mismo número de llamadas a BD
- ✅ Arc<Repository> compartido

### Riesgos
- ⚠️ Requiere validación exhaustiva con proto
- ⚠️ Posibles errores de compilación iniciales
- ⚠️ Necesita actualizar imports

---

## 📝 NOTAS DE IMPLEMENTACIÓN

### Patrón Común
Todos los servicios siguen el mismo patrón:

```rust
pub struct XxxService {
    repository: Arc<RepositoryAdapter>,
}

impl XxxService {
    pub fn new(repository: Arc<RepositoryAdapter>) -> Self {
        Self { repository }
    }
    
    pub async fn create(...) -> Result<Response<...>, Status> {
        // 1. Validar input
        // 2. Llamar repository
        // 3. Mapear a response
        // 4. Log y error handling
    }
    
    // Helpers privados
    fn validate_xxx(...) -> Result<...> { }
    fn parse_xxx(...) -> Result<...> { }
}
```

### Error Handling
```rust
.map_err(|e| {
    error!("Failed to xxx: {}", e);
    Status::internal(format!("Failed to xxx: {}", e))
})?
```

### Logging
```rust
info!("Creating xxx: {}", id);
error!("Failed to xxx: {}", e);
```

---

## 🎯 PRÓXIMOS PASOS

1. **Validar proto definitions** contra servicios propuestos
2. **Implementar servicio por servicio** (incremental)
3. **Compilar y testear** después de cada servicio
4. **Commit incremental** para facilitar rollback
5. **Documentar** cambios

---

## 📚 REFERENCIAS

- [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
- [Clean Code](https://www.amazon.com/Clean-Code-Handbook-Software-Craftsmanship/dp/0132350882)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [tonic Best Practices](https://github.com/hyperium/tonic/blob/master/examples/README.md)

---

**Estado:** 📋 PLAN DOCUMENTADO - PENDIENTE DE IMPLEMENTACIÓN

**Razón de postponer:** Requiere validación exhaustiva con proto definitions para evitar errores de compilación. Es mejor hacerlo en una sesión dedicada con tiempo suficiente.

**Recomendación:** Implementar en próxima sesión, servicio por servicio, con tests después de cada uno.
