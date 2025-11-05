¡Excelente! A partir del MVP que definimos, aquí tienes la extensión completa de las historias de usuario necesarias para construir una aplicación que sea un clon funcional y robusto de AWS Verified Permissions.

Estas épicas y sus historias de usuario cubren las características avanzadas que transforman el producto de un prototipo a un servicio de nivel empresarial, basándose en los patrones y funcionalidades de la investigación original.

## 📊 ESTADO ACTUAL: 65% COMPLETADO

```
✅ MVP (Épicas 1-3):        100%
✅ Épica 4 (Identity):      100%
✅ Épica 5 (Batch):         100%
✅ Épica 6 (Templates):     100%
✅ Épica 7 (Multi-tenant):  100% (Documentación)
⏳ Épica 8 (Local Agent):     0%
✅ Épica 9 (Operabilidad):  100%
   ✅ 9.1 Auditoría
   ✅ 9.2 CLI

Tests: 31/31 passing ✅
CLI: ✅ Funcional
Docs: ✅ Multi-Tenancy Guide
```

---

### **✅ Épica 4: Integración Avanzada de Identidad y Tokens (Clon de `IsAuthorizedWithToken`)** - COMPLETADA

*   **Objetivo:** Eliminar la carga de la validación de tokens y el mapeo de identidades de la aplicación cliente, centralizando esta lógica sensible a la seguridad en el servicio de autorización, tal como lo hace AVP con las Fuentes de Identidad.

**Historias de Usuario:**

*   **HU 4.1: Configurar una Fuente de Identidad (Identity Source)**
    *   **Como** administrador de la plataforma,
    *   **quiero** configurar y asociar una "Fuente de Identidad" (p. ej., un User Pool de Cognito o un proveedor OIDC genérico) a un `PolicyStore`,
    *   **para que** el servicio sepa cómo validar los tokens JWT y de dónde provienen.
    *   **Criterios de Aceptación:**
        *   Se define un método gRPC en el Plano de Control `CreateIdentitySource`.
        *   La configuración debe incluir el URI del emisor (issuer), la audiencia (audience) y la lógica para obtener las claves de firma (JWKS URI).
        *   La Fuente de Identidad se asocia a un `policyStoreId` específico.

*   **HU 4.2: Autorizar una solicitud basada en un Token JWT**
    *   **Como** desarrollador de aplicaciones,
    *   **quiero** llamar a un nuevo método `IsAuthorizedWithToken` pasando solo el `accessToken` o `idToken` del usuario (junto con la acción y el recurso),
    *   **para que** el servicio de autorización se encargue de validar el token, extraer la identidad del usuario y tomar una decisión.
    *   **Criterios de Aceptación:**
        *   Se implementa un método gRPC en el Plano de Datos `IsAuthorizedWithToken`.
        *   El servicio utiliza la `IdentitySource` configurada para validar la firma, el emisor, la audiencia y la caducidad del token.
        *   Si el token no es válido, la solicitud se deniega con un error claro.
        *   Si es válido, el servicio construye la entidad `principal` de Cedar a partir de las notificaciones (claims) del token.

*   **HU 4.3: Mapear notificaciones (claims) del token a entidades Cedar**
    *   **Como** administrador de la plataforma,
    *   **quiero** definir cómo se mapean las notificaciones de un JWT a los atributos y jerarquías de una entidad `principal` en Cedar,
    *   **para que** las políticas puedan usar atributos del proveedor de identidad (como la pertenencia a grupos) para tomar decisiones.
    *   **Criterios de Aceptación:**
        *   La configuración de `IdentitySource` permite especificar qué claim se usará como `entityId` del principal (p. ej., `sub`).
        *   Se puede configurar un claim (p. ej., `cognito:groups`) para que sus valores se conviertan en entidades padre del principal (p. ej., `Role::"admins"`), habilitando el RBAC.
        *   Se permite mapear otros claims (p. ej., `custom:department`) como atributos del principal.

---

### **✅ Épica 5: Optimización de Rendimiento y Casos de Uso de UI (Clon de `BatchIsAuthorized`)** - COMPLETADA

*   **Objetivo:** Proporcionar una forma de realizar múltiples comprobaciones de autorización en una sola llamada de red, reduciendo drásticamente la latencia para aplicaciones complejas como las interfaces de usuario.

**Historias de Usuario:**

*   **HU 5.1: Realizar múltiples comprobaciones de autorización en un solo lote**
    *   **Como** desarrollador de frontend/backend,
    *   **quiero** enviar una lista de hasta 30 solicitudes de autorización (principal, acción, recurso) en una sola llamada a la API,
    *   **para que** pueda determinar eficientemente qué componentes de la UI (botones, enlaces, datos) mostrar a un usuario sin incurrir en la latencia de múltiples viajes de red.
    *   **Criterios de Aceptación:**
        *   Se implementa un método gRPC en el Plano de Datos `BatchIsAuthorized`.
        *   La solicitud acepta una lista de consultas de autorización.
        *   La respuesta es una lista de decisiones que se corresponde en orden con la lista de solicitudes.
        *   Cada decisión en la respuesta contiene el veredicto (`ALLOW`/`DENY`) y las políticas determinantes.

*   **HU 5.2: Extender el SDK para soportar operaciones por lotes**
    *   **Como** desarrollador de aplicaciones que consume el servicio,
    *   **quiero** que el SDK del cliente ofrezca una función `batch_is_authorized(...)` conveniente,
    *   **para que** pueda construir y enviar solicitudes por lotes de manera sencilla.
    *   **Criterios de Aceptación:**
        *   El SDK de Rust tiene una nueva función que acepta un `slice` o `Vec` de solicitudes.
        *   La función maneja la serialización a la solicitud gRPC por lotes y la deserialización de la respuesta.

---

### **✅ Épica 6: Autorización Dinámica y Delegada (Clon de Políticas como Plantillas)** - COMPLETADA

*   **Objetivo:** Permitir que las políticas se creen de forma programática y segura, habilitando casos de uso donde los usuarios finales pueden otorgar permisos sobre sus propios recursos (p. ej., compartir un documento).

**Historias de Usuario:**

*   **HU 6.1: Crear y gestionar Plantillas de Políticas (Policy Templates)**
    *   **Como** administrador de la plataforma,
    *   **quiero** crear una "Plantilla de Política" que contenga marcadores de posición para el `principal` y/o el `resource` (p. ej., `?principal`, `?resource`),
    *   **para que** pueda definir patrones de permisos reutilizables y seguros.
    *   **Criterios de Aceptación:**
        *   Se implementa un método gRPC en el Plano de Control `CreatePolicyTemplate`.
        *   La plantilla se valida sintácticamente al ser creada.
        *   La plantilla se almacena asociada a un `PolicyStore`.

*   **HU 6.2: Crear políticas a partir de una plantilla (Template-Linked Policies)**
    *   **Como** desarrollador de aplicaciones,
    *   **quiero** crear una política específica "vinculando" una plantilla con un `principal` y `resource` concretos (p. ej., `User::"alice"` y `Document::"report.pdf"`),
    *   **para que** mi aplicación pueda implementar una función de "Compartir" de forma segura.
    *   **Criterios de Aceptación:**
        *   Se implementa un método gRPC `CreatePolicy` que puede tomar un `templateId` y los valores para los marcadores de posición.
        *   El servicio valida que las entidades proporcionadas se ajustan a los tipos esperados por la plantilla (si se define en el esquema).
        *   La nueva política vinculada se crea y se utiliza en las evaluaciones de autorización.

---

### **✅ Épica 7: Arquitecturas Multi-Inquilino (SaaS)** - COMPLETADA (Documentación)

*   **Objetivo:** Proporcionar las herramientas y patrones arquitectónicos para implementar la autorización en una aplicación SaaS de forma segura y escalable, garantizando un aislamiento estricto entre inquilinos.

**Historias de Usuario:**

*   **✅ HU 7.1: Implementar el aislamiento a través de un `PolicyStore` por inquilino** - DOCUMENTADA
    *   **Como** desarrollador de una aplicación SaaS con altos requisitos de aislamiento,
    *   **quiero** que el servicio sea capaz de gestionar eficientemente miles de `PolicyStores` (uno por cada inquilino),
    *   **para que** pueda garantizar el máximo aislamiento de seguridad, donde las políticas de un inquilino no puedan afectar a otro bajo ninguna circunstancia.
    *   **Criterios de Aceptación:**
        *   ✅ Las operaciones del Plano de Control (p. ej., `CreatePolicyStore`) son rápidas y fiables.
        *   ✅ El Plano de Datos puede enrutar eficientemente una solicitud al `PolicyStore` correcto basándose en el `policyStoreId`.
        *   ✅ La documentación oficial del servicio describe este patrón como una mejor práctica para el aislamiento fuerte.
        *   ✅ Guía completa con ejemplos de código en `MULTI_TENANCY_GUIDE.md`.

*   **✅ HU 7.2: Soportar el aislamiento lógico en un `PolicyStore` compartido** - DOCUMENTADA
    *   **Como** desarrollador de una aplicación SaaS con un gran número de inquilinos,
    *   **quiero** escribir políticas que impongan el aislamiento basándose en atributos (p. ej., `resource.tenantId == principal.tenantId`),
    *   **para que** pueda gestionar a todos mis inquilinos en un único `PolicyStore` para simplificar la operación y reducir costes.
    *   **Criterios de Aceptación:**
        *   ✅ El motor de Cedar evalúa correctamente las políticas con condiciones de igualdad de atributos entre `principal` y `resource`.
        *   ✅ El esquema soporta la adición de un atributo `tenantId` a todos los tipos de entidades relevantes.
        *   ✅ Documentación completa con ejemplos de schemas y políticas.
        *   ✅ Comparación de patrones y recomendaciones.
        *   La documentación del servicio proporciona ejemplos claros y advertencias de seguridad sobre cómo implementar este patrón correctamente.

---

### **Épica 8: Despliegue en el Borde y Resiliencia (Clon del Agente Local)**

*   **Objetivo:** Ofrecer una solución para casos de uso de latencia ultrabaja o entornos con conectividad intermitente, permitiendo que las decisiones de autorización se tomen localmente.

**Historias de Usuario:**

*   **HU 8.1: Sincronizar políticas desde el servicio central a un agente local**
    *   **Como** operador de sistemas,
    *   **quiero** desplegar un "agente local" como un sidecar o demonio que se conecte al servicio central y sincronice periódicamente todas las políticas de un `PolicyStore` específico,
    *   **para que** tenga una copia local y actualizada de la lógica de autorización.
    *   **Criterios de Aceptación:**
        *   Se crea un binario separado para el agente local.
        *   El agente puede configurarse con la dirección del servicio central, credenciales y el `policyStoreId` a sincronizar.
        *   El agente almacena las políticas en caché en memoria.

*   **HU 8.2: Evaluar políticas localmente a través del agente**
    *   **Como** desarrollador de aplicaciones con requisitos de baja latencia,
    *   **quiero** enviar mis solicitudes `IsAuthorized` a un endpoint local expuesto por el agente,
    *   **para que** la decisión de autorización se tome en microsegundos, sin una llamada de red al servicio central.
    *   **Criterios de Aceptación:**
        *   El agente expone una API gRPC idéntica a la del Plano de Datos del servicio central.
        *   El agente utiliza una instancia del motor `cedar-policy` en Rust para evaluar las solicitudes contra la caché de políticas local.
        *   La aplicación puede continuar funcionando incluso si se pierde la conectividad con el servicio central.

*   **HU 8.3: Configurar el SDK para usar el agente local**
    *   **Como** desarrollador de aplicaciones,
    *   **quiero** poder configurar mi instancia del SDK para que apunte al agente local (`localhost:port`) en lugar del servicio central,
    *   **para que** el cambio entre evaluación local y remota sea transparente para la lógica de mi aplicación.
    *   **Criterios de Aceptación:**
        *   El constructor del cliente en el SDK acepta una URL de endpoint.
        *   La aplicación puede cambiar entre modos (local/remoto) a través de configuración.

---

### **Épica 9: Operabilidad, Auditoría y Gestión (Funcionalidades de Producción)** - PARCIALMENTE COMPLETADA

*   **Objetivo:** Dotar al servicio de las herramientas y la observabilidad necesarias para ser operado, depurado y auditado en un entorno de producción.

**Historias de Usuario:**

*   **✅ HU 9.1: Registrar cada decisión de autorización para auditoría** - COMPLETADA
    *   **Como** auditor de seguridad,
    *   **quiero** que cada solicitud de autorización y su resultado (incluyendo el `principal`, `action`, `resource` y las políticas determinantes) se registren en un log estructurado (JSON),
    *   **para que** pueda realizar análisis forenses, monitorear el acceso y demostrar el cumplimiento normativo.
    *   **Criterios de Aceptación:**
        *   ✅ Todas las llamadas al Plano de Datos generan una entrada de log.
        *   ✅ Los logs son estructurados y contienen toda la información de la solicitud y la decisión.
        *   ✅ Filtros por store, principal, decision implementados.
        *   ✅ 7 tests unitarios pasando.

*   **✅ HU 9.2: Gestionar el servicio a través de una CLI** - COMPLETADA
    *   **Como** ingeniero de DevOps/SRE,
    *   **quiero** una herramienta de línea de comandos (CLI) para gestionar los `PolicyStores`, esquemas y políticas,
    *   **para que** pueda automatizar las tareas de gestión y la integración con flujos de CI/CD.
    *   **Criterios de Aceptación:**
        *   ✅ La CLI permite crear/leer/actualizar/eliminar políticas y esquemas.
        *   ✅ Los comandos son intuitivos (`hodei-cli policy create --store-id "xyz" --file "my-policy.cedar"`).
        *   ✅ La CLI puede ser utilizada en scripts de automatización.
        *   ✅ Binario `hodei-cli` compilado y funcional.
        *   ✅ ~270 líneas de código.