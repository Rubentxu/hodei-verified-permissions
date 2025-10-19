# 📋 Plan de Arquitectura Híbrida - Cache en Memoria + Multi-DB

**Objetivo**: Implementar arquitectura de alto rendimiento con Cedar en memoria y soporte para múltiples bases de datos.

## 🎯 Principios de Diseño

### Cedar es Stateless
- ✅ Evaluación 100% en memoria
- ✅ Sin I/O durante autorización
- ✅ PolicySet y Schema cargados en RAM
- ✅ Latencia ultra-baja (~100μs)

### Base de Datos = Persistencia
- ✅ Solo para CRUD de políticas/schemas
- ✅ Recuperación al inicio
- ✅ Auditoría de decisiones
- ❌ NO para evaluación de políticas

## 📊 Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                      gRPC Server                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         AuthorizationService                         │  │
│  │  - is_authorized() → 100% memoria (~100μs)          │  │
│  │  - batch_is_authorized()                            │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         CacheManager (En Memoria)                    │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │  │
│  │  │ Store 1    │  │ Store 2    │  │ Store 3    │    │  │
│  │  │ PolicySet  │  │ PolicySet  │  │ PolicySet  │    │  │
│  │  │ Schema     │  │ Schema     │  │ Schema     │    │  │
│  │  └────────────┘  └────────────┘  └────────────┘    │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                   │
│                   (Solo para CRUD)                          │
│                         ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │      PolicyRepository (Trait)                        │  │
│  └──────────────────────────────────────────────────────┘  │
│         ↓                ↓                ↓                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐            │
│  │ SQLite   │    │Postgres  │    │SurrealDB │            │
│  │ 3.46+    │    │ 17.x     │    │ 2.x      │            │
│  └──────────┘    └──────────┘    └──────────┘            │
└─────────────────────────────────────────────────────────────┘
```

## 🗓️ Plan de Implementación (5 Semanas)

### Semana 1: Abstracción de Repository ✅

#### Objetivos
- Crear trait `PolicyRepository`
- Refactorizar código existente
- Mantener compatibilidad

#### Tareas
1. **Crear trait base** (`src/storage/repository_trait.rs`)
   - Definir todas las operaciones
   - Usar `async_trait`
   - Documentar cada método

2. **Refactorizar SqliteRepository**
   - Implementar trait
   - Mantener funcionalidad actual
   - Actualizar tests

3. **Tests**
   - Tests del trait con mock
   - Tests de SqliteRepository
   - Verificar 40 tests siguen pasando

#### Entregables
- ✅ `PolicyRepository` trait
- ✅ `SqliteRepository` implementa trait
- ✅ Tests pasando (40/40)

---

### Semana 2: Sistema de Cache en Memoria 🔥

#### Objetivos
- Cache de PolicySet por store
- Sincronización DB ↔ Cache
- Rendimiento ~100μs

#### Tareas
1. **PolicyStoreCache** (`src/cache/policy_store_cache.rs`)
   ```rust
   pub struct PolicyStoreCache {
       policy_store_id: String,
       policy_set: Arc<RwLock<PolicySet>>,
       schema: Arc<RwLock<Option<Schema>>>,
       last_updated: Arc<RwLock<DateTime<Utc>>>,
   }
   ```

2. **CacheManager** (`src/cache/cache_manager.rs`)
   ```rust
   pub struct CacheManager {
       caches: Arc<RwLock<HashMap<String, Arc<PolicyStoreCache>>>>,
       repository: Arc<dyn PolicyRepository>,
   }
   ```

3. **Operaciones**
   - `initialize()` - Cargar todo desde DB
   - `get_cache()` - Obtener cache de un store
   - `create_policy()` - DB + Cache
   - `delete_policy()` - DB + Cache
   - `reload_cache()` - Sincronizar desde DB

4. **Tests**
   - Tests de cache aislado
   - Tests de sincronización
   - Benchmarks de rendimiento

#### Entregables
- ✅ `PolicyStoreCache` funcional
- ✅ `CacheManager` con sincronización
- ✅ Tests de cache (10+ tests)
- ✅ Benchmarks (~100μs)

---

### Semana 3: Authorization Service con Cache

#### Objetivos
- Servicio de autorización usando cache
- Evaluación 100% en memoria
- Logging asíncrono

#### Tareas
1. **AuthorizationService** (`src/authorization/service.rs`)
   ```rust
   pub struct AuthorizationService {
       cache_manager: Arc<CacheManager>,
       authorizer: Authorizer,
   }
   ```

2. **Métodos**
   - `is_authorized()` - Lectura de cache + Cedar
   - `batch_is_authorized()` - Múltiples evaluaciones
   - `log_authorization_async()` - Log sin bloquear

3. **Integración gRPC**
   - Actualizar `DataPlaneService`
   - Usar `AuthorizationService`
   - Mantener API compatible

4. **Tests E2E**
   - Tests con cache real
   - Tests de concurrencia
   - Tests de rendimiento

#### Entregables
- ✅ `AuthorizationService` funcional
- ✅ Integración gRPC completa
- ✅ Tests E2E (15+ tests)
- ✅ Latencia <200μs

---

### Semana 4: Multi-Database Support

#### Objetivos
- Soporte para PostgreSQL
- Soporte para SurrealDB
- Configuración flexible

#### Tareas

##### 4.1 PostgreSQL Repository
1. **Dependencias**
   ```toml
   sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls"] }
   ```

2. **Implementación** (`src/storage/postgres_repository.rs`)
   - Implementar `PolicyRepository`
   - Migrar schemas SQLite → PostgreSQL
   - Optimizar queries

3. **Tests**
   - Tests con PostgreSQL real
   - Tests de migración

##### 4.2 SurrealDB Repository
1. **Dependencias**
   ```toml
   surrealdb = "2.0"
   ```

2. **Implementación** (`src/storage/surreal_repository.rs`)
   - Implementar `PolicyRepository`
   - Definir schemas SurrealDB
   - Aprovechar graph queries

3. **Tests**
   - Tests con SurrealDB embebido
   - Tests de graph queries

##### 4.3 Factory Pattern
1. **Config** (`src/config.rs`)
   ```rust
   pub enum DatabaseProvider {
       Sqlite,
       Postgres,
       Surreal,
   }
   ```

2. **Factory** (`src/storage/factory.rs`)
   ```rust
   pub async fn create_repository(config: &DatabaseConfig) 
       -> Result<Arc<dyn PolicyRepository>>
   ```

3. **Main**
   - Cargar config desde env
   - Crear repository dinámicamente
   - Inicializar cache

#### Entregables
- ✅ `PostgresRepository` funcional
- ✅ `SurrealRepository` funcional
- ✅ Factory pattern
- ✅ Configuración flexible
- ✅ Tests para cada DB (30+ tests)

---

### Semana 5: Optimización y Sincronización

#### Objetivos
- Reload automático de cache
- Métricas de rendimiento
- Optimizaciones finales

#### Tareas
1. **Background Reload Task** (`src/cache/reload_task.rs`)
   - Task periódico para recargar
   - Configurable (default: 5 min)
   - Manejo de errores

2. **Métricas** (`src/metrics/mod.rs`)
   - Cache hits/misses
   - Latencias de autorización
   - Tamaño de cache
   - Operaciones por segundo

3. **Optimizaciones**
   - Profiling con `cargo flamegraph`
   - Optimizar hot paths
   - Reducir allocations

4. **Documentación**
   - Guía de configuración
   - Guía de rendimiento
   - Comparativa de DBs

#### Entregables
- ✅ Reload automático
- ✅ Sistema de métricas
- ✅ Documentación completa
- ✅ Benchmarks finales

---

## 📦 Dependencias

### Cargo.toml
```toml
[dependencies]
# Existentes
cedar-policy = "4.7.0"
tokio = { version = "1", features = ["full"] }
tonic = "0.14"
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "2.0"

# Database - SQLite (actual)
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }

# Database - PostgreSQL (nuevo)
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"] }

# Database - SurrealDB (nuevo)
surrealdb = "2.0"

# Async
async-trait = "0.1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Config
config = "0.14"
```

## 🎯 Versiones Estables

| Dependencia | Versión | Notas |
|-------------|---------|-------|
| **Cedar** | 4.7.0 | Última estable |
| **SQLite** | 3.46+ | Via sqlx 0.8 |
| **PostgreSQL** | 17.x | Última estable |
| **SurrealDB** | 2.0+ | Última estable |
| **Rust** | 1.83+ | Edition 2024 |

## 🔧 Configuración

### Variables de Entorno

```bash
# Database Provider (sqlite, postgres, surreal)
DATABASE_PROVIDER=sqlite

# Database URL
DATABASE_URL=sqlite:./hodei.db
# DATABASE_URL=postgresql://user:pass@localhost:5432/hodei
# DATABASE_URL=ws://localhost:8000

# Cache
CACHE_ENABLED=true
CACHE_RELOAD_INTERVAL_SECS=300

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=50051
```

### Ejemplo de Uso

```bash
# SQLite (desarrollo)
DATABASE_PROVIDER=sqlite DATABASE_URL=sqlite:./hodei.db ./hodei-server

# PostgreSQL (producción)
DATABASE_PROVIDER=postgres DATABASE_URL=postgresql://user:pass@db:5432/hodei ./hodei-server

# SurrealDB (experimental)
DATABASE_PROVIDER=surreal DATABASE_URL=ws://localhost:8000 ./hodei-server
```

## 📊 Rendimiento Esperado

### Latencias

| Operación | Sin Cache | Con Cache | Mejora |
|-----------|-----------|-----------|--------|
| **IsAuthorized** | ~1-2ms | ~100μs | 10-20x 🔥 |
| **BatchIsAuthorized (30)** | ~30-60ms | ~3ms | 10-20x 🔥 |
| **CreatePolicy** | ~1ms | ~1ms | - |
| **GetPolicy** | ~0.5ms | ~10ns | 50,000x 🔥 |

### Throughput

| Operación | Ops/seg |
|-----------|---------|
| **IsAuthorized** | 100,000+ |
| **BatchIsAuthorized** | 10,000+ |
| **CreatePolicy** | 1,000+ |

### Memoria

| Componente | Uso |
|------------|-----|
| **Cache por Store** | ~1-10 MB |
| **10 Stores** | ~10-100 MB |
| **100 Stores** | ~100MB-1GB |

## 🧪 Testing

### Estrategia
1. **Unit Tests** - Cada componente aislado
2. **Integration Tests** - Repository + Cache
3. **E2E Tests** - Servidor completo
4. **Benchmark Tests** - Rendimiento

### Cobertura Objetivo
- **Unit**: 90%+
- **Integration**: 80%+
- **E2E**: 70%+

### Tests por Semana
- Semana 1: 40 tests (existentes)
- Semana 2: +10 tests (cache)
- Semana 3: +15 tests (authorization)
- Semana 4: +30 tests (multi-db)
- Semana 5: +5 tests (optimización)
- **Total: 100 tests** 🎯

## 📈 Métricas de Éxito

### Funcionales
- ✅ Soporte para 3 bases de datos
- ✅ Cache en memoria funcional
- ✅ Sincronización automática
- ✅ 100% compatibilidad con API actual

### No Funcionales
- ✅ Latencia <200μs (IsAuthorized)
- ✅ Throughput >50K ops/s
- ✅ 100 tests pasando
- ✅ Documentación completa

## 🚀 Roadmap

```
Semana 1: ████████░░░░░░░░░░░░ 20% - Abstracción
Semana 2: ████████████░░░░░░░░ 40% - Cache
Semana 3: ████████████████░░░░ 60% - Authorization
Semana 4: ████████████████████ 80% - Multi-DB
Semana 5: ████████████████████ 100% - Optimización ✅
```

## 📝 Notas de Implementación

### Prioridades
1. **Corrección** > Rendimiento
2. **Tests** > Features
3. **Documentación** > Optimización

### Decisiones de Diseño
- **RwLock** para cache (muchas lecturas, pocas escrituras)
- **Arc** para compartir entre threads
- **async_trait** para trait async
- **Factory pattern** para crear repositories

### Trade-offs
- **Memoria vs Latencia**: Usamos más memoria para latencia ultra-baja
- **Consistencia vs Disponibilidad**: Eventual consistency en cache
- **Complejidad vs Flexibilidad**: Más complejo pero soporta 3 DBs

## ✅ Checklist de Inicio

Antes de empezar:
- [x] 40 tests actuales pasando
- [x] Código compilando sin errores
- [x] Documentación actualizada
- [ ] Branch `feature/hybrid-architecture` creado
- [ ] Plan revisado y aprobado

## 🎯 Próximo Paso

**Empezar Semana 1: Abstracción de Repository**

Crear:
1. `src/storage/repository_trait.rs`
2. Refactorizar `src/storage/repository.rs`
3. Tests del trait

¿Listo para empezar? 🚀
