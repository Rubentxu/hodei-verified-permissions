# Plan de Restauración de Funcionalidad

## Resumen Ejecutivo

Durante el debugging de E2E tests, se convirtieron implementaciones reales a "dummy implementations". **Toda la funcionalidad del sistema se perdió**. Este documento describe cómo restaurarla.

## Estado Actual vs Requerido

### ❌ Estado Actual (BROKEN)
- Servicios gRPC retornan datos dummy o unimplemented
- No hay persistencia en base de datos
- No hay validación de políticas Cedar
- No hay evaluación real de autorización
- Siempre retorna ALLOW (inseguro)
- JWT tokens no se validan

### ✅ Estado Requerido (AWS Verified Permissions Compatible)
- Persistencia completa en SQLite/PostgreSQL/SurrealDB
- Validación de políticas Cedar contra schemas
- Evaluación real de autorización con Cedar Policy Engine
- JWT validation con JWKS
- Claims mapping a entidades Cedar
- Audit logging de decisiones

## Archivos a Restaurar

### 1. `verified-permissions/api/src/grpc/control_plane.rs`

**Cambios necesarios:**

```rust
// ANTES (DUMMY):
pub struct AuthorizationControlService;

impl AuthorizationControlService {
    pub fn new() -> Self {
        Self
    }
}

// DESPUÉS (REAL):
use hodei_infrastructure::repository::RepositoryAdapter;

pub struct AuthorizationControlService {
    repository: Arc<RepositoryAdapter>,
}

impl AuthorizationControlService {
    pub fn new(repository: Arc<RepositoryAdapter>) -> Self {
        Self { repository }
    }
}
```

**Métodos a restaurar:**
- `create_policy_store` - Usar `repository.create_policy_store()`
- `get_policy_store` - Usar `repository.get_policy_store()`
- `list_policy_stores` - Usar `repository.list_policy_stores()`
- `delete_policy_store` - Usar `repository.delete_policy_store()`
- `put_schema` - Validar schema Cedar + `repository.put_schema()`
- `get_schema` - Usar `repository.get_schema()`
- `create_policy` - Validar política Cedar + `repository.create_policy()`
- `get_policy` - Usar `repository.get_policy()`
- `update_policy` - Validar política + `repository.update_policy()`
- `delete_policy` - Usar `repository.delete_policy()`
- `list_policies` - Usar `repository.list_policies()`
- `create_identity_source` - Parsear config + `repository.create_identity_source()`
- `get_identity_source` - Usar `repository.get_identity_source()`
- `list_identity_sources` - Usar `repository.list_identity_sources()`
- `delete_identity_source` - Usar `repository.delete_identity_source()`
- `create_policy_template` - Validar template + `repository.create_policy_template()`
- `get_policy_template` - Usar `repository.get_policy_template()`
- `list_policy_templates` - Usar `repository.list_policy_templates()`
- `delete_policy_template` - Usar `repository.delete_policy_template()`

### 2. `verified-permissions/api/src/grpc/data_plane.rs`

**Cambios necesarios:**

```rust
// ANTES (DUMMY):
pub struct AuthorizationDataService;

impl AuthorizationDataService {
    pub fn new() -> Self {
        Self
    }
}

// DESPUÉS (REAL):
use hodei_infrastructure::repository::RepositoryAdapter;

pub struct AuthorizationDataService {
    repository: Arc<RepositoryAdapter>,
}

impl AuthorizationDataService {
    pub fn new(repository: Arc<RepositoryAdapter>) -> Self {
        Self { repository }
    }
}
```

**Métodos a restaurar:**
- `is_authorized` - Cargar políticas de BD + evaluar con Cedar + audit log
- `batch_is_authorized` - Llamar is_authorized para cada request
- `is_authorized_with_token` - Validar JWT + extraer claims + mapear a principal + evaluar

**Componentes críticos:**
- JWT validation con JWKS
- Claims mapping configuration
- Cedar policy evaluation
- Audit logging

### 3. `verified-permissions/main/src/main.rs`

**Cambios necesarios:**

```rust
// ANTES (DUMMY):
let control_service = AuthorizationControlService::new();
let data_service = AuthorizationDataService::new();

// DESPUÉS (REAL):
// Leer DATABASE_URL del environment
let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "sqlite:///app/data/hodei.db".to_string());

// Crear repository adapter
let repository = Arc::new(
    RepositoryAdapter::new(&database_url)
        .await
        .expect("Failed to create repository")
);

// Inyectar repository en servicios
let control_service = AuthorizationControlService::new(repository.clone());
let data_service = AuthorizationDataService::new(repository.clone());
```

## Referencia de Implementación Original

El commit `942fca9` contiene la implementación completa funcional. Puedes extraerla con:

```bash
git show 942fca9:verified-permissions/api/src/grpc/control_plane.rs > /tmp/control_plane_original.rs
git show 942fca9:verified-permissions/api/src/grpc/data_plane.rs > /tmp/data_plane_original.rs
git show 942fca9:verified-permissions/main/src/main.rs > /tmp/main_original.rs
```

## Pasos de Restauración

### Paso 1: Restaurar control_plane.rs
1. Añadir campo `repository: Arc<RepositoryAdapter>` al struct
2. Actualizar constructor para recibir repository
3. Restaurar cada método usando repository
4. Añadir validación de políticas Cedar
5. Añadir validación de schemas

### Paso 2: Restaurar data_plane.rs
1. Añadir campo `repository: Arc<RepositoryAdapter>` al struct
2. Actualizar constructor para recibir repository
3. Restaurar `is_authorized` con evaluación Cedar real
4. Restaurar `is_authorized_with_token` con JWT validation
5. Añadir audit logging

### Paso 3: Restaurar main.rs
1. Crear repository adapter desde DATABASE_URL
2. Inyectar repository en servicios
3. Configurar logging

### Paso 4: Actualizar Tests E2E
1. Verificar que tests usan funcionalidad real
2. Añadir tests para cada operación CRUD
3. Añadir tests para validación de políticas
4. Añadir tests para JWT validation
5. Añadir tests para audit logging

### Paso 5: Ejecutar y Verificar
1. Compilar: `cargo build --bin hodei-verified-permissions`
2. Ejecutar tests SQLite: `./scripts/test-e2e-sqlite.sh`
3. Ejecutar tests PostgreSQL: `./scripts/test-e2e-postgres.sh`
4. Ejecutar tests SurrealDB: `./scripts/test-e2e-surrealdb.sh`
5. Verificar logs de audit

## Validación de Funcionalidad

### Checklist de Funcionalidad Restaurada

#### Control Plane
- [ ] Policy Store CRUD funciona
- [ ] Schema management funciona
- [ ] Policy CRUD funciona con validación
- [ ] Identity Source CRUD funciona
- [ ] Policy Template CRUD funciona

#### Data Plane
- [ ] IsAuthorized evalúa políticas reales
- [ ] IsAuthorized retorna DENY cuando corresponde
- [ ] IsAuthorizedWithToken valida JWT
- [ ] IsAuthorizedWithToken extrae claims correctamente
- [ ] BatchIsAuthorized funciona

#### Persistencia
- [ ] Datos persisten en SQLite
- [ ] Datos persisten en PostgreSQL
- [ ] Datos persisten en SurrealDB
- [ ] Datos sobreviven restart del servidor

#### Seguridad
- [ ] Políticas inválidas son rechazadas
- [ ] JWT inválidos son rechazados
- [ ] Schemas inválidos son rechazados
- [ ] Audit log registra todas las decisiones

## Comandos Útiles

```bash
# Ver implementación original
git show 942fca9:verified-permissions/api/src/grpc/control_plane.rs | less

# Comparar con actual
diff <(git show 942fca9:verified-permissions/api/src/grpc/control_plane.rs) \
     verified-permissions/api/src/grpc/control_plane.rs

# Ejecutar test específico
cargo test --test e2e_multi_database test_sqlite_policy_store_creation -- --ignored --nocapture

# Ver logs del servidor
docker logs hodei-server-sqlite -f

# Inspeccionar base de datos SQLite
sqlite3 /path/to/hodei.db ".schema"
```

## Tiempo Estimado

- Restaurar control_plane.rs: 2-3 horas
- Restaurar data_plane.rs: 2-3 horas
- Restaurar main.rs: 30 minutos
- Actualizar tests: 1-2 horas
- Testing y debugging: 2-3 horas
- **Total: 8-12 horas**

## Prioridad

🚨 **CRÍTICO** - El sistema actual no funciona y no es seguro para producción.

## Próximos Pasos Inmediatos

1. Restaurar `control_plane.rs` con repository
2. Restaurar `data_plane.rs` con repository
3. Actualizar `main.rs` para inyectar dependencias
4. Ejecutar tests y verificar funcionalidad
5. Actualizar documentación

## Notas Importantes

- **NO** eliminar las implementaciones dummy hasta verificar que las reales funcionan
- Hacer commits incrementales después de cada archivo restaurado
- Ejecutar tests después de cada cambio
- Mantener compatibilidad con AWS Verified Permissions API
