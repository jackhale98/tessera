// Module declarations
mod core;
mod models;
mod storage;
mod commands;
mod utils;

// Re-exports
pub use core::{EdtError, EdtResult, AppState};
use commands::{
    create_task, get_task, update_task, delete_task, list_tasks,
    create_milestone, get_milestone, update_milestone, delete_milestone, list_milestones,
    create_resource, get_resource, update_resource, delete_resource, list_resources,
    create_calendar, get_calendar, update_calendar, delete_calendar, list_calendars,
    create_baseline, get_baseline, update_baseline, delete_baseline, list_baselines,
    calculate_critical_path, calculate_evm,
    calculate_worst_case, calculate_rss, calculate_monte_carlo,
    generate_bom,
    create_assembly, get_assembly, update_assembly, delete_assembly, list_assemblies,
    create_component, get_component, update_component, delete_component, list_components,
    create_feature, get_feature, update_feature, delete_feature, list_features,
    create_mate, get_mate, update_mate, delete_mate, list_mates,
    create_stackup, get_stackup, update_stackup, delete_stackup, list_stackups,
    create_supplier, get_supplier, update_supplier, delete_supplier, list_suppliers,
    create_quote, get_quote, update_quote, delete_quote, list_quotes,
    create_verification, get_verification, update_verification, delete_verification, list_verifications,
    create_validation, get_validation, update_validation, delete_validation, list_validations,
    create_manufacturing, get_manufacturing, update_manufacturing, delete_manufacturing, list_manufacturing,
    create_requirement, get_requirement, update_requirement, delete_requirement, list_requirements,
    create_risk, get_risk, update_risk, delete_risk, list_risks,
    create_hazard, get_hazard, update_hazard, delete_hazard, list_hazards,
    create_risk_control, get_risk_control, update_risk_control, delete_risk_control, list_risk_controls,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize app state with a default project directory
    // In a real app, this would be selected by the user
    let project_root = std::env::current_dir()
        .unwrap()
        .join("edt_project");

    let app_state = match AppState::new(project_root) {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize app state: {}", e);
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Task commands
            create_task,
            get_task,
            update_task,
            delete_task,
            list_tasks,
            // Milestone commands
            create_milestone,
            get_milestone,
            update_milestone,
            delete_milestone,
            list_milestones,
            // Resource commands
            create_resource,
            get_resource,
            update_resource,
            delete_resource,
            list_resources,
            // Calendar commands
            create_calendar,
            get_calendar,
            update_calendar,
            delete_calendar,
            list_calendars,
            // Baseline commands
            create_baseline,
            get_baseline,
            update_baseline,
            delete_baseline,
            list_baselines,
            // Calculation commands
            calculate_critical_path,
            calculate_evm,
            // Tolerance analysis commands
            calculate_worst_case,
            calculate_rss,
            calculate_monte_carlo,
            // BOM generation commands
            generate_bom,
            // Design commands - Assembly
            create_assembly,
            get_assembly,
            update_assembly,
            delete_assembly,
            list_assemblies,
            // Design commands - Component
            create_component,
            get_component,
            update_component,
            delete_component,
            list_components,
            // Design commands - Feature
            create_feature,
            get_feature,
            update_feature,
            delete_feature,
            list_features,
            // Design commands - Mate
            create_mate,
            get_mate,
            update_mate,
            delete_mate,
            list_mates,
            // Design commands - Stackup
            create_stackup,
            get_stackup,
            update_stackup,
            delete_stackup,
            list_stackups,
            // Design commands - Supplier
            create_supplier,
            get_supplier,
            update_supplier,
            delete_supplier,
            list_suppliers,
            // Design commands - Quote
            create_quote,
            get_quote,
            update_quote,
            delete_quote,
            list_quotes,
            // Verification commands
            create_verification,
            get_verification,
            update_verification,
            delete_verification,
            list_verifications,
            // Validation commands
            create_validation,
            get_validation,
            update_validation,
            delete_validation,
            list_validations,
            // Manufacturing commands
            create_manufacturing,
            get_manufacturing,
            update_manufacturing,
            delete_manufacturing,
            list_manufacturing,
            // Requirement commands
            create_requirement,
            get_requirement,
            update_requirement,
            delete_requirement,
            list_requirements,
            // Risk commands
            create_risk,
            get_risk,
            update_risk,
            delete_risk,
            list_risks,
            // Hazard commands
            create_hazard,
            get_hazard,
            update_hazard,
            delete_hazard,
            list_hazards,
            // RiskControl commands
            create_risk_control,
            get_risk_control,
            update_risk_control,
            delete_risk_control,
            list_risk_controls,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
