# Plan de Refactorización a Arquitectura Hexagonal

## Estado Actual

El proyecto actual tiene una arquitectura en capas simple:
- `grpc/` - Servicios gRPC
- `storage/` - Repositorios y modelos
- `error.rs` - Errores

**Problema**: Acoplamiento entre capas, difícil de testear, dominio mezclado con infraestructura.

## Objetivo

Refactorizar a arquitectura hexagonal manteniendo funcionalidad actual pero con:
- Dominio puro sin dependencias
- Ports (interfaces) bien definidos
- Adapters intercambiables
- Alta testabilidad

## Estrategia de Migración

### Opción A: Big Bang (No recomendada)
- Reescribir todo de una vez
- Alto riesgo
- Proyecto no funcional durante migración

### Opción B: Strangler Fig Pattern (Recomendada)
- Crear nueva estructura en paralelo
- Migrar módulo por módulo
- Mantener funcionalidad en todo momento
- Eliminar código viejo gradualmente

## Plan de Ejecución (Strangler Pattern)

### Fase 1: Fundamentos (2-3 horas)
1. ✅ Crear documentación de arquitectura
2. ⏳ Crear estructura de carpetas
3. ⏳ Implementar domain layer básico
   - Value Objects (PolicyStoreId, PolicyId, etc.)
   - Entities (PolicyStore, Policy, Schema)
   - Domain errors

### Fase 2: Application Layer (3-4 horas)
4. ⏳ Definir ports (traits)
   - Driving ports (use case interfaces)
   - Driven ports (repository interfaces)
5. ⏳ Implementar use cases
   - CreatePolicyStoreUseCase
   - CreatePolicyUseCase
   - IsAuthorizedUseCase
   - etc.

### Fase 3: Infrastructure Adapters (4-5 horas)
6. ⏳ Refactorizar SQLite adapter
   - Implementar repository ports
   - Mappers domain ↔ DB
7. ⏳ Refactorizar gRPC adapter
   - Implementar como driving adapter
   - Mappers proto ↔ domain
8. ⏳ Implementar Cedar evaluator adapter

### Fase 4: Composición (1-2 horas)
9. ⏳ Actualizar main.rs
   - Dependency injection manual
   - Composición de adapters y use cases
10. ⏳ Eliminar código viejo

### Fase 5: Testing (2-3 horas)
11. ⏳ Tests unitarios por capa
12. ⏳ Tests de integración
13. ⏳ Tests end-to-end

**Total estimado: 12-17 horas**

## Alternativa Pragmática

Dado que el MVP ya funciona, una alternativa es:

### Opción C: Mejora Incremental
1. ✅ Completar HU 3.3 (SDK Cliente) con arquitectura actual
2. ✅ Documentar arquitectura hexagonal deseada
3. ⏳ Refactorizar en futuras iteraciones cuando:
   - Se necesite cambiar tecnologías
   - Se requiera mejor testabilidad
   - El equipo crezca

**Ventajas**:
- Entrega rápida de valor (SDK)
- Menor riesgo
- Refactorización justificada por necesidad real

**Desventajas**:
- Deuda técnica temporal
- Arquitectura no ideal

## Recomendación

Para este momento, recomiendo:

1. **Completar HU 3.3 (SDK Cliente)** - 1-2 horas
   - Valor inmediato para usuarios
   - Completa el MVP
   
2. **Crear estructura hexagonal base** - 2-3 horas
   - Domain layer con entidades principales
   - Ports definidos
   - Un use case de ejemplo refactorizado
   
3. **Documentar plan de migración completa** - 30 min
   - Para futuras iteraciones
   - Con ejemplos concretos

**Total: 3.5-5.5 horas** para tener:
- ✅ MVP completo (con SDK)
- ✅ Base hexagonal demostrada
- ✅ Camino claro para migración futura

## Decisión

¿Qué prefieres?

**A)** Refactorización completa ahora (12-17 horas)
**B)** Completar SDK + Base hexagonal (3.5-5.5 horas) ← **Recomendado**
**C)** Solo completar SDK (1-2 horas)

Por favor indica tu preferencia para continuar.
