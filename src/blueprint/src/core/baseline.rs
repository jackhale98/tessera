use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBaseline {
    pub baseline_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_date: NaiveDateTime,
    pub created_by: String,
    pub is_current: bool,
    pub baseline_type: BaselineType,
    pub project_snapshot: BaselineProjectSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineType {
    Initial,     // Original approved baseline
    Approved,    // Board/stakeholder approved changes
    Working,     // Current working baseline
    Archived,    // Historical baseline
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineProjectSnapshot {
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_cost: f32,
    pub total_effort_hours: f32,
    pub tasks: IndexMap<Uuid, BaselineTask>,
    pub milestones: IndexMap<Uuid, BaselineMilestone>,
    pub resources: IndexMap<Uuid, BaselineResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineTask {
    pub task_id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub duration_days: i32,
    pub effort_hours: f32,
    pub cost: f32,
    pub assigned_resources: Vec<String>,
    pub dependencies: Vec<String>,
    pub work_breakdown_structure: Option<String>,
    pub task_type: crate::core::TaskType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMilestone {
    pub milestone_id: Uuid,
    pub name: String,
    pub target_date: NaiveDate,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResource {
    pub resource_id: Uuid,
    pub name: String,
    pub hourly_rate: f32,
    pub total_allocated_hours: f32,
    pub total_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub current_baseline: String,
    pub comparison_baseline: String,
    pub generated_date: NaiveDateTime,
    pub schedule_variance_days: i32,
    pub cost_variance: f32,
    pub effort_variance_hours: f32,
    pub task_changes: Vec<TaskVariance>,
    pub milestone_changes: Vec<MilestoneVariance>,
    pub summary: BaselineVarianceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskVariance {
    pub task_id: String,
    pub task_name: String,
    pub variance_type: VarianceType,
    pub schedule_variance_days: i32,
    pub cost_variance: f32,
    pub effort_variance_hours: f32,
    pub baseline_start: NaiveDate,
    pub baseline_end: NaiveDate,
    pub current_start: Option<NaiveDate>,
    pub current_end: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneVariance {
    pub milestone_id: String,
    pub milestone_name: String,
    pub baseline_date: NaiveDate,
    pub current_date: Option<NaiveDate>,
    pub variance_days: i32,
    pub status: MilestoneStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarianceType {
    NoChange,
    ScheduleVariance,
    CostVariance,
    ScopeChange,
    TaskAdded,
    TaskRemoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    OnTrack,
    AtRisk,
    Delayed,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineVarianceSummary {
    pub total_tasks_changed: u32,
    pub total_milestones_at_risk: u32,
    pub schedule_performance_index: f32, // SPI = EV / PV
    pub cost_performance_index: f32,     // CPI = EV / AC
    pub overall_health: ProjectHealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectHealth {
    Green,   // On track
    Yellow,  // At risk but manageable
    Red,     // Critical issues
}

impl ProjectBaseline {
    pub fn new(
        baseline_id: Uuid,
        name: String,
        created_by: String,
        baseline_type: BaselineType,
        project: &crate::core::Project,
        schedule: &crate::scheduling::Schedule,
    ) -> Self {
        let project_snapshot = BaselineProjectSnapshot::from_project_and_schedule(project, schedule);
        
        Self {
            baseline_id,
            name,
            description: None,
            created_date: chrono::Utc::now().naive_utc(),
            created_by,
            is_current: baseline_type == BaselineType::Working,
            baseline_type,
            project_snapshot,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_as_current(&mut self) {
        self.is_current = true;
    }

    pub fn archive(&mut self) {
        self.is_current = false;
        self.baseline_type = BaselineType::Archived;
    }

    pub fn compare_to(&self, other: &ProjectBaseline) -> BaselineComparison {
        let mut task_changes = Vec::new();
        let mut milestone_changes = Vec::new();

        // Compare tasks
        for (task_id, current_task) in &self.project_snapshot.tasks {
            if let Some(baseline_task) = other.project_snapshot.tasks.get(task_id) {
                let variance = self.calculate_task_variance(current_task, baseline_task);
                if variance.variance_type != VarianceType::NoChange {
                    task_changes.push(variance);
                }
            } else {
                // Task was added
                task_changes.push(TaskVariance {
                    task_id: task_id.to_string(),
                    task_name: current_task.name.clone(),
                    variance_type: VarianceType::TaskAdded,
                    schedule_variance_days: 0,
                    cost_variance: current_task.cost,
                    effort_variance_hours: current_task.effort_hours,
                    baseline_start: current_task.start_date,
                    baseline_end: current_task.end_date,
                    current_start: Some(current_task.start_date),
                    current_end: Some(current_task.end_date),
                });
            }
        }

        // Check for removed tasks
        for (task_id, baseline_task) in &other.project_snapshot.tasks {
            if !self.project_snapshot.tasks.contains_key(task_id) {
                task_changes.push(TaskVariance {
                    task_id: task_id.to_string(),
                    task_name: baseline_task.name.clone(),
                    variance_type: VarianceType::TaskRemoved,
                    schedule_variance_days: 0,
                    cost_variance: -baseline_task.cost,
                    effort_variance_hours: -baseline_task.effort_hours,
                    baseline_start: baseline_task.start_date,
                    baseline_end: baseline_task.end_date,
                    current_start: None,
                    current_end: None,
                });
            }
        }

        // Compare milestones
        for (milestone_id, current_milestone) in &self.project_snapshot.milestones {
            if let Some(baseline_milestone) = other.project_snapshot.milestones.get(milestone_id) {
                let variance_days = (current_milestone.target_date - baseline_milestone.target_date).num_days() as i32;
                let status = if variance_days == 0 {
                    MilestoneStatus::OnTrack
                } else if variance_days > 0 && variance_days <= 5 {
                    MilestoneStatus::AtRisk
                } else if variance_days > 5 {
                    MilestoneStatus::Delayed
                } else {
                    MilestoneStatus::OnTrack // Earlier than planned
                };

                if variance_days != 0 {
                    milestone_changes.push(MilestoneVariance {
                        milestone_id: milestone_id.to_string(),
                        milestone_name: current_milestone.name.clone(),
                        baseline_date: baseline_milestone.target_date,
                        current_date: Some(current_milestone.target_date),
                        variance_days,
                        status,
                    });
                }
            }
        }

        // Calculate summary metrics
        let schedule_variance_days = (self.project_snapshot.end_date - other.project_snapshot.end_date).num_days() as i32;
        let cost_variance = self.project_snapshot.total_cost - other.project_snapshot.total_cost;
        let effort_variance_hours = self.project_snapshot.total_effort_hours - other.project_snapshot.total_effort_hours;

        let summary = BaselineVarianceSummary {
            total_tasks_changed: task_changes.len() as u32,
            total_milestones_at_risk: milestone_changes.iter()
                .filter(|m| matches!(m.status, MilestoneStatus::AtRisk | MilestoneStatus::Delayed))
                .count() as u32,
            schedule_performance_index: 1.0, // Placeholder - would need earned value data
            cost_performance_index: 1.0,     // Placeholder - would need earned value data
            overall_health: self.calculate_project_health(&task_changes, &milestone_changes),
        };

        BaselineComparison {
            current_baseline: self.baseline_id.to_string(),
            comparison_baseline: other.baseline_id.to_string(),
            generated_date: chrono::Utc::now().naive_utc(),
            schedule_variance_days,
            cost_variance,
            effort_variance_hours,
            task_changes,
            milestone_changes,
            summary,
        }
    }

    fn calculate_task_variance(&self, current: &BaselineTask, baseline: &BaselineTask) -> TaskVariance {
        let schedule_variance_days = (current.end_date - baseline.end_date).num_days() as i32;
        let cost_variance = current.cost - baseline.cost;
        let effort_variance_hours = current.effort_hours - baseline.effort_hours;

        let variance_type = if schedule_variance_days.abs() > 0 {
            VarianceType::ScheduleVariance
        } else if cost_variance.abs() > 0.01 {
            VarianceType::CostVariance
        } else {
            VarianceType::NoChange
        };

        TaskVariance {
            task_id: current.task_id.to_string(),
            task_name: current.name.clone(),
            variance_type,
            schedule_variance_days,
            cost_variance,
            effort_variance_hours,
            baseline_start: baseline.start_date,
            baseline_end: baseline.end_date,
            current_start: Some(current.start_date),
            current_end: Some(current.end_date),
        }
    }

    fn calculate_project_health(&self, task_changes: &[TaskVariance], milestone_changes: &[MilestoneVariance]) -> ProjectHealth {
        let critical_task_changes = task_changes.iter()
            .filter(|t| t.schedule_variance_days > 5 || t.cost_variance > 1000.0)
            .count();

        let critical_milestone_delays = milestone_changes.iter()
            .filter(|m| matches!(m.status, MilestoneStatus::Delayed))
            .count();

        if critical_task_changes > 0 || critical_milestone_delays > 0 {
            ProjectHealth::Red
        } else if task_changes.len() > 3 || milestone_changes.len() > 1 {
            ProjectHealth::Yellow
        } else {
            ProjectHealth::Green
        }
    }
}

impl BaselineProjectSnapshot {
    pub fn from_project_and_schedule(
        project: &crate::core::Project,
        schedule: &crate::scheduling::Schedule,
    ) -> Self {
        let mut tasks = IndexMap::new();
        let mut milestones = IndexMap::new();
        let mut resources = IndexMap::new();

        // Convert scheduled tasks to baseline tasks
        for (task_id, scheduled_task) in &schedule.tasks {
            if let Some(project_task) = project.tasks.get(task_id) {
                let baseline_task = BaselineTask {
                    task_id: uuid::Uuid::new_v4(),
                    name: scheduled_task.name.clone(),
                    start_date: scheduled_task.start_date,
                    end_date: scheduled_task.end_date,
                    duration_days: (scheduled_task.end_date - scheduled_task.start_date).num_days() as i32 + 1,
                    effort_hours: scheduled_task.effort,
                    cost: scheduled_task.cost,
                    assigned_resources: vec![scheduled_task.assigned_to.clone()],
                    dependencies: project_task.get_dependency_ids(),
                    work_breakdown_structure: None,
                    task_type: project_task.task_type,
                };
                let baseline_uuid = uuid::Uuid::new_v4();
                tasks.insert(baseline_uuid, baseline_task);
            }
        }

        // Convert scheduled milestones to baseline milestones
        for (milestone_id, scheduled_milestone) in &schedule.milestones {
            if let Some(project_milestone) = project.milestones.get(milestone_id) {
                let baseline_milestone = BaselineMilestone {
                    milestone_id: uuid::Uuid::new_v4(),
                    name: scheduled_milestone.name.clone(),
                    target_date: scheduled_milestone.date,
                    dependencies: project_milestone.dependencies.clone(),
                };
                let milestone_uuid = uuid::Uuid::new_v4();
                milestones.insert(milestone_uuid, baseline_milestone);
            }
        }

        // Convert resource utilization to baseline resources
        for (resource_id, utilization) in &schedule.resource_utilization {
            if let Some(project_resource) = project.resources.get(resource_id) {
                let baseline_resource = BaselineResource {
                    resource_id: uuid::Uuid::new_v4(),
                    name: utilization.name.clone(),
                    hourly_rate: project_resource.hourly_rate,
                    total_allocated_hours: utilization.total_hours,
                    total_cost: utilization.total_hours * project_resource.hourly_rate,
                };
                let resource_uuid = uuid::Uuid::new_v4();
                resources.insert(resource_uuid, baseline_resource);
            }
        }

        Self {
            project_name: schedule.project_name.clone(),
            start_date: schedule.start_date,
            end_date: schedule.end_date,
            total_cost: schedule.total_cost,
            total_effort_hours: tasks.values().map(|t| t.effort_hours).sum(),
            tasks,
            milestones,
            resources,
        }
    }
}

impl Default for BaselineType {
    fn default() -> Self {
        BaselineType::Working
    }
}

impl std::fmt::Display for BaselineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaselineType::Initial => write!(f, "Initial"),
            BaselineType::Approved => write!(f, "Approved"),
            BaselineType::Working => write!(f, "Working"),
            BaselineType::Archived => write!(f, "Archived"),
        }
    }
}

impl std::fmt::Display for ProjectHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectHealth::Green => write!(f, "Green"),
            ProjectHealth::Yellow => write!(f, "Yellow"),
            ProjectHealth::Red => write!(f, "Red"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_baseline_creation() {
        let project = crate::core::Project::new(
            "Test Project".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
        
        let schedule = crate::scheduling::Schedule {
            project_name: "Test Project".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            tasks: IndexMap::new(),
            milestones: IndexMap::new(),
            critical_path: Vec::new(),
            total_cost: 1000.0,
            resource_utilization: HashMap::new(),
        };

        let baseline = ProjectBaseline::new(
            uuid::Uuid::new_v4(),
            "Initial Baseline".to_string(),
            "project_manager".to_string(),
            BaselineType::Initial,
            &project,
            &schedule,
        );

        assert!(!baseline.baseline_id.to_string().is_empty());
        assert_eq!(baseline.name, "Initial Baseline");
        assert_eq!(baseline.baseline_type, BaselineType::Initial);
        // Note: baseline.is_current depends on baseline_type
        assert_eq!(baseline.baseline_type, BaselineType::Initial);
    }

    #[test]
    fn test_baseline_comparison() {
        let project = crate::core::Project::new(
            "Test Project".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
        
        let schedule1 = crate::scheduling::Schedule {
            project_name: "Test Project".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            tasks: IndexMap::new(),
            milestones: IndexMap::new(),
            critical_path: Vec::new(),
            total_cost: 1000.0,
            resource_utilization: HashMap::new(),
        };

        let schedule2 = crate::scheduling::Schedule {
            project_name: "Test Project".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), // 5 days delay
            tasks: IndexMap::new(),
            milestones: IndexMap::new(),
            critical_path: Vec::new(),
            total_cost: 1200.0, // Cost overrun
            resource_utilization: HashMap::new(),
        };

        let baseline1 = ProjectBaseline::new(
            uuid::Uuid::new_v4(),
            "Original".to_string(),
            "pm".to_string(),
            BaselineType::Initial,
            &project,
            &schedule1,
        );

        let baseline2 = ProjectBaseline::new(
            uuid::Uuid::new_v4(),
            "Updated".to_string(),
            "pm".to_string(),
            BaselineType::Working,
            &project,
            &schedule2,
        );

        let comparison = baseline2.compare_to(&baseline1);
        
        assert_eq!(comparison.schedule_variance_days, 5);
        assert_eq!(comparison.cost_variance, 200.0);
    }
}