//! Local Agent Binary
//!
//! Standalone binary for running the local agent

use clap::Parser;
// TODO: Agent module needs to be updated for new architecture
// use hodei_verified_permissions::agent::{AgentConfig, LocalAgent};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "hodei-agent")]
#[command(about = "Hodei Verified Permissions Local Agent", long_about = None)]
struct Args {
    /// Central service URL
    #[arg(short, long, default_value = "http://localhost:50051")]
    central_url: String,

    /// Policy store ID to sync
    #[arg(short, long)]
    policy_store_id: String,

    /// Sync interval in seconds
    #[arg(short, long, default_value = "60")]
    sync_interval: u64,

    /// Local gRPC port
    #[arg(short, long, default_value = "50052")]
    local_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    println!("🚀 Starting Hodei Local Agent");
    println!("   Central Service: {}", args.central_url);
    println!("   Policy Store: {}", args.policy_store_id);
    println!("   Sync Interval: {}s", args.sync_interval);
    println!("   Local Port: {}", args.local_port);

    // TODO: Implement agent with new architecture
    println!("⚠️  Agent functionality not yet implemented in new architecture");
    println!("   This will be implemented after core refactor is complete");

    Ok(())
}
