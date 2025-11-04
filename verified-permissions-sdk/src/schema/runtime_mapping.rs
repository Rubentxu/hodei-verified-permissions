//! Runtime mapping for SimpleRest pattern
//!
//! This module provides runtime route matching and action resolution
//! based on Cedar schemas with SimpleRest annotations.

use super::types::{CedarSchemaJson, HttpMethod};
use matchit::{Match, Router};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MappingError {
    #[error("Failed to parse schema: {0}")]
    SchemaParseError(String),
    
    #[error("Invalid schema format: {0}")]
    InvalidSchema(String),
    
    #[error("No mapping found for {method} {path}")]
    NoMappingFound { method: String, path: String },
    
    #[error("Route conflict: {0}")]
    RouteConflict(String),
}

/// Route information resolved from a request
#[derive(Debug, Clone)]
pub struct ResolvedRoute {
    /// Cedar action name
    pub action_name: String,
    /// Resource type(s) this action applies to
    pub resource_types: Vec<String>,
    /// Path template that matched
    pub path_template: String,
    /// Path parameters extracted from the URL
    pub path_params: HashMap<String, String>,
}

/// Action route metadata
#[derive(Debug, Clone)]
struct ActionRoute {
    /// Action name in Cedar
    action_name: String,
    /// Resource types this action applies to
    resource_types: Vec<String>,
    /// Original path template from schema
    path_template: String,
}

/// SimpleRest mapping for runtime route resolution
///
/// This struct loads a Cedar schema with SimpleRest annotations and
/// builds efficient route matchers per HTTP method for fast lookup.
pub struct SimpleRestMapping {
    /// Namespace from the schema
    namespace: String,
    /// Route matchers by HTTP method
    routers: HashMap<HttpMethod, Router<ActionRoute>>,
}

impl SimpleRestMapping {
    /// Create a new SimpleRestMapping from a Cedar schema JSON
    pub fn from_schema_json(schema_json: &str) -> Result<Self, MappingError> {
        let schema: CedarSchemaJson = serde_json::from_str(schema_json)
            .map_err(|e| MappingError::SchemaParseError(e.to_string()))?;
        
        Self::from_schema(schema)
    }
    
    /// Create a new SimpleRestMapping from a parsed Cedar schema
    pub fn from_schema(schema: CedarSchemaJson) -> Result<Self, MappingError> {
        // Get the first (and should be only) namespace
        let (namespace, namespace_fragment) = schema.namespaces.iter().next()
            .ok_or_else(|| MappingError::InvalidSchema("No namespace found in schema".to_string()))?;
        
        // Verify it's a SimpleRest schema
        if let Some(annotations) = &namespace_fragment.annotations {
            if annotations.get("mappingType").and_then(|v| v.as_str()) != Some("SimpleRest") {
                return Err(MappingError::InvalidSchema(
                    "Schema is not marked as SimpleRest".to_string()
                ));
            }
        } else {
            return Err(MappingError::InvalidSchema(
                "Schema missing mappingType annotation".to_string()
            ));
        }
        
        // Collect routes per HTTP method
        let mut routes_by_method: HashMap<HttpMethod, Vec<(String, ActionRoute)>> = HashMap::new();
        
        for (action_name, action_type) in &namespace_fragment.actions {
            // Extract annotations
            let annotations = action_type.annotations.as_ref()
                .ok_or_else(|| MappingError::InvalidSchema(
                    format!("Action '{}' missing annotations", action_name)
                ))?;
            
            let http_verb = annotations.get("httpVerb")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MappingError::InvalidSchema(
                    format!("Action '{}' missing httpVerb annotation", action_name)
                ))?;
            
            let http_path_template = annotations.get("httpPathTemplate")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MappingError::InvalidSchema(
                    format!("Action '{}' missing httpPathTemplate annotation", action_name)
                ))?;
            
            // Parse HTTP method
            let method = match http_verb.to_lowercase().as_str() {
                "get" => HttpMethod::Get,
                "post" => HttpMethod::Post,
                "put" => HttpMethod::Put,
                "patch" => HttpMethod::Patch,
                "delete" => HttpMethod::Delete,
                _ => continue, // Skip unsupported methods
            };
            
            // Get resource types
            let resource_types = if let Some(applies_to) = &action_type.applies_to {
                applies_to.resource_types.clone()
            } else {
                vec!["Application".to_string()]
            };
            
            // Convert OpenAPI path template to matchit format
            // OpenAPI: /documents/{id} -> matchit: /documents/:id
            let matchit_path = convert_path_template(http_path_template);
            
            let route = ActionRoute {
                action_name: action_name.clone(),
                resource_types,
                path_template: http_path_template.to_string(),
            };
            
            // Add route to the collection for this method
            routes_by_method.entry(method)
                .or_insert_with(Vec::new)
                .push((matchit_path, route));
        }
        
        // Build routers from collected routes
        let mut routers = HashMap::new();
        for (method, routes) in routes_by_method {
            let mut router = Router::new();
            for (path, route) in routes {
                if let Err(e) = router.insert(&path, route) {
                    return Err(MappingError::RouteConflict(
                        format!("{:?} {}: {}", method, path, e)
                    ));
                }
            }
            routers.insert(method, router);
        }
        
        Ok(Self {
            namespace: namespace.clone(),
            routers,
        })
    }
    
    /// Resolve a route from an HTTP method and path
    pub fn resolve(&self, method: &http::Method, path: &str) -> Result<ResolvedRoute, MappingError> {
        // Convert http::Method to our HttpMethod
        let http_method = match method.as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "PATCH" => HttpMethod::Patch,
            "DELETE" => HttpMethod::Delete,
            _ => return Err(MappingError::NoMappingFound {
                method: method.to_string(),
                path: path.to_string(),
            }),
        };
        
        // Get router for this method
        let router = self.routers.get(&http_method)
            .ok_or_else(|| MappingError::NoMappingFound {
                method: method.to_string(),
                path: path.to_string(),
            })?;
        
        // Match the path
        let matched: Match<&ActionRoute> = router.at(path)
            .map_err(|_| MappingError::NoMappingFound {
                method: method.to_string(),
                path: path.to_string(),
            })?;
        
        // Extract path parameters
        let mut path_params = HashMap::new();
        for (key, value) in matched.params.iter() {
            path_params.insert(key.to_string(), value.to_string());
        }
        
        Ok(ResolvedRoute {
            action_name: matched.value.action_name.clone(),
            resource_types: matched.value.resource_types.clone(),
            path_template: matched.value.path_template.clone(),
            path_params,
        })
    }
    
    /// Get the namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
    
    /// Get all supported methods
    pub fn supported_methods(&self) -> Vec<HttpMethod> {
        self.routers.keys().copied().collect()
    }
    
    /// Check if a method is supported
    pub fn supports_method(&self, method: &HttpMethod) -> bool {
        self.routers.contains_key(method)
    }
}

/// Convert OpenAPI path template to matchit format
/// In matchit 0.8+, the format is the same: /documents/{id}
/// So we just return the path as-is
fn convert_path_template(openapi_path: &str) -> String {
    openapi_path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_path_template() {
        // In matchit 0.8+, we keep the OpenAPI format with braces
        assert_eq!(convert_path_template("/documents/{id}"), "/documents/{id}");
        assert_eq!(
            convert_path_template("/documents/{id}/comments/{commentId}"),
            "/documents/{id}/comments/{commentId}"
        );
        assert_eq!(convert_path_template("/users"), "/users");
    }
    
    #[test]
    fn test_matchit_basic() {
        // Test without parameters first
        let mut router1 = Router::new();
        router1.insert("/documents", "list_docs").unwrap();
        let matched1 = router1.at("/documents");
        eprintln!("Match result for /documents: {:?}", matched1);
        
        // Test with colon syntax
        let mut router2 = Router::new();
        router2.insert("/documents/:id", "get_doc_colon").unwrap();
        let match2 = router2.at("/documents/123");
        eprintln!("Match with :id: {:?}", match2);
        
        // Test with brace syntax
        let mut router3 = Router::new();
        router3.insert("/documents/{id}", "get_doc_brace").unwrap();
        let match3 = router3.at("/documents/123");
        eprintln!("Match with {{id}}: {:?}", match3);
        
        // Test with asterisk
        let mut router4 = Router::new();
        router4.insert("/documents/*id", "get_doc_asterisk").unwrap();
        let match4 = router4.at("/documents/123");
        eprintln!("Match with *id: {:?}", match4);
    }

    #[test]
    fn test_simple_rest_mapping() {
        let schema_json = r#"{
            "TestApp": {
                "annotations": {
                    "mappingType": "SimpleRest"
                },
                "entityTypes": {
                    "User": {
                        "shape": { "type": "Record", "attributes": {} }
                    },
                    "Document": {
                        "shape": { "type": "Record", "attributes": {} }
                    }
                },
                "actions": {
                    "getDocument": {
                        "annotations": {
                            "httpVerb": "get",
                            "httpPathTemplate": "/documents/{id}"
                        },
                        "appliesTo": {
                            "principalTypes": ["User"],
                            "resourceTypes": ["Document"],
                            "context": { "type": "Record", "attributes": {} }
                        }
                    }
                }
            }
        }"#;

        let mapping = SimpleRestMapping::from_schema_json(schema_json);
        if let Err(e) = &mapping {
            eprintln!("Error creating mapping: {:?}", e);
        }
        let mapping = mapping.unwrap();
        
        assert_eq!(mapping.namespace(), "TestApp");
        assert!(mapping.supports_method(&HttpMethod::Get));
        
        let method = http::Method::GET;
        let resolved = mapping.resolve(&method, "/documents/123");
        if let Err(e) = &resolved {
            eprintln!("Error resolving: {:?}", e);
            eprintln!("Routers: {:?}", mapping.routers.keys().collect::<Vec<_>>());
        }
        let resolved = resolved.unwrap();
        
        assert_eq!(resolved.action_name, "getDocument");
        assert_eq!(resolved.resource_types, vec!["Document"]);
        assert_eq!(resolved.path_params.get("id"), Some(&"123".to_string()));
    }
}
