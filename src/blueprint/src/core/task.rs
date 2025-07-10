use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<TaskDependency>,
    #[serde(default = "default_priority")]
    pub priority: TaskPriority,
    #[serde(default)]
    pub skills_required: Vec<String>,
    #[serde(default)]
    pub resource_assignments: HashMap<String, ResourceAssignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub completion: f32, // 0.0 to 1.0
    #[serde(default)]
    pub task_type: TaskType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_units: Option<f32>, // For fixed work tasks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAssignment {
    pub resource_id: String,
    pub allocation_type: AllocationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationType {
    #[serde(rename = "hours")]
    Hours(f32),        // Total hours for this resource
    #[serde(rename = "percentage")] 
    Percentage(f32),   // Percentage of resource's capacity (0.0-100.0)
    #[serde(rename = "fulltime")]
    FullTime,          // 100% allocation
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Critical => write!(f, "Critical"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::Low => write!(f, "Low"),
        }
    }
}

fn default_priority() -> TaskPriority {
    TaskPriority::Medium
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskDependency {
    pub task_id: String,
    #[serde(default)]
    pub dependency_type: DependencyType,
    #[serde(default)]
    pub lag_days: f32, // Positive for lag, negative for lead
}

impl std::hash::Hash for TaskDependency {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.task_id.hash(state);
        self.dependency_type.hash(state);
        // Convert f32 to bits for hashing to handle NaN consistently
        self.lag_days.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    FinishToStart,  // Predecessor must finish before successor starts
    StartToStart,   // Predecessor must start before successor starts
    FinishToFinish, // Predecessor must finish before successor finishes
    StartToFinish,  // Predecessor must start before successor finishes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    EffortDriven,  // Effort is fixed, duration changes with resource allocation
    FixedDuration, // Duration is fixed, effort changes with resource allocation
    FixedWork,     // Work units are fixed, both effort and duration can change
}

impl Default for DependencyType {
    fn default() -> Self {
        DependencyType::FinishToStart
    }
}

impl Default for TaskType {
    fn default() -> Self {
        TaskType::EffortDriven
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::EffortDriven => write!(f, "Effort Driven"),
            TaskType::FixedDuration => write!(f, "Fixed Duration"),
            TaskType::FixedWork => write!(f, "Fixed Work"),
        }
    }
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::FinishToStart => write!(f, "Finish to Start"),
            DependencyType::StartToStart => write!(f, "Start to Start"),
            DependencyType::FinishToFinish => write!(f, "Finish to Finish"),
            DependencyType::StartToFinish => write!(f, "Start to Finish"),
        }
    }
}

impl Task {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            dependencies: Vec::new(),
            priority: TaskPriority::Medium,
            skills_required: Vec::new(),
            resource_assignments: HashMap::new(),
            duration_days: None,
            start_date: None,
            end_date: None,
            completion: 0.0,
            task_type: TaskType::EffortDriven,
            work_units: None,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<TaskDependency>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_simple_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps.into_iter()
            .map(|task_id| TaskDependency::new(task_id))
            .collect();
        self
    }

    pub fn with_task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = task_type;
        self
    }

    pub fn with_work_units(mut self, work_units: f32) -> Self {
        self.work_units = Some(work_units);
        self
    }

    pub fn with_skills(mut self, skills: Vec<String>) -> Self {
        self.skills_required = skills;
        self
    }

    pub fn with_duration(mut self, days: i32) -> Self {
        self.duration_days = Some(days);
        self
    }

    pub fn add_resource_assignment(&mut self, resource_id: String, allocation: AllocationType) {
        let assignment = ResourceAssignment {
            resource_id: resource_id.clone(),
            allocation_type: allocation,
            start_date: None,
            end_date: None,
        };
        self.resource_assignments.insert(resource_id, assignment);
    }

    pub fn add_resource_hours(&mut self, resource_id: String, hours: f32) {
        self.add_resource_assignment(resource_id, AllocationType::Hours(hours));
    }

    pub fn add_resource_percentage(&mut self, resource_id: String, percentage: f32) {
        self.add_resource_assignment(resource_id, AllocationType::Percentage(percentage));
    }

    pub fn add_resource_full_time(&mut self, resource_id: String) {
        self.add_resource_assignment(resource_id, AllocationType::FullTime);
    }

    pub fn is_complete(&self) -> bool {
        self.completion >= 1.0
    }

    pub fn total_effort_hours(&self) -> f32 {
        // Calculate total effort across all resource assignments
        self.resource_assignments.values()
            .map(|assignment| match &assignment.allocation_type {
                AllocationType::Hours(hours) => *hours,
                // For percentage and full-time, we need more context to calculate actual hours
                // Return a default estimate based on duration
                AllocationType::Percentage(percentage) => {
                    let daily_hours = 8.0; // Standard work day
                    let days = self.duration_days.unwrap_or(1) as f32;
                    (percentage / 100.0) * daily_hours * days
                }
                AllocationType::FullTime => {
                    let daily_hours = 8.0; // Standard work day
                    let days = self.duration_days.unwrap_or(1) as f32;
                    daily_hours * days
                }
            })
            .sum()
    }

    // Calculate effort for a specific resource assignment
    pub fn calculate_effort_hours_for_resource(&self, resource_id: &str, resource: &crate::core::Resource) -> f32 {
        // Calculate from specific resource assignment
        if let Some(assignment) = self.resource_assignments.get(resource_id) {
            match &assignment.allocation_type {
                AllocationType::Hours(hours) => *hours,
                AllocationType::Percentage(percentage) => {
                    let days = self.duration_days.unwrap_or(1) as f32;
                    (percentage / 100.0) * resource.daily_hours() * days
                }
                AllocationType::FullTime => {
                    let days = self.duration_days.unwrap_or(1) as f32;
                    resource.daily_hours() * days
                }
            }
        } else {
            0.0
        }
    }

    // Calculate total effort across all resource assignments with proper resource info
    pub fn calculate_total_effort_from_assignments(&self, project: &crate::core::Project) -> f32 {
        // Calculate from all resource assignments
        self.resource_assignments.iter()
            .map(|(resource_id, assignment)| {
                if let Some(resource) = project.resources.get(resource_id) {
                    match &assignment.allocation_type {
                        AllocationType::Hours(hours) => *hours,
                        AllocationType::Percentage(percentage) => {
                            let days = self.duration_days.unwrap_or(1) as f32;
                            (percentage / 100.0) * resource.daily_hours() * days
                        }
                        AllocationType::FullTime => {
                            let days = self.duration_days.unwrap_or(1) as f32;
                            resource.daily_hours() * days
                        }
                    }
                } else {
                    0.0
                }
            })
            .sum()
    }

    pub fn remaining_effort(&self) -> f32 {
        self.total_effort_hours() * (1.0 - self.completion)
    }

    pub fn has_resource_assignments(&self) -> bool {
        !self.resource_assignments.is_empty()
    }

    pub fn get_assigned_resources(&self) -> Vec<String> {
        self.resource_assignments.keys().cloned().collect()
    }

    pub fn add_dependency(&mut self, dependency: TaskDependency) {
        self.dependencies.push(dependency);
    }

    pub fn add_simple_dependency(&mut self, task_id: String) {
        self.dependencies.push(TaskDependency::new(task_id));
    }

    pub fn get_dependency_ids(&self) -> Vec<String> {
        self.dependencies.iter().map(|dep| dep.task_id.clone()).collect()
    }

    pub fn calculate_duration_from_effort(&self, project: &crate::core::Project) -> Option<f32> {
        match self.task_type {
            TaskType::EffortDriven => {
                // Duration = Effort / Resource Allocation
                let total_effort = self.calculate_total_effort_from_assignments(project);
                let total_allocation = self.calculate_total_resource_allocation(project);
                if total_allocation > 0.0 {
                    Some(total_effort / total_allocation)
                } else {
                    None
                }
            }
            TaskType::FixedDuration => {
                // Duration is fixed
                self.duration_days.map(|d| d as f32)
            }
            TaskType::FixedWork => {
                // Duration depends on work units and resource allocation
                if let Some(work_units) = self.work_units {
                    let total_allocation = self.calculate_total_resource_allocation(project);
                    if total_allocation > 0.0 {
                        Some(work_units / total_allocation)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn calculate_effort_from_duration(&self, project: &crate::core::Project) -> Option<f32> {
        match self.task_type {
            TaskType::EffortDriven => {
                // Effort is fixed
                Some(self.calculate_total_effort_from_assignments(project))
            }
            TaskType::FixedDuration => {
                // Effort = Duration * Resource Allocation
                if let Some(duration) = self.duration_days {
                    let total_allocation = self.calculate_total_resource_allocation(project);
                    Some(duration as f32 * total_allocation)
                } else {
                    None
                }
            }
            TaskType::FixedWork => {
                // Effort depends on work units
                self.work_units
            }
        }
    }

    fn calculate_total_resource_allocation(&self, project: &crate::core::Project) -> f32 {
        self.resource_assignments.iter()
            .map(|(resource_id, assignment)| {
                if let Some(resource) = project.resources.get(resource_id) {
                    match &assignment.allocation_type {
                        crate::core::AllocationType::Hours(hours) => {
                            // Convert hours to daily allocation
                            hours / resource.daily_hours()
                        }
                        crate::core::AllocationType::Percentage(percentage) => {
                            percentage / 100.0
                        }
                        crate::core::AllocationType::FullTime => 1.0,
                    }
                } else {
                    0.0
                }
            })
            .sum()
    }
}

impl TaskDependency {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            dependency_type: DependencyType::FinishToStart,
            lag_days: 0.0,
        }
    }

    pub fn with_type(mut self, dependency_type: DependencyType) -> Self {
        self.dependency_type = dependency_type;
        self
    }

    pub fn with_lag(mut self, lag_days: f32) -> Self {
        self.lag_days = lag_days;
        self
    }

    pub fn with_lead(mut self, lead_days: f32) -> Self {
        self.lag_days = -lead_days;
        self
    }
}

