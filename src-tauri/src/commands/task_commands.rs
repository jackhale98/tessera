use tauri::State;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::core::{AppState, EdtError};
use crate::models::{Task, TaskType};
use serde::{Deserialize, Serialize};

/// Request to create a new task
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: String,
    pub scheduled_start: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub task_type: TaskType,
}

/// Response with task data
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub task: Task,
}

/// List of task IDs
#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub task_ids: Vec<Uuid>,
}

/// Create a new task
#[tauri::command]
pub async fn create_task(
    state: State<'_, AppState>,
    request: CreateTaskRequest,
) -> Result<TaskResponse, String> {
    let task = state
        .entity_manager
        .create_task(
            request.name,
            request.description,
            request.scheduled_start,
            request.deadline,
            request.task_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(TaskResponse { task })
}

/// Get a task by ID
#[tauri::command]
pub async fn get_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<TaskResponse, String> {
    let id = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;

    let task = state
        .entity_manager
        .get_task(&id)
        .map_err(|e| e.to_string())?;

    Ok(TaskResponse { task })
}

/// Update a task
#[tauri::command]
pub async fn update_task(
    state: State<'_, AppState>,
    task: Task,
) -> Result<TaskResponse, String> {
    let updated_task = state
        .entity_manager
        .update_task(task)
        .map_err(|e| e.to_string())?;

    Ok(TaskResponse { task: updated_task })
}

/// Delete a task
#[tauri::command]
pub async fn delete_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_task(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all task IDs
#[tauri::command]
pub async fn list_tasks(
    state: State<'_, AppState>,
) -> Result<TaskListResponse, String> {
    let task_ids = state
        .entity_manager
        .list_task_ids()
        .map_err(|e| e.to_string())?;

    Ok(TaskListResponse { task_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
