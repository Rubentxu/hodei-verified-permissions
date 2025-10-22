# 📋 Reporte de Verificación de Historias de Usuario
## Hodei Verified Permissions - Verificación Sistemática

**Fecha:** 22 de Octubre de 2025  
**Documento Base:** `docs/historias-usuario.md`  
**Método:** Inspección de código + Ejecución de tests

---

## 🎯 ÉPICA 1: PLANO DE DATOS FUNDAMENTAL

---

### ✅ HU 1.1: Evaluar una solicitud de autorización simple

**Estado en Docs:** ✅ COMPLETADA  
**Estado Real:** ❌ **NO FUNCIONAL**

#### Criterios de Aceptación

| # | Criterio | Estado | Evidencia |
|---|----------|--------|-----------|
| 1 | Se define un método gRPC `IsAuthorized` | ✅ CUMPLE | `proto/authorization.proto:11` |
| 2 | El servicio puede cargar políticas Cedar desde almacenamiento | ❌ NO CUMPLE | No hay llamada a repository |
| 3 | El servicio utiliza `cedar-policy` para evaluar | ❌ NO CUMPLE | No usa Authorizer |
| 4 | La respuesta indica decisión y políticas determinantes | ⚠️ PARCIAL | Retorna estructura pero con datos dummy |

#### Código Actual

**Archivo:** `verified-permissions/api/src/grpc/data_plane.rs:90-106`

```rust
async fn is_authorized(
    &self,
    request: Request<IsAuthorizedRequest>,
) -> Result<Response<IsAuthorizedResponse>, Status> {
    let req = request.into_inner();
    info!(
        "Authorization request for policy store: {}",
        req.policy_store_id
    );

    // Dummy implementation - always allow
    Ok(Response::new(IsAuthorizedResponse {
        decision: Decision::Allow as i32,
        determining_policies: vec!["dummy-policy".to_string()],
        errors: vec![],
    }))
}
```

#### Problemas Identificados

1. **❌ Sin Repository**
   - `AuthorizationDataService` no tiene campo `repository`
   - No puede cargar políticas de base de datos

2. **❌ Sin Evaluación Cedar**
   - No construye `PolicySet`
   - No usa `cedar_policy::Authorizer`
   - No evalúa realmente las políticas

3. **❌ Siempre ALLOW**
   - Decisión hardcoded
   - Riesgo crítico de seguridad
   - No respeta políticas reales

4. **❌ Políticas Determinantes Fake**
   - Retorna "dummy-policy"
   - No refleja políticas reales que aplicaron

#### Test Requerido

```rust
#[tokio::test]
async fn test_hu_1_1_is_authorized_with_real_policy() {
    // Setup: Crear policy store con política DENY
    let repository = create_test_repository().await;
    let policy_store = repository.create_policy_store(None).await.unwrap();
    
    // Crear política que NIEGA acceso
    let policy = r#"
        forbid(
            principal == User::"alice",
            action == Action::"read",
            resource == Document::"secret"
        );
    "#;
    
    repository.create_policy(
        &policy_store.id,
        &PolicyId::new("deny-alice").unwrap(),
        CedarPolicy::new(policy.to_string()).unwrap(),
        None
    ).await.unwrap();
    
    // Test: Evaluar autorización
    let service = AuthorizationDataService::new(Arc::new(repository));
    let request = IsAuthorizedRequest {
        policy_store_id: policy_store.id.into_string(),
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "read".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "Document".to_string(),
            entity_id: "secret".to_string(),
        }),
        context: None,
        entities: vec![],
    };
    
    let response = service.is_authorized(Request::new(request)).await.unwrap();
    let result = response.into_inner();
    
    // Verificar: Debe DENEGAR
    assert_eq!(result.decision, Decision::Deny as i32);
    assert!(result.determining_policies.contains(&"deny-alice".to_string()));
    assert!(result.errors.is_empty());
}
```

#### Código Correcto Requerido

```rust
pub struct AuthorizationDataService {
    repository: Arc<RepositoryAdapter>,  // ✅ Añadir repository
}

impl AuthorizationDataService {
    pub fn new(repository: Arc<RepositoryAdapter>) -> Self {
        Self { repository }
    }
}

async fn is_authorized(
    &self,
    request: Request<IsAuthorizedRequest>,
) -> Result<Response<IsAuthorizedResponse>, Status> {
    let req = request.into_inner();
    
    // 1. Cargar políticas de BD
    let policies = self.repository
        .list_policies(&PolicyStoreId::new(req.policy_store_id.clone())?)
        .await
        .map_err(Status::from)?;
    
    if policies.is_empty() {
        return Ok(Response::new(IsAuthorizedResponse {
            decision: Decision::Deny as i32,
            determining_policies: vec![],
            errors: vec!["No policies found".to_string()],
        }));
    }
    
    // 2. Construir PolicySet de Cedar
    let mut policy_set_str = String::new();
    for policy in &policies {
        policy_set_str.push_str(&policy.statement.as_str());
        policy_set_str.push('\n');
    }
    
    let policy_set = PolicySet::from_str(&policy_set_str)
        .map_err(|e| Status::internal(format!("Invalid policy set: {}", e)))?;
    
    // 3. Construir entidades Cedar
    let principal = EntityUid::from_str(&format!(
        "{}::\"{}\"",
        req.principal.as_ref().unwrap().entity_type,
        req.principal.as_ref().unwrap().entity_id
    )).map_err(|e| Status::invalid_argument(format!("Invalid principal: {}", e)))?;
    
    let action = EntityUid::from_str(&format!(
        "{}::\"{}\"",
        req.action.as_ref().unwrap().entity_type,
        req.action.as_ref().unwrap().entity_id
    )).map_err(|e| Status::invalid_argument(format!("Invalid action: {}", e)))?;
    
    let resource = EntityUid::from_str(&format!(
        "{}::\"{}\"",
        req.resource.as_ref().unwrap().entity_type,
        req.resource.as_ref().unwrap().entity_id
    )).map_err(|e| Status::invalid_argument(format!("Invalid resource: {}", e)))?;
    
    // 4. Construir context (vacío por ahora)
    let context = Context::empty();
    
    // 5. Construir entities (vacío por ahora)
    let entities = Entities::empty();
    
    // 6. Crear request de Cedar
    let cedar_request = CedarRequest::new(principal, action, resource, context, None)
        .map_err(|e| Status::internal(format!("Failed to create Cedar request: {}", e)))?;
    
    // 7. Evaluar con Cedar Authorizer
    let authorizer = Authorizer::new();
    let response = authorizer.is_authorized(&cedar_request, &policy_set, &entities);
    
    // 8. Convertir decisión
    let decision = match response.decision() {
        cedar_policy::Decision::Allow => Decision::Allow,
        cedar_policy::Decision::Deny => Decision::Deny,
    };
    
    // 9. Extraer políticas determinantes
    let determining_policies: Vec<String> = response
        .diagnostics()
        .reason()
        .map(|policy_id| policy_id.to_string())
        .collect();
    
    // 10. Extraer errores
    let errors: Vec<String> = response
        .diagnostics()
        .errors()
        .map(|err| err.to_string())
        .collect();
    
    // 11. Retornar respuesta
    Ok(Response::new(IsAuthorizedResponse {
        decision: decision as i32,
        determining_policies,
        errors,
    }))
}
```

#### Pasos para Completar

1. ✅ **Añadir imports necesarios**
   ```rust
   use cedar_policy::{Authorizer, Context, Entities, EntityUid, PolicySet, Request as CedarRequest};
   use std::str::FromStr;
   ```

2. ✅ **Modificar struct para incluir repository**
   ```rust
   pub struct AuthorizationDataService {
       repository: Arc<RepositoryAdapter>,
   }
   ```

3. ✅ **Actualizar constructor**
   ```rust
   impl AuthorizationDataService {
       pub fn new(repository: Arc<RepositoryAdapter>) -> Self {
           Self { repository }
       }
   }
   ```

4. ✅ **Implementar lógica completa de is_authorized**
   - Cargar políticas
   - Construir PolicySet
   - Evaluar con Cedar
   - Retornar decisión real

5. ✅ **Actualizar main.rs para inyectar repository**
   ```rust
   let database_url = std::env::var("DATABASE_URL")
       .unwrap_or_else(|_| "sqlite:///app/data/hodei.db".to_string());
   
   let repository = Arc::new(
       RepositoryAdapter::new(&database_url).await?
   );
   
   let data_service = AuthorizationDataService::new(repository.clone());
   ```

6. ✅ **Crear test E2E**
   - Test con política ALLOW
   - Test con política DENY
   - Test sin políticas (debe DENY)

#### Estimación

- **Tiempo:** 2-3 horas
- **Complejidad:** Media
- **Dependencias:** Repository debe estar funcional
- **Prioridad:** 🔴 CRÍTICA

#### Resultado Final

**Estado:** ❌ **NO COMPLETADA**  
**Funcionalidad Real:** 0%  
**Bloqueadores:** Sin repository, sin evaluación Cedar  
**Riesgo:** 🔴 CRÍTICO DE SEGURIDAD (siempre ALLOW)

---

### ✅ HU 1.2: Incluir datos de entidades en la solicitud (ABAC)

**Estado en Docs:** ✅ COMPLETADA  
**Estado Real:** ❌ **NO FUNCIONAL**

#### Criterios de Aceptación

| # | Criterio | Estado | Evidencia |
|---|----------|--------|-----------|
| 1 | `IsAuthorized` acepta lista de entidades | ✅ CUMPLE | Proto define `repeated Entity entities` |
| 2 | El servicio deserializa entidades correctamente | ⚠️ CÓDIGO EXISTE | Función `build_entities` existe pero no se usa |
| 3 | Pasa entidades al motor Cedar | ❌ NO CUMPLE | No se llama a `build_entities` |
| 4 | Políticas con `in` operator funcionan | ❌ NO CUMPLE | No se evalúan entidades |
| 5 | Acceso a atributos funciona | ❌ NO CUMPLE | No se procesan atributos |

#### Código Actual

**Archivo:** `verified-permissions/api/src/grpc/data_plane.rs:26-67`

```rust
// ✅ Función existe pero NO se usa
fn build_entities(entities: &[Entity]) -> Result<Entities, Status> {
    let mut entities_json = Vec::new();
    
    for entity in entities {
        let uid = Self::build_entity_uid(entity.identifier.as_ref().unwrap())?;
        
        let mut attrs = HashMap::new();
        for (key, value) in &entity.attributes {
            attrs.insert(key.clone(), serde_json::from_str(value).map_err(|e| {
                error!("Failed to parse attribute: {}", e);
                Status::invalid_argument(format!("Invalid attribute {}: {}", key, e))
            })?);
        }
        
        let parents = entity.parents.iter()
            .map(|p| Self::build_entity_uid(p))
            .collect::<Result<Vec<_>, _>>()?;
        
        entities_json.push(serde_json::json!({
            "uid": uid.to_string(),
            "attrs": attrs,
            "parents": parents.iter().map(|p| p.to_string()).collect::<Vec<_>>()
        }));
    }
    
    Entities::from_json_value(serde_json::Value::Array(entities_json), None).map_err(|e| {
        error!("Failed to build entities: {}", e);
        Status::internal(format!("Failed to build entities: {}", e))
    })
}
```

#### Problemas Identificados

1. **⚠️ Código Existe Pero No Se Usa**
   - Función `build_entities` implementada
   - Nunca se llama en `is_authorized`
   - Entidades se ignoran completamente

2. **❌ ABAC No Funciona**
   - Políticas con `principal in Group::"admins"` no evalúan
   - Acceso a atributos como `resource.owner` no funciona
   - Jerarquías de entidades ignoradas

#### Test Requerido

```rust
#[tokio::test]
async fn test_hu_1_2_abac_with_group_membership() {
    let repository = create_test_repository().await;
    let policy_store = repository.create_policy_store(None).await.unwrap();
    
    // Política que permite a admins
    let policy = r#"
        permit(
            principal in Group::"admins",
            action == Action::"delete",
            resource
        );
    "#;
    
    repository.create_policy(
        &policy_store.id,
        &PolicyId::new("allow-admins").unwrap(),
        CedarPolicy::new(policy.to_string()).unwrap(),
        None
    ).await.unwrap();
    
    let service = AuthorizationDataService::new(Arc::new(repository));
    
    // Test: Usuario que ES admin
    let request_admin = IsAuthorizedRequest {
        policy_store_id: policy_store.id.into_string(),
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "delete".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "Document".to_string(),
            entity_id: "doc1".to_string(),
        }),
        context: None,
        entities: vec![
            Entity {
                identifier: Some(EntityIdentifier {
                    entity_type: "User".to_string(),
                    entity_id: "alice".to_string(),
                }),
                attributes: HashMap::new(),
                parents: vec![EntityIdentifier {
                    entity_type: "Group".to_string(),
                    entity_id: "admins".to_string(),
                }],
            },
            Entity {
                identifier: Some(EntityIdentifier {
                    entity_type: "Group".to_string(),
                    entity_id: "admins".to_string(),
                }),
                attributes: HashMap::new(),
                parents: vec![],
            },
        ],
    };
    
    let response = service.is_authorized(Request::new(request_admin)).await.unwrap();
    assert_eq!(response.into_inner().decision, Decision::Allow as i32);
    
    // Test: Usuario que NO es admin
    let request_non_admin = IsAuthorizedRequest {
        policy_store_id: policy_store.id.into_string(),
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "bob".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "delete".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "Document".to_string(),
            entity_id: "doc1".to_string(),
        }),
        context: None,
        entities: vec![
            Entity {
                identifier: Some(EntityIdentifier {
                    entity_type: "User".to_string(),
                    entity_id: "bob".to_string(),
                }),
                attributes: HashMap::new(),
                parents: vec![],  // No es admin
            },
        ],
    };
    
    let response = service.is_authorized(Request::new(request_non_admin)).await.unwrap();
    assert_eq!(response.into_inner().decision, Decision::Deny as i32);
}

#[tokio::test]
async fn test_hu_1_2_abac_with_attributes() {
    // Test con atributos: resource.owner == principal
    let policy = r#"
        permit(
            principal,
            action == Action::"read",
            resource
        ) when {
            resource.owner == principal
        };
    "#;
    
    // ... similar al test anterior pero verificando atributos
}
```

#### Modificación Requerida en is_authorized

```rust
async fn is_authorized(...) {
    // ... código existente ...
    
    // 5. Construir entities (AÑADIR ESTO)
    let entities = Self::build_entities(&req.entities)?;  // ✅ Usar la función
    
    // 6. Crear request de Cedar
    let cedar_request = CedarRequest::new(principal, action, resource, context, None)?;
    
    // 7. Evaluar con entidades
    let response = authorizer.is_authorized(&cedar_request, &policy_set, &entities);  // ✅ Pasar entities
    
    // ... resto del código ...
}
```

#### Pasos para Completar

1. ✅ **Usar build_entities en is_authorized**
   - Llamar a `Self::build_entities(&req.entities)?`
   - Pasar resultado al `authorizer.is_authorized`

2. ✅ **Crear tests ABAC**
   - Test con `in` operator
   - Test con atributos
   - Test con jerarquías complejas

#### Estimación

- **Tiempo:** 1 hora
- **Complejidad:** Baja (código ya existe)
- **Dependencias:** HU 1.1 completada
- **Prioridad:** 🟡 ALTA

#### Resultado Final

**Estado:** ❌ **NO COMPLETADA**  
**Funcionalidad Real:** 30% (código existe pero no se usa)  
**Bloqueadores:** Depende de HU 1.1  
**Riesgo:** 🟡 ALTO (ABAC no funciona)

---

### ✅ HU 1.3: Incorporar el context en la decisión

**Estado en Docs:** ✅ COMPLETADA  
**Estado Real:** ❌ **NO FUNCIONAL**

#### Criterios de Aceptación

| # | Criterio | Estado | Evidencia |
|---|----------|--------|-----------|
| 1 | `IsAuthorized` acepta context JSON | ✅ CUMPLE | Proto define `optional string context` |
| 2 | El servicio pasa context a Cedar | ❌ NO CUMPLE | No se llama a `build_context` |
| 3 | Políticas con `when` basadas en context funcionan | ❌ NO CUMPLE | Context ignorado |

#### Código Actual

```rust
// ✅ Función existe pero NO se usa
fn build_context(context_json: Option<&str>) -> Result<Context, Status> {
    if let Some(json_str) = context_json {
        let value: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
            error!("Failed to parse context JSON: {}", e);
            Status::invalid_argument(format!("Invalid context JSON: {}", e))
        })?;

        Context::from_json_value(value, None).map_err(|e| {
            error!("Failed to build context: {}", e);
            Status::invalid_argument(format!("Invalid context: {}", e))
        })
    } else {
        Ok(Context::empty())
    }
}
```

#### Test Requerido

```rust
#[tokio::test]
async fn test_hu_1_3_context_based_policy() {
    let repository = create_test_repository().await;
    let policy_store = repository.create_policy_store(None).await.unwrap();
    
    // Política que permite solo durante horas de oficina
    let policy = r#"
        permit(
            principal,
            action == Action::"access",
            resource
        ) when {
            context.hour >= 9 && context.hour <= 17
        };
    "#;
    
    repository.create_policy(...).await.unwrap();
    
    let service = AuthorizationDataService::new(Arc::new(repository));
    
    // Test: Durante horas de oficina (debe ALLOW)
    let request_office_hours = IsAuthorizedRequest {
        policy_store_id: policy_store.id.into_string(),
        principal: Some(EntityIdentifier {
            entity_type: "User".to_string(),
            entity_id: "alice".to_string(),
        }),
        action: Some(EntityIdentifier {
            entity_type: "Action".to_string(),
            entity_id: "access".to_string(),
        }),
        resource: Some(EntityIdentifier {
            entity_type: "System".to_string(),
            entity_id: "production".to_string(),
        }),
        context: Some(r#"{"hour": 14}"#.to_string()),  // 2 PM
        entities: vec![],
    };
    
    let response = service.is_authorized(Request::new(request_office_hours)).await.unwrap();
    assert_eq!(response.into_inner().decision, Decision::Allow as i32);
    
    // Test: Fuera de horas de oficina (debe DENY)
    let request_after_hours = IsAuthorizedRequest {
        // ... mismo request pero con context: {"hour": 22}
        context: Some(r#"{"hour": 22}"#.to_string()),  // 10 PM
        ..request_office_hours
    };
    
    let response = service.is_authorized(Request::new(request_after_hours)).await.unwrap();
    assert_eq!(response.into_inner().decision, Decision::Deny as i32);
}
```

#### Modificación Requerida

```rust
async fn is_authorized(...) {
    // ... código existente ...
    
    // 4. Construir context (AÑADIR ESTO)
    let context = Self::build_context(req.context.as_deref())?;  // ✅ Usar la función
    
    // 6. Crear request de Cedar con context
    let cedar_request = CedarRequest::new(principal, action, resource, context, None)?;
    
    // ... resto del código ...
}
```

#### Estimación

- **Tiempo:** 30 minutos
- **Complejidad:** Muy Baja
- **Dependencias:** HU 1.1 completada
- **Prioridad:** 🟡 ALTA

#### Resultado Final

**Estado:** ❌ **NO COMPLETADA**  
**Funcionalidad Real:** 30% (código existe pero no se usa)  
**Bloqueadores:** Depende de HU 1.1  
**Riesgo:** 🟡 MEDIO (políticas condicionales no funcionan)

---

## 📊 RESUMEN ÉPICA 1

| HU | Docs | Real | Funcionalidad | Tiempo | Prioridad |
|----|------|------|---------------|--------|-----------|
| 1.1 | ✅ | ❌ | 0% | 2-3h | 🔴 CRÍTICA |
| 1.2 | ✅ | ❌ | 30% | 1h | 🟡 ALTA |
| 1.3 | ✅ | ❌ | 30% | 30min | 🟡 ALTA |

**Total Épica 1:** 20% funcional (vs 100% documentado)  
**Tiempo Total Estimado:** 4-5 horas  
**Bloqueador Principal:** HU 1.1 debe completarse primero

---

## 🎯 PLAN DE ACCIÓN ÉPICA 1

### Orden de Implementación

1. **HU 1.1** (2-3h) - Base crítica
   - Añadir repository a AuthorizationDataService
   - Implementar carga de políticas
   - Implementar evaluación Cedar
   - Crear tests E2E

2. **HU 1.2** (1h) - ABAC
   - Usar build_entities en is_authorized
   - Crear tests con grupos y atributos

3. **HU 1.3** (30min) - Context
   - Usar build_context en is_authorized
   - Crear tests con políticas condicionales

### Tests E2E Requeridos

```bash
# Crear archivo: tests/e2e_epica1.rs

cargo test --test e2e_epica1 test_hu_1_1 -- --ignored --nocapture
cargo test --test e2e_epica1 test_hu_1_2 -- --ignored --nocapture
cargo test --test e2e_epica1 test_hu_1_3 -- --ignored --nocapture
```

---

**Siguiente:** Continuar con Épica 2 una vez completada Épica 1

