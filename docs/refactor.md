# Plan de Reorganización Arquitectural - Hexagonal + SOLID + Rust Best Practices

## 🚀 ESTADO: PLANIFICADO / EN PROGRESO

**Objetivo:** Implementar una arquitectura hexagonal robusta y mantenible siguiendo principios SOLID y mejores prácticas de Rust.

## Estado Inicial y Objetivo del Proyecto

### Estructura Actual (Punto de Partida)
El proyecto actualmente se encuentra en una estructura monolítica o con una separación incipiente. El objetivo es migrar a una arquitectura multicrate bien definida dentro del directorio `verified-permissions`.

### Objetivo: Estructura Multicrate
El workspace de Cargo se definirá en el directorio raíz `verified-permissions/`, incluyendo todos los crates del core del servicio, pero **excluyendo el `sdk`**, que se gestionará de forma independiente.

```
verified-permissions/  # <- Raíz del workspace
├── shared/           # Tipos comunes y utilidades
├── domain/           # Núcleo puro del negocio  
├── application/      # Casos de uso y coordinación
├── infrastructure/   # Adaptadores externos
├── api/              # Interfaces gRPC/CLI
└── main/             # Binario principal de la aplicación

sdk/                  # <- EXCLUIDO del workspace principal
└── ...               # Cliente SDK, con su propio Cargo.toml
```

### Problemas Arquitecturales a Resolver
1.  **Acoplamiento**: Capas mezcladas, dependencias incorrectas.
2.  **Dominio Impuro**: Lógica de negocio contaminada con detalles de infraestructura.
3.  **Dificultad de Testeo**: Código difícil de probar de forma aislada.
4.  **Baja Mantenibilidad**: Cambios en una capa afectan a otras inesperadamente.

## Arquitectura Propuesta - Hexagonal + Multicrate

### Capas Hexagonales

#### 1. **Domain Layer** (`domain/`)
- **Entidades**: `Policy`, `PolicyStore`, `Schema`, `Principal`, `Resource`, `Action`
- **Value Objects**: `PolicyId`, `PolicyStoreId`, `CedarPolicy`, `AuthorizationDecision`
- **Domain Services**: `AuthorizationEvaluator`, `PolicyValidator`
- **Domain Events**: Eventos de dominio puros
- **Repository Traits**: Interfaces abstractas para persistencia
- **Domain Errors**: Solo errores de negocio

#### 2. **Application Layer** (`application/`)
- **Use Cases / Commands**: `AuthorizeRequest`, `CreatePolicyStore`, `UpdatePolicy`
- **Queries**: `GetPolicyStore`, `ListPolicies`
- **Application Services**: Coordinan dominio + infraestructura
- **DTOs**: Para comunicación entre capas
- **Application Events**: Eventos de aplicación

#### 3. **Infrastructure Layer** (`infrastructure/`)
- **Adapters Primarios**: gRPC handlers, CLI commands
- **Adapters Secundarios**: Database repos, Cache impl, JWT validator
- **External Services**: Integraciones externas
- **Configuration**: Configuración de infraestructura

#### 4. **API Layer** (`api/`)
- **gRPC Services**: Handlers que delegan a application
- **CLI Interface**: Comandos que usan application services
- **HTTP API**: Futura expansión

#### 5. **Shared** (`shared/`)
- **Common Types**: UUIDs, Timestamps, Results
- **Infrastructure Traits**: Interfaces comunes
- **Utilities**: Helpers compartidos

### Principios SOLID Aplicados

#### Single Responsibility
- Cada crate tiene una responsabilidad única.
- Cada módulo dentro de crates tiene propósito claro.
- Servicios pequeños y enfocados.

#### Open/Closed
- Traits para extensión sin modificación.
- Implementaciones concretas separadas de interfaces.

#### Liskov Substitution
- Implementaciones de traits intercambiables.
- Contratos claros en traits.

#### Interface Segregation
- Traits pequeños y específicos.
- Interfaces por rol, no por cliente.

#### Dependency Inversion
- Dependencias apuntan hacia el centro (dominio).
- Interfaces en dominio, implementaciones en infraestructura.

### Rust Best Practices Aplicadas

#### Ownership y Borrowing
- Clear ownership en domain entities.
- Rc/Arc solo donde necesario.
- Lifetimes apropiados.

#### Error Handling
- `thiserror` para errores específicos por capa.
- `anyhow` solo en boundaries.
- Domain errors separados de infrastructure errors.

#### Async/Await
- Proper async traits.
- Non-blocking operations.
- Tokio ecosystem.

#### Testing
- Unit tests en domain (puros).
- Integration tests en boundaries.
- Mock implementations para infraestructura.

## Migración Paso a Paso

#### Fase 1: Análisis y Diseño
- [ ] Analizar estructura actual
- [ ] Identificar problemas y dependencias
- [ ] Diseñar arquitectura multicrate definitiva
- [ ] Documentar plan y obtener aprobación

#### Fase 2: Crear Crates Base
- [ ] Crear crate `shared` - Tipos comunes (EntityId, Timestamp, etc.)
- [ ] Crear crate `domain` - Núcleo puro del negocio
- [ ] Mover tipos comunes existentes a `shared`

#### Fase 3: Domain Layer
- [ ] Definir entidades del dominio (PolicyStore, Policy, Schema, PolicyTemplate)
- [ ] Crear value objects (PolicyId, CedarPolicy, Principal, Action, Resource)
- [ ] Implementar domain services (AuthorizationService, PolicyValidationService)
- [ ] Definir repository traits (PolicyRepository, SchemaRepository, etc.)
- [ ] Mover/crear domain errors (DomainError enum)
- [ ] Crear domain events (PolicyStoreCreatedEvent, etc.)

#### Fase 4: Application Layer
- [ ] Crear crate `application`
- [ ] Definir use cases (CreatePolicyStoreUseCase, EvaluateAuthorizationUseCase)
- [ ] Implementar application services (PolicyStoreService, AuthorizationService)
- [ ] Crear DTOs (CreatePolicyStoreRequest, AuthorizationResponse, etc.)
- [ ] Implementar commands y queries

#### Fase 5: Infrastructure Layer
- [ ] Crear crate `infrastructure`
- [ ] Implementar SQLite repository completo con schema initialization
- [ ] Crear stubs para PostgreSQL y SurrealDB repositories
- [ ] Implementar cache básico (InMemoryCache)
- [ ] Crear external services (JwtValidator, HttpClient)
- [ ] Implementar configuración completa (DatabaseConfig, ServerConfig, etc.)

#### Fase 6: API Layer
- [ ] Crear crate `api`
- [ ] Implementar request handlers (PolicyStoreHandler, AuthorizationHandler)
- [ ] Crear gRPC services preparados (AuthorizationDataServiceImpl)
- [ ] Implementar CLI completa con comandos (CliRunner, policy-store commands)

#### Fase 7: Integration y Testing
- [ ] Configurar el workspace en `verified-permissions/Cargo.toml`
- [ ] Actualizar dependencias entre crates
- [ ] Resolver todos los imports entre crates
- [ ] Actualizar `main.rs` para usar nueva arquitectura
- [ ] Actualizar `bin/cli.rs` para usar nueva arquitectura
- [ ] Crear tests unitarios completos para domain layer
- [ ] Verificar compilación de todos los crates

#### Fase 8: Verificación Final
- [ ] Verificar que la arquitectura hexagonal esté implementada correctamente
- [ ] Validar que los principios SOLID se hayan aplicado
- [ ] Revisar que se sigan las mejores prácticas de Rust
- [ ] Ejecutar suite de tests unitarios y de integración
- [ ] Realizar pruebas de compilación y ejecución end-to-end

---

## 📊 Plan de Verificación y Métricas

### 🧪 Tests Unitarios - 📋 POR EJECUTAR
```bash
cargo test -p hodei-domain
# Objetivo: Crear y ejecutar tests unitarios puros para la lógica de negocio.
```

### 🏗️ Compilación Multicrate - 📋 POR VERIFICAR
```bash
cargo check --workspace
# Objetivo: Asegurar que todos los crates del workspace compilen sin errores.
```

### 📏 Cobertura Arquitectural - 🎯 OBJETIVO
- **Capas Hexagonales**: 100% implementadas
- **Principios SOLID**: 100% aplicados
- **Rust Best Practices**: 100% seguidas
- **Separación de Responsabilidades**: 100% lograda
- **Estructura Multicrate**: 100% implementada

### 🔄 Flujo de Dependencias - 🎯 OBJETIVO
```
api → application → domain ← infrastructure
         ↓              ↑
      shared ←──────────┘
```

### 🎯 Validación de Arquitectura - 📋 POR VALIDAR
- [ ] **Domain Layer**: Verificar que es puro, sin dependencias externas.
- [ ] **Application Layer**: Comprobar que coordina use cases correctamente.
- [ ] **Infrastructure Layer**: Asegurar que adapta servicios externos.
- [ ] **API Layer**: Validar que expone interfaces externas.
- [ ] **Shared Layer**: Comprobar que los tipos comunes son reutilizables.
- [ ] **Main Binary**: Verificar que usa la nueva arquitectura.

---

## 🎯 Beneficios Esperados

1.  **Mantenibilidad**: Arquitectura clara permitirá cambios independientes por capa.
2.  **Testabilidad**: Tests unitarios puros en domain + estructura para integración.
3.  **Flexibilidad**: Infrastructure adapters intercambiables sin afectar negocio.
4.  **Escalabilidad**: Varios crates permiten desarrollo paralelo por equipos.
5.  **Claridad**: Responsabilidades perfectamente separadas por capa/crate.
6.  **Type Safety**: Value Objects con validación prevendrán errores en runtime.
7.  **Error Handling**: Errores específicos por capa con `thiserror`.
8.  **Performance**: Zero-cost abstractions con traits de Rust.

### Riesgos y Mitigaciones - ⚠️ IDENTIFICADOS

1.  **Complejidad Inicial**: Mayor número de crates.
- *Mitigación*: Usar workspace management de Cargo y documentar claramente las dependencias.

2.  **Dependencias Circulares**: Imports entre crates.
- *Mitigación*: Imponer dependencias unidireccionales estrictas y usar herramientas de análisis estático.

3.  **Performance**: Más indirección.
- *Mitigación*: Confiar en las zero-cost abstractions de Rust y realizar benchmarks si es necesario.

4.  **Migración**: Mucho código que mover.
- *Mitigación*: Realizar la migración de forma incremental, fase por fase, manteniendo la funcionalidad existente operativa el mayor tiempo posible.

---

## 🚀 Próximos Pasos Inmediatos

1.  **Iniciar Fase 1**: Realizar el análisis detallado del código base actual para mapear componentes a las nuevas capas.
2.  **Configurar Workspace**: Crear el archivo `verified-permissions/Cargo.toml` definiendo los miembros del workspace.
3.  **Crear Crates Vacíos**: Generar la estructura base de `shared`, `domain`, `application`, etc., para empezar a mover el código.