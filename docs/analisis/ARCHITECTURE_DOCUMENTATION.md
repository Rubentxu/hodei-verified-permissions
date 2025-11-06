# Documentación de Arquitectura: Servicio de Auditoría Separado

## 📋 Tabla de Contenidos

1. [Resumen Ejecutivo](#resumen-ejecutivo)
2. [Estado Actual](#estado-actual)
3. [Arquitectura Propuesta](#arquitectura-propuesta)
4. [Especificaciones Técnicas](#especificaciones-técnicas)
5. [Patrón Strategy](#patrón-strategy)
6. [Plan de Migración](#plan-de-migración)
7. [Configuración y Despliegue](#configuración-y-despliegue)
8. [Métricas y Monitoreo](#métricas-y-monitoreo)
9. [Casos de Uso](#casos-de-uso)
10. [FAQ](#faq)

---

## 1. Resumen Ejecutivo

### 🎯 Objetivo

Separar el servicio de auditoría de `verified-permissions` en un microservicio independiente usando el **Patrón Strategy**, permitiendo múltiples modos de operación:

- **Standalone**: Persistencia directa a la base de datos local
- **External**: Publicación via gRPC a servicio externo
- **Hybrid**: Combinación de ambos con write-through cache

### 💡 Beneficios Clave

- ✅ **Desacoplamiento**: Reducción del 80% en acoplamiento entre servicios
- ✅ **Escalabilidad**: Capacidad de procesar 10x más eventos de auditoría
- ✅ **Flexibilidad**: Cambio de modo sin recompilación
- ✅ **Observabilidad**: Métricas y logging dedicados
- ✅ **Mantenibilidad**: Testing y evolución independiente

### 📊 Métricas de Éxito

- **Latencia**: <10ms overhead en modo standalone
- **Throughput**: >1000 events/second en modo batch
- **Disponibilidad**: 99.9% uptime del audit service
- **Pérdida de datos**: 0% en modo standalone/hybrid

---

## 2. Estado Actual

### 🔍 Análisis del Sistema Actual

#### Arquitectura Monolítica

```
┌─────────────────────────────────────────┐
│     verified-permissions (Monolito)     │
├─────────────────────────────────────────┤
│  AuthorizationControlService            │
│  ├── PolicyStore Operations             │
│  ├── Policy Operations                  │
│  └── ...                                │
├─────────────────────────────────────────┤
│  Event Publishing                       │
│  ├── EventDispatcher                    │
│  ├── InMemoryEventBus                   │
│  └── EventStoreBox                      │
├─────────────────────────────────────────┤
│  Database (SQLite)                      │
│  ├── Policy Stores                      │
│  ├── Policies                          │
│  └── Audit Logs                         │
└─────────────────────────────────────────┘
```

#### Problemas Identificados

1. **Alto Acoplamiento**
   - AuthorizationControlService conoce detalles de EventDispatcher
   - Mezcla de responsabilidades de negocio y auditoría
   - Difícil testing unitario independiente

2. **Limitaciones de Escalabilidad**
   - Todo el tráfico pasa por el monolito
   - Sin posibilidad de escalado horizontal de auditoría
   - Bloqueos en la base de datos afectan ambas funcionalidades

3. **Rigidez Operacional**
   - Sin opción de cambiar modo de persistencia
   - No hay fallback en caso de fallo de auditoría
   - Métricas mezcladas con lógica de negocio

4. **Mantenimiento Complejo**
   - Cambios en auditoría requieren rebuild del monolito
   - No se puede versionar independientemente
   - Testing de integración más complejo

### 📈 Código Afectado

**Archivos que requieren modificación:**

```bash
# Domain Layer
verified-permissions/domain/src/events.rs        # Event definitions
verified-permissions/domain/src/repository.rs     # AuditLogFilters

# API Layer
verified-permissions/api/src/grpc/control_plane.rs  # Event publishing
verified-permissions/api/src/grpc/data_plane.rs     # Event publishing (if any)

# Infrastructure Layer
verified-permissions/infrastructure/src/repository/adapter.rs  # get_audit_log
verified-perperties/infrastructure/src/events.rs               # EventStore implementation
verified-permissions/main/src/main.rs                          # Service initialization
```

---

## 3. Arquitectura Propuesta

### 🏗️ Nueva Arquitectura (Microservicios)

```
┌──────────────────────┐         ┌───────────────────────┐
│   verified-permissions│         │  hodei-audit-service  │
│     (Core Service)   │         │  (Audit Microservice) │
├──────────────────────┤         ├───────────────────────┤
│  AuthorizationControl│         │  AuditControlService  │
│  ├── PolicyStore     │         │  ├── PublishEvent     │
│  ├── Policy          │         │  ├── PublishEvents    │
│  └── ...             │         │  └── FlushEvents      │
├──────────────────────┤         ├───────────────────────┤
│  AuditTrail (Trait)  │    gRPC │  AuditQueryService    │
│  ├── Local           │◄──────► │  ├── GetEvents        │
│  ├── External        │         │  ├── GetByTimeRange   │
│  └── Hybrid          │         │  ├── GetByResource    │
├──────────────────────┤         │  └── GetByUser        │
│  Database (Policy)   │         ├───────────────────────┤
│  └── Policy Stores   │         │  Database (Audit)     │
└──────────────────────┘         │  └── Audit Events     │
         │                       └───────────────────────┘
         │                                │
         ▼                                ▼
┌─────────────────────────────────────────────────────┐
│              Shared Infrastructure                   │
│  ┌──────────────┐  ┌──────────────┐                │
│  │    NATS      │  │   Redis      │ (Optional)      │
│  │   (Queue)    │  │   (Cache)    │                │
│  └──────────────┘  └──────────────┘                │
└─────────────────────────────────────────────────────┘
```

### 🔌 Flujo de Datos

#### Modo Standalone (Local)
```
AuthorizationControlService
        ↓
   AuditTrail.log()
        ↓
   LocalAuditTrail
        ↓
   RepositoryAdapter
        ↓
   SQLite Database
        ↓
   AuditEvent Stored
```

#### Modo External (gRPC)
```
AuthorizationControlService
        ↓
   AuditTrail.log()
        ↓
   ExternalAuditTrail
        ↓
   gRPC Client
        ↓
   hodei-audit-service:50052
        ↓
   AuditControlService
        ↓
   Audit Database
        ↓
   AuditEvent Stored
```

#### Modo Hybrid (Write-through)
```
AuthorizationControlService
        ↓
   AuditTrail.log()
        ↓
   HybridAuditTrail
        ├─► LocalAuditTrail (Async)
        │        ↓
        │    Repository
        │        ↓
        │    Local Buffer
        │
        └─► ExternalAuditTrail (Async)
                 ↓
             gRPC
                 ↓
             Audit Service
                 ↓
             External DB
```

---

## 4. Especificaciones Técnicas

### 📦 Módulos de la Nueva Arquitectura

#### 4.1 AuditTrail Trait (Domain)

```rust
// verified-permissions/domain/src/audit.rs

#[async_trait]
pub trait AuditTrail: Send + Sync {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError>;
    
    async fn log_batch(&self, events: Vec<AuditEvent>) -> Result<(), AuditError> {
        for event in events {
            if let Err(e) = self.log(event).await {
                return Err(e);
            }
        }
        Ok(())
    }
    
    async fn flush(&self) -> Result<(), AuditError> {
        Ok(()) // No-op for most implementations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: String,
    pub correlation_id: String,
    pub source: String,
    pub user_id: String,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
    pub resource_id: String,
    pub resource_type: String,
    pub action: String,  // CREATE, READ, UPDATE, DELETE, LIST
    pub outcome: String, // SUCCESS, FAILURE, UNKNOWN
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub version: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Publish error: {0}")]
    PublishError(String),
    
    #[error("gRPC error: {0}")]
    GrpcError(String),
    
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
}
```

#### 4.2 Implementaciones Concretas

##### LocalAuditTrail
```rust
// verified-permissions/infrastructure/src/audit/local.rs

pub struct LocalAuditTrail {
    repository: Arc<RepositoryAdapter>,
    buffer: Arc<Mutex<Vec<AuditEvent>>>,
    batch_size: usize,
    flush_interval: Duration,
}

impl LocalAuditTrail {
    pub fn new(
        repository: Arc<RepositoryAdapter>,
        batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        Self {
            repository,
            buffer: Arc::new(Mutex::new(Vec::new())),
            batch_size,
            flush_interval,
        }
    }
}

#[async_trait]
impl AuditTrail for LocalAuditTrail {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.push(event);
        
        if buffer.len() >= self.batch_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    async fn flush(&self) -> Result<(), AuditError> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            return Ok(());
        }
        
        // Convert to domain events and store
        let events = buffer.drain(..).collect();
        self.store_events(events).await?;
        
        Ok(())
    }
}
```

##### ExternalAuditTrail
```rust
// verified-permissions/infrastructure/src/audit/external.rs

pub struct ExternalAuditTrail {
    client: AuditControlServiceClient<tonic::transport::Channel>,
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    timeout: Duration,
}

impl ExternalAuditTrail {
    pub async fn new(
        endpoint: String,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<Self, AuditError> {
        let channel = tonic::transport::Channel::from_shared(endpoint)
            .map_err(|e| AuditError::GrpcError(e.to_string()))?
            .connect()
            .await
            .map_err(|e| AuditError::GrpcError(e.to_string()))?;
            
        Ok(Self {
            client: AuditControlServiceClient::new(channel),
            circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::new(max_retries))),
            timeout,
        })
    }
}

#[async_trait]
impl AuditTrail for ExternalAuditTrail {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        // Check circuit breaker
        {
            let cb = self.circuit_breaker.lock().unwrap();
            if !cb.is_closed() {
                return Err(AuditError::CircuitBreakerOpen(
                    "Circuit breaker is open".to_string()
                ));
            }
        }
        
        let request = tonic::Request::new(PublishEventRequest {
            event: Some(event.into()),
        });
        
        let response = self.client
            .publish_event(request)
            .await
            .map_err(|e| AuditError::GrpcError(e.to_string()))?
            .into_inner();
            
        if response.persisted {
            Ok(())
        } else {
            Err(AuditError::PublishError(
                "Event not persisted".to_string()
            ))
        }
    }
}
```

##### HybridAuditTrail
```rust
// verified-permissions/infrastructure/src/audit/hybrid.rs

pub struct HybridAuditTrail {
    local: Arc<LocalAuditTrail>,
    external: Arc<ExternalAuditTrail>,
    background_task: Arc<AtomicBool>,
    flush_handle: Option<JoinHandle<()>>,
}

impl HybridAuditTrail {
    pub fn new(
        local: Arc<LocalAuditTrail>,
        external: Arc<ExternalAuditTrail>,
    ) -> Self {
        let background_task = Arc::new(AtomicBool::new(true));
        
        // Start background flush task
        let task_local = local.clone();
        let task_external = external.clone();
        let task_running = background_task.clone();
        
        let flush_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            while task_running.load(Ordering::SeqCst) {
                interval.tick().await;
                if let Err(e) = sync_with_external(&task_local, &task_external).await {
                    eprintln!("Sync error: {}", e);
                }
            }
        }));
        
        Self {
            local,
            external,
            background_task,
            flush_handle,
        }
    }
}

#[async_trait]
impl AuditTrail for HybridAuditTrail {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        // Write to local immediately (for consistency)
        self.local.log(event.clone()).await?;
        
        // Also send to external (fire and forget)
        let external = self.external.clone();
        tokio::spawn(async move {
            if let Err(e) = external.log(event).await {
                eprintln!("External audit log failed: {}", e);
            }
        });
        
        Ok(())
    }
}
```

#### 4.3 gRPC Service Definitions

```proto
// hodei-audit-service/proto/audit.proto

syntax = "proto3";

package hodei.audit.v1;

service AuditControlService {
    rpc PublishEvent(PublishEventRequest) returns (PublishEventResponse);
    rpc PublishEvents(PublishEventsRequest) returns (PublishEventsResponse);
    rpc FlushEvents(FlushEventsRequest) returns (FlushEventsResponse);
}

service AuditQueryService {
    rpc GetEvents(GetEventsRequest) returns (GetEventsResponse);
    rpc GetEventsByTimeRange(GetEventsByTimeRangeRequest) returns (GetEventsResponse);
    rpc GetEventsByResource(GetEventsByResourceRequest) returns (GetEventsResponse);
    rpc GetEventsByUser(GetEventsByUserRequest) returns (GetEventsResponse);
    rpc GetEventStatistics(GetEventStatisticsRequest) returns (GetEventStatisticsResponse);
}
```

---

## 5. Patrón Strategy

### 🎨 Aplicación del Patrón Strategy

El **Patrón Strategy** nos permite encapsular cada algoritmo de auditoría en clases separadas, haciendo que sean intercambiables en tiempo de ejecución.

#### Diagrama de Clases

```
┌─────────────────────┐
│ AuthorizationControl│
│      Service        │
├─────────────────────┤
│ - audit_trail: Arc< │
│   dyn AuditTrail    │
└──────────┬──────────┘
           │
           │ uses
           ▼
    ┌──────────────┐
    │  AuditTrail  │◄─────── Trait (Strategy)
    │   (Trait)    │
    └──────┬───────┘
           │ implements
    ┌──────┴────────────────────┐
    │                           │
    ▼                           ▼
┌──────────┐            ┌──────────────┐
│   Local  │            │   External   │
│  Audit   │            │   Audit      │
│  Trail   │            │   Trail      │
│          │            │              │
│ - repo   │            │ - client     │
│ - buffer │            │ - circuit    │
└──────────┘            └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   Hybrid     │
                    │   Audit      │
                    │   Trail      │
                    │              │
                    │ - local      │
                    │ - external   │
                    └──────────────┘
```

#### Configuración Dinámica

```rust
// main.rs

#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub mode: AuditMode,
    pub local: LocalConfig,
    pub external: ExternalConfig,
    pub hybrid: HybridConfig,
}

#[derive(Debug, Clone)]
pub enum AuditMode {
    Standalone,
    External,
    Hybrid,
}

impl AuditConfig {
    pub async fn create_audit_trail(&self) -> Result<Arc<dyn AuditTrail>, AuditError> {
        match self.mode {
            AuditMode::Standalone => {
                let local = LocalAuditTrail::new(
                    self.repository.clone(),
                    self.local.batch_size,
                    Duration::from_secs(self.local.flush_interval),
                );
                Ok(Arc::new(local))
            }
            AuditMode::External => {
                let external = ExternalAuditTrail::new(
                    self.external.endpoint.clone(),
                    Duration::from_secs(self.external.timeout),
                    self.external.max_retries,
                ).await?;
                Ok(Arc::new(external))
            }
            AuditMode::Hybrid => {
                let local = Arc::new(LocalAuditTrail::new(
                    self.repository.clone(),
                    self.hybrid.local_buffer,
                    Duration::from_secs(self.hybrid.flush_interval),
                ));
                let external = Arc::new(ExternalAuditTrail::new(
                    self.external.endpoint.clone(),
                    Duration::from_secs(self.external.timeout),
                    self.external.max_retries,
                ).await?);
                Ok(Arc::new(HybridAuditTrail::new(local, external)))
            }
        }
    }
}
```

---

## 6. Plan de Migración

### 📅 Timeline: 2-3 Sprints (6-8 semanas)

#### Sprint 1: Preparación y Refactorización (2 semanas)

**Semana 1:**
- [ ] **Día 1-2**: Análisis detallado del código actual
- [ ] **Día 3-4**: Crear trait AuditTrail en domain layer
- [ ] **Día 5**: Definir tipos AuditEvent y AuditError

**Semana 2:**
- [ ] **Día 1-2**: Implementar LocalAuditTrail
- [ ] **Día 3-4**: Implementar ExternalAuditTrail
- [ ] **Día 5**: Implementar HybridAuditTrail

**Entregables Sprint 1:**
- ✅ Trait AuditTrail implementado
- ✅ Tres implementaciones (Local, External, Hybrid)
- ✅ Tests unitarios para cada implementación
- ✅ Documentación técnica actualizada

#### Sprint 2: Servicio de Auditoría (2 semanas)

**Semana 3:**
- [ ] **Día 1-2**: Crear estructura hodei-audit-service
- [ ] **Día 3-4**: Implementar base de datos y schema
- [ ] **Día 5**: Implementar AuditControlService gRPC

**Semana 4:**
- [ ] **Día 1-2**: Implementar AuditQueryService gRPC
- [ ] **Día 3-4**: Implementar sistema de filtros y paginación
- [ ] **Día 5**: Testing de integración

**Entregables Sprint 2:**
- ✅ Servicio hodei-audit-service funcional
- ✅ gRPC APIs documentadas
- ✅ Base de datos optimizada
- ✅ Tests de integración

#### Sprint 3: Migración y Despliegue (2 semanas)

**Semana 5:**
- [ ] **Día 1-2**: Migrar AuthorizationControlService
- [ ] **Día 3-4**: Actualizar main.rs con configuración dinámica
- [ ] **Día 5**: Crear Docker Compose

**Semana 6:**
- [ ] **Día 1-2**: Testing E2E en todos los modos
- [ ] **Día 3-4**: Performance testing y optimización
- [ ] **Día 5**: Documentación final y guías

**Entregables Sprint 3:**
- ✅ Sistema completo migrado
- ✅ Modo standalone funcionando
- ✅ Modo external funcionando
- ✅ Modo hybrid funcionando
- ✅ Documentación completa
- ✅ Guías de despliegue

### 🛣️ Plan de Migración Detallado

#### Paso 1: Backup y Preparación

```bash
# Crear branch de migración
git checkout -b feature/audit-separation

# Backup de la base de datos actual
cp hodei.db hodei.db.backup.$(date +%Y%m%d)

# Documentar estado actual
cargo test --all > test-results-before.txt
```

#### Paso 2: Crear Nueva Estructura

```bash
# Crear módulos de auditoría
mkdir -p verified-permissions/domain/src/audit
mkdir -p verified-permissions/infrastructure/src/audit

# Crear servicio de auditoría
mkdir -p hodei-audit-service/{src,proto}
```

#### Paso 3: Implementar por Fases

**Fase A: Implementar Local (Sin romper)**
1. Implementar LocalAuditTrail
2. Mantener EventDispatcher como fallback
3. Testing exhaustivo

**Fase B: Integrar con Service (Progressive Rollout)**
1. Configurar feature flag `AUDIT_MODE=local`
2. Deploy en staging
3. Monitorear métricas
4. Gradual rollout

**Fase C: Crear Servicio Externo**
1. Implementar hodei-audit-service
2. Testing con ExternalAuditTrail
3. Deploy en staging

**Fase D: Modo Híbrido**
1. Implementar HybridAuditTrail
2. Testing de sincronización
3. Validar performance

**Fase E: Cutover**
1. Configurar producción
2. Monitoreo 24/7
3. Rollback plan preparado

---

## 7. Configuración y Despliegue

### 🔧 Variables de Entorno

#### Para verified-permissions

```bash
# Modo de auditoría (REQUIRED)
export AUDIT_MODE=hybrid  # standalone | external | hybrid

# Configuración base
export AUDIT_SOURCE=verified-permissions
export AUDIT_USER=system

# Modo Standalone
export AUDIT_LOCAL_BUFFER=1000
export AUDIT_LOCAL_FLUSH_INTERVAL=5

# Modo External
export AUDIT_GRPC_ENDPOINT=http://audit-service:50052
export AUDIT_TIMEOUT=10
export AUDIT_MAX_RETRIES=3
export AUDIT_CIRCUIT_BREAKER_THRESHOLD=5

# Modo Hybrid
export AUDIT_HYBRID_LOCAL_BUFFER=500
export AUDIT_HYBRID_EXTERNAL_BATCH=50
export AUDIT_HYBRID_FAIL_FAST=false
export AUDIT_HYBRID_SYNC_INTERVAL=5
```

#### Para hodei-audit-service

```bash
# Server
export AUDIT_SERVER_ADDRESS=0.0.0.0:50052

# Database
export AUDIT_DATABASE_URL=sqlite:///data/audit.db

# Audit Configuration
export AUDIT_BATCH_SIZE=100
export AUDIT_FLUSH_INTERVAL=5
export AUDIT_RETENTION_DAYS=90
export AUDIT_COMPRESSION=true

# Cache
export AUDIT_CACHE_MAX_SIZE=10000
export AUDIT_CACHE_TTL=3600

# Observability
export AUDIT_LOG_LEVEL=info
export AUDIT_METRICS_ENABLED=true
```

### 🐳 Docker Compose

```yaml
version: '3.8'

services:
  # ========================================================================
  # verified-permissions - Core Authorization Service
  # ========================================================================
  verified-permissions:
    build:
      context: ./verified-permissions
      dockerfile: Dockerfile
    container_name: verified-permissions
    environment:
      - DATABASE_URL=sqlite:///data/hodei.db
      - AUDIT_MODE=${AUDIT_MODE:-standalone}
      - AUDIT_SOURCE=verified-permissions
      - AUDIT_USER=system
      # Standalone mode
      - AUDIT_LOCAL_BUFFER=1000
      - AUDIT_LOCAL_FLUSH_INTERVAL=5
      # External mode
      - AUDIT_GRPC_ENDPOINT=http://audit-service:50052
      - AUDIT_TIMEOUT=10
      - AUDIT_MAX_RETRIES=3
      # Hybrid mode
      - AUDIT_HYBRID_LOCAL_BUFFER=500
      - AUDIT_HYBRID_EXTERNAL_BATCH=50
      - AUDIT_HYBRID_FAIL_FAST=false
    volumes:
      - hodei-data:/data
    ports:
      - "50051:50051"
      - "3000:3000"  # Next.js frontend
    depends_on:
      audit-service:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
    networks:
      - hodei-network

  # ========================================================================
  # hodei-audit-service - CloudTrail-like Audit Service
  # ========================================================================
  audit-service:
    build:
      context: ./hodei-audit-service
      dockerfile: Dockerfile
    container_name: hodei-audit-service
    environment:
      - AUDIT_SERVER_ADDRESS=0.0.0.0:50052
      - AUDIT_DATABASE_URL=sqlite:///data/audit.db
      - AUDIT_BATCH_SIZE=100
      - AUDIT_FLUSH_INTERVAL=5
      - AUDIT_RETENTION_DAYS=90
      - AUDIT_COMPRESSION=true
      - AUDIT_CACHE_MAX_SIZE=10000
      - AUDIT_CACHE_TTL=3600
      - AUDIT_LOG_LEVEL=info
      - AUDIT_METRICS_ENABLED=true
    volumes:
      - audit-data:/data
    ports:
      - "50052:50052"
    healthcheck:
      test: ["CMD", "grpc_health_probe", "-addr=:50052"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
    networks:
      - hodei-network

  # ========================================================================
  # NATS - Message Queue (Optional, for async mode)
  # ========================================================================
  nats:
    image: nats:2.10-alpine
    container_name: hodei-nats
    command: ["-js", "-m", "8222"]
    ports:
      - "4222:4222"  # Client connection
      - "8222:8222"  # HTTP monitoring
    volumes:
      - nats-data:/data
    restart: unless-stopped
    networks:
      - hodei-network

  # ========================================================================
  # Prometheus - Metrics Collection (Optional)
  # ========================================================================
  prometheus:
    image: prom/prometheus:latest
    container_name: hodei-prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    restart: unless-stopped
    networks:
      - hodei-network

  # ========================================================================
  # Grafana - Dashboards (Optional)
  # ========================================================================
  grafana:
    image: grafana/grafana:latest
    container_name: hodei-grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    ports:
      - "3001:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources
    restart: unless-stopped
    networks:
      - hodei-network

# ========================================================================
# Volumes
# ========================================================================
volumes:
  hodei-data:
    driver: local
  audit-data:
    driver: local
  nats-data:
    driver: local
  prometheus-data:
    driver: local
  grafana-data:
    driver: local

# ========================================================================
# Networks
# ========================================================================
networks:
  hodei-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

### 🐳 Dockerfile para verified-permissions

```dockerfile
# verified-permissions/Dockerfile

FROM rust:1.75-slim as builder

WORKDIR /app

# Copy dependency files
COPY verified-permissions/Cargo.toml verified-permissions/Cargo.lock ./
COPY verified-permissions/domain/Cargo.toml verified-permissions/domain/
COPY verified-permissions/infrastructure/Cargo.toml verified-permissions/infrastructure/
COPY verified-permissions/api/Cargo.toml verified-permissions/api/
COPY verified-permissions/main/Cargo.toml verified-permissions/main/
COPY verified-permissions/shared/Cargo.toml verified-permissions/shared/

# Build dependencies
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY verified-permissions ./

# Build application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false verified-permissions

# Copy binary from builder
COPY --from=builder /app/target/release/hodei-verified-permissions /usr/local/bin/
COPY --from=builder /app/target/release/hvp /usr/local/bin/

# Copy web app
COPY --from=builder /app/web-nextjs/dist ./web-nextjs/dist
COPY --from=builder /app/web-nextjs/public ./web-nextjs/public

# Create data directory
RUN mkdir -p /data && chown -R verified-permissions:verified-permissions /data

USER verified-permissions

EXPOSE 50051 3000

CMD ["hodei-verified-permissions"]
```

### 🐳 Dockerfile para audit-service

```dockerfile
# hodei-audit-service/Dockerfile

FROM rust:1.75-slim as builder

WORKDIR /app

# Copy dependency files
COPY hodei-audit-service/Cargo.toml hodei-audit-service/Cargo.lock ./
COPY verified-permissions/domain verified-permissions/domain/
COPY verified-permissions/infrastructure verified-permissions/infrastructure/

# Build dependencies
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY hodei-audit-service ./

# Build application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Install grpc_health_probe
RUN GRPC_HEALTH_PROBE_VERSION=v0.4.24 && \
    wget -qO/usr/local/bin/grpc_health_probe \
    https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
    chmod +x /usr/local/bin/grpc_health_probe

# Create non-root user
RUN useradd -r -s /bin/false audit-service

# Copy binary from builder
COPY --from=builder /app/target/release/hodei-audit-service /usr/local/bin/

# Create data directory
RUN mkdir -p /data && chown -R audit-service:audit-service /data

USER audit-service

EXPOSE 50052

CMD ["hodei-audit-service"]
```

---

## 8. Métricas y Monitoreo

### 📊 Métricas Clave

#### Latencia
```rust
// En cada implementación de AuditTrail
pub struct AuditMetrics {
    pub log_latency: Histogram,      // Tiempo de log()
    pub batch_latency: Histogram,    // Tiempo de log_batch()
    pub flush_latency: Histogram,    // Tiempo de flush()
}

impl AuditMetrics {
    pub async fn time_log<F, T>(&self, f: F) -> Result<T, AuditError>
    where
        F: Future<Output = Result<T, AuditError>>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();
        
        self.log_latency.observe(duration.as_secs_f64());
        
        match result {
            Ok(_) => self.log_success_total.inc(),
            Err(_) => self.log_failure_total.inc(),
        }
        
        result
    }
}
```

#### Throughput
```rust
pub struct AuditMetrics {
    pub events_logged_total: Counter,    // Total de eventos loggeados
    pub events_failed_total: Counter,    // Total de fallos
    pub events_per_second: Gauge,        // Rate de eventos/seg
    pub current_buffer_size: Gauge,      // Tamaño actual del buffer
}

impl AuditMetrics {
    pub fn record_event(&self, outcome: AuditOutcome) {
        self.events_logged_total.inc();
        
        match outcome {
            AuditOutcome::Success => {}
            AuditOutcome::Failure => self.events_failed_total.inc(),
        }
    }
}
```

#### Health Checks
```rust
// gRPC health check
impl AuditService {
    pub async fn health_check(&self) -> HealthStatus {
        let db_healthy = self.check_database().await.is_ok();
        let cache_healthy = self.check_cache().await.is_ok();
        
        match (db_healthy, cache_healthy) {
            (true, true) => HealthStatus::Healthy,
            (false, _) => HealthStatus::Unhealthy("Database down".to_string()),
            (_, false) => HealthStatus::Degraded("Cache down".to_string()),
        }
    }
}
```

### 📈 Grafana Dashboards

#### Dashboard 1: Audit Service Health
```
┌─────────────────────────────────────────────────────┐
│               Audit Service Overview                │
├─────────────────────────────────────────────────────┤
│  Status: 🟢 HEALTHY                                 │
│                                                     │
│  ┌──────────────┐  ┌──────────────┐                │
│  │   Events/s   │  │  Latency     │                │
│  │     125      │  │   5.2ms      │                │
│  └──────────────┘  └──────────────┘                │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │              Events by Type                 │   │
│  │  PolicyStoreCreated  ████████  40          │   │
│  │  PolicyStoreAccessed ██████    30          │   │
│  │  PolicyCreated      ████      20          │   │
│  │  Other              ██         10          │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

#### Dashboard 2: Mode Comparison
```
┌─────────────────────────────────────────────────────┐
│              Audit Mode Performance                 │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Mode: HYBRID (Primary)                             │
│  ┌──────────┬──────────┬──────────┬──────────┐     │
│  │   Mode   │   TPS    │  Latency │  Errors  │     │
│  ├──────────┼──────────┼──────────┼──────────┤     │
│  │ Standalone│   150   │   3.1ms  │    0     │     │
│  │ External │   120   │   8.5ms  │    0     │     │
│  │ Hybrid   │   135   │   5.2ms  │    0     │     │
│  └──────────┴──────────┴──────────┴──────────┘     │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │          Mode Distribution (24h)           │   │
│  │  Standalone  ████████████████████  50%     │   │
│  │  External   ██████████         25%         │   │
│  │  Hybrid     ████████████████    25%        │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### 🔔 Alertas

#### Alerta 1: Latencia Alta
```yaml
# alerting-rules.yml
- alert: AuditLatencyHigh
  expr: audit_log_latency_seconds{quantile="0.95"} > 0.01
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "High audit log latency"
    description: "95th percentile latency is {{ $value }}s"
```

#### Alerta 2: Pérdida de Eventos
```yaml
- alert: AuditEventsDropped
  expr: increase(audit_events_failed_total[5m]) > 10
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Audit events being dropped"
    description: "{{ $value }} events failed in last 5m"
```

#### Alerta 3: Servicio No Disponible
```yaml
- alert: AuditServiceDown
  expr: up{job="audit-service"} == 0
  for: 30s
  labels:
    severity: critical
  annotations:
    summary: "Audit service is down"
    description: "Service has been down for more than 30s"
```

---

## 9. Casos de Uso

### 🧩 Caso de Uso 1: Startup Inicial (Standalone)

**Contexto**: Nueva instalación, sin dependencias externas

**Pasos**:
1. Servicio inicia con `AUDIT_MODE=standalone`
2. LocalAuditTrail se inicializa con buffer vacío
3. Primer evento loggeado va al buffer local
4. Buffer se flusha automáticamente cuando está lleno

**Código**:
```rust
// main.rs
let audit_trail = match std::env::var("AUDIT_MODE").unwrap_or_default().as_str() {
    "standalone" | _ => {
        let local = LocalAuditTrail::new(
            repository.clone(),
            1000,  // batch_size
            Duration::from_secs(5),  // flush_interval
        );
        Arc::new(local)
    }
};
```

**Métricas**:
- Latencia: ~1-2ms (buffer en memoria)
- Throughput: Limitado por flush interval
- Disponibilidad: 100% (sin dependencias)

---

### 🌐 Caso de Uso 2: Producción con Externo

**Contexto**: Ambiente productivo, auditoría desacoplada

**Pasos**:
1. Servicio inicia con `AUDIT_MODE=external`
2. ExternalAuditTrail conecta a audit-service via gRPC
3. Evento se serializa y envía via red
4. audit-service persiste en su base de datos
5. Respuesta confirma persistencia

**Código**:
```rust
let external = ExternalAuditTrail::new(
    "http://audit-service:50052".to_string(),
    Duration::from_secs(10),  // timeout
    3,  // max_retries
).await?;

Arc::new(external)
```

**Métricas**:
- Latencia: ~5-15ms (network hop)
- Throughput: Limitado por gRPC batch
- Disponibilidad: 99.9% (con circuit breaker)

---

### ⚡ Caso de Uso 3: Modo Híbrido (Write-Through Cache)

**Contexto**: Balance entre consistencia y desacoplamiento

**Pasos**:
1. Evento llega a HybridAuditTrail
2. Se escribe inmediatamente en LocalAuditTrail (local buffer)
3. Se envía asíncronamente a ExternalAuditTrail
4. Tarea background sincroniza buffers periódicamente
5. Si external falla, local mantiene eventos

**Código**:
```rust
let local = Arc::new(LocalAuditTrail::new(repository.clone(), 500, Duration::from_secs(5)));
let external = Arc::new(ExternalAuditTrail::new("http://audit-service:50052".to_string()).await?);
let hybrid = HybridAuditTrail::new(local, external);

Arc::new(hybrid)
```

**Métricas**:
- Latencia: ~2-5ms (local write async)
- Throughput: Alto (async external)
- Disponibilidad: 100% (con fallback local)

---

### 🔄 Caso de Uso 4: Failover Automático

**Contexto**: External service temporalmente no disponible

**Pasos**:
1. ExternalAuditTrail intenta enviar evento
2. GRPC call falla (timeout o connection refused)
3. Circuit breaker se abre después de threshold
4. Events se write localmente como fallback
5. Service continúa funcionando sin degradación
6. Circuit breaker se cierra automáticamente después de cooldown
7. Events pendientes se sincronizan

**Código**:
```rust
// CircuitBreaker implementation
let mut breaker = CircuitBreaker::new(5, Duration::from_secs(30));
if !breaker.is_closed() {
    // Write to local fallback
    local.log(event).await?;
    return Ok(());
}

match external.log(event).await {
    Ok(_) => breaker.record_success(),
    Err(_) => {
        breaker.record_failure();
        // Fallback to local
        local.log(event).await?;
    }
}
```

---

### 📊 Caso de Uso 5: Batch Processing

**Contexto**: Alto volumen de eventos, optimizar throughput

**Pasos**:
1. Múltiples eventos llegan rápidamente
2. Se acumulan en buffer local (batch)
3. Cuando batch_size alcanzado, flush automático
4. Eventos se serializan en JSON
5. Se envían en una sola gRPC call
6. Respuesta confirma todos los eventos

**Código**:
```rust
let batch = vec![
    event1, event2, event3, // ... up to batch_size
];

let request = PublishEventsRequest {
    events: batch.into_iter().map(|e| e.into()).collect(),
};

let response = client.publish_events(request).await?;
assert_eq!(response.count, batch.len());
```

**Métricas**:
- Throughput: >1000 events/second
- Latencia: ~10-20ms per batch
- Eficiencia: Reducción 90% en network calls

---

## 10. FAQ

### ❓ Preguntas Frecuentes

#### P: ¿Por qué no usar Kafka/NATS directamente?
**R**: 
- **Simplicidad**: gRPC es más simple para RPC
- **Strong Typing**: Protocol buffers proporcionan tipos fuertes
- **Idiomatic Rust**: Mejor integración con async/await
- **Batching**: gRPC soporta batch requests nativamente
- **Puedes usar NATS** como transport layer debajo si quieres async

#### P: ¿Qué pasa si el audit service se cae en modo external?
**R**: 
- **Circuit Breaker** se abre después de 5 fallos
- Events se **write localmente** como fallback
- Service continúa sin degradación
- **Dead Letter Queue** para events no sincronizables
- **Retry logic** con backoff exponencial

#### P: ¿Cómo garantizar consistencia en modo hybrid?
**R**: 
- **Write local first** (inmediato)
- **Async external** (fire and forget)
- **Background sync** cada 5 segundos
- **Last-write-wins** para conflictos
- **Version checking** para evitar sobrescribir

#### P: ¿Puede el audit service escalar horizontalmente?
**R**: 
- **Sí**, usando load balancer
- **Stateless** design (database compartida)
- **Sharding** por time o source
- **Kubernetes** HPA para auto-scaling
- **Event partitioning** por tenant

#### P: ¿Qué database usar para production?
**R**: 
- **PostgreSQL** para production (ACID, performance)
- **SQLite** para development/testing
- **ClickHouse** para analytics (opcional)
- **TimescaleDB** para time-series (opcional)

#### P: ¿Cómo migrar datos existentes?
**R**: 
```sql
-- Migrar events existentes a nueva estructura
INSERT INTO audit_events (
    event_id, event_type, source, user_id, 
    resource_id, action, outcome, metadata, timestamp
)
SELECT 
    event_id, event_type, 'verified-permissions', 
    'system', aggregate_id, 'UNKNOWN', 'SUCCESS',
    event_data, occurred_at
FROM events_table
WHERE event_type LIKE '%Access%';
```

#### P: ¿Cómo hacer rollback si algo falla?
**R**: 
1. **Configurar feature flag** para rollback rápido
2. **Mantener EventDispatcher** como fallback durante transición
3. **Backup automático** antes de cada deployment
4. **Blue-green deployment** para zero downtime
5. **Monitoring** automático para detectar problemas

#### P: ¿Cómo manejar PII en audit logs?
**R**: 
```rust
// Sanitizar datos sensibles
fn sanitize_audit_event(event: &mut AuditEvent) {
    // Hash email addresses
    if let Some(metadata) = event.metadata.get_mut("email") {
        *metadata = Value::String(hash_email(metadata.as_str().unwrap()));
    }
    
    // Redact IP addresses in test environments
    if cfg!(test) {
        if let Some(ip) = event.client_ip.take() {
            event.metadata.insert("client_ip_redacted".to_string(), Value::String(ip));
        }
    }
}
```

#### P: ¿Cómo optimizar queries de auditoría?
**R**: 
```sql
-- Índices optimizados
CREATE INDEX CONCURRENTLY idx_audit_events_timestamp 
ON audit_events (timestamp DESC);

CREATE INDEX CONCURRENTLY idx_audit_events_source_timestamp 
ON audit_events (source, timestamp DESC);

CREATE INDEX CONCURRENTLY idx_audit_events_user_timestamp 
ON audit_events (user_id, timestamp DESC);

-- Partitioning por mes (PostgreSQL)
CREATE TABLE audit_events_y2024m01 PARTITION OF audit_events
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
```

#### P: ¿Cómo hacer compliance (GDPR, SOX)?
**R**: 
```rust
// Derecho al olvido (GDPR)
async fn delete_user_events(user_id: &str) -> Result<(), AuditError> {
    // Marcar events como deleted (no hard delete)
    sqlx::query!(
        "UPDATE audit_events SET deleted = true WHERE user_id = $1",
        user_id
    ).execute(&self.pool).await?;
    
    // O hard delete si requerido por compliance
    if self.config.gdpr_hard_delete {
        sqlx::query!(
            "DELETE FROM audit_events WHERE user_id = $1",
            user_id
        ).execute(&self.pool).await?;
    }
    
    Ok(())
}

// Retención automática (SOX)
async fn apply_retention_policy() -> Result<u32, AuditError> {
    let deleted = sqlx::query!(
        "DELETE FROM audit_events 
         WHERE timestamp < NOW() - INTERVAL '90 days'"
    ).execute(&self.pool).await?;
    
    Ok(deleted.rows_affected())
}
```

---

## 📖 Referencias

### Documentación Técnica
- [Patrón Strategy en Rust](https://refactoring.guru/design-patterns/strategy)
- [gRPC Rust Tutorial](https://grpc.io/docs/languages/rust/)
- [Async Trait RFC](https://rust-lang.github.io/rfcs/3185-async-trait.html)
- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)

### Herramientas
- [tonic](https://docs.rs/tonic/) - gRPC framework para Rust
- [sqlx](https://docs.rs/sqlx/) - Async SQLx library
- [async-trait](https://docs.rs/async-trait/) - Async traits para Rust
- [thiserror](https://docs.rs/thiserror/) - Ergonomic error types

### Artículos y Papers
- ["Microservices Patterns" by Chris Richardson](https://microservices.io/patterns/)
- ["Building Microservices" by Sam Newman](https://samnewman.io/books/building-microservices/)
- ["Designing Data-Intensive Applications" by Martin Kleppmann](https://dataintensive.net/)

---

**© 2024 Hodei Verified Permissions - Documentación de Arquitectura**
