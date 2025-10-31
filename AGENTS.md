# Directrices para Agentes de IA en el Desarrollo de Hodei Verified Permissions

Este documento establece las mejores prácticas para el desarrollo de agentes de IA que contribuyen al proyecto Hodei Verified Permissions. El objetivo es garantizar que los agentes produzcan código de alta calidad, mantenible y robusto, alineado con los principios de diseño del proyecto.

## Principios Fundamentales

### 1. Implementación Real Impulsada por TDD (Test-Driven Development)

**La regla más importante es: no hay atajos.** Todas las implementaciones deben ser reales y estar impulsadas por pruebas.

- **No `fallback`:** El código no debe contener implementaciones alternativas o de respaldo. La funcionalidad debe ser completa.
- **No `placeholders`:** No se permiten marcadores de posición. Si una función no está implementada, no debe existir.
- **No simulaciones (`mocking`) en la lógica de negocio:** Las simulaciones solo se permiten en los límites del sistema (por ejemplo, para simular una base de datos en una prueba unitaria), no para simular la lógica de negocio principal.
- **No `hardcodeo`:** Los valores no deben estar codificados. Utilice archivos de configuración o variables de entorno.

### 2. Principios SOLID

Los agentes deben aplicar los principios SOLID en todo el código que generen:

-   **S - Principio de Responsabilidad Única (SRP):** Cada módulo, clase o función debe tener una única responsabilidad.
-   **O - Principio de Abierto/Cerrado (OCP):** Las entidades de software deben estar abiertas para la extensión, pero cerradas para la modificación. En Rust, esto se logra a menudo mediante el uso de `traits`.
-   **L - Principio de Sustitución de Liskov (LSP):** Los subtipos deben ser sustituibles por sus tipos base. En Rust, esto se traduce en un diseño cuidadoso de los `traits` y sus implementaciones.
-   **I - Principio de Segregación de la Interfaz (ISP):** Ningún cliente debe ser forzado a depender de interfaces que no utiliza. Cree `traits` pequeños y específicos en lugar de `traits` grandes y monolíticos.
-   **D - Principio de Inversión de Dependencia (DIP):** Los módulos de alto nivel no deben depender de los módulos de bajo nivel. Ambos deben depender de abstracciones. En Rust, esto se logra mediante el uso de `traits` y la inyección de dependencias.

### 3. Arquitectura Hexagonal (Puertos y Adaptadores)

El proyecto sigue una arquitectura hexagonal. Los agentes deben respetar esta estructura:

-   **Núcleo de la aplicación (`domain`):** Contiene la lógica de negocio pura, sin dependencias de frameworks o infraestructura.
-   **Puertos (`ports`):** Definen las interfaces (`traits` en Rust) a través de las cuales el núcleo se comunica con el mundo exterior.
-   **Adaptadores (`adapters`):** Implementan los puertos. Hay dos tipos de adaptadores:
    -   **Adaptadores primarios (o de conducción):** Impulsan la aplicación (por ejemplo, un controlador de API gRPC).
    -   **Adaptadores secundarios (o conducidos):** Son impulsados por la aplicación (por ejemplo, una implementación de repositorio de base de datos).

### 4. Idioma y Estilo de Rust

-   **Seguridad y concurrencia:** Aproveche las garantías de seguridad de Rust. Evite el uso de `unsafe` a menos que sea absolutamente necesario y esté debidamente justificado.
-   **Manejo de errores:** Utilice `Result` y `Option` para un manejo de errores explícito y robusto. Evite `panic!` en el código de la biblioteca.
-   **Estilo de código:** Siga las convenciones de formato de `rustfmt`.
-   **Clippy:** Utilice `clippy` para identificar y corregir errores comunes y mejorar el código idiomático.

### 5. Desarrollo y Compilación Rápida en Rust

Para mantener un ciclo de desarrollo ágil, es crucial optimizar los tiempos de compilación y la ejecución de pruebas.

-   **Utilizar `cargo-watch`:** Para una experiencia TDD fluida, usa `cargo-watch` para ejecutar automáticamente las pruebas cada vez que se guarda un archivo.
    ```bash
    cargo install cargo-watch
    cargo watch -x test
    ```

-   **Pruebas Selectivas:** Ejecuta solo las pruebas relevantes para los cambios que estás realizando.
    ```bash
    # Ejecutar pruebas de un módulo específico
    cargo test tests::my_module

    # Ejecutar una prueba específica por nombre
    cargo test my_specific_test_name
    ```

-   **`cargo-nextest`:** Utiliza `cargo-nextest` como una alternativa más rápida y con mejor interfaz a `cargo test`.
    ```bash
    cargo install cargo-nextest
    cargo nextest run
    ```

-   **Evitar Dependencias Pesadas:** Sé consciente del impacto de las dependencias en los tiempos de compilación. Si una dependencia solo se usa para desarrollo, muévela a `[dev-dependencies]`.



### 6. Cohesión y Acoplamiento (Principios de Connascence)

- **Objetivo:** Maximizar la cohesión y minimizar el acoplamiento.
- **Regla general:** Refactorizar para transformar las formas más fuertes de connascence en formas más débiles (ej. de `Connascence of Position` a `Connascence of Name` usando structs).

### 7. Automatización con Makefile

-   **Centralización:** Todos los comandos comunes (compilar, probar, etc.) deben estar en un `Makefile` raíz.
-   **Documentación:** El `Makefile` debe incluir un target `help`.
-   **Simplicidad:** Los agentes deben usar los targets del `Makefile`.

### 8. Requisitos de Configuración del Proyecto Rust

-   **Edición de Rust:** El proyecto debe utilizar la edición **Rust 2024** para aprovechar las últimas características del lenguaje.
-   **Dependencias Centralizadas:** Todas las dependencias de los workspaces deben ser declaradas en el `Cargo.toml` de la raíz del proyecto para garantizar la consistencia.
-   **Dependencias Actualizadas:** Utilice siempre las últimas versiones **estables** de las dependencias. Verifique con `cargo outdated`.

## Flujo de Trabajo del Agente (Spec-First & TDD)

1.  **Comprender el Requisito:** Analice la solicitud del usuario.
2.  **Crear/Actualizar la Especificación (Spec):**
    -   Antes de escribir código, defina el comportamiento esperado en un archivo de especificación (ej. un `spec.md` o un `ADR` - Architecture Decision Record).
    -   Esta spec debe describir las entradas, salidas, y el comportamiento del sistema de forma clara.
3.  **Identificar Archivos Relevantes:** Localice los archivos a modificar o crear.
4.  **Escribir Pruebas que Fallan (TDD):**
    -   Traduzca la especificación a pruebas de aceptación o unitarias que fallen.
    -   Utilice el `Makefile` para ejecutar las pruebas (ej. `make watch-test`).
5.  **Implementar la Lógica:** Escriba el código mínimo necesario para que las pruebas pasen.
6.  **Refactorizar:** Mejore la calidad del código (SOLID, Connascence) sin cambiar el comportamiento.
7.  **Verificar y Actualizar Documentación:**
    -   Asegúrese de que todas las pruebas pasen (`make test`).
    -   Actualice la documentación relevante (READMEs, specs, etc.) para reflejar los cambios.

Al seguir estas directrices, los agentes de IA contribuirán a mantener la alta calidad y la integridad del proyecto Hodei Verified Permissions.
