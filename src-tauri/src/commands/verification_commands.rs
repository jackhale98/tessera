use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::{Verification, TestPriority, TestStep};
use serde::{Deserialize, Serialize};

/// Request to create a new verification
#[derive(Debug, Deserialize)]
pub struct CreateVerificationRequest {
    pub name: String,
    pub description: String,
    pub test_type: String,
    pub test_steps: Vec<TestStep>,
    pub acceptance_criteria: Vec<String>,
    pub priority: TestPriority,
}

/// Response with verification data
#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    pub verification: Verification,
}

/// List of verification IDs
#[derive(Debug, Serialize)]
pub struct VerificationListResponse {
    pub verification_ids: Vec<Uuid>,
}

/// Create a new verification
#[tauri::command]
pub async fn create_verification(
    state: State<'_, AppState>,
    request: CreateVerificationRequest,
) -> Result<VerificationResponse, String> {
    let verification = state
        .entity_manager
        .create_verification(
            request.name,
            request.description,
            request.test_type,
            request.test_steps,
            request.acceptance_criteria,
            request.priority,
        )
        .map_err(|e| e.to_string())?;

    Ok(VerificationResponse { verification })
}

/// Get a verification by ID
#[tauri::command]
pub async fn get_verification(
    state: State<'_, AppState>,
    verification_id: String,
) -> Result<VerificationResponse, String> {
    let id = Uuid::parse_str(&verification_id).map_err(|e| e.to_string())?;

    let verification = state
        .entity_manager
        .get_verification(&id)
        .map_err(|e| e.to_string())?;

    Ok(VerificationResponse { verification })
}

/// Update a verification
#[tauri::command]
pub async fn update_verification(
    state: State<'_, AppState>,
    verification: Verification,
) -> Result<VerificationResponse, String> {
    let updated_verification = state
        .entity_manager
        .update_verification(verification)
        .map_err(|e| e.to_string())?;

    Ok(VerificationResponse { verification: updated_verification })
}

/// Delete a verification
#[tauri::command]
pub async fn delete_verification(
    state: State<'_, AppState>,
    verification_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&verification_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_verification(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all verification IDs
#[tauri::command]
pub async fn list_verifications(
    state: State<'_, AppState>,
) -> Result<VerificationListResponse, String> {
    let verification_ids = state
        .entity_manager
        .list_verification_ids()
        .map_err(|e| e.to_string())?;

    Ok(VerificationListResponse { verification_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
