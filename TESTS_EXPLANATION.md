# Explicación de Tests Ignorados

## 📋 Resumen

El proyecto tiene **47 tests ignorados** distribuidos en 8 archivos. Estos son **tests E2E (End-to-End)** que requieren infraestructura externa para ejecutarse.

---

## 🔍 Desglose de Tests Ignorados

### **1. Tests E2E Full Stack (6 tests)**
**Archivo:** `tests/e2e_full_stack.rs`

Requiere:
- ✅ Servidor Hodei gRPC en `localhost:50051`
- ✅ TODO App en `localhost:3000`
- ✅ Base de datos configurada

Tests:
- `test_e2e_policy_store_creation`
- `test_e2e_todo_app_health_check`
- `test_e2e_authorization_with_real_server`
- `test_e2e_rbac_scenarios`
- `test_e2e_simplerest_mapping`
- `test_e2e_todo_app_with_authorization`

### **2. Tests E2E Identity Providers (11 tests)**
**Archivo:** `tests/e2e_identity_providers.rs`

Requiere:
- ✅ Servidor Hodei gRPC
- ✅ Keycloak en `localhost:8080`
- ✅ Zitadel en `localhost:8000`
- ✅ Base de datos

Tests:
- `test_keycloak_health`
- `test_keycloak_identity_source_creation`
- `test_keycloak_authorization_flow`
- `test_keycloak_todo_app_integration`
- `test_claims_mapping_keycloak`
- `test_zitadel_health`
- `test_zitadel_identity_source_creation`
- `test_zitadel_authorization_flow`
- `test_zitadel_todo_app_integration`
- `test_claims_mapping_zitadel`
- `test_identity_provider_auto_detection`

### **3. Tests E2E Multi Database (10 tests)**
**Archivo:** `tests/e2e_multi_database.rs`

Requiere:
- ✅ Servidor Hodei gRPC
- ✅ PostgreSQL en `localhost:5432`
- ✅ SQLite (local)
- ✅ SurrealDB en `localhost:8000`

Tests:
- `test_postgres_policy_store_creation`
- `test_postgres_authorization_flow`
- `test_sqlite_policy_store_creation`
- `test_sqlite_authorization_flow`
- `test_surrealdb_policy_store_creation`
- `test_surrealdb_authorization_flow`
- `test_database_isolation`
- `test_concurrent_database_operations`
- `test_all_databases_health`
- `test_all_databases_todo_app_integration`

### **4. Tests E2E User Stories (12 tests)**
**Archivo:** `tests/e2e_user_stories.rs`

Requiere:
- ✅ Servidor Hodei gRPC
- ✅ Base de datos

Tests:
- `epic_14_hu_14_1_list_policy_stores`
- `epic_14_hu_14_2_create_policy_store`
- `epic_14_hu_14_3_get_policy_store_details`
- `epic_15_hu_15_1_view_and_edit_schema`
- `epic_15_hu_15_2_schema_real_time_validation`
- `epic_16_hu_16_1_list_and_filter_policies`
- `epic_16_hu_16_2_create_static_policy`
- `epic_16_hu_16_3_validate_policy_against_schema`
- `epic_17_hu_17_1_formulate_test_request`
- `epic_17_hu_17_2_provide_entity_data`
- `epic_17_hu_17_3_execute_simulation_and_view_results`
- `summary_all_user_stories`

### **5. Tests Internos (8 tests)**
**Archivos:** 
- `verified-permissions/main/tests/e2e_policy_store_tests.rs` (4 tests)
- `verified-permissions/main/tests/container_integration_tests.rs` (2 tests)
- `verified-permissions/main/tests/testcontainers/server_container.rs` (1 test)
- `verified-permissions/infrastructure/src/cache/policy_store_cache.rs` (1 test)

Requieren:
- ✅ Docker/Testcontainers para levantar servicios
- ✅ Servidor Hodei

---

## ✅ Tests Que SÍ Se Ejecutan

Los tests que **SÍ se ejecutan** son los **tests unitarios**:

```
✅ 38 tests passed en hodei-permissions-sdk:
   - auth_decision tests (3)
   - authorization engine tests (6)
   - entities tests (6)
   - builders tests (6)
   - validation tests (12)
   - schema tests (2)
   - middleware tests (3)

✅ 1 doc-test en hodei-permissions-sdk
```

---

## 🚀 Cómo Ejecutar Tests E2E

### **Opción 1: Ejecutar tests específicos**

```bash
# Ejecutar solo tests E2E full stack
cargo test --test e2e_full_stack -- --ignored

# Ejecutar solo tests de identity providers
cargo test --test e2e_identity_providers -- --ignored

# Ejecutar solo tests de múltiples bases de datos
cargo test --test e2e_multi_database -- --ignored
```

### **Opción 2: Ejecutar todos los tests E2E**

```bash
# Ejecutar todos los tests ignorados
cargo test -- --ignored
```

### **Opción 3: Usar Docker Compose (Recomendado)**

```bash
# Levantar todos los servicios necesarios
docker-compose up -d

# Esperar a que los servicios estén listos (2-3 minutos)
sleep 180

# Ejecutar tests E2E
cargo test -- --ignored

# Detener servicios
docker-compose down
```

---

## 📊 Resumen

| Categoría | Cantidad | Estado |
|-----------|----------|--------|
| **Tests Unitarios** | 38 | ✅ Ejecutándose |
| **Tests E2E** | 47 | ⏭️ Ignorados (requieren infraestructura) |
| **Total** | 85 | 38 ejecutándose, 47 ignorados |

---

## 🎯 Por Qué Están Ignorados

Los tests E2E están ignorados porque:

1. **Requieren infraestructura externa**: Servidores, bases de datos, servicios OIDC
2. **Son lentos**: Pueden tardar varios minutos en ejecutarse
3. **Pueden fallar por razones externas**: Servicios no disponibles, puertos ocupados, etc.
4. **No son necesarios para CI/CD rápido**: Los tests unitarios son suficientes para validar cambios de código

---

## ✨ Conclusión

- ✅ **38 tests unitarios** se ejecutan automáticamente en cada build
- ⏭️ **47 tests E2E** están disponibles pero ignorados (requieren infraestructura)
- 🎯 **Estrategia recomendada**: Ejecutar tests unitarios en CI/CD, tests E2E en ambiente de staging

