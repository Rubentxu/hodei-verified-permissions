//! Integration tests for TODO application
//!
//! These tests verify the core functionality without requiring
//! the authorization service to be running.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;

// Helper to create test app without auth middleware
fn create_test_app() -> Router {
    use todo_app::handlers;
    use todo_app::storage::AppState;
    use axum::routing::{get, post};

    let state = AppState::with_sample_data();

    Router::new()
        .route("/health", get(handlers::health))
        .route("/tasks", get(handlers::list_tasks).post(handlers::create_task))
        .route(
            "/tasks/{taskId}",
            get(handlers::get_task)
                .put(handlers::update_task)
                .delete(handlers::delete_task),
        )
        .route("/tasks/{taskId}/assign", post(handlers::assign_task))
        .route("/tasks/{taskId}/complete", post(handlers::complete_task))
        .route(
            "/projects",
            get(handlers::list_projects).post(handlers::create_project),
        )
        .route(
            "/projects/{projectId}",
            get(handlers::get_project).delete(handlers::delete_project),
        )
        .with_state(state)
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_tasks() {
    let app = create_test_app();

    let response = app
        .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["tasks"].is_array());
    assert_eq!(json["total"], 3); // Sample data has 3 tasks
}

#[tokio::test]
async fn test_list_tasks_with_filter() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks?status=pending")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["tasks"].is_array());
    // Should have pending tasks from sample data
    assert!(json["total"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_create_task() {
    let app = create_test_app();

    let new_task = json!({
        "title": "Test Task",
        "description": "This is a test task"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_task).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["title"], "Test Task");
    assert_eq!(json["description"], "This is a test task");
    assert_eq!(json["status"], "pending");
}

#[tokio::test]
async fn test_get_task_not_found() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks/nonexistent-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_update_task() {
    let app = create_test_app();
    
    // First, get the list of tasks to get a valid ID
    let list_response = app
        .clone()
        .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = json["tasks"][0]["id"].as_str().unwrap();
    
    // Now update the task
    let update = json!({
        "title": "Updated Title",
        "status": "inprogress"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/tasks/{}", task_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&update).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["title"], "Updated Title");
    assert_eq!(json["status"], "inprogress");
}

#[tokio::test]
async fn test_complete_task() {
    let app = create_test_app();
    
    // Get a task ID
    let list_response = app
        .clone()
        .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = json["tasks"][0]["id"].as_str().unwrap();
    
    // Complete the task
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/tasks/{}/complete", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "completed");
}

#[tokio::test]
async fn test_assign_task() {
    let app = create_test_app();
    
    // Get a task ID
    let list_response = app
        .clone()
        .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = json["tasks"][0]["id"].as_str().unwrap();
    
    // Assign the task
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/tasks/{}/assign?userId=testuser", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["assignee"], "testuser");
}

#[tokio::test]
async fn test_delete_task() {
    let app = create_test_app();
    
    // Get a task ID
    let list_response = app
        .clone()
        .oneshot(Request::builder().uri("/tasks").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = json["tasks"][0]["id"].as_str().unwrap();
    
    // Delete the task
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/tasks/{}", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_list_projects() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["projects"].is_array());
    assert_eq!(json["total"], 2); // Sample data has 2 projects
}

#[tokio::test]
async fn test_create_project() {
    let app = create_test_app();

    let new_project = json!({
        "name": "Test Project",
        "description": "This is a test project"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_project).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["name"], "Test Project");
    assert_eq!(json["description"], "This is a test project");
}

#[tokio::test]
async fn test_delete_project() {
    let app = create_test_app();
    
    // Get a project ID
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let project_id = json["projects"][0]["id"].as_str().unwrap();
    
    // Delete the project
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/projects/{}", project_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
