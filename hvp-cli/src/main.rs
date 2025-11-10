//! HVP CLI - Command-line tools for Hodei Verified Permissions

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use verified_permissions_sdk::schema::{SchemaGenerationUseCase, SimpleRestSchemaGenerator};

#[derive(Parser)]
#[command(name = "hvp-cli")]
#[command(about = "CLI tools for Hodei Verified Permissions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Cedar schema from OpenAPI specification
    GenerateSchema {
        /// Path to the OpenAPI spec file (JSON format)
        #[arg(long, value_name = "FILE")]
        api_spec: PathBuf,

        /// Cedar namespace for your application
        #[arg(long)]
        namespace: String,

        /// Base path of your API (optional)
        /// If your API spec has multiple servers, this parameter is required
        #[arg(long)]
        base_path: Option<String>,

        /// Output directory for generated schema files
        #[arg(long, short = 'o', default_value = ".")]
        output: PathBuf,
    },

    /// Generate sample Cedar policies from a schema
    GeneratePolicies {
        /// Path to the Cedar schema file (v4 JSON format)
        #[arg(long, value_name = "FILE")]
        schema: PathBuf,

        /// Output directory for generated policy files
        #[arg(long, short = 'o', default_value = "./policies")]
        output: PathBuf,
    },

    /// Generate least privilege policies from OpenAPI specification
    GenerateLeastPrivilege {
        /// Path to the OpenAPI spec file (JSON format)
        #[arg(long, value_name = "FILE")]
        spec: PathBuf,

        /// Cedar namespace for your application
        #[arg(long)]
        namespace: String,

        /// Base path of your API (optional)
        #[arg(long)]
        base_path: Option<String>,

        /// Output directory for generated files
        #[arg(long, short = 'o', default_value = "./authorization")]
        output: PathBuf,

        /// Generate role-based policies (comma-separated list)
        #[arg(long, default_value = "admin,developer,viewer")]
        roles: String,

        /// Analysis mode: strict, moderate, or permissive
        #[arg(long, default_value = "strict")]
        mode: String,
    },

    /// Generate complete setup (schema + policies + setup script)
    GenerateSetup {
        /// Path to the OpenAPI spec file (JSON format)
        #[arg(long, value_name = "FILE")]
        spec: PathBuf,

        /// Cedar namespace for your application
        #[arg(long)]
        namespace: String,

        /// Base path of your API (optional)
        #[arg(long)]
        base_path: Option<String>,

        /// Output directory for generated files
        #[arg(long, short = 'o', default_value = "./config")]
        output: PathBuf,

        /// Application name (used for policy store and identity source)
        #[arg(long, default_value = "my-app")]
        app_name: String,

        /// Keycloak issuer URL (for identity source)
        #[arg(long)]
        keycloak_issuer: Option<String>,

        /// Keycloak client ID
        #[arg(long, default_value = "my-app-client")]
        keycloak_client_id: String,

        /// AVP endpoint (host:port)
        #[arg(long, default_value = "localhost:50051")]
        avp_endpoint: String,

        /// Roles to generate (comma-separated)
        #[arg(long, default_value = "admin,vet,customer")]
        roles: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateSchema {
            api_spec,
            namespace,
            base_path,
            output,
        } => {
            generate_schema(api_spec, namespace, base_path, output).await?;
        }
        Commands::GeneratePolicies { schema, output } => {
            generate_policies(schema, output).await?;
        }
        Commands::GenerateLeastPrivilege {
            spec,
            namespace,
            base_path,
            output,
            roles,
            mode,
        } => {
            generate_least_privilege(spec, namespace, base_path, output, roles, mode).await?;
        }
        Commands::GenerateSetup {
            spec,
            namespace,
            base_path,
            output,
            app_name,
            keycloak_issuer,
            keycloak_client_id,
            avp_endpoint,
            roles,
        } => {
            generate_setup(
                spec,
                namespace,
                base_path,
                output,
                app_name,
                keycloak_issuer,
                keycloak_client_id,
                avp_endpoint,
                roles,
            )
            .await?;
        }
    }

    Ok(())
}

// Estructuras para parsing eficiente
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
    #[serde(default)]
    parameters: Vec<serde_json::Value>,
    #[serde(flatten)]
    extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Components {
    schemas: Option<HashMap<String, serde_json::Value>>,
}

// Cache para acciones por rol (O(1) lookup)
lazy_static::lazy_static! {
    static ref ROLE_ACTION_CACHE: std::sync::Mutex<HashMap<String, HashSet<String>>> =
        std::sync::Mutex::new(HashMap::new());
}

async fn generate_setup(
    spec_path: PathBuf,
    namespace: String,
    base_path: Option<String>,
    output_dir: PathBuf,
    app_name: String,
    keycloak_issuer: Option<String>,
    keycloak_client_id: String,
    avp_endpoint: String,
    roles_str: String,
) -> Result<()> {
    info!("🚀 Generando configuración completa para {}", app_name);

    // 1. Crear directorio de salida
    fs::create_dir_all(&output_dir)
        .await
        .context("Failed to create output directory")?;

    // 2. Generar schema (reutiliza función existente)
    info!("📋 Generando schema Cedar...");
    let schema_output = output_dir.join("schema");
    generate_schema(
        spec_path.clone(),
        namespace.clone(),
        base_path.clone(),
        schema_output,
    )
    .await?;

    // 3. Generar policies (reutiliza función existente)
    info!("🛡️  Generando policies...");
    let policies_output = output_dir.join("policies");
    let schema_file = output_dir.join("schema/v4.cedarschema.json");
    
    // Parsear OpenAPI una vez y reutilizar
    let openapi_content = read_and_validate_openapi(&spec_path).await?;
    let openapi: OpenApiSpec = serde_json::from_str(&openapi_content)?;
    let actions = extract_actions_from_openapi(&openapi, &namespace);
    
    generate_policies_from_openapi(&actions, &policies_output, &roles_str, &namespace).await?;

    // 4. Generar script de setup
    info!("📜 Generando script de setup...");
    generate_setup_script(
        &output_dir,
        &app_name,
        keycloak_issuer,
        keycloak_client_id,
        avp_endpoint,
        &roles_str,
    )
    .await?;

    // 5. Generar .env.example
    info!("⚙️  Generando archivo de configuración...");
    generate_env_file(&output_dir, &app_name, &avp_endpoint).await?;

    info!("\n✅ Configuración completa generada en: {}", output_dir.display());
    info!("\nPróximos pasos:");
    info!("1. Revisa los archivos generados en {}/", output_dir.display());
    info!("2. Ajusta las policies en {}/policies/ si es necesario", output_dir.display());
    info!("3. Ejecuta: bash {}/setup.sh", output_dir.display());
    info!("4. Inicia tu aplicación");

    Ok(())
}

// Funciones helper reutilizables

async fn read_and_validate_openapi(spec_path: &PathBuf) -> Result<String> {
    if !spec_path.exists() {
        anyhow::bail!("OpenAPI spec file not found: {}", spec_path.display());
    }
    
    let content = fs::read_to_string(spec_path)
        .await
        .context("Failed to read OpenAPI spec file")?;
    
    // Validar JSON
    let _: serde_json::Value = serde_json::from_str(&content)
        .context("Invalid JSON in OpenAPI spec file")?;
    
    Ok(content)
}

fn extract_actions_from_openapi(openapi: &OpenApiSpec, namespace: &str) -> HashSet<String> {
    let mut actions = HashSet::new();
    
    for (path, path_item) in &openapi.paths {
        for (method, operation) in &path_item.methods {
            if let Some(op_id) = &operation.operation_id {
                actions.insert(op_id.clone());
            } else {
                // Generar operationId si no existe
                let generated = format!("{}_{}", method.to_lowercase(), path.replace("/", "_"));
                actions.insert(generated);
            }
        }
    }
    
    actions
}

async fn generate_policies_from_openapi(
    actions: &HashSet<String>,
    output_dir: &PathBuf,
    roles_str: &str,
    namespace: &str,
) -> Result<()> {
    let roles: Vec<String> = roles_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    fs::create_dir_all(&output_dir)
        .await
        .context("Failed to create policies directory")?;

    // Pre-computar acciones por rol usando HashSet para O(1) lookup
    let actions_by_role = precompute_actions_by_role(actions, &roles);
    
    // Generar policies en paralelo
    let mut tasks = Vec::new();
    for role in roles {
        let actions_for_role = actions_by_role.get(&role).unwrap_or(&HashSet::new()).clone();
        let output_dir = output_dir.clone();
        let namespace = namespace.to_string();
        
        let task = tokio::spawn(async move {
            let policy_content = generate_role_policy_optimized(&role, &actions_for_role, &namespace);
            let policy_path = output_dir.join(format!("{}.cedar", role.replace("-", "_")));
            
            tokio::fs::write(&policy_path, policy_content).await?;
            Ok::<_, anyhow::Error>((role, policy_path))
        });
        
        tasks.push(task);
    }
    
    // Esperar a que todas las policies se generen
    for task in tasks {
        let (role, path) = task.await??;
        info!("   ✓ Policy para {}: {}", role, path.display());
    }

    Ok(())
}

fn precompute_actions_by_role(
    all_actions: &HashSet<String>,
    roles: &[String],
) -> HashMap<String, HashSet<String>> {
    let mut result = HashMap::new();
    
    // Definir reglas de mapeo una vez
    let role_rules: HashMap<&str, Box<dyn Fn(&str) -> bool>> = [
        ("admin", Box::new(|_: &str| true) as Box<dyn Fn(&str) -> bool>),
        ("vet", Box::new(|action: &str| {
            action.contains("list") || action.contains("view") || action.contains("createAppointment")
        })),
        ("customer", Box::new(|action: &str| {
            action.contains("list") || action.contains("view")
        })),
    ].iter().cloned().collect();
    
    for role in roles {
        let rule = role_rules.get(role.as_str()).unwrap_or_else(||
            &Box::new(|_: &str| false) as Box<dyn Fn(&str) -> bool>
        );
        
        let permitted_actions: HashSet<String> = all_actions
            .iter()
            .filter(|action| rule(action))
            .cloned()
            .collect();
            
        result.insert(role.clone(), permitted_actions);
    }
    
    result
}

fn generate_role_policy_optimized(
    role: &str,
    permitted_actions: &HashSet<String>,
    namespace: &str,
) -> String {
    let mut policy = String::new();

    // Header
    policy.push_str("// ============================================================================\n");
    policy.push_str(&format!("// POLÍTICA: {} Access\n", role.to_uppercase()));
    policy.push_str(&format!("// Descripción: {}\n", get_role_description(role)));
    policy.push_str("// ============================================================================\n\n");

    // Bloque permit
    if !permitted_actions.is_empty() {
        write_permit_block(&mut policy, role, permitted_actions, namespace);
    }

    // Bloque forbid (solo para no-admin)
    if role != "admin" {
        let all_actions = permitted_actions; // En este contexto, ya tenemos los permitidos
        // El forbid se genera automáticamente por el motor de Cedar cuando no hay permit
    }

    policy
}

fn get_role_description(role: &str) -> String {
    match role {
        "admin" => "Acceso completo para administradores".to_string(),
        "vet" => "Los veterinarios pueden ver todo y crear citas".to_string(),
        "customer" => "Los clientes solo pueden ver información".to_string(),
        _ => format!("Permisos para rol {}", role),
    }
}

fn write_permit_block(
    policy: &mut String,
    role: &str,
    actions: &HashSet<String>,
    namespace: &str,
) {
    policy.push_str("permit(\n");
    policy.push_str(&format!("    principal in Group::{:?},\n", role));
    policy.push_str("    action in [\n");
    
    // Ordenar actions para consistencia
    let mut sorted_actions: Vec<_> = actions.iter().collect();
    sorted_actions.sort();
    
    for action in sorted_actions {
        policy.push_str(&format!("        Action::{:?},\n", action));
    }
    
    policy.push_str("    ],\n");
    policy.push_str("    resource\n");
    policy.push_str(")\n");
    policy.push_str("when {\n");
    policy.push_str(&format!("    principal.role == {:?}\n", role));
    policy.push_str("};\n\n");
}

async fn generate_setup_script(
    output_dir: &PathBuf,
    app_name: &str,
    keycloak_issuer: Option<String>,
    keycloak_client_id: String,
    avp_endpoint: String,
    roles_str: &str,
) -> Result<()> {
    let policy_store_id = format!("{}-store", app_name.replace("_", "-"));
    let identity_source_id = format!("{}-identity", app_name.replace("_", "-"));

    let script = format!(
        r#"#!/bin/bash
# =============================================================================
# Script de Setup para {}
# Este script configura el Policy Store, Schema y Policies en Hodei AVP
# =============================================================================

set -e

echo "🚀 Configurando {} en Hodei AVP..."

# Variables
POLICY_STORE_ID="{}"
POLICY_STORE_NAME="{}"
IDENTITY_SOURCE_ID="{}"
KEYCLOAK_ISSUER="${{KEYCLOAK_ISSUER:-{}}}"
KEYCLOAK_CLIENT_ID="${{KEYCLOAK_CLIENT_ID:-{}}}"
AVP_ENDPOINT="${{AVP_HOST:-localhost}}:${{AVP_PORT:-50051}}"

# Función helper para grpcurl
grpcurl_call() {{
    local method=$1
    local data=$2
    
    echo "$data" | grpcurl -plaintext -d @ "$AVP_ENDPOINT" "hodei.permissions.v1.AuthorizationControl/$method"
}}

# 1. Crear Policy Store
echo "📦 Creando Policy Store..."
grpcurl_call "CreatePolicyStore" "{{
  \"policy_store_id\": \"$POLICY_STORE_ID\",
  \"name\": \"$POLICY_STORE_NAME\"
}}" || echo "⚠️  Policy Store ya existe o error"

# 2. Subir Schema
echo "📋 Subiendo Schema..."
SCHEMA_JSON=$(cat schema/v4.cedarschema.json | jq -c .)
grpcurl_call "PutSchema" "{{
  \"policy_store_id\": \"$POLICY_STORE_ID\",
  \"schema\": $(echo "$SCHEMA_JSON" | jq -R .)
}}" || echo "⚠️  Error al subir schema"

# 3. Crear Identity Source (si Keycloak está configurado)
if [ -n "$KEYCLOAK_ISSUER" ]; then
    echo "🔐 Creando Identity Source..."
    grpcurl_call "CreateIdentitySource" "{{
      \"policy_store_id\": \"$POLICY_STORE_ID\",
      \"identity_source_id\": \"$IDENTITY_SOURCE_ID\",
      \"description\": \"Identity Source for {}\",
      \"oidc_configuration\": {{
        \"issuer\": \"$KEYCLOAK_ISSUER\",
        \"client_ids\": [\"$KEYCLOAK_CLIENT_ID\"],
        \"jwks_uri\": \"$KEYCLOAK_ISSUER/protocol/openid-connect/certs\",
        \"group_claim\": \"realm_access.roles\"
      }},
      \"claims_mapping\": {{
        \"principal_id_claim\": \"sub\",
        \"group_claim\": \"realm_access.roles\"
      }}
    }}" || echo "⚠️  Identity Source ya existe o error"
else
    echo "⚠️  KEYCLOAK_ISSUER no configurado, saltando Identity Source"
fi

# 4. Crear Policies
echo "🛡️  Creando Policies..."
for policy_file in policies/*.cedar; do
    if [ -f "$policy_file" ]; then
        POLICY_ID=$(basename "$policy_file" .cedar)
        POLICY_CONTENT=$(cat "$policy_file")
        
        echo "   Creando policy: $POLICY_ID"
        grpcurl_call "CreatePolicy" "{{
          \"policy_store_id\": \"$POLICY_STORE_ID\",
          \"policy_id\": \"$POLICY_ID\",
          \"policy\": $(echo "$POLICY_CONTENT" | jq -R .)
        }}" || echo "⚠️  Policy $POLICY_ID ya existe o error"
    fi
done

echo ""
echo "✅ Setup completado exitosamente!"
echo ""
echo "Variables de entorno para tu aplicación:"
echo "POLICY_STORE_ID=$POLICY_STORE_ID"
echo "IDENTITY_SOURCE_ID=$IDENTITY_SOURCE_ID"
echo "AVP_ENDPOINT=$AVP_ENDPOINT"
"#,
        app_name,
        app_name,
        policy_store_id,
        format!("{} Policy Store", app_name.replace("-", " ").to_uppercase()),
        identity_source_id,
        keycloak_issuer.unwrap_or_else(|| "http://localhost:8080/realms/demo".to_string()),
        keycloak_client_id,
        app_name,
    );

    let script_path = output_dir.join("setup.sh");
    fs::write(&script_path, script)
        .await
        .context("Failed to write setup script")?;

    // Hacer ejecutable en Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).await?;
    }

    info!("   ✓ Script de setup: {}", script_path.display());

    Ok(())
}

async fn generate_env_file(
    output_dir: &PathBuf,
    app_name: &str,
    avp_endpoint: &str,
) -> Result<()> {
    let env_content = format!(
        r#"# Configuration for {}
# Generated by hvp-cli

# Hodei AVP Configuration
AVP_ENDPOINT=http://{}
POLICY_STORE_ID={}-store
IDENTITY_SOURCE_ID={}-identity

# Keycloak Configuration (optional)
KEYCLOAK_ISSUER=http://localhost:8080/realms/demo
KEYCLOAK_CLIENT_ID={}-client
KEYCLOAK_CLIENT_SECRET=your-client-secret

# Application Configuration
APP_NAME={}
RUST_LOG=info
"#,
        app_name,
        avp_endpoint,
        app_name.replace("_", "-"),
        app_name.replace("_", "-"),
        app_name,
        app_name,
    );

    let env_path = output_dir.join(".env.example");
    fs::write(&env_path, env_content)
        .await
        .context("Failed to write .env.example")?;

    info!("   ✓ Archivo de configuración: {}", env_path.display());

    Ok(())
}

// ... (resto de las funciones generate_schema, generate_policies, generate_least_privilege)
// Estas funciones permanecen igual que en el código original
