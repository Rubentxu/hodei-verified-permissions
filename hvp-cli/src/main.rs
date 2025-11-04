//! HVP CLI - Command-line tools for Hodei Verified Permissions

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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
    }

    Ok(())
}

async fn generate_least_privilege(
    spec_path: PathBuf,
    namespace: String,
    base_path: Option<String>,
    output_dir: PathBuf,
    roles_str: String,
    mode_str: String,
) -> Result<()> {
    info!("Reading OpenAPI spec from: {}", spec_path.display());

    // Check if API spec file exists
    if !spec_path.exists() {
        anyhow::bail!("OpenAPI spec file not found: {}", spec_path.display());
    }

    // Read OpenAPI spec
    let spec_content = fs::read_to_string(&spec_path)
        .await
        .context("Failed to read OpenAPI spec file")?;

    // Validate JSON
    let _: serde_json::Value =
        serde_json::from_str(&spec_content).context("Invalid JSON in OpenAPI spec file")?;

    info!(
        "Generating least privilege policies with namespace: {}",
        namespace
    );

    // Parse roles
    let roles: Vec<String> = roles_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if roles.is_empty() {
        anyhow::bail!("At least one role must be specified");
    }

    info!("Roles: {}", roles.join(", "));

    // Parse privilege mode
    let mode = verified_permissions_sdk::policies::PrivilegeMode::parse(&mode_str)
        .map_err(|e| anyhow::anyhow!("Invalid privilege mode: {}", e))?;

    info!("Privilege mode: {}", mode);

    // Generate schema first
    let generator = verified_permissions_sdk::schema::SimpleRestSchemaGenerator::new();
    let schema_bundle = generator
        .generate_simple_rest_schema(&spec_content, &namespace, base_path.as_deref())
        .await
        .context("Failed to generate Cedar schema")?;

    // Generate least privilege policies
    let policy_roles = vec![
        verified_permissions_sdk::policies::Role::admin(),
        verified_permissions_sdk::policies::Role::developer(),
        verified_permissions_sdk::policies::Role::viewer(),
    ];

    let policy_generator = verified_permissions_sdk::policies::LeastPrivilegeGenerator::new(
        namespace.clone(),
        policy_roles,
        mode,
    );

    let policy_bundle = policy_generator
        .generate_from_openapi(&spec_content, &schema_bundle)
        .context("Failed to generate policy bundle")?;

    // Create output directory
    fs::create_dir_all(&output_dir)
        .await
        .context("Failed to create output directory")?;

    // Write schema files
    let v4_schema_path = output_dir.join("v4.cedarschema.json");
    fs::write(&v4_schema_path, &schema_bundle.v4)
        .await
        .context("Failed to write v4 schema file")?;
    info!("✓ Cedar schema v4 generated: {}", v4_schema_path.display());

    // Write policies
    let policies_dir = output_dir.join("policies");
    fs::create_dir_all(&policies_dir)
        .await
        .context("Failed to create policies directory")?;

    for (i, policy) in policy_bundle.policies.iter().enumerate() {
        let policy_path = policies_dir.join(format!("policy_{}.cedar", i + 1));
        fs::write(&policy_path, &policy.content)
            .await
            .context(format!(
                "Failed to write policy file {}",
                policy_path.display()
            ))?;
        info!("✓ Cedar policy generated: {}", policy_path.display());
    }

    // Write security report
    let report_path = output_dir.join("security_report.md");
    let report_content = format!(
        "# Security Analysis Report\n\n## Summary\n{}\n\n## Warnings\n",
        policy_bundle.security_report.summary()
    );

    fs::write(&report_path, report_content)
        .await
        .context("Failed to write security report")?;
    info!("✓ Security report generated: {}", report_path.display());

    // Display summary
    info!("\nAuthorization bundle successfully generated!");
    info!("  Namespace: {}", namespace);
    info!("  Policies: {}", policy_bundle.policies.len());
    info!("  Roles: {}", roles.join(", "));
    info!(
        "  Security score: {}/100",
        policy_bundle.security_report.score()
    );

    if !policy_bundle.security_report.warnings().is_empty() {
        info!("\nSecurity warnings:");
        for warning in policy_bundle.security_report.warnings() {
            info!("  ⚠️  {}", warning.message());
        }
    }

    info!("\nAll files written to: {}", output_dir.display());

    Ok(())
}

async fn generate_schema(
    api_spec_path: PathBuf,
    namespace: String,
    base_path: Option<String>,
    output_dir: PathBuf,
) -> Result<()> {
    info!("Reading OpenAPI spec from: {}", api_spec_path.display());

    // Check if API spec file exists
    if !api_spec_path.exists() {
        anyhow::bail!("API spec file not found: {}", api_spec_path.display());
    }

    // Read OpenAPI spec
    let spec_content = fs::read_to_string(&api_spec_path)
        .await
        .context("Failed to read API spec file")?;

    // Validate JSON
    let _: serde_json::Value =
        serde_json::from_str(&spec_content).context("Invalid JSON in API spec file")?;

    info!("Generating Cedar schema with namespace: {}", namespace);

    // Generate schema
    let generator = SimpleRestSchemaGenerator::new();
    let bundle = generator
        .generate_simple_rest_schema(&spec_content, &namespace, base_path.as_deref())
        .await
        .context("Failed to generate Cedar schema")?;

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)
        .await
        .context("Failed to create output directory")?;

    // Write v4 schema
    let v4_path = output_dir.join("v4.cedarschema.json");
    fs::write(&v4_path, &bundle.v4)
        .await
        .context("Failed to write v4 schema file")?;

    info!("✓ Cedar schema v4 generated: {}", v4_path.display());
    info!("  Namespace: {}", bundle.metadata.namespace);
    info!("  Mapping type: {}", bundle.metadata.mapping_type);
    info!("  Actions: {}", bundle.metadata.action_count);
    info!("  Entity types: {}", bundle.metadata.entity_type_count);

    if let Some(bp) = bundle.metadata.base_path {
        info!("  Base path: {}", bp);
    }

    // Write v2 schema if available
    if let Some(v2_content) = bundle.v2 {
        let v2_path = output_dir.join("v2.cedarschema.json");
        fs::write(&v2_path, v2_content)
            .await
            .context("Failed to write v2 schema file")?;
        info!("✓ Cedar schema v2 generated: {}", v2_path.display());
    }

    info!("\nSchema files successfully generated!");
    info!("v4.cedarschema.json is compatible with Cedar 4.x and required by nodejs Cedar plugins.");

    Ok(())
}

async fn generate_policies(schema_path: PathBuf, output_dir: PathBuf) -> Result<()> {
    info!("Reading Cedar schema from: {}", schema_path.display());

    // Check if schema file exists
    if !schema_path.exists() {
        anyhow::bail!("Schema file not found: {}", schema_path.display());
    }

    // Read schema
    let schema_content = fs::read_to_string(&schema_path)
        .await
        .context("Failed to read schema file")?;

    // Parse schema to extract namespace and actions
    let schema: serde_json::Value =
        serde_json::from_str(&schema_content).context("Invalid JSON in schema file")?;

    // Extract namespace (first key in the schema object)
    let namespace = schema
        .as_object()
        .and_then(|obj| obj.keys().next())
        .context("Schema must have at least one namespace")?;

    // Extract actions
    let actions = schema
        .get(namespace)
        .and_then(|ns| ns.get("actions"))
        .and_then(|a| a.as_object())
        .context("Schema must have actions defined")?;

    let action_names: Vec<String> = actions.keys().cloned().collect();

    if action_names.is_empty() {
        anyhow::bail!("No actions found in schema");
    }

    info!("Found {} actions in schema", action_names.len());

    // Create output directory
    fs::create_dir_all(&output_dir)
        .await
        .context("Failed to create output directory")?;

    // Generate sample policies
    let admin_policy = format!(
        r#"// Allows admin usergroup access to everything
permit(
    principal in {}::UserGroup::"admin",
    action,
    resource
);"#,
        namespace
    );

    let role_policy = format!(
        r#"// Allows more granular user group control, change actions as needed
permit(
    principal in {}::UserGroup::"ENTER_THE_USER_GROUP_HERE",
    action in [
        {}
    ],
    resource
);"#,
        namespace,
        action_names
            .iter()
            .map(|a| format!("        {}::Action::\"{}\"", namespace, a))
            .collect::<Vec<_>>()
            .join(",\n")
    );

    // Write policies
    let admin_path = output_dir.join("policy_1.cedar");
    fs::write(&admin_path, admin_policy)
        .await
        .context("Failed to write admin policy")?;
    info!("✓ Cedar policy generated: {}", admin_path.display());

    let role_path = output_dir.join("policy_2.cedar");
    fs::write(&role_path, role_policy)
        .await
        .context("Failed to write role policy")?;
    info!("✓ Cedar policy generated: {}", role_path.display());

    info!(
        "\nSample policies successfully generated in: {}",
        output_dir.display()
    );

    Ok(())
}
