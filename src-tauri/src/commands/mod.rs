// Tauri command handlers (API layer)
pub mod task_commands;
pub mod milestone_commands;
pub mod resource_commands;
pub mod calendar_commands;
pub mod baseline_commands;
pub mod calculation_commands;
pub mod design_commands;

pub use task_commands::{create_task, get_task, update_task, delete_task, list_tasks};
pub use milestone_commands::{create_milestone, get_milestone, update_milestone, delete_milestone, list_milestones};
pub use resource_commands::{create_resource, get_resource, update_resource, delete_resource, list_resources};
pub use calendar_commands::{create_calendar, get_calendar, update_calendar, delete_calendar, list_calendars};
pub use baseline_commands::{create_baseline, get_baseline, update_baseline, delete_baseline, list_baselines};
pub use calculation_commands::{
    calculate_critical_path, calculate_evm,
    calculate_worst_case, calculate_rss, calculate_monte_carlo,
    generate_bom,
};
pub use design_commands::{
    create_assembly, get_assembly, update_assembly, delete_assembly, list_assemblies,
    create_component, get_component, update_component, delete_component, list_components,
    create_feature, get_feature, update_feature, delete_feature, list_features,
    create_mate, get_mate, update_mate, delete_mate, list_mates,
    create_stackup, get_stackup, update_stackup, delete_stackup, list_stackups,
    create_supplier, get_supplier, update_supplier, delete_supplier, list_suppliers,
    create_quote, get_quote, update_quote, delete_quote, list_quotes,
};
