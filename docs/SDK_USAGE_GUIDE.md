# Guía de Uso: Cuándo Usar Cada SDK/Herramienta

**Fecha:** 2025-11-04
**Tema:** Arquitectura de Herramientas para Verified Permissions

---

## 🎯 Arquitectura Recomendada: Separación de Responsabilidades

```
┌─────────────────────────────────────────────────────────────┐
│                    DESARROLLO (Development)                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────┐     │
│  │ SDK Cliente │    │   CLI Tool   │    │ Web Console │     │
│  │             │    │              │    │             │     │
│  │ Data Plane  │    │  Management  │    │ Management  │     │
│  │ Only        │    │  (Automated) │    │ (Visual)    │     │
│  └─────────────┘    └──────────────┘    └─────────────┘     │
│         │                    │                    │         │
│         └────────────────────┼────────────────────┘         │
│                              │                               │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                  PRODUCCIÓN (Production)                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────┐     │
│  │ SDK Cliente │    │   Scripts    │    │ Admin API   │     │
│  │             │    │   (CI/CD)    │    │ (Opcional)  │     │
│  │ Data Plane  │    │  (Promoted)  │    │  (Ops)      │     │
│  │ Only        │    │              │    │             │     │
│  └─────────────┘    └──────────────┘    └─────────────┘     │
│         │                    │                    │         │
│         └────────────────────┼────────────────────┘         │
│                              │                               │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
                    ┌─────────────────────────┐
                    │   Hodei Server gRPC     │
                    │   (Both Data+Control)   │
                    └─────────────────────────┘
```

---

## 🔑 CUÁNDO USAR: SDK Cliente (Solo Data Plane)

### Casos de Uso

| Escenario | ¿Por qué SDK? | Ejemplo |
|-----------|---------------|---------|
| **API REST** | Verificar permisos en request/response | Express, Axum, Fastify |
| **Microservicio** | Authorization como sidecar o libreria | Rust, Node.js, Python |
| **Middleware** | Protección automática de endpoints | Tower, Express middleware |
| **Lambda/Serverless** | Verificación rápida de permisos | AWS Lambda, Cloudflare Workers |
| **CLI Tool** | Verificar permisos antes de ejecutar | `hodei user list --require-role=admin` |
| **Testing** | Mock authorization en tests | Unit tests, integration tests |

### Ejemplos Reales

#### API REST con Express/Axum

```typescript
// SDK CLIENTE (Data Plane only)
import { AVPAuthorizationEngine } from '@verifiedpermissions/authorization-clients-js';

const engine = new AVPAuthorizationEngine({
    policyStoreId: 'ps123',
    callType: 'isAuthorized'
});

// En middleware
app.use(async (req, res, next) => {
    const request = {
        principal: { type: 'User', id: req.user.id },
        action: { type: 'Action', id: req.method },
        resource: { type: 'Resource', id: req.path },
        context: { ip: req.ip }
    };

    const result = await engine.isAuthorized(request, []);

    if (result.type === 'deny') {
        return res.status(403).send('Forbidden');
    }

    req.principal = result.authorizerInfo?.principalUid;
    next();
});
```

#### Microservicio Rust

```rust
// SDK CLIENTE (Data Plane only)
use hodei_permissions_sdk::AuthorizationClient;

struct UserService {
    auth_client: AuthorizationClient,
    policy_store_id: String,
}

impl UserService {
    async fn get_user(&self, user_id: &str, caller_id: &str) -> Result<User> {
        // Verificar que el caller puede ver este usuario
        self.auth_client
            .is_authorized(
                &self.policy_store_id,
                &format!("User::{}", caller_id),
                "Action::read",
                &format!("User::{}", user_id)
            )
            .await?;

        // Lógica de negocio
        Ok(self.repository.get_user(user_id).await?)
    }
}
```

#### Lambda/Serverless

```typescript
// SDK CLIENTE (Data Plane only)
import { AuthorizationClient } from 'hodei-permissions-sdk';

export const handler = async (event: APIGatewayProxyEvent) => {
    const client = new AuthorizationClient(process.env.AVP_ENDPOINT!);

    // Verificar en cada request
    const decision = await client.is_authorized(
        process.env.POLICY_STORE_ID!,
        event.requestContext.authorizer?.principalId,
        event.httpMethod,
        event.resource
    );

    if (decision.decision() === Decision::Deny) {
        return { statusCode: 403, body: 'Forbidden' };
    }

    // Lógica de la función
    return { statusCode: 200, body: JSON.stringify({ data: '...' }) };
};
```

#### Testing

```rust
// SDK CLIENTE (Data Plane only)
use hodei_permissions_sdk::client_trait::AuthorizationClientTrait;

// Mock para tests
struct MockAuthClient;

#[async_trait]
impl AuthorizationClientTrait for MockAuthClient {
    async fn is_authorized(&self, ...) -> Result<IsAuthorizedResponse> {
        Ok(IsAuthorizedResponse {
            decision: Decision::Allow as i32,
            determining_policies: vec![],
            errors: vec![],
        })
    }
}

// Usar en test
#[tokio::test]
async fn test_user_creation() {
    let mock_client = MockAuthClient;
    let service = UserService::new(mock_client);

    let result = service.create_user("alice").await;
    assert!(result.is_ok());
}
```

### ✅ Cuándo Usar SDK Cliente

- ✅ **Tu aplicación necesita verificar permisos** (cualquier framework)
- ✅ **Performance es crítica** (gRPC directo)
- ✅ **Tests unitarios/integration**
- ✅ **Middleware/authorization layer**
- ✅ **Microservicios**
- ✅ **Serverless functions**

---

## 🛠️ CUÁNDO USAR: CLI Tool (Control Plane)

### Casos de Uso

| Escenario | ¿Por qué CLI? | Ejemplo |
|-----------|---------------|---------|
| **Setup inicial** | Crear policy stores, schemas | `hodei init` |
| **CI/CD** | Deploy automático de políticas | GitHub Actions |
| **Bulk operations** | Importar/actualizar políticas masivamente | `hodei policies import policies.json` |
| **DevOps** | Automatización de infraestructura | Terraform provider |
| **Testing** | Setup/teardown de test environments | `hodei test setup` |
| **Migration** | Migrar desde otro sistema | `hodei migrate from-auth0` |

### Ejemplos Reales

#### Setup Inicial de Proyecto

```bash
# CLI TOOL (Control Plane only)
hodei init my-app
# Crear policy store automáticamente
# Generar schema template
# Configurar identity sources

hodei schema apply --file=schema.cedar.json
# Upload schema al policy store

hodei policies import --dir=policies/
# Importar todas las políticas

hodei identity-source create keycloak \
  --issuer=https://keycloak.example.com/realms/myapp \
  --client-id=myapp
# Configurar IdP
```

#### CI/CD Pipeline

```yaml
# .github/workflows/deploy-authorization.yml
name: Deploy Authorization

on:
  push:
    branches: [main]
    paths: ['policies/**', 'schema.json']

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Hodei CLI
        run: npm install -g @hodei/cli

      - name: Deploy Schema
        run: hodei schema apply --file=schema.json

      - name: Deploy Policies
        run: hodei policies import --dir=policies/

      - name: Run Tests
        run: hodei test authorize --store-id=${{ secrets.POLICY_STORE_ID }}
```

#### Bulk Import

```bash
# Importar 100+ políticas desde archivo JSON
hodei policies import \
  --file=./policies/bulk-import.json \
  --validate \
  --dry-run

# Output:
# ✓ Validating policies...
# ✓ 150 policies validated
# ✓ Creating policy store...
# ✓ Uploading policies...
# ✓ 150/150 policies created
# ✓ Done in 5.2s
```

#### DevOps Automation

```bash
# Script de infraestructura como código
#!/bin/bash

# Crear environment
hodei env create staging \
  --policy-store-name="App-Staging" \
  --identity-source=keycloak

# Promover políticas desde dev a staging
hodei policies promote \
  --from=dev-environment \
  --to=staging-environment \
  --filter="tag:staging"

# Configurar monitoring
hodei monitoring setup \
  --policy-store-id=$POLICY_STORE_ID \
  --alerts-enabled
```

#### Testing Environments

```bash
# Setup test environment
hodei test setup \
  --store-name="Test-$(date +%s)" \
  --with-policies=./test/policies/ \
  --with-schema=./test/schema.json

# Run integration tests
cargo test --integration

# Cleanup
hodei test cleanup --store-id=$TEST_STORE_ID
```

### ✅ Cuándo Usar CLI Tool

- ✅ **Setup inicial** de proyectos
- ✅ **CI/CD pipelines** para deploy automático
- ✅ **Bulk operations** (importar 100+ políticas)
- ✅ **DevOps automation** (Terraform, Ansible)
- ✅ **Testing environments** (setup/teardown)
- ✅ **Migration** entre sistemas
- ✅ **Administrative tasks** (no en hot path)

---

## 🎨 CUÁNDO USAR: Web Console (Control Plane)

### Casos de Uso

| Escenario | ¿Por qué Web Console? | Ejemplo |
|-----------|----------------------|---------|
| **Exploración visual** | Ver policies, schemas gráficamente | Developers explorando permisos |
| **Debugging** | Ver qué políticas matched | "Por qué me denegó?" |
| **Colaboración** | PMs/Devs revisan políticas juntos | Policy review sessions |
| **Training** | Enseñar autorización a nuevos devs | Onboarding |
| **Auditoría** | Revisar cambios históricos | Compliance checks |
| **One-off tasks** | Cambios poco frecuentes | "Solo cambiar este policy" |

### Ejemplos Reales

#### Developer Debugging

```
1. Developer recibe 403 en endpoint
2. Abre Web Console
3. Ve en tiempo real: "alice tried to access Document::123"
4. Ve qué políticas se evaluaron
5. Ve que la política "deny-contractors" matched
6. Entiende por qué fue denegado
7. Ajusta la política visualmente
8. Testea inmediatamente
```

#### Policy Review Session

```
PM: "¿Por qué los managers pueden ver todos los documentos?"
Dev: "Vamos a la consola... Aquí está la política..."

[En la Web Console]
PM: "Ah, veo que la política #45 permite esto"
PM: "¿Podemos ser más específicos?"
Dev: "Claro, cambiemos para que solo vean sus propios dept..."

[Editan visualmente]
Dev: "Testear..."
[Verificación en tiempo real]
PM: "Perfecto, ahora los managers ven solo su dept"
```

#### Auditoría/Compliance

```
Auditor: "Necesito ver todos los cambios de políticas del último mes"

[En Web Console]
- Filtra por fecha: último mes
- Filtra por acción: solo updates
- Export a CSV
- Ve quién hizo cada cambio
- Ve el diff de cada política

Auditor: "¿Quién cambió la política de acceso a datos financieros?"
[Console muestra: "john@company.com - 2025-01-15 - Reason: 'PCI compliance update'"]
```

### ✅ Cuándo Usar Web Console

- ✅ **Desarrollo/debugging** (visual feedback)
- ✅ **Colaboración** (PMs, Devs, SecOps)
- ✅ **Capacitación** (onboarding, training)
- ✅ **Auditoría** (compliance, security reviews)
- ✅ **One-off tasks** (cambios raros)
- ✅ **Exploración** (entender el sistema)

---

## 🚀 CUÁNDO USAR: Admin SDK (Opcional)

### Casos de Uso

| Escenario | ¿Por qué Admin SDK? | Ejemplo |
|-----------|---------------------|---------|
| **Admin portal** | Build tu propia web console | Customer portal |
| **Bulk automation** | Programmatic management | Migration tools |
| **Integration** | Conectar con otros admin tools | Okta, Auth0 admin |
| **Custom workflows** | Políticas como código | CloudFormation-like |

### Ejemplo: Building Custom Admin Portal

```typescript
// ADMIN SDK (Control Plane only)
import { HodeiAdminClient } from '@hodei/admin-sdk';

const admin = new HodeiAdminClient({
    endpoint: process.env.AVP_ADMIN_ENDPOINT,
    credentials: adminCredentials
});

// Build tu propia web console
app.get('/admin/policies', async (req, res) => {
    const policies = await admin.listPolicies({
        policyStoreId: req.query.storeId,
        limit: 100
    });
    res.json(policies);
});

app.post('/admin/policies', async (req, res) => {
    const policy = await admin.createPolicy({
        policyStoreId: req.body.storeId,
        policyId: req.body.id,
        statement: req.body.cedar
    });
    res.json(policy);
});
```

### ✅ Cuándo Usar Admin SDK

- ✅ **Building admin portals** (custom UIs)
- ✅ **Bulk automation** (programmatic)
- ✅ **Integration** con other admin tools
- ⚠️ **Para la mayoría de casos, CLI + Web Console es suficiente**

---

## 📊 Matriz de Decisión: ¿Qué Usar?

### Pregunta 1: ¿Qué estás haciendo?

| Tarea | Herramienta |
|-------|-------------|
| Verificar permisos en mi app | **SDK Cliente** |
| Crear/actualizar políticas | **CLI Tool** o **Web Console** |
| Setup inicial del proyecto | **CLI Tool** |
| Debug por qué falló autorización | **Web Console** |
| Deploy en CI/CD | **CLI Tool** |
| Tests automatizados | **CLI Tool** |
| Migrar desde otro sistema | **CLI Tool** |
| Revisar políticas con el equipo | **Web Console** |
| Auditoría/compliance | **Web Console** |
| Onboarding de nuevos devs | **Web Console** |

### Pregunta 2: ¿Con qué frecuencia?

| Frecuencia | Herramienta |
|------------|-------------|
| **Muy frecuente** (cada request) | **SDK Cliente** |
| **Frecuente** (diario) | **CLI Tool** o **SDK Cliente** |
| **Ocasional** (semanal) | **Web Console** |
| **Raro** (mensual) | **Web Console** |
| **Una vez** (setup inicial) | **CLI Tool** |

### Pregunta 3: ¿Quién lo usa?

| Usuario | Herramienta |
|---------|-------------|
| **Developers** (en código) | **SDK Cliente** |
| **DevOps** (CI/CD) | **CLI Tool** |
| **Product Managers** | **Web Console** |
| **Security/Compliance** | **Web Console** |
| **New developers** (onboarding) | **Web Console** |

---

## ✅ Resumen: Reglas de Oro

### 1️⃣ SDK Cliente (Data Plane)

**USA SIEMPRE** cuando:
- Tu aplicación necesita verificar permisos
- Estás en hot path (request/response)
- Necesitas performance
- Escribes tests

**NUNCA** para:
- Crear políticas
- Gestionar schemas
- Administrative tasks

### 2️⃣ CLI Tool (Control Plane)

**USA CUANDO:**
- Setup inicial
- CI/CD automation
- Bulk operations
- DevOps scripts
- Migration

**NO PARA:**
- Verificar permisos en runtime
- Hot path authorization

### 3️⃣ Web Console (Control Plane)

**USA CUANDO:**
- Exploración visual
- Debugging
- Colaboración
- Auditoría
- Training

**NO PARA:**
- Automated tasks
- Integration con código

### 4️⃣ Admin SDK (Control Plane, Opcional)

**USA SOLO SI:**
- Estás building tu propia admin UI
- Necesitas programmatic bulk operations
- Integración con otros admin tools

**Generalmente NO necesario** porque:
- CLI Tool + Web Console cubre 95% de casos
- Admin SDK añade complejidad
- Maintenance burden

---

## 🎯 Ejemplo Completo: Ciclo de Vida de una App

### Fase 1: Setup (Una vez)

```bash
# Usar CLI Tool
hodei init my-app
hodei schema apply --file=schema.json
hodei policies import --dir=policies/
hodei identity-source create keycloak --issuer=...
```

### Fase 2: Desarrollo

```typescript
// Usar SDK Cliente
const engine = new AVPAuthorizationEngine({
    policyStoreId: 'ps123',
    callType: 'isAuthorized'
});

// En middleware
app.use(async (req, res, next) => {
    const result = await engine.isAuthorized(buildRequest(req), []);
    if (result.type === 'deny') return res.status(403).send('Forbidden');
    next();
});
```

### Fase 3: Debug/Collaborate

```
[Web Console]
1. Developer: "Me da 403, ¿por qué?"
2. Abre console, ve logs en tiempo real
3. PM: "Ah, esa política es muy restrictiva"
4. Editan juntos la política visualmente
5. Testean inmediatamente
```

### Fase 4: Deploy (CI/CD)

```yaml
# CLI Tool en CI
- name: Deploy authorization
  run: |
    hodei schema apply --file=schema.json
    hodei policies import --dir=policies/
```

### Fase 5: Monitoreo

```
[Web Console - Auditoría]
1. Security team revisa logs mensual
2. "Estos cambios de políticas están bien"
3. Export para compliance report
```

---

## 🏁 Conclusión

**Separación clara = Mejor arquitectura**

- **SDK Cliente** → Verificar permisos (en código)
- **CLI Tool** → Automatizar gestión (en scripts/CI)
- **Web Console** → Exploración visual (en browser)
- **Admin SDK** → Solo si necesitas custom admin UI

**Cada herramienta hace UNA cosa bien.**

