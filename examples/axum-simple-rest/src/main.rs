//! Axum example with SimpleRest mapping and Cedar authorization
//!
//! This example demonstrates how to use the Hodei Verified Permissions SDK
//! with Axum to automatically resolve Cedar actions from HTTP requests.

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use hodei_permissions_sdk::{
    middleware::VerifiedPermissionsLayer,
    AuthorizationClient,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateDocumentRequest {
    title: String,
    content: String,
}

// Handlers
async fn health() -> &'static str {
    "OK"
}

async fn list_documents() -> Json<Vec<Document>> {
    info!("Listing documents");
    Json(vec![
        Document {
            id: "1".to_string(),
            title: "First Document".to_string(),
            content: "Content of first document".to_string(),
        },
        Document {
            id: "2".to_string(),
            title: "Second Document".to_string(),
            content: "Content of second document".to_string(),
        },
    ])
}

async fn get_document(Path(id): Path<String>) -> Result<Json<Document>, AppError> {
    info!("Getting document: {}", id);
    Ok(Json(Document {
        id: id.clone(),
        title: format!("Document {}", id),
        content: format!("Content of document {}", id),
    }))
}

async fn create_document(
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<Document>), AppError> {
    info!("Creating document: {}", payload.title);
    Ok((
        StatusCode::CREATED,
        Json(Document {
            id: "new-id".to_string(),
            title: payload.title,
            content: payload.content,
        }),
    ))
}

async fn update_document(
    Path(id): Path<String>,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, AppError> {
    info!("Updating document: {}", id);
    Ok(Json(Document {
        id,
        title: payload.title,
        content: payload.content,
    }))
}

async fn delete_document(Path(id): Path<String>) -> Result<StatusCode, AppError> {
    info!("Deleting document: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

async fn share_document(Path(id): Path<String>) -> Result<Json<String>, AppError> {
    info!("Sharing document: {}", id);
    Ok(Json(format!("Document {} shared successfully", id)))
}

// Error handling
#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Axum server with Cedar authorization...");

    // Load Cedar schema (generated from OpenAPI)
    let schema_json = include_str!("../v4.cedarschema.json");

    // Connect to authorization service
    // In production, use environment variables for the endpoint
    let auth_endpoint = std::env::var("AUTH_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:50051".to_string());
    
    info!("Connecting to authorization service at {}", auth_endpoint);
    
    let client = AuthorizationClient::connect(auth_endpoint).await?;

    // Configure authorization layer with SimpleRest mapping
    let auth_layer = VerifiedPermissionsLayer::new(
        client,
        "policy-store-id", // In production, use env var
        "identity-source-id", // In production, use env var
    )
    .with_simple_rest_mapping(schema_json)?
    .skip_endpoint("get", "/health") // Health check doesn't need auth
    .skip_prefix("get", "/public/"); // Public endpoints

    // Build application router
    let app = Router::new()
        // Public endpoints
        .route("/health", get(health))
        // Protected endpoints (will use Cedar authorization)
        .route("/documents", get(list_documents).post(create_document))
        .route(
            "/documents/:id",
            get(get_document)
                .put(update_document)
                .delete(delete_document),
        )
        .route("/documents/:id/share", post(share_document))
        // Apply authorization layer
        .layer(auth_layer);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);
    info!("Try:");
    info!("  curl http://localhost:3000/health");
    info!("  curl -H 'Authorization: Bearer <token>' http://localhost:3000/documents");
    info!("  curl -H 'Authorization: Bearer <token>' http://localhost:3000/documents/123");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
