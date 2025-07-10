pub mod cli;
pub mod core;
pub mod git;
pub mod reporting;
pub mod scheduling;
pub mod templates;

pub use core::{Project, Task, Resource, Milestone};
pub use scheduling::{Schedule, SchedulingEngine};

/// Hook for future user interface implementations
pub trait UserInterface {
    fn display_project_overview(&self, project: &Project);
    fn edit_task_interactively(&self, task: &mut Task) -> anyhow::Result<()>;
    fn show_gantt_chart(&self, schedule: &Schedule);
}

/// Hook for future optimization algorithms
pub trait SchedulingAlgorithm {
    fn optimize_schedule(&self, project: &Project) -> anyhow::Result<Schedule>;
}

/// Hook for future time tracking backends
pub trait TimeTrackingBackend {
    fn fetch_entries(&self, date_range: &chrono::NaiveDate) -> anyhow::Result<Vec<TimeEntry>>;
    fn sync_to_project(&self, project: &mut Project) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct TimeEntry {
    pub task_id: String,
    pub resource_id: String,
    pub hours: f32,
    pub date: chrono::NaiveDate,
}
