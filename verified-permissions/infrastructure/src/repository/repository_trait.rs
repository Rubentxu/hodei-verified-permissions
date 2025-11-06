//! Policy Repository Trait - Abstracción para múltiples bases de datos

use async_trait::async_trait;
use crate::storage::models::{PolicyStore, Schema, Policy, IdentitySource};
use crate::error::Result;

/// Trait que define las operaciones de persistencia para el sistema de autorización.
///
/// Este trait abstrae la capa de persistencia permitiendo múltiples implementaciones:
/// - SQLite (desarrollo, embedded)
/// - PostgreSQL (producción, escalable)
/// - SurrealDB (experimental, graph queries)
///
/// # Principio de Diseño
///
/// La base de datos se usa SOLO para:
/// - Persistencia de políticas y schemas
/// - Recuperación al inicio del servidor
/// - Auditoría de decisiones
///
/// NO se usa para:
/// - Evaluación de políticas (eso es Cedar en memoria)
/// - Cada request de autorización
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    // ============================================================================
    // Policy Store Operations
    // ============================================================================

    /// Crea un nuevo Policy Store
    ///
    /// # Arguments
    /// * `description` - Descripción opcional del store
    ///
    /// # Returns
    /// El Policy Store creado con su ID generado
    async fn create_policy_store(&self, description: Option<String>) -> Result<PolicyStore>;

    /// Obtiene un Policy Store por su ID
    ///
    /// # Arguments
    /// * `id` - ID del Policy Store
    ///
    /// # Returns
    /// El Policy Store si existe
    ///
    /// # Errors
    /// Retorna error si el store no existe
    async fn get_policy_store(&self, id: &str) -> Result<PolicyStore>;

    /// Lista todos los Policy Stores
    ///
    /// # Returns
    /// Vector con todos los Policy Stores
    async fn list_policy_stores(&self) -> Result<Vec<PolicyStore>>;

    /// Elimina un Policy Store y todo su contenido (cascade)
    ///
    /// # Arguments
    /// * `id` - ID del Policy Store a eliminar
    ///
    /// # Note
    /// Esta operación elimina en cascada:
    /// - Todas las políticas del store
    /// - El schema del store
    /// - Todos los identity sources del store
    async fn delete_policy_store(&self, id: &str) -> Result<()>;

    // ============================================================================
    // Schema Operations
    // ============================================================================

    /// Guarda o actualiza el schema de un Policy Store
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `schema` - Schema en formato JSON (Cedar schema format)
    ///
    /// # Note
    /// Si ya existe un schema, lo reemplaza
    async fn put_schema(&self, policy_store_id: &str, schema: String) -> Result<()>;

    /// Obtiene el schema de un Policy Store
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    ///
    /// # Returns
    /// El schema si existe
    ///
    /// # Errors
    /// Retorna error si el store no existe o no tiene schema
    async fn get_schema(&self, policy_store_id: &str) -> Result<Schema>;

    /// Elimina el schema de un Policy Store
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    async fn delete_schema(&self, policy_store_id: &str) -> Result<()>;

    // ============================================================================
    // Policy Operations
    // ============================================================================

    /// Crea una nueva política
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `policy_id` - ID único de la política
    /// * `statement` - Statement de la política en formato Cedar
    /// * `description` - Descripción opcional
    ///
    /// # Returns
    /// La política creada
    ///
    /// # Errors
    /// Retorna error si:
    /// - El store no existe
    /// - El policy_id ya existe
    /// - El statement es inválido
    async fn create_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy>;

    /// Obtiene una política por su ID
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `policy_id` - ID de la política
    ///
    /// # Returns
    /// La política si existe
    async fn get_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<Policy>;

    /// Lista todas las políticas de un Policy Store
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    ///
    /// # Returns
    /// Vector con todas las políticas del store
    async fn list_policies(&self, policy_store_id: &str) -> Result<Vec<Policy>>;

    /// Actualiza una política existente
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `policy_id` - ID de la política
    /// * `statement` - Nuevo statement
    /// * `description` - Nueva descripción opcional
    async fn update_policy(
        &self,
        policy_store_id: &str,
        policy_id: &str,
        statement: String,
        description: Option<String>,
    ) -> Result<Policy>;

    /// Elimina una política
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `policy_id` - ID de la política
    async fn delete_policy(&self, policy_store_id: &str, policy_id: &str) -> Result<()>;

    // ============================================================================
    // Identity Source Operations
    // ============================================================================

    /// Crea un nuevo Identity Source
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `provider_type` - Tipo de proveedor ("oidc", "cognito")
    /// * `config` - Configuración en JSON
    /// * `claims_mapping` - Mapeo de claims opcional
    /// * `description` - Descripción opcional
    ///
    /// # Returns
    /// El Identity Source creado
    async fn create_identity_source(
        &self,
        policy_store_id: &str,
        provider_type: &str,
        config: &str,
        claims_mapping: Option<&str>,
        description: Option<&str>,
    ) -> Result<IdentitySource>;

    /// Obtiene un Identity Source por su ID
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `identity_source_id` - ID del Identity Source
    ///
    /// # Returns
    /// El Identity Source si existe
    async fn get_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<IdentitySource>;

    /// Lista todos los Identity Sources de un Policy Store
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    ///
    /// # Returns
    /// Vector con todos los Identity Sources
    async fn list_identity_sources(&self, policy_store_id: &str) -> Result<Vec<IdentitySource>>;

    /// Elimina un Identity Source
    ///
    /// # Arguments
    /// * `policy_store_id` - ID del Policy Store
    /// * `identity_source_id` - ID del Identity Source
    async fn delete_identity_source(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
    ) -> Result<()>;

    // ============================================================================
    // Audit Operations
    // ============================================================================
}
