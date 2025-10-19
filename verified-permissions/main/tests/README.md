# E2E Tests con Testcontainers

Tests end-to-end completos usando Testcontainers para probar toda la funcionalidad del sistema.

## 📋 Requisitos

- Docker instalado y corriendo
- Rust 1.75+
- Imagen de Docker del servidor construida

## 🚀 Configuración

### 1. Construir la imagen de Docker para tests

```bash
./scripts/build-test-image.sh
```

Esto construye la imagen `hodei-server:test` que será usada por Testcontainers.

### 2. Verificar que Docker está corriendo

```bash
docker ps
```

## 🧪 Ejecutar Tests

### Todos los tests E2E

```bash
cargo test --test 'e2e_*' --ignored
```

### Por categoría específica

```bash
# Policy Store tests
cargo test --test e2e_policy_store_tests --ignored

# Schema tests
cargo test --test e2e_schema_tests --ignored

# Authorization tests
cargo test --test e2e_authorization_tests --ignored
```

### Con logs detallados

```bash
RUST_LOG=debug cargo test --test 'e2e_*' --ignored -- --nocapture
```

### Test específico

```bash
cargo test --test e2e_policy_store_tests test_policy_store_lifecycle --ignored -- --nocapture
```

## 📂 Estructura

```
tests/
├── testcontainers/
│   ├── mod.rs                    # Módulo principal
│   ├── server_container.rs       # Container del servidor
│   └── fixtures.rs                # Datos de prueba
├── e2e_policy_store_tests.rs     # ✅ Tests Policy Store (4 tests)
├── e2e_schema_tests.rs            # ⏳ Pendiente
├── e2e_policy_tests.rs            # ⏳ Pendiente
├── e2e_authorization_tests.rs     # ⏳ Pendiente
├── e2e_batch_tests.rs             # ⏳ Pendiente
├── e2e_identity_tests.rs          # ⏳ Pendiente
├── e2e_templates_tests.rs         # ⏳ Pendiente
├── e2e_audit_tests.rs             # ⏳ Pendiente
├── e2e_multitenancy_tests.rs      # ⏳ Pendiente
└── e2e_sdk_tests.rs               # ⏳ Pendiente
```

## ✅ Tests Implementados

### Policy Store Management (4 tests)
- ✅ `test_policy_store_lifecycle` - CRUD completo
- ✅ `test_multiple_policy_stores_isolation` - Aislamiento
- ✅ `test_policy_store_validation` - Validación
- ✅ `test_policy_store_pagination` - Paginación

## ⏳ Tests Pendientes

Ver `/docs/PLAN_TESTS_TESTCONTAINERS.md` para el plan completo de 41 tests.

## 🐛 Troubleshooting

### Error: "Cannot connect to Docker daemon"

```bash
# Verificar que Docker está corriendo
sudo systemctl start docker

# O en macOS
open -a Docker
```

### Error: "Image hodei-server:test not found"

```bash
# Construir la imagen
./scripts/build-test-image.sh
```

### Tests muy lentos

Los tests E2E son más lentos porque:
- Inician containers de Docker
- Esperan a que el servidor esté listo
- Hacen llamadas gRPC reales

Esto es normal y esperado.

## 📊 Progreso

- **Implementados**: 4/41 tests (10%)
- **Categorías completas**: 1/10
- **Próximo**: Schema Management tests

## 🎯 Próximos Pasos

1. Implementar tests de Schema (3 tests)
2. Implementar tests de Policy (5 tests)
3. Implementar tests de Authorization (6 tests)
4. Continuar con el resto de categorías
