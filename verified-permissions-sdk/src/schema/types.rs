//! Domain types for schema generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bundle containing generated Cedar schemas in different versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaBundle {
    /// Cedar schema in v4 format (JSON)
    pub v4: String,
    /// Cedar schema in v2 format (JSON) - optional for backwards compatibility
    pub v2: Option<String>,
    /// Metadata about the generated schema
    pub metadata: SchemaMetadata,
}

/// Metadata about a generated schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    /// Namespace used in the schema
    pub namespace: String,
    /// Base path applied to routes
    pub base_path: Option<String>,
    /// Mapping type (always "SimpleRest" for this implementation)
    pub mapping_type: String,
    /// Number of actions generated
    pub action_count: usize,
    /// Number of entity types generated
    pub entity_type_count: usize,
}

/// Cedar schema JSON structure (v4 format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CedarSchemaJson {
    #[serde(flatten)]
    pub namespaces: HashMap<String, NamespaceFragment>,
}

/// A namespace fragment in the Cedar schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceFragment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "entityTypes")]
    pub entity_types: HashMap<String, EntityType>,
    pub actions: HashMap<String, ActionType>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "commonTypes")]
    pub common_types: Option<HashMap<String, CommonType>>,
}

/// Cedar entity type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub shape: TypeDefinition,
    #[serde(skip_serializing_if = "Option::is_none", rename = "memberOfTypes")]
    pub member_of_types: Option<Vec<String>>,
}

/// Cedar action type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "appliesTo")]
    pub applies_to: Option<AppliesTo>,
}

/// Defines what an action applies to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliesTo {
    #[serde(rename = "principalTypes")]
    pub principal_types: Vec<String>,
    #[serde(rename = "resourceTypes")]
    pub resource_types: Vec<String>,
    pub context: TypeDefinition,
}

/// Cedar type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TypeDefinition {
    Record {
        attributes: HashMap<String, AttributeType>,
    },
    Set {
        element: Box<TypeDefinition>,
    },
    String,
    Long,
    Boolean,
    #[serde(untagged)]
    Named {
        #[serde(rename = "type")]
        type_name: String,
    },
}

/// Cedar attribute type with optional required flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeType {
    #[serde(flatten)]
    pub type_def: TypeDefinition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Cedar common type definition
pub type CommonType = TypeDefinition;

/// Supported HTTP methods for SimpleRest mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Action definition derived from an OpenAPI operation
#[derive(Debug, Clone)]
pub struct ActionDefinition {
    pub action_name: String,
    pub http_verb: HttpMethod,
    pub http_path_template: String,
    pub resource_types: Vec<String>,
    pub context_attributes: HashMap<String, AttributeType>,
}
