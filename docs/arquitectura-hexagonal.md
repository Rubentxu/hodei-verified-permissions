# Arquitectura Hexagonal - Hodei Verified Permissions

## Visión General

Refactorización del servicio de autorización siguiendo arquitectura hexagonal (Ports & Adapters) para lograr:
- **Independencia de frameworks**: El dominio no depende de gRPC, SQLite, etc.
- **Testabilidad**: Fácil crear mocks de adaptadores
- **Flexibilidad**: Cambiar tecnologías sin afectar el dominio
- **Claridad**: Separación clara de responsabilidades

## Capas de la Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                    ADAPTERS (Driving)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  gRPC API    │  │   REST API   │  │   CLI        │     │
│  │  (Tonic)     │  │   (Axum)     │  │              │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │             │
├─────────┼──────────────────┼──────────────────┼─────────────┤
│         ▼                  ▼                  ▼             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              APPLICATION LAYER                       │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │           USE CASES (Ports)                    │  │  │
│  │  │  - CreatePolicyStoreUseCase                    │  │  │
│  │  │  - CreatePolicyUseCase                         │  │  │
│  │  │  - IsAuthorizedUseCase                         │  │  │
│  │  │  - PutSchemaUseCase                            │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    DOMAIN LAYER                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  ENTITIES                                            │  │
│  │  - PolicyStore                                       │  │
│  │  - Policy                                            │  │
│  │  - Schema                                            │  │
│  │  - AuthorizationRequest                             │  │
│  │  - AuthorizationResponse                            │  │
│  ├──────────────────────────────────────────────────────┤  │
│  │  VALUE OBJECTS                                       │  │
│  │  - PolicyStoreId                                     │  │
│  │  - PolicyId                                          │  │
│  │  - EntityIdentifier                                  │  │
│  │  - Decision (Allow/Deny)                            │  │
│  ├──────────────────────────────────────────────────────┤  │
│  │  DOMAIN SERVICES                                     │  │
│  │  - PolicyValidator                                   │  │
│  │  - AuthorizationEvaluator (Cedar wrapper)           │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────┐  │
│  │           PORTS (Interfaces)                         │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  Driving Ports (Input - Use Cases)             │  │  │
│  │  │  - PolicyStoreUseCasePort                       │  │  │
│  │  │  - PolicyUseCasePort                            │  │  │
│  │  │  - AuthorizationUseCasePort                     │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  Driven Ports (Output - Repositories)          │  │  │
│  │  │  - PolicyStoreRepositoryPort                    │  │  │
│  │  │  - PolicyRepositoryPort                         │  │  │
│  │  │  - SchemaRepositoryPort                         │  │  │
│  │  │  - AuthorizationEvaluatorPort                   │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                 ADAPTERS (Driven)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  SQLite      │  │  PostgreSQL  │  │  In-Memory   │     │
│  │  Adapter     │  │  Adapter     │  │  Adapter     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐                       │
│  │  Cedar       │  │  Mock        │                       │
│  │  Evaluator   │  │  Evaluator   │                       │
│  └──────────────┘  └──────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## Estructura de Directorios

```
src/
├── lib.rs                          # Re-exports públicos
├── main.rs                         # Composición de la aplicación
│
├── domain/                         # CAPA DE DOMINIO
│   ├── mod.rs
│   ├── entities/                   # Entidades de dominio
│   │   ├── mod.rs
│   │   ├── policy_store.rs
│   │   ├── policy.rs
│   │   ├── schema.rs
│   │   └── authorization.rs
│   ├── value_objects/              # Value Objects
│   │   ├── mod.rs
│   │   ├── policy_store_id.rs
│   │   ├── policy_id.rs
│   │   ├── entity_identifier.rs
│   │   └── decision.rs
│   ├── services/                   # Servicios de dominio
│   │   ├── mod.rs
│   │   ├── policy_validator.rs
│   │   └── authorization_evaluator.rs
│   └── errors.rs                   # Errores de dominio
│
├── application/                    # CAPA DE APLICACIÓN
│   ├── mod.rs
│   ├── ports/                      # Ports (Interfaces)
│   │   ├── mod.rs
│   │   ├── driving/                # Input ports (use cases)
│   │   │   ├── mod.rs
│   │   │   ├── policy_store_use_case_port.rs
│   │   │   ├── policy_use_case_port.rs
│   │   │   └── authorization_use_case_port.rs
│   │   └── driven/                 # Output ports (repositories)
│   │       ├── mod.rs
│   │       ├── policy_store_repository_port.rs
│   │       ├── policy_repository_port.rs
│   │       ├── schema_repository_port.rs
│   │       └── authorization_evaluator_port.rs
│   ├── use_cases/                  # Implementación de use cases
│   │   ├── mod.rs
│   │   ├── policy_store/
│   │   │   ├── mod.rs
│   │   │   ├── create_policy_store.rs
│   │   │   ├── get_policy_store.rs
│   │   │   ├── list_policy_stores.rs
│   │   │   └── delete_policy_store.rs
│   │   ├── schema/
│   │   │   ├── mod.rs
│   │   │   ├── put_schema.rs
│   │   │   └── get_schema.rs
│   │   ├── policy/
│   │   │   ├── mod.rs
│   │   │   ├── create_policy.rs
│   │   │   ├── get_policy.rs
│   │   │   ├── update_policy.rs
│   │   │   ├── delete_policy.rs
│   │   │   └── list_policies.rs
│   │   └── authorization/
│   │       ├── mod.rs
│   │       ├── is_authorized.rs
│   │       └── batch_is_authorized.rs
│   └── dto/                        # DTOs de aplicación
│       ├── mod.rs
│       ├── policy_store_dto.rs
│       ├── policy_dto.rs
│       ├── schema_dto.rs
│       └── authorization_dto.rs
│
└── infrastructure/                 # CAPA DE INFRAESTRUCTURA
    ├── mod.rs
    ├── adapters/                   # Adaptadores
    │   ├── mod.rs
    │   ├── driving/                # Adapters de entrada
    │   │   ├── mod.rs
    │   │   ├── grpc/
    │   │   │   ├── mod.rs
    │   │   │   ├── server.rs
    │   │   │   ├── control_plane_adapter.rs
    │   │   │   ├── data_plane_adapter.rs
    │   │   │   └── mappers/        # Conversión proto <-> domain
    │   │   │       ├── mod.rs
    │   │   │       ├── policy_store_mapper.rs
    │   │   │       ├── policy_mapper.rs
    │   │   │       └── authorization_mapper.rs
    │   │   └── rest/               # Futuro: REST API
    │   │       └── mod.rs
    │   └── driven/                 # Adapters de salida
    │       ├── mod.rs
    │       ├── persistence/
    │       │   ├── mod.rs
    │       │   ├── sqlite/
    │       │   │   ├── mod.rs
    │       │   │   ├── policy_store_repository.rs
    │       │   │   ├── policy_repository.rs
    │       │   │   ├── schema_repository.rs
    │       │   │   ├── models.rs
    │       │   │   └── mappers.rs
    │       │   ├── postgres/       # Futuro
    │       │   │   └── mod.rs
    │       │   └── in_memory/      # Para tests
    │       │       └── mod.rs
    │       └── authorization/
    │           ├── mod.rs
    │           ├── cedar_evaluator.rs
    │           └── mock_evaluator.rs
    └── config/                     # Configuración
        ├── mod.rs
        └── settings.rs
```

## Principios Clave

### 1. Dependency Rule
Las dependencias apuntan hacia adentro:
- **Infrastructure** → **Application** → **Domain**
- El dominio NO conoce la infraestructura
- La aplicación define ports, la infraestructura los implementa

### 2. Domain Layer (Núcleo)
- **Entidades**: Lógica de negocio central
- **Value Objects**: Inmutables, validación en construcción
- **Domain Services**: Lógica que no pertenece a una entidad
- **Sin dependencias externas**: Solo Rust std y traits propios

### 3. Application Layer
- **Use Cases**: Orquestación de lógica de negocio
- **Ports**: Interfaces (traits) que definen contratos
- **DTOs**: Objetos de transferencia para comunicación

### 4. Infrastructure Layer
- **Adapters**: Implementaciones concretas de ports
- **Driving Adapters**: Reciben peticiones (gRPC, REST, CLI)
- **Driven Adapters**: Proveen servicios (DB, Cedar, etc.)

## Flujo de una Petición

```
1. gRPC Request
   ↓
2. gRPC Adapter (driving)
   ↓ (convierte proto → DTO)
3. Use Case (application)
   ↓ (usa domain entities)
4. Domain Logic
   ↓ (usa ports)
5. Repository Adapter (driven)
   ↓
6. Database
   ↓
7. Response (camino inverso)
```

## Beneficios

1. **Testabilidad**: Fácil mockear adapters
2. **Flexibilidad**: Cambiar DB sin tocar dominio
3. **Claridad**: Cada capa tiene responsabilidad clara
4. **Mantenibilidad**: Cambios localizados
5. **Independencia**: Dominio puro, sin frameworks

## Ejemplo: CreatePolicyStore

```rust
// Domain Entity
pub struct PolicyStore {
    id: PolicyStoreId,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

// Application Port (Interface)
#[async_trait]
pub trait PolicyStoreRepositoryPort: Send + Sync {
    async fn save(&self, store: &PolicyStore) -> Result<()>;
    async fn find_by_id(&self, id: &PolicyStoreId) -> Result<Option<PolicyStore>>;
}

// Application Use Case
pub struct CreatePolicyStoreUseCase {
    repository: Arc<dyn PolicyStoreRepositoryPort>,
}

impl CreatePolicyStoreUseCase {
    pub async fn execute(&self, description: Option<String>) -> Result<PolicyStore> {
        let store = PolicyStore::new(description);
        self.repository.save(&store).await?;
        Ok(store)
    }
}

// Infrastructure Adapter
pub struct SqlitePolicyStoreRepository {
    pool: SqlitePool,
}

#[async_trait]
impl PolicyStoreRepositoryPort for SqlitePolicyStoreRepository {
    async fn save(&self, store: &PolicyStore) -> Result<()> {
        // Implementación SQLite
    }
}

// gRPC Adapter
pub struct GrpcControlPlaneAdapter {
    create_policy_store_uc: Arc<CreatePolicyStoreUseCase>,
}

impl AuthorizationControl for GrpcControlPlaneAdapter {
    async fn create_policy_store(&self, req: Request<CreatePolicyStoreRequest>) 
        -> Result<Response<CreatePolicyStoreResponse>, Status> {
        let store = self.create_policy_store_uc
            .execute(req.into_inner().description)
            .await?;
        Ok(Response::new(store.into()))
    }
}
```

## Migración Gradual

1. ✅ Crear estructura de carpetas
2. ✅ Definir entidades de dominio
3. ✅ Definir ports (interfaces)
4. ✅ Implementar use cases
5. ✅ Migrar adapters existentes
6. ✅ Actualizar main.rs con composición
7. ✅ Tests unitarios por capa
8. ✅ Tests de integración

## Próximos Pasos

1. Crear estructura de carpetas
2. Implementar domain layer
3. Definir ports
4. Implementar use cases
5. Refactorizar adapters
6. Actualizar composición en main.rs
