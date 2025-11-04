//! Schema serialization utilities

use super::types::*;
use std::error::Error;

/// Serialize a Cedar schema to v4 JSON format
pub fn serialize_schema_v4(schema: &CedarSchemaJson) -> Result<String, Box<dyn Error>> {
    serde_json::to_string_pretty(schema).map_err(|e| e.into())
}

/// Serialize a Cedar schema to v2 JSON format (not yet implemented)
pub fn serialize_schema_v2(_schema: &CedarSchemaJson) -> Result<String, Box<dyn Error>> {
    Err("v2 serialization not yet implemented".into())
}
