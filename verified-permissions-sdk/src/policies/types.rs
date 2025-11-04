//! Types for policy generation

/// A role in the authorization system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    /// The name of the role
    pub name: String,
    /// The display name for the role
    pub display_name: String,
    /// Optional description
    pub description: Option<String>,
    /// The level of privilege (higher numbers = more privilege)
    pub privilege_level: u8,
}

impl Role {
    /// Create a new role
    pub fn new(name: String, display_name: String, privilege_level: u8) -> Self {
        Self {
            name,
            display_name,
            description: None,
            privilege_level,
        }
    }

    /// Create an admin role (highest privilege)
    pub fn admin() -> Self {
        Self::new("admin".to_string(), "Administrator".to_string(), 100)
    }

    /// Create a developer role (moderate privilege)
    pub fn developer() -> Self {
        Self::new("developer".to_string(), "Developer".to_string(), 50)
    }

    /// Create a viewer role (lowest privilege)
    pub fn viewer() -> Self {
        Self::new("viewer".to_string(), "Viewer".to_string(), 10)
    }

    /// Create a custom role with specified privilege level
    pub fn custom(name: String, display_name: String, privilege_level: u8) -> Self {
        Self::new(name, display_name, privilege_level)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// The mode for privilege generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivilegeMode {
    /// Default deny, explicit allow (RECOMMENDED for production)
    /// Only allows what is explicitly specified
    Strict,
    /// Read access by default for GET endpoints
    /// Allows read operations for public resources
    Moderate,
    /// Common CRUD patterns allowed
    /// Permits standard create, read, update, delete operations
    Permissive,
}

impl std::fmt::Display for PrivilegeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivilegeMode::Strict => write!(f, "strict"),
            PrivilegeMode::Moderate => write!(f, "moderate"),
            PrivilegeMode::Permissive => write!(f, "permissive"),
        }
    }
}

impl PrivilegeMode {
    /// Parse privilege mode from string
    pub fn parse(mode: &str) -> Result<Self, String> {
        match mode.to_lowercase().as_str() {
            "strict" => Ok(PrivilegeMode::Strict),
            "moderate" => Ok(PrivilegeMode::Moderate),
            "permissive" => Ok(PrivilegeMode::Permissive),
            _ => Err(format!(
                "Invalid privilege mode '{}'. Valid options: strict, moderate, permissive",
                mode
            )),
        }
    }
}

/// HTTP method for API endpoints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
}

impl HttpMethod {
    /// Convert to uppercase string
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
        }
    }

    /// Check if this is a read-only method
    pub fn is_read_only(&self) -> bool {
        matches!(self, HttpMethod::Get | HttpMethod::Head | HttpMethod::Options)
    }

    /// Check if this is a write method
    pub fn is_write(&self) -> bool {
        matches!(self, HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch | HttpMethod::Delete)
    }
}

/// An API endpoint from OpenAPI spec
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    /// The HTTP method
    pub method: HttpMethod,
    /// The path pattern
    pub path: String,
    /// The path template with parameters
    pub path_template: String,
    /// The operation ID if available
    pub operation_id: Option<String>,
    /// The tags for this endpoint
    pub tags: Vec<String>,
    /// Whether authentication is required
    pub requires_auth: bool,
    /// The resource type this endpoint operates on
    pub resource_type: String,
}

impl ApiEndpoint {
    /// Create a new API endpoint
    pub fn new(
        method: HttpMethod,
        path: String,
        operation_id: Option<String>,
        resource_type: String,
    ) -> Self {
        let path_template = normalize_path_template(&path);
        Self {
            method,
            path,
            path_template,
            operation_id,
            tags: Vec::new(),
            requires_auth: true,
            resource_type,
        }
    }

    /// Convert to Cedar action name
    pub fn to_action_name(&self) -> String {
        format!("{}::{}", self.method.as_str(), self.path_template)
    }
}

/// Normalize a path template for Cedar action names
fn normalize_path_template(path: &str) -> String {
    // Convert path parameters from {id} to {id}
    // Remove trailing slashes
    // Normalize double slashes
    let normalized = path
        .replace("//", "/")
        .trim_end_matches('/')
        .to_string();

    // Ensure it starts with a single slash
    if !normalized.starts_with('/') {
        format!("/{}", normalized)
    } else {
        normalized
    }
}

/// Security warning types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityWarning {
    /// Wildcard actions detected
    WildcardActions {
        policy_id: String,
        count: usize,
    },
    /// Overly permissive principals
    OverlyPermissivePrincipals {
        policy_id: String,
        principals: Vec<String>,
    },
    /// Missing default deny
    MissingDefaultDeny,
    /// High number of wildcards
    HighWildcardCount {
        count: usize,
        threshold: usize,
    },
    /// Unauthenticated endpoint
    UnauthenticatedEndpoint {
        endpoint: String,
    },
    /// Sensitive operation without proper controls
    SensitiveOperation {
        endpoint: String,
        operation: String,
    },
}

impl SecurityWarning {
    /// Get the warning message
    pub fn message(&self) -> String {
        match self {
            SecurityWarning::WildcardActions { policy_id, count } => {
                format!(
                    "Policy '{}' contains {} wildcard action(s) - violates least privilege",
                    policy_id, count
                )
            }
            SecurityWarning::OverlyPermissivePrincipals { policy_id, principals } => {
                format!(
                    "Policy '{}' has overly permissive principals: {}",
                    policy_id,
                    principals.join(", ")
                )
            }
            SecurityWarning::MissingDefaultDeny => {
                "No default deny policy found - all access is allowed by default".to_string()
            }
            SecurityWarning::HighWildcardCount { count, threshold } => {
                format!(
                    "High number of wildcards ({}) exceeds recommended threshold ({})",
                    count, threshold
                )
            }
            SecurityWarning::UnauthenticatedEndpoint { endpoint } => {
                format!(
                    "Endpoint '{}' allows unauthenticated access - review security implications",
                    endpoint
                )
            }
            SecurityWarning::SensitiveOperation { endpoint, operation } => {
                format!(
                    "Sensitive operation '{}' on endpoint '{}' requires additional security review",
                    operation, endpoint
                )
            }
        }
    }
}
