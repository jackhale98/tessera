use tauri::State;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::core::AppState;
use crate::models::Milestone;
use serde::{Deserialize, Serialize};

/// Request to create a new milestone
#[derive(Debug, Deserialize)]
pub struct CreateMilestoneRequest {
    pub name: String,
    pub description: String,
    pub date: DateTime<Utc>,
}

/// Response with milestone data
#[derive(Debug, Serialize)]
pub struct MilestoneResponse {
    pub milestone: Milestone,
}

/// List of milestone IDs
#[derive(Debug, Serialize)]
pub struct MilestoneListResponse {
    pub milestone_ids: Vec<Uuid>,
}

/// Create a new milestone
#[tauri::command]
pub async fn create_milestone(
    state: State<'_, AppState>,
    request: CreateMilestoneRequest,
) -> Result<MilestoneResponse, String> {
    let milestone = state
        .entity_manager
        .create_milestone(
            request.name,
            request.description,
            request.date,
        )
        .map_err(|e| e.to_string())?;

    Ok(MilestoneResponse { milestone })
}

/// Get a milestone by ID
#[tauri::command]
pub async fn get_milestone(
    state: State<'_, AppState>,
    milestone_id: String,
) -> Result<MilestoneResponse, String> {
    let id = Uuid::parse_str(&milestone_id).map_err(|e| e.to_string())?;

    let milestone = state
        .entity_manager
        .get_milestone(&id)
        .map_err(|e| e.to_string())?;

    Ok(MilestoneResponse { milestone })
}

/// Update a milestone
#[tauri::command]
pub async fn update_milestone(
    state: State<'_, AppState>,
    milestone: Milestone,
) -> Result<MilestoneResponse, String> {
    let updated_milestone = state
        .entity_manager
        .update_milestone(milestone)
        .map_err(|e| e.to_string())?;

    Ok(MilestoneResponse { milestone: updated_milestone })
}

/// Delete a milestone
#[tauri::command]
pub async fn delete_milestone(
    state: State<'_, AppState>,
    milestone_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&milestone_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_milestone(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all milestone IDs
#[tauri::command]
pub async fn list_milestones(
    state: State<'_, AppState>,
) -> Result<MilestoneListResponse, String> {
    let milestone_ids = state
        .storage
        .list_ids(&crate::models::EntityType::Milestone)
        .map_err(|e| e.to_string())?;

    Ok(MilestoneListResponse { milestone_ids })
}
