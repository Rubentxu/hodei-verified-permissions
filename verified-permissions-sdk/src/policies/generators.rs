//! Policy Generators
//!
//! Core logic for generating least privilege policies from OpenAPI specifications

use crate::policies::{
    GeneratedSchema, PolicyBundle, PolicyEffect, PolicyScope, SchemaMetadata, SchemaVersion,
    SecurityReport,
    types::{ApiEndpoint, HttpMethod, PrivilegeMode, Role},
};
use crate::schema::SchemaBundle;
use anyhow::{Context, Result};
use serde_json::Value;

/// Generator for least privilege policies from OpenAPI specifications
#[derive(Debug, Clone)]
pub struct LeastPrivilegeGenerator {
    /// The application namespace
    namespace: String,
    /// The roles to generate policies for
    roles: Vec<Role>,
    /// The privilege mode to use
    mode: PrivilegeMode,
}

impl LeastPrivilegeGenerator {
    /// Create a new least privilege generator
    pub fn new(namespace: String, roles: Vec<Role>, mode: PrivilegeMode) -> Self {
        Self {
            namespace,
            roles,
            mode,
        }
    }

    /// Generate a complete policy bundle from an OpenAPI specification
    pub fn generate_from_openapi(
        &self,
        spec_content: &str,
        schema_bundle: &SchemaBundle,
    ) -> Result<PolicyBundle> {
        // Parse OpenAPI spec
        let spec: Value =
            serde_json::from_str(spec_content).context("Failed to parse OpenAPI specification")?;

        // Extract metadata from spec
        let openapi_version = spec
            .get("openapi")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Generate schemas
        let schemas = self.generate_schemas(&spec, schema_bundle, openapi_version)?;

        // Extract endpoints from OpenAPI spec
        let endpoints = self.extract_endpoints(&spec)?;

        // Generate policies
        let policies = self.generate_policies(&endpoints)?;

        // Generate templates
        let templates = self.generate_templates(&endpoints)?;

        // Analyze security
        let security_report = SecurityReport::analyze(&policies);

        Ok(PolicyBundle {
            schemas,
            policies,
            templates,
            security_report,
        })
    }

    /// Generate schemas from OpenAPI specification
    fn generate_schemas(
        &self,
        spec: &Value,
        schema_bundle: &SchemaBundle,
        openapi_version: &str,
    ) -> Result<Vec<GeneratedSchema>> {
        let mut schemas = Vec::new();

        // Create v4 schema
        let v4_metadata = SchemaMetadata {
            namespace: self.namespace.clone(),
            mapping_type: "SimpleRest".to_string(),
            action_count: self.extract_action_count(spec)?,
            entity_type_count: self.extract_entity_type_count(spec)?,
            base_path: self.extract_base_path(spec),
            openapi_version: openapi_version.to_string(),
        };

        schemas.push(GeneratedSchema {
            namespace: self.namespace.clone(),
            content: schema_bundle.v4.clone(),
            version: SchemaVersion::V4,
            metadata: v4_metadata.clone(),
        });

        // Add v2 schema if available
        if let Some(v2_content) = &schema_bundle.v2 {
            schemas.push(GeneratedSchema {
                namespace: self.namespace.clone(),
                content: v2_content.clone(),
                version: SchemaVersion::V2,
                metadata: v4_metadata,
            });
        }

        Ok(schemas)
    }

    /// Extract all endpoints from OpenAPI spec
    fn extract_endpoints(&self, spec: &Value) -> Result<Vec<ApiEndpoint>> {
        let mut endpoints = Vec::new();
        let paths = spec
            .get("paths")
            .and_then(|p| p.as_object())
            .context("OpenAPI spec must have a 'paths' object")?;

        for (path, operations) in paths {
            for (method, operation) in operations.as_object().unwrap() {
                if let Some(method_enum) = HttpMethod::from_str(method) {
                    let operation_id = operation
                        .get("operationId")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let tags: Vec<String> = operation
                        .get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    // Determine resource type from path or tags
                    let resource_type = self.extract_resource_type(path, &tags);

                    // Check if authentication is required
                    let requires_auth = !self.has_security_requirements(operation, "none");

                    let endpoint = ApiEndpoint {
                        method: method_enum,
                        path: path.clone(),
                        path_template: normalize_path_template(path),
                        operation_id,
                        tags,
                        requires_auth,
                        resource_type,
                    };

                    endpoints.push(endpoint);
                }
            }
        }

        Ok(endpoints)
    }

    /// Extract resource type from path or tags
    fn extract_resource_type(&self, path: &str, tags: &[String]) -> String {
        // Try to extract from tags first
        if let Some(tag) = tags.first() {
            return self.sanitize_resource_name(tag);
        }

        // Try to extract from path
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();

        if parts.is_empty() {
            return "Resource".to_string();
        }

        // Skip versioning components
        let mut candidate = parts
            .iter()
            .find(|&&p| !p.starts_with('v') && !p.chars().all(|c| c.is_numeric()));

        // If not found, use the first non-empty part
        if candidate.is_none() && !parts.is_empty() {
            candidate = parts.first();
        }

        if let Some(part) = candidate {
            self.sanitize_resource_name(part)
        } else {
            "Resource".to_string()
        }
    }

    /// Sanitize a string to be a valid Cedar type name
    fn sanitize_resource_name(&self, name: &str) -> String {
        // Replace invalid characters
        let sanitized = name
            .chars()
            .map(|c| match c {
                '-' => '_',
                _ => c,
            })
            .collect::<String>();

        // Ensure it starts with a capital letter
        if sanitized.is_empty() {
            "Resource".to_string()
        } else {
            let mut chars = sanitized.chars();
            if let Some(first) = chars.next() {
                first.to_uppercase().collect::<String>() + chars.as_str()
            } else {
                "Resource".to_string()
            }
        }
    }

    /// Generate policies for all endpoints
    fn generate_policies(&self, endpoints: &[ApiEndpoint]) -> Result<Vec<super::CedarPolicy>> {
        let mut policies = Vec::new();

        // Group endpoints by resource type
        let mut endpoints_by_resource: std::collections::HashMap<String, Vec<ApiEndpoint>> =
            std::collections::HashMap::new();

        for endpoint in endpoints {
            endpoints_by_resource
                .entry(endpoint.resource_type.clone())
                .or_insert_with(Vec::new)
                .push(endpoint.clone());
        }

        // Generate policies for each resource type
        for (resource_type, resource_endpoints) in endpoints_by_resource {
            let resource_policies =
                self.generate_resource_policies(&resource_type, resource_endpoints)?;
            policies.extend(resource_policies);
        }

        // Add global policies
        let global_policies = self.generate_global_policies(endpoints)?;
        policies.extend(global_policies);

        Ok(policies)
    }

    /// Generate policies for a specific resource type
    fn generate_resource_policies(
        &self,
        resource_type: &str,
        endpoints: Vec<ApiEndpoint>,
    ) -> Result<Vec<super::CedarPolicy>> {
        let mut policies = Vec::new();

        for role in &self.roles {
            for endpoint in &endpoints {
                // Check if role should have access based on mode and endpoint
                if self.should_allow_access(role, endpoint) {
                    let policy = self.create_endpoint_policy(role, endpoint, resource_type)?;
                    policies.push(policy);
                }
            }
        }

        Ok(policies)
    }

    /// Determine if a role should have access to an endpoint
    fn should_allow_access(&self, role: &Role, endpoint: &ApiEndpoint) -> bool {
        match self.mode {
            PrivilegeMode::Strict => {
                // Strict mode: only allow based on role privilege level
                if endpoint.method.is_read_only() {
                    role.privilege_level >= 10 // Read access for all authenticated users
                } else if endpoint.method.is_write() {
                    role.privilege_level >= 50 // Write access for privileged users
                } else {
                    false
                }
            }
            PrivilegeMode::Moderate => {
                // Moderate mode: allow read for everyone, write for privileged
                if endpoint.method.is_read_only() {
                    true
                } else if endpoint.method.is_write() {
                    role.privilege_level >= 50
                } else {
                    false
                }
            }
            PrivilegeMode::Permissive => {
                // Permissive mode: allow most operations based on role level
                if endpoint.method.is_read_only() {
                    true
                } else {
                    role.privilege_level >= 30
                }
            }
        }
    }

    /// Create a policy for a specific endpoint
    fn create_endpoint_policy(
        &self,
        role: &Role,
        endpoint: &ApiEndpoint,
        resource_type: &str,
    ) -> Result<super::CedarPolicy> {
        let action_name = endpoint.to_action_name();

        Ok(super::CedarPolicy {
            id: format!(
                "{}_{}_{}",
                resource_type,
                role.name,
                endpoint.method.as_str()
            ),
            description: format!(
                "Allow {} role to {} {} resources",
                role.display_name,
                endpoint.method.as_str().to_lowercase(),
                resource_type
            ),
            content: format!(
                r#"// Allow {} role to {} {} resources on {}
permit(
    principal in {}::UserGroup::"{}",
    action in [{}::Action::"{}"],
    resource in {}::{}::*
);"#,
                role.display_name,
                endpoint.method.as_str().to_lowercase(),
                resource_type,
                endpoint.path,
                self.namespace,
                role.name,
                self.namespace,
                action_name,
                self.namespace,
                resource_type
            ),
            effect: PolicyEffect::Permit,
            scope: PolicyScope::Role(role.name.clone()),
        })
    }

    /// Generate global policies (default deny, etc.)
    fn generate_global_policies(
        &self,
        _endpoints: &[ApiEndpoint],
    ) -> Result<Vec<super::CedarPolicy>> {
        let mut policies = Vec::new();

        // Add default deny policy
        policies.push(super::CedarPolicy {
            id: "default_deny".to_string(),
            description: "Default deny policy - explicit deny all access".to_string(),
            content: format!(
                r#"// Default deny policy - explicit deny all access not covered by other policies
forbid(
    principal,
    action,
    resource
);"#
            ),
            effect: PolicyEffect::Forbid,
            scope: PolicyScope::Global,
        });

        // Add OPTIONS policy for CORS (read-only for all roles)
        policies.push(super::CedarPolicy {
            id: "cors_options".to_string(),
            description: "Allow OPTIONS requests for CORS preflight".to_string(),
            content: format!(
                r#"// Allow OPTIONS requests for CORS preflight
permit(
    principal,
    action in [{}::Action::"OPTIONS::*"],
    resource
);"#,
                self.namespace
            ),
            effect: PolicyEffect::Permit,
            scope: PolicyScope::Global,
        });

        Ok(policies)
    }

    /// Generate policy templates
    fn generate_templates(&self, endpoints: &[ApiEndpoint]) -> Result<Vec<super::PolicyTemplate>> {
        use crate::policies::templates::TemplateFactory;

        let mut templates = Vec::new();

        // Group endpoints by resource type
        let mut resources: std::collections::HashSet<String> = std::collections::HashSet::new();
        for endpoint in endpoints {
            resources.insert(endpoint.resource_type.clone());
        }

        // Create CRUD templates for each resource type
        for resource_type in resources {
            templates.push(TemplateFactory::crud(
                self.namespace.clone(),
                resource_type,
                self.roles.clone(),
                self.mode.clone(),
            ));
        }

        Ok(templates)
    }

    /// Check if security requirements include "none"
    fn has_security_requirements(&self, operation: &Value, _security_type: &str) -> bool {
        // Simplified: if operation has security field and it's an empty array, no auth required
        // In a real implementation, this would be more sophisticated
        if let Some(security) = operation.get("security") {
            return !security.as_array().map_or(false, |arr| arr.is_empty());
        }
        true // Default: requires auth
    }

    /// Extract action count from OpenAPI spec
    fn extract_action_count(&self, spec: &Value) -> Result<usize> {
        let paths = spec.get("paths").and_then(|p| p.as_object());
        let mut count = 0;

        if let Some(paths) = paths {
            for operations in paths.values() {
                if let Some(obj) = operations.as_object() {
                    count += obj.len();
                }
            }
        }

        Ok(count)
    }

    /// Extract entity type count from OpenAPI spec
    fn extract_entity_type_count(&self, spec: &Value) -> Result<usize> {
        let schemas = spec
            .get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(|s| s.as_object());
        Ok(schemas.map_or(0, |s| s.len()))
    }

    /// Extract base path from OpenAPI spec
    fn extract_base_path(&self, spec: &Value) -> Option<String> {
        spec.get("servers")
            .and_then(|s| s.as_array())
            .and_then(|servers| servers.first())
            .and_then(|server| server.get("url"))
            .and_then(|url| url.as_str())
            .map(|s| s.to_string())
    }
}

/// Normalize a path template
fn normalize_path_template(path: &str) -> String {
    let normalized = path.replace("//", "/").trim_end_matches('/').to_string();
    if !normalized.starts_with('/') {
        format!("/{}", normalized)
    } else {
        normalized
    }
}

impl HttpMethod {
    /// Convert string to HttpMethod
    pub fn from_str(method: &str) -> Option<HttpMethod> {
        match method.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::Get),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            "PATCH" => Some(HttpMethod::Patch),
            "DELETE" => Some(HttpMethod::Delete),
            "HEAD" => Some(HttpMethod::Head),
            "OPTIONS" => Some(HttpMethod::Options),
            "TRACE" => Some(HttpMethod::Trace),
            _ => None,
        }
    }
}
