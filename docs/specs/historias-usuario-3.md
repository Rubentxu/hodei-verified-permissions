# Historias de Usuario

Aquí tienes un conjunto de épicas e historias de usuario diseñadas específicamente para la interfaz web de gestión.

**Persona Principal:** Administrador de Seguridad / DevOps / Desarrollador a cargo de la autorización.

---

### **Épica 14: Navegación y Gestión Fundamental de Almacenes de Políticas (Policy Stores)**

*   **Objetivo:** Crear el esqueleto de la aplicación web, permitiendo a los usuarios ver y gestionar los contenedores de más alto nivel: los `PolicyStores`.

**Historias de Usuario:**

*   **HU 14.1: Ver una lista de todos los Almacenes de Políticas**
    *   **Como** administrador,
    *   **quiero** ver una lista de todos los `PolicyStores` existentes en una página principal o dashboard,
    *   **para que** pueda tener una visión general de todas las aplicaciones que están utilizando el servicio de autorización.
    *   **Criterios de Aceptación:**
        *   La página de inicio muestra una tabla con el nombre/ID y la descripción de cada `PolicyStore`.
        *   Cada elemento de la lista es un enlace que lleva a la vista de detalles de ese `PolicyStore`.

*   **HU 14.2: Crear un nuevo Almacén de Políticas a través de un formulario**
    *   **Como** administrador,
    *   **quiero** poder crear un nuevo `PolicyStore` utilizando un formulario simple en la interfaz web,
    *   **para que** pueda aprovisionar un nuevo entorno de autorización para una nueva aplicación sin usar la CLI o la API.
    *   **Criterios de Aceptación:**
        *   Hay un botón de "Crear Policy Store".
        *   El formulario pide un nombre/descripción y, al enviarlo, realiza la llamada a la API del Plano de Control.
        *   Tras la creación exitosa, se me redirige a la página de detalles del nuevo `PolicyStore`.

*   **HU 14.3: Ver la página de detalles de un Almacén de Políticas**
    *   **Como** administrador,
    *   **quiero** navegar a una página de detalles para un `PolicyStore` específico,
    *   **para que** pueda gestionar los recursos que contiene (esquema, políticas, etc.).
    *   **Criterios de Aceptación:**
        *   La página de detalles muestra el ID del `PolicyStore`.
        *   La navegación principal (p. ej., pestañas o menú lateral) permite acceder a las secciones de "Esquema", "Políticas", "Plantillas", etc.

---

### **Épica 15: Edición y Validación Visual del Esquema**

*   **Objetivo:** Proporcionar una experiencia de usuario de alta calidad para definir y modificar el esquema, que es la base para la validación de políticas.

**Historias de Usuario:**

*   **HU 15.1: Ver y editar el esquema en un editor de código**
    *   **Como** administrador,
    *   **quiero** un editor de texto dentro de la UI para ver y modificar el esquema JSON de mi `PolicyStore`,
    *   **para que** pueda definir los tipos de entidades, acciones y atributos de mi aplicación.
    *   **Criterios de Aceptación:**
        *   Dentro de la página de detalles del `PolicyStore`, hay una sección de "Esquema".
        *   Se muestra el esquema actual en un editor de código con resaltado de sintaxis para JSON.
        *   Puedo modificar el texto y guardar los cambios.

*   **HU 15.2: Recibir validación del esquema en tiempo real**
    *   **Como** administrador,
    *   **quiero** que el editor de esquemas me informe inmediatamente si el JSON que estoy escribiendo es inválido o no cumple con la estructura de un esquema de Cedar,
    *   **para que** pueda corregir errores antes de intentar guardar.
    *   **Criterios de Aceptación:**
        *   El editor muestra indicadores visuales (p. ej., subrayado rojo) para errores de sintaxis JSON.
        *   Se realiza una validación lógica para asegurar que la estructura del esquema es correcta (p. ej., que `entityTypes`, `actions`, etc., están bien formados).

---

### **Épica 16: Experiencia de Creación y Gestión de Políticas (Policy Authoring)**

*   **Objetivo:** Crear el núcleo de la interfaz: un entorno potente y amigable para escribir, validar y gestionar las políticas de Cedar.

**Historias de Usuario:**

*   **HU 16.1: Listar y filtrar políticas**
    *   **Como** administrador,
    *   **quiero** ver una lista de todas las políticas (estáticas y vinculadas a plantillas) dentro de un `PolicyStore`,
    *   **para que** pueda auditar y gestionar los permisos existentes.
    *   **Criterios de Aceptación:**
        *   La sección de "Políticas" muestra una tabla con el ID de la política, su tipo (`permit`/`forbid`), y un resumen de su contenido.
        *   Puedo filtrar la lista por efecto (`permit`/`forbid`) o buscar por texto en el contenido de la política.

*   **HU 16.2: Crear una política estática con un editor inteligente**
    *   **Como** administrador,
    *   **quiero** un editor de texto para escribir políticas de Cedar que ofrezca resaltado de sintaxis específico del lenguaje Cedar,
    *   **para que** pueda escribir políticas de forma más rápida y con menos errores.
    *   **Criterios de Aceptación:**
        *   El formulario de creación de políticas incluye un editor de código (como Monaco o CodeMirror) configurado para el lenguaje Cedar.
        *   Las palabras clave (`permit`, `forbid`, `when`, `unless`, `in`, `==`) se resaltan correctamente.

*   **HU 16.3: Validar una política contra el esquema en tiempo real**
    *   **Como** administrador,
    *   **quiero** que, mientras escribo una política, el editor me notifique si estoy usando tipos de entidades, atributos o acciones que no existen en el esquema,
    *   **para que** pueda evitar errores lógicos y de escritura antes de guardar.
    *   **Criterios de Aceptación:**
        *   La UI realiza una llamada a la API de validación (`cedar-policy-validator`) en segundo plano.
        *   Si escribo `resource.propietario` pero el esquema define `resource.owner`, el editor me lo señala como un error.
        *   El botón de "Guardar" está deshabilitado si la política no es válida.

---

### **Épica 17: Simulador de Autorización (Testing Sandbox)**

*   **Objetivo:** Replicar una de las características más valiosas de la consola de AVP: la capacidad de probar las políticas con solicitudes simuladas para depurar y verificar el comportamiento del sistema.

**Historias de Usuario:**

*   **HU 17.1: Formular una solicitud de autorización de prueba**
    *   **Como** administrador,
    *   **quiero** acceder a una sección de "Pruebas" o "Simulador" donde pueda introducir un `principal`, una `action`, un `resource` y un `context` JSON,
    *   **para que** pueda simular una solicitud de autorización real.
    *   **Criterios de Aceptación:**
        *   Hay una sección dedicada a las pruebas dentro de un `PolicyStore`.
        *   Existen campos de formulario claros para cada componente del modelo PARC.
        *   El campo de `context` es un editor JSON.

*   **HU 17.2: Proporcionar datos de entidades para la simulación**
    *   **Como** administrador,
    *   **quiero** poder definir un conjunto de entidades (el `entities slice`) en formato JSON que se utilizará en la simulación,
    *   **para que** pueda probar políticas que dependen de jerarquías (p. ej., pertenencia a grupos) o atributos de entidades.
    *   **Criterios de Aceptación:**
        *   Hay un editor JSON para introducir la lista de entidades.
        *   (Avanzado) Se ofrece una UI más estructurada para construir entidades, especificando sus atributos y padres de forma visual.

*   **HU 17.3: Ejecutar la simulación y visualizar los resultados**
    *   **Como** administrador,
    *   **quiero** hacer clic en un botón de "Evaluar" y ver el resultado de la autorización,
    *   **para que** pueda entender por qué se tomó una decisión específica de `Allow` o `Deny`.
    *   **Criterios de Aceptación:**
        *   La UI muestra claramente la decisión final: `ALLOW` o `DENY`.
        *   La UI muestra una lista de las "políticas determinantes" (las que llevaron a esa decisión).
        *   Si la decisión es `Deny` debido a una política `forbid`, esa política se resalta de forma prominente como la causa raíz.
        *   Se muestran los errores de evaluación si ocurrieron (p. ej., un atributo faltante).