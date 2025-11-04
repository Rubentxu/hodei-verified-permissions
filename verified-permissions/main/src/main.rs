// TODO: Update main.rs to use new architecture with dependency injection
// use hodei_infrastructure::create_repository;
use hodei_api::grpc::{AuthorizationControlService, AuthorizationDataService};
use hodei_api::proto::authorization_control_server::AuthorizationControlServer;
use hodei_api::proto::authorization_data_server::AuthorizationDataServer;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Hodei Verified Permissions Server");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        info!("DATABASE_URL not set, using default SQLite");
        "sqlite:///home/rubentxu/hodei-data/hodei.db".to_string()
    });

    info!("Connecting to database: {}", database_url);

    // Create repository adapter
    let repository = std::sync::Arc::new(
        hodei_infrastructure::repository::RepositoryAdapter::new(&database_url)
            .await
            .expect("Failed to create repository adapter"),
    );

    info!("✅ Repository initialized successfully");

    // Create services with repository
    let control_service = AuthorizationControlService::new(repository.clone());
    let data_service = AuthorizationDataService::new(repository.clone());

    // Start server
    let addr = "0.0.0.0:50051".parse()?;
    info!("🚀 Server listening on {}", addr);

    Server::builder()
        .add_service(AuthorizationControlServer::new(control_service))
        .add_service(AuthorizationDataServer::new(data_service))
        .serve(addr)
        .await?;

    Ok(())
}
