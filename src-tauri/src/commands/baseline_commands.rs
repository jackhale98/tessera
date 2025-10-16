use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::Baseline;
use serde::{Deserialize, Serialize};

/// Request to create a new baseline
#[derive(Debug, Deserialize)]
pub struct CreateBaselineRequest {
    pub name: String,
    pub description: String,
    pub task_ids: Vec<Uuid>,
}

/// Response with baseline data
#[derive(Debug, Serialize)]
pub struct BaselineResponse {
    pub baseline: Baseline,
}

/// List of baseline IDs
#[derive(Debug, Serialize)]
pub struct BaselineListResponse {
    pub baseline_ids: Vec<Uuid>,
}

/// Create a new baseline
#[tauri::command]
pub async fn create_baseline(
    state: State<'_, AppState>,
    request: CreateBaselineRequest,
) -> Result<BaselineResponse, String> {
    let baseline = state
        .entity_manager
        .create_baseline(
            request.name,
            request.description,
            request.task_ids,
        )
        .map_err(|e| e.to_string())?;

    Ok(BaselineResponse { baseline })
}

/// Get a baseline by ID
#[tauri::command]
pub async fn get_baseline(
    state: State<'_, AppState>,
    baseline_id: String,
) -> Result<BaselineResponse, String> {
    let id = Uuid::parse_str(&baseline_id).map_err(|e| e.to_string())?;

    let baseline = state
        .entity_manager
        .get_baseline(&id)
        .map_err(|e| e.to_string())?;

    Ok(BaselineResponse { baseline })
}

/// Update a baseline
#[tauri::command]
pub async fn update_baseline(
    state: State<'_, AppState>,
    baseline: Baseline,
) -> Result<BaselineResponse, String> {
    let updated_baseline = state
        .entity_manager
        .update_baseline(baseline)
        .map_err(|e| e.to_string())?;

    Ok(BaselineResponse { baseline: updated_baseline })
}

/// Delete a baseline
#[tauri::command]
pub async fn delete_baseline(
    state: State<'_, AppState>,
    baseline_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&baseline_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_baseline(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all baseline IDs
#[tauri::command]
pub async fn list_baselines(
    state: State<'_, AppState>,
) -> Result<BaselineListResponse, String> {
    let baseline_ids = state
        .storage
        .list_ids(&crate::models::EntityType::Baseline)
        .map_err(|e| e.to_string())?;

    Ok(BaselineListResponse { baseline_ids })
}
