//! # Generador de Configuración para Pet Store
//! 
//! Esta herramienta genera automáticamente:
//! 1. Schema Cedar desde openapi.json
//! 2. Policies por defecto para cada rol (admin, vet, customer)
//! 
//! Uso: cargo run --bin generate_config

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct OpenApiSpec {
    paths: HashMap<String, PathItem>,
    components: Option<Components>,
}

#[derive(Debug, Deserialize)]
struct PathItem {
    #[serde(flatten)]
    methods: HashMap<String, Operation>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    operation_id: Option<String>,
    summary: Option<String>,
    #[serde(default)]
    parameters: Vec<Value>,
    #[serde(flatten)]
    extensions: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct Components {
    schemas: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
struct CedarSchema {
    #[serde(flatten)]
    namespaces: HashMap<String, Namespace>,
}

#[derive(Debug, Serialize)]
struct Namespace {
    entityTypes: HashMap<String, EntityType>,
    actions: HashMap<String, Action>,
}

#[derive(Debug, Serialize)]
struct EntityType {
    #[serde(skip_serializing_if = "Option::is_none")]
    memberOfTypes: Option<Vec<String>>,
    shape: Shape,
}

#[derive(Debug, Serialize)]
struct Shape {
    #[serde(rename = "type")]
    shape_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<String, Attribute>>,
}

#[derive(Debug, Serialize)]
struct Attribute {
    #[serde(rename = "type")]
    attr_type: String,
}

#[derive(Debug, Serialize)]
struct Action {
    appliesTo: AppliesTo,
}

#[derive(Debug, Serialize)]
struct AppliesTo {
    principalTypes: Vec<String>,
    resourceTypes: Vec<String>,
}

fn main() -> Result<()> {
    println!("🚀 Generando configuración para Pet Store...\n");

    // 1. Leer OpenAPI spec
    let openapi_content = fs::read_to_string("openapi.json")?;
    let openapi: OpenApiSpec = serde_json::from_str(&openapi_content)?;

    // 2. Generar Schema Cedar
    println!("📋 Generando schema Cedar...");
    let schema = generate_cedar_schema(&openapi)?;
    let schema_json = serde_json::to_string_pretty(&schema)?;
    fs::write("config/schema.json", schema_json)?;
    println!("   ✅ Schema guardado en config/schema.json\n");

    // 3. Generar policies por defecto
    println!("🛡️  Generando policies por defecto...");
    generate_default_policies(&openapi)?;
    println!("   ✅ Policies guardadas en config/policies/\n");

    // 4. Generar script de setup
    println!("📜 Generando script de setup...");
    generate_setup_script()?;
    println!("   ✅ Script guardado en config/setup.sh\n");

    println!("✨ Configuración generada exitosamente!");
    println!("\nPróximos pasos:");
    println!("1. Asegúrate de que Hodei AVP está corriendo");
    println!("2. Ejecuta: bash config/setup.sh");
    println!("3. Inicia la app: cargo run");

    Ok(())
}

fn generate_cedar_schema(openapi: &OpenApiSpec) -> Result<CedarSchema> {
    let mut namespace = Namespace {
        entityTypes: HashMap::new(),
        actions: HashMap::new(),
    };

    // Agregar entity types básicos
    namespace.entityTypes.insert("User".to_string(), EntityType {
        memberOfTypes: Some(vec!["Group".to_string()]),
        shape: Shape {
            shape_type: "Record".to_string(),
            attributes: Some({
                let mut attrs = HashMap::new();
                attrs.insert("username".to_string(), Attribute { attr_type: "String".to_string() });
                attrs.insert("email".to_string(), Attribute { attr_type: "String".to_string() });
                attrs.insert("role".to_string(), Attribute { attr_type: "String".to_string() });
                attrs
            }),
        },
    });

    namespace.entityTypes.insert("Group".to_string(), EntityType {
        memberOfTypes: None,
        shape: Shape {
            shape_type: "Record".to_string(),
            attributes: Some(HashMap::new()),
        },
    });

    // Extraer entity types de components
    if let Some(components) = &openapi.components {
        if let Some(schemas) = &components.schemas {
            for (name, schema) in schemas {
                if let Ok(schema_obj) = serde_json::from_value::<HashMap<String, Value>>(schema.clone()) {
                    if let Some(properties) = schema_obj.get("properties") {
                        if let Ok(props) = serde_json::from_value::<HashMap<String, HashMap<String, String>>>(properties.clone()) {
                            let mut attributes = HashMap::new();
                            for (prop_name, prop_def) in props {
                                let prop_type = prop_def.get("type").map(|s| s.as_str()).unwrap_or("String");
                                attributes.insert(prop_name, Attribute {
                                    attr_type: match prop_type {
                                        "string" => "String".to_string(),
                                        "number" | "integer" => "Long".to_string(),
                                        "boolean" => "Boolean".to_string(),
                                        _ => "String".to_string(),
                                    },
                                });
                            }

                            namespace.entityTypes.insert(name.clone(), EntityType {
                                memberOfTypes: None,
                                shape: Shape {
                                    shape_type: "Record".to_string(),
                                    attributes: Some(attributes),
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    // Generar actions desde paths
    for (path, path_item) in &openapi.paths {
        for (method, operation) in &path_item.methods {
            if let Some(op_id) = &operation.operation_id {
                // Extraer resource type de x-cedar extension o inferir
                let resource_types = if let Some(cedar_ext) = operation.extensions.get("x-cedar") {
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
                    // Inferir del path
                    infer_resource_type_from_path(path)
                };

                namespace.actions.insert(op_id.clone(), Action {
                    appliesTo: AppliesTo {
                        principalTypes: vec!["User".to_string()],
                        resourceTypes: resource_types,
                    },
                });
            }
        }
    }

    let mut namespaces = HashMap::new();
    namespaces.insert("PetStore".to_string(), namespace);

    Ok(CedarSchema { namespaces })
}

fn infer_resource_type_from_path(path: &str) -> Vec<String> {
    if path.contains("pet") {
        vec!["Pet".to_string()]
    } else if path.contains("appointment") {
        vec!["Appointment".to_string()]
    } else {
        vec!["Application".to_string()]
    }
}

fn generate_default_policies(openapi: &OpenApiSpec) -> Result<()> {
    fs::create_dir_all("config/policies")?;

    // Generar policy para admin
    let admin_policy = generate_role_policy("pet-admin", openapi, true)?;
    fs::write("config/policies/admin.cedar", admin_policy)?;

    // Generar policy para vet
    let vet_policy = generate_role_policy("pet-vet", openapi, false)?;
    fs::write("config/policies/vet.cedar", vet_policy)?;

    // Generar policy para customer
    let customer_policy = generate_role_policy("pet-customer", openapi, false)?;
    fs::write("config/policies/customer.cedar", customer_policy)?;

    Ok(())
}

fn generate_role_policy(role: &str, openapi: &OpenApiSpec, is_admin: bool) -> Result<String> {
    let mut policy = String::new();
    
    // Header
    policy.push_str(&format!(
        "// ============================================================================\n"
    ));
    policy.push_str(&format!(
        "// POLÍTICA: {} Access\n", role.replace("-", " ").to_uppercase()
    ));
    policy.push_str(&format!(
        "// Descripción: {}\n", 
        if is_admin {
            "Acceso completo para administradores".to_string()
        } else if role == "pet-vet" {
            "Los veterinarios pueden ver todo y crear citas".to_string()
        } else {
            "Los clientes solo pueden ver información".to_string()
        }
    ));
    policy.push_str(&format!(
        "// ============================================================================\n\n"
    ));

    // Permit actions
    let mut permit_actions = Vec::new();
    let mut forbid_actions = Vec::new();

    for (path, path_item) in &openapi.paths {
        for (method, operation) in &path_item.methods {
            if let Some(op_id) = &operation.operation_id {
                // Determinar si esta acción debe estar en permit o forbid
                let should_permit = match role {
                    "pet-admin" => true, // Admin puede todo
                    "pet-vet" => {
                        // Vet puede listar, ver y crear citas
                        op_id.contains("list") || 
                        op_id.contains("view") || 
                        op_id.contains("createAppointment")
                    },
                    "pet-customer" => {
                        // Customer solo puede ver
                        op_id.contains("list") || op_id.contains("view")
                    },
                    _ => false,
                };

                if should_permit {
                    permit_actions.push(op_id.clone());
                } else if !is_admin {
                    forbid_actions.push(op_id.clone());
                }
            }
        }
    }

    // Generar bloque permit
    if !permit_actions.is_empty() {
        policy.push_str("permit(\n");
        policy.push_str(&format!("    principal in Group::{:?},\n", role));
        policy.push_str("    action in [\n");
        for action in &permit_actions {
            policy.push_str(&format!("        Action::{:?},\n", action));
        }
        policy.push_str("    ],\n");
        policy.push_str("    resource\n");
        policy.push_str(")\n");
        policy.push_str("when {\n");
        policy.push_str(&format!("    principal.role == {:?}\n", role));
        policy.push_str("};\n\n");
    }

    // Generar bloque forbid para no-admin
    if !is_admin && !forbid_actions.is_empty() {
        policy.push_str("// Denegar acciones no permitidas\n");
        policy.push_str("forbid(\n");
        policy.push_str(&format!("    principal in Group::{:?},\n", role));
        policy.push_str("    action in [\n");
        for action in &forbid_actions {
            policy.push_str(&format!("        Action::{:?},\n", action));
        }
        policy.push_str("    ],\n");
        policy.push_str("    resource\n");
        policy.push_str(")\n");
        policy.push_str("when {\n");
        policy.push_str(&format!("    principal.role == {:?}\n", role));
        policy.push_str("};\n");
    }

    Ok(policy)
}

fn generate_setup_script() -> Result<()> {
    let script = r#"#!/bin/bash
# =============================================================================
# Script de Setup para Pet Store Demo
# Este script configura el Policy Store, Schema y Policies en Hodei AVP
# =============================================================================

set -e

echo "🚀 Configurando Pet Store Demo en Hodei AVP..."

# Variables
POLICY_STORE_ID="petstore-demo"
POLICY_STORE_NAME="Pet Store Demo"
IDENTITY_SOURCE_ID="petstore-identity"
KEYCLOAK_ISSUER="${KEYCLOAK_ISSUER:-http://localhost:8080/realms/demo}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-demo-app}"
AVP_ENDPOINT="${AVP_HOST:-localhost}:${AVP_PORT:-50051}"

# Función helper para grpcurl
grpcurl_call() {
    local method=$1
    local data=$2
    
    echo "$data" | grpcurl -plaintext -d @ "$AVP_ENDPOINT" "hodei.permissions.v1.AuthorizationControl/$method"
}

# 1. Crear Policy Store
echo "📦 Creando Policy Store..."
grpcurl_call "CreatePolicyStore" "{
  \"policy_store_id\": \"$POLICY_STORE_ID\",
  \"name\": \"$POLICY_STORE_NAME\"
}" || echo "⚠️  Policy Store ya existe o error"

# 2. Subir Schema
echo "📋 Subiendo Schema..."
SCHEMA_JSON=$(cat config/schema.json | jq -c .)
grpcurl_call "PutSchema" "{
  \"policy_store_id\": \"$POLICY_STORE_ID\",
  \"schema\": $(echo "$SCHEMA_JSON" | jq -R .)
}" || echo "⚠️  Error al subir schema"

# 3. Crear Identity Source
echo "🔐 Creando Identity Source..."
grpcurl_call "CreateIdentitySource" "{
  \"policy_store_id\": \"$POLICY_STORE_ID\",
  \"identity_source_id\": \"$IDENTITY_SOURCE_ID\",
  \"description\": \"Keycloak Identity Source for Pet Store\",
  \"oidc_configuration\": {
    \"issuer\": \"$KEYCLOAK_ISSUER\",
    \"client_ids\": [\"$KEYCLOAK_CLIENT_ID\"],
    \"jwks_uri\": \"$KEYCLOAK_ISSUER/protocol/openid-connect/certs\",
    \"group_claim\": \"realm_access.roles\"
  },
  \"claims_mapping\": {
    \"principal_id_claim\": \"sub\",
    \"group_claim\": \"realm_access.roles\"
  }
}" || echo "⚠️  Identity Source ya existe o error"

# 4. Crear Policies
echo "🛡️  Creando Policies..."
for policy_file in config/policies/*.cedar; do
    if [ -f "$policy_file" ]; then
        POLICY_ID=$(basename "$policy_file" .cedar)
        POLICY_CONTENT=$(cat "$policy_file")
        
        echo "   Creando policy: $POLICY_ID"
        grpcurl_call "CreatePolicy" "{
          \"policy_store_id\": \"$POLICY_STORE_ID\",
          \"policy_id\": \"$POLICY_ID\",
          \"policy\": $(echo "$POLICY_CONTENT" | jq -R .)
        }" || echo "⚠️  Policy $POLICY_ID ya existe o error"
    fi
done

echo ""
echo "✅ Setup completado exitosamente!"
echo ""
echo "Próximos pasos:"
echo "1. Asegúrate de que Keycloak está corriendo"
echo "2. Inicia la app: cargo run"
echo "3. Accede a: http://localhost:3000"
echo ""
echo "Usuarios de demo:"
echo "  - pet_admin / Password123! (admin)"
echo "  - pet_vet / Password123! (vet)"
echo "  - pet_customer / Password123! (customer)"
"#;

    fs::write("config/setup.sh", script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata("config/setup.sh")?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions("config/setup.sh", perms)?;
    }

    Ok(())
}