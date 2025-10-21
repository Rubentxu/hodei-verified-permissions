# Multi-Database E2E Testing

## 🎯 Objetivo

Validar que Hodei Verified Permissions funciona correctamente con **todas las bases de datos soportadas**:
- ✅ **SQLite** - Base de datos embebida
- ✅ **PostgreSQL** - Base de datos relacional
- ✅ **SurrealDB** - Base de datos multi-modelo

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│                    Multi-Database Test Setup                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Hodei      │    │   Hodei      │    │   Hodei      │      │
│  │   Server     │    │   Server     │    │   Server     │      │
│  │  (SQLite)    │    │ (PostgreSQL) │    │ (SurrealDB)  │      │
│  │  :50051      │    │  :50052      │    │  :50053      │      │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘      │
│         │                   │                   │               │
│         ▼                   ▼                   ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   SQLite     │    │  PostgreSQL  │    │  SurrealDB   │      │
│  │   File       │    │   :5432      │    │   :8000      │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                                                                   │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │  TODO App    │    │  TODO App    │    │  TODO App    │      │
│  │  (SQLite)    │    │ (PostgreSQL) │    │ (SurrealDB)  │      │
│  │  :3000       │    │  :3001       │    │  :3002       │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 🚀 Servicios Desplegados

### Bases de Datos

1. **PostgreSQL** (:5432)
   - Database: `hodei_test`
   - User: `hodei`
   - Password: `hodei_pass`

2. **SurrealDB** (:8000)
   - Mode: Memory
   - User: `root`
   - Password: `root`

### Servidores Hodei

1. **hodei-server-sqlite** (:50051)
   - Database: SQLite file
   - Volume: `sqlite-data`

2. **hodei-server-postgres** (:50052)
   - Database: PostgreSQL
   - Depends on: postgres

3. **hodei-server-surrealdb** (:50053)
   - Database: SurrealDB
   - Depends on: surrealdb

### Aplicaciones TODO

1. **todo-app-sqlite** (:3000)
   - Backend: hodei-server-sqlite

2. **todo-app-postgres** (:3001)
   - Backend: hodei-server-postgres

3. **todo-app-surrealdb** (:3002)
   - Backend: hodei-server-surrealdb

## 🧪 Tests Implementados

### 1. Test de Creación de Policy Store

Verifica que cada base de datos puede crear policy stores:

```rust
test_sqlite_policy_store_creation()
test_postgres_policy_store_creation()
test_surrealdb_policy_store_creation()
```

### 2. Test de Health Check

Verifica que todas las aplicaciones TODO están corriendo:

```rust
test_all_databases_health()
```

### 3. Test de Flujo de Autorización

Prueba el flujo completo en cada base de datos:
- Crear policy store
- Crear identity source
- Cargar políticas Cedar
- Evaluar autorización

```rust
test_sqlite_authorization_flow()
test_postgres_authorization_flow()
test_surrealdb_authorization_flow()
```

### 4. Test de Integración TODO App

Valida que las aplicaciones TODO funcionan con cada backend:
- Listar tasks
- Crear tasks
- Autorización funciona correctamente

```rust
test_all_databases_todo_app_integration()
```

### 5. Test de Aislamiento

Verifica que cada base de datos está aislada:
- Policy stores tienen IDs únicos
- No hay interferencia entre bases de datos

```rust
test_database_isolation()
```

### 6. Test de Operaciones Concurrentes

Prueba operaciones simultáneas en todas las bases de datos:

```rust
test_concurrent_database_operations()
```

## 🚀 Cómo Ejecutar

### Opción 1: Script Automático (Recomendado)

```bash
./scripts/test-e2e.sh
```

Este script:
1. Inicia todas las bases de datos
2. Inicia todos los servidores Hodei
3. Inicia todas las aplicaciones TODO
4. Ejecuta tests de full stack
5. Ejecuta tests multi-database
6. Limpia todo al terminar

### Opción 2: Manual

```bash
# 1. Iniciar todos los servicios
docker-compose -f docker-compose.test.yml up -d

# 2. Verificar que estén corriendo
docker-compose -f docker-compose.test.yml ps

# 3. Ejecutar tests multi-database
cargo test --test e2e_multi_database -- --ignored --nocapture

# 4. Limpiar
docker-compose -f docker-compose.test.yml down -v
```

### Opción 3: Probar Base de Datos Específica

```bash
# Solo SQLite
docker-compose -f docker-compose.test.yml up -d postgres hodei-server-sqlite todo-app-sqlite
cargo test --test e2e_multi_database test_sqlite -- --ignored --nocapture

# Solo PostgreSQL
docker-compose -f docker-compose.test.yml up -d postgres hodei-server-postgres todo-app-postgres
cargo test --test e2e_multi_database test_postgres -- --ignored --nocapture

# Solo SurrealDB
docker-compose -f docker-compose.test.yml up -d surrealdb hodei-server-surrealdb todo-app-surrealdb
cargo test --test e2e_multi_database test_surrealdb -- --ignored --nocapture
```

## 📊 Escenarios de Prueba

### Escenario 1: Validación de Backends

**Objetivo**: Verificar que cada base de datos funciona correctamente

**Tests**:
- ✅ SQLite puede almacenar y recuperar policy stores
- ✅ PostgreSQL puede almacenar y recuperar policy stores
- ✅ SurrealDB puede almacenar y recuperar policy stores

### Escenario 2: Autorización Multi-DB

**Objetivo**: Verificar que la autorización funciona en todas las bases de datos

**Flow**:
1. Crear policy store en cada DB
2. Crear identity source en cada DB
3. Cargar políticas Cedar en cada DB
4. Evaluar decisiones de autorización
5. Verificar que todas retornan ALLOW/DENY correctamente

### Escenario 3: Aplicaciones TODO Multi-DB

**Objetivo**: Verificar que las aplicaciones pueden usar cualquier backend

**Tests**:
- ✅ TODO app con SQLite puede listar/crear tasks
- ✅ TODO app con PostgreSQL puede listar/crear tasks
- ✅ TODO app con SurrealDB puede listar/crear tasks

### Escenario 4: Aislamiento de Datos

**Objetivo**: Verificar que cada base de datos está aislada

**Validación**:
- Policy stores en SQLite no afectan PostgreSQL
- Policy stores en PostgreSQL no afectan SurrealDB
- Policy stores en SurrealDB no afectan SQLite

### Escenario 5: Performance Concurrente

**Objetivo**: Verificar que todas las bases de datos manejan concurrencia

**Test**:
- Operaciones simultáneas en las 3 bases de datos
- Todas completan exitosamente
- No hay race conditions

## 🔧 Configuración

### Variables de Entorno por Servidor

**SQLite Server**:
```bash
DATABASE_URL=sqlite:///app/data/hodei.db
RUST_LOG=debug
```

**PostgreSQL Server**:
```bash
DATABASE_URL=postgres://hodei:hodei_pass@postgres:5432/hodei_test
RUST_LOG=debug
```

**SurrealDB Server**:
```bash
DATABASE_URL=surreal://root:root@surrealdb:8000/hodei/test
RUST_LOG=debug
```

## 🐛 Troubleshooting

### Error: "PostgreSQL connection refused"

```bash
# Verificar que PostgreSQL está corriendo
docker-compose -f docker-compose.test.yml logs postgres

# Verificar health check
docker-compose -f docker-compose.test.yml ps postgres
```

### Error: "SurrealDB not responding"

```bash
# Verificar logs de SurrealDB
docker-compose -f docker-compose.test.yml logs surrealdb

# Verificar que el puerto está abierto
curl http://localhost:8000/health
```

### Error: "SQLite file not found"

```bash
# Verificar que el volumen está montado
docker volume inspect hodei-verified-permissions_sqlite-data

# Verificar logs del servidor
docker-compose -f docker-compose.test.yml logs hodei-server-sqlite
```

### Tests fallan en una base de datos específica

```bash
# Ver logs del servidor específico
docker-compose -f docker-compose.test.yml logs hodei-server-[sqlite|postgres|surrealdb]

# Verificar conectividad
docker-compose -f docker-compose.test.yml exec hodei-server-sqlite sh
```

## 📈 Métricas de Cobertura

| Base de Datos | Policy Store | Identity Source | Policies | Authorization | TODO App |
|---------------|--------------|-----------------|----------|---------------|----------|
| SQLite        | ✅           | ✅              | ✅       | ✅            | ✅       |
| PostgreSQL    | ✅           | ✅              | ✅       | ✅            | ✅       |
| SurrealDB     | ✅           | ✅              | ✅       | ✅            | ✅       |

## ✅ Checklist de Validación

- [ ] PostgreSQL inicia correctamente
- [ ] SurrealDB inicia correctamente
- [ ] Los 3 servidores Hodei inician
- [ ] Las 3 aplicaciones TODO inician
- [ ] Test de health check pasa
- [ ] Test de policy store creation pasa (x3)
- [ ] Test de authorization flow pasa (x3)
- [ ] Test de TODO app integration pasa (x3)
- [ ] Test de isolation pasa
- [ ] Test de concurrent operations pasa

## 🎯 Próximos Pasos

1. ✅ Implementar soporte para PostgreSQL
2. ✅ Implementar soporte para SurrealDB
3. ✅ Crear tests multi-database
4. ⏳ Optimizar performance de cada backend
5. ⏳ Añadir métricas de performance
6. ⏳ Comparar velocidad entre bases de datos

---

**Estado**: ✅ **MULTI-DATABASE SUPPORT COMPLETO**  
**Bases de Datos**: ✅ **3/3 SOPORTADAS**  
**Tests**: ✅ **10 TESTS IMPLEMENTADOS**  
**Cobertura**: ✅ **100% POR BASE DE DATOS**
