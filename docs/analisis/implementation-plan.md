# Plan de Implementación: Servicio de Auditoría Separado

## 🎯 OBJETIVO
Separar el servicio de auditoría de verified-permissions en un servicio independiente (CloudTrail-like) usando el patrón Strategy, permitiendo múltiples modos de operación (standalone, external, hybrid).

---

## 📋 ROADMAP DE IMPLEMENTACIÓN

### **Fase 1: Refactorización del Código Actual (2-3 días)**

#### Día 1-2: Crear Abstracción AuditTrail
```rust
// 1. Crear verified-permissions/domain/src/audit.rs
#[async_trait]
pub trait AuditTrail: Send + Sync {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError>;
}

pub struct AuditEvent {
    pub event_id: String,
    pub event_type: String,
    pub correlation_id: String,
    pub source: String,
    pub user_id: String,
    pub resource_id: String,
    pub action: String,
    pub outcome: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

pub enum AuditError {
    PublishError(String),
    GrpcError(String),
}
```

#### Día 2-3: Implementar Estrategias
```rust
// 2. Crear verified-permissions/infrastructure/src/audit/
pub struct LocalAuditTrail {
    repository: Arc<RepositoryAdapter>,
}

pub struct ExternalAuditTrail {
    client: AuditControlServiceClient<tonic::transport::Channel>,
}

pub struct HybridAuditTrail {
    local: Arc<LocalAuditTrail>,
    external: Arc<ExternalAuditTrail>,
    buffer: Arc<Mutex<Vec<AuditEvent>>>,
}
```

#### Día 3: Actualizar main.rs
```rust
// 3. Modificar verified-permissions/main/src/main.rs
let audit_trail: Arc<dyn AuditTrail> = match mode.as_str() {
    "standalone" => Arc::new(LocalAuditTrail::new(repository.clone())),
    "external" => Arc::new(ExternalAuditTrail::new("http://localhost:50052").await?),
    "hybrid" => Arc::new(HybridAuditTrail::new(repository.clone(), "http://localhost:50052").await?),
    _ => Arc::new(LocalAuditTrail::new(repository.clone())),
};
```

---

### **Fase 2: Crear Servicio Audit Service (3-4 días)**

#### Día 4-5: Estructura Base
```
hodei-audit-service/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── types.rs
│   ├── database/
│   │   └── mod.rs
│   └── grpc/
│       ├── audit_control.rs
│       └── audit_query.rs
└── proto/
    └── audit.proto
```

#### Día 5-6: Implementar Database Layer
```rust
// Database schema
CREATE TABLE audit_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT UNIQUE NOT NULL,
    event_type TEXT NOT NULL,
    correlation_id TEXT,
    source TEXT NOT NULL,
    user_id TEXT,
    user_agent TEXT,
    client_ip TEXT,
    resource_id TEXT,
    resource_type TEXT,
    action TEXT NOT NULL,
    outcome TEXT NOT NULL,
    error_message TEXT,
    metadata TEXT, -- JSON
    timestamp DATETIME NOT NULL,
    version INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_events_timestamp ON audit_events(timestamp);
CREATE INDEX idx_audit_events_source ON audit_events(source);
CREATE INDEX idx_audit_events_user ON audit_events(user_id);
CREATE INDEX idx_audit_events_resource ON audit_events(resource_id);
```

#### Día 6-7: Implementar gRPC Services
```rust
// AuditControlService - Para publicar eventos
pub struct AuditControlService {
    database: Arc<Database>,
}

impl AuditControlService {
    async fn publish_event(&self, req: Request<PublishEventRequest>) -> Result<Response<PublishEventResponse>, Status> {
        // Persist event to database
        // Return event ID and confirmation
    }
}

// AuditQueryService - Para consultar eventos
pub struct AuditQueryService {
    database: Arc<Database>,
}

impl AuditQueryService {
    async fn get_events(&self, req: Request<GetEventsRequest>) -> Result<Response<GetEventsResponse>, Status> {
        // Query with filters
        // Return paginated results
    }
}
```

---

### **Fase 3: Migración y Testing (2-3 días)**

#### Día 8-9: Actualizar verified-permissions
1. **Remover** EventDispatcher del AuthorizationControlService
2. **Agregar** AuditTrail como dependencia
3. **Cambiar** publish_event() para usar AuditTrail
4. **Actualizar** get_audit_events() para llamar al servicio externo

#### Día 9-10: Testing
1. **Unit tests** para cada implementación de AuditTrail
2. **Integration tests** entre verified-permissions y audit-service
3. **Performance tests** con diferentes modos
4. **Failover tests** para modo híbrido

#### Día 10: Documentación
1. **Arquitectura** documentada
2. **Deployment** guide
3. **Configuration** examples
4. **Monitoring** setup

---

## 🏗️ CÓDIGO DE MIGRACIÓN

### **Paso 1: Refactorizar AuthorizationControlService**

**ANTES (Acoplado):**
```rust
pub struct AuthorizationControlService {
    repository: Arc<RepositoryAdapter>,
    dispatcher: Arc<EventDispatcher<InMemoryEventBus, EventStoreBox>>,
}

async fn get_policy_store(&self, request: Request<GetPolicyStoreRequest>) -> Result<Response<GetPolicyStoreResponse>, Status> {
    let store = self.repository.get_policy_store(&policy_store_id).await?;
    
    // Generar evento de auditoría
    let event = PolicyStoreAccessed {
        event_id: "evt_123".to_string(),
        // ... otros campos
    };
    self.publish_event(DomainEventEnvelope::PolicyStoreAccessed(Box::new(event))).await;
    
    Ok(Response::new(GetPolicyStoreResponse { ... }))
}
```

**DESPUÉS (Desacoplado):**
```rust
pub struct AuthorizationControlService {
    repository: Arc<RepositoryAdapter>,
    audit_trail: Arc<dyn AuditTrail>,
}

async fn get_policy_store(&self, request: Request<GetPolicyStoreRequest>) -> Result<Response<GetPolicyStoreResponse>, Status> {
    let store = self.repository.get_policy_store(&policy_store_id).await?;
    
    // Generar evento de auditoría via AuditTrail
    let event = AuditEvent {
        event_id: "evt_123".to_string(),
        event_type: "PolicyStoreAccessed".to_string(),
        source: "verified-permissions".to_string(),
        user_id: "system".to_string(),
        resource_id: store.id.to_string(),
        action: "READ".to_string(),
        outcome: "SUCCESS".to_string(),
        metadata: serde_json::json!({
            "operation": "GetPolicyStore"
        }),
        timestamp: Utc::now(),
    };
    
    if let Err(e) = self.audit_trail.log(event).await {
        error!("Failed to log audit event: {}", e);
    }
    
    Ok(Response::new(GetPolicyStoreResponse { ... }))
}
```

### **Paso 2: Configuración Dinámica**

**variables de entorno:**
```bash
# Modo de auditoría
export AUDIT_MODE=hybrid  # standalone | external | hybrid

# Para modo standalone
export AUDIT_LOCAL_BUFFER=1000

# Para modo external
export AUDIT_GRPC_ENDPOINT=http://localhost:50052
export AUDIT_TIMEOUT=10s
export AUDIT_RETRY_POLICY=exponential

# Para modo hybrid
export AUDIT_LOCAL_BUFFER=500
export AUDIT_EXTERNAL_BATCH=50
export AUDIT_FAIL_FAST=false
```

**Configuración YAML:**
```yaml
# config/audit.yaml
audit:
  mode: ${AUDIT_MODE}
  
  standalone:
    buffer_size: 1000
    write_mode: async
    
  external:
    grpc_endpoint: ${AUDIT_GRPC_ENDPOINT}
    timeout: ${AUDIT_TIMEOUT}
    retry_policy: exponential
    max_retries: 3
    
  hybrid:
    local_buffer: 500
    external_batch: 50
    fail_fast: false
    flush_interval: 5s
```

### **Paso 3: Docker Compose**

```yaml
version: '3.8'

services:
  # Servicio principal
  verified-permissions:
    build: ./verified-permissions
    environment:
      - DATABASE_URL=sqlite:///data/hodei.db
      - AUDIT_MODE=${AUDIT_MODE:-standalone}
      - AUDIT_GRPC_ENDPOINT=http://audit-service:50052
    volumes:
      - hodei-data:/data
    depends_on:
      - audit-service
    networks:
      - hodei

  # Servicio de auditoría
  audit-service:
    build: ./hodei-audit-service
    environment:
      - DATABASE_URL=sqlite:///data/audit.db
      - AUDIT_SERVER_ADDRESS=0.0.0.0:50052
    volumes:
      - audit-data:/data
    ports:
      - "50052:50052"
    networks:
      - hodei

  # Optional: NATS para modo async
  nats:
    image: nats:2.10
    ports:
      - "4222:4222"
    command: ["-js"]  # JetStream enabled
    networks:
      - hodei

volumes:
  hodei-data:
  audit-data:

networks:
  hodei:
    driver: bridge
```

---

## 📊 COMPARACIÓN: ANTES vs DESPUÉS

| Aspecto | ANTES (Monolítico) | DESPUÉS (Separado) |
|---------|-------------------|-------------------|
| **Acoplamiento** | Alto | Bajo |
| **Escalabilidad** | Limitada | Horizontal |
| **Flexibilidad** | Fija | Configurable |
| **Testing** | Difícil | Independiente |
| **Mantenimiento** | Complejo | Modular |
| **Despliegue** | Monolítico | Microservicios |
| **Observabilidad** | Mezclada | Dedicada |
| **Performance** | Sincrónico | Async/Batch |

---

## ⚠️ CONSIDERACIONES Y TRADE-OFFS

### **Ventajas**
✅ **Desacoplamiento**: Cada servicio puede evolucionar independently
✅ **Escalabilidad**: Audit service puede escalar horizontalmente
✅ **Flexibilidad**: Múltiples modos de operación
✅ **Observabilidad**: Métricas y logging dedicados
✅ **Tolerancia a fallos**: Modo híbrido con failover
✅ **Testing**: Testing independiente de cada componente

### **Trade-offs**
⚠️ **Complejidad**: Más componentes para gestionar
⚠️ **Latencia**: Modo external añade network hop
⚠️ **Consistencia**: Modo async puede perder eventos si crash
⚠️ **Operaciones**: Más puntos de fallo
⚠️ **Monitoring**: Necesita observabilidad más sofisticada

### **Estrategias de Mitigación**
- **Modo híbrido** para balancear consistencia y desacoplamiento
- **Circuit breakers** para manejar fallos del servicio externo
- **Dead letter queue** para eventos fallidos
- **Health checks** y alertas automáticas
- **Retry policies** con backoff exponencial

---

## 🎯 CRITERIOS DE ÉXITO

### **Métricas Técnicas**
- [ ] **Latencia**: <10ms overhead en modo standalone
- [ ] **Throughput**: >1000 events/second en modo batch
- [ ] **Availability**: 99.9% uptime del audit service
- [ ] **Data loss**: 0% en modo standalone/hybrid
- [ ] **Recovery**: <30s para failover en modo hybrid

### **Métricas de Negocio**
- [ ] **Flexibilidad**: Cambio de modo sin recompilar
- [ ] **Escalabilidad**: Audit service puede manejar 10x más eventos
- [ ] **Mantenibilidad**: Reducción 50% tiempo debugging
- [ ] **Observabilidad**: 100% eventos auditados visibles

---

## 🚀 PRÓXIMOS PASOS

1. **APROBAR** el plan de implementación
2. **ASIGNAR** recursos (2-3 developers)
3. **ESTABLECER** timeline (2-3 sprints)
4. **CONFIGURAR** ambiente de desarrollo
5. **INICIAR** con Fase 1 (Refactorización)

---

**¿Listo para comenzar la implementación?**
