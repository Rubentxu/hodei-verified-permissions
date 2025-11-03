# Plan Detallado de Tests E2E - Policy Store (Fases 1-3.1)

## 📋 Resumen Ejecutivo

Este documento presenta un plan comprehensive y detallado para implementar tests end-to-end (E2E) para todas las funcionalidades de **Policy Store** implementadas en las Fases 1-3.1 del proyecto Hodei Verified Permissions.

### Estado Actual del Proyecto

**Fases Completadas:**
- ✅ **FASE 1**: Datos y Métricas Reales (CRUD básico, métricas reales)
- ✅ **FASE 2**: Auditoría y Trazabilidad (audit logs)
- ✅ **FASE 3**: Gestión Avanzada (tags, filtros, identidad)
- ✅ **FASE 3.1**: Versionado + Gestión Masiva (snapshots, rollback, batch operations)

**Infraestructura de Testing Existente:**
- ✅ Tests unitarios: 38 tests
- ✅ Tests de integración: Con Testcontainers (PostgreSQL, SurrealDB, SQLite)
- ✅ Tests e2e Rust: Tests de backend con contenedores
- ✅ Tests e2e Frontend: 47 tests con Playwright
- ✅ Makefile: Comandos completos de testing automatizado

---

## 🎯 Objetivos del Plan

### Objetivo Principal
Establecer una suite completa de tests E2E que valide todas las funcionalidades de Policy Store desde la perspectiva del usuario final, cubriendo el flujo completo desde la creación hasta el versionado.

### Objetivos Específicos

1. **Cobertura Completa**: 100% de las funcionalidades de Policy Store
2. **Testing Integral**: Backend (gRPC) + Frontend (UI/UX) + API REST
3. **Automatización**: Ejecución con un solo comando Makefile
4. **Performance**: Validación de métricas y latencia
5. **Confiabilidad**: Tests estables y reproducibles
6. **Documentación**: Casos de prueba documentados y ejecutables

---

## 📊 Funcionalidades por Fase

### FASE 1: Datos y Métricas Reales
- [x] **CRUD de Policy Stores**
  - Crear policy store (con/ sin descripción)
  - Obtener policy store por ID
  - Listar policy stores (con paginación)
  - Actualizar policy store
  - Eliminar policy store
- [x] **Gestión de Schema**
  - Definir schema Cedar JSON
  - Validar schema
  - Actualizar schema
- [x] **Gestión de Policies**
  - Crear policies (estáticas y con templates)
  - Obtener policies
  - Listar policies
  - Actualizar policies
  - Eliminar policies
- [x] **Métricas Reales**
  - Contadores de uso
  - Tiempo de respuesta
  - Decisiones de autorización

### FASE 2: Auditoría y Trazabilidad
- [x] **Audit Logs**
  - Registro de todas las operaciones CRUD
  - Metadata (timestamp, usuario, IP)
  - Trazabilidad completa
  - Consultas de auditoría

### FASE 3: Gestión Avanzada
- [x] **Tags y Categorización**
  - Asignar tags a policy stores
  - Filtrar por tags
  - Búsqueda avanzada
- [x] **Filtros y Búsqueda**
  - Filtros por fecha, estado, tags
  - Búsqueda de texto completo
  - Sorting y ordenamiento
- [x] **Gestión de Identidad**
  - Identity Sources (OIDC, Cognito)
  - Mapeo de claims
  - Configuración JWT

### FASE 3.1: Versionado + Gestión Masiva
- [x] **Snapshot Management**
  - Crear snapshot del policy store
  - Listar snapshots
  - Obtener snapshot detallado
  - Rollback a snapshot
  - Eliminar snapshot
- [x] **Batch Operations**
  - Batch create policies
  - Batch update policies
  - Batch delete policies
  - Manejo de errores

### FASE 3.1: Playground / Testing (Bonus)
- [x] **Test Authorization**
  - Testing sin persistencia
  - Validación en tiempo real
  - Debug mode
- [x] **Batch Testing**
  - Predefined test suites
  - Custom test scenarios
  - Export results (CSV)

---

## 🏗️ Estrategia de Testing E2E

### Capas de Testing

```
┌─────────────────────────────────────────────────────┐
│               FRONTEND (Next.js)                    │
│              Playwright E2E Tests                   │
│  - UI/UX Testing                                    │
│  - User Journeys                                    │
│  - BFF Integration                                  │
└─────────────────────────────────────────────────────┘
                        ↕
┌─────────────────────────────────────────────────────┐
│                 BFF (API REST)                      │
│              Integration Tests                      │
│  - REST API endpoints                               │
│  - gRPC-Bridge                                      │
│  - Error handling                                   │
└─────────────────────────────────────────────────────┘
                        ↕
┌─────────────────────────────────────────────────────┐
│              BACKEND (gRPC)                         │
│              E2E Tests                              │
│  - Service integration                              │
│  - Database operations                              │
│  - Full workflows                                   │
└─────────────────────────────────────────────────────┘
                        ↕
┌─────────────────────────────────────────────────────┐
│              DATABASE                               │
│              (Testcontainers)                       │
│  - PostgreSQL                                       │
│  - SurrealDB                                        │
│  - SQLite                                           │
└─────────────────────────────────────────────────────┘
```

### Tipos de Tests

1. **Functional E2E Tests**
   - Flujo completo CRUD
   - Validación de datos
   - Manejo de errores

2. **Performance E2E Tests**
   - Métricas de latencia
   - Throughput
   - Resource usage

3. **Integration E2E Tests**
   - gRPC + Database
   - UI + gRPC
   - Cross-service communication

4. **Regression E2E Tests**
   - Snapshot & rollback
   - Batch operations
   - Data consistency

---

## 📝 Plan Detallado de Implementación

### Fase 1: Configuración y Preparación

#### 1.1 Setup del Entorno de Testing

```bash
# Crear directorios
mkdir -p tests/e2e/fixtures
mkdir -p tests/e2e/helpers
mkdir -p tests/e2e/data
mkdir -p tests/e2e/reports
```

#### 1.2 Datos de Prueba (Fixtures)

**1. Policy Stores de Prueba**
```rust
// tests/e2e/fixtures/policy_stores.rs
pub struct PolicyStoreFixture {
    pub basic_store: CreatePolicyStoreRequest,
    pub store_with_description: CreatePolicyStoreRequest,
    pub store_for_snapshots: CreatePolicyStoreRequest,
    pub store_for_batch: CreatePolicyStoreRequest,
}

impl PolicyStoreFixture {
    pub fn new() -> Self {
        Self {
            basic_store: CreatePolicyStoreRequest {
                description: None,
            },
            store_with_description: CreatePolicyStoreRequest {
                description: Some("E2E Test Policy Store".to_string()),
            },
            store_for_snapshots: CreatePolicyStoreRequest {
                description: Some("Snapshot Test Store".to_string()),
            },
            store_for_batch: CreatePolicyStoreRequest {
                description: Some("Batch Operations Store".to_string()),
            },
        }
    }
}
```

**2. Schema de Prueba**
```json
// tests/e2e/data/test_schema.json
{
  "": {
    "entityTypes": {
      "User": {
        "shape": {
          "type": "Record",
          "attributes": {
            "department": { "type": "String" },
            "role": { "type": "String" }
          }
        }
      },
      "Document": {
        "shape": {
          "type": "Record",
          "attributes": {
            "owner": { "type": "String" },
            "classification": { "type": "String" }
          }
        }
      },
      "Action": {}
    },
    "actions": {
      "view": {},
      "edit": {},
      "delete": {},
      "approve": {}
    }
  }
}
```

**3. Policies de Prueba**
```rust
// tests/e2e/fixtures/policies.rs
pub struct PolicyFixtures {
    pub allow_user_view_own_docs: &'static str,
    pub allow_admin_all: &'static str,
    pub deny_delete_without_approval: &'static str,
    pub conditional_by_department: &'static str,
}

impl PolicyFixtures {
    pub fn new() -> Self {
        Self {
            allow_user_view_own_docs: r#"permit(principal, action == Action::"view", resource)
when { principal == resource.owner };"#,
            allow_admin_all: r#"permit(principal, action, resource)
when { principal.role == "admin" };"#,
            deny_delete_without_approval: r#"forbid(principal, action == Action::"delete", resource)
unless { "approved" in resource.attributes };"#,
            conditional_by_department: r#"permit(principal, action == Action::"edit", resource)
when { principal.department == resource.department };"#,
        }
    }
}
```

#### 1.3 Test Harness

```rust
// tests/e2e/helpers/test_harness.rs
pub struct PolicyStoreTestHarness {
    pub client: AuthorizationControlClient<InterceptedService<Channel, GrpcWebProxyLayer>>,
    pub data_plane: AuthorizationDataClient<Channel>,
    pub test_db: TestDatabase,
}

impl PolicyStoreTestHarness {
    pub async fn new() -> Self {
        let config = DatabaseConfig {
            provider: DatabaseProvider::Sqlite,
            url: ":memory:".to_string(),
            max_connections: 10,
        };
        
        let repository = create_repository(&config)
            .await
            .expect("Failed to create repository");
            
        let repository = Arc::new(repository);
        let control_service = AuthorizationControlService::new(repository.clone());
        let data_service = AuthorizationDataService::new(repository);
        
        let mut server = Server::builder();
        server.add_service(AuthorizationControlServer::new(control_service));
        server.add_service(AuthorizationDataServer::new(data_service));
        
        let addr = server.bind("127.0.0.1:0").await.unwrap();
        let addr_str = format!("http://{}", addr);
        
        let channel = Channel::from_shared(addr_str.clone())
            .unwrap()
            .connect()
            .await
            .unwrap();
            
        let client = AuthorizationControlClient::new(channel.clone());
        let data_plane = AuthorizationDataClient::new(channel);
        
        let test_db = TestDatabase::new(config).await;
        
        Self {
            client,
            data_plane,
            test_db,
        }
    }
    
    pub async fn cleanup(&self) {
        self.test_db.cleanup().await;
    }
}
```

### Fase 2: Tests de Backend (gRPC)

#### 2.1 Policy Store CRUD Tests

```rust
// tests/e2e/backend/test_policy_store_crud.rs
#[cfg(test)]
mod policy_store_crud_tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_create_and_get_policy_store() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Crear policy store
        let request = CreatePolicyStoreRequest {
            description: Some("Test Store".to_string()),
        };
        
        let response = harness.client
            .create_policy_store(request)
            .await
            .expect("Failed to create policy store")
            .into_inner();
            
        assert!(!response.policy_store_id.is_empty());
        assert_eq!(response.created_at.len(), 20); // RFC3339 format
        
        // Obtener policy store
        let get_request = GetPolicyStoreRequest {
            policy_store_id: response.policy_store_id.clone(),
        };
        
        let store = harness.client
            .get_policy_store(get_request)
            .await
            .expect("Failed to get policy store")
            .into_inner();
            
        assert_eq!(store.policy_store_id, response.policy_store_id);
        assert_eq!(store.description, Some("Test Store".to_string()));
    }
    
    #[test]
    async fn test_list_policy_stores_with_pagination() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Crear múltiples stores
        let mut store_ids = Vec::new();
        for i in 0..10 {
            let request = CreatePolicyStoreRequest {
                description: Some(format!("Store {}", i)),
            };
            
            let response = harness.client
                .create_policy_store(request)
                .await
                .unwrap()
                .into_inner();
                
            store_ids.push(response.policy_store_id);
        }
        
        // Probar paginación (limit = 3)
        let list_request = ListPolicyStoresRequest {
            max_results: Some(3),
            next_token: None,
        };
        
        let page1 = harness.client
            .list_policy_stores(list_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(page1.policy_stores.len(), 3);
        
        // Segunda página si hay next_token
        if let Some(next_token) = page1.next_token {
            let list_request = ListPolicyStoresRequest {
                max_results: Some(3),
                next_token: Some(next_token),
            };
            
            let page2 = harness.client
                .list_policy_stores(list_request)
                .await
                .unwrap()
                .into_inner();
                
            assert!(!page2.policy_stores.is_empty());
        }
        
        // Verificar total count
        let all_stores = harness.client
            .list_policy_stores(Default::default())
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(all_stores.policy_stores.len(), 10);
    }
    
    #[test]
    async fn test_update_policy_store() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Crear store
        let created = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Original".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        // Actualizar store
        let update_request = UpdatePolicyStoreRequest {
            policy_store_id: created.policy_store_id.clone(),
            description: Some("Updated".to_string()),
        };
        
        let updated = harness.client
            .update_policy_store(update_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(updated.description, Some("Updated".to_string()));
        assert_ne!(updated.updated_at, created.created_at);
        
        // Verificar que se guardó
        let retrieved = harness.client
            .get_policy_store(GetPolicyStoreRequest {
                policy_store_id: created.policy_store_id,
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(retrieved.description, Some("Updated".to_string()));
    }
    
    #[test]
    async fn test_delete_policy_store_cascade() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Crear store
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Cascade Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Crear schema
        let schema_request = PutSchemaRequest {
            policy_store_id: store_id.clone(),
            schema: TEST_SCHEMA_JSON.to_string(),
        };
        
        harness.client
            .put_schema(schema_request)
            .await
            .unwrap();
            
        // Crear policy
        let policy_request = CreatePolicyRequest {
            policy_store_id: store_id.clone(),
            policy_id: "test-policy".to_string(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(
                    StaticPolicy {
                        statement: TEST_POLICY.to_string(),
                    }
                )),
            }),
            description: Some("Test Policy".to_string()),
        };
        
        harness.client
            .create_policy(policy_request)
            .await
            .unwrap();
            
        // Verificar que existen
        let _schema = harness.client
            .get_schema(GetSchemaRequest {
                policy_store_id: store_id.clone(),
            })
            .await
            .unwrap();
            
        let _policies = harness.client
            .list_policies(ListPoliciesRequest {
                policy_store_id: store_id.clone(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(_policies.policies.len(), 1);
        
        // Eliminar store (debe cascade delete)
        let delete_request = DeletePolicyStoreRequest {
            policy_store_id: store_id.clone(),
        };
        
        harness.client
            .delete_policy_store(delete_request)
            .await
            .unwrap();
            
        // Verificar que el store no existe
        let result = harness.client
            .get_policy_store(GetPolicyStoreRequest {
                policy_store_id: store_id,
            })
            .await;
            
        assert!(result.is_err());
    }
    
    #[test]
    async fn test_audit_log_on_crud_operations() {
        // TODO: Implementar cuando esté listo el sistema de audit logs
        // - Verificar que cada CRUD operation genera un audit log
        // - Validar contenido del audit log (timestamp, operation, user, etc.)
        // - Probar filtros de audit log
    }
}
```

#### 2.2 Snapshot & Rollback Tests

```rust
// tests/e2e/backend/test_snapshots.rs
#[cfg(test)]
mod snapshot_tests {
    use super::*;
    
    #[test]
    async fn test_create_and_get_snapshot() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Crear store con contenido
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Snapshot Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Agregar schema
        let schema_request = PutSchemaRequest {
            policy_store_id: store_id.clone(),
            schema: TEST_SCHEMA_JSON.to_string(),
        };
        
        harness.client
            .put_schema(schema_request)
            .await
            .unwrap();
            
        // Agregar policies
        for i in 0..5 {
            let policy_request = CreatePolicyRequest {
                policy_store_id: store_id.clone(),
                policy_id: format!("policy-{}", i),
                definition: Some(PolicyDefinition {
                    policy_type: Some(policy_definition::PolicyType::Static(
                        StaticPolicy {
                            statement: TEST_POLICY.to_string(),
                        }
                    )),
                }),
                description: Some(format!("Policy {}", i)),
            };
            
            harness.client
                .create_policy(policy_request)
                .await
                .unwrap();
        }
        
        // Crear snapshot
        let snapshot_request = CreatePolicyStoreSnapshotRequest {
            policy_store_id: store_id.clone(),
            description: Some("Initial snapshot".to_string()),
        };
        
        let snapshot = harness.client
            .create_policy_store_snapshot(snapshot_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(snapshot.policy_store_id, store_id);
        assert_eq!(snapshot.description, "Initial snapshot");
        assert!(!snapshot.snapshot_id.is_empty());
        
        // Obtener snapshot
        let get_snapshot_request = GetPolicyStoreSnapshotRequest {
            policy_store_id: store_id.clone(),
            snapshot_id: snapshot.snapshot_id.clone(),
        };
        
        let retrieved_snapshot = harness.client
            .get_policy_store_snapshot(get_snapshot_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(retrieved_snapshot.snapshot_id, snapshot.snapshot_id);
        assert_eq!(retrieved_snapshot.policy_count, 5);
        assert!(retrieved_snapshot.has_schema);
        assert_eq!(retrieved_snapshot.policies.len(), 5);
    }
    
    #[test]
    async fn test_rollback_to_snapshot() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Setup inicial
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Rollback Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Agregar contenido inicial
        let schema_request = PutSchemaRequest {
            policy_store_id: store_id.clone(),
            schema: TEST_SCHEMA_JSON.to_string(),
        };
        
        harness.client
            .put_schema(schema_request)
            .await
            .unwrap();
            
        let policy_request = CreatePolicyRequest {
            policy_store_id: store_id.clone(),
            policy_id: "original-policy".to_string(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(
                    StaticPolicy {
                        statement: TEST_POLICY.to_string(),
                    }
                )),
            }),
            description: Some("Original".to_string()),
        };
        
        harness.client
            .create_policy(policy_request)
            .await
            .unwrap();
            
        // Crear snapshot
        let snapshot = harness.client
            .create_policy_store_snapshot(CreatePolicyStoreSnapshotRequest {
                policy_store_id: store_id.clone(),
                description: Some("Original state".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        // Modificar contenido
        let updated_policy = CreatePolicyRequest {
            policy_store_id: store_id.clone(),
            policy_id: "original-policy".to_string(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(
                    StaticPolicy {
                        statement: "permit(principal, action, resource);".to_string(),
                    }
                )),
            }),
            description: Some("Modified".to_string()),
        };
        
        harness.client
            .update_policy(UpdatePolicyRequest {
                policy_store_id: store_id.clone(),
                policy_id: "original-policy".to_string(),
                definition: Some(PolicyDefinition {
                    policy_type: Some(policy_definition::PolicyType::Static(
                        StaticPolicy {
                            statement: "permit(principal, action, resource);".to_string(),
                        }
                    )),
                }),
                description: Some("Modified".to_string()),
            })
            .await
            .unwrap();
            
        // Verificar modificación
        let policies_after_modify = harness.client
            .list_policies(ListPoliciesRequest {
                policy_store_id: store_id.clone(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(policies_after_modify.policies[0].description, Some("Modified".to_string()));
        
        // Rollback a snapshot
        let rollback_request = RollbackToSnapshotRequest {
            policy_store_id: store_id.clone(),
            snapshot_id: snapshot.snapshot_id,
            description: Some("Rollback to original".to_string()),
        };
        
        let rollback_result = harness.client
            .rollback_to_snapshot(rollback_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(rollback_result.policies_restored, 1);
        assert!(rollback_result.schema_restored);
        
        // Verificar rollback
        let policies_after_rollback = harness.client
            .list_policies(ListPoliciesRequest {
                policy_store_id: store_id,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(policies_after_rollback.policies[0].description, Some("Original".to_string()));
    }
}
```

#### 2.3 Batch Operations Tests

```rust
// tests/e2e/backend/test_batch_operations.rs
#[cfg(test)]
mod batch_operations_tests {
    use super::*;
    
    #[test]
    async fn test_batch_create_policies() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Crear store
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Batch Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Agregar schema
        let schema_request = PutSchemaRequest {
            policy_store_id: store_id.clone(),
            schema: TEST_SCHEMA_JSON.to_string(),
        };
        
        harness.client
            .put_schema(schema_request)
            .await
            .unwrap();
            
        // Batch create policies
        let batch_request = BatchCreatePoliciesRequest {
            policy_store_id: store_id.clone(),
            policies: vec![
                BatchPolicyItem {
                    policy_id: "policy-1".to_string(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(
                            StaticPolicy { statement: TEST_POLICY.to_string() }
                        )),
                    }),
                    description: Some("Policy 1".to_string()),
                },
                BatchPolicyItem {
                    policy_id: "policy-2".to_string(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(
                            StaticPolicy { statement: TEST_POLICY.to_string() }
                        )),
                    }),
                    description: Some("Policy 2".to_string()),
                },
                BatchPolicyItem {
                    policy_id: "policy-3".to_string(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(
                            StaticPolicy { statement: TEST_POLICY.to_string() }
                        )),
                    }),
                    description: Some("Policy 3".to_string()),
                },
            ],
        };
        
        let batch_response = harness.client
            .batch_create_policies(batch_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(batch_response.results.len(), 3);
        assert!(batch_response.errors.is_empty());
        
        for result in batch_response.results {
            assert!(!result.created_at.is_empty());
            assert!(result.error.is_none());
        }
        
        // Verificar que se crearon
        let policies = harness.client
            .list_policies(ListPoliciesRequest {
                policy_store_id: store_id,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(policies.policies.len(), 3);
    }
    
    #[test]
    async fn test_batch_create_with_errors() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Batch Error Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Batch create con un policy inválido
        let batch_request = BatchCreatePoliciesRequest {
            policy_store_id: store_id.clone(),
            policies: vec![
                BatchPolicyItem {
                    policy_id: "valid-policy".to_string(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(
                            StaticPolicy { statement: TEST_POLICY.to_string() }
                        )),
                    }),
                    description: Some("Valid".to_string()),
                },
                BatchPolicyItem {
                    policy_id: "invalid-policy".to_string(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(
                            StaticPolicy { statement: "invalid syntax @@##".to_string() }
                        )),
                    }),
                    description: Some("Invalid".to_string()),
                },
            ],
        };
        
        let batch_response = harness.client
            .batch_create_policies(batch_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(batch_response.results.len(), 2);
        assert_eq!(batch_response.errors.len(), 1);
        
        // Verificar que el primero succeeded y el segundo failed
        assert!(batch_response.results[0].error.is_none());
        assert!(batch_response.results[1].error.is_some());
        assert!(batch_response.errors[0].contains("Invalid policy syntax"));
        
        // Verificar que solo se creó el válido
        let policies = harness.client
            .list_policies(ListPoliciesRequest {
                policy_store_id: store_id,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(policies.policies.len(), 1);
        assert_eq!(policies.policies[0].policy_id, "valid-policy");
    }
    
    #[test]
    async fn test_batch_update_policies() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Batch Update Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Setup: crear policies
        let initial_policies = vec!["policy-1", "policy-2", "policy-3"];
        
        for policy_id in &initial_policies {
            let request = CreatePolicyRequest {
                policy_store_id: store_id.clone(),
                policy_id: policy_id.to_string(),
                definition: Some(PolicyDefinition {
                    policy_type: Some(policy_definition::PolicyType::Static(
                        StaticPolicy { statement: TEST_POLICY.to_string() }
                    )),
                }),
                description: Some("Initial".to_string()),
            };
            
            harness.client
                .create_policy(request)
                .await
                .unwrap();
        }
        
        // Batch update
        let batch_request = BatchUpdatePoliciesRequest {
            policy_store_id: store_id.clone(),
            policies: vec![
                BatchPolicyItem {
                    policy_id: "policy-1".to_string(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(
                            StaticPolicy { statement: "permit(principal, action, resource);".to_string() }
                        )),
                    }),
                    description: Some("Updated 1".to_string()),
                },
                BatchPolicyItem {
                    policy_id: "policy-2".to_string(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(
                            StaticPolicy { statement: "permit(principal, action, resource);".to_string() }
                        )),
                    }),
                    description: Some("Updated 2".to_string()),
                },
            ],
        };
        
        let batch_response = harness.client
            .batch_update_policies(batch_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(batch_response.results.len(), 2);
        assert!(batch_response.errors.is_empty());
        
        // Verificar updates
        for result in batch_response.results {
            assert!(!result.updated_at.is_empty());
            assert!(result.error.is_none());
        }
        
        // Verificar que policy-3 no se actualizó
        let policy_3 = harness.client
            .get_policy(GetPolicyRequest {
                policy_store_id: store_id.clone(),
                policy_id: "policy-3".to_string(),
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(policy_3.description, Some("Initial".to_string()));
    }
    
    #[test]
    async fn test_batch_delete_policies() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Batch Delete Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Setup: crear policies
        for i in 1..=5 {
            let request = CreatePolicyRequest {
                policy_store_id: store_id.clone(),
                policy_id: format!("policy-{}", i),
                definition: Some(PolicyDefinition {
                    policy_type: Some(policy_definition::PolicyType::Static(
                        StaticPolicy { statement: TEST_POLICY.to_string() }
                    )),
                }),
                description: Some(format!("Policy {}", i)),
            };
            
            harness.client
                .create_policy(request)
                .await
                .unwrap();
        }
        
        // Batch delete policies 1, 2, 3
        let batch_request = BatchDeletePoliciesRequest {
            policy_store_id: store_id.clone(),
            policy_ids: vec![
                "policy-1".to_string(),
                "policy-2".to_string(),
                "policy-3".to_string(),
            ],
        };
        
        let batch_response = harness.client
            .batch_delete_policies(batch_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(batch_response.results.len(), 3);
        assert!(batch_response.errors.is_empty());
        
        // Verificar que solo quedaron 4 y 5
        let remaining_policies = harness.client
            .list_policies(ListPoliciesRequest {
                policy_store_id: store_id,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(remaining_policies.policies.len(), 2);
        
        let remaining_ids: Vec<String> = remaining_policies
            .policies
            .into_iter()
            .map(|p| p.policy_id)
            .collect();
            
        assert!(remaining_ids.contains(&"policy-4".to_string()));
        assert!(remaining_ids.contains(&"policy-5".to_string()));
        assert!(!remaining_ids.contains(&"policy-1".to_string()));
    }
}
```

#### 2.4 Authorization Tests

```rust
// tests/e2e/backend/test_authorization.rs
#[cfg(test)]
mod authorization_tests {
    use super::*;
    
    #[test]
    async fn test_is_authorized_allow() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        // Setup
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Auth Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Agregar schema
        let schema_request = PutSchemaRequest {
            policy_store_id: store_id.clone(),
            schema: TEST_SCHEMA_JSON.to_string(),
        };
        
        harness.client
            .put_schema(schema_request)
            .await
            .unwrap();
            
        // Agregar policy
        let policy_request = CreatePolicyRequest {
            policy_store_id: store_id.clone(),
            policy_id: "allow-view".to_string(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(
                    StaticPolicy {
                        statement: r#"permit(principal == User::"alice", action == Action::"view", resource == Document::"doc123");"#
                            .to_string(),
                    }
                )),
            }),
            description: Some("Allow Alice to view doc123".to_string()),
        };
        
        harness.client
            .create_policy(policy_request)
            .await
            .unwrap();
            
        // Test authorization
        let auth_request = IsAuthorizedRequest {
            policy_store_id: store_id,
            principal: Some(EntityIdentifier {
                entity_type: "User".to_string(),
                entity_id: "alice".to_string(),
            }),
            action: Some(EntityIdentifier {
                entity_type: "Action".to_string(),
                entity_id: "view".to_string(),
            }),
            resource: Some(EntityIdentifier {
                entity_type: "Document".to_string(),
                entity_id: "doc123".to_string(),
            }),
            context: None,
            entities: vec![],
        };
        
        let response = harness.data_plane
            .is_authorized(auth_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(response.decision, Decision::Allow as i32);
        assert!(response.determining_policies.contains(&"allow-view".to_string()));
        assert!(response.errors.is_empty());
    }
    
    #[test]
    async fn test_is_authorized_deny() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Auth Deny Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Setup igual que el test anterior
        let schema_request = PutSchemaRequest {
            policy_store_id: store_id.clone(),
            schema: TEST_SCHEMA_JSON.to_string(),
        };
        
        harness.client
            .put_schema(schema_request)
            .await
            .unwrap();
            
        let policy_request = CreatePolicyRequest {
            policy_store_id: store_id.clone(),
            policy_id: "allow-view".to_string(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(
                    StaticPolicy {
                        statement: r#"permit(principal == User::"alice", action == Action::"view", resource == Document::"doc123");"#
                            .to_string(),
                    }
                )),
            }),
            description: None,
        };
        
        harness.client
            .create_policy(policy_request)
            .await
            .unwrap();
            
        // Test authorization con usuario diferente (bob, no alice)
        let auth_request = IsAuthorizedRequest {
            policy_store_id: store_id,
            principal: Some(EntityIdentifier {
                entity_type: "User".to_string(),
                entity_id: "bob".to_string(),
            }),
            action: Some(EntityIdentifier {
                entity_type: "Action".to_string(),
                entity_id: "view".to_string(),
            }),
            resource: Some(EntityIdentifier {
                entity_type: "Document".to_string(),
                entity_id: "doc123".to_string(),
            }),
            context: None,
            entities: vec![],
        };
        
        let response = harness.data_plane
            .is_authorized(auth_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(response.decision, Decision::Deny as i32);
        assert!(response.errors.is_empty());
    }
    
    #[test]
    async fn test_batch_is_authorized() {
        let harness = PolicyStoreTestHarness::new().await;
        defer { harness.cleanup(); }
        
        let store = harness.client
            .create_policy_store(CreatePolicyStoreRequest {
                description: Some("Batch Auth Test".to_string()),
            })
            .await
            .unwrap()
            .into_inner();
            
        let store_id = store.policy_store_id;
        
        // Setup
        let schema_request = PutSchemaRequest {
            policy_store_id: store_id.clone(),
            schema: TEST_SCHEMA_JSON.to_string(),
        };
        
        harness.client
            .put_schema(schema_request)
            .await
            .unwrap();
            
        let policy_request = CreatePolicyRequest {
            policy_store_id: store_id.clone(),
            policy_id: "allow-alice".to_string(),
            definition: Some(PolicyDefinition {
                policy_type: Some(policy_definition::PolicyType::Static(
                    StaticPolicy {
                        statement: r#"permit(principal == User::"alice", action, resource);"#.to_string(),
                    }
                )),
            }),
            description: None,
        };
        
        harness.client
            .create_policy(policy_request)
            .await
            .unwrap();
            
        // Batch authorize
        let batch_request = BatchIsAuthorizedRequest {
            policy_store_id: store_id,
            requests: vec![
                IsAuthorizedRequest {
                    policy_store_id: store_id.clone(),
                    principal: Some(EntityIdentifier {
                        entity_type: "User".to_string(),
                        entity_id: "alice".to_string(),
                    }),
                    action: Some(EntityIdentifier {
                        entity_type: "Action".to_string(),
                        entity_id: "view".to_string(),
                    }),
                    resource: Some(EntityIdentifier {
                        entity_type: "Document".to_string(),
                        entity_id: "doc1".to_string(),
                    }),
                    context: None,
                    entities: vec![],
                },
                IsAuthorizedRequest {
                    policy_store_id: store_id.clone(),
                    principal: Some(EntityIdentifier {
                        entity_type: "User".to_string(),
                        entity_id: "bob".to_string(),
                    }),
                    action: Some(EntityIdentifier {
                        entity_type: "Action".to_string(),
                        entity_id: "view".to_string(),
                    }),
                    resource: Some(EntityIdentifier {
                        entity_type: "Document".to_string(),
                        entity_id: "doc1".to_string(),
                    }),
                    context: None,
                    entities: vec![],
                },
            ],
        };
        
        let batch_response = harness.data_plane
            .batch_is_authorized(batch_request)
            .await
            .unwrap()
            .into_inner();
            
        assert_eq!(batch_response.responses.len(), 2);
        
        // Primera request (alice) debe ser ALLOW
        assert_eq!(batch_response.responses[0].decision, Decision::Allow as i32);
        
        // Segunda request (bob) debe ser DENY
        assert_eq!(batch_response.responses[1].decision, Decision::Deny as i32);
    }
}
```

### Fase 3: Tests de Frontend (Playwright)

#### 3.1 Policy Store Management UI Tests

```typescript
// tests/e2e/frontend/policy-store-management.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Policy Store Management - E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Login o setup de autenticación si es necesario
    // await page.click('[data-testid="login-button"]');
  });

  test('should create a new policy store via UI', async ({ page }) => {
    // Navegar a la página de policy stores
    await page.click('[data-testid="policy-stores-menu"]');
    await page.waitForURL('**/policy-stores');
    
    // Click en "Create New Store"
    await page.click('[data-testid="create-store-button"]');
    
    // Llenar el formulario
    await page.fill('[data-testid="store-description"]', 'E2E Test Policy Store');
    
    // Enviar
    await page.click('[data-testid="submit-store"]');
    
    // Verificar que se creó
    await expect(page.locator('[data-testid="store-card"]')).toContainText('E2E Test Policy Store');
    
    // Verificar que aparece en la lista
    const stores = await page.locator('[data-testid="store-list"]').count();
    expect(stores).toBeGreaterThan(0);
  });

  test('should display policy store details', async ({ page }) => {
    // Primero crear un store
    await page.click('[data-testid="policy-stores-menu"]');
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Details Test Store');
    await page.click('[data-testid="submit-store"]');
    
    // Hacer click en el store creado
    await page.click('[data-testid="store-card"]');
    
    // Verificar detalles
    await expect(page.locator('[data-testid="store-details"]')).toBeVisible();
    await expect(page.locator('[data-testid="store-id"]')).toBeVisible();
    await expect(page.locator('[data-testid="store-description"]')).toContainText('Details Test Store');
    await expect(page.locator('[data-testid="store-created-at"]')).toBeVisible();
  });

  test('should edit a policy store', async ({ page }) => {
    // Crear store
    await page.click('[data-testid="policy-stores-menu"]');
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Original Description');
    await page.click('[data-testid="submit-store"]');
    
    // Editar
    await page.click('[data-testid="edit-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Updated Description');
    await page.click('[data-testid="save-store"]');
    
    // Verificar actualización
    await expect(page.locator('[data-testid="store-description"]')).toContainText('Updated Description');
  });

  test('should delete a policy store with confirmation', async ({ page }) => {
    // Crear store
    await page.click('[data-testid="policy-stores-menu"]');
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'To Delete');
    await page.click('[data-testid="submit-store"]');
    
    // Intentar eliminar
    await page.click('[data-testid="delete-store-button"]');
    
    // Confirmar eliminación
    await expect(page.locator('[data-testid="confirm-delete-modal"]')).toBeVisible();
    await page.click('[data-testid="confirm-delete"]');
    
    // Verificar que se eliminó
    await expect(page.locator('[data-testid="store-card"]')).not.toContainText('To Delete');
  });

  test('should filter policy stores by tags', async ({ page }) => {
    // Crear stores con tags
    await page.click('[data-testid="policy-stores-menu"]');
    
    // Store 1 con tag "production"
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Production Store');
    await page.fill('[data-testid="store-tags"]', 'production,critical');
    await page.click('[data-testid="submit-store"]');
    
    // Store 2 con tag "test"
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Test Store');
    await page.fill('[data-testid="store-tags"]', 'testing');
    await page.click('[data-testid="submit-store"]');
    
    // Filtrar por tag "production"
    await page.click('[data-testid="tag-filter"]');
    await page.fill('[data-testid="tag-search"]', 'production');
    await page.keyboard.press('Enter');
    
    // Verificar filtro
    await expect(page.locator('[data-testid="store-card"]')).toHaveCount(1);
    await expect(page.locator('[data-testid="store-card"]')).toContainText('Production Store');
  });

  test('should search policy stores', async ({ page }) => {
    // Crear stores
    await page.click('[data-testid="policy-stores-menu"]');
    
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Search Me Store');
    await page.click('[data-testid="submit-store"]');
    
    // Buscar
    await page.fill('[data-testid="search-stores"]', 'Search Me');
    await page.keyboard.press('Enter');
    
    // Verificar resultado
    await expect(page.locator('[data-testid="store-card"]')).toContainText('Search Me Store');
  });
});
```

#### 3.2 Snapshot Management UI Tests

```typescript
// tests/e2e/frontend/snapshot-management.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Snapshot Management - UI Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Crear un store con contenido
    await page.click('[data-testid="policy-stores-menu"]');
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Snapshot Test Store');
    await page.click('[data-testid="submit-store"]');
    
    // Navegar al store
    await page.click('[data-testid="store-card"]');
  });

  test('should create a snapshot', async ({ page }) => {
    // Ir a la pestaña de snapshots
    await page.click('[data-testid="snapshots-tab"]');
    
    // Click en "Create Snapshot"
    await page.click('[data-testid="create-snapshot-button"]');
    
    // Llenar formulario
    await page.fill('[data-testid="snapshot-description"]', 'Initial snapshot');
    await page.click('[data-testid="submit-snapshot"]');
    
    // Verificar que se creó
    await expect(page.locator('[data-testid="snapshot-card"]')).toContainText('Initial snapshot');
    await expect(page.locator('[data-testid="snapshot-date"]')).toBeVisible();
  });

  test('should list all snapshots', async ({ page }) => {
    // Crear múltiples snapshots
    for (let i = 1; i <= 3; i++) {
      await page.click('[data-testid="create-snapshot-button"]');
      await page.fill('[data-testid="snapshot-description"]', `Snapshot ${i}`);
      await page.click('[data-testid="submit-snapshot"]');
      await page.waitForTimeout(500);
    }
    
    // Verificar lista
    const snapshots = await page.locator('[data-testid="snapshot-card"]').count();
    expect(snapshots).toBe(3);
  });

  test('should view snapshot details', async ({ page }) => {
    // Crear snapshot
    await page.click('[data-testid="create-snapshot-button"]');
    await page.fill('[data-testid="snapshot-description"]', 'Detailed Snapshot');
    await page.click('[data-testid="submit-snapshot"]');
    
    // Click en ver detalles
    await page.click('[data-testid="view-snapshot-details"]');
    
    // Verificar detalles
    await expect(page.locator('[data-testid="snapshot-modal"]')).toBeVisible();
    await expect(page.locator('[data-testid="snapshot-policies-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="snapshot-schema-included"]')).toBeVisible();
    await expect(page.locator('[data-testid="snapshot-policies-list"]')).toBeVisible();
  });

  test('should rollback to a snapshot', async ({ page }) => {
    // Crear snapshot inicial
    await page.click('[data-testid="create-snapshot-button"]');
    await page.fill('[data-testid="snapshot-description"]', 'Original State');
    await page.click('[data-testid="submit-snapshot"]');
    
    // Hacer cambios (crear un policy)
    await page.click('[data-testid="policies-tab"]');
    await page.click('[data-testid="create-policy-button"]');
    await page.fill('[data-testid="policy-content"]', 'permit(principal, action, resource);');
    await page.click('[data-testid="submit-policy"]');
    
    // Regresar a snapshots
    await page.click('[data-testid="snapshots-tab"]');
    
    // Rollback
    await page.click('[data-testid="rollback-snapshot-button"]');
    await expect(page.locator('[data-testid="confirm-rollback-modal"]')).toBeVisible();
    await page.click('[data-testid="confirm-rollback"]');
    
    // Verificar rollback
    await page.click('[data-testid="policies-tab"]');
    await expect(page.locator('[data-testid="policies-list"]')).toHaveCount(0);
    
    // Verificar mensaje de éxito
    await expect(page.locator('[data-testid="rollback-success-message"]')).toContainText('rollback successful');
  });

  test('should delete a snapshot', async ({ page }) => {
    // Crear snapshot
    await page.click('[data-testid="create-snapshot-button"]');
    await page.fill('[data-testid="snapshot-description"]', 'To Delete');
    await page.click('[data-testid="submit-snapshot"]');
    
    // Eliminar
    await page.click('[data-testid="delete-snapshot-button"]');
    await expect(page.locator('[data-testid="confirm-delete-modal"]')).toBeVisible();
    await page.click('[data-testid="confirm-delete"]');
    
    // Verificar que se eliminó
    await expect(page.locator('[data-testid="snapshot-card"]')).not.toContainText('To Delete');
  });
});
```

#### 3.3 Batch Operations UI Tests

```typescript
// tests/e2e/frontend/batch-operations.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Batch Operations - UI Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Setup: crear store con schema
    await page.click('[data-testid="policy-stores-menu"]');
    await page.click('[data-testid="create-store-button"]');
    await page.fill('[data-testid="store-description"]', 'Batch Test Store');
    await page.click('[data-testid="submit-store"]');
    
    await page.click('[data-testid="store-card"]');
    await page.click('[data-testid="schemas-tab"]');
    await page.fill('[data-testid="schema-content"]', JSON.stringify(TEST_SCHEMA));
    await page.click('[data-testid="save-schema"]');
  });

  test('should batch create policies', async ({ page }) => {
    await page.click('[data-testid="policies-tab"]');
    
    // Click en "Batch Operations"
    await page.click('[data-testid="batch-operations-button"]');
    await page.click('[data-testid="batch-create-option"]');
    
    // Llenar múltiples policies
    for (let i = 1; i <= 3; i++) {
      await page.click(`[data-testid="add-policy-${i}"]`);
      await page.fill(`[data-testid="policy-id-${i}"]', `policy-${i}`);
      await page.fill(`[data-testid="policy-content-${i}"]', TEST_POLICY);
      await page.fill(`[data-testid="policy-description-${i}"]', `Batch Policy ${i}`);
    }
    
    // Ejecutar batch create
    await page.click('[data-testid="execute-batch-create"]');
    
    // Verificar resultados
    await expect(page.locator('[data-testid="batch-results"]')).toBeVisible();
    await expect(page.locator('[data-testid="success-count"]')).toContainText('3');
    
    // Verificar que se crearon
    const policies = await page.locator('[data-testid="policy-item"]').count();
    expect(policies).toBe(3);
  });

  test('should show batch operation progress', async ({ page }) => {
    await page.click('[data-testid="policies-tab"]');
    await page.click('[data-testid="batch-operations-button"]');
    await page.click('[data-testid="batch-create-option"]');
    
    // Llenar policies
    for (let i = 1; i <= 5; i++) {
      await page.click(`[data-testid="add-policy-${i}"]`);
      await page.fill(`[data-testid="policy-id-${i}"]', `policy-${i}`);
      await page.fill(`[data-testid="policy-content-${i}"]', TEST_POLICY);
    }
    
    // Ejecutar y verificar progress
    await page.click('[data-testid="execute-batch-create"]');
    await expect(page.locator('[data-testid="batch-progress-bar"]')).toBeVisible();
    
    // Esperar a que termine
    await expect(page.locator('[data-testid="batch-complete"]')).toBeVisible({ timeout: 10000 });
  });

  test('should handle batch operation errors', async ({ page }) => {
    await page.click('[data-testid="policies-tab"]');
    await page.click('[data-testid="batch-operations-button"]');
    await page.click('[data-testid="batch-create-option"]');
    
    // Llenar con un policy inválido
    await page.click('[data-testid="add-policy-1"]');
    await page.fill('[data-testid="policy-id-1"]', 'invalid-policy');
    await page.fill('[data-testid="policy-content-1"]', 'invalid syntax @@##');
    
    // Ejecutar
    await page.click('[data-testid="execute-batch-create"]');
    
    // Verificar manejo de errores
    await expect(page.locator('[data-testid="batch-errors"]')).toBeVisible();
    await expect(page.locator('[data-testid="batch-errors"]')).toContainText('Invalid policy syntax');
    
    // Verificar que el count de errores es 1
    await expect(page.locator('[data-testid="error-count"]')).toContainText('1');
  });

  test('should export batch results to CSV', async ({ page }) => {
    // Crear policies
    await page.click('[data-testid="policies-tab"]');
    await page.click('[data-testid="batch-operations-button"]');
    await page.click('[data-testid="batch-create-option"]');
    
    await page.click('[data-testid="add-policy-1"]');
    await page.fill('[data-testid="policy-id-1"]', 'export-policy');
    await page.fill('[data-testid="policy-content-1"]', TEST_POLICY);
    
    await page.click('[data-testid="execute-batch-create"]');
    
    // Esperar a que termine
    await expect(page.locator('[data-testid="batch-complete"]')).toBeVisible();
    
    // Exportar
    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="export-csv-button"]');
    const download = await downloadPromise;
    
    // Verificar download
    expect(download.suggestedFilename()).toMatch(/\.csv$/);
  });
});
```

---

## 🤖 Automatización con Makefile

### Comandos de Testing Actualizados

```makefile
# =============================================================================
# E2E TEST TARGETS - POLICY STORE
# =============================================================================

# Ejecutar todos los tests E2E (backend + frontend)
test-e2e-policy-store: docker-up
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  E2E Tests - Policy Store (Fases 1-3.1)                   ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Waiting for services to be ready...$(NC)"
	sleep 10
	@echo "$(GREEN)Running backend E2E tests...$(NC)"
	cargo test --test e2e_policy_store -- --ignored --nocapture
	@echo "$(GREEN)✅ Backend E2E tests completed!$(NC)"
	@echo "$(GREEN)Running frontend E2E tests...$(NC)"
	cd web-nextjs && npm run test:e2e
	@echo "$(GREEN)✅ Frontend E2E tests completed!$(NC)"
	@echo "$(GREEN)✅ All Policy Store E2E tests completed!$(NC)"

# Tests E2E solo backend
test-e2e-backend:
	@echo "$(BLUE)Running Backend E2E Tests (gRPC)...$(NC)"
	cargo test --test e2e_policy_store -- --ignored --nocapture

# Tests E2E solo frontend
test-e2e-frontend:
	@echo "$(BLUE)Running Frontend E2E Tests (Playwright)...$(NC)"
	cd web-nextjs && npm run test:e2e

# Tests E2E con real server (release)
test-e2e-real-server: build-server
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  E2E Tests Against Real Server (Release)                   ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	./scripts/run-e2e-tests-real.sh release

# Tests E2E con real server (debug)
test-e2e-real-server-debug: build-server-debug
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  E2E Tests Against Real Server (Debug)                     ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	./scripts/run-e2e-tests-real.sh debug

# Tests de performance E2E
test-e2e-performance:
	@echo "$(BLUE)Running Performance E2E Tests...$(NC)"
	@echo "$(YELLOW)Note: Performance tests require --release build$(NC)"
	cargo test --test e2e_policy_store performance -- --ignored --nocapture --release

# Tests de snapshot & rollback
test-e2e-snapshots: docker-up
	@echo "$(BLUE)Running Snapshot & Rollback E2E Tests...$(NC)"
	sleep 10
	cargo test --test e2e_policy_store snapshot -- --ignored --nocapture

# Tests de batch operations
test-e2e-batch: docker-up
	@echo "$(BLUE)Running Batch Operations E2E Tests...$(NC)"
	sleep 10
	cargo test --test e2e_policy_store batch -- --ignored --nocapture

# Tests de authorization
test-e2e-authorization: docker-up
	@echo "$(BLUE)Running Authorization E2E Tests...$(NC)"
	sleep 10
	cargo test --test e2e_policy_store authorization -- --ignored --nocapture

# Tests UI/UX completos
test-e2e-ui: docker-up
	@echo "$(BLUE)Running UI/UX E2E Tests...$(NC)"
	cd web-nextjs && npm run test:e2e:headed

# Tests con coverage
test-e2e-coverage:
	@echo "$(BLUE)Running E2E Tests with Coverage...$(NC)"
	cargo test --test e2e_policy_store -- --ignored --nocapture --cov
	cd web-nextjs && npm run test:e2e:coverage

# Tests de regresión
test-e2e-regression:
	@echo "$(BLUE)Running Regression E2E Tests...$(NC)"
	@echo "$(YELLOW)Running full test suite to detect regressions...$(NC)"
	$(MAKE) test-e2e-policy-store

# Tests de integración continua (CI)
test-e2e-ci: build docker-up
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  E2E Tests for CI/CD Pipeline                              ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	sleep 15
	cargo test --test e2e_policy_store -- --ignored --nocapture --quiet
	cd web-nextjs && CI=true npm run test:e2e
	@echo "$(GREEN)✅ CI E2E tests completed successfully!$(NC)"

# Generar reporte de tests
test-e2e-report: docker-up
	@echo "$(BLUE)Generating E2E Test Report...$(NC)"
	mkdir -p reports/e2e
	cargo test --test e2e_policy_store -- --ignored --nocapture --report=json > reports/e2e/backend-results.json
	cd web-nextjs && npm run test:e2e --reporter=json > ../../reports/e2e/frontend-results.json
	@echo "$(GREEN)Reports generated in reports/e2e/$(NC)"

# Test de smoke (pruebas rápidas)
test-e2e-smoke:
	@echo "$(BLUE)Running Smoke E2E Tests (quick validation)...$(NC)"
	cargo test --test e2e_policy_store smoke -- --ignored --nocapture
	cd web-nextjs && npm run test:e2e --grep "smoke"

# Verificar que el entorno esté listo para E2E
test-e2e-env-check:
	@echo "$(BLUE)Checking E2E Test Environment...$(NC)"
	@echo "$(YELLOW)Checking Docker...$(NC)"
	@docker --version || (echo "Docker not found!" && exit 1)
	@echo "$(GREEN)✅ Docker is installed$(NC)"
	@echo "$(YELLOW)Checking Rust...$(NC)"
	@rustc --version
	@echo "$(GREEN)✅ Rust is installed$(NC)"
	@echo "$(YELLOW)Checking Node.js...$(NC)"
	@node --version
	@echo "$(GREEN)✅ Node.js is installed$(NC)"
	@echo "$(YELLOW)Checking npm...$(NC)"
	@npm --version
	@echo "$(GREEN)✅ npm is installed$(NC)"
	@echo "$(YELLOW)Checking Playwright...$(NC)"
	@cd web-nextjs && npx playwright --version
	@echo "$(GREEN)✅ Playwright is installed$(NC)"
	@echo "$(GREEN)✅ Environment is ready for E2E tests!$(NC)"
```

### Scripts de Apoyo

```bash
#!/bin/bash
# scripts/run-e2e-tests-real.sh

#!/bin/bash
# scripts/run-e2e-tests-real.sh
set -e

MODE=${1:-debug}

echo "Starting Hodei Verified Permissions server ($MODE mode)..."

if [ "$MODE" = "release" ]; then
    $(MAKE) build-server
    SERVER_BINARY="./verified-permissions/target/release/hodei-verified-permissions"
else
    $(MAKE) build-server-debug
    SERVER_BINARY="./verified-permissions/target/debug/hodei-verified-permissions"
fi

# Kill any existing server
pkill -9 -f "hodei-verified-permissions" 2>/dev/null || true
sleep 2

# Start server
mkdir -p /home/rubentxu/hodei-data
DATABASE_URL="sqlite:////home/rubentxu/hodei-data/hodei.db" \
    $SERVER_BINARY > /tmp/rust-server.log 2>&1 &

SERVER_PID=$!
echo "Server started with PID: $SERVER_PID"

# Wait for server to be ready
echo "Waiting for server to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:50051/health > /dev/null 2>&1; then
        echo "Server is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Server failed to start within 30 seconds"
        cat /tmp/rust-server.log
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
    sleep 1
done

# Run E2E tests
echo "Running E2E tests..."
cargo test --test e2e_policy_store -- --ignored --nocapture

# Cleanup
kill $SERVER_PID 2>/dev/null || true
echo "E2E tests completed!"
```

---

## 📊 Métricas de Éxito

### 1. Métricas Cuantitativas

| Métrica | Objetivo | Actual | Estado |
|---------|----------|--------|--------|
| **Cobertura de Código** | ≥ 85% | - | ⏳ |
| **Tests E2E Pasados** | 100% | - | ⏳ |
| **Tiempo de Ejecución** | < 5 min | - | ⏳ |
| **Tests Frontend** | 47+ | 47 | ✅ |
| **Tests Backend** | 50+ | - | ⏳ |
| **Cobertura UI** | 100% | - | ⏳ |
| **Cobertura API** | 100% | - | ⏳ |

### 2. Métricas de Performance

| Operación | Latencia Objetivo | Latencia Actual |
|-----------|------------------|-----------------|
| Crear Policy Store | < 500ms | - |
| Listar Policy Stores | < 300ms | - |
| Snapshot Creation | < 1s | - |
| Rollback to Snapshot | < 2s | - |
| Batch Create (10 policies) | < 2s | - |
| Authorization Decision | < 100ms | - |

### 3. Criterios de Aceptación

**Backend (gRPC):**
- [ ] CRUD de Policy Stores funcionando al 100%
- [ ] Snapshots y Rollback operativos
- [ ] Batch operations sin errores
- [ ] Authorization decisiones correctas
- [ ] Audit logs registrados

**Frontend (UI):**
- [ ] Crear/Editar/Eliminar stores desde UI
- [ ] Gestionar snapshots desde UI
- [ ] Ejecutar batch operations desde UI
- [ ] Playback de authorization tests
- [ ] Export de resultados

**Integración:**
- [ ] Frontend ↔ Backend comunicación correcta
- [ ] Error handling consistente
- [ ] Performance bajo carga
- [ ] Datos consistentes

---

## 🛠️ Implementación Step-by-Step

### Paso 1: Preparación del Entorno (1 día)

```bash
# 1. Verificar dependencias
make test-e2e-env-check

# 2. Crear estructura de directorios
mkdir -p tests/e2e/{backend,frontend,fixtures,data,reports}

# 3. Configurar Testcontainers
# Ya está configurado en docker-compose.test.yml

# 4. Instalar Playwright browsers
cd web-nextjs
npm run playwright:install
npm run playwright:install-deps
```

### Paso 2: Implementar Test Harness (2 días)

- [ ] Crear `PolicyStoreTestHarness` en Rust
- [ ] Implementar fixtures para datos de prueba
- [ ] Configurar database test setup/teardown
- [ ] Implementar helpers para limpieza

### Paso 3: Tests de Backend (3 días)

- [ ] Policy Store CRUD tests (1 día)
- [ ] Snapshot & Rollback tests (1 día)
- [ ] Batch Operations tests (1 día)
- [ ] Authorization tests (1 día)
- [ ] Performance tests (0.5 días)
- [ ] Error handling tests (0.5 días)

### Paso 4: Tests de Frontend (3 días)

- [ ] Policy Store UI tests (1 día)
- [ ] Snapshot Management UI tests (1 día)
- [ ] Batch Operations UI tests (1 día)
- [ ] Playground integration tests (0.5 días)
- [ ] Error handling UI tests (0.5 días)

### Paso 5: Integración y Automatización (1 día)

- [ ] Integrar tests con Makefile
- [ ] Configurar CI/CD pipeline
- [ ] Generar reportes de cobertura
- [ ] Documentar casos de prueba

### Paso 6: Validación Final (1 día)

- [ ] Ejecutar suite completa
- [ ] Verificar métricas
- [ ] Generar reporte final
- [ ] Documentar lecciones aprendidas

---

## 📁 Estructura de Archivos Propuesta

```
verified-permissions/
├── tests/
│   ├── e2e/
│   │   ├── backend/
│   │   │   ├── mod.rs
│   │   │   ├── test_policy_store_crud.rs
│   │   │   ├── test_snapshots.rs
│   │   │   ├── test_batch_operations.rs
│   │   │   ├── test_authorization.rs
│   │   │   └── test_performance.rs
│   │   ├── frontend/
│   │   │   ├── policy-store-management.spec.ts
│   │   │   ├── snapshot-management.spec.ts
│   │   │   ├── batch-operations.spec.ts
│   │   │   ├── playground.spec.ts
│   │   │   └── authorization.spec.ts
│   │   ├── fixtures/
│   │   │   ├── mod.rs
│   │   │   ├── policy_stores.rs
│   │   │   ├── policies.rs
│   │   │   └── schemas.rs
│   │   ├── helpers/
│   │   │   ├── mod.rs
│   │   │   ├── test_harness.rs
│   │   │   └── assert_helpers.rs
│   │   ├── data/
│   │   │   ├── test_schema.json
│   │   │   ├── test_policies.json
│   │   │   └── test_data.sql
│   │   └── reports/
│   │       ├── backend-results.json
│   │       └── frontend-results.json
│   └── integration/
│       └── container_integration_tests.rs (existing)
├── docs/
│   └── E2E_TEST_PLAN_POLICY_STORE.md (this document)
├── scripts/
│   ├── run-e2e-tests.sh
│   ├── run-e2e-tests-real.sh
│   └── generate-test-report.sh
└── Makefile (updated with new targets)
```

---

## 🧪 Casos de Prueba Detallados

### TC-001: Crear Policy Store

**Precondiciones:**
- Servidor gRPC ejecutándose
- Base de datos disponible

**Pasos:**
1. Enviar CreatePolicyStoreRequest con descripción
2. Verificar respuesta con policy_store_id y timestamp
3. Recuperar el store con GetPolicyStoreRequest
4. Verificar que los datos coinciden

**Resultados Esperados:**
- ✅ Store creado exitosamente
- ✅ ID generado correctamente (formato UUID v4)
- ✅ Timestamps en formato RFC3339
- ✅ Descripción almacenada correctamente

**Postcondiciones:**
- Store existe en base de datos
- No quedan recursos pendientes

---

### TC-025: Snapshot y Rollback

**Precondiciones:**
- Policy store con schema y políticas

**Pasos:**
1. Crear snapshot con descripción
2. Modificar políticas
3. Ejecutar rollback al snapshot
4. Verificar estado anterior restaurado

**Resultados Esperados:**
- ✅ Snapshot creado con metadata correcta
- ✅ Políticas restauradas (count igual al original)
- ✅ Schema restaurado si estaba incluido
- ✅ Timestamps actualizados correctamente

---

### TC-042: Batch Create con Errores

**Precondiciones:**
- Policy store con schema válido

**Pasos:**
1. Enviar BatchCreatePolicies con 3 políticas (2 válidas, 1 inválida)
2. Verificar respuesta parcial

**Resultados Esperados:**
- ✅ 2 políticas creadas exitosamente
- ✅ 1 política falló con error descriptivo
- ✅ Lista de errores incluye mensaje específico
- ✅ Store contiene solo las políticas válidas

---

## 📈 Reportes de Ejecución

### Template de Reporte

```markdown
# E2E Test Execution Report

**Fecha:** {timestamp}
**Entorno:** {environment}
**Versión:** {version}

## Resumen Ejecutivo

- **Total Tests:** {total}
- **Passed:** {passed} ✅
- **Failed:** {failed} ❌
- **Skipped:** {skipped} ⏭️
- **Success Rate:** {rate}%

## Detalles por Módulo

### Backend Tests (gRPC)

| Test Suite | Total | Passed | Failed | Duration |
|------------|-------|--------|--------|----------|
| Policy Store CRUD | 15 | 15 | 0 | 2.3s |
| Snapshots | 8 | 8 | 0 | 3.1s |
| Batch Operations | 12 | 12 | 0 | 4.2s |
| Authorization | 10 | 10 | 0 | 1.8s |
| **Total Backend** | **45** | **45** | **0** | **11.4s** |

### Frontend Tests (Playwright)

| Test Suite | Total | Passed | Failed | Duration |
|------------|-------|--------|--------|----------|
| Policy Store UI | 12 | 12 | 0 | 45.2s |
| Snapshots UI | 8 | 8 | 0 | 38.7s |
| Batch UI | 10 | 10 | 0 | 52.3s |
| Playground | 17 | 17 | 0 | 61.4s |
| **Total Frontend** | **47** | **47** | **0** | **197.6s** |

## Cobertura de Código

- **Backend:** {backend_coverage}%
- **Frontend:** {frontend_coverage}%

## Performance Metrics

| Operación | Objetivo | Actual | Status |
|-----------|----------|--------|--------|
| Create Policy Store | < 500ms | 245ms | ✅ |
| List Policy Stores | < 300ms | 178ms | ✅ |
| Snapshot Create | < 1s | 623ms | ✅ |
| Rollback | < 2s | 1.2s | ✅ |
| Batch Create (10) | < 2s | 1.7s | ✅ |
| Authorization | < 100ms | 45ms | ✅ |

## Issues Encontrados

| ID | Descripción | Severidad | Status |
|----|-------------|-----------|--------|
| - | No issues found | - | - |

## Conclusiones

✅ Todos los tests E2E pasaron exitosamente
✅ Cobertura cumple objetivos (≥ 85%)
✅ Performance dentro de objetivos
✅ Sistema listo para producción

---
```

---

## 🚀 Ejecución

### Comandos Rápidos

```bash
# Verificar entorno
make test-e2e-env-check

# Ejecutar todos los tests E2E
make test-e2e-policy-store

# Solo backend
make test-e2e-backend

# Solo frontend
make test-e2e-frontend

# Tests de smoke (rápidos)
make test-e2e-smoke

# Tests con servidor real
make test-e2e-real-server

# Performance tests
make test-e2e-performance

# Generar reportes
make test-e2e-report

# CI pipeline
make test-e2e-ci
```

### Ejecución Individual

```bash
# Tests de snapshot
cargo test --test e2e_policy_store snapshot -- --ignored --nocapture

# Tests de batch
cargo test --test e2e_policy_store batch -- --ignored --nocapture

# Tests de authorization
cargo test --test e2e_policy_store authorization -- --ignored --nocapture

# Tests de performance
cargo test --test e2e_policy_store performance -- --ignored --nocapture --release

# Frontend tests
cd web-nextjs
npm run test:e2e
npm run test:e2e:ui
npm run test:e2e:debug
```

---

## 📚 Referencias

- **Proto File:** `/proto/authorization.proto`
- **Existing Tests:** `/main/tests/`
- **Playwright Config:** `/web-nextjs/playwright.config.ts`
- **Makefile:** `/Makefile`
- **Architecture:** Arquitectura Hexagonal (domain, application, infrastructure, api)

---

## ✅ Checklist de Validación

### Antes de Implementar

- [ ] Revisar este plan completo
- [ ] Verificar dependencias instaladas
- [ ] Validar configuración de Docker
- [ ] Configurar Testcontainers

### Durante la Implementación

- [ ] Seguir TDD (escribir test primero)
- [ ] Documentar cada caso de prueba
- [ ] Ejecutar tests después de cada módulo
- [ ] Mantener clean code

### Después de Implementar

- [ ] Ejecutar suite completa
- [ ] Verificar métricas de cobertura
- [ ] Generar reporte final
- [ ] Documentar lecciones aprendidas
- [ ] Actualizar README con instrucciones

---

## 📝 Notas de Implementación

1. **Idempotencia:** Todos los tests deben ser independientes y repetibles
2. **Clean-up:** Implementar limpieza automática después de cada test
3. **Datos de prueba:** Usar fixtures para datos consistentes
4. **Documentación:** Cada test debe tener documentación clara
5. **Performance:** Monitorear tiempo de ejecución de cada test
6. **Error handling:** Probar casos de error tanto en backend como frontend

---

**Fin del Plan de Tests E2E - Policy Store**

*Documento generado el: 2025-11-03*
*Versión: 1.0*
*Autor: Hodei Verified Permissions Team*
