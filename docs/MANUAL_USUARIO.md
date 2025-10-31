# Manual de Usuario - Hodei Verified Permissions

## 📋 Índice
1. [Introducción](#introducción)
2. [Requisitos Previos](#requisitos-previos)
3. [Instalación y Configuración](#instalación-y-configuración)
4. [Navegación General](#navegación-general)
5. [Dashboard](#dashboard)
6. [Authorization Playground](#authorization-playground)
7. [Gestión de Policy Stores](#gestión-de-policy-stores)
8. [Gestión de Policies](#gestión-de-policies)
9. [Gestión de Schemas](#gestión-de-schemas)
10. [Gestión de Templates](#gestión-de-templates)
11. [Identity Sources](#identity-sources)
12. [Settings](#settings)
13. [Testing y Validación](#testing-y-validación)
14. [Solución de Problemas](#solución-de-problemas)

---

## Introducción

**Hodei Verified Permissions** es una aplicación web completa para gestionar y probar sistemas de autorización basados en policies. La aplicación permite:

- 📊 Visualizar métricas y estado del sistema en tiempo real
- 🧪 Probar autorizaciones individuales y en lote
- 🔍 Depurar el flujo de autorización paso a paso
- 💾 Guardar y reutilizar escenarios de prueba
- 📈 Monitorear la salud del sistema

---

## Requisitos Previos

### Software Necesario
- **Node.js** 18+ y npm
- **Rust** (para el servidor backend)
- **Git** para clonar el repositorio
- **Docker** (opcional, para testing E2E)

### Puertos Utilizados
- **3001**: Frontend Next.js (BFF - Backend for Frontend)
- **50051**: Servidor gRPC Rust (Backend)

---

## Instalación y Configuración

### 1. Clonar el Repositorio
```bash
git clone <repository-url>
cd hodei-verified-permissions
```

### 2. Instalar Dependencias
```bash
# Frontend
cd web-nextjs
npm install

# Backend
cd ../verified-permissions
cargo build --release
```

### 3. Iniciar la Aplicación

La aplicación utiliza una **arquitectura BFF (Backend for Frontend)**:

```
Browser (3001) → Next.js Frontend → API Routes → gRPC Client → Rust Server (50051)
```

#### Opción A: Script Automatizado
```bash
# Desde la raíz del proyecto
./scripts/dev-start.sh
```

#### Opción B: Manual
```bash
# Terminal 1: Backend Rust (puerto 50051)
cd verified-permissions
./target/release/hodei-verified-permissions

# Terminal 2: Frontend Next.js (puerto 3001)
cd web-nextjs
npm run dev
```

### 4. Verificar Funcionamiento
Abrir en el navegador: **http://localhost:3001**

---

## Navegación General

La aplicación utiliza un diseño de **sidebar** con las siguientes secciones:

```
┌─────────────────────────────────────────┐
│  ☰ Hodei Verified Permissions          │
├─────────────────────────────────────────┤
│  📊 Dashboard                          │
│  🧪 Authorization Playground           │
│  🏪 Policy Stores                      │
│  📜 Policies                           │
│  📋 Schemas                            │
│  📄 Templates                          │
│  🔑 Identity Sources                   │
│  ⚙️  Settings                          │
└─────────────────────────────────────────┘
```

---

## Dashboard

### Vista General
El Dashboard proporciona una **visión completa** del sistema con:

#### 1. Métricas del Sistema
- **Policy Stores**: Número total de stores configurados
- **Policies**: Cantidad de policies activas
- **Schemas**: Esquemas de autorización definidos
- **Templates**: Templates de policy disponibles

#### 2. System Health
- **gRPC Server**: Estado de conexión con el servidor backend
  - ✅ Connected: Verde
  - ❌ Disconnected: Rojo
- **Database**: Estado de la base de datos
  - ✅ Connected: Verde
  - ❌ Disconnected: Rojo

#### 3. Authorization Requests (Gráficos)
- **Authorization Requests**: Volumen de peticiones en el tiempo
- **Authorization Decisions**: Distribución ALLOW vs DENY

#### 4. Recent Activity
Feed en tiempo real con las últimas acciones:
- Creación/eliminación de policies
- Cambios en schemas
- Tests de autorización ejecutados

### Cómo Usar
1. **Refrescar Datos**: Click en el botón "Refresh" (esquina superior derecha)
2. **Auto-refresh**: Los datos se actualizan automáticamente cada 30 segundos
3. **Navegación**: Click en cualquier métrica para ir a la sección correspondiente

---

## Authorization Playground

El **Authorization Playground** es el corazón de la aplicación para testing de autorizaciones.

### Pestañas Disponibles

#### 1. Single Test (Prueba Individual)

##### Configuración del Escenario

**Campos Requeridos:**
- **Scenario Name**: Nombre descriptivo del escenario
- **Description**: Descripción detallada (opcional)
- **Policy Store ID**: ID del policy store a utilizar
- **Principal**: El usuario/entidad que solicita acceso
  - Entity Type (ej: "User")
  - Entity ID (ej: "alice")
- **Action**: La acción a realizar
  - Entity Type (ej: "Action")
  - Entity ID (ej: "viewDocument")
- **Resource**: El recurso al que se quiere acceder
  - Entity Type (ej: "Document")
  - Entity ID (ej: "doc123")
- **Context**: Contexto adicional en formato JSON
  ```json
  {
    "role": "admin",
    "department": "IT"
  }
  ```

##### Guardar Escenario
1. Completar todos los campos requeridos
2. Click en **"Save Scenario"**
3. El escenario aparecerá en la lista "Saved Scenarios"

##### Cargar Escenario
1. En la lista "Saved Scenarios", click en el ícono **📁** (Load)
2. Los campos se llenarán automáticamente con los datos del escenario

##### Eliminar Escenario
1. En la lista "Saved Scenarios", click en el ícono **🗑️** (Delete)
2. Confirmar la eliminación en el diálogo

##### Ejecutar Test
1. Configurar el escenario
2. (Opcional) Activar **Debug Mode**
3. Click en **"Run Test"**

##### Debug Mode
Activar con el botón **"Enable Debug"**:

**Pasos de Debug Mostrados:**
1. Parse authorization request
2. Load policy store configuration
3. Fetch relevant policies
4. Evaluate policies against entities
5. Determine final authorization decision

**Características del Debug:**
- ✅ **Expandible**: Click en cada paso para ver detalles
- 📊 **Métricas**: Duración de cada paso
- 🎯 **Status**: Pending, Running, Completed, Failed
- 📋 **Detalles**: Policies evaluadas, entidades, errores

##### Interpretar Resultados

**Decisión:**
- ✅ **ALLOW**: Autorización granted
- ❌ **DENY**: Autorización denied
- ⚠️ **UNSPECIFIED**: Sin decisión clara

**Determining Policies:**
Lista de policies que determinaron la decisión final.

**Errores:**
Si hay errores, se muestran con detalles para debugging.

#### 2. Batch Test (Pruebas en Lote)

Permite ejecutar **múltiples escenarios** simultáneamente.

##### Predefined Test Suites

**User Access Tests (3 escenarios)**
- Test 1: Usuario normal acceso a documento
- Test 2: Usuario sin permisos
- Test 3: Usuario con permisos especiales

**Role-Based Tests (3 escenarios)**
- Test 1: Admin con acceso completo
- Test 2: Editor con acceso limitado
- Test 3: Viewer solo lectura

##### Custom Scenarios
Configurar escenarios personalizados:
1. Hacer click en **"Add Custom Scenario"**
2. Completar los campos
3. Repetir para múltiples escenarios
4. Click en **"Run Batch Test"**

##### Resultados del Batch Test

**Estadísticas:**
- **Total**: Número total de tests
- **Allow**: Tests con decisión ALLOW
- **Deny**: Tests con decisión DENY
- **Success Rate**: Porcentaje de éxito

**Lista de Resultados:**
Tabla con:
- Scenario Name
- Principal
- Action
- Resource
- Decision
- Determining Policies
- Errors (si aplica)

##### Exportar Resultados
1. Ejecutar el batch test
2. Click en **"Export to CSV"**
3. Descargar archivo con timestamp

---

## Gestión de Policy Stores

### Lista de Policy Stores
Ver todos los policy stores configurados con:
- Nombre y descripción
- Estado (Active/Inactive)
- Número de policies asociadas
- Fecha de creación y última actualización

### Crear Policy Store
1. Click en **"Create Policy Store"**
2. Completar:
   - **Name**: Nombre único
   - **Description**: Descripción detallada
   - **Configuration**: Settings específicos
3. Click en **"Create"**

### Acciones Disponibles
- **👁️ View**: Ver detalles del policy store
- **✏️ Edit**: Modificar configuración
- **🗑️ Delete**: Eliminar (con confirmación)

---

## Gestión de Policies

### Lista de Policies
Visualizar policies por:
- Policy Store
- Nombre
- Estado
- Tipo de regla

### Crear Policy
1. Seleccionar **Policy Store**
2. Click en **"Create Policy"**
3. Definir:
   - **Policy Name**: Nombre único
   - **Effect**: Allow o Deny
   - **Principal**: Patrón de usuarios
   - **Action**: Acciones permitidas/restringidas
   - **Resource**: Recursos afectados
   - **Condition**: Condiciones adicionales (opcional)

### Ejemplo de Policy
```json
{
  "effect": "Allow",
  "principal": "User:alice",
  "action": "viewDocument",
  "resource": "Document:doc123",
  "condition": {
    "role": "admin"
  }
}
```

---

## Gestión de Schemas

### Definición de Esquemas
Los schemas definen la estructura de entidades y relationships:
- **Entity Types**: Tipos de entidades (User, Document, Action)
- **Relationships**: Relaciones entre entidades
- **Attributes**: Atributos de cada tipo

### Crear Schema
1. Click en **"Create Schema"**
2. Definir estructura JSON
3. Validar sintaxis
4. Guardar

### Ejemplo de Schema
```json
{
  "entityTypes": {
    "User": {
      "attributes": ["name", "email", "role"]
    },
    "Document": {
      "attributes": ["title", "owner", "classification"]
    }
  },
  "relationships": {
    "User": ["owns", "can_read", "can_write"],
    "Document": ["owned_by", "classified_as"]
  }
}
```

---

## Gestión de Templates

### Templates Predefinidos
Biblioteca de templates comunes:
- **Role-Based Access Control (RBAC)**
- **Attribute-Based Access Control (ABAC)**
- **Owner-Based Permissions**
- **Time-Based Access**

### Crear Template Personalizado
1. Click en **"Create Template"**
2. Seleccionar template base
3. Personalizar parameters
4. Guardar como nuevo template

### Usar Template
1. Crear nueva policy
2. Seleccionar **"Use Template"**
3. Elegir template de la lista
4. Personalizar valores
5. Crear policy

---

## Identity Sources

### Configuración de Fuentes de Identidad
Integración con:
- **LDAP/Active Directory**
- **OAuth Providers** (Google, GitHub, etc.)
- **SAML**
- **Custom Identity Providers**

### Configurar Identity Source
1. Click en **"Add Identity Source"**
2. Seleccionar tipo
3. Completar configuración:
   - **Name**: Nombre descriptivo
   - **Type**: Tipo de proveedor
   - **Connection Settings**: URL, credenciales
   - **Mapping Rules**: Cómo mapear atributos

---

## Settings

### Configuración General
- **Server Settings**: URLs y timeouts
- **gRPC Configuration**: Endpoints del backend
- **Logging Level**: Debug, Info, Warning, Error
- **Performance**: Cache settings, timeouts

### User Preferences
- **Theme**: Light/Dark mode
- **Language**: Español/Inglés
- **Dashboard Refresh Rate**: Frecuencia de actualización
- **Default Policy Store**: Store por defecto

---

## Testing y Validación

### Tests E2E Disponibles

La aplicación incluye tests automatizados con Playwright:

```bash
# Ejecutar todos los tests E2E
npm run test:e2e

# Tests específicos
npm run test:e2e:dashboard
npm run test:e2e:playground
npm run test:e2e:scenarios
```

#### Cobertura de Tests
- ✅ **Dashboard**: Métricas, health checks, refresh
- ✅ **Playground**: Single test, debug mode, batch test
- ✅ **Scenarios**: Save, load, list, delete scenarios
- ✅ **Policies**: CRUD operations
- ✅ **Schemas**: Create, validate schemas

### Validación Manual

#### Checklist de Funcionalidades

**Dashboard:**
- [ ] Métricas se muestran correctamente
- [ ] System Health muestra conexiones
- [ ] Botón Refresh funciona
- [ ] Auto-refresh cada 30s
- [ ] Activity feed actualizado

**Playground - Single Test:**
- [ ] Formulario se completa correctamente
- [ ] Save Scenario guarda y aparece en lista
- [ ] Load Scenario carga datos
- [ ] Run Test ejecuta y muestra resultado
- [ ] Debug Mode muestra steps detallados

**Playground - Batch Test:**
- [ ] Predefined tests ejecutan correctamente
- [ ] Custom scenarios se añaden
- [ ] Resultados muestran estadísticas
- [ ] Export to CSV funciona

**Policy Stores:**
- [ ] Lista muestra stores existentes
- [ ] Create Policy Store funciona
- [ ] Edit/Update store funciona
- [ ] Delete store funciona

**Policies:**
- [ ] Lista filtrada por policy store
- [ ] Create Policy con template
- [ ] Edit Policy existente
- [ ] Delete Policy

**Schemas:**
- [ ] Crear schema con JSON válido
- [ ] Validar schema
- [ ] Editar schema

---

## Solución de Problemas

### Problemas Comunes

#### 1. Dashboard no carga métricas
**Síntoma**: Dashboard muestra "Error Loading Dashboard"

**Solución**:
```bash
# Verificar que el backend Rust esté corriendo en puerto 50051
curl http://localhost:50051/health

# Verificar logs del backend
tail -f /tmp/rust-server.log

# Reiniciar backend
killall hodei-verified-permissions
./target/release/hodei-verified-permissions

# Verificar que el frontend esté corriendo en puerto 3001
curl http://localhost:3001/api/health
```

#### 2. Playground muestra errores de conexión
**Síntoma**: "Failed to run authorization test"

**Solución**:
1. Verificar que el backend Rust esté corriendo en puerto 50051
2. Verificar que el frontend esté corriendo en puerto 3001
3. Verificar que la comunicación gRPC funciona:
   ```bash
   curl http://localhost:50051/health
   ```
4. Check logs: `tail -f /tmp/rust-server.log`
5. Verificar API frontend:
   ```bash
   curl http://localhost:3001/api/authorize
   ```

#### 3. Escenarios no se guardan
**Síntoma**: Scenario save falla

**Solución**:
```bash
# Verificar API de escenarios en el frontend
curl -X POST http://localhost:3001/api/scenarios \
  -H "Content-Type: application/json" \
  -d '{"name":"Test","policy_store_id":"ps_123"}'

# Verificar localStorage en navegador (F12 DevTools → Application → Local Storage)

# Verificar que el React Query cache esté funcionando
# (Revisar DevTools Console para errores)
```

#### 4. Tests E2E fallan
**Síntoma**: Playwright tests timeout

**Solución**:
```bash
# Instalar navegadores
npx playwright install

# Verificar que servidor esté en puerto 3001
lsof -i :3001

# Ejecutar tests en modo headed (ver navegador)
npx playwright test --headed
```

#### 5. Errores de compilación
**Síntoma**: Build errors en frontend/backend

**Solución**:
```bash
# Frontend
cd web-nextjs
rm -rf node_modules package-lock.json
npm install

# Backend
cd verified-permissions
cargo clean
cargo build --release
```

### Logs y Debugging

#### Frontend Logs
Abrir **DevTools** (F12) → **Console**

#### Backend Logs
```bash
# Logs en tiempo real
tail -f /tmp/rust-server.log

# Ver últimos 100 líneas
tail -100 /tmp/rust-server.log

# Buscar errores
grep -i error /tmp/rust-server.log
```

#### API Testing
```bash
# Health check
curl http://localhost:3001/api/health

# Test authorization
curl -X POST http://localhost:3001/api/authorize \
  -H "Content-Type: application/json" \
  -d '{
    "policy_store_id":"ps_123",
    "principal":{"entity_type":"User","entity_id":"alice"},
    "action":{"entity_type":"Action","entity_id":"viewDocument"},
    "resource":{"entity_type":"Document","entity_id":"doc123"},
    "context":{}
  }'
```

### Performance

#### Monitoreo de Performance
- **Dashboard** → Métricas de tiempo de respuesta
- **Network tab** (F12) → Request duration
- **Playwright traces** → Para debugging de tests

#### Optimización
- **Cache**: React Query caching habilitado
- **Refresh Rate**: Ajustar en Settings (default: 30s)
- **Batch Size**: Limitar número de escenarios en batch test

---

## FAQ

### ¿Cómo cambiar el puerto del frontend?
Editar `web-nextjs/package.json`:
```json
{
  "scripts": {
    "dev": "next dev -p 3002"
  }
}
```

### ¿Cómo configurar múltiples policy stores?
1. Ir a **Policy Stores**
2. Crear múltiples stores
3. En **Playground**, seleccionar el store deseado

### ¿Puedo importar/exportar policies?
**Import**: Desde Policies → Import (JSON/YAML)
**Export**: Desde Policies → Export Selected

### ¿Cómo resetear la aplicación?
```bash
# Limpiar localStorage del navegador
localStorage.clear()

# Reiniciar backend
killall hodei-verified-permissions

# Limpiar caches
cd web-nextjs
rm -rf .next node_modules/.cache
npm run dev
```

### ¿El debug mode afecta performance?
No, debug mode solo agrega logging visual. No impacta la performance real de authorization.

---

## Arquitectura de la Aplicación (BFF Pattern)

### Overview

Hodei Verified Permissions utiliza una **arquitectura BFF (Backend for Frontend)**, que es un patrón de diseño donde el **frontend Next.js actúa como Backend para el Frontend**, proporcionando una API especializada para la interfaz de usuario.

### Diagrama de Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│                        BROWSER (Usuario)                        │
│                         localhost:3001                          │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  📱 Frontend Next.js                                       │ │
│  │  ├─ React + TypeScript                                     │ │
│  │  ├─ React Query (Estado y Caching)                         │ │
│  │  ├─ Tailwind CSS (Styling)                                 │ │
│  │  └─ Components (Dashboard, Playground, etc.)               │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────┬───────────────────────────────────────┘
                          │ HTTP (API Routes)
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      API LAYER (BFF)                            │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  🔌 Next.js API Routes (/api/*)                           │ │
│  │  ├─ /api/authorize - Authorization testing                │ │
│  │  ├─ /api/scenarios - Scenario management                  │ │
│  │  ├─ /api/metrics - Dashboard metrics                      │ │
│  │  ├─ /api/activity - Activity feed                         │ │
│  │  └─ /api/policy-stores/* - Policy store operations       │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────┬───────────────────────────────────────┘
                          │ gRPC Client (Node.js)
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     BACKEND RUST                                │
│                         localhost:50051                         │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  🔧 gRPC Server                                           │ │
│  │  ├─ Authorization Logic                                   │ │
│  │  ├─ Policy Evaluation Engine                              │ │
│  │  ├─ JWT Token Validation                                  │ │
│  │  └─ Database Integration                                  │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Flujo de Datos (Ejemplo: Authorization Test)

1. **Usuario** hace click en "Run Test" en el Playground
   ```
   Browser (React) → useMutation → /api/authorize
   ```

2. **API Route** (`/api/authorize`) recibe la request
   ```typescript
   // /api/authorize.ts
   export default async function handler(req, res) {
     // 1. Parse request body
     const scenario = req.body;

     // 2. Call gRPC server via client
     const result = await grpcClient.authorize(scenario);

     // 3. Transform response
     res.status(200).json(result);
   }
   ```

3. **gRPC Client** (Node.js) envía request al Rust server
   ```typescript
   // /lib/grpc/node-client.ts
   const response = await client.isAuthorized({
     policyStoreId: scenario.policy_store_id,
     principal: scenario.principal,
     action: scenario.action,
     resource: scenario.resource,
     context: scenario.context,
   });
   ```

4. **Rust Server** procesa la autorización
   ```rust
   // verified-permissions/src/server.rs
   async fn is_authorized(
       &self,
       request: Request<IsAuthorizedRequest>,
   ) -> Result<Response<IsAuthorizedResponse>, Status> {
       // 1. Load policies from database
       // 2. Evaluate against entities
       // 3. Return ALLOW/DENY decision
   }
   ```

5. **Respuesta** fluye de vuelta:
   ```
   Rust Server → gRPC → API Route → React Query → UI
   ```

### Componentes del Sistema

#### 1. **Frontend Next.js (Puerto 3001)**

**Responsabilidades:**
- Renderizado de la UI (React)
- Gestión de estado (React Query)
- Routing (Next.js Pages)
- API Routes (Backend logic)

**Tecnologías:**
- React 18
- TypeScript
- Tailwind CSS
- React Query (@tanstack/react-query)
- Lucide Icons (UI icons)

**Ejemplo de estructura:**
```
web-nextjs/src/
├── components/          # React components
│   ├── Dashboard.tsx
│   ├── Playground.tsx
│   ├── DebugPanel.tsx
│   └── ui/              # Reusable UI components
├── pages/              # Next.js pages
│   ├── dashboard.tsx
│   ├── playground.tsx
│   └── api/            # API Routes
│       ├── authorize/
│       ├── scenarios/
│       └── metrics/
├── hooks/              # Custom React hooks
│   ├── useDashboardMetrics.ts
│   └── useSavedScenarios.ts
└── lib/                # Utilities
    └── grpc/           # gRPC client
```

#### 2. **Backend Rust (Puerto 50051)**

**Responsabilidades:**
- Lógica de autorización (ABAC/RBAC)
- Evaluación de policies
- Gestión de schemas
- Validación de tokens JWT
- Comunicación con base de datos

**Tecnologías:**
- Rust (language)
- Tokio (async runtime)
- gRPC (communication)
- Prost (Protocol Buffers)
- JWT validation

**Ejemplo de estructura:**
```
verified-permissions/
├── src/
│   ├── server.rs        # gRPC server implementation
│   ├── auth/            # Authorization logic
│   ├── policies/        # Policy evaluation
│   ├── jwt/            # JWT token handling
│   └── db/             # Database integration
├── proto/              # Protocol Buffer definitions
└── Cargo.toml          # Dependencies
```

### API Routes (BFF Layer)

Cada API Route en el frontend actúa como un **BFF endpoint**:

| Endpoint | Propósito | Frontend→Backend |
|----------|-----------|------------------|
| `GET /api/health` | Health check | Frontend checks backend status |
| `POST /api/authorize` | Run authorization test | Frontend → gRPC: IsAuthorized |
| `GET /api/scenarios` | List saved scenarios | Frontend → gRPC: ListScenarios |
| `POST /api/scenarios` | Create scenario | Frontend → gRPC: CreateScenario |
| `GET /api/metrics` | Get dashboard metrics | Frontend → gRPC: GetMetrics |

### Ventajas del Patrón BFF

#### 1. **Separación de Responsabilidades**
- **Frontend**: UI/UX y state management
- **BFF**: API especializada para el frontend
- **Backend**: Lógica de negocio y autorización

#### 2. **API Personalizada**
```typescript
// En lugar de exponer todas las operaciones gRPC,
// creamos endpoints especializados para la UI

// Esto es lo que ve el frontend:
POST /api/authorize
{
  "scenario": { ... }
}

// En lugar de:
POST /hodei.v1.AuthorizationService/IsAuthorized
```

#### 3. **Transformación de Datos**
```typescript
// El BFF puede transformar datos del backend para el frontend
export default async function handler(req, res) {
  const grpcResult = await client.getMetrics();

  // Transform gRPC response to frontend format
  const frontendData = {
    policyStores: grpcResult.policy_store_count,
    policies: grpcResult.total_policies,
    schemas: grpcResult.schema_count,
    lastUpdated: new Date().toISOString()
  };

  res.json(frontendData);
}
```

#### 4. **Caching y Optimización**
```typescript
// El BFF puede implementar caching específico para la UI
export default async function handler(req, res) {
  const cached = await cache.get('dashboard_metrics');

  if (cached) {
    return res.json(cached); // Return cached data
  }

  const data = await fetchFromBackend();
  await cache.set('dashboard_metrics', data, 300); // Cache 5 min

  res.json(data);
}
```

#### 5. **Error Handling**
```typescript
// El BFF puede manejar errores gRPC y convertirlos a HTTP errors
export default async function handler(req, res) {
  try {
    const result = await client.authorize(request);
    res.json(result);
  } catch (error) {
    if (error.code === gRPCStatus.PERMISSION_DENIED) {
      res.status(403).json({ error: 'Permission denied' });
    } else if (error.code === gRPCStatus.UNAVAILABLE) {
      res.status(503).json({ error: 'Service unavailable' });
    } else {
      res.status(500).json({ error: 'Internal error' });
    }
  }
}
```

### Comunicación Entre Capas

#### Frontend → BFF (HTTP)
```typescript
// React Query hook
const { data } = useQuery({
  queryKey: ['metrics'],
  queryFn: () => fetch('/api/metrics').then(res => res.json())
});
```

#### BFF → Backend (gRPC)
```typescript
// gRPC client
import { createClient } from './grpc-client';

const client = createClient('http://localhost:50051');

const response = await client.isAuthorized({
  policyStoreId: 'ps_123',
  principal: { entityType: 'User', entityId: 'alice' },
  action: { entityType: 'Action', entityId: 'viewDocument' },
  resource: { entityType: 'Document', entityId: 'doc123' },
  context: {}
});
```

### Testing con Arquitectura BFF

#### 1. **Mocking Backend**
```typescript
// Jest test for API Route
jest.mock('@/lib/grpc/node-client');

test('should return authorization result', async () => {
  const mockClient = {
    isAuthorized: jest.fn().mockResolvedValue({
      decision: 'ALLOW',
      determiningPolicies: ['policy_1']
    })
  };

  const response = await handler(
    { body: testScenario },
    { status: jest.fn().mockReturnThis(), json: jest.fn() }
  );

  expect(response.decision).toBe('ALLOW');
});
```

#### 2. **Integration Tests**
```typescript
// Playwright E2E test
test('authorization test flow', async ({ page }) => {
  await page.goto('/playground');
  await page.fill('#scenario-name', 'Test Scenario');
  await page.click('button:has-text("Run Test")');

  // Verificar que la UI muestra el resultado
  await expect(page.locator('text=ALLOW')).toBeVisible();
});
```

### Comparación: Con BFF vs Sin BFF

| Aspecto | Con BFF (Actual) | Sin BFF (Directo) |
|---------|------------------|-------------------|
| **Arquitectura** | Frontend → API → gRPC → Backend | Frontend → gRPC → Backend |
| **Complejidad** | ✅ 3 capas bien definidas | ⚠️ Frontend habla directo con backend |
| **Flexibilidad** | ✅ BFF puede transformar datos | ❌ Datos acoplados al frontend |
| **Caching** | ✅ BFF puede cachear respuestas | ❌ Sin control de cache |
| **Error Handling** | ✅ BFF maneja errores HTTP/gRPC | ❌ Errores gRPC directo al frontend |
| **Testing** | ✅ Mock BFF endpoints | ❌ Mock gRPC en frontend |
| **Evolución** | ✅ Frontend/Backend independientes | ⚠️ Cambios requieren coordinación |

### Puerto y Configuración

| Componente | Puerto | Propósito |
|------------|--------|-----------|
| **Frontend Next.js** | 3001 | UI, API Routes, BFF |
| **Backend Rust** | 50051 | gRPC server, autorización |
| **Base de Datos** | Configurable | Persistencia (no mostrada en diagrama) |

### Despliegue en Producción

En un entorno de producción real, la arquitectura se desplegaría así:

```
┌─────────────────────────────────────────┐
│           Load Balancer                 │
└──────────────┬──────────────────────────┘
               │
        ┌──────┴──────┐
        ▼             ▼
┌──────────┐   ┌──────────┐
│Frontend 1│   │Frontend 2│  ← Next.js (Multiple instances)
└────┬─────┘   └────┬─────┘
     │              │
     └──────┬───────┘
            │
            ▼
┌──────────────────────────┐
│   Backend Services       │  ← Rust gRPC Servers
│  (Multiple instances)    │
└──────────────────────────┘
```

- **Frontend**: Escalable horizontalmente ( múltiples instancias de Next.js)
- **Backend**: Rust gRPC servers con load balancing
- **BFF**: Integrado en cada instancia de Next.js

---

## Soporte y Recursos

### Documentación Adicional
- `docs/PLAN_IMPLEMENTACION_DETALLADO.md` - Plan de implementación
- `docs/analisis-frontend.md` - Análisis del frontend
- `docs/analisis.backend.md` - Análisis del backend

### Historias de Usuario
- `docs/historias-usuario.md` - Historia general
- `docs/historias-usuario-*..md` - Historias específicas

### Repositorio
- **Frontend**: `/web-nextjs/`
- **Backend**: `/verified-permissions/`
- **Tests E2E**: `/web-nextjs/tests/e2e/`

---

## Changelog

### v1.0.0 (Fase 3)
- ✅ Dashboard con métricas en tiempo real
- ✅ Authorization Playground completo
- ✅ Sistema de escenarios guardados
- ✅ Debug Mode con step-by-step
- ✅ Batch Authorization Testing
- ✅ Gestión completa de Policies/Schemas/Templates
- ✅ Identity Sources integration
- ✅ Settings y configuración

### Próximas Versiones
- 🔄 Real-time collaboration
- 📱 Mobile responsive improvements
- 🔐 Advanced security features
- 📊 Advanced analytics dashboard

---

**¡Gracias por usar Hodei Verified Permissions!** 🎉

Para más información, consultar la documentación técnica en el repositorio.
