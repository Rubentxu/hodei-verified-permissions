# 📋 Plan de Tests con Testcontainers

**Objetivo**: Cobertura completa E2E usando Testcontainers

## 🎯 Estrategia

### Arquitectura
```
Test Suite → Testcontainer (gRPC Server + SQLite) → SDK Client → Validación
```

### Dependencias Necesarias
```toml
[dev-dependencies]
testcontainers = "0.15"
tokio-test = "0.4"
```

## 🧪 Suite de Tests (10 Categorías)

### 1. Policy Store Management (4 tests)
- `test_policy_store_lifecycle` - CRUD completo
- `test_multiple_stores_isolation` - Aislamiento entre stores
- `test_store_validation` - Validación de datos
- `test_store_pagination` - Paginación de listados

### 2. Schema Management (3 tests)
- `test_schema_upload_and_validation` - Upload y validación
- `test_schema_with_complex_types` - Tipos complejos
- `test_schema_update_and_versioning` - Actualización

### 3. Policy Management (5 tests)
- `test_policy_crud_operations` - CRUD completo
- `test_policies_with_conditions` - Condiciones when/unless
- `test_policy_validation_against_schema` - Validación con schema
- `test_forbid_policies` - Políticas forbid
- `test_policy_conflicts` - Resolución de conflictos

### 4. Authorization Basic (6 tests)
- `test_basic_allow_decision` - Decisión ALLOW
- `test_basic_deny_decision` - Decisión DENY
- `test_authorization_with_entities` - Con entidades
- `test_authorization_with_hierarchies` - Con jerarquías
- `test_authorization_with_context` - Con context
- `test_authorization_with_attributes` - Con atributos

### 5. Batch Operations (3 tests)
- `test_batch_basic` - Batch básico
- `test_batch_30_requests_limit` - Límite de 30
- `test_batch_performance` - Performance

### 6. Identity Integration (5 tests)
- `test_create_identity_source_oidc` - OIDC source
- `test_create_identity_source_cognito` - Cognito source
- `test_is_authorized_with_valid_jwt` - JWT válido
- `test_is_authorized_with_invalid_jwt` - JWT inválido
- `test_claims_mapping` - Mapeo de claims

### 7. Policy Templates (4 tests)
- `test_create_policy_template` - Crear template
- `test_template_linked_policy` - Política desde template
- `test_share_document_use_case` - Caso de uso compartir
- `test_template_validation` - Validación de placeholders

### 8. Audit Logging (3 tests)
- `test_audit_logging_basic` - Logging básico
- `test_audit_log_filtering` - Filtrado de logs
- `test_audit_log_retention` - Retención de logs

### 9. Multi-Tenancy (3 tests)
- `test_policy_store_isolation` - Aislamiento por store
- `test_logical_isolation_pattern` - Aislamiento lógico
- `test_tenant_data_separation` - Separación de datos

### 10. SDK Integration (5 tests)
- `test_sdk_connection_handling` - Manejo de conexiones
- `test_sdk_error_handling` - Manejo de errores
- `test_sdk_retry_logic` - Lógica de reintentos
- `test_sdk_builders` - Builders del SDK
- `test_sdk_concurrent_requests` - Requests concurrentes

## 📝 Estructura de Archivos

```
tests/
├── testcontainers/
│   ├── mod.rs                    # Helpers para containers
│   ├── server_container.rs       # Container del servidor
│   └── fixtures.rs                # Datos de prueba
├── e2e_policy_store_tests.rs     # Tests categoría 1
├── e2e_schema_tests.rs            # Tests categoría 2
├── e2e_policy_tests.rs            # Tests categoría 3
├── e2e_authorization_tests.rs     # Tests categoría 4
├── e2e_batch_tests.rs             # Tests categoría 5
├── e2e_identity_tests.rs          # Tests categoría 6
├── e2e_templates_tests.rs         # Tests categoría 7
├── e2e_audit_tests.rs             # Tests categoría 8
├── e2e_multitenancy_tests.rs      # Tests categoría 9
└── e2e_sdk_tests.rs               # Tests categoría 10
```

## 🔧 Implementación Base

### Helper: Server Container
```rust
// tests/testcontainers/server_container.rs
pub async fn start_server_container() -> Container {
    let container = GenericImage::new("hodei-server", "latest")
        .with_exposed_port(50051)
        .with_env_var("DATABASE_URL", "sqlite::memory:")
        .start()
        .await;
    
    wait_for_server_ready(&container).await;
    container
}
```

### Helper: SDK Client
```rust
pub async fn create_sdk_client(container: &Container) -> AuthorizationClient {
    let port = container.get_host_port(50051).await;
    let url = format!("http://localhost:{}", port);
    AuthorizationClient::connect(url).await.unwrap()
}
```

### Helper: Setup Fixtures
```rust
pub async fn setup_store_with_schema(client: &AuthorizationClient) -> PolicyStore {
    let store = client.create_policy_store("Test Store").await.unwrap();
    let schema = include_str!("../fixtures/basic_schema.json");
    client.put_schema(&store.policy_store_id, schema).await.unwrap();
    store
}
```

## 📊 Cobertura Esperada

| Componente | Tests | Cobertura |
|------------|-------|-----------|
| Policy Store | 4 | 100% |
| Schema | 3 | 100% |
| Policy | 5 | 100% |
| Authorization | 6 | 100% |
| Batch | 3 | 100% |
| Identity | 5 | 100% |
| Templates | 4 | 100% |
| Audit | 3 | 100% |
| Multi-Tenancy | 3 | 100% |
| SDK | 5 | 100% |
| **TOTAL** | **41** | **100%** |

## 🚀 Ejecución

```bash
# Todos los tests E2E
cargo test --test 'e2e_*'

# Por categoría
cargo test --test e2e_authorization_tests

# Con logs
RUST_LOG=debug cargo test --test 'e2e_*' -- --nocapture
```

## ✅ Checklist de Implementación

- [ ] Configurar testcontainers en Cargo.toml
- [ ] Crear Dockerfile para servidor de tests
- [ ] Implementar helpers de testcontainers
- [ ] Implementar tests categoría 1 (Policy Store)
- [ ] Implementar tests categoría 2 (Schema)
- [ ] Implementar tests categoría 3 (Policy)
- [ ] Implementar tests categoría 4 (Authorization)
- [ ] Implementar tests categoría 5 (Batch)
- [ ] Implementar tests categoría 6 (Identity)
- [ ] Implementar tests categoría 7 (Templates)
- [ ] Implementar tests categoría 8 (Audit)
- [ ] Implementar tests categoría 9 (Multi-Tenancy)
- [ ] Implementar tests categoría 10 (SDK)
- [ ] Configurar CI/CD para ejecutar tests E2E
- [ ] Documentar resultados y cobertura

## 🎯 Resultado Final

**74 tests totales** (33 actuales + 41 E2E) con cobertura 100% de funcionalidad.
