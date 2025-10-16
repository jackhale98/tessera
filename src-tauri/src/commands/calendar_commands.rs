use tauri::State;
use uuid::Uuid;
use chrono::Weekday;
use crate::core::AppState;
use crate::models::Calendar;
use serde::{Deserialize, Serialize};

/// Request to create a new calendar
#[derive(Debug, Deserialize)]
pub struct CreateCalendarRequest {
    pub name: String,
    pub work_hours_per_day: f64,
    pub work_days: Vec<Weekday>,
}

/// Response with calendar data
#[derive(Debug, Serialize)]
pub struct CalendarResponse {
    pub calendar: Calendar,
}

/// List of calendar IDs
#[derive(Debug, Serialize)]
pub struct CalendarListResponse {
    pub calendar_ids: Vec<Uuid>,
}

/// Create a new calendar
#[tauri::command]
pub async fn create_calendar(
    state: State<'_, AppState>,
    request: CreateCalendarRequest,
) -> Result<CalendarResponse, String> {
    let calendar = state
        .entity_manager
        .create_calendar(
            request.name,
            request.work_hours_per_day,
            request.work_days,
        )
        .map_err(|e| e.to_string())?;

    Ok(CalendarResponse { calendar })
}

/// Get a calendar by ID
#[tauri::command]
pub async fn get_calendar(
    state: State<'_, AppState>,
    calendar_id: String,
) -> Result<CalendarResponse, String> {
    let id = Uuid::parse_str(&calendar_id).map_err(|e| e.to_string())?;

    let calendar = state
        .entity_manager
        .get_calendar(&id)
        .map_err(|e| e.to_string())?;

    Ok(CalendarResponse { calendar })
}

/// Update a calendar
#[tauri::command]
pub async fn update_calendar(
    state: State<'_, AppState>,
    calendar: Calendar,
) -> Result<CalendarResponse, String> {
    let updated_calendar = state
        .entity_manager
        .update_calendar(calendar)
        .map_err(|e| e.to_string())?;

    Ok(CalendarResponse { calendar: updated_calendar })
}

/// Delete a calendar
#[tauri::command]
pub async fn delete_calendar(
    state: State<'_, AppState>,
    calendar_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&calendar_id).map_err(|e| e.to_string())?;

    state
        .entity_manager
        .delete_calendar(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// List all calendar IDs
#[tauri::command]
pub async fn list_calendars(
    state: State<'_, AppState>,
) -> Result<CalendarListResponse, String> {
    let calendar_ids = state
        .storage
        .list_ids(&crate::models::EntityType::Calendar)
        .map_err(|e| e.to_string())?;

    Ok(CalendarListResponse { calendar_ids })
}
