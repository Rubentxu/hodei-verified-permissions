//! Schema generation module for Cedar authorization schemas
//! 
//! This module provides functionality to generate Cedar schemas from OpenAPI specifications
//! following the SimpleRest mapping pattern.

pub mod types;
pub mod service;
pub mod openapi_mapper;
pub mod serialization;

#[cfg(feature = "runtime-mapping")]
pub mod runtime_mapping;

pub use types::*;
pub use service::SimpleRestSchemaGenerator;

#[cfg(feature = "runtime-mapping")]
pub use runtime_mapping::{SimpleRestMapping, ResolvedRoute};

/// Port for schema generation use case
#[async_trait::async_trait]
pub trait SchemaGenerationUseCase: Send + Sync {
    /// Generate a SimpleRest Cedar schema from an OpenAPI specification
    async fn generate_simple_rest_schema(
        &self,
        spec: &str,
        namespace: &str,
        base_path: Option<&str>,
    ) -> Result<SchemaBundle, anyhow::Error>;
}
