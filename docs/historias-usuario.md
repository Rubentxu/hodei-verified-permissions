### **Análisis y Estrategia del MVP**

El objetivo es crear un servicio de autorización centralizado, desacoplado de la lógica de negocio, utilizando Rust y el motor de políticas Cedar. El MVP se centrará en replicar la funcionalidad principal de AWS Verified Permissions: un **Plano de Datos** de alto rendimiento para decisiones en tiempo real y un **Plano de Control** funcional para gestionar las políticas y los esquemas.

**Arquitectura Propuesta para el MVP:**

1.  **Servicio de Autorización (PDP - Policy Decision Point):**
    *   **Lenguaje:** Rust.
    *   **Núcleo:** El crate `cedar-policy` para el motor de evaluación.
    *   **API:** Un servidor gRPC construido con `tonic` para una comunicación de baja latencia y alto rendimiento.
    *   **Validación:** El crate `cedar-policy-validator` para la validación estática de políticas contra el esquema en el Plano de Control.

2.  **Almacenamiento de Políticas (Policy Store):**
    *   Para el MVP, se utilizará una base de datos simple (como PostgreSQL o SQLite) para persistir los esquemas y las políticas. Esto evita la complejidad de una solución en memoria con caché distribuida en la fase inicial.

3.  **Cliente SDK (PEP - Policy Enforcement Point):**
    *   Un crate de Rust que actuará como cliente del servicio gRPC. Este SDK ofrecerá una API sencilla para que otras aplicaciones (los "middlewares") puedan integrarlo fácilmente y realizar llamadas de autorización.

A continuación, se desglosa el MVP en Épicas e Historias de Usuario.

---

### **Épica 1: Implementación del Plano de Datos Fundamental**

*   **Objetivo:** Crear el núcleo del servicio capaz de recibir una solicitud de autorización y devolver una decisión de `Allow`/`Deny` basada en un conjunto de políticas existentes. Esta es la funcionalidad más crítica.

**Historias de Usuario:**

*   **HU 1.1: Evaluar una solicitud de autorización simple** ✅ **COMPLETADA**
    *   **Como** desarrollador de aplicaciones,
    *   **quiero** enviar una solicitud de autorización con un `principal`, `action` y `resource` definidos a un endpoint gRPC,
    *   **para que** el servicio evalúe las políticas relevantes y me devuelva una decisión final de `ALLOW` o `DENY`.
    *   **Criterios de Aceptación:**
        *   ✅ Se define un método gRPC `IsAuthorized`.
        *   ✅ El servicio puede cargar un conjunto de políticas Cedar desde el almacenamiento.
        *   ✅ El servicio utiliza el crate `cedar-policy` para evaluar la solicitud contra las políticas.
        *   ✅ La respuesta indica claramente la decisión (`ALLOW`/`DENY`) y qué políticas (IDs) determinaron esa decisión.
    *   **Implementación:**
        *   Método IsAuthorized en Data Plane
        *   Evaluación con Cedar Authorizer
        *   Respuesta con decisión y políticas determinantes

*   **HU 1.2: Incluir datos de entidades en la solicitud de autorización** ✅ **COMPLETADA**
    *   **Como** desarrollador de aplicaciones,
    *   **quiero** pasar un conjunto de datos de entidades adicionales (el `entities slice`) en mi llamada `IsAuthorized`,
    *   **para que** el motor Cedar pueda evaluar políticas basadas en atributos y jerarquías (p. ej., `principal in Group::"admins"` o `resource.owner == principal`).
    *   **Criterios de Aceptación:**
        *   ✅ El método gRPC `IsAuthorized` se amplía para aceptar una lista de entidades.
        *   ✅ El servicio deserializa correctamente estos datos y los pasa al motor de Cedar para la evaluación.
        *   ✅ Las políticas que utilizan el operador `in` o que acceden a atributos de entidades se evalúan correctamente.
    *   **Implementación:**
        *   Soporte para entidades con atributos y jerarquías de padres
        *   Conversión de entidades a formato Cedar
        *   Evaluación ABAC completa

*   **HU 1.3: Incorporar el `context` en la decisión** ✅ **COMPLETADA**
    *   **Como** desarrollador de aplicaciones,
    *   **quiero** enviar un `context` con información transitoria (p. ej., dirección IP, hora de la solicitud) en la llamada `IsAuthorized`,
    *   **para que** se puedan evaluar políticas condicionales que dependen de este contexto.
    *   **Criterios de Aceptación:**
        *   ✅ El método gRPC `IsAuthorized` acepta un objeto `context` arbitrario (JSON).
        *   ✅ El servicio pasa este `context` al motor de Cedar.
        *   ✅ Políticas con cláusulas `when` basadas en contexto se evalúan correctamente.
    *   **Implementación:**
        *   Campo context opcional en IsAuthorizedRequest
        *   Parsing de JSON a Cedar Context
        *   Soporte para políticas condicionales

---

### **Épica 2: Gestión de Políticas y Esquemas (Plano de Control Básico)**

*   **Objetivo:** Implementar las funcionalidades básicas para que los administradores puedan definir el modelo de autorización: crear y gestionar los `PolicyStores`, los esquemas y las políticas.

**Historias de Usuario:**

*   **HU 2.1: Crear y gestionar un Almacén de Políticas (Policy Store)** ✅ **COMPLETADA**
    *   **Como** administrador de la plataforma,
    *   **quiero** poder crear un `PolicyStore` como un contenedor aislado para las políticas y esquemas de una aplicación,
    *   **para que** cada aplicación tenga su propio ámbito de autorización bien definido.
    *   **Criterios de Aceptación:**
        *   ✅ Se define un método gRPC `CreatePolicyStore`.
        *   ✅ Cada `PolicyStore` tiene un ID único.
        *   ✅ Las llamadas al Plano de Datos (como `IsAuthorized`) deben especificar el `policyStoreId` al que se dirigen.
    *   **Implementación:**
        *   CRUD completo: Create, Get, List, Delete
        *   UUIDs para identificadores únicos
        *   Almacenamiento en SQLite

*   **HU 2.2: Definir el esquema de una aplicación** ✅ **COMPLETADA**
    *   **Como** administrador de la plataforma,
    *   **quiero** poder enviar y actualizar un esquema para un `PolicyStore` específico,
    *   **para que** se pueda validar la integridad de las políticas antes de guardarlas.
    *   **Criterios de Aceptación:**
        *   ✅ Se define un método gRPC `PutSchema`.
        *   ✅ El servicio persiste el esquema asociado a un `PolicyStore`.
        *   ✅ El formato del esquema es compatible con el que espera Cedar (JSON).
    *   **Implementación:**
        *   Métodos: PutSchema, GetSchema
        *   Validación de formato Cedar al subir
        *   Nota: Validación completa de políticas contra esquema pendiente para iteración futura

*   **HU 2.3: Crear y validar políticas estáticas** ✅ **COMPLETADA**
    *   **Como** administrador de la plataforma,
    *   **quiero** crear una nueva política Cedar y asociarla a un `PolicyStore`,
    *   **para que** el servicio valide la política contra el esquema existente antes de guardarla.
    *   **Criterios de Aceptación:**
        *   ✅ Se define un método gRPC `CreatePolicy`.
        *   ✅ Al recibir una nueva política, el servicio valida su corrección sintáctica.
        *   ✅ Si la política no es válida, el servicio devuelve un error detallado y no la guarda.
        *   ✅ Si es válida, la política se persiste en el almacenamiento asociada a su `PolicyStore`.
    *   **Implementación:**
        *   CRUD completo: Create, Get, Update, Delete, List
        *   Validación sintáctica con Cedar
        *   Validación completa contra esquema: pendiente para iteración futura

---

### **Épica 3: Implementación del Servidor gRPC y el SDK de Cliente**

*   **Objetivo:** Construir la infraestructura de comunicación entre el servicio y las aplicaciones cliente, proporcionando una experiencia de desarrollo fluida.

**Historias de Usuario:**

*   **HU 3.1: Definir el contrato de la API con Protocol Buffers** ✅ **COMPLETADA**
    *   **Como** desarrollador del servicio de autorización,
    *   **quiero** definir todos los servicios y mensajes de la API (Plano de Control y Datos) en un archivo `.proto`,
    *   **para que** se pueda generar código de servidor y cliente de forma consistente y tipada.
    *   **Criterios de Aceptación:**
        *   ✅ Existe un archivo `.proto` que define los servicios `AuthorizationData` (con `IsAuthorized`) y `AuthorizationControl` (con `CreatePolicyStore`, `PutSchema`, `CreatePolicy`).
        *   ✅ Las estructuras de datos para Principal, Acción, Recurso, Contexto, Política y Esquema están claramente definidas como mensajes.
    *   **Implementación:**
        *   Archivo: `proto/authorization.proto`
        *   Servicios: `AuthorizationData` (IsAuthorized, BatchIsAuthorized) y `AuthorizationControl` (gestión completa de policy stores, schemas y policies)
        *   Build script configurado en `build.rs` para generar código con tonic-build

*   **HU 3.2: Implementar el servidor gRPC en Rust** ✅ **COMPLETADA**
    *   **Como** desarrollador del servicio de autorización,
    *   **quiero** implementar la lógica del servidor gRPC utilizando el crate `tonic`,
    *   **para que** las llamadas de los clientes se enruten a la lógica de negocio correspondiente (evaluación o gestión de políticas).
    *   **Criterios de Aceptación:**
        *   ✅ El proyecto Rust incluye un binario de servidor que se inicia y escucha en un puerto configurable.
        *   ✅ El servidor implementa los traits generados a partir del archivo `.proto`.
        *   ✅ Las solicitudes se manejan de forma asíncrona para un alto rendimiento.
    *   **Implementación:**
        *   Servidor gRPC asíncrono con Tokio
        *   Servicios: `AuthorizationDataService` y `AuthorizationControlService`
        *   Puerto configurable via `SERVER_ADDR` (default: 50051)

*   **HU 3.3: Crear un SDK de cliente en Rust** ✅ **COMPLETADA**
    *   **Como** desarrollador de aplicaciones que consume el servicio,
    *   **quiero** un crate de Rust (SDK) simple que abstraiga los detalles de la comunicación gRPC,
    *   **para que** pueda realizar una llamada de autorización de forma idiomática y sencilla desde mi middleware.
    *   **Criterios de Aceptación:**
        *   ✅ Se publica un nuevo crate (`hodei-permissions-sdk`).
        *   ✅ El SDK proporciona funciones fáciles de usar (`is_authorized`, etc.).
        *   ✅ El SDK maneja la configuración del cliente.
    *   **Implementación:**
        *   Crate completo en `/sdk`
        *   Cliente con todos los métodos (data + control plane)
        *   Builder patterns para requests complejos
        *   Manejo de errores robusto
        *   Ejemplos y documentación completa
        *   Nota: Requiere ajuste menor para tonic 0.14.2

---

### **Plan de Evolución Post-MVP**

Una vez que el MVP esté funcional, se puede seguir esta hoja de ruta para añadir características más avanzadas, basándose en la investigación proporcionada:

1.  **Integración de Identidad (IsAuthorizedWithToken):**
    *   Añadir un endpoint `IsAuthorizedWithToken` que acepte un JWT.
    *   Integrar la validación de JWT (usando crates como `jsonwebtoken`).
    *   Permitir configurar "Fuentes de Identidad" (Identity Sources) para mapear las notificaciones (claims) del token a entidades y atributos de Cedar, replicando una de las características más potentes de AVP.

2.  **Operaciones por Lotes (BatchIsAuthorized):**
    *   Implementar un endpoint `BatchIsAuthorized` que acepte hasta N solicitudes de autorización en una sola llamada, optimizando casos de uso de renderizado de UI.

3.  **Políticas como Código (Policy-as-Code) y Plantillas:**
    *   Añadir soporte para plantillas de políticas (`Policy Templates`) para permitir escenarios de permisos dinámicos (p. ej., "el usuario A comparte el recurso B con el usuario C").
    *   Crear herramientas de CLI o integraciones con CI/CD para gestionar las políticas y los esquemas desde repositorios de Git.

4.  **Soporte para Multi-Inquilinato (Multi-Tenancy):**
    *   Refinar el modelo de `PolicyStore` para soportar explícitamente el patrón de **un PolicyStore por inquilino**, incluyendo la automatización del aprovisionamiento.
    *   Proporcionar directrices y ejemplos para implementar el patrón de **PolicyStore compartido**, que requiere un diseño cuidadoso de políticas.

5.  **Agente de Evaluación Local (Local Agent):**
    *   Desarrollar un "agente local" que pueda ejecutarse como un sidecar. Este agente sincronizaría las políticas desde el servicio central y realizaría las evaluaciones localmente, eliminando la latencia de red para casos de uso de rendimiento ultra bajo. Esto se inspiraría directamente en el `avp-local-agent` de código abierto.