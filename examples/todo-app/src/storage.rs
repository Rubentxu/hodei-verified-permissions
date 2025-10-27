//! In-memory storage for tasks and projects

use crate::models::{Project, Task, TaskStatus};
use dashmap::DashMap;
use hodei_permissions_sdk::AuthorizationClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub tasks: Arc<DashMap<String, Task>>,
    pub projects: Arc<DashMap<String, Project>>,
    pub auth_client: Arc<AuthorizationClient>,
}

impl AppState {
    pub fn new(auth_client: Arc<AuthorizationClient>) -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            projects: Arc::new(DashMap::new()),
            auth_client,
        }
    }

    pub fn with_sample_data(auth_client: Arc<AuthorizationClient>) -> Self {
        let state = Self::new(auth_client);
        
        // Create sample projects
        let project1 = Project::new(
            "Website Redesign".to_string(),
            Some("Redesign company website".to_string()),
            "alice".to_string(),
        );
        let project2 = Project::new(
            "Mobile App".to_string(),
            Some("Develop mobile application".to_string()),
            "bob".to_string(),
        );
        
        state.projects.insert(project1.id.clone(), project1.clone());
        state.projects.insert(project2.id.clone(), project2.clone());
        
        // Create sample tasks
        let mut task1 = Task::new(
            "Design homepage mockup".to_string(),
            Some("Create initial design for homepage".to_string()),
            "alice".to_string(),
        );
        task1.project_id = Some(project1.id.clone());
        task1.assignee = Some("charlie".to_string());
        
        let mut task2 = Task::new(
            "Implement login page".to_string(),
            Some("Build authentication UI".to_string()),
            "bob".to_string(),
        );
        task2.project_id = Some(project2.id.clone());
        task2.assignee = Some("bob".to_string());
        task2.status = TaskStatus::InProgress;
        
        let task3 = Task::new(
            "Write API documentation".to_string(),
            Some("Document all API endpoints".to_string()),
            "alice".to_string(),
        );
        
        state.tasks.insert(task1.id.clone(), task1);
        state.tasks.insert(task2.id.clone(), task2);
        state.tasks.insert(task3.id.clone(), task3);
        
        state
    }

    pub fn get_task(&self, id: &str) -> Option<Task> {
        self.tasks.get(id).map(|t| t.clone())
    }

    pub fn get_project(&self, id: &str) -> Option<Project> {
        self.projects.get(id).map(|p| p.clone())
    }

    pub fn list_tasks(&self, status: Option<TaskStatus>, assignee: Option<String>) -> Vec<Task> {
        self.tasks
            .iter()
            .filter(|entry| {
                let task = entry.value();
                let status_match = status.as_ref().map_or(true, |s| &task.status == s);
                let assignee_match = assignee.as_ref().map_or(true, |a| {
                    task.assignee.as_ref().map_or(false, |ta| ta == a)
                });
                status_match && assignee_match
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn list_projects(&self) -> Vec<Project> {
        self.projects
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn create_task(&self, task: Task) -> Task {
        let id = task.id.clone();
        self.tasks.insert(id, task.clone());
        task
    }

    pub fn update_task(&self, id: &str, task: Task) -> Option<Task> {
        self.tasks.insert(id.to_string(), task.clone());
        Some(task)
    }

    pub fn delete_task(&self, id: &str) -> Option<Task> {
        self.tasks.remove(id).map(|(_, task)| task)
    }

    pub fn create_project(&self, project: Project) -> Project {
        let id = project.id.clone();
        self.projects.insert(id, project.clone());
        project
    }

    pub fn delete_project(&self, id: &str) -> Option<Project> {
        // Also delete all tasks in this project
        let task_ids: Vec<String> = self.tasks
            .iter()
            .filter(|entry| {
                entry.value().project_id.as_ref().map_or(false, |pid| pid == id)
            })
            .map(|entry| entry.key().clone())
            .collect();
        
        for task_id in task_ids {
            self.tasks.remove(&task_id);
        }
        
        self.projects.remove(id).map(|(_, project)| project)
    }
}
