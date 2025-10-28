# Historias de usuario 6 - Frontend Verified Permissions con BFF  en Next.js

## 1. Contexto y arquitectura de comunicación

- Se adopta un BFF (Backend‑for‑Frontend) dentro de Next.js (Node.js runtime) que habla gRPC nativo (HTTP/2) con el servidor Verified Permissions usando `@grpc/grpc-js` + `@grpc/proto-loader`.
- El navegador consume **API Routes** de Next.js (HTTP/JSON). No hay mocks ni fallbacks. No se requiere habilitar gRPC‑web en el servidor.
- Compatibilidad SDK: se mantiene intacto el gRPC nativo del servidor.

## 2. Ingeniería Inversa de AWS Verified Permissions (AVP)

El objetivo es recrear la experiencia de usuario y las funcionalidades clave de la consola de AWS Verified Permissions (AVP) en nuestra implementación de Hodei. Esto implica un análisis de las características de AVP y su mapeo a nuestros servicios gRPC.

### 2.1. Funcionalidades Clave de AVP y Mapeo a Hodei

| Funcionalidad AVP | Descripción | Endpoints gRPC de Hodei | API Routes (Next.js) |
|---|---|---|---|
| **Gestión de Policy Stores** | Crear, listar y eliminar contenedores de políticas. | `ListPolicyStores`, `CreatePolicyStore`, `DeletePolicyStore` | `/api/policy-stores`, `/api/policy-stores/[id]` |
| **Gestión de Esquemas** | Definir y actualizar el esquema de entidades y acciones. | `PutSchema`, `GetSchema` | `/api/policy-stores/[id]/schema` |
| **Gestión de Políticas** | Crear, ver, actualizar y eliminar políticas estáticas. | `CreatePolicy`, `GetPolicy`, `UpdatePolicy`, `DeletePolicy`, `ListPolicies` | `/api/policy-stores/[id]/policies`, `/api/policy-stores/[id]/policies/[policyId]` |
| **Gestión de Plantillas de Políticas** | Crear y gestionar plantillas para políticas parametrizadas. | `CreatePolicyTemplate`, `GetPolicyTemplate`, `ListPolicyTemplates` | `/api/policy-stores/[id]/templates`, `/api/policy-stores/[id]/templates/[templateId]` |
| **"Test Bench" de Autorización** | Evaluar solicitudes de autorización en un entorno de prueba. | `IsAuthorized` | `/api/policy-stores/[id]/is_authorized` |

### 2.2. Estructura de la Interfaz de Usuario (UI)

La interfaz de usuario se organizará de la siguiente manera para emular la navegación de AVP:

- `/dashboard`: Página principal con estado de conexión y métricas.
- `/policy-stores`: Listado de todos los policy stores.
- `/policy-stores/[id]/`: Dashboard de un policy store específico, con acceso a:
  - `/policy-stores/[id]/schema`: Editor de esquemas.
  - `/policy-stores/[id]/policies`: Listado y gestión de políticas.
  - `/policy-stores/[id]/policies/new`: Creación de una nueva política.
  - `/policy-stores/[id]/policies/[policyId]`: Edición de una política existente.
  - `/policy-stores/[id]/templates`: Gestión de plantillas de políticas.
  - `/policy-stores/[id]/test-bench`: Banco de pruebas de autorización.

## 3. Historias de Usuario Detalladas (Plan Completo)

### HU1: Conexión y Salud del Sistema
- **Como:** Desarrollador
- **Quiero:** Un indicador visual en el dashboard que muestre el estado de la conexión con el servidor gRPC.
- **Para:** Saber si el sistema está operativo de un vistazo.
- **Criterios de Aceptación:**
  - El indicador muestra "Connected" si la API `/api/health` responde correctamente.
  - El indicador muestra "Error" si la API falla.
  - Se realiza una comprobación de salud cada 30 segundos.

### HU2: Gestión de Policy Stores
- **Como:** Administrador
- **Quiero:** Crear, listar y eliminar Policy Stores.
- **Para:** Organizar mis políticas en contenedores aislados.
- **Criterios de Aceptación:**
  - Puedo ver una lista de todos los policy stores en `/policy-stores`.
  - Puedo crear un nuevo policy store con una descripción.
  - Puedo eliminar un policy store existente (con confirmación).

### HU3: Gestión de Esquemas
- **Como:** Administrador
- **Quiero:** Definir y actualizar el esquema de mi policy store usando un editor JSON.
- **Para:** Establecer la estructura de mis entidades y acciones.
- **Criterios de Aceptación:**
  - En `/policy-stores/[id]/schema`, puedo ver el esquema actual.
  - Puedo editar y guardar el esquema. El editor valida que el JSON sea correcto.

### HU4: Gestión de Políticas Estáticas
- **Como:** Administrador
- **Quiero:** Crear, listar, ver y eliminar políticas estáticas en un policy store.
- **Para:** Definir las reglas de autorización de mi aplicación.
- **Criterios de Aceptación:**
  - En `/policy-stores/[id]/policies`, veo una lista de todas las políticas.
  - Puedo crear una nueva política proporcionando un ID y el código Cedar.
  - Puedo ver el contenido de una política existente.
  - Puedo eliminar una política.

### HU5: Gestión de Plantillas de Políticas (NUEVO)
- **Como:** Administrador
- **Quiero:** Crear y gestionar plantillas de políticas para reutilizar la lógica de autorización.
- **Para:** Simplificar la creación de políticas similares.
- **Criterios de Aceptación:**
  - En `/policy-stores/[id]/templates`, puedo ver una lista de plantillas.
  - Puedo crear una nueva plantilla con un ID y el código Cedar parametrizado.

### HU6: Banco de Pruebas de Autorización (Test Bench)
- **Como:** Desarrollador
- **Quiero:** Un "Test Bench" para simular solicitudes de autorización y ver el resultado.
- **Para:** Depurar y verificar mis políticas.
- **Criterios de Aceptación:**
  - En `/policy-stores/[id]/test-bench`, tengo campos para `principal`, `action`, `resource` y un editor para el `context` JSON.
  - Al enviar la solicitud, la API `/api/policy-stores/[id]/is_authorized` se invoca.
  - El resultado (`ALLOW`/`DENY`) y las políticas determinantes se muestran claramente.

## 4. Plan de Implementación por Fases

1.  **Fase 1: Base y Gestión de Policy Stores (Ya implementado)**
    -   Cliente gRPC, API Routes para `health` y `policy-stores`.
2.  **Fase 2: Gestión de Esquemas y Políticas**
    -   Crear las páginas y API Routes para `schema` y `policies`.
    -   Integrar un editor de código (ej. Monaco) para Cedar y JSON.
3.  **Fase 3: Banco de Pruebas (Test Bench)**
    -   Implementar la UI del Test Bench y la API Route `is_authorized`.
4.  **Fase 4: Gestión de Plantillas de Políticas**
    -   Implementar la UI y las API Routes para la gestión de plantillas.

## 5. Plan de Pruebas E2E Extendido (Playwright)

### 5.1. Estructura de Archivos de Prueba
- `tests/e2e/health.spec.ts`
- `tests/e2e/policy-stores.spec.ts`
- `tests/e2e/schemas.spec.ts` **(NUEVO)**
- `tests/e2e/policies.spec.ts` **(NUEVO)**
- `tests/e2e/templates.spec.ts` **(NUEVO)**
- `tests/e2e/authorization.spec.ts` (Test Bench)

### 5.2. Criterios de Prueba por Funcionalidad
- **schemas.spec.ts:**
  - Navega a la página de un policy store y accede a la pestaña de esquema.
  - Verifica que se puede cargar y mostrar un esquema existente.
  - Modifica el esquema, lo guarda y recarga la página para verificar la persistencia.
- **policies.spec.ts:**
  - Navega a la sección de políticas.
  - Crea una nueva política estática y verifica que aparece en la lista.
  - Hace clic en la política para ver su contenido.
  - Elimina la política y verifica que desaparece de la lista.
- **templates.spec.ts:**
  - Navega a la sección de plantillas.
  - Crea una nueva plantilla de política y verifica que aparece en la lista.
- **authorization.spec.ts:**
  - Navega al Test Bench.
  - Rellena los campos para una solicitud que debería ser `ALLOW` y verifica el resultado.
  - Rellena los campos para una solicitud que debería ser `DENY` y verifica el resultado.

---
Actualizado: 2025-10-28. Responsable: Equipo Hodei.
