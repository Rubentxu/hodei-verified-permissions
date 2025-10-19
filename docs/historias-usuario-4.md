Inspirarse en los SDK de middleware de Express.js es la estrategia perfecta. La filosofía de Express se centra en la **simplicidad, la configuración declarativa y la componibilidad**, principios que podemos traducir a un middleware de Rust potente y ergonómico.

El middleware de Express para AVP típicamente se ve así:

```javascript
// En Express.js (el objetivo a emular)
const avpMiddleware = require('aws-verified-permissions-middleware');

app.use(avpMiddleware({
  policyStoreId: '...',
  // La lógica específica de la app se inyecta aquí
  requestMapper: (req) => ({
    action: { type: 'Action', id: mapMethodToAction(req.method) },
    resource: { type: 'Document', id: req.params.docId }
  })
}));
```

Nuestro objetivo es crear una experiencia similarmente fluida en Rust, pero aprovechando la seguridad de tipos y los patrones del lenguaje.

---

### **1. El Diseño: Un Middleware Configurable con el Patrón Builder**

En lugar de un objeto de configuración, usaremos el **patrón Builder**, que es más idiomático y seguro en Rust. Crearemos una librería (un nuevo crate, p. ej., `avp_middleware`) que exponga este builder.

El corazón de la librería será un `trait` que el usuario debe implementar, similar al `requestMapper` de JavaScript. Este `trait` será el contrato para extraer la información de autorización de una solicitud.

```rust
// En el crate del middleware: src/extractor.rs

use async_trait::async_trait;

/// Define las partes de una solicitud de autorización que son específicas de la aplicación.
pub struct AuthorizationRequestParts {
    pub action: ActionIdentifier,
    pub resource: ResourceIdentifier,
    pub context: Option<serde_json::Value>,
}

/// El trait que una aplicación debe implementar para mapear una solicitud HTTP
/// a las partes necesarias para una decisión de autorización.
#[async_trait]
pub trait AuthorizationRequestExtractor<Req> {
    type Error: std.error::Error + Send + Sync + 'static;

    /// Implementa la lógica para extraer la acción, el recurso y el contexto
    /// de la solicitud específica de tu framework.
    async fn extract_parts(&self, req: &Req) -> Result<AuthorizationRequestParts, Self::Error>;
}
```

### **2. El Middleware Builder: La Interfaz de Usuario del SDK**

El builder hará que la configuración sea declarativa y a prueba de errores.

```rust
// En el crate del middleware: src/builder.rs

use tower::Layer; // Usamos los traits de Tower para máxima compatibilidad (Axum se basa en esto)

pub struct VerifiedPermissionsMiddlewareBuilder<E, C> {
    client: Arc<C>,
    policy_store_id: String,
    extractor: Arc<E>,
}

impl<E, C> VerifiedPermissionsMiddlewareBuilder<E, C> {
    pub fn new(
        client: Arc<C>,
        policy_store_id: impl Into<String>,
        extractor: Arc<E>,
    ) -> Self {
        // ...
    }

    /// Construye la capa de middleware compatible con Axum/Tower.
    pub fn build(self) -> VerifiedPermissionsLayer {
        VerifiedPermissionsLayer { inner: Arc::new(self) }
    }
}
```

### **3. La Lógica del Middleware (La Capa de Axum/Tower)**

Esta es la implementación interna que realiza el trabajo. Se integra con el ecosistema de Axum (`tower::Layer` y `tower::Service`).

```rust
// En el crate del middleware: src/layer.rs

use axum::http::{Request, Response, StatusCode};
use futures::future::BoxFuture;
use tower::{Layer, Service};
use std::task::{Context, Poll};

// ... (builder y otras importaciones)

#[derive(Clone)]
pub struct VerifiedPermissionsLayer {
    // Usamos un Arc para compartir la configuración de forma eficiente.
    inner: Arc<VerifiedPermissionsMiddlewareBuilder<E, C>>,
}

// Implementación de Layer para que se pueda usar con .layer() o .route_layer() en Axum
impl<S> Layer<S> for VerifiedPermissionsLayer {
    type Service = VerifiedPermissionsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        VerifiedPermissionsMiddleware {
            inner,
            config: self.inner.clone(),
        }
    }
}

// El middleware real que envuelve al servicio de la ruta
#[derive(Clone)]
pub struct VerifiedPermissionsMiddleware<S> {
    inner: S,
    config: Arc<VerifiedPermissionsMiddlewareBuilder<E, C>>,
}

// La implementación del servicio donde ocurre la magia
impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for VerifiedPermissionsMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    // ... otras restricciones de traits
{
    // ...

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 1. Extraer el token del header 'Authorization'
            let token = match extract_token_from_header(req.headers()) {
                Ok(t) => t,
                Err(e) => return Ok(e.into_response()), // Retorna 401 Unauthorized
            };

            // 2. Usar el extractor del usuario para obtener las partes de la solicitud
            let parts = match config.extractor.extract_parts(&req).await {
                Ok(p) => p,
                Err(_) => return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(...).unwrap()),
            };

            // 3. Construir la solicitud al servicio de autorización
            let auth_req = IsAuthorizedWithTokenRequest::builder(config.policy_store_id.clone(), token)
                .action(parts.action.entity_type, parts.action.entity_id)
                .resource(parts.resource.entity_type, parts.resource.entity_id)
                // Opcionalmente añadir contexto si el extractor lo proveyó
                .build();

            // 4. Realizar la llamada de autorización
            let decision = match config.client.is_authorized_with_token(auth_req).await {
                Ok(d) => d,
                Err(_) => return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(...).unwrap()),
            };

            // 5. Aplicar la decisión
            if decision.is_allowed() {
                // Decisión: ALLOW
                // **INSPIRACIÓN EXPRESS**: Pasar contexto a los siguientes handlers
                // Insertamos la decisión completa en las extensiones de la solicitud.
                req.extensions_mut().insert(decision);
                // Llamar al siguiente servicio en la cadena (el handler de la ruta)
                inner.call(req).await
            } else {
                // Decisión: DENY
                // Terminar la cadena y devolver 403 Forbidden
                Ok(Response::builder().status(StatusCode::FORBIDDEN).body(...).unwrap())
            }
        })
    }
}
```
---

### **4. Caso de Uso Completo: Integración en una Aplicación Axum**

Así es como un desarrollador usaría nuestra nueva librería de middleware.

**Paso 1: La aplicación implementa el `trait` `AuthorizationRequestExtractor`**

Esta es la única pieza de código "compleja" que el desarrollador debe escribir, y es específica para su API.

```rust
// En la aplicación del usuario: src/auth_extractor.rs

use my_avp_middleware::{AuthorizationRequestExtractor, AuthorizationRequestParts};
use axum::extract::Path;
use axum::http::Request;

// Nuestra implementación específica para la API
pub struct ApiRequestExtractor;

#[async_trait]
impl<B: Send + Sync> AuthorizationRequestExtractor<Request<B>> for ApiRequestExtractor {
    type Error = anyhow::Error;

    async fn extract_parts(&self, req: &Request<B>) -> Result<AuthorizationRequestParts, Self::Error> {
        let method = req.method().clone();
        
        // Extraer los parámetros de la ruta de forma segura
        let params: Path<HashMap<String, String>> = Path::try_from_uri(req.uri())?;
        
        // Mapear método HTTP a una acción de Cedar
        let action_id = match method {
            Method::GET => "read",
            Method::PUT | Method::PATCH => "update",
            Method::POST => "create",
            Method::DELETE => "delete",
            _ => "unknown"
        };
        
        let action = ActionIdentifier::new("Action", action_id);

        // Ejemplo para una ruta como /documents/:docId
        if let Some(doc_id) = params.get("docId") {
            let resource = ResourceIdentifier::new("Document", doc_id);
            
            return Ok(AuthorizationRequestParts {
                action,
                resource,
                context: None, // Podríamos extraer info del body o headers aquí
            });
        }
        
        // ... otra lógica para otras rutas ...
        
        Err(anyhow::anyhow!("No authorization mapping for this route"))
    }
}
```

**Paso 2: Configurar y aplicar el middleware en `main.rs`**

Esta parte ahora se vuelve increíblemente declarativa y limpia, muy al estilo de Express.

```rust
// En la aplicación del usuario: src/main.rs

use my_avp_middleware::VerifiedPermissionsMiddlewareBuilder;
use my_avp_sdk::GrpcClient; // El cliente del SDK
use axum::{routing::get, Router, Extension};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 1. Inicializar el cliente del SDK de autorización
    let auth_client = Arc::new(GrpcClient::connect("http://auth-service:50051").await.unwrap());
    
    // 2. Inicializar nuestro extractor específico
    let extractor = Arc::new(ApiRequestExtractor);

    // 3. Usar el builder para configurar el middleware
    let auth_layer = VerifiedPermissionsMiddlewareBuilder::new(
        auth_client.clone(),
        "my-app-policy-store-id", // ID del Policy Store
        extractor,
    ).build();

    let app = Router::new()
        .route(
            "/documents/:docId",
            get(get_document_handler).patch(update_document_handler)
        )
        // Aplicar el middleware a todas las rutas definidas arriba. ¡Declarativo!
        .route_layer(auth_layer);
    
    // ... iniciar servidor ...
}

// El handler ahora puede asumir que la autorización fue exitosa
async fn get_document_handler(
    // Opcional: podemos extraer la decisión para logging detallado
    Extension(decision): Extension<AuthorizationDecision>
) -> &'static str {
    // Sabemos que la decisión fue "Allow", pero podemos registrar por qué.
    log::info!("Access granted by policies: {:?}", decision.determining_policies);
    
    "Aquí está tu documento secreto."
}
```

### **Resumen de los Beneficios de este Enfoque**

1.  **Declarativo:** La configuración en `main.rs` es limpia y expresa la *intención* sin mostrar la complejidad de la implementación.
2.  **Reusable:** La `VerifiedPermissionsLayer` se puede aplicar a diferentes `Routers` o rutas individuales.
3.  **Desacoplado:** La lógica de la aplicación (el `ApiRequestExtractor`) está completamente separada de la lógica del middleware.
4.  **Seguro por Tipos:** El compilador de Rust garantiza que todas las piezas encajen correctamente.
5.  **Extensible:** Inspirado en Express, el middleware puede pasar información enriquecida (como `AuthorizationDecision`) a los handlers a través de las extensiones de la solicitud, para casos de uso avanzados como la auditoría.

---
Aquí tienes un documento completo de requisitos funcionales y no funcionales, estructurado en épicas e historias de usuario, que abarca todas las funcionalidades avanzadas discutidas: la integración profunda con proveedores de identidad, el SDK de cliente mejorado y el middleware de integración para frameworks web.

Este documento está diseñado para servir como una guía detallada para la implementación, haciendo referencia conceptual a los ejemplos de código propuestos anteriormente.

---

### **Documento de Requisitos y Diseño: Plataforma de Autorización Avanzada (Clon de AVP)**

**Versión:** 1.0
**Fecha:** 18 de octubre de 2025

#### **1. Resumen Ejecutivo**

Este documento detalla los requisitos para extender la plataforma de autorización, llevándola más allá del MVP para incluir funcionalidades de nivel empresarial. El enfoque principal está en tres áreas estratégicas:

1.  **Integración Profunda con Proveedores de Identidad (IdP):** Centralizar la validación de tokens JWT y la traducción de identidades, eliminando esta carga de las aplicaciones cliente.
2.  **SDK de Cliente Ergonómico:** Proporcionar una librería cliente en Rust que sea potente, segura y un placer de usar para los desarrolladores de aplicaciones.
3.  **Middleware de Integración Web:** Ofrecer una capa de middleware configurable y reutilizable para integrar la autorización de forma declarativa en frameworks web modernos como Axum.

El objetivo es crear un sistema que no solo tome decisiones de autorización, sino que se integre de forma nativa y segura en el ecosistema de identidad y desarrollo de aplicaciones modernas.

---

### **Épica 18: Integración Profunda con Proveedores de Identidad (Identity Sources)**

*   **Objetivo:** Centralizar la lógica de validación de tokens JWT y el mapeo de identidades dentro del servicio, simplificando radicalmente las aplicaciones cliente y mejorando la seguridad general del sistema.
*   **Referencia de Diseño:** La funcionalidad `IsAuthorizedWithToken` de AWS Verified Permissions y su concepto de "Fuentes de Identidad".

**Requisitos Funcionales:**

*   **HU 18.1: Configurar una Fuente de Identidad OIDC a través de la API**
    *   **Como** administrador de la plataforma,
    *   **quiero** poder crear y gestionar una "Fuente de Identidad" para un `PolicyStore` específico a través de una API del Plano de Control,
    *   **para que** pueda establecer una relación de confianza formal con un proveedor de identidad externo compatible con OpenID Connect (como Keycloak u Okta).
    *   **Criterios de Aceptación:**
        *   La API (gRPC) debe permitir especificar el `issuer_uri` y la `audience` del proveedor OIDC.
        *   El sistema debe utilizar el `issuer_uri` para descubrir automáticamente la URL del JWKS (JSON Web Key Set) a través del documento de descubrimiento `.well-known/openid-configuration`.
        *   La configuración se almacena de forma segura y se asocia a un único `PolicyStore`.

*   **HU 18.2: Definir Mapeo de Notificaciones (Claims) a Entidades Cedar**
    *   **Como** administrador de seguridad,
    *   **quiero** especificar en la configuración de la Fuente de Identidad cómo se mapean las notificaciones del JWT a la estructura de entidades de Cedar,
    *   **para que** mis políticas de autorización puedan tomar decisiones basadas en la información rica del proveedor de identidad.
    *   **Criterios de Aceptación:**
        *   La configuración debe permitir definir qué claim se usa para el ID único del `principal` (p. ej., `sub`).
        *   Debe permitir definir un claim cuyos valores (típicamente un array de strings) se conviertan en entidades padre del `principal` (p. ej., mapear el claim `groups` a roles de Cedar), habilitando el RBAC.
        *   Debe permitir mapeos de claims arbitrarios a atributos del `principal` (p. ej., `{"department": "custom:department_claim"}`), habilitando el ABAC.

*   **HU 18.3: Autorizar Solicitudes Basadas en Tokens JWT**
    *   **Como** desarrollador de aplicaciones (a través del SDK),
    *   **quiero** invocar un endpoint `IsAuthorizedWithToken` en el Plano de Datos, pasando únicamente el JWT, la acción y el recurso,
    *   **para que** el servicio de autorización realice la validación completa del token y la traducción de la identidad antes de evaluar las políticas.
    *   **Criterios de Aceptación:**
        *   El servicio debe realizar una validación criptográfica completa del token: firma (usando la clave pública del JWKS), expiración, emisor y audiencia.
        *   Debe construir la entidad `principal` de Cedar con sus atributos y padres según el `ClaimMapping` configurado.
        *   El resto del proceso de evaluación de políticas debe ser idéntico al del endpoint `IsAuthorized`.

**Requisitos No Funcionales:**

*   **Rendimiento:** El proceso de obtención y validación de claves JWKS debe ser cacheado agresivamente para no introducir latencia significativa en las llamadas de autorización. La latencia de `IsAuthorizedWithToken` debe ser comparable a la de `IsAuthorized` en llamadas sucesivas.
*   **Seguridad:** El servicio nunca debe confiar en un JWT hasta que su firma y todas sus notificaciones relevantes hayan sido validadas criptográficamente.

---

### **Épica 11 (Revisada): SDK de Cliente Ergonómico y Potente**

*   **Objetivo:** Mejorar el SDK de cliente en Rust para que sea idiomático, seguro por tipos y abstraiga completamente la complejidad de la comunicación con el servicio de autorización, incluyendo la nueva funcionalidad basada en tokens.
*   **Referencia de Diseño:** Mejores prácticas del ecosistema de Rust (patrón Builder, manejo de errores tipado) y la simplicidad de los SDK de AWS.

**Requisitos Funcionales:**

*   **HU 11.2 (Revisada): Exponer la Funcionalidad de Autorización Basada en Tokens**
    *   **Como** desarrollador de aplicaciones Rust,
    *   **quiero** una función `async fn is_authorized_with_token(...)` en el cliente del SDK,
    *   **para que** pueda aprovechar la nueva funcionalidad del servicio de forma sencilla y directa.
    *   **Criterios de Aceptación:**
        *   El `trait AuthorizationClient` se extiende para incluir el nuevo método, asegurando la capacidad de mocking.
        *   La implementación del cliente gRPC maneja la serialización y deserialización de los tipos de solicitud/respuesta.

*   **HU 12.1 (Revisada): Proporcionar Builders para Todas las Solicitudes**
    *   **Como** desarrollador de aplicaciones Rust,
    *   **quiero** usar un patrón `Builder` fluido para construir tanto `IsAuthorizedRequest` como `IsAuthorizedWithTokenRequest`,
    *   **para que** mi código sea más legible, mantenible y menos propenso a errores al configurar opcionales como `context` o `entities`.
    *   **Implementación de Referencia:** Ver el diseño del builder en la sección de SDK y middleware.

*   **HU 13.1 (Revisada): Garantizar la Capacidad de Pruebas Unitarias**
    *   **Como** desarrollador de aplicaciones Rust,
    *   **quiero** que toda la API pública del cliente esté definida en un `trait`,
    *   **para que** pueda crear fácilmente dobles de prueba (mocks) de la librería de autorización y probar la lógica de mi aplicación de forma aislada y determinista.

---

### **Épica 22: Middleware de Integración para Frameworks Web**

*   **Objetivo:** Proporcionar una librería de middleware configurable ("baterías incluidas") que permita a los desarrolladores integrar la autorización en sus aplicaciones web (inicialmente Axum) de forma declarativa y con un mínimo de código repetitivo.
*   **Referencia de Diseño:** La simplicidad y filosofía de configuración de los middlewares de Express.js.

**Requisitos Funcionales:**

*   **HU 22.1: Definir un Contrato de Extracción de Solicitudes**
    *   **Como** diseñador de la librería de middleware,
    *   **quiero** definir un `trait` (`AuthorizationRequestExtractor`) que sirva como el contrato que las aplicaciones deben implementar,
    *   **para que** la lógica del middleware sea genérica y reutilizable, mientras que el mapeo específico de la aplicación (ruta -> acción/recurso) permanezca desacoplado.
    *   **Implementación de Referencia:** Ver el diseño del `trait AuthorizationRequestExtractor` en la sección de middleware.

*   **HU 22.2: Proveer un Builder de Middleware Configurable**
    *   **Como** desarrollador de aplicaciones Rust,
    *   **quiero** usar un patrón `Builder` para configurar la capa de middleware, inyectando mi cliente del SDK, el ID del `PolicyStore` y mi implementación del `Extractor`,
    *   **para que** la configuración de la autorización en mi aplicación sea declarativa y fácil de entender.

*   **HU 22.3: Implementar la Lógica de Middleware para el Ecosistema Tower/Axum**
    *   **Como** desarrollador de aplicaciones Axum,
    *   **quiero** aplicar el middleware configurado a mis rutas usando el método `.layer()` o `.route_layer()` estándar de Axum,
    *   **para que** las solicitudes no autorizadas sean rechazadas automáticamente con un código de estado `403 Forbidden` antes de que lleguen a mis handlers de ruta.
    *   **Criterios de Aceptación:**
        *   El middleware debe extraer automáticamente el token JWT del header `Authorization: Bearer`.
        *   Debe invocar el `Extractor` proporcionado por el usuario para obtener la acción y el recurso.
        *   Debe llamar a `is_authorized_with_token` del SDK con toda la información recopilada.
        *   Debe rechazar la solicitud o pasarla al siguiente servicio en la cadena basándose en la decisión.

*   **HU 22.4: Pasar Contexto de Decisión a los Handlers de Ruta**
    *   **Como** desarrollador de aplicaciones Rust,
    *   **quiero** (opcionalmente) acceder a la `AuthorizationDecision` completa dentro de mi handler de ruta si la autorización fue exitosa,
    *   **para que** pueda implementar lógica avanzada, como el logging de auditoría detallado, que registre qué políticas permitieron el acceso.
    *   **Criterios de Aceptación:**
        *   Si la decisión es `Allow`, el middleware debe insertar la estructura `AuthorizationDecision` en las extensiones de la solicitud de Axum (`req.extensions_mut().insert(...)`).
        *   Los handlers pueden extraer esta información usando el extractor `Extension<AuthorizationDecision>` de Axum.