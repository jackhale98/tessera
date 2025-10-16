use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::{Manufacturing, WorkInstructionStep};
use serde::{Deserialize, Serialize};

/// Request to create a new manufacturing process
#[derive(Debug, Deserialize)]
pub struct CreateManufacturingRequest {
    pub name: String,
    pub description: String,
    pub process_type: String,
    pub work_instructions: Vec<WorkInstructionStep>,
    pub priority: u32,
}

/// Response with manufacturing data
#[derive(Debug, Serialize)]
pub struct ManufacturingResponse {
    pub manufacturing: Manufacturing,
}

/// List of manufacturing IDs
#[derive(Debug, Serialize)]
pub struct ManufacturingListResponse {
    pub manufacturing_ids: Vec<Uuid>,
}

/// Create a new manufacturing process
#[tauri::command]
pub async fn create_manufacturing(
    state: State<'_, AppState>,
    request: CreateManufacturingRequest,
) -> Result<ManufacturingResponse, String> {
    let manufacturing = state
        .entity_manager
        .create_manufacturing(
            request.name,
            request.description,
            request.process_type,
            request.work_instructions,
            request.priority,
        )
        .map_err(|e| e.to_string())?;

    Ok(ManufacturingResponse { manufacturing })
}

/// Get a manufacturing process by ID
#[tauri::command]
pub async fn get_manufacturing(
    state: State<'_, AppState>,
    manufacturing_id: String,
) -> Result<ManufacturingResponse, String> {
    let id = Uuid::parse_str(&manufacturing_id).map_err(|e| e.to_string())?;

    let manufacturing = state
        .entity_manager
        .get_manufacturing(&id)
        .map_err(|e| e.to_string())?;

    Ok(ManufacturingResponse { manufacturing })
}

/// Update a manufacturing process
#[tauri::command]
pub async fn update_manufacturing(
    state: State<'_, AppState>,
    manufacturing: Manufacturing,
) -> Result<ManufacturingResponse, String> {
    let updated_manufacturing = state
        .entity_manager
        .update_manufacturing(manufacturing)
        .map_err(|e| e.to_string())?;

    Ok(ManufacturingResponse { manufacturing: updated_manufacturing })
}

/// Delete a manufacturing process
#[tauri::command]
pub async fn delete_manufacturing(
    state: State<'_, AppState>,
    manufacturing_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&manufacturing_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_manufacturing(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all manufacturing process IDs
#[tauri::command]
pub async fn list_manufacturing(
    state: State<'_, AppState>,
) -> Result<ManufacturingListResponse, String> {
    let manufacturing_ids = state
        .entity_manager
        .list_manufacturing_ids()
        .map_err(|e| e.to_string())?;

    Ok(ManufacturingListResponse { manufacturing_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
