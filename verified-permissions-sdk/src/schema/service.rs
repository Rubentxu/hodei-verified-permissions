//! Schema generation service implementation

use super::openapi_mapper::*;
use super::types::*;
use super::SchemaGenerationUseCase;
use openapiv3::OpenAPI;
use std::collections::{HashMap, HashSet};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaGenerationError {
    #[error("Failed to parse OpenAPI spec: {0}")]
    ParseError(String),
    #[error("Mapping error: {0}")]
    MappingError(#[from] MappingError),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Service that generates Cedar schemas from OpenAPI specifications
pub struct SimpleRestSchemaGenerator;

impl SimpleRestSchemaGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate schema synchronously
    pub fn generate_schema(
        &self,
        spec_json: &str,
        namespace: &str,
        base_path: Option<&str>,
    ) -> Result<SchemaBundle> {
        // Validate namespace
        validate_namespace(namespace)?;

        // Parse OpenAPI spec
        let openapi: OpenAPI = serde_json::from_str(spec_json)
            .map_err(|e| SchemaGenerationError::ParseError(e.to_string()))?;

        // Validate and normalize base path
        let normalized_base_path = validate_base_path(&openapi, base_path)?;

        // Initialize schema structure
        let mut entity_types = HashMap::new();
        let mut actions = HashMap::new();
        let mut resources_to_add = HashSet::new();

        // Add default entity types
        entity_types.insert(
            "User".to_string(),
            EntityType {
                shape: TypeDefinition::Record {
                    attributes: HashMap::new(),
                },
                member_of_types: Some(vec!["UserGroup".to_string()]),
            },
        );

        entity_types.insert(
            "UserGroup".to_string(),
            EntityType {
                shape: TypeDefinition::Record {
                    attributes: HashMap::new(),
                },
                member_of_types: None,
            },
        );

        entity_types.insert(
            "Application".to_string(),
            EntityType {
                shape: TypeDefinition::Record {
                    attributes: HashMap::new(),
                },
                member_of_types: None,
            },
        );

        // Process paths and operations
        let paths = &openapi.paths;
        if !paths.paths.is_empty() {
            for (path_template, path_item_ref) in paths.paths.iter() {
                let path_item = match path_item_ref {
                    openapiv3::ReferenceOr::Item(item) => item,
                    openapiv3::ReferenceOr::Reference { .. } => continue,
                };

                // Process each HTTP method
                self.process_operation(
                    &path_item.get,
                    HttpMethod::Get,
                    path_template,
                    &normalized_base_path,
                    &mut actions,
                    &mut resources_to_add,
                )?;

                self.process_operation(
                    &path_item.post,
                    HttpMethod::Post,
                    path_template,
                    &normalized_base_path,
                    &mut actions,
                    &mut resources_to_add,
                )?;

                self.process_operation(
                    &path_item.put,
                    HttpMethod::Put,
                    path_template,
                    &normalized_base_path,
                    &mut actions,
                    &mut resources_to_add,
                )?;

                self.process_operation(
                    &path_item.patch,
                    HttpMethod::Patch,
                    path_template,
                    &normalized_base_path,
                    &mut actions,
                    &mut resources_to_add,
                )?;

                self.process_operation(
                    &path_item.delete,
                    HttpMethod::Delete,
                    path_template,
                    &normalized_base_path,
                    &mut actions,
                    &mut resources_to_add,
                )?;
            }
        } else {
            return Err(MappingError::MissingPaths.into());
        }

        // Add discovered resource types
        for resource_name in resources_to_add {
            if !entity_types.contains_key(&resource_name) {
                entity_types.insert(
                    resource_name,
                    EntityType {
                        shape: TypeDefinition::Record {
                            attributes: HashMap::new(),
                        },
                        member_of_types: None,
                    },
                );
            }
        }

        // Build namespace fragment
        let mut annotations = HashMap::new();
        annotations.insert(
            "mappingType".to_string(),
            serde_json::Value::String("SimpleRest".to_string()),
        );

        let namespace_fragment = NamespaceFragment {
            annotations: Some(annotations),
            entity_types,
            actions,
            common_types: None,
        };

        // Build complete schema
        let mut namespaces = HashMap::new();
        namespaces.insert(namespace.to_string(), namespace_fragment.clone());

        let schema = CedarSchemaJson { namespaces };

        // Serialize to v4
        let v4 = serde_json::to_string_pretty(&schema)
            .map_err(|e| SchemaGenerationError::SerializationError(e.to_string()))?;

        // Create metadata
        let metadata = SchemaMetadata {
            namespace: namespace.to_string(),
            base_path: if normalized_base_path.is_empty() {
                None
            } else {
                Some(normalized_base_path)
            },
            mapping_type: "SimpleRest".to_string(),
            action_count: namespace_fragment.actions.len(),
            entity_type_count: namespace_fragment.entity_types.len(),
        };

        Ok(SchemaBundle {
            v4,
            v2: None, // v2 not implemented yet
            metadata,
        })
    }

    fn process_operation(
        &self,
        operation: &Option<openapiv3::Operation>,
        method: HttpMethod,
        path_template: &str,
        base_path: &str,
        actions: &mut HashMap<String, ActionType>,
        resources_to_add: &mut HashSet<String>,
    ) -> Result<()> {
        if let Some(op) = operation {
            let action_def = generate_action_from_operation(method, path_template, op, base_path)?;

            // Collect resource types
            for resource_type in &action_def.resource_types {
                let resource_name = resource_type.split("::").last().unwrap_or(resource_type);
                if resource_name != "Application" {
                    resources_to_add.insert(resource_name.to_string());
                }
            }

            // Build action type
            let mut action_annotations = HashMap::new();
            action_annotations.insert(
                "httpVerb".to_string(),
                serde_json::Value::String(action_def.http_verb.as_str().to_string()),
            );
            action_annotations.insert(
                "httpPathTemplate".to_string(),
                serde_json::Value::String(action_def.http_path_template.clone()),
            );

            let action_type = ActionType {
                annotations: Some(action_annotations),
                applies_to: Some(AppliesTo {
                    principal_types: vec!["User".to_string()],
                    resource_types: action_def.resource_types.clone(),
                    context: TypeDefinition::Record {
                        attributes: action_def.context_attributes,
                    },
                }),
            };

            actions.insert(action_def.action_name, action_type);
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl SchemaGenerationUseCase for SimpleRestSchemaGenerator {
    async fn generate_simple_rest_schema(
        &self,
        spec: &str,
        namespace: &str,
        base_path: Option<&str>,
    ) -> Result<SchemaBundle, anyhow::Error> {
        self.generate_schema(spec, namespace, base_path)
    }
}

impl Default for SimpleRestSchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}
