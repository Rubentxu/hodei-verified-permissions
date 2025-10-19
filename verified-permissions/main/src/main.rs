// TODO: Update main.rs to use new architecture with dependency injection
// use hodei_infrastructure::create_repository;
// use hodei_api::grpc::{AuthorizationControlService, AuthorizationDataService};
// use hodei_api::proto::authorization_control_server::AuthorizationControlServer;
// use hodei_api::proto::authorization_data_server::AuthorizationDataServer;
// use tonic::transport::Server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Hodei Verified Permissions Server");
    info!("⚠️  Main server functionality not yet implemented in new architecture");
    info!("   This will be implemented after core refactor is complete");
    info!("   Use the CLI binary for now: cargo run --bin hodei-cli");

    // TODO: Implement server with new architecture:
    // 1. Create repository adapter
    // 2. Create application use cases
    // 3. Create gRPC services
    // 4. Start server

    Ok(())
}
