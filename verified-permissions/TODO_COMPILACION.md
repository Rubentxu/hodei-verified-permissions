# TODO: Completar Compilación del Servidor

## Estado Actual

El servidor `verified-permissions/` está en proceso de refactor a arquitectura hexagonal.
**Estado**: 33% completado, no compila actualmente.

## Errores Principales (103 errores totales)

### 1. Crate `api` - Errores de Imports

**Problema**: Faltan módulos y tipos necesarios

```
❌ error[E0433]: failed to resolve: could not find `jwt` in the crate root
❌ error[E0433]: failed to resolve: use of undeclared type `AuthorizationError`
❌ error[E0277]: the trait bound `Status: From<DomainError>` is not satisfied
```

**Solución Necesaria**:

1. **Crear módulo de errores en api**:
```rust
// api/src/errors.rs
use tonic::Status;
use hodei_domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl From<DomainError> for Status {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => Status::not_found(msg),
            DomainError::InvalidInput(msg) => Status::invalid_argument(msg),
            DomainError::Internal(msg) => Status::internal(msg),
            // ... otros casos
        }
    }
}

impl From<AuthorizationError> for Status {
    fn from(err: AuthorizationError) -> Self {
        match err {
            AuthorizationError::Domain(e) => e.into(),
            AuthorizationError::InvalidToken(msg) => Status::unauthenticated(msg),
            AuthorizationError::Unauthorized(msg) => Status::permission_denied(msg),
        }
    }
}
```

2. **Mover módulo JWT desde infrastructure**:
```bash
# El módulo jwt debería estar en infrastructure, no en api
# Necesita ser re-exportado o movido
```

3. **Actualizar imports en data_plane.rs y control_plane.rs**:
```rust
use crate::errors::{AuthorizationError};
use hodei_infrastructure::jwt;
```

### 2. Crate `infrastructure` - Repository Adapter Incompleto

**Problema**: Todos los métodos retornan "Not yet implemented"

**Solución Necesaria**:

Implementar cada método del `RepositoryAdapter` para convertir entre tipos de dominio y tipos de DB:

```rust
// infrastructure/src/repository/adapter.rs
impl PolicyRepository for RepositoryAdapter {
    async fn create_policy_store(&self, description: Option<String>) -> DomainResult<PolicyStore> {
        // 1. Llamar al sqlite_repo
        let db_store = self.sqlite_repo.create_policy_store(description).await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        // 2. Convertir a entidad de dominio
        let store_id = PolicyStoreId::new(db_store.id)?;
        Ok(PolicyStore::new(store_id, db_store.description))
    }
    
    // ... implementar los otros ~20 métodos
}
```

### 3. Módulos Comentados en Infrastructure

**Archivos comentados que necesitan ser habilitados**:
- `infrastructure/src/cache/`
- `infrastructure/src/jwt/`
- `infrastructure/src/config.rs`

**Solución**: Descomentar y actualizar imports uno por uno.

### 4. Crate `application` - Use Cases

**Estado**: Estructura creada pero no probada

**Solución**: Verificar compilación después de arreglar infrastructure.

### 5. Crate `main` - Binarios

**Problema**: Imports desactualizados

**Solución**: Actualizar imports después de que compile el resto.

## Plan de Acción (Estimado: 4-6 horas)

### Fase 1: Errores Críticos (2 horas)
1. ✅ Crear `api/src/errors.rs` con conversiones de errores
2. ✅ Mover/re-exportar módulo JWT
3. ✅ Actualizar imports en `data_plane.rs` y `control_plane.rs`
4. ✅ Verificar que `api` compila

### Fase 2: Repository Adapter (2 horas)
1. ✅ Implementar métodos del `RepositoryAdapter`
2. ✅ Crear conversores entre tipos de dominio y DB
3. ✅ Verificar que `infrastructure` compila

### Fase 3: Módulos Comentados (1 hora)
1. ✅ Descomentar y actualizar `cache/`
2. ✅ Descomentar y actualizar `jwt/`
3. ✅ Descomentar y actualizar `config.rs`

### Fase 4: Integration (1 hora)
1. ✅ Verificar `application` compila
2. ✅ Actualizar `main` binarios
3. ✅ Ejecutar tests
4. ✅ Verificar workspace completo

## Comandos Útiles

```bash
# Compilar crate específico
cd verified-permissions
cargo check -p hodei-api

# Ver errores detallados
cargo check -p hodei-api 2>&1 | less

# Compilar todo el workspace
cargo check --workspace

# Ejecutar tests cuando compile
cargo test --workspace
```

## Notas Importantes

1. **No afecta al proyecto principal**: El SDK y los ejemplos en la raíz funcionan perfectamente sin este servidor.

2. **Servidor necesario para E2E completos**: Para tests end-to-end con el servidor gRPC real, este servidor debe compilar y ejecutarse.

3. **Arquitectura hexagonal**: El refactor está bien diseñado, solo necesita completarse.

4. **Prioridad**: Baja si el objetivo es solo el SDK. Alta si se necesita el servidor completo.

## Estado de Compilación por Crate

| Crate | Estado | Errores | Acción Requerida |
|-------|--------|---------|------------------|
| hodei-shared | ✅ Compila | 0 | Ninguna |
| hodei-domain | ✅ Compila | 0 | Ninguna |
| hodei-application | ⏳ No probado | ? | Probar después de infrastructure |
| hodei-infrastructure | ❌ No compila | ~20 | Implementar RepositoryAdapter |
| hodei-api | ❌ No compila | 103 | Crear errors.rs, actualizar imports |
| hodei-main | ❌ No compila | ? | Actualizar después de api |

## Próximo Paso Inmediato

**Crear `api/src/errors.rs`** con las conversiones de errores. Este es el bloqueador principal.
