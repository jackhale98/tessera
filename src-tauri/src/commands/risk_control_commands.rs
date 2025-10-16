use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::RiskControl;
use serde::{Deserialize, Serialize};

/// Request to create a new risk control
#[derive(Debug, Deserialize)]
pub struct CreateRiskControlRequest {
    pub name: String,
    pub description: String,
    pub control_type: String,
}

/// Response with risk control data
#[derive(Debug, Serialize)]
pub struct RiskControlResponse {
    pub risk_control: RiskControl,
}

/// List of risk control IDs
#[derive(Debug, Serialize)]
pub struct RiskControlListResponse {
    pub risk_control_ids: Vec<Uuid>,
}

/// Create a new risk control
#[tauri::command]
pub async fn create_risk_control(
    state: State<'_, AppState>,
    request: CreateRiskControlRequest,
) -> Result<RiskControlResponse, String> {
    let risk_control = state
        .entity_manager
        .create_risk_control(
            request.name,
            request.description,
            request.control_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(RiskControlResponse { risk_control })
}

/// Get a risk control by ID
#[tauri::command]
pub async fn get_risk_control(
    state: State<'_, AppState>,
    risk_control_id: String,
) -> Result<RiskControlResponse, String> {
    let id = Uuid::parse_str(&risk_control_id).map_err(|e| e.to_string())?;

    let risk_control = state
        .entity_manager
        .get_risk_control(&id)
        .map_err(|e| e.to_string())?;

    Ok(RiskControlResponse { risk_control })
}

/// Update a risk control
#[tauri::command]
pub async fn update_risk_control(
    state: State<'_, AppState>,
    risk_control: RiskControl,
) -> Result<RiskControlResponse, String> {
    let updated_risk_control = state
        .entity_manager
        .update_risk_control(risk_control)
        .map_err(|e| e.to_string())?;

    Ok(RiskControlResponse { risk_control: updated_risk_control })
}

/// Delete a risk control
#[tauri::command]
pub async fn delete_risk_control(
    state: State<'_, AppState>,
    risk_control_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&risk_control_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_risk_control(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all risk control IDs
#[tauri::command]
pub async fn list_risk_controls(
    state: State<'_, AppState>,
) -> Result<RiskControlListResponse, String> {
    let risk_control_ids = state
        .entity_manager
        .list_risk_control_ids()
        .map_err(|e| e.to_string())?;

    Ok(RiskControlListResponse { risk_control_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
