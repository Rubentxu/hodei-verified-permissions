//! TODO Task Manager with Cedar Authorization
//!
//! This is a complete example application demonstrating Cedar authorization
//! with the Hodei Verified Permissions SDK.
//!
//! This version demonstrates SDK integration without the middleware layer.
//! The middleware layer requires additional Sync trait bounds that are being addressed.

mod handlers;
mod models;
mod storage;

use axum::{
    routing::{get, post},
    Router,
};
use hodei_permissions_sdk::AuthorizationClient;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::storage::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting TODO Task Manager with Cedar Authorization...");

    // Connect to authorization service first
    let auth_endpoint =
        std::env::var("AUTH_ENDPOINT").unwrap_or_else(|_| "http://localhost:50051".to_string());

    info!("Connecting to authorization service at {}", auth_endpoint);

    let client = AuthorizationClient::connect(auth_endpoint).await?;
    
    // Verify connection by creating a test policy store
    info!("Verifying connection to authorization service...");
    match client.create_policy_store(Some("TODO App Policy Store".to_string())).await {
        Ok(response) => {
            info!("✅ Successfully connected to authorization service");
            info!("   Policy Store ID: {}", response.policy_store_id);
        }
        Err(e) => {
            info!("⚠️  Warning: Could not verify authorization service: {}", e);
            info!("   Continuing without authorization checks");
        }
    }

    // Wrap client in Arc for sharing across handlers
    let auth_client = Arc::new(client);
    
    info!("✅ Authorization client configured and ready");

    // Initialize app state with sample data and auth client
    let state = AppState::with_sample_data(auth_client);
    info!("Loaded {} sample tasks", state.tasks.len());
    info!("Loaded {} sample projects", state.projects.len());

    // Load Cedar schema
    let _schema_json = include_str!("../v4.cedarschema.json");

    // Build API router with correct Axum 0.8 syntax
    let api_router = Router::new()
        // Task endpoints
        .route("/tasks", get(handlers::list_tasks).post(handlers::create_task))
        .route(
            "/tasks/{taskId}",
            get(handlers::get_task)
                .put(handlers::update_task)
                .delete(handlers::delete_task),
        )
        .route("/tasks/{taskId}/assign", post(handlers::assign_task))
        .route("/tasks/{taskId}/complete", post(handlers::complete_task))
        // Project endpoints
        .route(
            "/projects",
            get(handlers::list_projects).post(handlers::create_project),
        )
        .route(
            "/projects/{projectId}",
            get(handlers::get_project).delete(handlers::delete_project),
        )
        .with_state(state);

    // Build main router
    let app = Router::new()
        .route("/health", get(handlers::health))
        .nest("/api/v1", api_router)
        .layer(CorsLayer::permissive());

    // Start server (listen on all interfaces for Docker compatibility)
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);
    info!("");
    info!("API Endpoints:");
    info!("  GET    /health");
    info!("  GET    /api/v1/tasks");
    info!("  POST   /api/v1/tasks");
    info!("  GET    /api/v1/tasks/{{taskId}}");
    info!("  PUT    /api/v1/tasks/{{taskId}}");
    info!("  DELETE /api/v1/tasks/{{taskId}}");
    info!("  POST   /api/v1/tasks/{{taskId}}/assign?userId=<user>");
    info!("  POST   /api/v1/tasks/{{taskId}}/complete");
    info!("  GET    /api/v1/projects");
    info!("  POST   /api/v1/projects");
    info!("  GET    /api/v1/projects/{{projectId}}");
    info!("  DELETE /api/v1/projects/{{projectId}}");
    info!("");
    info!("Try:");
    info!("  curl http://localhost:3000/health");
    info!("  curl -H 'Authorization: Bearer <token>' http://localhost:3000/api/v1/tasks");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
