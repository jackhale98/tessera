use tauri::State;
use uuid::Uuid;
use crate::core::{AppState, CriticalPathResult, EvmMetrics};
use crate::models::{StackupResult, MonteCarloResult, BomResult};
use serde::{Deserialize, Serialize};

/// Response with critical path analysis results
#[derive(Debug, Serialize)]
pub struct CriticalPathResponse {
    pub result: CriticalPathResult,
}

/// Response with EVM metrics
#[derive(Debug, Serialize)]
pub struct EvmResponse {
    pub metrics: EvmMetrics,
}

/// Calculate critical path using CPM algorithm
#[tauri::command]
pub async fn calculate_critical_path(
    state: State<'_, AppState>,
) -> Result<CriticalPathResponse, String> {
    let result = state
        .calculation_engine
        .calculate_critical_path()
        .map_err(|e| e.to_string())?;

    Ok(CriticalPathResponse { result })
}

/// Calculate Earned Value Management metrics
#[tauri::command]
pub async fn calculate_evm(
    state: State<'_, AppState>,
) -> Result<EvmResponse, String> {
    let metrics = state
        .calculation_engine
        .calculate_evm()
        .map_err(|e| e.to_string())?;

    Ok(EvmResponse { metrics })
}

// ============================================================================
// Tolerance Analysis Commands
// ============================================================================

/// Response with worst case tolerance analysis
#[derive(Debug, Serialize)]
pub struct WorstCaseResponse {
    pub result: StackupResult,
}

/// Response with RSS tolerance analysis
#[derive(Debug, Serialize)]
pub struct RssResponse {
    pub result: StackupResult,
}

/// Response with Monte Carlo tolerance analysis
#[derive(Debug, Serialize)]
pub struct MonteCarloResponse {
    pub result: MonteCarloResult,
}

/// Request for Monte Carlo simulation
#[derive(Debug, Deserialize)]
pub struct MonteCarloRequest {
    pub stackup_id: String,
    pub iterations: usize,
}

/// Calculate worst case tolerance for a stackup
#[tauri::command]
pub async fn calculate_worst_case(
    state: State<'_, AppState>,
    stackup_id: String,
) -> Result<WorstCaseResponse, String> {
    let id = Uuid::parse_str(&stackup_id).map_err(|e| e.to_string())?;

    let result = state
        .calculation_engine
        .calculate_worst_case(&id)
        .map_err(|e| e.to_string())?;

    Ok(WorstCaseResponse { result })
}

/// Calculate RSS (Root Sum Squared) tolerance for a stackup
#[tauri::command]
pub async fn calculate_rss(
    state: State<'_, AppState>,
    stackup_id: String,
) -> Result<RssResponse, String> {
    let id = Uuid::parse_str(&stackup_id).map_err(|e| e.to_string())?;

    let result = state
        .calculation_engine
        .calculate_rss(&id)
        .map_err(|e| e.to_string())?;

    Ok(RssResponse { result })
}

/// Calculate Monte Carlo tolerance simulation for a stackup
#[tauri::command]
pub async fn calculate_monte_carlo(
    state: State<'_, AppState>,
    request: MonteCarloRequest,
) -> Result<MonteCarloResponse, String> {
    let id = Uuid::parse_str(&request.stackup_id).map_err(|e| e.to_string())?;

    let result = state
        .calculation_engine
        .calculate_monte_carlo(&id, request.iterations)
        .map_err(|e| e.to_string())?;

    Ok(MonteCarloResponse { result })
}

// ============================================================================
// BOM Generation Commands
// ============================================================================

/// Response with BOM generation result
#[derive(Debug, Serialize)]
pub struct BomResponse {
    pub result: BomResult,
}

/// Request for BOM generation
#[derive(Debug, Deserialize)]
pub struct GenerateBomRequest {
    pub assembly_id: String,
    pub volume: u32,
}

/// Generate a Bill of Materials (BOM) for an assembly
#[tauri::command]
pub async fn generate_bom(
    state: State<'_, AppState>,
    request: GenerateBomRequest,
) -> Result<BomResponse, String> {
    let id = Uuid::parse_str(&request.assembly_id).map_err(|e| e.to_string())?;

    let result = state
        .calculation_engine
        .generate_bom(&id, request.volume)
        .map_err(|e| e.to_string())?;

    Ok(BomResponse { result })
}
