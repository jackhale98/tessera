use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::Requirement;
use serde::{Deserialize, Serialize};

/// Request to create a new requirement
#[derive(Debug, Deserialize)]
pub struct CreateRequirementRequest {
    pub name: String,
    pub description: String,
    pub requirement_type: String,
}

/// Response with requirement data
#[derive(Debug, Serialize)]
pub struct RequirementResponse {
    pub requirement: Requirement,
}

/// List of requirement IDs
#[derive(Debug, Serialize)]
pub struct RequirementListResponse {
    pub requirement_ids: Vec<Uuid>,
}

/// Create a new requirement
#[tauri::command]
pub async fn create_requirement(
    state: State<'_, AppState>,
    request: CreateRequirementRequest,
) -> Result<RequirementResponse, String> {
    let requirement = state
        .entity_manager
        .create_requirement(
            request.name,
            request.description,
            request.requirement_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(RequirementResponse { requirement })
}

/// Get a requirement by ID
#[tauri::command]
pub async fn get_requirement(
    state: State<'_, AppState>,
    requirement_id: String,
) -> Result<RequirementResponse, String> {
    let id = Uuid::parse_str(&requirement_id).map_err(|e| e.to_string())?;

    let requirement = state
        .entity_manager
        .get_requirement(&id)
        .map_err(|e| e.to_string())?;

    Ok(RequirementResponse { requirement })
}

/// Update a requirement
#[tauri::command]
pub async fn update_requirement(
    state: State<'_, AppState>,
    requirement: Requirement,
) -> Result<RequirementResponse, String> {
    let updated_requirement = state
        .entity_manager
        .update_requirement(requirement)
        .map_err(|e| e.to_string())?;

    Ok(RequirementResponse { requirement: updated_requirement })
}

/// Delete a requirement
#[tauri::command]
pub async fn delete_requirement(
    state: State<'_, AppState>,
    requirement_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&requirement_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_requirement(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all requirement IDs
#[tauri::command]
pub async fn list_requirements(
    state: State<'_, AppState>,
) -> Result<RequirementListResponse, String> {
    let requirement_ids = state
        .entity_manager
        .list_requirement_ids()
        .map_err(|e| e.to_string())?;

    Ok(RequirementListResponse { requirement_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
