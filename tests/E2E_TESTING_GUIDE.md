# Guía de Tests E2E - Hodei Verified Permissions

## Estructura de Tests

Los tests E2E están organizados para probar cada base de datos de forma independiente, evitando conflictos de recursos y facilitando el debugging.

### Archivos Docker Compose

Cada base de datos tiene su propio archivo docker-compose:

- **`docker-compose.sqlite.yml`** - SQLite backend (sin base de datos externa)
- **`docker-compose.postgres.yml`** - PostgreSQL backend
- **`docker-compose.surrealdb.yml`** - SurrealDB backend

### Scripts de Test

```bash
# Test solo SQLite
./scripts/test-e2e-sqlite.sh

# Test solo PostgreSQL
./scripts/test-e2e-postgres.sh

# Test solo SurrealDB
./scripts/test-e2e-surrealdb.sh

# Test todas las bases de datos secuencialmente
./scripts/test-e2e-all.sh
```

## Arquitectura de Tests

### Puertos Asignados

#### Hodei Servers
- **SQLite**: `localhost:50051`
- **PostgreSQL**: `localhost:50052`
- **SurrealDB**: `localhost:50053`

#### TODO Apps
- **SQLite**: `localhost:3000`
- **PostgreSQL**: `localhost:3001`
- **SurrealDB**: `localhost:3002`

#### Bases de Datos
- **PostgreSQL**: `localhost:5432`
- **SurrealDB**: `localhost:8001`

### Estructura de Servicios

Cada docker-compose incluye:

1. **Base de datos** (excepto SQLite que usa archivo local)
2. **hodei-server** - Servidor de autorización con la BD específica
3. **todo-app** - Aplicación de ejemplo que usa hodei-server

## Tests Disponibles

### Tests por Base de Datos

Los tests están en `tests/e2e_multi_database.rs` y están marcados con `#[ignore]` para ejecutarse solo cuando se solicita explícitamente.

#### SQLite Tests
```bash
cargo test --test e2e_multi_database test_sqlite -- --ignored --nocapture
```

Tests incluidos:
- `test_sqlite_policy_store_creation` - Creación de policy store
- `test_sqlite_authorization_flow` - Flujo completo de autorización

#### PostgreSQL Tests
```bash
cargo test --test e2e_multi_database test_postgres -- --ignored --nocapture
```

Tests incluidos:
- `test_postgres_policy_store_creation` - Creación de policy store
- `test_postgres_authorization_flow` - Flujo completo de autorización

#### SurrealDB Tests
```bash
cargo test --test e2e_multi_database test_surrealdb -- --ignored --nocapture
```

Tests incluidos:
- `test_surrealdb_policy_store_creation` - Creación de policy store
- `test_surrealdb_authorization_flow` - Flujo completo de autorización

### Tests de Integración

Estos tests requieren que todos los servicios estén corriendo (usar `docker-compose.test.yml` antiguo):

- `test_all_databases_health` - Verifica que todas las TODO apps respondan
- `test_database_isolation` - Verifica aislamiento entre bases de datos
- `test_concurrent_database_operations` - Operaciones concurrentes

## Ventajas de la Nueva Estructura

### ✅ Aislamiento
- Cada test usa solo los recursos que necesita
- No hay conflictos de puertos entre tests
- Fácil identificar problemas específicos de una BD

### ✅ Eficiencia
- Tests más rápidos al no levantar servicios innecesarios
- Menor uso de recursos del sistema
- Builds paralelos más eficientes

### ✅ Debugging
- Logs más limpios y específicos
- Fácil reproducir fallos de una BD específica
- Mejor aislamiento de problemas

### ✅ CI/CD Friendly
- Tests pueden ejecutarse en paralelo en diferentes jobs
- Fácil configurar matrix de tests por BD
- Mejor reporte de fallos por backend

## Flujo de Trabajo Recomendado

### Desarrollo Local

1. **Test rápido de una BD específica:**
   ```bash
   ./scripts/test-e2e-sqlite.sh  # Más rápido, sin deps externas
   ```

2. **Verificar cambios en todas las BDs:**
   ```bash
   ./scripts/test-e2e-all.sh
   ```

### CI/CD

```yaml
# Ejemplo GitHub Actions
jobs:
  test-sqlite:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: ./scripts/test-e2e-sqlite.sh

  test-postgres:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: ./scripts/test-e2e-postgres.sh

  test-surrealdb:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: ./scripts/test-e2e-surrealdb.sh
```

## Troubleshooting

### Servicios no inician

```bash
# Ver logs de un servicio específico
docker compose -f docker-compose.sqlite.yml logs hodei-server-sqlite

# Reiniciar servicios
docker compose -f docker-compose.sqlite.yml down -v
docker compose -f docker-compose.sqlite.yml up -d
```

### Puertos en uso

```bash
# Verificar qué está usando un puerto
lsof -i :50051

# Limpiar todos los contenedores
docker compose -f docker-compose.sqlite.yml down -v
docker compose -f docker-compose.postgres.yml down -v
docker compose -f docker-compose.surrealdb.yml down -v
```

### Tests fallan intermitentemente

- Aumentar el timeout en los scripts (línea `timeout 60`)
- Verificar healthchecks en docker-compose files
- Revisar logs para ver si los servicios están listos

## Migración desde docker-compose.test.yml

El archivo `docker-compose.test.yml` antiguo sigue disponible para:
- Tests de integración que requieren todas las BDs
- Tests de concurrencia entre BDs
- Verificación de aislamiento

Para usarlo:
```bash
./scripts/test-e2e.sh  # Script original
```

## Próximos Pasos

- [ ] Añadir tests de performance por BD
- [ ] Tests de migración entre BDs
- [ ] Tests de backup/restore
- [ ] Benchmarks comparativos
