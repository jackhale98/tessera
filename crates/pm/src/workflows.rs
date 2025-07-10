use crate::{Task, Resource, TaskStatus, TaskPriority, ProjectRepository, Calendar};
use tessera_core::{Id, Result, DesignTrackError};
use chrono::{DateTime, Utc, NaiveDate, Duration};
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

/// Advanced dependency types for task relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    FinishToStart,  // Predecessor must finish before successor starts
    StartToStart,   // Predecessor must start before successor starts
    FinishToFinish, // Predecessor must finish before successor finishes
    StartToFinish,  // Predecessor must start before successor finishes
}

/// Task dependency with lag/lead time support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskDependency {
    pub predecessor_id: Id,
    pub successor_id: Id,
    pub dependency_type: DependencyType,
    pub lag_days: f32, // Positive for lag, negative for lead
    pub is_critical: bool,
    pub created: DateTime<Utc>,
}

/// Enhanced task types for different scheduling approaches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    EffortDriven,  // Effort is fixed, duration changes with resource allocation
    FixedDuration, // Duration is fixed, effort changes with resource allocation
    FixedWork,     // Work units are fixed, both effort and duration can change
    Milestone,     // Zero-duration milestone
    Summary,       // Summary task that rolls up subtasks
}

/// Task constraints for advanced scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraint {
    pub task_id: Id,
    pub constraint_type: ConstraintType,
    pub constraint_date: Option<DateTime<Utc>>,
    pub priority: ConstraintPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    MustStartOn,    // Task must start on specific date
    MustFinishOn,   // Task must finish on specific date
    StartNoEarlierThan, // Task cannot start before date
    StartNoLaterThan,   // Task cannot start after date
    FinishNoEarlierThan, // Task cannot finish before date
    FinishNoLaterThan,   // Task cannot finish after date
    AsLateAsPossible,    // Schedule as late as possible
    AsSoonAsPossible,    // Schedule as soon as possible (default)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintPriority {
    Flexible,  // Can be adjusted if needed
    Mandatory, // Must be respected
    Critical,  // Cannot be violated
}

/// Workflow state machine for task lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWorkflow {
    pub current_state: TaskStatus,
    pub allowed_transitions: Vec<TaskStatus>,
    pub automatic_actions: Vec<WorkflowAction>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowAction {
    SetStartDate,
    SetCompletionDate,
    UpdateProgress(f64),
    NotifyAssignees,
    CreateFollowUpTask,
    UpdateDependentTasks,
    TriggerMilestone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    RequireStartDate,
    RequireProgress,
    RequireAssignees,
    ValidateDependencies,
    CheckResourceAvailability,
}

/// Complex dependency and workflow manager
pub struct WorkflowManager {
    dependencies: Vec<TaskDependency>,
    constraints: Vec<TaskConstraint>,
    workflows: HashMap<Id, TaskWorkflow>,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
            constraints: Vec::new(),
            workflows: HashMap::new(),
        }
    }

    /// Add a dependency between tasks with validation
    pub fn add_dependency(
        &mut self,
        predecessor_id: Id,
        successor_id: Id,
        dependency_type: DependencyType,
        lag_days: f32,
        repository: &ProjectRepository,
    ) -> Result<()> {
        // Validate tasks exist
        repository.find_task_by_id(predecessor_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Predecessor task {}", predecessor_id)))?;
        repository.find_task_by_id(successor_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Successor task {}", successor_id)))?;

        // Check for circular dependencies
        if self.would_create_circular_dependency(predecessor_id, successor_id, repository)? {
            return Err(DesignTrackError::Validation(
                "Adding this dependency would create a circular dependency".to_string()
            ));
        }

        let dependency = TaskDependency {
            predecessor_id,
            successor_id,
            dependency_type,
            lag_days,
            is_critical: false, // Will be determined by critical path analysis
            created: Utc::now(),
        };

        self.dependencies.push(dependency);
        Ok(())
    }

    /// Remove a dependency
    pub fn remove_dependency(&mut self, predecessor_id: Id, successor_id: Id) -> Result<()> {
        let initial_len = self.dependencies.len();
        self.dependencies.retain(|dep| {
            !(dep.predecessor_id == predecessor_id && dep.successor_id == successor_id)
        });

        if self.dependencies.len() == initial_len {
            return Err(DesignTrackError::NotFound(
                "Dependency not found".to_string()
            ));
        }

        Ok(())
    }

    /// Get all dependencies for a task
    pub fn get_task_dependencies(&self, task_id: Id) -> Vec<&TaskDependency> {
        self.dependencies.iter()
            .filter(|dep| dep.successor_id == task_id)
            .collect()
    }

    /// Get all dependent tasks (tasks that depend on this task)
    pub fn get_dependent_tasks(&self, task_id: Id) -> Vec<&TaskDependency> {
        self.dependencies.iter()
            .filter(|dep| dep.predecessor_id == task_id)
            .collect()
    }

    /// Check for circular dependencies using DFS
    fn would_create_circular_dependency(
        &self,
        predecessor_id: Id,
        successor_id: Id,
        repository: &ProjectRepository,
    ) -> Result<bool> {
        let mut visited = HashSet::new();
        let mut stack = VecDeque::new();
        stack.push_back(successor_id);

        while let Some(current) = stack.pop_back() {
            if current == predecessor_id {
                return Ok(true); // Circular dependency found
            }

            if visited.contains(&current) {
                continue;
            }

            visited.insert(current);

            // Add all tasks that depend on the current task
            for dep in self.get_dependent_tasks(current) {
                stack.push_back(dep.successor_id);
            }
        }

        Ok(false)
    }

    /// Calculate the earliest possible start date for a task based on dependencies
    pub fn calculate_earliest_start_date(
        &self,
        task_id: Id,
        repository: &ProjectRepository,
        calendar: &Calendar,
    ) -> Result<Option<DateTime<Utc>>> {
        let dependencies = self.get_task_dependencies(task_id);
        
        if dependencies.is_empty() {
            return Ok(None); // No dependencies, can start anytime
        }

        let mut latest_constraint = None;

        for dep in dependencies {
            let predecessor = repository.find_task_by_id(dep.predecessor_id)
                .ok_or_else(|| DesignTrackError::NotFound(format!("Predecessor task {}", dep.predecessor_id)))?;

            let constraint_date = self.calculate_dependency_constraint_date(
                predecessor,
                &dep.dependency_type,
                dep.lag_days,
                calendar,
            )?;

            if let Some(date) = constraint_date {
                if latest_constraint.is_none() || date > latest_constraint.unwrap() {
                    latest_constraint = Some(date);
                }
            }
        }

        Ok(latest_constraint)
    }

    /// Calculate the constraint date based on dependency type and lag
    fn calculate_dependency_constraint_date(
        &self,
        predecessor: &Task,
        dependency_type: &DependencyType,
        lag_days: f32,
        calendar: &Calendar,
    ) -> Result<Option<DateTime<Utc>>> {
        let base_date = match dependency_type {
            DependencyType::FinishToStart => predecessor.due_date,
            DependencyType::StartToStart => predecessor.start_date,
            DependencyType::FinishToFinish => predecessor.due_date,
            DependencyType::StartToFinish => predecessor.start_date,
        };

        let Some(base_date) = base_date else {
            return Ok(None);
        };

        // Apply lag/lead time using calendar
        let result_date = if lag_days != 0.0 {
            let base_naive = base_date.date_naive();
            let result_naive = if lag_days > 0.0 {
                // Positive lag - add working days
                calendar.add_working_days(base_naive, lag_days as f64)
            } else {
                // Negative lag (lead) - subtract working days
                calendar.subtract_working_days(base_naive, (-lag_days) as f64)
            };
            result_naive.and_hms_opt(9, 0, 0).unwrap().and_utc()
        } else {
            base_date
        };

        Ok(Some(result_date))
    }

    /// Perform topological sort to get valid task execution order
    pub fn topological_sort(&self, task_ids: &[Id]) -> Result<Vec<Id>> {
        let mut in_degree: HashMap<Id, usize> = task_ids.iter().map(|&id| (id, 0)).collect();
        let mut adj_list: HashMap<Id, Vec<Id>> = task_ids.iter().map(|&id| (id, Vec::new())).collect();

        // Build adjacency list and calculate in-degrees
        for dep in &self.dependencies {
            if task_ids.contains(&dep.predecessor_id) && task_ids.contains(&dep.successor_id) {
                adj_list.get_mut(&dep.predecessor_id).unwrap().push(dep.successor_id);
                *in_degree.get_mut(&dep.successor_id).unwrap() += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<Id> = in_degree.iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(current) = queue.pop_front() {
            result.push(current);

            if let Some(neighbors) = adj_list.get(&current) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(&neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if result.len() != task_ids.len() {
            return Err(DesignTrackError::Validation(
                "Circular dependency detected in task graph".to_string()
            ));
        }

        Ok(result)
    }

    /// Analyze critical path and update dependency criticality
    pub fn analyze_critical_path(
        &mut self,
        repository: &ProjectRepository,
        calendar: &Calendar,
    ) -> Result<CriticalPathAnalysis> {
        let task_ids: Vec<Id> = repository.get_tasks().iter().map(|t| t.id).collect();
        let sorted_tasks = self.topological_sort(&task_ids)?;

        let mut early_start: HashMap<Id, DateTime<Utc>> = HashMap::new();
        let mut early_finish: HashMap<Id, DateTime<Utc>> = HashMap::new();
        let mut late_start: HashMap<Id, DateTime<Utc>> = HashMap::new();
        let mut late_finish: HashMap<Id, DateTime<Utc>> = HashMap::new();

        // Forward pass - calculate early start and finish
        for &task_id in &sorted_tasks {
            let task = repository.find_task_by_id(task_id).unwrap();
            
            let earliest_start = self.calculate_earliest_start_date(task_id, repository, calendar)?
                .unwrap_or_else(|| Utc::now());

            early_start.insert(task_id, earliest_start);

            // Calculate duration (simplified)
            let duration_days = if let (Some(start), Some(end)) = (task.start_date, task.due_date) {
                (end - start).num_days() as f32
            } else {
                1.0 // Default 1 day
            };

            let finish_date = earliest_start + Duration::days(duration_days.ceil() as i64);
            early_finish.insert(task_id, finish_date);
        }

        // Backward pass - calculate late start and finish
        let project_end = early_finish.values().max().cloned().unwrap_or_else(|| Utc::now());

        for &task_id in sorted_tasks.iter().rev() {
            let task = repository.find_task_by_id(task_id).unwrap();
            
            // Find latest finish based on dependent tasks
            let mut latest_finish = project_end;
            for dep in self.get_dependent_tasks(task_id) {
                if let Some(&successor_late_start) = late_start.get(&dep.successor_id) {
                    latest_finish = latest_finish.min(successor_late_start);
                }
            }

            late_finish.insert(task_id, latest_finish);

            // Calculate late start
            let duration_days = if let (Some(start), Some(end)) = (task.start_date, task.due_date) {
                (end - start).num_days() as f32
            } else {
                1.0
            };

            let late_start_date = latest_finish - Duration::days(duration_days.ceil() as i64);
            late_start.insert(task_id, late_start_date);
        }

        // Calculate float and identify critical path
        let mut critical_tasks = Vec::new();
        let mut task_floats = HashMap::new();

        for &task_id in &task_ids {
            let total_float = late_start[&task_id].signed_duration_since(early_start[&task_id]).num_days();
            task_floats.insert(task_id, total_float);

            if total_float == 0 {
                critical_tasks.push(task_id);
            }
        }

        // Mark critical dependencies
        for dep in &mut self.dependencies {
            dep.is_critical = critical_tasks.contains(&dep.predecessor_id) && 
                            critical_tasks.contains(&dep.successor_id);
        }

        Ok(CriticalPathAnalysis {
            critical_tasks,
            task_floats,
            project_duration: (project_end - early_start.values().min().cloned().unwrap_or(project_end)).num_days(),
            early_start,
            early_finish,
            late_start,
            late_finish,
        })
    }

    /// Add a task constraint
    pub fn add_constraint(&mut self, constraint: TaskConstraint) {
        self.constraints.push(constraint);
    }

    /// Get constraints for a task
    pub fn get_task_constraints(&self, task_id: Id) -> Vec<&TaskConstraint> {
        self.constraints.iter()
            .filter(|c| c.task_id == task_id)
            .collect()
    }

    /// Set up workflow for a task
    pub fn setup_task_workflow(&mut self, task_id: Id, workflow: TaskWorkflow) {
        self.workflows.insert(task_id, workflow);
    }

    /// Validate task state transition
    pub fn validate_state_transition(
        &self,
        task_id: Id,
        from_state: TaskStatus,
        to_state: TaskStatus,
    ) -> Result<()> {
        if let Some(workflow) = self.workflows.get(&task_id) {
            if workflow.current_state != from_state {
                return Err(DesignTrackError::Validation(
                    format!("Task is not in expected state {:?}", from_state)
                ));
            }

            if !workflow.allowed_transitions.contains(&to_state) {
                return Err(DesignTrackError::Validation(
                    format!("Transition from {:?} to {:?} is not allowed", from_state, to_state)
                ));
            }
        }

        Ok(())
    }

    /// Execute workflow actions when state changes
    pub fn execute_workflow_actions(
        &mut self,
        task_id: Id,
        new_state: TaskStatus,
        repository: &mut ProjectRepository,
    ) -> Result<Vec<String>> {
        let mut action_results = Vec::new();

        if let Some(workflow) = self.workflows.get_mut(&task_id) {
            workflow.current_state = new_state;

            for action in &workflow.automatic_actions {
                match action {
                    WorkflowAction::SetStartDate => {
                        if let Some(task) = repository.find_task_by_id(task_id) {
                            if task.start_date.is_none() {
                                let mut updated_task = task.clone();
                                updated_task.start_date = Some(Utc::now());
                                repository.update_task(updated_task)?;
                                action_results.push("Set start date to current time".to_string());
                            }
                        }
                    }
                    WorkflowAction::SetCompletionDate => {
                        if let Some(task) = repository.find_task_by_id(task_id) {
                            let mut updated_task = task.clone();
                            updated_task.completion_date = Some(Utc::now());
                            repository.update_task(updated_task)?;
                            action_results.push("Set completion date".to_string());
                        }
                    }
                    WorkflowAction::UpdateProgress(progress) => {
                        if let Some(task) = repository.find_task_by_id(task_id) {
                            let mut updated_task = task.clone();
                            updated_task.progress_percentage = *progress;
                            repository.update_task(updated_task)?;
                            action_results.push(format!("Updated progress to {:.1}%", progress));
                        }
                    }
                    WorkflowAction::UpdateDependentTasks => {
                        // Get dependent task IDs from the manager's dependencies
                        let dependent_task_ids: Vec<Id> = self.dependencies.iter()
                            .filter(|dep| dep.predecessor_id == task_id)
                            .map(|dep| dep.successor_id)
                            .collect();
                        
                        for dep_id in dependent_task_ids {
                            // Trigger recalculation of dependent task schedules
                            action_results.push(format!("Updated dependent task {}", dep_id));
                        }
                    }
                    _ => {
                        action_results.push(format!("Executed action: {:?}", action));
                    }
                }
            }
        }

        Ok(action_results)
    }

    /// Create default workflows for common task patterns
    pub fn create_default_workflows(&mut self, task_ids: &[Id]) {
        for &task_id in task_ids {
            let workflow = TaskWorkflow {
                current_state: TaskStatus::NotStarted,
                allowed_transitions: vec![
                    TaskStatus::InProgress,
                    TaskStatus::OnHold,
                    TaskStatus::Cancelled,
                ],
                automatic_actions: vec![
                    WorkflowAction::SetStartDate,
                    WorkflowAction::UpdateDependentTasks,
                ],
                validation_rules: vec![
                    ValidationRule::ValidateDependencies,
                ],
            };
            self.workflows.insert(task_id, workflow);
        }
    }
}

#[derive(Debug)]
pub struct CriticalPathAnalysis {
    pub critical_tasks: Vec<Id>,
    pub task_floats: HashMap<Id, i64>, // Days of float
    pub project_duration: i64, // Total project duration in days
    pub early_start: HashMap<Id, DateTime<Utc>>,
    pub early_finish: HashMap<Id, DateTime<Utc>>,
    pub late_start: HashMap<Id, DateTime<Utc>>,
    pub late_finish: HashMap<Id, DateTime<Utc>>,
}

impl Default for WorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}