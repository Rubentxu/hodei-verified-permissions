# 🚀 Quick Start Guide - Hodei Verified Permissions

**Guía rápida para ejecutar tests E2E**

---

## ⚡ Ejecución Rápida

### 1. Tests E2E Multi-Database

```bash
# Ejecutar todo el stack (SQLite, PostgreSQL, SurrealDB)
./scripts/test-e2e.sh
```

### 2. Tests E2E Identity Providers

```bash
# Ejecutar tests con Keycloak y Zitadel
./scripts/test-identity-providers.sh
```

---

## 📋 Prerequisitos

```bash
# Verificar Docker
docker info

# Si no está corriendo:
# Linux:
sudo systemctl start docker

# macOS:
open -a Docker

# Windows:
# Iniciar Docker Desktop
```

---

## 🧪 Comandos de Testing

### Tests Unitarios

```bash
# SDK (22 tests)
cargo test --workspace --lib

# Servidor (18 tests)
cd verified-permissions && cargo test --workspace

# TODO App (12 tests)
cd examples/todo-app && cargo test

# Todos los tests unitarios
cargo test --workspace --lib
```

### Tests E2E - Multi-Database

```bash
# Opción 1: Script automático - TODAS las bases de datos (recomendado)
./scripts/test-e2e.sh

# Opción 2: Solo SQLite (más rápido, sin dependencias externas)
docker compose -f docker-compose.test.yml --profile sqlite up -d
cargo test --test e2e_full_stack -- --ignored --nocapture
docker compose -f docker-compose.test.yml --profile sqlite down -v

# Opción 3: Solo PostgreSQL
docker compose -f docker-compose.test.yml --profile postgres up -d
cargo test --test e2e_multi_database test_postgres -- --ignored --nocapture
docker compose -f docker-compose.test.yml --profile postgres down -v

# Opción 4: Solo SurrealDB
docker compose -f docker-compose.test.yml --profile surrealdb up -d
cargo test --test e2e_multi_database test_surrealdb -- --ignored --nocapture
docker compose -f docker-compose.test.yml --profile surrealdb down -v

# Opción 5: Todas las bases de datos
docker compose -f docker-compose.test.yml --profile all up -d
cargo test --test e2e_multi_database -- --ignored --nocapture
docker compose -f docker-compose.test.yml --profile all down -v
```

### Tests E2E - Identity Providers

```bash
# Opción 1: Script automático (recomendado)
./scripts/test-identity-providers.sh

# Opción 2: Manual
docker compose -f docker-compose.identity-providers.yml up -d
cargo test --test e2e_identity_providers -- --ignored --nocapture
docker compose -f docker-compose.identity-providers.yml down -v

# Solo Keycloak
cargo test --test e2e_identity_providers test_keycloak -- --ignored --nocapture

# Solo Zitadel
cargo test --test e2e_identity_providers test_zitadel -- --ignored --nocapture
```

---

## 🐳 Docker Compose

### Multi-Database Stack

```bash
# Iniciar servicios
docker compose -f docker-compose.test.yml up -d

# Ver logs
docker compose -f docker-compose.test.yml logs -f

# Ver estado
docker compose -f docker-compose.test.yml ps

# Parar servicios
docker compose -f docker-compose.test.yml down -v
```

**Servicios** (8):
- PostgreSQL (:5432)
- SurrealDB (:8000)
- hodei-server-sqlite (:50051)
- hodei-server-postgres (:50052)
- hodei-server-surrealdb (:50053)
- todo-app-sqlite (:3000)
- todo-app-postgres (:3001)
- todo-app-surrealdb (:3002)

### Identity Providers Stack

```bash
# Iniciar servicios
docker compose -f docker-compose.identity-providers.yml up -d

# Ver logs
docker compose -f docker-compose.identity-providers.yml logs -f

# Ver estado
docker compose -f docker-compose.identity-providers.yml ps

# Parar servicios
docker compose -f docker-compose.identity-providers.yml down -v
```

**Servicios** (8):
- Keycloak (:8080) - admin/admin
- Zitadel (:8082)
- PostgreSQL for Keycloak
- CockroachDB for Zitadel
- hodei-server-keycloak (:50054)
- hodei-server-zitadel (:50055)
- todo-app-keycloak (:3003)
- todo-app-zitadel (:3004)

---

## 🌐 URLs de Servicios

### Multi-Database

| Servicio | URL | Descripción |
|----------|-----|-------------|
| Hodei (SQLite) | http://localhost:50051 | gRPC Server |
| Hodei (PostgreSQL) | http://localhost:50052 | gRPC Server |
| Hodei (SurrealDB) | http://localhost:50053 | gRPC Server |
| TODO App (SQLite) | http://localhost:3000 | REST API |
| TODO App (PostgreSQL) | http://localhost:3001 | REST API |
| TODO App (SurrealDB) | http://localhost:3002 | REST API |

### Identity Providers

| Servicio | URL | Credenciales |
|----------|-----|--------------|
| Keycloak | http://localhost:8080 | admin/admin |
| Zitadel | http://localhost:8082 | - |
| Hodei (Keycloak) | http://localhost:50054 | gRPC |
| Hodei (Zitadel) | http://localhost:50055 | gRPC |
| TODO App (Keycloak) | http://localhost:3003 | REST API |
| TODO App (Zitadel) | http://localhost:3004 | REST API |

---

## 📊 Resumen de Tests

```
Tests Unitarios:          52 tests
├── SDK:                  22 tests
├── Servidor:             18 tests
└── TODO App:             12 tests

Tests E2E:                28 tests
├── Full Stack:           6 tests
├── Multi-Database:       10 tests
└── Identity Providers:   12 tests

Total:                    80 tests
```

---

## 🔧 Troubleshooting

### Docker no está corriendo

```bash
# Verificar
docker info

# Iniciar (Linux)
sudo systemctl start docker

# Iniciar (macOS)
open -a Docker
```

### Puertos en uso

```bash
# Verificar puertos
lsof -i :50051  # Hodei server
lsof -i :3000   # TODO app
lsof -i :8080   # Keycloak

# Limpiar contenedores
docker compose -f docker-compose.test.yml down -v
docker compose -f docker-compose.identity-providers.yml down -v
```

### Tests fallan

```bash
# Ver logs de servicios
docker compose -f docker-compose.test.yml logs

# Ver logs de un servicio específico
docker compose -f docker-compose.test.yml logs hodei-server-sqlite

# Reiniciar servicios
docker compose -f docker-compose.test.yml restart
```

---

## 📚 Documentación Completa

Para más detalles, consulta:

1. **RESUMEN_FINAL.md** - Resumen ejecutivo completo
2. **VALIDATION_REPORT.md** - Reporte de validación
3. **tests/E2E_README.md** - Guía de tests E2E
4. **tests/MULTI_DATABASE_README.md** - Guía multi-database
5. **tests/IDENTITY_PROVIDERS_README.md** - Guía de Identity Providers
6. **ESTADO_PROYECTO.md** - Estado del proyecto
7. **E2E_STATUS.md** - Checklist E2E

---

## ⚡ Comandos Más Usados

```bash
# Tests unitarios rápidos
cargo test --workspace --lib

# Tests E2E multi-database
./scripts/test-e2e.sh

# Tests E2E identity providers
./scripts/test-identity-providers.sh

# Limpiar todo
docker compose -f docker-compose.test.yml down -v
docker compose -f docker-compose.identity-providers.yml down -v
docker system prune -f

# Ver estado de todo
docker ps -a
```

---

**Última actualización**: 21 de Octubre, 2025  
**Versión**: 1.0.0  
**Estado**: ✅ LISTO PARA USAR
