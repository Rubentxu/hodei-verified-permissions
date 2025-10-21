//! HTTP handlers for the TODO API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use hodei_macros::cedar_action;
use serde::Deserialize;
use tracing::info;

use crate::models::*;
use crate::storage::AppState;

// Error handling
#[derive(Debug)]
pub struct AppError(anyhow::Error);

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

// Query parameters
#[derive(Debug, Deserialize)]
pub struct TaskListQuery {
    pub status: Option<String>,
    pub assignee: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTaskQuery {
    #[serde(rename = "userId")]
    pub user_id: String,
}

// Task handlers
#[cedar_action(
    action = "listTasks",
    resource = "Application",
    description = "List all tasks with optional filters"
)]
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskListQuery>,
) -> Result<Json<TaskListResponse>, AppError> {
    info!("Listing tasks with filters: status={:?}, assignee={:?}", query.status, query.assignee);
    
    let status = query.status.and_then(|s| match s.as_str() {
        "pending" => Some(TaskStatus::Pending),
        "in_progress" => Some(TaskStatus::InProgress),
        "completed" => Some(TaskStatus::Completed),
        _ => None,
    });
    
    let tasks = state.list_tasks(status, query.assignee);
    let total = tasks.len();
    
    Ok(Json(TaskListResponse { tasks, total }))
}

#[cedar_action(
    action = "getTask",
    resource = "Task",
    description = "Get a specific task by ID"
)]
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Task>, AppError> {
    info!("Getting task: {}", task_id);
    
    state
        .get_task(&task_id)
        .map(Json)
        .ok_or_else(|| anyhow::anyhow!("Task not found").into())
}

#[cedar_action(
    action = "createTask",
    resource = "Application",
    description = "Create a new task"
)]
pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    info!("Creating task: {}", payload.title);
    
    // In a real app, get user from JWT token
    let user_id = "current_user".to_string();
    
    let mut task = Task::new(payload.title, payload.description, user_id);
    task.project_id = payload.project_id;
    
    let task = state.create_task(task);
    
    Ok((StatusCode::CREATED, Json(task)))
}

#[cedar_action(
    action = "updateTask",
    resource = "Task",
    description = "Update task details"
)]
pub async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    info!("Updating task: {}", task_id);
    
    let mut task = state
        .get_task(&task_id)
        .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
    
    if let Some(title) = payload.title {
        task.title = title;
    }
    if let Some(description) = payload.description {
        task.description = Some(description);
    }
    if let Some(status) = payload.status {
        task.status = status;
    }
    
    task.updated_at = chrono::Utc::now();
    
    state
        .update_task(&task_id, task)
        .map(Json)
        .ok_or_else(|| anyhow::anyhow!("Failed to update task").into())
}

#[cedar_action(
    action = "deleteTask",
    resource = "Task",
    description = "Delete a task"
)]
pub async fn delete_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, AppError> {
    info!("Deleting task: {}", task_id);
    
    state
        .delete_task(&task_id)
        .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
    
    Ok(StatusCode::NO_CONTENT)
}

#[cedar_action(
    action = "assignTask",
    resource = "Task",
    description = "Assign task to a user"
)]
pub async fn assign_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(query): Query<AssignTaskQuery>,
) -> Result<Json<Task>, AppError> {
    info!("Assigning task {} to user {}", task_id, query.user_id);
    
    let mut task = state
        .get_task(&task_id)
        .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
    
    task.assignee = Some(query.user_id);
    task.updated_at = chrono::Utc::now();
    
    state
        .update_task(&task_id, task)
        .map(Json)
        .ok_or_else(|| anyhow::anyhow!("Failed to assign task").into())
}

#[cedar_action(
    action = "completeTask",
    resource = "Task",
    description = "Mark task as completed"
)]
pub async fn complete_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Task>, AppError> {
    info!("Completing task: {}", task_id);
    
    let mut task = state
        .get_task(&task_id)
        .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
    
    task.status = TaskStatus::Completed;
    task.updated_at = chrono::Utc::now();
    
    state
        .update_task(&task_id, task)
        .map(Json)
        .ok_or_else(|| anyhow::anyhow!("Failed to complete task").into())
}

// Project handlers
#[cedar_action(
    action = "listProjects",
    resource = "Application",
    description = "List all projects"
)]
pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<ProjectListResponse>, AppError> {
    info!("Listing projects");
    
    let projects = state.list_projects();
    let total = projects.len();
    
    Ok(Json(ProjectListResponse { projects, total }))
}

#[cedar_action(
    action = "getProject",
    resource = "Project",
    description = "Get project details"
)]
pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Project>, AppError> {
    info!("Getting project: {}", project_id);
    
    state
        .get_project(&project_id)
        .map(Json)
        .ok_or_else(|| anyhow::anyhow!("Project not found").into())
}

#[cedar_action(
    action = "createProject",
    resource = "Application",
    description = "Create a new project"
)]
pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    info!("Creating project: {}", payload.name);
    
    // In a real app, get user from JWT token
    let user_id = "current_user".to_string();
    
    let project = Project::new(payload.name, payload.description, user_id);
    let project = state.create_project(project);
    
    Ok((StatusCode::CREATED, Json(project)))
}

#[cedar_action(
    action = "deleteProject",
    resource = "Project",
    description = "Delete a project and all its tasks"
)]
pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, AppError> {
    info!("Deleting project: {}", project_id);
    
    state
        .delete_project(&project_id)
        .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
    
    Ok(StatusCode::NO_CONTENT)
}

// Health check
pub async fn health() -> &'static str {
    "OK"
}
