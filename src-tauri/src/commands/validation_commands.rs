use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::{Validation, TestPriority};
use serde::{Deserialize, Serialize};

/// Request to create a new validation
#[derive(Debug, Deserialize)]
pub struct CreateValidationRequest {
    pub name: String,
    pub description: String,
    pub validation_type: String,
    pub participants: Vec<String>,
    pub success_criteria: Vec<String>,
    pub priority: TestPriority,
}

/// Response with validation data
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub validation: Validation,
}

/// List of validation IDs
#[derive(Debug, Serialize)]
pub struct ValidationListResponse {
    pub validation_ids: Vec<Uuid>,
}

/// Create a new validation
#[tauri::command]
pub async fn create_validation(
    state: State<'_, AppState>,
    request: CreateValidationRequest,
) -> Result<ValidationResponse, String> {
    let validation = state
        .entity_manager
        .create_validation(
            request.name,
            request.description,
            request.validation_type,
            request.participants,
            request.success_criteria,
            request.priority,
        )
        .map_err(|e| e.to_string())?;

    Ok(ValidationResponse { validation })
}

/// Get a validation by ID
#[tauri::command]
pub async fn get_validation(
    state: State<'_, AppState>,
    validation_id: String,
) -> Result<ValidationResponse, String> {
    let id = Uuid::parse_str(&validation_id).map_err(|e| e.to_string())?;

    let validation = state
        .entity_manager
        .get_validation(&id)
        .map_err(|e| e.to_string())?;

    Ok(ValidationResponse { validation })
}

/// Update a validation
#[tauri::command]
pub async fn update_validation(
    state: State<'_, AppState>,
    validation: Validation,
) -> Result<ValidationResponse, String> {
    let updated_validation = state
        .entity_manager
        .update_validation(validation)
        .map_err(|e| e.to_string())?;

    Ok(ValidationResponse { validation: updated_validation })
}

/// Delete a validation
#[tauri::command]
pub async fn delete_validation(
    state: State<'_, AppState>,
    validation_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&validation_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_validation(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all validation IDs
#[tauri::command]
pub async fn list_validations(
    state: State<'_, AppState>,
) -> Result<ValidationListResponse, String> {
    let validation_ids = state
        .entity_manager
        .list_validation_ids()
        .map_err(|e| e.to_string())?;

    Ok(ValidationListResponse { validation_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
