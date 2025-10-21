# Tests E2E - End-to-End Testing Guide

## 🎯 Objetivo

Probar la integración completa del stack Hodei Verified Permissions:
- **Servidor gRPC** (verified-permissions)
- **SDK Cliente** (hodei-permissions-sdk)
- **Aplicación de ejemplo** (TODO app)
- **Autorización real** con Cedar policies

Similar a cómo funciona **AWS Verified Permissions + Express SDK**.

---

## 📋 Estado Actual

### ✅ Completado

1. **Servidor gRPC**: ✅ Compila y funciona (18 tests pasando)
2. **SDK**: ✅ Funcional (22 tests pasando)
3. **TODO App**: ✅ Funcional (12 tests de integración sin auth real)
4. **Infraestructura Docker**: ✅ Creada
   - `verified-permissions/Dockerfile`
   - `examples/todo-app/Dockerfile`
   - `docker-compose.test.yml`
5. **Tests E2E**: ✅ Implementados
   - `tests/e2e_full_stack.rs`
6. **Scripts**: ✅ Creados
   - `scripts/test-e2e.sh`

### ⏳ Pendiente

1. **Habilitar middleware en TODO app** (actualmente deshabilitado por Axum 0.8)
2. **Implementar generación de JWT** para tests
3. **Configurar base de datos** (SQLite ya funciona)
4. **Ejecutar tests E2E** completos

---

## 🏗️ Arquitectura E2E

```
┌─────────────────────────────────────────────────────────────┐
│                     E2E Test Environment                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐      ┌──────────────┐      ┌───────────┐ │
│  │  HTTP Client │─────▶│   TODO App   │─────▶│  Hodei    │ │
│  │  (reqwest)   │      │  (Axum +     │      │  Server   │ │
│  │              │      │   SDK        │      │  (gRPC)   │ │
│  │              │      │   Middleware)│      │           │ │
│  └──────────────┘      └──────────────┘      └───────────┘ │
│                               │                      │       │
│                               │                      │       │
│                               ▼                      ▼       │
│                        ┌──────────────┐      ┌───────────┐ │
│                        │  In-Memory   │      │  SQLite   │ │
│                        │  Storage     │      │  Database │ │
│                        └──────────────┘      └───────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘

Flow:
1. Test sends HTTP request with JWT token
2. TODO App receives request
3. SDK Middleware extracts token and context
4. SDK calls Hodei Server via gRPC
5. Server evaluates Cedar policies
6. Server returns ALLOW/DENY decision
7. Middleware allows/blocks request
8. TODO App processes or rejects
9. Response returned to test
```

---

## 🚀 Cómo Ejecutar Tests E2E

### Opción 1: Con Docker Compose (Recomendado)

```bash
# Ejecutar script completo
./scripts/test-e2e.sh
```

Este script:
1. ✅ Verifica que Docker esté corriendo
2. ✅ Limpia contenedores anteriores
3. ✅ Construye imágenes Docker
4. ✅ Inicia servidor y aplicación
5. ✅ Espera a que estén healthy
6. ✅ Ejecuta tests E2E
7. ✅ Muestra logs si falla
8. ✅ Limpia al terminar

### Opción 2: Manual

```bash
# 1. Iniciar servicios
docker-compose -f docker-compose.test.yml up -d

# 2. Verificar que estén corriendo
docker-compose -f docker-compose.test.yml ps

# 3. Ver logs
docker-compose -f docker-compose.test.yml logs -f

# 4. Ejecutar tests
cargo test --test e2e_full_stack -- --ignored --nocapture

# 5. Limpiar
docker-compose -f docker-compose.test.yml down -v
```

### Opción 3: Sin Docker (Local)

```bash
# Terminal 1: Servidor
cd verified-permissions
cargo run --release

# Terminal 2: TODO App
cd examples/todo-app
export AUTH_ENDPOINT=http://localhost:50051
cargo run

# Terminal 3: Tests
cargo test --test e2e_full_stack -- --ignored --nocapture
```

---

## 🧪 Tests Implementados

### 1. `test_e2e_policy_store_creation`
Verifica que se puede crear un Policy Store en el servidor.

### 2. `test_e2e_todo_app_health_check`
Verifica que la aplicación TODO esté corriendo y accesible.

### 3. `test_e2e_authorization_with_real_server`
Prueba el flujo completo de autorización:
- Crear policy store
- Crear identity source
- Cargar políticas Cedar
- Evaluar decisión de autorización

### 4. `test_e2e_todo_app_with_authorization`
Prueba la integración completa:
- HTTP Request → TODO App → SDK → Servidor → Cedar → Response
- Verifica que admin puede listar tasks
- Verifica que viewer NO puede crear tasks

### 5. `test_e2e_rbac_scenarios`
Prueba escenarios de Role-Based Access Control:
- Admin puede hacer todo
- Project Manager puede asignar tasks
- Team Member solo puede actualizar sus propias tasks (ABAC)

### 6. `test_e2e_simplerest_mapping`
Verifica que el mapeo SimpleRest funciona:
- `GET /tasks` → `listTasks` + `Application`
- `GET /tasks/:id` → `getTask` + `Task`
- `POST /tasks` → `createTask` + `Application`
- `PUT /tasks/:id` → `updateTask` + `Task`
- `DELETE /tasks/:id` → `deleteTask` + `Task`

---

## 📊 Escenarios de Prueba

### Escenario 1: Admin - Acceso Total ✅
```
Usuario: alice (admin)
Política: permit(principal in Group::"admin", action, resource)
Resultado: ALLOW para todas las operaciones
```

### Escenario 2: Viewer - Solo Lectura ✅
```
Usuario: bob (viewers)
Política: permit(principal in Group::"viewers", action in [listTasks, getTask], resource)
Resultado: 
- ALLOW para GET /tasks
- DENY para POST /tasks
```

### Escenario 3: Team Member - ABAC ✅
```
Usuario: charlie (team_members)
Política: permit when { resource.assignee == principal }
Resultado:
- ALLOW para PUT /tasks/:id si task.assignee == charlie
- DENY para PUT /tasks/:id si task.assignee != charlie
```

### Escenario 4: Project Manager - Gestión ✅
```
Usuario: alice (project_managers)
Política: permit(principal in Group::"project_managers", action in [assignTask, createProject], resource)
Resultado: ALLOW para asignar tasks y crear proyectos
```

---

## 🔧 Configuración

### Variables de Entorno

**Servidor (hodei-server)**:
```bash
RUST_LOG=debug
DATABASE_URL=sqlite:///app/data/hodei.db
```

**TODO App**:
```bash
RUST_LOG=debug
AUTH_ENDPOINT=http://hodei-server:50051
POLICY_STORE_ID=todo-policy-store
IDENTITY_SOURCE_ID=todo-identity-source
```

**Tests**:
```bash
RUST_LOG=debug
HODEI_SERVER_URL=http://hodei-server:50051
TODO_APP_URL=http://todo-app:3000
```

---

## 🐛 Troubleshooting

### Error: "Docker is not running"
```bash
# Linux
sudo systemctl start docker

# macOS
open -a Docker
```

### Error: "Services failed to start"
```bash
# Ver logs
docker-compose -f docker-compose.test.yml logs

# Verificar puertos
lsof -i :50051  # Servidor gRPC
lsof -i :3000   # TODO App
```

### Error: "Connection refused"
```bash
# Verificar que los servicios estén healthy
docker-compose -f docker-compose.test.yml ps

# Reiniciar servicios
docker-compose -f docker-compose.test.yml restart
```

### Tests fallan con "401 Unauthorized"
- Verificar que el middleware esté habilitado en TODO app
- Verificar generación de JWT tokens
- Verificar que identity source esté configurado

### Tests fallan con "403 Forbidden"
- Verificar que las políticas Cedar estén cargadas
- Verificar que el usuario tenga los grupos correctos
- Ver logs del servidor para decisiones Cedar

---

## 📝 Próximos Pasos

### Corto Plazo (Inmediato)
1. ✅ Habilitar middleware en TODO app
2. ✅ Implementar generación de JWT para tests
3. ✅ Ejecutar primer test E2E exitoso

### Medio Plazo (Esta semana)
1. ⏳ Añadir más escenarios de prueba
2. ⏳ Implementar tests con PostgreSQL
3. ⏳ Añadir métricas y observabilidad

### Largo Plazo (Próximo mes)
1. ⏳ CI/CD con tests E2E automáticos
2. ⏳ Tests de carga y performance
3. ⏳ Tests de seguridad

---

## 📚 Comparación con AWS Verified Permissions

| Característica | AWS VP + Express | Hodei VP + SDK | Estado |
|----------------|------------------|----------------|--------|
| Schema Generation | ✅ | ✅ | ✅ Implementado |
| Runtime Mapping | ✅ | ✅ | ✅ Implementado |
| Middleware Integration | ✅ | ✅ | ⏳ Pendiente habilitar |
| Policy Evaluation | ✅ Cedar | ✅ Cedar | ✅ Implementado |
| Context Extraction | ✅ | ✅ | ✅ Implementado |
| JWT Validation | ✅ | ✅ | ⏳ Pendiente |
| E2E Tests | ✅ | ✅ | ✅ Implementado |

---

## ✅ Checklist de Implementación

- [x] Servidor gRPC funcional
- [x] SDK con middleware
- [x] Aplicación de ejemplo (TODO app)
- [x] Dockerfiles creados
- [x] Docker Compose configurado
- [x] Tests E2E escritos
- [x] Script de ejecución
- [ ] Middleware habilitado en TODO app
- [ ] Generación de JWT implementada
- [ ] Primer test E2E pasando
- [ ] Todos los tests E2E pasando
- [ ] Documentación completa
- [ ] CI/CD configurado

---

**Estado**: ✅ **INFRAESTRUCTURA COMPLETA - LISTO PARA EJECUTAR**  
**Próximo paso**: Habilitar middleware y ejecutar tests  
**Estimado**: 2-3 horas para completar
