//! OpenAPI to Cedar schema mapping logic

use super::types::*;
use openapiv3::{OpenAPI, Operation, Parameter, ParameterSchemaOrContent, ReferenceOr, SchemaKind, Type as OApiType};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MappingError {
    #[error("Invalid namespace format: {0}")]
    InvalidNamespace(String),
    #[error("Base path '{0}' does not match any server in the API spec")]
    InvalidBasePath(String),
    #[error("OpenAPI spec is missing paths")]
    MissingPaths,
    #[error("Unsupported parameter type: {0}")]
    UnsupportedParameterType(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

const RESERVED_WORDS: &[&str] = &["if", "in", "is", "__cedar"];
// Supported HTTP methods for SimpleRest mapping
#[allow(dead_code)]
const SUPPORTED_HTTP_METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

/// Validates a Cedar namespace
pub fn validate_namespace(namespace: &str) -> Result<(), MappingError> {
    // Regex: ^[_a-zA-Z][_a-zA-Z0-9]*(::(?:[_a-zA-Z][_a-zA-Z0-9]*))*$
    let namespace_lower = namespace.to_lowercase();
    
    if RESERVED_WORDS.contains(&namespace_lower.as_str()) {
        return Err(MappingError::InvalidNamespace(
            format!("Namespace '{}' is a reserved word", namespace)
        ));
    }

    // Check format: starts with letter or underscore, contains only alphanumeric and underscore
    // Can have :: separators
    let parts: Vec<&str> = namespace.split("::").collect();
    for part in parts {
        if part.is_empty() {
            return Err(MappingError::InvalidNamespace(
                "Namespace components cannot be empty".to_string()
            ));
        }
        
        let first_char = part.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(MappingError::InvalidNamespace(
                format!("Namespace component '{}' must start with a letter or underscore", part)
            ));
        }
        
        if !part.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(MappingError::InvalidNamespace(
                format!("Namespace component '{}' contains invalid characters", part)
            ));
        }
    }
    
    Ok(())
}

/// Normalizes a path by trimming and ensuring it starts with /
pub fn sanitize_path(path: &str) -> String {
    let trimmed: Vec<&str> = path
        .split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    format!("/{}", trimmed.join("/"))
}

/// Validates base path against OpenAPI servers
pub fn validate_base_path(
    openapi: &OpenAPI,
    base_path: Option<&str>,
) -> Result<String, MappingError> {
    let servers = &openapi.servers;
    
    if servers.is_empty() {
        return Ok(base_path.map(sanitize_path).unwrap_or_default());
    }
    
    if servers.len() > 1 && base_path.is_none() {
        return Err(MappingError::InvalidBasePath(
            "API spec has multiple servers. Base path parameter required for disambiguation.".to_string()
        ));
    }
    
    if let Some(bp) = base_path {
        let normalized_bp = sanitize_path(bp);
        let exists = servers.iter().any(|server| {
            let server_url = &server.url;
            server_url.ends_with(&normalized_bp) || server_url.ends_with(&format!("{}/", normalized_bp))
        });
        
        if !exists {
            return Err(MappingError::InvalidBasePath(normalized_bp));
        }
        
        Ok(normalized_bp)
    } else if servers.len() == 1 {
        // Extract path from single server URL
        let server_url = &servers[0].url;
        if let Ok(url) = url::Url::parse(server_url) {
            Ok(sanitize_path(url.path()))
        } else {
            // Relative URL, treat as path
            Ok(sanitize_path(server_url))
        }
    } else {
        Ok(String::new())
    }
}

/// Generates an action definition from an OpenAPI operation
pub fn generate_action_from_operation(
    http_verb: HttpMethod,
    path_template: &str,
    operation: &Operation,
    base_path: &str,
) -> Result<ActionDefinition, MappingError> {
    // Action name: prefer operationId, fallback to "{verb} {path}"
    let action_name = operation
        .operation_id
        .clone()
        .unwrap_or_else(|| format!("{} {}", http_verb.as_str(), path_template));
    
    // Resource types: default to ["Application"], or from x-cedar extension
    let resource_types = {
        let extensions = &operation.extensions;
        if let Some(cedar_ext) = extensions.get("x-cedar") {
            if let Some(applies_to) = cedar_ext.get("appliesToResourceTypes") {
                if let Some(arr) = applies_to.as_array() {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                } else {
                    vec!["Application".to_string()]
                }
            } else {
                vec!["Application".to_string()]
            }
        } else {
            vec!["Application".to_string()]
        }
    };
    
    // Context from parameters
    let context_attributes = extract_context_from_parameters(&operation.parameters)?;
    
    // Full path with base
    let full_path = if base_path.is_empty() {
        path_template.to_string()
    } else {
        format!("{}{}", base_path, path_template)
    };
    
    Ok(ActionDefinition {
        action_name,
        http_verb,
        http_path_template: full_path,
        resource_types,
        context_attributes,
    })
}

/// Extracts context attributes from OpenAPI parameters
fn extract_context_from_parameters(
    parameters: &[ReferenceOr<Parameter>],
) -> Result<HashMap<String, AttributeType>, MappingError> {
    let mut path_params: HashMap<String, AttributeType> = HashMap::new();
    let mut query_params: HashMap<String, AttributeType> = HashMap::new();
    
    for param_ref in parameters {
        let param = match param_ref {
            ReferenceOr::Item(p) => p,
            ReferenceOr::Reference { .. } => {
                return Err(MappingError::UnsupportedParameterType(
                    "Parameter $refs are not supported".to_string()
                ));
            }
        };
        
        let param_data = match param {
            Parameter::Query { parameter_data, .. } => (parameter_data, "query"),
            Parameter::Path { parameter_data, .. } => (parameter_data, "path"),
            _ => continue, // Skip header, cookie params
        };
        
        let (data, location) = param_data;
        
        // Get schema
        let schema = match &data.format {
            ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
                ReferenceOr::Item(s) => s,
                ReferenceOr::Reference { .. } => continue,
            },
            ParameterSchemaOrContent::Content(_) => continue,
        };
        
        // Convert OpenAPI type to Cedar type
        let type_def = convert_openapi_type_to_cedar(&schema.schema_kind)?;
        
        let attr_type = AttributeType {
            type_def,
            required: if data.required { Some(true) } else { None },
        };
        
        let target_map = if location == "path" {
            &mut path_params
        } else {
            &mut query_params
        };
        
        target_map.insert(data.name.clone(), attr_type);
    }
    
    // Build context structure
    let mut context_attrs = HashMap::new();
    
    if !path_params.is_empty() {
        context_attrs.insert(
            "pathParameters".to_string(),
            AttributeType {
                type_def: TypeDefinition::Record {
                    attributes: path_params,
                },
                required: None,
            },
        );
    }
    
    if !query_params.is_empty() {
        context_attrs.insert(
            "queryStringParameters".to_string(),
            AttributeType {
                type_def: TypeDefinition::Record {
                    attributes: query_params,
                },
                required: None,
            },
        );
    }
    
    Ok(context_attrs)
}

/// Converts OpenAPI schema type to Cedar type definition
fn convert_openapi_type_to_cedar(
    schema_kind: &SchemaKind,
) -> Result<TypeDefinition, MappingError> {
    match schema_kind {
        SchemaKind::Type(OApiType::String(_)) => Ok(TypeDefinition::String),
        SchemaKind::Type(OApiType::Number(_)) | SchemaKind::Type(OApiType::Integer(_)) => {
            Ok(TypeDefinition::Long)
        }
        SchemaKind::Type(OApiType::Boolean(_)) => Ok(TypeDefinition::Boolean),
        SchemaKind::Type(OApiType::Array(arr)) => {
            let element = if let Some(items) = &arr.items {
                match items {
                    ReferenceOr::Item(schema) => {
                        convert_openapi_type_to_cedar(&schema.schema_kind)?
                    }
                    ReferenceOr::Reference { .. } => TypeDefinition::String, // Fallback
                }
            } else {
                TypeDefinition::String
            };
            
            Ok(TypeDefinition::Set {
                element: Box::new(element),
            })
        }
        SchemaKind::Type(OApiType::Object(_)) => {
            // For now, return empty record
            Ok(TypeDefinition::Record {
                attributes: HashMap::new(),
            })
        }
        _ => Err(MappingError::UnsupportedParameterType(
            "Unsupported schema type".to_string()
        )),
    }
}

/// Converts OpenAPI path template to normalized format
/// Example: /users/{id} stays as /users/{id}
pub fn normalize_path_template(path: &str) -> String {
    sanitize_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_namespace() {
        assert!(validate_namespace("App").is_ok());
        assert!(validate_namespace("MyApp").is_ok());
        assert!(validate_namespace("My::App").is_ok());
        assert!(validate_namespace("_private").is_ok());
        
        assert!(validate_namespace("if").is_err());
        assert!(validate_namespace("123App").is_err());
        assert!(validate_namespace("My-App").is_err());
        assert!(validate_namespace("My::").is_err());
    }

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("/users"), "/users");
        assert_eq!(sanitize_path("users"), "/users");
        assert_eq!(sanitize_path("/users/"), "/users");
        assert_eq!(sanitize_path("  /users/  "), "/users");
    }
}
