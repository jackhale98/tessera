use tauri::State;
use uuid::Uuid;
use crate::core::AppState;
use crate::models::{Resource, ResourceType};
use serde::{Deserialize, Serialize};

/// Request to create a new resource
#[derive(Debug, Deserialize)]
pub struct CreateResourceRequest {
    pub name: String,
    pub description: String,
    pub resource_type: ResourceType,
    pub email: Option<String>,
    pub bill_rate: Option<f64>,
}

/// Response with resource data
#[derive(Debug, Serialize)]
pub struct ResourceResponse {
    pub resource: Resource,
}

/// List of resource IDs
#[derive(Debug, Serialize)]
pub struct ResourceListResponse {
    pub resource_ids: Vec<Uuid>,
}

/// Create a new resource
#[tauri::command]
pub async fn create_resource(
    state: State<'_, AppState>,
    request: CreateResourceRequest,
) -> Result<ResourceResponse, String> {
    let mut resource = state
        .entity_manager
        .create_resource(
            request.name,
            request.description,
            request.resource_type,
        )
        .map_err(|e| e.to_string())?;

    // Set optional fields
    resource.email = request.email;
    resource.bill_rate = request.bill_rate;

    // Update to persist optional fields
    let resource = state
        .entity_manager
        .update_resource(resource)
        .map_err(|e| e.to_string())?;

    Ok(ResourceResponse { resource })
}

/// Get a resource by ID
#[tauri::command]
pub async fn get_resource(
    state: State<'_, AppState>,
    resource_id: String,
) -> Result<ResourceResponse, String> {
    let id = Uuid::parse_str(&resource_id).map_err(|e| e.to_string())?;

    let resource = state
        .entity_manager
        .get_resource(&id)
        .map_err(|e| e.to_string())?;

    Ok(ResourceResponse { resource })
}

/// Update a resource
#[tauri::command]
pub async fn update_resource(
    state: State<'_, AppState>,
    resource: Resource,
) -> Result<ResourceResponse, String> {
    let updated_resource = state
        .entity_manager
        .update_resource(resource)
        .map_err(|e| e.to_string())?;

    Ok(ResourceResponse { resource: updated_resource })
}

/// Delete a resource
#[tauri::command]
pub async fn delete_resource(
    state: State<'_, AppState>,
    resource_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&resource_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_resource(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all resource IDs
#[tauri::command]
pub async fn list_resources(
    state: State<'_, AppState>,
) -> Result<ResourceListResponse, String> {
    let resource_ids = state
        .storage
        .list_ids(&crate::models::EntityType::Resource)
        .map_err(|e| e.to_string())?;

    Ok(ResourceListResponse { resource_ids })
}
