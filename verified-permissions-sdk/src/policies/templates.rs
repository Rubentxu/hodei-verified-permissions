//! Policy Templates
//!
//! Pre-built policy templates for common authorization patterns

use super::types::{PrivilegeMode, Role};
use crate::policies::{CedarPolicy, PolicyEffect, PolicyScope};

/// A policy template
#[derive(Debug, Clone)]
pub enum PolicyTemplate {
    /// Standard CRUD operations on a resource
    Crud {
        namespace: String,
        resource_type: String,
        roles: Vec<Role>,
        mode: PrivilegeMode,
    },
    /// Read-only access to a resource
    ReadOnly {
        namespace: String,
        resource_type: String,
        roles: Vec<Role>,
    },
    /// Custom action with specific parameters
    Custom {
        namespace: String,
        action: String,
        principal: String,
        resource: String,
        effect: PolicyEffect,
    },
    /// Admin access to everything
    AdminAccess { namespace: String, admin_role: Role },
    /// Default deny policy (explicit deny all)
    DefaultDeny { namespace: String },
    /// Role-based access with specific actions
    RoleBased {
        namespace: String,
        role: Role,
        actions: Vec<String>,
    },
}

impl PolicyTemplate {
    /// Generate Cedar policies from this template
    pub fn generate_policies(&self) -> Vec<CedarPolicy> {
        match self {
            PolicyTemplate::Crud {
                namespace,
                resource_type,
                roles,
                mode,
            } => self.generate_crud_policies(namespace, resource_type, roles, mode),
            PolicyTemplate::ReadOnly {
                namespace,
                resource_type,
                roles,
            } => self.generate_readonly_policies(namespace, resource_type, roles),
            PolicyTemplate::Custom {
                namespace,
                action,
                principal,
                resource,
                effect,
            } => self.generate_custom_policy(namespace, action, principal, resource, effect),
            PolicyTemplate::AdminAccess {
                namespace,
                admin_role,
            } => self.generate_admin_policy(namespace, admin_role),
            PolicyTemplate::DefaultDeny { namespace } => self.generate_default_deny(namespace),
            PolicyTemplate::RoleBased {
                namespace,
                role,
                actions,
            } => self.generate_role_based_policy(namespace, role, actions),
        }
    }

    /// Generate CRUD policies for different roles
    fn generate_crud_policies(
        &self,
        namespace: &str,
        resource_type: &str,
        roles: &[Role],
        mode: &PrivilegeMode,
    ) -> Vec<CedarPolicy> {
        let mut policies = Vec::new();

        // Create role-based policies
        for role in roles {
            let mut role_policies = match mode {
                PrivilegeMode::Strict => self.strict_crud_for_role(namespace, resource_type, role),
                PrivilegeMode::Moderate => {
                    self.moderate_crud_for_role(namespace, resource_type, role)
                }
                PrivilegeMode::Permissive => {
                    self.permissive_crud_for_role(namespace, resource_type, role)
                }
            };
            policies.append(&mut role_policies);
        }

        policies
    }

    /// Generate strict CRUD policies (default deny, explicit allow)
    fn strict_crud_for_role(
        &self,
        namespace: &str,
        resource_type: &str,
        role: &Role,
    ) -> Vec<CedarPolicy> {
        let mut policies = Vec::new();

        // Read access - allowed for all roles
        if role.privilege_level >= 10 {
            policies.push(CedarPolicy {
                id: format!("read_{}_{}", resource_type, role.name),
                description: format!(
                    "Allow {} role to read {} resources",
                    role.name, resource_type
                ),
                content: format!(
                    r#"// Allow {} role to read {} resources
permit(
    principal in {}::UserGroup::"{}",
    action in [{}::Action::"GET::{}/*", {}::Action::"HEAD::{}/*"],
    resource in {}::{}::*
);"#,
                    role.display_name,
                    resource_type,
                    namespace,
                    role.name,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type
                ),
                effect: PolicyEffect::Permit,
                scope: PolicyScope::Role(role.name.clone()),
            });
        }

        // Write access - only for privileged roles
        if role.privilege_level >= 50 {
            policies.push(CedarPolicy {
                id: format!("write_{}_{}", resource_type, role.name),
                description: format!(
                    "Allow {} role to modify {} resources",
                    role.name, resource_type
                ),
                content: format!(
                    r#"// Allow {} role to modify {} resources
permit(
    principal in {}::UserGroup::"{}",
    action in [{}::Action::"POST::{}/*", {}::Action::"PUT::{}/*", {}::Action::"PATCH::{}/*"],
    resource in {}::{}::*
);"#,
                    role.display_name,
                    resource_type,
                    namespace,
                    role.name,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type
                ),
                effect: PolicyEffect::Permit,
                scope: PolicyScope::Role(role.name.clone()),
            });

            policies.push(CedarPolicy {
                id: format!("delete_{}_{}", resource_type, role.name),
                description: format!(
                    "Allow {} role to delete {} resources",
                    role.name, resource_type
                ),
                content: format!(
                    r#"// Allow {} role to delete {} resources
permit(
    principal in {}::UserGroup::"{}",
    action in [{}::Action::"DELETE::{}/*"],
    resource in {}::{}::*
);"#,
                    role.display_name,
                    resource_type,
                    namespace,
                    role.name,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type
                ),
                effect: PolicyEffect::Permit,
                scope: PolicyScope::Role(role.name.clone()),
            });
        }

        policies
    }

    /// Generate moderate CRUD policies (read by default)
    fn moderate_crud_for_role(
        &self,
        namespace: &str,
        resource_type: &str,
        role: &Role,
    ) -> Vec<CedarPolicy> {
        // Moderate mode allows read for everyone, write for privileged roles
        let mut policies = self.strict_crud_for_role(namespace, resource_type, role);

        // Add additional read permissions
        if role.privilege_level >= 10 {
            policies.push(CedarPolicy {
                id: format!("options_{}_{}", resource_type, role.name),
                description: format!(
                    "Allow {} role to OPTIONS on {} resources",
                    role.name, resource_type
                ),
                content: format!(
                    r#"// Allow {} role to OPTIONS on {} resources
permit(
    principal in {}::UserGroup::"{}",
    action in [{}::Action::"OPTIONS::{}/*"],
    resource in {}::{}::*
);"#,
                    role.display_name,
                    resource_type,
                    namespace,
                    role.name,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type
                ),
                effect: PolicyEffect::Permit,
                scope: PolicyScope::Role(role.name.clone()),
            });
        }

        policies
    }

    /// Generate permissive CRUD policies (common CRUD patterns)
    fn permissive_crud_for_role(
        &self,
        namespace: &str,
        resource_type: &str,
        role: &Role,
    ) -> Vec<CedarPolicy> {
        // Permissive mode allows more operations by default
        let mut policies = self.moderate_crud_for_role(namespace, resource_type, role);

        // Add wildcard GET for all roles
        if role.privilege_level >= 10 {
            policies.push(CedarPolicy {
                id: format!("read_all_{}_{}", resource_type, role.name),
                description: format!(
                    "Allow {} role to read any {} resource",
                    role.name, resource_type
                ),
                content: format!(
                    r#"// Allow {} role to read any {} resource (permissive mode)
permit(
    principal in {}::UserGroup::"{}",
    action in [{}::Action::"GET::*"],
    resource in {}::{}::*
);"#,
                    role.display_name,
                    resource_type,
                    namespace,
                    role.name,
                    namespace,
                    namespace,
                    resource_type
                ),
                effect: PolicyEffect::Permit,
                scope: PolicyScope::Role(role.name.clone()),
            });
        }

        policies
    }

    /// Generate read-only policies
    fn generate_readonly_policies(
        &self,
        namespace: &str,
        resource_type: &str,
        roles: &[Role],
    ) -> Vec<CedarPolicy> {
        let mut policies = Vec::new();

        for role in roles {
            policies.push(CedarPolicy {
                id: format!("readonly_{}_{}", resource_type, role.name),
                description: format!(
                    "Allow {} role to read-only access to {} resources",
                    role.name, resource_type
                ),
                content: format!(
                    r#"// Allow {} role to read-only access to {} resources
permit(
    principal in {}::UserGroup::"{}",
    action in [{}::Action::"GET::{}/*", {}::Action::"HEAD::{}/*"],
    resource in {}::{}::*
);"#,
                    role.display_name,
                    resource_type,
                    namespace,
                    role.name,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type,
                    namespace,
                    resource_type
                ),
                effect: PolicyEffect::Permit,
                scope: PolicyScope::Role(role.name.clone()),
            });
        }

        policies
    }

    /// Generate custom policy
    fn generate_custom_policy(
        &self,
        namespace: &str,
        action: &str,
        principal: &str,
        resource: &str,
        effect: &PolicyEffect,
    ) -> Vec<CedarPolicy> {
        let effect_str = match effect {
            PolicyEffect::Permit => "permit",
            PolicyEffect::Forbid => "forbid",
        };

        vec![CedarPolicy {
            id: format!("custom_{}", action.replace("::", "_")),
            description: format!("Custom policy for action {}", action),
            content: format!(
                r#"// Custom policy
{}(
    principal {}::UserGroup::"{}",
    action in [{}::Action::"{}"],
    resource {}::{}::*
);"#,
                effect_str, namespace, principal, namespace, action, namespace, resource
            ),
            effect: effect.clone(),
            scope: PolicyScope::Action(action.to_string()),
        }]
    }

    /// Generate admin policy (full access)
    fn generate_admin_policy(&self, namespace: &str, admin_role: &Role) -> Vec<CedarPolicy> {
        vec![CedarPolicy {
            id: format!("admin_full_access_{}", admin_role.name),
            description: format!("Full access for {} role", admin_role.display_name),
            content: format!(
                r#"// Full access for {} role
permit(
    principal in {}::UserGroup::"{}",
    action,
    resource
);"#,
                admin_role.display_name, namespace, admin_role.name
            ),
            effect: PolicyEffect::Permit,
            scope: PolicyScope::Role(admin_role.name.clone()),
        }]
    }

    /// Generate default deny policy
    fn generate_default_deny(&self, _namespace: &str) -> Vec<CedarPolicy> {
        vec![CedarPolicy {
            id: "default_deny".to_string(),
            description: "Default deny policy (explicit deny all access)".to_string(),
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
        }]
    }

    /// Generate role-based policy with specific actions
    fn generate_role_based_policy(
        &self,
        namespace: &str,
        role: &Role,
        actions: &[String],
    ) -> Vec<CedarPolicy> {
        let actions_str = actions
            .iter()
            .map(|a| format!("        {}::Action::\"{}\"", namespace, a))
            .collect::<Vec<_>>()
            .join(",\n");

        vec![CedarPolicy {
            id: format!("role_based_{}", role.name),
            description: format!(
                "Role-based access for {} with specific actions",
                role.display_name
            ),
            content: format!(
                r#"// Role-based access for {} with specific actions
permit(
    principal in {}::UserGroup::"{}",
    action in [
{}
    ],
    resource
);"#,
                role.display_name, namespace, role.name, actions_str
            ),
            effect: PolicyEffect::Permit,
            scope: PolicyScope::Role(role.name.clone()),
        }]
    }
}

/// Helper functions for creating common templates
pub struct TemplateFactory;

impl TemplateFactory {
    /// Create a CRUD template for the given resource and roles
    pub fn crud(
        namespace: String,
        resource_type: String,
        roles: Vec<Role>,
        mode: PrivilegeMode,
    ) -> PolicyTemplate {
        PolicyTemplate::Crud {
            namespace,
            resource_type,
            roles,
            mode,
        }
    }

    /// Create a read-only template for the given resource and roles
    pub fn readonly(namespace: String, resource_type: String, roles: Vec<Role>) -> PolicyTemplate {
        PolicyTemplate::ReadOnly {
            namespace,
            resource_type,
            roles,
        }
    }

    /// Create an admin access template
    pub fn admin(namespace: String, admin_role: Role) -> PolicyTemplate {
        PolicyTemplate::AdminAccess {
            namespace,
            admin_role,
        }
    }

    /// Create a default deny template
    pub fn default_deny(namespace: String) -> PolicyTemplate {
        PolicyTemplate::DefaultDeny { namespace }
    }

    /// Create a role-based template
    pub fn role_based(namespace: String, role: Role, actions: Vec<String>) -> PolicyTemplate {
        PolicyTemplate::RoleBased {
            namespace,
            role,
            actions,
        }
    }

    /// Create a custom template
    pub fn custom(
        namespace: String,
        action: String,
        principal: String,
        resource: String,
        effect: PolicyEffect,
    ) -> PolicyTemplate {
        PolicyTemplate::Custom {
            namespace,
            action,
            principal,
            resource,
            effect,
        }
    }
}
