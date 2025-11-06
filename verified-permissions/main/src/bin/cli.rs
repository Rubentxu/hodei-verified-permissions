//! Hodei Verified Permissions CLI
//!
//! Command-line interface for managing policy stores, schemas, and policies.

use clap::{Parser, Subcommand};
use hodei_api::proto::{
    CreatePolicyRequest, CreatePolicyStoreRequest, DeletePolicyRequest, DeletePolicyStoreRequest,
    GetPolicyStoreRequest, ListPoliciesRequest, ListPolicyStoresRequest, PolicyDefinition,
    PutSchemaRequest, StaticPolicy, authorization_control_client::AuthorizationControlClient,
    policy_definition,
};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hodei-cli")]
#[command(about = "Hodei Verified Permissions CLI", long_about = None)]
struct Cli {
    /// gRPC server address
    #[arg(short, long, default_value = "http://localhost:50051")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Policy Store management
    #[command(subcommand)]
    Store(StoreCommands),

    /// Policy management
    #[command(subcommand)]
    Policy(PolicyCommands),

    /// Schema management
    #[command(subcommand)]
    Schema(SchemaCommands),
}

#[derive(Subcommand)]
enum StoreCommands {
    /// Create a new policy store
    Create {
        /// Name of the policy store
        #[arg(short, long)]
        name: String,
        /// Description of the policy store
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Get policy store details
    Get {
        /// Policy store ID
        #[arg(short, long)]
        id: String,
    },
    /// List all policy stores
    List,
    /// Delete a policy store
    Delete {
        /// Policy store ID
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum PolicyCommands {
    /// Create a new policy
    Create {
        /// Policy store ID
        #[arg(short = 's', long)]
        store_id: String,
        /// Policy ID
        #[arg(short = 'i', long)]
        policy_id: String,
        /// Path to Cedar policy file
        #[arg(short, long)]
        file: PathBuf,
        /// Policy description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List policies in a store
    List {
        /// Policy store ID
        #[arg(short = 's', long)]
        store_id: String,
    },
    /// Delete a policy
    Delete {
        /// Policy store ID
        #[arg(short = 's', long)]
        store_id: String,
        /// Policy ID
        #[arg(short = 'i', long)]
        policy_id: String,
    },
}

#[derive(Subcommand)]
enum SchemaCommands {
    /// Upload a schema
    Put {
        /// Policy store ID
        #[arg(short = 's', long)]
        store_id: String,
        /// Path to Cedar schema JSON file
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut client = AuthorizationControlClient::connect(cli.server).await?;

    match cli.command {
        Commands::Store(cmd) => handle_store_command(&mut client, cmd).await?,
        Commands::Policy(cmd) => handle_policy_command(&mut client, cmd).await?,
        Commands::Schema(cmd) => handle_schema_command(&mut client, cmd).await?,
    }

    Ok(())
}

async fn handle_store_command(
    client: &mut AuthorizationControlClient<tonic::transport::Channel>,
    cmd: StoreCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        StoreCommands::Create { name, description } => {
            let response = client
                .create_policy_store(CreatePolicyStoreRequest {
                    name,
                    description,
                    tags: vec![],
                    user: "cli_user".to_string(),
                })
                .await?;
            let store = response.into_inner();
            println!("✅ Policy store created:");
            println!("   ID: {}", store.policy_store_id);
            println!("   Created at: {}", store.created_at);
        }
        StoreCommands::Get { id } => {
            let response = client
                .get_policy_store(GetPolicyStoreRequest {
                    policy_store_id: id,
                })
                .await?;
            let store = response.into_inner();
            println!("Policy Store:");
            println!("   ID: {}", store.policy_store_id);
            if let Some(desc) = store.description {
                println!("   Description: {}", desc);
            }
            println!("   Created at: {}", store.created_at);
            println!("   Updated at: {}", store.updated_at);
        }
        StoreCommands::List => {
            let response = client
                .list_policy_stores(ListPolicyStoresRequest {
                    max_results: None,
                    next_token: None,
                })
                .await?;
            let stores = response.into_inner().policy_stores;
            println!("Policy Stores ({} total):", stores.len());
            for store in stores {
                println!("\n  📦 {}", store.policy_store_id);
                if let Some(desc) = store.description {
                    println!("     Description: {}", desc);
                }
                println!("     Created: {}", store.created_at);
            }
        }
        StoreCommands::Delete { id } => {
            client
                .delete_policy_store(DeletePolicyStoreRequest {
                    policy_store_id: id.clone(),
                })
                .await?;
            println!("✅ Policy store '{}' deleted", id);
        }
    }
    Ok(())
}

async fn handle_policy_command(
    client: &mut AuthorizationControlClient<tonic::transport::Channel>,
    cmd: PolicyCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        PolicyCommands::Create {
            store_id,
            policy_id,
            file,
            description,
        } => {
            let statement = fs::read_to_string(&file)?;
            let response = client
                .create_policy(CreatePolicyRequest {
                    policy_store_id: store_id,
                    policy_id: policy_id.clone(),
                    definition: Some(PolicyDefinition {
                        policy_type: Some(policy_definition::PolicyType::Static(StaticPolicy {
                            statement,
                        })),
                    }),
                    description,
                })
                .await?;
            let policy = response.into_inner();
            println!("✅ Policy created:");
            println!("   ID: {}", policy.policy_id);
            println!("   Store: {}", policy.policy_store_id);
            println!("   Created at: {}", policy.created_at);
        }
        PolicyCommands::List { store_id } => {
            let response = client
                .list_policies(ListPoliciesRequest {
                    policy_store_id: store_id,
                    max_results: None,
                    next_token: None,
                })
                .await?;
            let policies = response.into_inner().policies;
            println!("Policies ({} total):", policies.len());
            for policy in policies {
                println!("\n  📜 {}", policy.policy_id);
                if let Some(desc) = policy.description {
                    println!("     Description: {}", desc);
                }
                println!("     Created: {}", policy.created_at);
            }
        }
        PolicyCommands::Delete {
            store_id,
            policy_id,
        } => {
            client
                .delete_policy(DeletePolicyRequest {
                    policy_store_id: store_id,
                    policy_id: policy_id.clone(),
                })
                .await?;
            println!("✅ Policy '{}' deleted", policy_id);
        }
    }
    Ok(())
}

async fn handle_schema_command(
    client: &mut AuthorizationControlClient<tonic::transport::Channel>,
    cmd: SchemaCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SchemaCommands::Put { store_id, file } => {
            let schema = fs::read_to_string(&file)?;
            client
                .put_schema(PutSchemaRequest {
                    policy_store_id: store_id.clone(),
                    schema,
                })
                .await?;
            println!("✅ Schema uploaded to store '{}'", store_id);
        }
    }
    Ok(())
}
