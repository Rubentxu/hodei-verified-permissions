# 📘 Guía de Multi-Tenancy - Hodei Verified Permissions

**Versión**: 1.0  
**Fecha**: 18 de Octubre, 2025  
**Estado**: Épica 7 - Documentación Completa

## 🎯 Objetivo

Esta guía documenta los patrones arquitectónicos para implementar autorización en aplicaciones SaaS multi-tenant, garantizando aislamiento estricto entre tenants y escalabilidad.

## 📊 Patrones Soportados

### 1. PolicyStore por Tenant (Recomendado)
**Aislamiento**: Máximo  
**Complejidad**: Baja  
**Escalabilidad**: Alta

### 2. Aislamiento Lógico Compartido
**Aislamiento**: Medio  
**Complejidad**: Media  
**Escalabilidad**: Muy Alta

## 🏗️ Patrón 1: PolicyStore por Tenant

### Descripción

Cada tenant tiene su propio PolicyStore independiente, garantizando aislamiento completo de políticas, schemas y datos.

### Arquitectura

```
Tenant A                    Tenant B                    Tenant C
┌─────────────────┐        ┌─────────────────┐        ┌─────────────────┐
│ PolicyStore A   │        │ PolicyStore B   │        │ PolicyStore C   │
│ ├─ Schema A     │        │ ├─ Schema B     │        │ ├─ Schema C     │
│ ├─ Policies A   │        │ ├─ Policies B   │        │ ├─ Policies C   │
│ └─ Entities A   │        │ └─ Entities B   │        │ └─ Entities C   │
└─────────────────┘        └─────────────────┘        └─────────────────┘
```

### Ventajas

- ✅ **Aislamiento Total**: Imposible acceder a datos de otro tenant
- ✅ **Personalización**: Cada tenant puede tener su propio schema
- ✅ **Seguridad**: Máxima garantía de privacidad
- ✅ **Simplicidad**: Fácil de implementar y entender
- ✅ **Migración**: Fácil mover tenants entre instancias

### Desventajas

- ⚠️ **Recursos**: Más PolicyStores = más recursos
- ⚠️ **Gestión**: Más stores para administrar
- ⚠️ **Políticas Comunes**: Duplicación si hay políticas compartidas

### Implementación

#### Paso 1: Crear PolicyStore por Tenant

```rust
use hodei_verified_permissions::proto::*;

async fn onboard_tenant(
    client: &mut AuthorizationControlClient<Channel>,
    tenant_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Crear PolicyStore para el tenant
    let response = client
        .create_policy_store(CreatePolicyStoreRequest {
            description: Some(format!("PolicyStore for tenant: {}", tenant_id)),
        })
        .await?;
    
    let policy_store_id = response.into_inner().policy_store_id;
    
    // Guardar mapping tenant_id -> policy_store_id en tu DB
    save_tenant_mapping(tenant_id, &policy_store_id).await?;
    
    Ok(policy_store_id)
}
```

#### Paso 2: Resolver PolicyStore en Requests

```rust
async fn authorize_request(
    tenant_id: &str,
    principal: &str,
    action: &str,
    resource: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Obtener PolicyStore del tenant
    let policy_store_id = get_policy_store_for_tenant(tenant_id).await?;
    
    // Realizar autorización
    let response = client
        .is_authorized(IsAuthorizedRequest {
            policy_store_id,
            principal: Some(EntityIdentifier {
                entity_type: "User".to_string(),
                entity_id: principal.to_string(),
            }),
            action: Some(EntityIdentifier {
                entity_type: "Action".to_string(),
                entity_id: action.to_string(),
            }),
            resource: Some(EntityIdentifier {
                entity_type: "Resource".to_string(),
                entity_id: resource.to_string(),
            }),
            context: None,
            entities: vec![],
        })
        .await?;
    
    Ok(response.into_inner().decision() == Decision::Allow)
}
```

#### Paso 3: Middleware de Tenant

```rust
// Middleware para extraer tenant_id
async fn tenant_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, StatusCode> {
    // Extraer tenant_id del header, subdomain, o JWT
    let tenant_id = extract_tenant_id(&req)?;
    
    // Agregar a request context
    req.extensions_mut().insert(TenantContext { tenant_id });
    
    Ok(next.run(req).await)
}

fn extract_tenant_id(req: &Request<Body>) -> Result<String, StatusCode> {
    // Opción 1: Header
    if let Some(tenant_id) = req.headers().get("X-Tenant-ID") {
        return Ok(tenant_id.to_str().unwrap().to_string());
    }
    
    // Opción 2: Subdomain
    if let Some(host) = req.headers().get("Host") {
        let host_str = host.to_str().unwrap();
        if let Some(tenant_id) = host_str.split('.').next() {
            return Ok(tenant_id.to_string());
        }
    }
    
    // Opción 3: JWT claim
    if let Some(auth) = req.headers().get("Authorization") {
        let token = extract_jwt(auth)?;
        if let Some(tenant_id) = token.claims.get("tenant_id") {
            return Ok(tenant_id.to_string());
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}
```

### Ejemplo Completo

```rust
// Aplicación SaaS con PolicyStore por Tenant

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AuthorizationControlClient::connect("http://localhost:50051").await?;
    
    // Onboarding de tenants
    let tenant_a_store = onboard_tenant(&mut client, "acme-corp").await?;
    let tenant_b_store = onboard_tenant(&mut client, "globex").await?;
    
    // Subir schema para cada tenant
    upload_schema(&mut client, &tenant_a_store, ACME_SCHEMA).await?;
    upload_schema(&mut client, &tenant_b_store, GLOBEX_SCHEMA).await?;
    
    // Crear políticas para cada tenant
    create_policies(&mut client, &tenant_a_store, ACME_POLICIES).await?;
    create_policies(&mut client, &tenant_b_store, GLOBEX_POLICIES).await?;
    
    // Autorización para Tenant A
    let allowed = authorize_request(
        "acme-corp",
        "alice",
        "view",
        "document:123"
    ).await?;
    
    println!("Tenant A - Alice can view document: {}", allowed);
    
    Ok(())
}
```

## 🏗️ Patrón 2: Aislamiento Lógico Compartido

### Descripción

Todos los tenants comparten un único PolicyStore, pero el aislamiento se logra mediante atributos en las entidades y condiciones en las políticas.

### Arquitectura

```
                    PolicyStore Compartido
┌────────────────────────────────────────────────────────┐
│  Schema Común                                          │
│  ├─ User (con atributo tenant_id)                     │
│  ├─ Document (con atributo tenant_id)                 │
│  └─ ...                                                │
│                                                         │
│  Políticas con Condiciones                            │
│  ├─ permit(...) when { principal.tenant_id ==         │
│  │                      resource.tenant_id }          │
│  └─ ...                                                │
└────────────────────────────────────────────────────────┘
```

### Ventajas

- ✅ **Eficiencia**: Un solo PolicyStore para todos
- ✅ **Políticas Comunes**: Fácil compartir políticas base
- ✅ **Escalabilidad**: Menos overhead de gestión
- ✅ **Análisis**: Más fácil analizar patrones globales

### Desventajas

- ⚠️ **Complejidad**: Políticas más complejas
- ⚠️ **Riesgo**: Error en política puede afectar aislamiento
- ⚠️ **Personalización**: Difícil tener schemas diferentes

### Implementación

#### Paso 1: Schema con Tenant ID

```json
{
  "": {
    "entityTypes": {
      "User": {
        "shape": {
          "type": "Record",
          "attributes": {
            "tenant_id": { "type": "String" },
            "email": { "type": "String" },
            "department": { "type": "String" }
          }
        }
      },
      "Document": {
        "shape": {
          "type": "Record",
          "attributes": {
            "tenant_id": { "type": "String" },
            "owner": { "type": "String" },
            "classification": { "type": "String" }
          }
        }
      }
    },
    "actions": {
      "view": {},
      "edit": {},
      "delete": {}
    }
  }
}
```

#### Paso 2: Políticas con Aislamiento

```cedar
// Política base: Solo acceso dentro del mismo tenant
permit(
    principal,
    action,
    resource
)
when {
    principal.tenant_id == resource.tenant_id
};

// Política específica: Owners pueden editar
permit(
    principal,
    action == Action::"edit",
    resource
)
when {
    principal.tenant_id == resource.tenant_id &&
    resource.owner == principal.email
};
```

#### Paso 3: Entidades con Tenant ID

```rust
async fn create_entities_with_tenant(
    tenant_id: &str,
) -> Vec<Entity> {
    vec![
        Entity {
            identifier: Some(EntityIdentifier {
                entity_type: "User".to_string(),
                entity_id: format!("{}::alice", tenant_id),
            }),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("tenant_id".to_string(), tenant_id.to_string());
                attrs.insert("email".to_string(), "alice@example.com".to_string());
                attrs
            },
            parents: vec![],
        },
        Entity {
            identifier: Some(EntityIdentifier {
                entity_type: "Document".to_string(),
                entity_id: format!("{}::doc123", tenant_id),
            }),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("tenant_id".to_string(), tenant_id.to_string());
                attrs.insert("owner".to_string(), "alice@example.com".to_string());
                attrs
            },
            parents: vec![],
        },
    ]
}
```

## 📊 Comparación de Patrones

| Aspecto | PolicyStore por Tenant | Aislamiento Compartido |
|---------|------------------------|------------------------|
| **Aislamiento** | ⭐⭐⭐⭐⭐ Máximo | ⭐⭐⭐ Medio |
| **Seguridad** | ⭐⭐⭐⭐⭐ Máxima | ⭐⭐⭐ Buena |
| **Simplicidad** | ⭐⭐⭐⭐ Simple | ⭐⭐ Complejo |
| **Escalabilidad** | ⭐⭐⭐ Buena | ⭐⭐⭐⭐⭐ Excelente |
| **Personalización** | ⭐⭐⭐⭐⭐ Total | ⭐⭐ Limitada |
| **Recursos** | ⭐⭐ Más recursos | ⭐⭐⭐⭐⭐ Eficiente |

## 🎯 Recomendaciones

### Usar PolicyStore por Tenant cuando:

- ✅ Seguridad y aislamiento son críticos
- ✅ Tenants necesitan schemas diferentes
- ✅ Compliance requiere separación física
- ✅ Número de tenants es manejable (< 10,000)
- ✅ Tenants grandes con muchas políticas

### Usar Aislamiento Compartido cuando:

- ✅ Tienes muchos tenants pequeños (> 10,000)
- ✅ Todos los tenants usan el mismo modelo
- ✅ Eficiencia de recursos es crítica
- ✅ Políticas base son comunes
- ✅ Análisis cross-tenant es necesario

## 🔒 Mejores Prácticas de Seguridad

### 1. Validación Estricta de Tenant ID

```rust
fn validate_tenant_id(tenant_id: &str) -> Result<(), Error> {
    // Solo alfanuméricos y guiones
    if !tenant_id.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(Error::InvalidTenantId);
    }
    
    // Longitud razonable
    if tenant_id.len() < 3 || tenant_id.len() > 50 {
        return Err(Error::InvalidTenantId);
    }
    
    Ok(())
}
```

### 2. Auditoría por Tenant

```rust
// Registrar todas las operaciones con tenant_id
audit_logger.log_decision(AuditEvent {
    tenant_id: Some(tenant_id.to_string()),
    principal,
    action,
    resource,
    decision,
    // ...
}).await;
```

### 3. Rate Limiting por Tenant

```rust
async fn check_rate_limit(tenant_id: &str) -> Result<(), Error> {
    let count = redis
        .incr(format!("rate:{}:{}", tenant_id, current_minute()))
        .await?;
    
    if count > MAX_REQUESTS_PER_MINUTE {
        return Err(Error::RateLimitExceeded);
    }
    
    Ok(())
}
```

### 4. Tests de Aislamiento

```rust
#[tokio::test]
async fn test_tenant_isolation() {
    // Crear dos tenants
    let store_a = create_tenant("tenant-a").await;
    let store_b = create_tenant("tenant-b").await;
    
    // Crear recurso en tenant A
    create_resource(&store_a, "doc-123").await;
    
    // Intentar acceder desde tenant B (debe fallar)
    let result = authorize(&store_b, "user-b", "view", "doc-123").await;
    assert_eq!(result.decision(), Decision::Deny);
}
```

## 📝 Checklist de Implementación

### PolicyStore por Tenant

- [ ] Implementar onboarding de tenants
- [ ] Crear mapping tenant_id → policy_store_id
- [ ] Middleware para extraer tenant_id
- [ ] Resolver PolicyStore en cada request
- [ ] Implementar offboarding de tenants
- [ ] Tests de aislamiento
- [ ] Auditoría por tenant
- [ ] Rate limiting por tenant
- [ ] Backup por tenant
- [ ] Documentación para desarrolladores

### Aislamiento Compartido

- [ ] Diseñar schema con tenant_id
- [ ] Crear políticas con condiciones de tenant
- [ ] Validar tenant_id en todas las entidades
- [ ] Implementar helpers para entidades
- [ ] Tests exhaustivos de aislamiento
- [ ] Auditoría con tenant_id
- [ ] Rate limiting por tenant
- [ ] Monitoring de políticas
- [ ] Documentación de patrones
- [ ] Guías de troubleshooting

## 🎉 Conclusión

Ambos patrones son válidos y soportados por Hodei Verified Permissions. La elección depende de tus requisitos específicos de seguridad, escala y personalización.

**Recomendación General**: Comenzar con **PolicyStore por Tenant** por su simplicidad y máximo aislamiento. Migrar a Aislamiento Compartido solo si la escala lo requiere.

---

**Épica 7 - Multi-Tenancy**: ✅ Documentación Completa  
**Fecha**: 18 de Octubre, 2025  
**Versión**: 1.0
