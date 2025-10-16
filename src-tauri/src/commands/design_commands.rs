use tauri::State;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::core::{AppState, EdtError};
use crate::models::{
    Assembly, Component, Feature, FeatureType, DistributionType,
    Mate, MateType, Stackup, AnalysisType, Supplier, Quote, CostDistribution,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// Assembly Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssemblyRequest {
    pub name: String,
    pub description: String,
    pub revision: String,
}

#[derive(Debug, Serialize)]
pub struct AssemblyResponse {
    pub assembly: Assembly,
}

#[derive(Debug, Serialize)]
pub struct AssemblyListResponse {
    pub assembly_ids: Vec<Uuid>,
}

#[tauri::command]
pub async fn create_assembly(
    state: State<'_, AppState>,
    request: CreateAssemblyRequest,
) -> Result<AssemblyResponse, String> {
    let assembly = state
        .entity_manager
        .create_assembly(
            request.name,
            request.description,
            request.revision,
        )
        .map_err(|e| e.to_string())?;

    Ok(AssemblyResponse { assembly })
}

#[tauri::command]
pub async fn get_assembly(
    state: State<'_, AppState>,
    assembly_id: String,
) -> Result<AssemblyResponse, String> {
    let id = Uuid::parse_str(&assembly_id).map_err(|e| e.to_string())?;

    let assembly = state
        .entity_manager
        .get_assembly(&id)
        .map_err(|e| e.to_string())?;

    Ok(AssemblyResponse { assembly })
}

#[tauri::command]
pub async fn update_assembly(
    state: State<'_, AppState>,
    assembly: Assembly,
) -> Result<AssemblyResponse, String> {
    let updated_assembly = state
        .entity_manager
        .update_assembly(assembly)
        .map_err(|e| e.to_string())?;

    Ok(AssemblyResponse { assembly: updated_assembly })
}

#[tauri::command]
pub async fn delete_assembly(
    state: State<'_, AppState>,
    assembly_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&assembly_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_assembly(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_assemblies(
    state: State<'_, AppState>,
) -> Result<AssemblyListResponse, String> {
    let assembly_ids = state
        .entity_manager
        .list_assembly_ids()
        .map_err(|e| e.to_string())?;

    Ok(AssemblyListResponse { assembly_ids })
}

// ============================================================================
// Component Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateComponentRequest {
    pub name: String,
    pub description: String,
    pub revision: String,
}

#[derive(Debug, Serialize)]
pub struct ComponentResponse {
    pub component: Component,
}

#[derive(Debug, Serialize)]
pub struct ComponentListResponse {
    pub component_ids: Vec<Uuid>,
}

#[tauri::command]
pub async fn create_component(
    state: State<'_, AppState>,
    request: CreateComponentRequest,
) -> Result<ComponentResponse, String> {
    let component = state
        .entity_manager
        .create_component(
            request.name,
            request.description,
            request.revision,
        )
        .map_err(|e| e.to_string())?;

    Ok(ComponentResponse { component })
}

#[tauri::command]
pub async fn get_component(
    state: State<'_, AppState>,
    component_id: String,
) -> Result<ComponentResponse, String> {
    let id = Uuid::parse_str(&component_id).map_err(|e| e.to_string())?;

    let component = state
        .entity_manager
        .get_component(&id)
        .map_err(|e| e.to_string())?;

    Ok(ComponentResponse { component })
}

#[tauri::command]
pub async fn update_component(
    state: State<'_, AppState>,
    component: Component,
) -> Result<ComponentResponse, String> {
    let updated_component = state
        .entity_manager
        .update_component(component)
        .map_err(|e| e.to_string())?;

    Ok(ComponentResponse { component: updated_component })
}

#[tauri::command]
pub async fn delete_component(
    state: State<'_, AppState>,
    component_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&component_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_component(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_components(
    state: State<'_, AppState>,
) -> Result<ComponentListResponse, String> {
    let component_ids = state
        .entity_manager
        .list_component_ids()
        .map_err(|e| e.to_string())?;

    Ok(ComponentListResponse { component_ids })
}

// ============================================================================
// Feature Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateFeatureRequest {
    pub name: String,
    pub description: String,
    pub feature_type: FeatureType,
    pub nominal: f64,
    pub upper_tolerance: f64,
    pub lower_tolerance: f64,
    pub distribution_type: DistributionType,
}

#[derive(Debug, Serialize)]
pub struct FeatureResponse {
    pub feature: Feature,
}

#[derive(Debug, Serialize)]
pub struct FeatureListResponse {
    pub feature_ids: Vec<Uuid>,
}

#[tauri::command]
pub async fn create_feature(
    state: State<'_, AppState>,
    request: CreateFeatureRequest,
) -> Result<FeatureResponse, String> {
    let feature = state
        .entity_manager
        .create_feature(
            request.name,
            request.description,
            request.feature_type,
            request.nominal,
            request.upper_tolerance,
            request.lower_tolerance,
            request.distribution_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(FeatureResponse { feature })
}

#[tauri::command]
pub async fn get_feature(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<FeatureResponse, String> {
    let id = Uuid::parse_str(&feature_id).map_err(|e| e.to_string())?;

    let feature = state
        .entity_manager
        .get_feature(&id)
        .map_err(|e| e.to_string())?;

    Ok(FeatureResponse { feature })
}

#[tauri::command]
pub async fn update_feature(
    state: State<'_, AppState>,
    feature: Feature,
) -> Result<FeatureResponse, String> {
    let updated_feature = state
        .entity_manager
        .update_feature(feature)
        .map_err(|e| e.to_string())?;

    Ok(FeatureResponse { feature: updated_feature })
}

#[tauri::command]
pub async fn delete_feature(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&feature_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_feature(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_features(
    state: State<'_, AppState>,
) -> Result<FeatureListResponse, String> {
    let feature_ids = state
        .entity_manager
        .list_feature_ids()
        .map_err(|e| e.to_string())?;

    Ok(FeatureListResponse { feature_ids })
}

// ============================================================================
// Mate Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateMateRequest {
    pub name: String,
    pub description: String,
    pub mate_type: MateType,
}

#[derive(Debug, Serialize)]
pub struct MateResponse {
    pub mate: Mate,
}

#[derive(Debug, Serialize)]
pub struct MateListResponse {
    pub mate_ids: Vec<Uuid>,
}

#[tauri::command]
pub async fn create_mate(
    state: State<'_, AppState>,
    request: CreateMateRequest,
) -> Result<MateResponse, String> {
    let mate = state
        .entity_manager
        .create_mate(
            request.name,
            request.description,
            request.mate_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(MateResponse { mate })
}

#[tauri::command]
pub async fn get_mate(
    state: State<'_, AppState>,
    mate_id: String,
) -> Result<MateResponse, String> {
    let id = Uuid::parse_str(&mate_id).map_err(|e| e.to_string())?;

    let mate = state
        .entity_manager
        .get_mate(&id)
        .map_err(|e| e.to_string())?;

    Ok(MateResponse { mate })
}

#[tauri::command]
pub async fn update_mate(
    state: State<'_, AppState>,
    mate: Mate,
) -> Result<MateResponse, String> {
    let updated_mate = state
        .entity_manager
        .update_mate(mate)
        .map_err(|e| e.to_string())?;

    Ok(MateResponse { mate: updated_mate })
}

#[tauri::command]
pub async fn delete_mate(
    state: State<'_, AppState>,
    mate_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&mate_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_mate(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_mates(
    state: State<'_, AppState>,
) -> Result<MateListResponse, String> {
    let mate_ids = state
        .entity_manager
        .list_mate_ids()
        .map_err(|e| e.to_string())?;

    Ok(MateListResponse { mate_ids })
}

// ============================================================================
// Stackup Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateStackupRequest {
    pub name: String,
    pub description: String,
    pub analysis_types: Vec<AnalysisType>,
}

#[derive(Debug, Serialize)]
pub struct StackupResponse {
    pub stackup: Stackup,
}

#[derive(Debug, Serialize)]
pub struct StackupListResponse {
    pub stackup_ids: Vec<Uuid>,
}

#[tauri::command]
pub async fn create_stackup(
    state: State<'_, AppState>,
    request: CreateStackupRequest,
) -> Result<StackupResponse, String> {
    let stackup = state
        .entity_manager
        .create_stackup(
            request.name,
            request.description,
            request.analysis_types,
        )
        .map_err(|e| e.to_string())?;

    Ok(StackupResponse { stackup })
}

#[tauri::command]
pub async fn get_stackup(
    state: State<'_, AppState>,
    stackup_id: String,
) -> Result<StackupResponse, String> {
    let id = Uuid::parse_str(&stackup_id).map_err(|e| e.to_string())?;

    let stackup = state
        .entity_manager
        .get_stackup(&id)
        .map_err(|e| e.to_string())?;

    Ok(StackupResponse { stackup })
}

#[tauri::command]
pub async fn update_stackup(
    state: State<'_, AppState>,
    stackup: Stackup,
) -> Result<StackupResponse, String> {
    let updated_stackup = state
        .entity_manager
        .update_stackup(stackup)
        .map_err(|e| e.to_string())?;

    Ok(StackupResponse { stackup: updated_stackup })
}

#[tauri::command]
pub async fn delete_stackup(
    state: State<'_, AppState>,
    stackup_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&stackup_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_stackup(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_stackups(
    state: State<'_, AppState>,
) -> Result<StackupListResponse, String> {
    let stackup_ids = state
        .entity_manager
        .list_stackup_ids()
        .map_err(|e| e.to_string())?;

    Ok(StackupListResponse { stackup_ids })
}

// ============================================================================
// Supplier Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct SupplierResponse {
    pub supplier: Supplier,
}

#[derive(Debug, Serialize)]
pub struct SupplierListResponse {
    pub supplier_ids: Vec<Uuid>,
}

#[tauri::command]
pub async fn create_supplier(
    state: State<'_, AppState>,
    request: CreateSupplierRequest,
) -> Result<SupplierResponse, String> {
    let supplier = state
        .entity_manager
        .create_supplier(
            request.name,
            request.description,
        )
        .map_err(|e| e.to_string())?;

    Ok(SupplierResponse { supplier })
}

#[tauri::command]
pub async fn get_supplier(
    state: State<'_, AppState>,
    supplier_id: String,
) -> Result<SupplierResponse, String> {
    let id = Uuid::parse_str(&supplier_id).map_err(|e| e.to_string())?;

    let supplier = state
        .entity_manager
        .get_supplier(&id)
        .map_err(|e| e.to_string())?;

    Ok(SupplierResponse { supplier })
}

#[tauri::command]
pub async fn update_supplier(
    state: State<'_, AppState>,
    supplier: Supplier,
) -> Result<SupplierResponse, String> {
    let updated_supplier = state
        .entity_manager
        .update_supplier(supplier)
        .map_err(|e| e.to_string())?;

    Ok(SupplierResponse { supplier: updated_supplier })
}

#[tauri::command]
pub async fn delete_supplier(
    state: State<'_, AppState>,
    supplier_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&supplier_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_supplier(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_suppliers(
    state: State<'_, AppState>,
) -> Result<SupplierListResponse, String> {
    let supplier_ids = state
        .entity_manager
        .list_supplier_ids()
        .map_err(|e| e.to_string())?;

    Ok(SupplierListResponse { supplier_ids })
}

// ============================================================================
// Quote Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateQuoteRequest {
    pub quote_number: String,
    pub quote_date: NaiveDate,
    pub quantity_price_pairs: Vec<(u32, f64)>,
    pub distribution_type: CostDistribution,
}

#[derive(Debug, Serialize)]
pub struct QuoteResponse {
    pub quote: Quote,
}

#[derive(Debug, Serialize)]
pub struct QuoteListResponse {
    pub quote_ids: Vec<Uuid>,
}

#[tauri::command]
pub async fn create_quote(
    state: State<'_, AppState>,
    request: CreateQuoteRequest,
) -> Result<QuoteResponse, String> {
    let quote = state
        .entity_manager
        .create_quote(
            request.quote_number,
            request.quote_date,
            request.quantity_price_pairs,
            request.distribution_type,
        )
        .map_err(|e| e.to_string())?;

    Ok(QuoteResponse { quote })
}

#[tauri::command]
pub async fn get_quote(
    state: State<'_, AppState>,
    quote_id: String,
) -> Result<QuoteResponse, String> {
    let id = Uuid::parse_str(&quote_id).map_err(|e| e.to_string())?;

    let quote = state
        .entity_manager
        .get_quote(&id)
        .map_err(|e| e.to_string())?;

    Ok(QuoteResponse { quote })
}

#[tauri::command]
pub async fn update_quote(
    state: State<'_, AppState>,
    quote: Quote,
) -> Result<QuoteResponse, String> {
    let updated_quote = state
        .entity_manager
        .update_quote(quote)
        .map_err(|e| e.to_string())?;

    Ok(QuoteResponse { quote: updated_quote })
}

#[tauri::command]
pub async fn delete_quote(
    state: State<'_, AppState>,
    quote_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&quote_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_quote(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_quotes(
    state: State<'_, AppState>,
) -> Result<QuoteListResponse, String> {
    let quote_ids = state
        .entity_manager
        .list_quote_ids()
        .map_err(|e| e.to_string())?;

    Ok(QuoteListResponse { quote_ids })
}
