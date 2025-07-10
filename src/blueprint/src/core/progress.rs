use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSnapshot {
    pub snapshot_id: Uuid,
    pub project_name: String,
    pub status_date: NaiveDate,
    pub recorded_by: String,
    pub recorded_at: NaiveDateTime,
    pub overall_status: ProjectStatus,
    pub task_progress: IndexMap<Uuid, TaskProgress>,
    pub milestone_progress: IndexMap<Uuid, MilestoneProgress>,
    pub resource_actuals: IndexMap<Uuid, ResourceActuals>,
    pub issues_summary: IssuesSummary,
    pub risks_summary: RisksSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: Uuid,
    pub name: String,
    pub status: TaskStatus,
    pub percent_complete: f32, // 0.0 to 100.0
    pub actual_start: Option<NaiveDate>,
    pub actual_end: Option<NaiveDate>,
    pub actual_effort_hours: f32,
    pub actual_cost: f32,
    pub estimated_completion_date: Option<NaiveDate>,
    pub remaining_effort_hours: f32,
    pub notes: Option<String>,
    pub last_updated: NaiveDateTime,
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProgress {
    pub milestone_id: Uuid,
    pub name: String,
    pub status: MilestoneStatus,
    pub actual_date: Option<NaiveDate>,
    pub forecast_date: Option<NaiveDate>,
    pub completion_percentage: f32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceActuals {
    pub resource_id: Uuid,
    pub name: String,
    pub actual_hours_to_date: f32,
    pub actual_cost_to_date: f32,
    pub current_allocation: f32, // 0.0 to 1.0
    pub availability_notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Green,      // On track
    Yellow,     // At risk but manageable
    Red,        // Critical issues requiring attention
    Completed,  // Project successfully completed
    OnHold,     // Project temporarily suspended
    Cancelled,  // Project terminated
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneStatus {
    Pending,
    AtRisk,
    Achieved,
    Missed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuesSummary {
    pub total_open: u32,
    pub critical_count: u32,
    pub high_count: u32,
    pub medium_count: u32,
    pub low_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisksSummary {
    pub total_active: u32,
    pub high_probability_high_impact: u32,
    pub risks_requiring_action: u32,
    pub risks_being_monitored: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnedValueMetrics {
    pub status_date: NaiveDate,
    pub project_name: String,
    pub planned_value: f32,           // PV - Budget for work scheduled
    pub earned_value: f32,            // EV - Budget for work performed
    pub actual_cost: f32,             // AC - Actual cost of work performed
    pub budget_at_completion: f32,    // BAC - Total project budget
    pub schedule_variance: f32,       // SV = EV - PV
    pub cost_variance: f32,           // CV = EV - AC
    pub schedule_performance_index: f32, // SPI = EV / PV
    pub cost_performance_index: f32,  // CPI = EV / AC
    pub estimate_at_completion: f32,  // EAC = BAC / CPI
    pub estimate_to_complete: f32,    // ETC = EAC - AC
    pub variance_at_completion: f32,  // VAC = BAC - EAC
    pub percent_complete: f32,        // PC = EV / BAC
    pub percent_spent: f32,           // PS = AC / BAC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    pub report_id: Uuid,
    pub project_name: String,
    pub reporting_period_start: NaiveDate,
    pub reporting_period_end: NaiveDate,
    pub generated_date: NaiveDateTime,
    pub generated_by: String,
    pub executive_summary: String,
    pub overall_health: ProjectStatus,
    pub earned_value_metrics: EarnedValueMetrics,
    pub schedule_summary: ScheduleSummary,
    pub cost_summary: CostSummary,
    pub scope_summary: ScopeSummary,
    pub risk_and_issue_summary: RiskIssueSummary,
    pub accomplishments: Vec<String>,
    pub upcoming_milestones: Vec<UpcomingMilestone>,
    pub action_items: Vec<ActionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSummary {
    pub tasks_completed_on_time: u32,
    pub tasks_completed_late: u32,
    pub tasks_in_progress: u32,
    pub tasks_not_started: u32,
    pub critical_path_status: String,
    pub schedule_variance_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub budget_consumed_percentage: f32,
    pub cost_variance_amount: f32,
    pub cost_variance_percentage: f32,
    pub forecasted_final_cost: f32,
    pub cost_performance_trend: CostTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeSummary {
    pub planned_deliverables: u32,
    pub completed_deliverables: u32,
    pub approved_scope_changes: u32,
    pub pending_scope_changes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIssueSummary {
    pub new_risks_identified: u32,
    pub risks_mitigated: u32,
    pub active_high_priority_risks: u32,
    pub new_issues_raised: u32,
    pub issues_resolved: u32,
    pub open_critical_issues: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingMilestone {
    pub milestone_name: String,
    pub target_date: NaiveDate,
    pub status: MilestoneStatus,
    pub completion_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub assigned_to: String,
    pub due_date: NaiveDate,
    pub priority: ActionPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostTrend {
    Improving,
    Stable,
    Declining,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl ProgressSnapshot {
    pub fn new(
        project_name: String,
        status_date: NaiveDate,
        recorded_by: String,
    ) -> Self {
        Self {
            snapshot_id: Uuid::new_v4(),
            project_name,
            status_date,
            recorded_by,
            recorded_at: chrono::Utc::now().naive_utc(),
            overall_status: ProjectStatus::Green,
            task_progress: IndexMap::new(),
            milestone_progress: IndexMap::new(),
            resource_actuals: IndexMap::new(),
            issues_summary: IssuesSummary::default(),
            risks_summary: RisksSummary::default(),
        }
    }

    pub fn add_task_progress(&mut self, task_progress: TaskProgress) {
        self.task_progress.insert(task_progress.task_id, task_progress);
    }

    pub fn add_milestone_progress(&mut self, milestone_progress: MilestoneProgress) {
        self.milestone_progress.insert(milestone_progress.milestone_id, milestone_progress);
    }

    pub fn add_resource_actuals(&mut self, resource_actuals: ResourceActuals) {
        self.resource_actuals.insert(resource_actuals.resource_id, resource_actuals);
    }

    pub fn calculate_overall_completion(&self) -> f32 {
        if self.task_progress.is_empty() {
            return 0.0;
        }

        let total_completion: f32 = self.task_progress.values()
            .map(|task| task.percent_complete)
            .sum();
        
        total_completion / self.task_progress.len() as f32
    }

    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<&TaskProgress> {
        self.task_progress.values()
            .filter(|task| task.status == status)
            .collect()
    }

    pub fn get_overdue_tasks(&self) -> Vec<&TaskProgress> {
        self.task_progress.values()
            .filter(|task| {
                if let Some(estimated_completion) = task.estimated_completion_date {
                    estimated_completion < self.status_date && task.status != TaskStatus::Completed
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn calculate_total_actual_cost(&self) -> f32 {
        self.task_progress.values()
            .map(|task| task.actual_cost)
            .sum()
    }
}

impl TaskProgress {
    pub fn new(
        task_id: Uuid,
        name: String,
        updated_by: String,
    ) -> Self {
        Self {
            task_id,
            name,
            status: TaskStatus::NotStarted,
            percent_complete: 0.0,
            actual_start: None,
            actual_end: None,
            actual_effort_hours: 0.0,
            actual_cost: 0.0,
            estimated_completion_date: None,
            remaining_effort_hours: 0.0,
            notes: None,
            last_updated: chrono::Utc::now().naive_utc(),
            updated_by,
        }
    }

    pub fn start_task(&mut self, start_date: NaiveDate) {
        self.actual_start = Some(start_date);
        self.status = TaskStatus::InProgress;
        self.last_updated = chrono::Utc::now().naive_utc();
    }

    pub fn complete_task(&mut self, end_date: NaiveDate) {
        self.actual_end = Some(end_date);
        self.status = TaskStatus::Completed;
        self.percent_complete = 100.0;
        self.remaining_effort_hours = 0.0;
        self.last_updated = chrono::Utc::now().naive_utc();
    }

    pub fn update_progress(&mut self, percent_complete: f32, remaining_hours: f32, notes: Option<String>) {
        self.percent_complete = percent_complete.clamp(0.0, 100.0);
        self.remaining_effort_hours = remaining_hours;
        self.notes = notes;
        self.last_updated = chrono::Utc::now().naive_utc();

        if percent_complete >= 100.0 {
            self.status = TaskStatus::Completed;
            self.remaining_effort_hours = 0.0;
        } else if percent_complete > 0.0 && self.status == TaskStatus::NotStarted {
            self.status = TaskStatus::InProgress;
        }
    }

    pub fn is_overdue(&self, current_date: NaiveDate) -> bool {
        if let Some(estimated_completion) = self.estimated_completion_date {
            estimated_completion < current_date && self.status != TaskStatus::Completed
        } else {
            false
        }
    }
}

impl EarnedValueMetrics {
    pub fn calculate(
        status_date: NaiveDate,
        project_name: String,
        baseline: &crate::core::ProjectBaseline,
        progress: &ProgressSnapshot,
    ) -> Self {
        let budget_at_completion = baseline.project_snapshot.total_cost;
        
        // Calculate Planned Value (PV) - Budget for work scheduled to be completed by status date
        let planned_value = Self::calculate_planned_value(status_date, baseline);
        
        // Calculate Earned Value (EV) - Budget for work actually completed
        let earned_value = Self::calculate_earned_value(baseline, progress);
        
        // Calculate Actual Cost (AC) - Actual cost incurred for work performed
        let actual_cost = progress.calculate_total_actual_cost();
        
        // Calculate derived metrics
        let schedule_variance = earned_value - planned_value;
        let cost_variance = earned_value - actual_cost;
        let schedule_performance_index = if planned_value > 0.0 { earned_value / planned_value } else { 1.0 };
        let cost_performance_index = if actual_cost > 0.0 { earned_value / actual_cost } else { 1.0 };
        let estimate_at_completion = if cost_performance_index > 0.0 { budget_at_completion / cost_performance_index } else { budget_at_completion };
        let estimate_to_complete = estimate_at_completion - actual_cost;
        let variance_at_completion = budget_at_completion - estimate_at_completion;
        let percent_complete = if budget_at_completion > 0.0 { earned_value / budget_at_completion * 100.0 } else { 0.0 };
        let percent_spent = if budget_at_completion > 0.0 { actual_cost / budget_at_completion * 100.0 } else { 0.0 };

        Self {
            status_date,
            project_name,
            planned_value,
            earned_value,
            actual_cost,
            budget_at_completion,
            schedule_variance,
            cost_variance,
            schedule_performance_index,
            cost_performance_index,
            estimate_at_completion,
            estimate_to_complete,
            variance_at_completion,
            percent_complete,
            percent_spent,
        }
    }

    fn calculate_planned_value(status_date: NaiveDate, baseline: &crate::core::ProjectBaseline) -> f32 {
        baseline.project_snapshot.tasks.values()
            .filter(|task| task.start_date <= status_date)
            .map(|task| {
                if task.end_date <= status_date {
                    // Task should be fully completed
                    task.cost
                } else {
                    // Task should be partially completed
                    let total_duration = (task.end_date - task.start_date).num_days() as f32 + 1.0;
                    let elapsed_duration = (status_date - task.start_date).num_days() as f32 + 1.0;
                    let completion_ratio = (elapsed_duration / total_duration).clamp(0.0, 1.0);
                    task.cost * completion_ratio
                }
            })
            .sum()
    }

    fn calculate_earned_value(baseline: &crate::core::ProjectBaseline, progress: &ProgressSnapshot) -> f32 {
        baseline.project_snapshot.tasks.values()
            .map(|task| {
                if let Some(task_progress) = progress.task_progress.get(&task.task_id) {
                    task.cost * (task_progress.percent_complete / 100.0)
                } else {
                    0.0
                }
            })
            .sum()
    }

    pub fn schedule_health(&self) -> ProjectStatus {
        if self.schedule_performance_index >= 0.95 {
            ProjectStatus::Green
        } else if self.schedule_performance_index >= 0.85 {
            ProjectStatus::Yellow
        } else {
            ProjectStatus::Red
        }
    }

    pub fn cost_health(&self) -> ProjectStatus {
        if self.cost_performance_index >= 0.95 {
            ProjectStatus::Green
        } else if self.cost_performance_index >= 0.85 {
            ProjectStatus::Yellow
        } else {
            ProjectStatus::Red
        }
    }
}

impl Default for IssuesSummary {
    fn default() -> Self {
        Self {
            total_open: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
        }
    }
}

impl Default for RisksSummary {
    fn default() -> Self {
        Self {
            total_active: 0,
            high_probability_high_impact: 0,
            risks_requiring_action: 0,
            risks_being_monitored: 0,
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::NotStarted => write!(f, "Not Started"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::OnHold => write!(f, "On Hold"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectStatus::Green => write!(f, "Green"),
            ProjectStatus::Yellow => write!(f, "Yellow"),
            ProjectStatus::Red => write!(f, "Red"),
            ProjectStatus::Completed => write!(f, "Completed"),
            ProjectStatus::OnHold => write!(f, "On Hold"),
            ProjectStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_progress_snapshot_creation() {
        let snapshot = ProgressSnapshot::new(
            "Test Project".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "project_manager".to_string(),
        );

        assert!(!snapshot.snapshot_id.to_string().is_empty());
        assert_eq!(snapshot.project_name, "Test Project");
        assert_eq!(snapshot.overall_status, ProjectStatus::Green);
    }

    #[test]
    fn test_task_progress_update() {
        let mut task_progress = TaskProgress::new(
            Uuid::new_v4(),
            "Test Task".to_string(),
            "developer".to_string(),
        );

        assert_eq!(task_progress.status, TaskStatus::NotStarted);
        assert_eq!(task_progress.percent_complete, 0.0);

        task_progress.update_progress(50.0, 20.0, Some("Half done".to_string()));
        
        assert_eq!(task_progress.status, TaskStatus::InProgress);
        assert_eq!(task_progress.percent_complete, 50.0);
        assert_eq!(task_progress.remaining_effort_hours, 20.0);

        task_progress.update_progress(100.0, 0.0, Some("Completed".to_string()));
        
        assert_eq!(task_progress.status, TaskStatus::Completed);
        assert_eq!(task_progress.percent_complete, 100.0);
        assert_eq!(task_progress.remaining_effort_hours, 0.0);
    }

    #[test]
    fn test_overall_completion_calculation() {
        let mut snapshot = ProgressSnapshot::new(
            "Test Project".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "project_manager".to_string(),
        );

        let mut task1 = TaskProgress::new(Uuid::new_v4(), "Task 1".to_string(), "dev".to_string());
        task1.percent_complete = 100.0;
        
        let mut task2 = TaskProgress::new(Uuid::new_v4(), "Task 2".to_string(), "dev".to_string());
        task2.percent_complete = 50.0;

        snapshot.add_task_progress(task1);
        snapshot.add_task_progress(task2);

        assert_eq!(snapshot.calculate_overall_completion(), 75.0);
    }
}