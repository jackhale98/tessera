use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::Risk;
use serde::{Deserialize, Serialize};

/// Request to create a new risk
#[derive(Debug, Deserialize)]
pub struct CreateRiskRequest {
    pub name: String,
    pub description: String,
    pub risk_type: String,
    pub probability: u32,
    pub severity: u32,
}

/// Response with risk data
#[derive(Debug, Serialize)]
pub struct RiskResponse {
    pub risk: Risk,
}

/// List of risk IDs
#[derive(Debug, Serialize)]
pub struct RiskListResponse {
    pub risk_ids: Vec<Uuid>,
}

/// Create a new risk
#[tauri::command]
pub async fn create_risk(
    state: State<'_, AppState>,
    request: CreateRiskRequest,
) -> Result<RiskResponse, String> {
    let risk = state
        .entity_manager
        .create_risk(
            request.name,
            request.description,
            request.risk_type,
            request.probability,
            request.severity,
        )
        .map_err(|e| e.to_string())?;

    Ok(RiskResponse { risk })
}

/// Get a risk by ID
#[tauri::command]
pub async fn get_risk(
    state: State<'_, AppState>,
    risk_id: String,
) -> Result<RiskResponse, String> {
    let id = Uuid::parse_str(&risk_id).map_err(|e| e.to_string())?;

    let risk = state
        .entity_manager
        .get_risk(&id)
        .map_err(|e| e.to_string())?;

    Ok(RiskResponse { risk })
}

/// Update a risk
#[tauri::command]
pub async fn update_risk(
    state: State<'_, AppState>,
    risk: Risk,
) -> Result<RiskResponse, String> {
    let updated_risk = state
        .entity_manager
        .update_risk(risk)
        .map_err(|e| e.to_string())?;

    Ok(RiskResponse { risk: updated_risk })
}

/// Delete a risk
#[tauri::command]
pub async fn delete_risk(
    state: State<'_, AppState>,
    risk_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&risk_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_risk(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all risk IDs
#[tauri::command]
pub async fn list_risks(
    state: State<'_, AppState>,
) -> Result<RiskListResponse, String> {
    let risk_ids = state
        .entity_manager
        .list_risk_ids()
        .map_err(|e| e.to_string())?;

    Ok(RiskListResponse { risk_ids })
}

// Note: Command tests are covered by EntityManager tests
// Integration tests with Tauri State will be added later
