# Historias de Usuario - Aplicación de Ejemplo con SDK y Docker

## Visión

Construir una aplicación de ejemplo dentro del directorio `examples/` que demuestre el uso completo de `hodei_permissions_sdk` ejecutándose sobre un entorno Docker reproducible. La aplicación debe cubrir la configuración de servicios externos, la inicialización de políticas/usuarios y el consumo del middleware Axum/Tower para validar autorizaciones con tokens reales.

---

## HU 25.1 - Infraestructura Docker reproducible

- **Como** desarrollador del SDK
- **Quiero** levantar con un solo comando los servicios externos necesarios (Keycloak, PostgreSQL, servidor de AVP, migraciones iniciales)
- **Para** disponer de un entorno realista donde probar la aplicación de ejemplo

### Criterios de aceptación
- [ ] Archivo `examples/docker-compose.yml` con servicios Keycloak + PostgreSQL + servidor AVP
- [ ] Scripts de inicialización (`examples/scripts/`) que creen realm, clientes, usuarios y roles en Keycloak
- [ ] Volúmenes/localstack que permitan reinicios limpios
- [ ] Variables `.env` documentadas y cargadas en `docker-compose.yml`

### Tareas
1. Definir `docker-compose.yml` con redes y dependencias
2. Crear script `bootstrap_keycloak.sh`
3. Añadir script `seed_policies.sh` para AVP (usa CLI o SDK)
4. Documentar comandos `make up`, `make down`, `make logs`

---

## HU 25.2 - Backend de ejemplo con Axum

- **Como** usuario del SDK
- **Quiero** un servicio en `examples/app/` con endpoints REST protegidos por el middleware
- **Para** aprender cómo integrar el SDK en mi propia aplicación

### Criterios de aceptación
- [ ] Proyecto Rust dentro de `examples/app/` con `Cargo.toml` propio
- [ ] Configuración para conectarse al servidor AVP (URL, policy store, identity source)
- [ ] Endpoints `/documents` (GET), `/documents/:id` (GET), `/documents/:id/share` (POST)
- [ ] Uso del middleware `VerifiedPermissionsLayer` + extractor personalizado opcional
- [ ] Handlers registrados y respuesta JSON consistente

### Tareas
1. Crear `examples/app/Cargo.toml` con dependencia al SDK local
2. Implementar `main.rs` con Axum + tower + tracing
3. Configurar carga de `.env` (dotenvy) para IDs y credenciales
4. Añadir módulo `client.rs` para reutilizar `AuthorizationClient`
5. Preparar datos simulados (documentos en memoria)

---

## HU 25.3 - Automatización de políticas y datos

- **Como** QA/DevOps
- **Quiero** inicializar el entorno (policy store, identity source, políticas) sin pasos manuales
- **Para** ejecutar la demo en CI/CD o en talleres

### Criterios de aceptación
- [ ] Script `examples/scripts/setup_avp.rs` (bin Rust o script shell que use SDK) para crear policy store
- [ ] Creación automática de Identity Source apuntando al Keycloak levantado en Docker
- [ ] Políticas Cedar cargadas desde `examples/policies/*.cedar`
- [ ] Usuarios y roles sincronizados entre Keycloak y AVP

### Tareas
1. Definir plantillas Cedar (`policies/allow_admin.cedar`, etc.)
2. Implementar script que lea configuraciones desde `.env`
3. Ejecutar script como parte de `docker-compose up` (usando `depends_on` + `command`)
4. Validar idempotencia (si se ejecuta dos veces no falla)

---

## HU 25.4 - Guía de ejecución y pruebas end-to-end

- **Como** persona evaluando el SDK
- **Quiero** una guía paso a paso para levantar, consumir y verificar la demo
- **Para** constatar todas las capacidades del sistema

### Criterios de aceptación
- [ ] Documento `examples/README.md` con instrucciones en inglés y enlace a traducción en español
- [ ] Pasos: `docker compose up`, ejecutar scripts, lanzar backend Axum y curl/postman examples
- [ ] Sección de “Troubleshooting” con errores comunes (esquema Keycloak, puertos ocupados, jwks cache)
- [ ] Comandos de stop/reset (`docker compose down -v`)

### Tareas
1. Elaborar README con secciones: overview, prerequisites, quick start, endpoints disponibles
2. Añadir ejemplos de curl para tokens válidos/invalidos, diferentes roles
3. Documentar cómo inspeccionar logs (`docker compose logs`) y cómo regenerar tokens
4. Crear versión traducida `examples/README.es.md`

---

## Seguimiento y métricas

- Kanban en `docs/PLAN_AVP_CLONE.md` sección Sprint 4.1 (Demo SDK)
- Definición de terminado (**DoD**): Demo ejecutándose en 15 minutos, todos los scripts idempotentes, documentación actualizada
- Métrica de verificación: script `end_to_end_check.sh` devuelve exit code 0 tras curl con token de admin y viewer
