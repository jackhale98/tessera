use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Duration, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub work_type: WorkType,
    pub task_type: TaskType,
    pub estimated_hours: f64,
    pub actual_hours: f64,
    pub duration_days: Option<f64>, // Duration in working days
    pub work_units: Option<f64>, // Total work units (for fixed work tasks)
    pub start_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub completion_date: Option<DateTime<Utc>>,
    pub dependencies: Vec<TaskDependency>, // Enhanced dependencies with relationship types
    pub assigned_resources: Vec<ResourceAssignment>, // Enhanced resource assignments
    pub progress_percentage: f64, // 0.0 to 100.0
    pub notes: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkType {
    Design,
    Analysis,
    Testing,
    Documentation,
    Review,
    Manufacturing,
    Other(String),
}

/// Defines how a task's duration, effort, and resources interact
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    /// Effort is fixed, duration changes with resource allocation
    EffortDriven,
    /// Duration is fixed, effort changes with resource allocation  
    FixedDuration,
    /// Work units are fixed, both effort and duration can change
    FixedWork,
    /// Zero-duration milestone
    Milestone,
}

/// Enhanced dependency with relationship types and lag/lead time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskDependency {
    pub predecessor_id: Id,
    pub dependency_type: DependencyType,
    pub lag_days: f32, // Positive for lag, negative for lead
    pub description: Option<String>,
}

/// Types of dependency relationships between tasks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    /// Predecessor must finish before successor starts (most common)
    FinishToStart,
    /// Predecessor must start before successor starts
    StartToStart,
    /// Predecessor must finish before successor finishes
    FinishToFinish,
    /// Predecessor must start before successor finishes
    StartToFinish,
}

/// Enhanced resource assignment with allocation details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceAssignment {
    pub resource_id: Id,
    pub allocation_percentage: f64, // 0.0 to 100.0 - percentage of resource's time
    pub assigned_hours: Option<f64>, // Specific hours assigned (if different from calculated)
    pub rate_override: Option<f64>, // Override hourly rate for this assignment
    pub role_in_task: Option<String>, // Role of resource in this specific task
}

impl Entity for Task {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Task name cannot be empty".to_string()
            ));
        }
        if self.estimated_hours < 0.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Estimated hours cannot be negative".to_string()
            ));
        }
        if self.progress_percentage < 0.0 || self.progress_percentage > 100.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Progress percentage must be between 0 and 100".to_string()
            ));
        }
        Ok(())
    }
}

impl Task {
    pub fn new(name: String, description: String, work_type: WorkType) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            status: TaskStatus::NotStarted,
            priority: TaskPriority::Medium,
            work_type,
            task_type: TaskType::EffortDriven,
            estimated_hours: 8.0, // Default to 1 day
            actual_hours: 0.0,
            duration_days: Some(1.0), // Default to 1 day duration
            work_units: None,
            start_date: None,
            due_date: None,
            completion_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
            progress_percentage: 0.0,
            notes: None,
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }

    /// Create a new task with specific type
    pub fn with_type(name: String, description: String, work_type: WorkType, task_type: TaskType) -> Self {
        let mut task = Self::new(name, description, work_type);
        task.task_type = task_type;
        
        // Set defaults based on task type
        match task_type {
            TaskType::EffortDriven => {
                task.estimated_hours = 8.0;
                task.duration_days = None; // Will be calculated
            }
            TaskType::FixedDuration => {
                task.duration_days = Some(1.0);
                task.estimated_hours = 8.0; // Will be adjusted based on resources
            }
            TaskType::FixedWork => {
                task.work_units = Some(8.0);
                task.estimated_hours = 8.0; // Will be calculated
                task.duration_days = None; // Will be calculated
            }
            TaskType::Milestone => {
                task.estimated_hours = 0.0;
                task.duration_days = Some(0.0);
            }
        }
        
        task.updated = Utc::now();
        task
    }
    
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }
    
    pub fn is_milestone(&self) -> bool {
        matches!(self.task_type, TaskType::Milestone)
    }
    
    pub fn is_ready_to_start(&self, completed_tasks: &[Id]) -> bool {
        self.dependencies.iter().all(|dep| completed_tasks.contains(&dep.predecessor_id))
    }
    
    pub fn duration_days(&self) -> Option<i64> {
        if let (Some(start), Some(end)) = (self.start_date, self.due_date) {
            Some((end - start).num_days())
        } else {
            self.duration_days.map(|d| d.ceil() as i64)
        }
    }

    /// Add a dependency to this task
    pub fn add_dependency(&mut self, predecessor_id: Id, dependency_type: DependencyType, lag_days: f32) {
        let dependency = TaskDependency {
            predecessor_id,
            dependency_type,
            lag_days,
            description: None,
        };
        
        // Remove existing dependency with same predecessor if exists
        self.dependencies.retain(|dep| dep.predecessor_id != predecessor_id);
        self.dependencies.push(dependency);
        self.updated = Utc::now();
    }

    /// Add a resource assignment to this task
    pub fn assign_resource(&mut self, resource_id: Id, allocation_percentage: f64) -> Result<()> {
        if allocation_percentage < 0.0 || allocation_percentage > 100.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Allocation percentage must be between 0 and 100".to_string()
            ));
        }

        let assignment = ResourceAssignment {
            resource_id,
            allocation_percentage,
            assigned_hours: None,
            rate_override: None,
            role_in_task: None,
        };

        // Remove existing assignment for same resource if exists
        self.assigned_resources.retain(|res| res.resource_id != resource_id);
        self.assigned_resources.push(assignment);
        self.updated = Utc::now();
        
        Ok(())
    }

    /// Remove a resource assignment
    pub fn unassign_resource(&mut self, resource_id: Id) {
        self.assigned_resources.retain(|res| res.resource_id != resource_id);
        self.updated = Utc::now();
    }

    /// Calculate effective effort based on task type and assignments
    pub fn calculate_effective_effort(&self) -> f64 {
        match self.task_type {
            TaskType::EffortDriven => self.estimated_hours,
            TaskType::FixedDuration => {
                if let Some(duration) = self.duration_days {
                    let total_allocation: f64 = self.assigned_resources.iter()
                        .map(|res| res.allocation_percentage / 100.0)
                        .sum();
                    duration * 8.0 * total_allocation // 8 hours per day
                } else {
                    self.estimated_hours
                }
            }
            TaskType::FixedWork => self.work_units.unwrap_or(self.estimated_hours),
            TaskType::Milestone => 0.0,
        }
    }

    /// Calculate effective duration based on task type and assignments
    pub fn calculate_effective_duration(&self) -> Option<f64> {
        match self.task_type {
            TaskType::EffortDriven => {
                if !self.assigned_resources.is_empty() {
                    let total_allocation: f64 = self.assigned_resources.iter()
                        .map(|res| res.allocation_percentage / 100.0)
                        .sum();
                    if total_allocation > 0.0 {
                        Some(self.estimated_hours / (8.0 * total_allocation))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            TaskType::FixedDuration => self.duration_days,
            TaskType::FixedWork => {
                if let Some(work_units) = self.work_units {
                    if !self.assigned_resources.is_empty() {
                        let total_allocation: f64 = self.assigned_resources.iter()
                            .map(|res| res.allocation_percentage / 100.0)
                            .sum();
                        if total_allocation > 0.0 {
                            Some(work_units / (8.0 * total_allocation))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            TaskType::Milestone => Some(0.0),
        }
    }

    /// Get list of predecessor task IDs
    pub fn get_predecessor_ids(&self) -> Vec<Id> {
        self.dependencies.iter().map(|dep| dep.predecessor_id).collect()
    }

    /// Update progress and automatically update status if needed
    pub fn update_progress(&mut self, progress: f64) -> Result<()> {
        if progress < 0.0 || progress > 100.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Progress must be between 0 and 100".to_string()
            ));
        }

        self.progress_percentage = progress;
        
        // Auto-update status based on progress
        match self.status {
            TaskStatus::NotStarted if progress > 0.0 => {
                self.status = TaskStatus::InProgress;
                if self.start_date.is_none() {
                    self.start_date = Some(Utc::now());
                }
            }
            TaskStatus::InProgress if progress >= 100.0 => {
                self.status = TaskStatus::Completed;
                self.completion_date = Some(Utc::now());
            }
            _ => {}
        }
        
        self.updated = Utc::now();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: Id,
    pub name: String,
    pub email: Option<String>,
    pub role: String,
    pub hourly_rate: Option<f64>,
    pub daily_hours: f64, // Standard daily working hours
    pub availability_percentage: f64, // 0.0 to 100.0
    pub skills: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

impl Entity for Resource {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Resource name cannot be empty".to_string()
            ));
        }
        if self.availability_percentage < 0.0 || self.availability_percentage > 100.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Availability percentage must be between 0 and 100".to_string()
            ));
        }
        Ok(())
    }
}

impl Resource {
    pub fn new(name: String, role: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            email: None,
            role,
            hourly_rate: None,
            daily_hours: 8.0, // Standard 8-hour workday
            availability_percentage: 100.0,
            skills: Vec::new(),
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub target_date: DateTime<Utc>,
    pub actual_date: Option<DateTime<Utc>>,
    pub status: MilestoneStatus,
    pub dependent_tasks: Vec<Id>, // Tasks that must be completed for this milestone (legacy)
    pub dependencies: Vec<TaskDependency>, // Dependencies on other tasks or milestones
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Pending,
    AtRisk,
    Achieved,
    Missed,
}

impl Entity for Milestone {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Milestone name cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

impl Milestone {
    pub fn new(name: String, description: String, target_date: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            target_date,
            actual_date: None,
            status: MilestoneStatus::Pending,
            dependent_tasks: Vec::new(),
            dependencies: Vec::new(),
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }

    /// Add a dependency to this milestone
    pub fn add_dependency(&mut self, predecessor_id: Id, dependency_type: DependencyType, lag_days: f32) {
        let dependency = TaskDependency {
            predecessor_id,
            dependency_type,
            lag_days,
            description: None,
        };
        
        // Remove existing dependency with same predecessor if exists
        self.dependencies.retain(|dep| dep.predecessor_id != predecessor_id);
        self.dependencies.push(dependency);
        self.updated = Utc::now();
    }
    
    pub fn is_overdue(&self) -> bool {
        !matches!(self.status, MilestoneStatus::Achieved) && Utc::now() > self.target_date
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSchedule {
    pub generated: DateTime<Utc>,
    pub project_start: DateTime<Utc>,
    pub project_end: DateTime<Utc>,
    pub critical_path: Vec<Id>, // Task and Milestone IDs on critical path
    pub total_duration_days: i64,
    pub task_schedule: IndexMap<Id, TaskScheduleInfo>,
    pub milestone_schedule: IndexMap<Id, MilestoneScheduleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScheduleInfo {
    pub task_id: Id,
    pub earliest_start: DateTime<Utc>,
    pub latest_start: DateTime<Utc>,
    pub earliest_finish: DateTime<Utc>,
    pub latest_finish: DateTime<Utc>,
    pub slack_days: i64,        // Total float (latest start - earliest start)
    pub free_float_days: i64,   // Free float (time can delay without affecting successors)
    pub is_critical: bool,
}

impl TaskScheduleInfo {
    pub fn new(task_id: Id, earliest_start: DateTime<Utc>, duration: Duration) -> Self {
        let earliest_finish = earliest_start + duration;
        Self {
            task_id,
            earliest_start,
            latest_start: earliest_start, // Will be updated by scheduling algorithm
            earliest_finish,
            latest_finish: earliest_finish, // Will be updated by scheduling algorithm
            slack_days: 0,
            free_float_days: 0,
            is_critical: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneScheduleInfo {
    pub milestone_id: Id,
    pub earliest_date: DateTime<Utc>,
    pub latest_date: DateTime<Utc>,
    pub target_date: DateTime<Utc>,
    pub slack_days: i64,
    pub is_critical: bool,
}

impl MilestoneScheduleInfo {
    pub fn new(milestone_id: Id, target_date: DateTime<Utc>) -> Self {
        Self {
            milestone_id,
            earliest_date: target_date,
            latest_date: target_date,
            target_date,
            slack_days: 0,
            is_critical: true,
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
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

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::EffortDriven => write!(f, "Effort Driven"),
            TaskType::FixedDuration => write!(f, "Fixed Duration"),
            TaskType::FixedWork => write!(f, "Fixed Work"),
            TaskType::Milestone => write!(f, "Milestone"),
        }
    }
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::FinishToStart => write!(f, "Finish-to-Start"),
            DependencyType::StartToStart => write!(f, "Start-to-Start"),
            DependencyType::FinishToFinish => write!(f, "Finish-to-Finish"),
            DependencyType::StartToFinish => write!(f, "Start-to-Finish"),
        }
    }
}

impl Default for TaskType {
    fn default() -> Self {
        TaskType::EffortDriven
    }
}

impl Default for DependencyType {
    fn default() -> Self {
        DependencyType::FinishToStart
    }
}