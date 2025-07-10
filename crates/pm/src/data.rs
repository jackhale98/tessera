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
    pub estimated_hours: f64,
    pub actual_hours: f64,
    pub start_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub completion_date: Option<DateTime<Utc>>,
    pub dependencies: Vec<Id>, // Other task IDs
    pub assigned_resources: Vec<Id>, // Resource IDs
    pub progress_percentage: f64, // 0.0 to 100.0
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
            estimated_hours: 1.0,
            actual_hours: 0.0,
            start_date: None,
            due_date: None,
            completion_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
            progress_percentage: 0.0,
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }
    
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }
    
    pub fn is_ready_to_start(&self, completed_tasks: &[Id]) -> bool {
        self.dependencies.iter().all(|dep_id| completed_tasks.contains(dep_id))
    }
    
    pub fn duration_days(&self) -> Option<i64> {
        if let (Some(start), Some(end)) = (self.start_date, self.due_date) {
            Some((end - start).num_days())
        } else {
            None
        }
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
    pub dependent_tasks: Vec<Id>, // Tasks that must be completed for this milestone
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
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
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
    pub critical_path: Vec<Id>, // Task IDs on critical path
    pub total_duration_days: i64,
    pub task_schedule: IndexMap<Id, TaskScheduleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScheduleInfo {
    pub task_id: Id,
    pub earliest_start: DateTime<Utc>,
    pub latest_start: DateTime<Utc>,
    pub earliest_finish: DateTime<Utc>,
    pub latest_finish: DateTime<Utc>,
    pub slack_days: i64,
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
            is_critical: false,
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