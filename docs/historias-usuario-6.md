# Historias de usuario 6 - Frontend Verified Permissions con BFF  en Next.js

## 1. Contexto y arquitectura de comunicación

- Se adopta un BFF (Backend‑for‑Frontend) dentro de Next.js (Node.js runtime) que habla gRPC nativo (HTTP/2) con el servidor Verified Permissions usando `@grpc/grpc-js` + `@grpc/proto-loader`.
- El navegador consume **API Routes** de Next.js (HTTP/JSON). No hay mocks ni fallbacks. No se requiere habilitar gRPC‑web en el servidor.
- Compatibilidad SDK: se mantiene intacto el gRPC nativo del servidor.

Diagrama de alto nivel:

```  
Next.js (browser)  
  └─ API Routes (HTTP/JSON)       └─ Node gRPC client (@grpc/grpc-js)            └─ tonic gRPC services (HTTP/2) → repositorios y dominio```  
  
## 2. Pasos de setup (backend + frontend)  
  
### Backend (servidor Verified Permissions)  
1. Mantener gRPC nativo (tonic actual). No es necesario habilitar gRPC‑web.  
2. Ejecutar:  
   - `cargo build -p hodei-verified-permissions-server`  
   - `DATABASE_URL=sqlite:///app/data/hodei.db cargo run -p hodei-verified-permissions-server`  
  
### Frontend (Next.js con BFF)  
1. Dependencias:  
   - `npm i @grpc/grpc-js @grpc/proto-loader`  
2. Cliente gRPC Node (`src/lib/grpc/node-client.ts`):  
   - Cargar `proto/authorization.proto` y crear clientes `AuthorizationControl`/`AuthorizationData` contra `process.env.VERIFIED_PERMISSIONS_ADDR || 'localhost:50051'`.  
3. API Routes (App Router):  
   - `src/app/api/health/route.ts` (GET)  
   - `src/app/api/policy-stores/route.ts` (GET/POST)  
   - `src/app/api/policy-stores/[id]/route.ts` (GET/DELETE)  
4. Hooks UI: consumir `/api/...` en lugar del cliente gRPC en el navegador.  
5. Arranque:  
   - `rm -rf .next && npm run dev -- --webpack`  
6. Verificación rápida:  
   - Abrir `http://localhost:3000/dashboard` y `http://localhost:3000/policy-stores`  
   - DevTools Console: no debería haber errores CORS ni “Connection Error”.  
  
  
## 3. Resolución de errores comunes  
  
- "Connection Error" / timeouts desde la UI:  
  - Causa: servidor gRPC no disponible o `VERIFIED_PERMISSIONS_ADDR` incorrecto.  
  - Solución: confirmar server en `:50051`; exportar `VERIFIED_PERMISSIONS_ADDR=localhost:50051` y reiniciar Next.  
- Errores gRPC (status/códigos) en API Routes:  
  - Mapear a respuestas HTTP coherentes (400/404/500) y mostrar mensajes en UI.  
  - Añadir logs de servidor Next (try/catch) con información del método y timing.  
- Serialización/DTO:  
  - Asegurar que los objetos devueltos por gRPC se transforman a JSON esperado por la UI (nombres de campos, fechas ISO).  
- Hidratación React (atributos extra en `<body>`):  
  - Usar `suppressHydrationWarning` en `<html>` y `<body>`.  
  
## 4. Historias de usuario E2E (plan completo)  
  
- HU1: Conexión/Salud  
  - Criterios: indicador Connected/Error; health cada 30s; reintentos exponenciales; trazas `[health]`.  
  - E2E: abrir `/dashboard`, comprobar indicador, simular caída/recuperación del backend.  
- HU2: Policy Stores – Listado  
  - Criterios: carga inicial, vacíos manejados, error de red manejado.  
  - E2E: `/policy-stores` lista renderizada; validación de filas y timestamps.  
- HU3: Policy Stores – Creación  
  - Criterios: validación de formulario; mensaje éxito; aparece en la tabla; id persistente.  
  - E2E: crear con descripción; verificar en UI y vía respuesta gRPC.  
- HU4: Policy Stores – Eliminación  
  - Criterios: confirmación; eliminación idempotente; tabla actualizada.  
  - E2E: eliminar y verificar desaparición.  
- HU5: Schemas – Guardar/Obtener  
  - Criterios: validar JSON/schema; guardar; recuperar; diff visible.  
  - E2E: `putSchema` y `getSchema` para un store; comparar contenido.  
- HU6: Authorization Test Bench – isAuthorized  
  - Criterios: entrada principal/acción/recurso/contexto; salida con `decision` y `determiningPolicies`.  
  - E2E: caso ALLOW y caso DENY; mostrar errores si el contexto es inválido.  
- HU7: Navegación/Accesibilidad  
  - Criterios: sidebar navegable vía teclado; rutas activas; focus visible; sin errores de consola.  
  - E2E: recorrer secciones; asserts de a11y básicos (roles/títulos).  
- HU8: Estados de carga/errores  
  - Criterios: spinners, mensajes de error, reintentar; no bloquea UI.  
  - E2E: interceptar fallos temporales, validar retry/estado final.  
- HU9: Persistencia básica (smoke)  
  - Criterios: crear → recargar página → el recurso sigue existiendo.  
  - E2E: reload y verificación consistente.  
- HU10: Seguridad (dev)  
  - Criterios: CORS correcto; sin credenciales expuestas; sin errores mixtos HTTP/2/1.1.  
  - E2E: validación de preflight (OPTIONS) y cabeceras en llamadas reales.  
  
### 4.1 Cobertura de endpoints gRPC  
- AuthorizationControl:  
  - `ListPolicyStores`, `CreatePolicyStore`, `DeletePolicyStore`  
  - `PutSchema`, `GetSchema`  
  - `IsAuthorized`  
- (Opcional) AuthorizationData si se usa en UI actual.  
  
### 4.2 Datos y entorno de pruebas  
- Semilla mínima: ningún store inicial (UI vacía) ó 1 store de ejemplo.  
- Variables:  
  - `VERIFIED_PERMISSIONS_ADDR=localhost:50051`  
  - `DATABASE_URL=sqlite:///app/data/hodei.db`  
- Limpieza: al finalizar tests, eliminar stores creados.  
  
## 5. Guía E2E con Playwright  
  
### Instalación  
- En `web-nextjs`:  
  - `npm i -D @playwright/test`  
  - `npx playwright install --with-deps`  
  
### Configuración mínima `playwright.config.ts`  
- Base URL: `http://localhost:3000`  
- Retries: 1  
- Reporter: `html`  
  
### Tests iniciales  
- `tests/e2e/health.spec.ts`  
  - Abre `/dashboard`; espera indicador "Connected".  
- `tests/e2e/policy-stores.spec.ts`  
  - Abre `/policy-stores` y espera lista cargada.  
  - Crea un Policy Store y verifica aparición.  
  - Elimina y verifica desaparición.  
  
### Ejecución  
- Asegurar ambos servidores corriendo (50051 y 3000).  
- `npx playwright test` (o script `npm run e2e`).  
- Reporte: `playwright-report/index.html`.  
  
## 6. Consideraciones de producción  
- Variables seguras: configurar `VERIFIED_PERMISSIONS_ADDR` mediante secretos.  
- Observabilidad: mantener trazas en API Routes (latencia por método y errores gRPC).  
- Validación de payloads en API Routes para robustez.  
- Versionado de protobuf/generadores coherente con el servidor y el SDK (sin `codegenv2`).  
  
## 7. Estructura y estrategia de tests E2E  
  
### 7.1 Estructura de carpetas  
- `web-nextjs/tests/e2e/`  
  - `health.spec.ts`, `policy-stores.spec.ts`, `schemas.spec.ts`, `authorization.spec.ts`, `navigation.spec.ts`, `persistence.spec.ts`, `api-contract.spec.ts`  
- `web-nextjs/tests/fixtures/`  
  - `server.ts` (helpers para esperar servidor 50051/3000)  
  - `seed.ts` (semilla mínima si fuera necesaria)  
- `web-nextjs/playwright.config.ts`  
  - `baseURL=http://localhost:3000`, `retries=1`, `reporter=html`  
  
### 7.2 Fixtures y datos  
- Esperar disponibilidad de backend y frontend antes de tests.  
- Variables: `NEXT_PUBLIC_GRPC_BASE_URL=http://localhost:50051`, `DATABASE_URL=sqlite:///app/data/hodei.db`.  
- Limpieza al finalizar: eliminar stores creados durante los tests.  
  
### 7.3 Iteraciones de implementación  
- Iteración 1: config Playwright + `health.spec.ts` + `policy-stores.spec.ts` (listar/crear/eliminar).  
- Iteración 2: `schemas.spec.ts` + `authorization.spec.ts`.  
- Iteración 3: `navigation.spec.ts` + `persistence.spec.ts` + `api-contract.spec.ts` (valida respuestas de API Routes: status codes, estructura JSON).  
  
### 7.4 Métricas de calidad  
- 100% de los tests E2E pasan con servidores reales (sin mocks ni proxies).  
- Sin warnings de hidratación en navegación base.  
- Sin errores CORS/preflight.  
- Tiempo total de la suite: < 3 minutos local.  
  
## 8. Alternativa: BFF en Next.js (lado servidor Node.js)  
  
Si no se habilita gRPC‑web en el servidor y queremos mantener comunicación directa con gRPC nativo desde la aplicación web, se puede implementar un BFF (Backend‑for‑Frontend) dentro de Next.js usando Node.js runtime.  
  
### 8.1 Arquitectura  
- Navegador → Next.js API Routes (HTTP/JSON)  
- Next.js API Routes (Node) → Servidor Verified Permissions (gRPC nativo, HTTP/2) con `@grpc/grpc-js` + `@grpc/proto-loader`.  
- Sin mocks, sin gateways externos. Next actúa como capa de orquestación interna.  
  
### 8.2 Componentes  
- `src/lib/grpc/node-client.ts` (Node‑only):  
  - Carga `proto/authorization.proto` con `proto-loader`.  
  - Crea clientes `AuthorizationControl` y `AuthorizationData` en `VERIFIED_PERMISSIONS_ADDR` (por defecto `localhost:50051`).  
  - Expone helpers: `listPolicyStores`, `createPolicyStore`, `getPolicyStore`, `deletePolicyStore`, `putSchema`, `getSchema`, `isAuthorized`, `healthCheck`.  
- API Routes (App Router):  
  - `src/app/api/health/route.ts` (GET) → `node-client.healthCheck()`  
  - `src/app/api/policy-stores/route.ts` (GET/POST)  
  - `src/app/api/policy-stores/[id]/route.ts` (GET/DELETE)  
- Hooks UI: consumen `/api/...` en lugar de llamar desde el navegador a gRPC.  
  
### 8.3 Variables de entorno  
- `VERIFIED_PERMISSIONS_ADDR=localhost:50051`  
  
### 8.4 Pros/Contras  
- Pros: Compatible con gRPC nativo actual; elimina CORS; mantiene 100% servicios reales.  
- Contras: Next.js se convierte en BFF (proxy interno); mayor latencia en salto adicional; lógica de serialización en API Routes.  
  
### 8.5 E2E con BFF  
- Los tests Playwright no cambian (UI → `/api/...`).  
- Precondiciones: servidor gRPC activo; Next.js levantado (API Routes disponibles).  
  
---  
Actualizado: YYYY-MM-DD. Responsable: Equipo Hodei.