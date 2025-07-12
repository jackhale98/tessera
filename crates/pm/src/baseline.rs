use crate::{Task, Resource, Milestone};
use tessera_core::{Id, Entity, Repository, Result, DesignTrackError, format_ron_pretty};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBaseline {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub created_date: DateTime<Utc>,
    pub created_by: String,
    pub is_current: bool,
    pub baseline_type: BaselineType,
    pub project_snapshot: BaselineProjectSnapshot,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    pub tasks: IndexMap<Id, BaselineTask>,
    pub milestones: IndexMap<Id, BaselineMilestone>,
    pub resources: IndexMap<Id, BaselineResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineTask {
    pub id: Id,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub duration_days: i32,
    pub effort_hours: f32,
    pub cost: f32,
    pub assigned_resources: Vec<String>,
    pub dependencies: Vec<Id>,
    pub work_breakdown_structure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMilestone {
    pub id: Id,
    pub name: String,
    pub target_date: NaiveDate,
    pub dependencies: Vec<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResource {
    pub id: Id,
    pub name: String,
    pub hourly_rate: f32,
    pub total_allocated_hours: f32,
    pub total_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub current_baseline: Id,
    pub comparison_baseline: Id,
    pub generated_date: DateTime<Utc>,
    pub schedule_variance_days: i32,
    pub cost_variance: f32,
    pub effort_variance_hours: f32,
    pub task_changes: Vec<TaskVariance>,
    pub milestone_changes: Vec<MilestoneVariance>,
    pub summary: BaselineVarianceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskVariance {
    pub task_id: Id,
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
    pub milestone_id: Id,
    pub milestone_name: String,
    pub baseline_date: NaiveDate,
    pub current_date: Option<NaiveDate>,
    pub variance_days: i32,
    pub status: MilestoneStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarianceType {
    NoChange,
    ScheduleVariance,
    CostVariance,
    ScopeChange,
    TaskAdded,
    TaskRemoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
pub enum ProjectHealth {
    Green,   // On track
    Yellow,  // At risk but manageable
    Red,     // Critical issues
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineManager {
    pub baselines: HashMap<Id, ProjectBaseline>,
    pub current_baseline_id: Option<Id>,
}

impl Entity for ProjectBaseline {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(DesignTrackError::Validation("Baseline name cannot be empty".to_string()));
        }
        if self.created_by.trim().is_empty() {
            return Err(DesignTrackError::Validation("Baseline must have a created_by field".to_string()));
        }
        Ok(())
    }
}

impl ProjectBaseline {
    pub fn new(
        name: String,
        created_by: String,
        baseline_type: BaselineType,
        project_name: String,
        tasks: &[Task],
        milestones: &[Milestone],
        resources: &[Resource],
    ) -> Self {
        let now = Utc::now();
        let project_snapshot = BaselineProjectSnapshot::from_current_state(
            project_name,
            tasks,
            milestones,
            resources,
        );
        
        Self {
            id: Id::new(),
            name,
            description: None,
            created_date: now,
            created_by,
            is_current: baseline_type == BaselineType::Working,
            baseline_type,
            project_snapshot,
            created: now,
            updated: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated = Utc::now();
        self
    }

    pub fn set_as_current(&mut self) {
        self.is_current = true;
        self.updated = Utc::now();
    }

    pub fn archive(&mut self) {
        self.is_current = false;
        self.baseline_type = BaselineType::Archived;
        self.updated = Utc::now();
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
                    task_id: *task_id,
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
                    task_id: *task_id,
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
                        milestone_id: *milestone_id,
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
            current_baseline: self.id,
            comparison_baseline: other.id,
            generated_date: Utc::now(),
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
        let start_variance_days = (current.start_date - baseline.start_date).num_days() as i32;
        let duration_variance_days = current.duration_days - baseline.duration_days;
        let cost_variance = current.cost - baseline.cost;
        let effort_variance_hours = current.effort_hours - baseline.effort_hours;

        let variance_type = if schedule_variance_days.abs() > 0 || start_variance_days.abs() > 0 || duration_variance_days.abs() > 0 {
            VarianceType::ScheduleVariance
        } else if cost_variance.abs() > 0.01 {
            VarianceType::CostVariance
        } else if effort_variance_hours.abs() > 0.01 {
            VarianceType::ScopeChange
        } else {
            VarianceType::NoChange
        };

        TaskVariance {
            task_id: current.id,
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
    pub fn from_current_state(
        project_name: String,
        tasks: &[Task],
        milestones: &[Milestone],
        resources: &[Resource],
    ) -> Self {
        let mut baseline_tasks = IndexMap::new();
        let mut baseline_milestones = IndexMap::new();
        let mut baseline_resources = IndexMap::new();

        let mut total_cost = 0.0;
        let mut total_effort_hours = 0.0;
        let mut start_date = None;
        let mut end_date = None;

        // Convert tasks to baseline tasks
        for task in tasks {
            // Use current dates if available, otherwise estimate based on task duration
            let (start_naive, end_naive) = match (task.start_date, task.due_date) {
                (Some(start), Some(end)) => (start.date_naive(), end.date_naive()),
                (Some(start), None) => {
                    // Calculate end date from duration or estimated hours
                    let duration_days = task.duration_days().unwrap_or_else(|| {
                        (task.estimated_hours / 8.0).ceil() as i64
                    });
                    let end = start + chrono::Duration::days(duration_days);
                    (start.date_naive(), end.date_naive())
                },
                (None, Some(end)) => {
                    // Calculate start date from duration or estimated hours
                    let duration_days = task.duration_days().unwrap_or_else(|| {
                        (task.estimated_hours / 8.0).ceil() as i64
                    });
                    let start = end - chrono::Duration::days(duration_days);
                    (start.date_naive(), end.date_naive())
                },
                (None, None) => {
                    // Use current date as start and calculate end from duration
                    let start = Utc::now();
                    let duration_days = task.duration_days().unwrap_or_else(|| {
                        (task.estimated_hours / 8.0).ceil() as i64
                    });
                    let end = start + chrono::Duration::days(duration_days);
                    (start.date_naive(), end.date_naive())
                }
            };

            let baseline_task = BaselineTask {
                id: task.id,
                name: task.name.clone(),
                start_date: start_naive,
                end_date: end_naive,
                duration_days: (end_naive - start_naive).num_days() as i32 + 1,
                effort_hours: task.estimated_hours as f32,
                cost: (task.estimated_hours * 100.0) as f32, // Assume $100/hour default
                assigned_resources: task.assigned_resources.iter().map(|assignment| assignment.resource_id.to_string()).collect(),
                dependencies: task.dependencies.iter().map(|dep| dep.predecessor_id).collect(),
                work_breakdown_structure: None,
            };
            
            total_cost += baseline_task.cost;
            total_effort_hours += baseline_task.effort_hours;
            
            if start_date.is_none() || start_naive < start_date.unwrap() {
                start_date = Some(start_naive);
            }
            if end_date.is_none() || end_naive > end_date.unwrap() {
                end_date = Some(end_naive);
            }
            
            baseline_tasks.insert(task.id, baseline_task);
        }

        // Convert milestones to baseline milestones
        for milestone in milestones {
            let baseline_milestone = BaselineMilestone {
                id: milestone.id,
                name: milestone.name.clone(),
                target_date: milestone.target_date.date_naive(),
                dependencies: milestone.dependent_tasks.clone(),
            };
            baseline_milestones.insert(milestone.id, baseline_milestone);
        }

        // Convert resources to baseline resources
        for resource in resources {
            let baseline_resource = BaselineResource {
                id: resource.id,
                name: resource.name.clone(),
                hourly_rate: resource.hourly_rate.unwrap_or(100.0) as f32,
                total_allocated_hours: 0.0, // Would need to calculate from task assignments
                total_cost: 0.0,
            };
            baseline_resources.insert(resource.id, baseline_resource);
        }

        Self {
            project_name,
            start_date: start_date.unwrap_or_else(|| Utc::now().date_naive()),
            end_date: end_date.unwrap_or_else(|| Utc::now().date_naive()),
            total_cost,
            total_effort_hours,
            tasks: baseline_tasks,
            milestones: baseline_milestones,
            resources: baseline_resources,
        }
    }
}

impl BaselineManager {
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            current_baseline_id: None,
        }
    }

    pub fn create_baseline(
        &mut self,
        name: String,
        created_by: String,
        baseline_type: BaselineType,
        project_name: String,
        tasks: &[Task],
        milestones: &[Milestone],
        resources: &[Resource],
    ) -> Id {
        // If this is being set as current, unset the previous current baseline
        if baseline_type == BaselineType::Working {
            if let Some(current_id) = self.current_baseline_id {
                if let Some(current_baseline) = self.baselines.get_mut(&current_id) {
                    current_baseline.is_current = false;
                }
            }
        }

        let baseline = ProjectBaseline::new(
            name,
            created_by,
            baseline_type,
            project_name,
            tasks,
            milestones,
            resources,
        );
        
        let baseline_id = baseline.id;
        
        if baseline_type == BaselineType::Working {
            self.current_baseline_id = Some(baseline_id);
        }
        
        self.baselines.insert(baseline_id, baseline);
        baseline_id
    }

    pub fn get_baseline(&self, baseline_id: &Id) -> Option<&ProjectBaseline> {
        self.baselines.get(baseline_id)
    }

    pub fn get_current_baseline(&self) -> Option<&ProjectBaseline> {
        self.current_baseline_id.and_then(|id| self.baselines.get(&id))
    }

    pub fn get_all_baselines(&self) -> Vec<&ProjectBaseline> {
        self.baselines.values().collect()
    }

    pub fn get_baselines_by_type(&self, baseline_type: BaselineType) -> Vec<&ProjectBaseline> {
        self.baselines.values()
            .filter(|b| b.baseline_type == baseline_type)
            .collect()
    }

    pub fn set_current_baseline(&mut self, baseline_id: Id) -> Result<()> {
        if !self.baselines.contains_key(&baseline_id) {
            return Err(DesignTrackError::NotFound(format!("Baseline {} not found", baseline_id)));
        }

        // Unset previous current baseline
        if let Some(current_id) = self.current_baseline_id {
            if let Some(current_baseline) = self.baselines.get_mut(&current_id) {
                current_baseline.is_current = false;
            }
        }

        // Set new current baseline
        if let Some(baseline) = self.baselines.get_mut(&baseline_id) {
            baseline.set_as_current();
            baseline.baseline_type = BaselineType::Working;
        }

        self.current_baseline_id = Some(baseline_id);
        Ok(())
    }

    pub fn archive_baseline(&mut self, baseline_id: Id) -> Result<()> {
        if let Some(baseline) = self.baselines.get_mut(&baseline_id) {
            baseline.archive();
            
            // If this was the current baseline, clear the current baseline
            if self.current_baseline_id == Some(baseline_id) {
                self.current_baseline_id = None;
            }
            
            Ok(())
        } else {
            Err(DesignTrackError::NotFound(format!("Baseline {} not found", baseline_id)))
        }
    }

    pub fn compare_baselines(&self, baseline1_id: Id, baseline2_id: Id) -> Result<BaselineComparison> {
        let baseline1 = self.get_baseline(&baseline1_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Baseline {} not found", baseline1_id)))?;
        let baseline2 = self.get_baseline(&baseline2_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Baseline {} not found", baseline2_id)))?;
        
        Ok(baseline1.compare_to(baseline2))
    }
}

impl Default for BaselineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BaselineType {
    fn default() -> Self {
        BaselineType::Working
    }
}

pub struct BaselineRepository {
    baselines: Vec<ProjectBaseline>,
}

impl BaselineRepository {
    pub fn new() -> Self {
        Self { baselines: Vec::new() }
    }
}

impl Repository<ProjectBaseline> for BaselineRepository {
    fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<ProjectBaseline>> {
        if !path.as_ref().exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let baselines: Vec<ProjectBaseline> = ron::from_str(&content)?;
        Ok(baselines)
    }

    fn save_to_file<P: AsRef<std::path::Path>>(items: &[ProjectBaseline], path: P) -> Result<()> {
        let content = format_ron_pretty(items)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn find_by_id(&self, id: Id) -> Option<&ProjectBaseline> {
        self.baselines.iter().find(|b| b.id == id)
    }

    fn find_by_name(&self, name: &str) -> Option<&ProjectBaseline> {
        self.baselines.iter().find(|b| b.name == name)
    }

    fn add(&mut self, item: ProjectBaseline) -> Result<()> {
        item.validate()?;
        self.baselines.push(item);
        Ok(())
    }

    fn update(&mut self, item: ProjectBaseline) -> Result<()> {
        item.validate()?;
        if let Some(index) = self.baselines.iter().position(|b| b.id == item.id) {
            self.baselines[index] = item;
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Baseline not found".to_string()))
        }
    }

    fn remove(&mut self, id: Id) -> Result<()> {
        if let Some(index) = self.baselines.iter().position(|b| b.id == id) {
            self.baselines.remove(index);
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Baseline not found".to_string()))
        }
    }

    fn list(&self) -> &[ProjectBaseline] {
        &self.baselines
    }
}

// Display implementations
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

impl std::fmt::Display for VarianceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarianceType::NoChange => write!(f, "No Change"),
            VarianceType::ScheduleVariance => write!(f, "Schedule Variance"),
            VarianceType::CostVariance => write!(f, "Cost Variance"),
            VarianceType::ScopeChange => write!(f, "Scope Change"),
            VarianceType::TaskAdded => write!(f, "Task Added"),
            VarianceType::TaskRemoved => write!(f, "Task Removed"),
        }
    }
}

impl std::fmt::Display for MilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MilestoneStatus::OnTrack => write!(f, "On Track"),
            MilestoneStatus::AtRisk => write!(f, "At Risk"),
            MilestoneStatus::Delayed => write!(f, "Delayed"),
            MilestoneStatus::Completed => write!(f, "Completed"),
            MilestoneStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}