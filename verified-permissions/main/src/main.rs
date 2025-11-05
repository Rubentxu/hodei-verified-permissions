//! Main entry point for Hodei Verified Permissions Server
//!
//! This application follows hexagonal architecture and SOLID principles.
//! Configuration is loaded from environment variables and .env files.

use hodei_api::grpc::{AuthorizationControlService, AuthorizationDataService};
use hodei_api::proto::authorization_control_server::AuthorizationControlServer;
use hodei_api::proto::authorization_data_server::AuthorizationDataServer;
use hodei_infrastructure::repository::RepositoryAdapter;
use hodei_shared::config::{Configuration, Settings};
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{FmtSubscriber, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration from .env files and environment variables
    info!("🚀 Hodei Verified Permissions Server");
    info!("📋 Loading configuration...");

    let settings = Settings::new().map_err(|e| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Config error: {}", e),
        )) as Box<dyn std::error::Error + Send + Sync>
    })?;

    info!("✅ Configuration loaded successfully");
    info!("\n{}", settings);

    // Initialize logging with configured level
    let log_level = match settings.log_level().parse::<tracing::Level>() {
        Ok(level) => level,
        Err(_) => tracing::Level::INFO,
    };

    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Create repository adapter using configuration
    info!("🔌 Connecting to database: {}", settings.database_url());
    let repository = match RepositoryAdapter::new(settings.database_url()).await {
        Ok(repo) => Arc::new(repo),
        Err(e) => {
            eprintln!("❌ Failed to create repository adapter: {}", e);
            return Err(e.into());
        }
    };

    info!("✅ Repository initialized successfully");

    // Create gRPC services with repository (Dependency Injection)
    let control_service = AuthorizationControlService::new(repository.clone());
    let data_service = AuthorizationDataService::new(repository.clone());

    // Configure gRPC server
    let mut server_builder = Server::builder();

    // Configure gRPC options from settings
    if let Some(max_frame_size) = settings.grpc_max_frame_size() {
        server_builder = server_builder.max_frame_size(max_frame_size as u32);
    }

    if let Some(keepalive_time) = settings.grpc_keepalive_time() {
        server_builder = server_builder
            .http2_keepalive_interval(Some(std::time::Duration::from_secs(keepalive_time)));
    }

    let addr = settings.server_address().parse()?;
    info!("🚀 Server listening on {}", addr);

    // Create a shutdown signal receiver
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Build and start server
    let server_future = server_builder
        .add_service(AuthorizationControlServer::new(control_service))
        .add_service(AuthorizationDataServer::new(data_service))
        .serve_with_shutdown(addr, async {
            let _ = shutdown_rx.await;
        });

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server_future.await {
            eprintln!("❌ Server error: {}", e);
        }
        info!("✅ Server stopped");
    });

    // Wait for Ctrl+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    info!("⏹️  Received shutdown signal");

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    // Wait for server to stop
    server_handle.await?;

    Ok(())
}
