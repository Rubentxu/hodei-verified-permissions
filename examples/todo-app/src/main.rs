//! TODO Task Manager with Cedar Authorization
//!
//! This is a complete example application demonstrating Cedar authorization
//! with the Hodei Verified Permissions SDK.

mod handlers;
mod models;
mod storage;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
// TODO: Re-enable when Axum 0.8 compatibility is fixed
// use hodei_permissions_sdk::{middleware::VerifiedPermissionsLayer, AuthorizationClient};
use std::net::SocketAddr;
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

    // Initialize app state with sample data
    let state = AppState::with_sample_data();
    info!("Loaded {} sample tasks", state.tasks.len());
    info!("Loaded {} sample projects", state.projects.len());

    // Load Cedar schema
    let schema_json = include_str!("../v4.cedarschema.json");

    // TODO: Re-enable authorization middleware when Axum 0.8 compatibility is fixed
    // Connect to authorization service
    // let auth_endpoint =
    //     std::env::var("AUTH_ENDPOINT").unwrap_or_else(|_| "http://localhost:50051".to_string());

    // info!("Connecting to authorization service at {}", auth_endpoint);

    // let client = AuthorizationClient::connect(auth_endpoint).await?;

    // Configure authorization layer with SimpleRest mapping
    // let auth_layer = VerifiedPermissionsLayer::new(
    //     client,
    //     std::env::var("POLICY_STORE_ID").unwrap_or_else(|_| "todo-policy-store".to_string()),
    //     std::env::var("IDENTITY_SOURCE_ID")
    //             .unwrap_or_else(|_| "todo-identity-source".to_string()),
    // )
    // .with_simple_rest_mapping(schema_json)?
    // .skip_endpoint("get", "/health"); // Health check doesn't need auth
    
    info!("⚠️  Running WITHOUT authorization middleware (Axum 0.8 compatibility pending)");

    // Build API router
    let api_router = Router::new()
        // Task endpoints
        .route("/tasks", get(handlers::list_tasks).post(handlers::create_task))
        .route(
            "/tasks/:taskId",
            get(handlers::get_task)
                .put(handlers::update_task)
                .delete(handlers::delete_task),
        )
        .route("/tasks/:taskId/assign", post(handlers::assign_task))
        .route("/tasks/:taskId/complete", post(handlers::complete_task))
        // Project endpoints
        .route(
            "/projects",
            get(handlers::list_projects).post(handlers::create_project),
        )
        .route(
            "/projects/:projectId",
            get(handlers::get_project).delete(handlers::delete_project),
        )
        .with_state(state);

    // Build main router
    let app = Router::new()
        .route("/health", get(handlers::health))
        .nest("/api/v1", api_router)
        .layer(CorsLayer::permissive());
        // .layer(auth_layer);  // TODO: Re-enable when Axum 0.8 compatibility is fixed

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);
    info!("");
    info!("API Endpoints:");
    info!("  GET    /health");
    info!("  GET    /api/v1/tasks");
    info!("  POST   /api/v1/tasks");
    info!("  GET    /api/v1/tasks/:taskId");
    info!("  PUT    /api/v1/tasks/:taskId");
    info!("  DELETE /api/v1/tasks/:taskId");
    info!("  POST   /api/v1/tasks/:taskId/assign?userId=<user>");
    info!("  POST   /api/v1/tasks/:taskId/complete");
    info!("  GET    /api/v1/projects");
    info!("  POST   /api/v1/projects");
    info!("  GET    /api/v1/projects/:projectId");
    info!("  DELETE /api/v1/projects/:projectId");
    info!("");
    info!("Try:");
    info!("  curl http://localhost:3000/health");
    info!("  curl -H 'Authorization: Bearer <token>' http://localhost:3000/api/v1/tasks");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
