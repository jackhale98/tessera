use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::Hazard;
use serde::{Deserialize, Serialize};

/// Request to create a new hazard
#[derive(Debug, Deserialize)]
pub struct CreateHazardRequest {
    pub name: String,
    pub description: String,
    pub causes: Vec<String>,
    pub harms: Vec<String>,
}

/// Response with hazard data
#[derive(Debug, Serialize)]
pub struct HazardResponse {
    pub hazard: Hazard,
}

/// List of hazard IDs
#[derive(Debug, Serialize)]
pub struct HazardListResponse {
    pub hazard_ids: Vec<Uuid>,
}

/// Create a new hazard
#[tauri::command]
pub async fn create_hazard(
    state: State<'_, AppState>,
    request: CreateHazardRequest,
) -> Result<HazardResponse, String> {
    let hazard = state
        .entity_manager
        .create_hazard(
            request.name,
            request.description,
            request.causes,
            request.harms,
        )
        .map_err(|e| e.to_string())?;

    Ok(HazardResponse { hazard })
}

/// Get a hazard by ID
#[tauri::command]
pub async fn get_hazard(
    state: State<'_, AppState>,
    hazard_id: String,
) -> Result<HazardResponse, String> {
    let id = Uuid::parse_str(&hazard_id).map_err(|e| e.to_string())?;

    let hazard = state
        .entity_manager
        .get_hazard(&id)
        .map_err(|e| e.to_string())?;

    Ok(HazardResponse { hazard })
}

/// Update a hazard
#[tauri::command]
pub async fn update_hazard(
    state: State<'_, AppState>,
    hazard: Hazard,
) -> Result<HazardResponse, String> {
    let updated_hazard = state
        .entity_manager
        .update_hazard(hazard)
        .map_err(|e| e.to_string())?;

    Ok(HazardResponse { hazard: updated_hazard })
}

/// Delete a hazard
#[tauri::command]
pub async fn delete_hazard(
    state: State<'_, AppState>,
    hazard_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&hazard_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_hazard(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all hazard IDs
#[tauri::command]
pub async fn list_hazards(
    state: State<'_, AppState>,
) -> Result<HazardListResponse, String> {
    let hazard_ids = state
        .entity_manager
        .list_hazard_ids()
        .map_err(|e| e.to_string())?;

    Ok(HazardListResponse { hazard_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
