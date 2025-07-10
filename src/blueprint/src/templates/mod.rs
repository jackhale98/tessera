use anyhow::Result;
use chrono::Local;
use include_dir::{include_dir, Dir};

static TEMPLATES_DIR: Dir = include_dir!("templates");

pub fn get_template(name: &str) -> Result<String> {
    let filename = format!("{}.ron", name);

    TEMPLATES_DIR
        .get_file(&filename)
        .and_then(|file| file.contents_utf8())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", name))
}

pub fn create_basic_project(name: &str) -> String {
    use crate::Project;
    
    let today = Local::now().date_naive();
    let project = Project::new(name.to_string(), today);
    
    let pretty = ron::ser::PrettyConfig::new()
        .depth_limit(4)
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    
    ron::ser::to_string_pretty(&project, pretty)
        .unwrap_or_else(|_| {
            // Fallback to a simple format if serialization fails
            format!(
                r#"(
    name: "{}",
    start_date: "{}",
    currency: "USD",
    description: None,
    resources: {{}},
    tasks: {{}},
    milestones: {{}},
    calendars: {{}},
    resource_calendars: [],
    default_calendar_id: None,
    baselines: {{}},
    current_baseline_id: None,
    progress_snapshots: {{}},
    issue_registry: (
        issues: {{}},
        issue_counter: 1,
        escalation_rules: [],
        sla_definitions: [],
    ),
    risk_registry: (
        risks: {{}},
        risk_counter: 1,
        risk_matrix: (
            probability_thresholds: {{}},
            impact_thresholds: {{}},
            risk_levels: {{}},
        ),
        risk_appetite: (
            low_threshold: 5,
            medium_threshold: 15,
            high_threshold: 20,
        ),
        review_cycle_days: 30,
    ),
)"#,
                name,
                today.format("%Y-%m-%d")
            )
        })
}

pub fn list_templates() -> Vec<String> {
    TEMPLATES_DIR
        .files()
        .filter_map(|file| {
            file.path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|s| s.to_string())
        })
        .collect()
}
