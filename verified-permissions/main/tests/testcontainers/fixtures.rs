//! Test fixtures and helper data

/// Basic Cedar schema for testing
pub const BASIC_SCHEMA: &str = r#"{
    "": {
        "entityTypes": {
            "User": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "email": { "type": "String" },
                        "department": { "type": "String" }
                    }
                },
                "memberOfTypes": ["Group"]
            },
            "Group": {},
            "Document": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "owner": { "type": "String" },
                        "classification": { "type": "String" }
                    }
                }
            }
        },
        "actions": {
            "view": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["Document"]
                }
            },
            "edit": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["Document"]
                }
            },
            "delete": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["Document"]
                }
            }
        }
    }
}"#;

/// Simple allow policy for testing
pub const SIMPLE_ALLOW_POLICY: &str = r#"
permit(
    principal == User::"alice",
    action == Action::"view",
    resource == Document::"doc123"
);
"#;

/// Policy with when condition
pub const POLICY_WITH_CONDITION: &str = r#"
permit(
    principal,
    action == Action::"view",
    resource
)
when {
    principal.department == "engineering"
};
"#;

/// Policy with hierarchy
pub const POLICY_WITH_HIERARCHY: &str = r#"
permit(
    principal,
    action == Action::"view",
    resource
)
when {
    principal in Group::"admins"
};
"#;

/// Forbid policy
pub const FORBID_POLICY: &str = r#"
forbid(
    principal,
    action == Action::"delete",
    resource
)
when {
    resource.classification == "confidential"
};
"#;

/// Policy template for sharing
pub const SHARE_TEMPLATE: &str = r#"
permit(
    principal == ?principal,
    action == Action::"view",
    resource == ?resource
);
"#;

/// Multi-tenancy schema with tenant_id
pub const MULTITENANT_SCHEMA: &str = r#"{
    "": {
        "entityTypes": {
            "User": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "tenant_id": { "type": "String" },
                        "email": { "type": "String" }
                    }
                }
            },
            "Document": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "tenant_id": { "type": "String" },
                        "owner": { "type": "String" }
                    }
                }
            }
        },
        "actions": {
            "view": {},
            "edit": {}
        }
    }
}"#;

/// Multi-tenancy isolation policy
pub const MULTITENANT_POLICY: &str = r#"
permit(
    principal,
    action,
    resource
)
when {
    principal.tenant_id == resource.tenant_id
};
"#;
