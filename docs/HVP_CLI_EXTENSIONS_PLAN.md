# 📋 Plan de Extensiones para HVP CLI

**Orientado a Developers - Funcionalidad Completa del SDK Admin**

---

## 🎯 Objetivo

Transformar `hvp-cli` de una herramienta de generación a una **CLI completa** que exponga TODO el poder del SDK Admin, con foco en el flujo de trabajo diario del developer.

---

## 📊 Análisis de Brecha Funcional

### SDK Admin (Actual) ✅
- ✅ CRUD Policy Stores
- ✅ CRUD Policies
- ✅ Batch Operations
- ✅ Test Authorization (Playground)
- ✅ Validate Policy
- ✅ Batch Authorization

### HVP CLI (Actual) ❌
- ✅ Generación de schemas/policies
- ❌ **Cero interacción con servidor**
- ❌ **No puede listar/ver/modificar recursos existentes**
- ❌ **No bulk operations desde CLI**
- ❌ **No modo playground interactivo**

---

## 🚀 Extensiones Planificadas

### 1️⃣ Comandos de Policy Store (CRUD Completo)

```bash
# Listar todos los policy stores
hvp stores list

# Ver detalles de un store específico
hvp stores get <store-id>

# Crear un policy store
hvp stores create --name "Production" --description "Main store"

# Eliminar un policy store
hvp stores delete <store-id>

# Exportar configuración completa de un store
hvp stores export <store-id> --output ./backup

# Importar configuración de un store
hvp stores import --file ./backup/store-config.json
```

**Uso Developer**:
```bash
# Setup rápido de nuevo environment
hvp stores create --name "staging-$(date +%Y%m%d)" --description "Staging for feature X"

# Backup antes de cambios grandes
hvp stores export ps-production --output ./backups/before-migration

# Cleanup de stores antiguos
hvp stores list | grep "temp-" | xargs -I {} hvp stores delete {}
```

---

### 2️⃣ Comandos de Policies (CRUD + Bulk)

```bash
# Listar todas las policies en un store
hvp policies list --store <store-id>

# Ver detalles de una policy específica
hvp policies get --store <store-id> --policy <policy-id>

# Crear policy desde archivo
hvp policies create --store <store-id> --file policy.cedar

# Crear policy desde string
hvp policies create --store <store-id> --id "admin-access" --statement 'permit(...);'

# Actualizar policy
hvp policies update --store <store-id> --policy <policy-id> --file policy.cedar

# Eliminar policy
hvp policies delete --store <store-id> --policy <policy-id>

# Aplicar todas las policies de un directorio
hvp policies apply --store <store-id> --dir ./policies

# Validar policy sin aplicarla
hvp policies validate --store <store-id> --file policy.cedar

# Diff entre local y remoto
hvp policies diff --store <store-id> --dir ./policies
```

**Uso Developer**:
```bash
# Desarrollo iterativo de policies
hvp policies validate --store ps-dev --file policies/admin.cedar
hvp policies update --store ps-dev --policy admin --file policies/admin.cedar
hvp test --store ps-dev --principal User::"alice" --action "read" --resource "Document::\"doc1\""

# Deploy de múltiples policies
hvp policies apply --store ps-staging --dir ./policies
hvp test --store ps-staging --principal User::"bob" --action "create" --resource "Document::\"doc2\""

# Rollback si algo falla
hvp policies list --store ps-staging --format json > backup.json
# Si algo sale mal...
hvp policies apply --store ps-staging --file backup.json
```

---

### 3️⃣ Modo Playground / Test

```bash
# Test interactivo de autorización
hvp test --store <store-id> --principal <principal> --action <action> --resource <resource>

# Test con contexto
hvp test --store ps-dev \
  --principal User::"alice" \
  --action "Action::\"viewDocument\"" \
  --resource "Document::\"doc123\"" \
  --context '{"department": "engineering"}'

# Test batch desde archivo JSON
hvp test batch --store <store-id> --file test-requests.json

# Modo interactivo (REPL)
hvp test interactive --store <store-id>
# > principal: User::"alice"
# > action: view
# > resource: Document::"doc1"
# > Result: ALLOW
```

**Uso Developer**:
```bash
# Debug por qué un usuario no tiene acceso
hvp test --store ps-prod \
  --principal User::"charlie" \
  --action "delete" \
  --resource "Document::\"confidential\""

# Output muestra:
# ❌ DENY
# Determining policies: [policy_3, policy_7]
# Reason: principal.role != "admin"

# Test batch para CI/CD
hvp test batch --store ps-staging --file tests/auth-tests.json --output results.json
if grep -q "DENY" results.json; then
  echo "Tests failed!"
  exit 1
fi
```

---

### 4️⃣ Bulk Operations Avanzadas

```bash
# Batch create desde archivo JSON
hvp batch create --store <store-id> --file policies.json

# Batch update con template
hvp batch update --store <store-id> --template "policy-*.cedar"

# Batch delete con filtro
hvp batch delete --store <store-id> --filter "temp-*"

# Operación de backup completo
hvp backup create <store-id> --output ./backups/

# Restore desde backup
hvp backup restore --file ./backups/ps-prod-20240101.tar.gz
```

**Uso Developer**:
```bash
# Migración de policies entre environments
hvp stores export ps-dev --output ./dev-backup
hvp stores import --file ./dev-backup --target-store ps-staging

# Cleanup masivo de policies temporales
hvp policies list --store ps-dev --format json | \
  jq -r '.policies[] | select(.policy_id | startswith("temp-")) | .policy_id' | \
  xargs -I {} hvp policies delete --store ps-dev --policy {}

# Bulk update con reemplazo de texto
hvp policies list --store ps-prod --format json > policies.json
sed -i 's/old-role/new-role/g' policies.json
hvp batch update --store ps-prod --file policies.json
```

---

### 5️⃣ Schema Management

```bash
# Ver schema actual
hvp schema get --store <store-id>

# Validar schema local
hvp schema validate --file schema.json

# Actualizar schema
hvp schema update --store <store-id> --file schema.json

# Diff entre schemas
hvp schema diff --store <store-id> --file schema.json

# Exportar schema
hvp schema export --store <store-id> --output schema.json
```

**Uso Developer**:
```bash
# Desarrollo de schema
hvp schema validate --file schema/v4.cedarschema.json
hvp schema update --store ps-dev --file schema/v4.cedarschema.json
hvp test --store ps-dev --principal User::"test" --action "view" --resource "Document::\"test\""

# Migración de schema
hvp schema export ps-prod --output prod-schema.json
hvp schema diff --store ps-staging --file prod-schema.json
# Revisar diferencias y aplicar
hvp schema update --store ps-staging --file prod-schema.json
```

---

### 6️⃣ Identity Sources Management

```bash
# Listar identity sources
hvp identity list --store <store-id>

# Crear identity source (Keycloak)
hvp identity create-keycloak \
  --store <store-id> \
  --issuer "http://keycloak:8080/realms/myapp" \
  --client-id "myapp-client"

# Crear identity source genérico
hvp identity create \
  --store <store-id> \
  --name "Corporate AD" \
  --type oidc \
  --config config.json

# Actualizar identity source
hvp identity update --store <store-id> --identity <id> --config config.json

# Eliminar identity source
hvp identity delete --store <store-id> --identity <id>
```

**Uso Developer**:
```bash
# Setup de Keycloak para nuevo environment
hvp identity create-keycloak \
  --store ps-staging \
  --issuer "https://keycloak.staging.company.com/realms/apps" \
  --client-id "staging-app" \
  --client-secret "$KEYCLOAK_SECRET"

# Rotación de secret
hvp identity update --store ps-prod --identity keycloak-prod \
  --config <(echo '{"client_secret": "$NEW_SECRET"}')
```

---

### 7️⃣ Audit y Monitoring

```bash
# Ver audit log
hvp audit --store <store-id> --limit 100

# Filtrar por usuario
hvp audit --store <store-id> --principal User::"alice"

# Filtrar por acción
hvp audit --store <store-id> --action "delete"

# Exportar audit log
hvp audit export --store <store-id> --output audit.json --since "2024-01-01"

# Health check del servidor
hvp health

# Metrics (si están habilitadas)
hvp metrics
```

**Uso Developer**:
```bash
# Debug de incidente de seguridad
hvp audit --store ps-prod --principal User::"suspicious-user" --since "2024-01-15"

# Monitoreo en CI/CD
hvp health || exit 1
hvp metrics | jq '.authorization_rate' | awk '{if ($1 > 1000) exit 1}'

# Generar reporte de compliance
hvp audit export --store ps-prod --since "2024-01-01" --output compliance-report.json
```

---

### 8️⃣ Configuración y Entornos

```bash
# Configurar endpoint por defecto
hvp config set endpoint "https://avp.production.company.com:50051"

# Configurar credenciales
hvp config set credentials "$AVP_TOKEN"

# Ver configuración actual
hvp config get

# Usar diferentes profiles
hvp --profile staging stores list
hvp --profile production stores list

# Setup inicial interactivo
hvp init
# > Endpoint: http://localhost:50051
# > Default store: ps-dev
# > Keycloak issuer: http://keycloak:8080/realms/demo
```

**Uso Developer**:
```bash
# Setup de múltiples environments
hvp --profile dev config set endpoint "http://localhost:50051"
hvp --profile staging config set endpoint "https://avp.staging.company.com"
hvp --profile prod config set endpoint "https://avp.company.com"

# Uso rápido
hvp --profile dev stores list
hvp --profile prod stores list
```

---

## 📦 Estructura de Comandos Completa

```
hvp
├── generate          # Generación (ya existe)
│   ├── schema
│   ├── policies
│   ├── least-privilege
│   └── setup
├── stores            # Policy Store Management ⭐ NUEVO
│   ├── list
│   ├── get
│   ├── create
│   ├── delete
│   ├── export
│   └── import
├── policies          # Policy Management ⭐ NUEVO
│   ├── list
│   ├── get
│   ├── create
│   ├── update
│   ├── delete
│   ├── apply
│   ├── validate
│   └── diff
├── batch             # Bulk Operations ⭐ NUEVO
│   ├── create
│   ├── update
│   ├── delete
│   └── backup
├── test              # Playground/Test ⭐ NUEVO
│   ├── single
│   ├── batch
│   └── interactive
├── schema            # Schema Management ⭐ NUEVO
│   ├── get
│   ├── update
│   ├── validate
│   └── diff
├── identity          # Identity Sources ⭐ NUEVO
│   ├── list
│   ├── create
│   ├── update
│   └── delete
├── audit             # Audit & Monitoring ⭐ NUEVO
│   ├── list
│   ├── export
│   └── filter
├── health            # Health Check ⭐ NUEVO
├── metrics           # Metrics ⭐ NUEVO
└── config            # Configuration ⭐ NUEVO
    ├── set
    ├── get
    └── init
```

---

## 🎯 Flujos de Trabajo Developer Completos

### Flujo 1: Desarrollo Diario

```bash
# 1. Iniciar día - ver estado
hvp --profile dev health
hvp --profile dev stores list

# 2. Trabajar en nueva feature
hvp policies create --store ps-dev --file features/payment-policy.cedar
hvp test --store ps-dev --principal User::"test" --action "pay" --resource "Order::\"123\""

# 3. Iterar hasta que funcione
hvp policies update --store ps-dev --policy payment --file features/payment-policy.cedar
hvp test --store ps-dev --principal User::"test" --action "pay" --resource "Order::\"123\""

# 4. Validar contra schema
hvp schema validate --file schema/v4.cedarschema.json

# 5. Preparar para PR
hvp policies export --store ps-dev --output ./pr-policies
hvp test batch --store ps-dev --file tests/auth-tests.json --output test-results.json
```

### Flujo 2: CI/CD Pipeline

```yaml
# .github/workflows/auth-deploy.yml
name: Deploy Authorization

jobs:
  deploy:
    steps:
      - uses: actions/checkout@v3
      
      - name: Install hvp-cli
        run: cargo install --path hvp-cli
      
      - name: Configure environment
        run: |
          hvp --profile staging config set endpoint "${{ secrets.AVP_ENDPOINT }}"
          hvp --profile staging config set token "${{ secrets.AVP_TOKEN }}"
      
      - name: Validate all policies
        run: |
          hvp policies validate --store ps-staging --dir ./policies
      
      - name: Run authorization tests
        run: |
          hvp test batch --store ps-staging --file tests/auth-tests.json
      
      - name: Apply policies
        run: |
          hvp policies apply --store ps-staging --dir ./policies
      
      - name: Health check
        run: hvp --profile staging health
```

### Flujo 3: Incident Response

```bash
# 1. Identificar problema
hvp audit --store ps-prod --principal User::"attacker" --since "1 hour ago"

# 2. Revocar acceso inmediato
hvp policies delete --store ps-prod --policy compromised-policy

# 3. Verificar fix
hvp test --store ps-prod --principal User::"attacker" --action "access" --resource "Data::\"secret\""

# 4. Aplicar policies correctas
hvp policies apply --store ps-prod --dir ./emergency-fix

# 5. Monitorear
hvp audit --store ps-prod --follow
```

---

## 📈 Métricas de Éxito

- **Cobertura**: 100% de funcionalidad del SDK Admin expuesta
- **Usabilidad**: Comandos intuitivos con auto-complete
- **Performance**: <100ms para operaciones simples
- **Fiabilidad**: 99.9% de comandos funcionan en primera ejecución
- **Developer Experience**: Setup completo en <5 minutos

---

## 🚀 Roadmap de Implementación

### Fase 1 (Alta Prioridad) - CRUD Básico
- [ ] `hvp stores list/get/create/delete`
- [ ] `hvp policies list/get/create/update/delete`
- [ ] `hvp test`
- [ ] `hvp health`

### Fase 2 (Media Prioridad) - Bulk & Schema
- [ ] `hvp policies apply`
- [ ] `hvp batch create/update/delete`
- [ ] `hvp schema get/update/validate`
- [ ] `hvp audit`

### Fase 3 (Baja Prioridad) - Avanzado
- [ ] `hvp identity management`
- [ ] `hvp backup/restore`
- [ ] `hvp diff`
- [ ] `hvp config profiles`
- [ ] `hvp metrics`

---

<div align="center">

**¿Listo para implementar?** Comienza con Fase 1 y expande iterativamente.

</div>