use hodei_verified_permissions::grpc::{AuthorizationControlService, AuthorizationDataService};
use hodei_verified_permissions::proto::authorization_control_server::AuthorizationControlServer;
use hodei_verified_permissions::proto::authorization_data_server::AuthorizationDataServer;
use hodei_verified_permissions::storage::Repository;
use tonic::transport::Server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Database URL (using SQLite for MVP)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://authorization.db".to_string());

    info!("Connecting to database: {}", database_url);
    let repository = Repository::new(&database_url).await?;

    // Server address
    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()?;

    info!("Starting Hodei Verified Permissions server on {}", addr);

    // Create services
    let data_service = AuthorizationDataService::new(repository.clone());
    let control_service = AuthorizationControlService::new(repository);

    // Start server
    Server::builder()
        .add_service(AuthorizationDataServer::new(data_service))
        .add_service(AuthorizationControlServer::new(control_service))
        .serve(addr)
        .await?;

    Ok(())
}
