# Historias de Usuario 7 - Frontend Web-NextJS Completo para Verified Permissions

## Resumen Ejecutivo

Este documento define la implementación completa de una interfaz web para Hodei Verified Permissions, diseñada para replicar toda la funcionalidad disponible en la consola web de AWS Verified Permissions (AVP). La implementación se basa en una ingeniería inversa exhaustiva de AVP y aprovehandose del protocolo gRPC nativo implementado en el backend.

## 1. Arquitectura y Tecnologías

- **Frontend**: Next.js 16 con React 19, TypeScript y Tailwind CSS
- **Comunicación**: API Routes de Next.js que hablan gRPC nativo con el servidor
- **Editor de Código**: Monaco Editor para sintaxis highlighting de Cedar
- **UI/UX**: Diseño responsive y accesible, inspirado en la consola AWS
- **Testing**: Playwright para E2E testing automatizado

## 2. Funcionalidades Implementadas vs Funcionalidades Requeridas

### 2.1. ✅ Implementado (Historias de Usuario 6)
- Gestión básica de Policy Stores (crear, listar, eliminar)
- Gestión básica de Esquemas con editor JSON
- Gestión básica de Políticas (crear, editar, listar, eliminar)
- Gestión básica de Plantillas de Políticas (crear, listar, eliminar, ver)
- Dashboard básico con health check
- Test Bench básico para autorización
- Navegación lateral entre secciones
- Client gRPC con @grpc/grpc-js y @grpc/proto-loader

### 2.2. 🔄 Implementación Requerida (Historias de Usuario 7)

## 3. Historias de Usuario Detalladas

### HU1: Dashboard Completo con Métricas y Monitoreo

**Como:** Administrador del sistema
**Quiero:** Un dashboard que me proporcione una visión completa del estado del sistema
**Para:** Monitorear la actividad, salud y rendimiento de mis policy stores

#### Criterios de Aceptación:
- [ ] Métricas de actividad: número de policy stores, políticas totales, últimas actualizaciones
- [ ] Gráficos de uso de autorización requests
- [ ] Estado de salud de conexiones gRPC en tiempo real
- [ ] Actividad reciente: últimos cambios en políticas, esquemas, etc.
- [ ] Alertas y warnings sobre configuraciones problemáticas
- [ ] Contador de requests de autorización en tiempo real
- [ ] Historial de auditoría básico

**Funcionalidades Técnicas:**
- WebSocket para updates en tiempo real
- Componentes de métricas con Chart.js o Recharts
- Estado de conexiones con indicadores visuales
- Data fetching optimizado con React Query/SWR

---

### HU2: Gestión Avanzada de Policy Stores

**Como:** Administrador
**Quiero:** Funcionalidades avanzadas para gestionar múltiples policy stores
**Para:** Organizar y administrar eficientemente mis contenedores de políticas

#### Criterios de Aceptación:
- [ ] **Search y Filtering Avanzado**
  - Buscar por nombre, descripción, fecha de creación
  - Filtros por estado (activo/inactivo)
  - Filtros por última modificación
  - Sorting por diferentes columnas
- [ ] **Bulk Operations**
  - Seleccionar múltiples policy stores
  - Eliminar en lote
  - Exportar múltiples stores
  - Cambiar estado en lote
- [ ] **Tagging System**
  - Agregar tags personalizados a policy stores
  - Filtrar por tags
  - Gestión de tags populares
- [ ] **Export/Import**
  - Exportar policy store como JSON
  - Importar desde JSON
  - Backup completo de stores
- [ ] **Versioning Information**
  - Mostrar historial de cambios
  - Rollback capabilities (si está en API)
  - Diff between versions

**Funcionalidades Técnicas:**
- Pagination con virtual scrolling para grandes datasets
- Advanced filtering con react-table
- Bulk selection UI
- Drag & drop para reordenamiento
- Contextual menus (right-click)

---

### HU3: Editor de Esquemas Avanzado con Entity Management

**Como:** Arquitecto de seguridad
**Quiero:** Herramientas completas para definir y gestionar el esquema de entidades
**Para:** Establecer la estructura de datos para autorización

#### Criterios de Aceptación:
- [ ] **Entity Type Management**
  - Crear, editar, eliminar tipos de entidades
  - Definir atributos para cada tipo
  - Configurar tipos de datos (string, number, boolean, set)
  - Validación de tipos de datos
- [ ] **Action Groups**
  - Crear grupos de acciones relacionadas
  - Jerarquía de actions
  - Herencia de actions
- [ ] **Schema Templates**
  - Templates predefinidos para casos comunes
  - Guardar schemas como templates
  - Aplicar templates a nuevos stores
- [ ] **Advanced Validation**
  - Validación en tiempo real del schema
  - Detección de inconsistencias
  - Referencias cruzadas entre entidades
  - Sugerencias de mejora
- [ ] **Schema Versioning**
  - Historial de cambios
  - Diff entre versiones
  - Rollback capability

**Funcionalidades Técnicas:**
- JSON Schema validation
- Entity relationship visualization
- Auto-completion para entity types
- Syntax highlighting específico para Cedar schema
- Visual schema designer (opcional)

---

### HU4: Editor de Políticas con Modo Wizard

**Como:** Desarrollador/Administrador
**Quiero:** Formularios guiados para crear políticas sin conocimiento profundo de Cedar
**Para:** Crear políticas de manera eficiente y sin errores

#### Criterios de Aceptación:
- [ ] **Policy Creation Wizard**
  - Step-by-step guided process
  - Form-based input para principal, resource, action
  - Selector visual para entidades disponibles
  - Builder de conditions (when/unless)
  - Preview en tiempo real del Cedar generado
- [ ] **Dual Editor Mode**
  - Toggle entre wizard y code editor
  - Sync entre ambos modos
  - Validación cruzada
- [ ] **Policy Templates Integration**
  - Usar templates desde el wizard
  - Customización de templates
  - Guardar configuraciones como templates
- [ ] **Validation and Suggestions**
  - Real-time Cedar syntax validation
  - Schema compliance checking
  - Auto-suggestions y auto-complete
  - Error highlighting con explanations
- [ ] **Policy Operations**
  - Copy/Duplicate policies
  - Export individual policies
  - Policy versioning
  - Change tracking

**Funcionalidades Técnicas:**
- CodeMirror o Monaco con Cedar syntax
- Form validation con react-hook-form
- Auto-complete con LSP integration
- Real-time preview generation
- Drag & drop para policy elements

---

### HU5: Gestión Completa de Policy Templates

**Como:** Arquitecto de seguridad
**Quiero:** Sistema completo de templates para reutilización de lógica
**Para:** Estandarizar y acelerar la creación de políticas

#### Criterios de Aceptación:
- [ ] **Template Categories**
  - Categorizar templates por tipo de uso
  - Templates built-in y custom
  - Rating y popularity system
- [ ] **Template Parameters**
  - Definir parámetros requeridos/opcionales
  - Validation de parámetros
  - Default values y examples
- [ ] **Template Application**
  - Aplicar templates con UI step-by-step
  - Preview del resultado final
  - Batch application de templates
- [ ] **Template Management**
  - Import/export de templates
  - Template sharing entre teams
  - Template versioning
  - Usage analytics

**Funcionalidades Técnicas:**
- Template engine con Handlebars-like syntax
- Parameter form generation automático
- Template library browser
- Usage tracking y analytics

---

### HU6: Authorization Playground Avanzado

**Como:** Desarrollador
**Quiero:** Herramientas completas para probar y depurar autorización
**Para:** Verificar que mis políticas funcionan correctamente

#### Criterios de Aceptación:
- [ ] **Multi-Scenario Testing**
  - Guardar/cargar casos de prueba
  - Templates de testing comunes
  - Batch testing de múltiples scenarios
- [ ] **Debug Mode**
  - Step-by-step evaluation
  - Visualización del policy evaluation path
  - Detail de cada policy match
  - Context exploration
- [ ] **Performance Testing**
  - Load testing de authorization
  - Performance metrics
  - Batch authorization testing
- [ ] **Test Results Analysis**
  - Detailed decision explanation
  - Policy impact analysis
  - Suggestion de policies faltantes
  - Coverage analysis

**Funcionalidades Técnicas:**
- Advanced debugging UI
- Real-time policy evaluation visualization
- Performance monitoring
- Test case management con database
- Export de test results

---

### HU7: Gestión de Identity Sources

**Como:** Administrador
**Quiero:** Integrar diferentes fuentes de identidad (Cognito, OIDC)
**Para:** Conectar mi sistema de autenticación con authorization

#### Criterios de Aceptación:
- [ ] **Cognito Integration**
  - Select de User Pool via dropdown
  - Client IDs configuration
  - Group configuration
  - Connection testing
- [ ] **OIDC Integration**
  - Generic OIDC provider setup
  - Issuer URL configuration
  - JWKS URI setup
  - Claims mapping
  - Certificate management
- [ ] **Identity Source Management**
  - View connected identity sources
  - Update/reconfigure existing
  - Disconnect identity sources
  - Connection health monitoring
- [ ] **Principal Mapping**
  - Map identity sources to principal types
  - Attribute mapping configuration
  - Group/role extraction setup

**Funcionalidades Técnicas:**
- OIDC client configuration
- JWT token validation
- Claims extraction y mapping
- Health check endpoints
- Certificate validation

---

### HU8: Advanced Search y Filtering

**Como:** Administrador/Desarrollador
**Quiero:** Búsqueda y filtrado potente en toda la aplicación
**Para:** Encontrar rápidamente policies, templates y configuraciones específicas

#### Criterios de Aceptación:
- [ ] **Global Search**
  - Search across policy stores, policies, templates
  - Full-text search en policy content
  - Search en schema definitions
  - Search suggestions y auto-complete
- [ ] **Advanced Filtering**
  - Filter por tipo de policy (static/template-linked)
  - Filter por principal/resource/action types
  - Filter por fechas y autores
  - Filter por estado de validación
- [ ] **Saved Searches**
  - Guardar queries de búsqueda frecuentes
  - Shared searches entre team members
  - Quick access a searches populares
- [ ] **Search Results Management**
  - Bulk operations en search results
  - Export search results
  - Sort y reorder results

**Funcionalidades Técnicas:**
- Elasticsearch integration (opcional)
- Client-side search para datasets pequeños
- Search indexing
- Query parser y autocomplete

---

### HU9: Audit Trail y Change Management

**Como:** Auditor/Administrador
**Quiero:** Tracking completo de cambios y actividades
**Para:** Cumplimiento y troubleshooting

#### Criterios de Aceptación:
- [ ] **Change Logging**
  - Track de todos los cambios en policies, schemas, templates
  - User attribution para cada change
  - Timestamp precision
  - Change description y reason
- [ ] **Activity Dashboard**
  - Recent activity feed
  - Activity by user/date/type
  - Change frequency analytics
  - Anomaly detection en changes
- [ ] **Audit Export**
  - Export audit logs
  - Compliance reports
  - Custom date ranges
  - Multiple formats (JSON, CSV, PDF)
- [ ] **Rollback Capabilities**
  - Undo recent changes (donde sea posible)
  - Restore previous versions
  - Rollback confirmation workflows

**Funcionalidades Técnicas:**
- Activity logging service
- Audit log storage y indexing
- Real-time activity feed
- Report generation

---

### HU10: User Experience y Accesibilidad

**Como:** Usuario final
**Quiero:** Interfaz intuitiva, accesible y responsive
**Para:** Usar la aplicación eficientemente en diferentes dispositivos

#### Criterios de Aceptación:
- [ ] **Responsive Design**
  - Mobile-first approach
  - Tablet optimization
  - Desktop full-feature experience
  - Touch-friendly interfaces
- [ ] **Accessibility (a11y)**
  - WCAG 2.1 AA compliance
  - Keyboard navigation
  - Screen reader support
  - High contrast mode
  - Focus management
- [ ] **Performance**
  - Loading states elegantes
  - Skeleton screens
  - Optimized bundle size
  - Lazy loading de componentes
- [ ] **Help y Guidance**
  - Inline help tooltips
  - Contextual documentation
  - Tutorial flows para nuevos usuarios
  - Help modal con ejemplos

**Funcionalidades Técnicas:**
- Responsive breakpoints
- Accessibility testing
- Performance monitoring
- Help system integration
- Progressive Web App features

---

### HU11: Configuration y Settings

**Como:** Administrador
**Quiero:** Configurar comportamiento de la aplicación
**Para:** Personalizar la experiencia según necesidades del equipo

#### Criterios de Aceptación:
- [ ] **User Preferences**
  - Theme selection (light/dark)
  - Editor preferences (font size, syntax highlighting)
  - Dashboard layout customization
  - Notification preferences
- [ ] **System Settings**
  - Default values para new policy stores
  - Validation strictness levels
  - Timeout configurations
  - Feature flags
- [ ] **Integration Settings**
  - External service configurations
  - API endpoint configurations
  - Authentication settings
  - Security policies

**Funcionalidades Técnicas:**
- Settings management con localStorage
- Theme system con CSS variables
- Feature flags implementation
- Configuration validation

---

### HU12: Error Handling y Feedback

**Como:** Usuario
**Quiero:** Manejo elegante de errores y feedback claro
**Para:** Entender qué salió mal y cómo solucionarlo

#### Criterios de Aceptación:
- [ ] **Error Boundaries**
  - Graceful error handling
  - Error reporting y logging
  - Recovery mechanisms
  - User-friendly error messages
- [ ] **Validation Feedback**
  - Real-time validation
  - Inline error messages
  - Field-level validation
  - Form submission feedback
- [ ] **Loading States**
  - Skeleton loading
  - Progress indicators
  - Optimistic updates
  - Retry mechanisms

**Funcionalidades Técnicas:**
- Error boundary implementation
- Validation library integration
- Toast notification system
- Progress tracking

---

### HU13: API Integration y Testing

**Como:** Desarrollador
**Quiero:** Herramientas para interactuar directamente con las APIs
**Para:** Debugging y testing avanzado

#### Criterios de Aceptación:
- [ ] **API Console**
  - Interactive API testing
  - Request/response viewing
  - Authentication testing
  - Performance monitoring
- [ ] **API Documentation**
  - Auto-generated API docs
  - Interactive examples
  - Code samples en múltiples lenguajes
  - API versioning information

**Funcionalidades Técnicas:**
- API client integration
- Swagger/OpenAPI integration
- Code generation tools

---

## 4. Plan de Implementación por Fases

### Fase 1: Mejoras Fundamentales (2-3 sprints)
1. **Dashboard Avanzado** - Métricas y monitoreo
2. **Search y Filtering** - Búsqueda global y filtros avanzados
3. **Error Handling** - Mejoras en feedback y validación
4. **Performance Optimization** - Loading states y responsive design

### Fase 2: Funcionalidades Avanzadas (3-4 sprints)
1. **Policy Wizard** - Form-based policy creation
2. **Identity Sources** - Cognito y OIDC integration
3. **Audit Trail** - Change logging y activity tracking
4. **Bulk Operations** - Operaciones en lote

### Fase 3: Experiencia de Usuario (2-3 sprints)
1. **Accessibility** - WCAG compliance
2. **Help System** - Inline guidance y documentation
3. **Settings** - User preferences y configuration
4. **Mobile Optimization** - Responsive design completo

### Fase 4: Herramientas Avanzadas (2-3 sprints)
1. **Authorization Playground** - Testing avanzado
2. **Schema Designer** - Visual entity management
3. **API Console** - Developer tools
4. **Export/Import** - Data management tools

---

## 5. Tecnologías y Dependencias Adicionales

### 5.1. UI/UX Libraries
- **Charts**: Chart.js, Recharts, o D3.js para métricas
- **Tables**: React Table con filtros avanzados
- **Drag & Drop**: React DnD o @dnd-kit
- **Notifications**: React Hot Toast o Sonner
- **Icons**: Heroicons o Lucide React

### 5.2. State Management
- **Global State**: Zustand o Jotai
- **Server State**: React Query (TanStack Query)
- **Forms**: React Hook Form con Zod validation
- **Real-time**: WebSocket client o Server-Sent Events

### 5.3. Development Tools
- **Testing**: Jest + React Testing Library
- **E2E**: Playwright (ya configurado)
- **Linting**: ESLint + Prettier
- **Type Safety**: TypeScript (ya configurado)

### 5.4. Backend Integration
- **gRPC**: @grpc/grpc-js (ya configurado)
- **HTTP Client**: Fetch API con interceptors
- **WebSocket**: Para updates en tiempo real
- **File Upload**: Para import/export functionality

---

## 6. Consideraciones de Arquitectura

### 6.1. Component Architecture
- **Atomic Design**: Atoms, Molecules, Organisms, Templates, Pages
- **Compound Components**: Para complex UI elements
- **Render Props/Hooks**: Para reusable logic
- **Error Boundaries**: En cada level de component hierarchy

### 6.2. Data Flow
- **Server State**: TanStack Query para API data
- **Client State**: Zustand para UI state
- **Form State**: React Hook Form para form management
- **Real-time State**: WebSocket para live updates

### 6.3. Performance Considerations
- **Code Splitting**: Route-based y component-based
- **Lazy Loading**: Para heavy components
- **Memoization**: React.memo y useMemo
- **Virtual Scrolling**: Para large lists

### 6.4. Security Considerations
- **Input Sanitization**: Para user inputs
- **XSS Prevention**: Proper escaping
- **CSRF Protection**: Para API calls
- **Content Security Policy**: Headers configuration

---

## 7. Testing Strategy

### 7.1. Unit Testing
- **Component Testing**: React Testing Library
- **Hook Testing**: Custom hooks validation
- **Utility Testing**: Helper functions
- **Integration Testing**: API integration tests

### 7.2. E2E Testing (Playwright)
- **Critical User Journeys**: Policy creation, editing, testing
- **Cross-browser Testing**: Chrome, Firefox, Safari
- **Mobile Testing**: Responsive behavior
- **Accessibility Testing**: Basic a11y validation

### 7.3. Performance Testing
- **Load Testing**: Con múltiples policy stores
- **Bundle Analysis**: Webpack bundle analyzer
- **Core Web Vitals**: LCP, FID, CLS monitoring
- **Memory Leaks**: Chrome DevTools profiling

---

## 8. Success Metrics

### 8.1. User Experience
- **Task Completion Rate**: >95% para tareas comunes
- **Error Rate**: <5% en operaciones críticas
- **Time to Complete**: <2 min para crear policy básica
- **User Satisfaction**: >4.5/5 en usability surveys

### 8.2. Technical Performance
- **Page Load Time**: <2 segundos para pages principales
- **API Response Time**: <500ms para requests críticos
- **Bundle Size**: <500KB para initial bundle
- **Test Coverage**: >90% para critical paths

### 8.3. Business Metrics
- **Feature Adoption**: >80% uso de features principales
- **Support Tickets**: <5% relacionadas con UI/UX
- **User Retention**: >90% monthly active users
- **Time to Value**: <5 min para primera policy exitosa

---

## 9. Conclusiones

Esta especificación proporciona un roadmap completo para implementar una interfaz web que rival con la funcionalidad y usabilidad de AWS Verified Permissions. La implementación por fases permite entregas incrementales de valor mientras se construye una base sólida para funcionalidades avanzadas.

**Prioridades Inmediatas:**
1. Completar funcionalidades básicas faltantes
2. Implementar search y filtering robusto
3. Mejorar error handling y feedback
4. Optimizar performance y UX

**Consideraciones a Largo Plazo:**
- Monitor de métricas de uso para guiar development
- Feedback continuo de usuarios para refinamiento
- Evolución con nuevas features de AWS AVP
- Scalability para enterprise deployments

---
**Documento creado:** 2025-10-29
**Responsable:** Equipo Hodei Verified Permissions
**Versión:** 1.0
