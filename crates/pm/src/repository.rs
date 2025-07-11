use crate::data::*;
use tessera_core::{Entity, Id, Result};
use std::path::Path;

pub struct ProjectRepository {
    tasks: Vec<Task>,
    resources: Vec<Resource>,
    milestones: Vec<Milestone>,
    schedules: Vec<ProjectSchedule>,
}

impl ProjectRepository {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            resources: Vec::new(),
            milestones: Vec::new(),
            schedules: Vec::new(),
        }
    }
    
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let mut repo = Self::new();
        
        let tasks_file = dir.join("tasks.ron");
        if tasks_file.exists() {
            repo.tasks = load_items_from_file(&tasks_file)?;
        }
        
        let resources_file = dir.join("resources.ron");
        if resources_file.exists() {
            repo.resources = load_items_from_file(&resources_file)?;
        }
        
        let milestones_file = dir.join("milestones.ron");
        if milestones_file.exists() {
            repo.milestones = load_items_from_file(&milestones_file)?;
        }
        
        let schedules_file = dir.join("schedules.ron");
        if schedules_file.exists() {
            repo.schedules = load_items_from_file(&schedules_file)?;
        }
        
        Ok(repo)
    }
    
    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        tessera_core::ensure_directory(dir)?;
        
        save_items_to_file(&self.tasks, dir.join("tasks.ron"))?;
        save_items_to_file(&self.resources, dir.join("resources.ron"))?;
        save_items_to_file(&self.milestones, dir.join("milestones.ron"))?;
        save_items_to_file(&self.schedules, dir.join("schedules.ron"))?;
        
        Ok(())
    }
    
    // Task methods
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        task.validate()?;
        self.tasks.push(task);
        Ok(())
    }
    
    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }
    
    pub fn find_task_by_id(&self, id: Id) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }
    
    pub fn update_task(&mut self, updated: Task) -> Result<()> {
        updated.validate()?;
        if let Some(pos) = self.tasks.iter().position(|t| t.id == updated.id) {
            self.tasks[pos] = updated;
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Task with id {} not found", updated.id)
            ))
        }
    }
    
    pub fn get_tasks_by_status(&self, status: &TaskStatus) -> Vec<&Task> {
        self.tasks.iter().filter(|t| std::mem::discriminant(&t.status) == std::mem::discriminant(status)).collect()
    }
    
    pub fn get_overdue_tasks(&self) -> Vec<&Task> {
        let now = chrono::Utc::now();
        self.tasks.iter()
            .filter(|t| !t.is_completed() && t.due_date.map_or(false, |due| due < now))
            .collect()
    }

    pub fn remove_task(&mut self, id: Id) -> Result<()> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            self.tasks.remove(pos);
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Task with id {} not found", id)
            ))
        }
    }
    
    // Resource methods
    pub fn add_resource(&mut self, resource: Resource) -> Result<()> {
        resource.validate()?;
        self.resources.push(resource);
        Ok(())
    }
    
    pub fn get_resources(&self) -> &[Resource] {
        &self.resources
    }
    
    pub fn find_resource_by_id(&self, id: Id) -> Option<&Resource> {
        self.resources.iter().find(|r| r.id == id)
    }
    
    pub fn update_resource(&mut self, updated: Resource) -> Result<()> {
        updated.validate()?;
        if let Some(pos) = self.resources.iter().position(|r| r.id == updated.id) {
            self.resources[pos] = updated;
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Resource with id {} not found", updated.id)
            ))
        }
    }

    pub fn remove_resource(&mut self, id: Id) -> Result<()> {
        if let Some(pos) = self.resources.iter().position(|r| r.id == id) {
            self.resources.remove(pos);
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Resource with id {} not found", id)
            ))
        }
    }
    
    // Milestone methods
    pub fn add_milestone(&mut self, milestone: Milestone) -> Result<()> {
        milestone.validate()?;
        self.milestones.push(milestone);
        Ok(())
    }
    
    pub fn get_milestones(&self) -> &[Milestone] {
        &self.milestones
    }
    
    pub fn find_milestone_by_id(&self, id: Id) -> Option<&Milestone> {
        self.milestones.iter().find(|m| m.id == id)
    }
    
    pub fn update_milestone(&mut self, updated: Milestone) -> Result<()> {
        updated.validate()?;
        if let Some(pos) = self.milestones.iter().position(|m| m.id == updated.id) {
            self.milestones[pos] = updated;
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Milestone with id {} not found", updated.id)
            ))
        }
    }
    
    pub fn get_overdue_milestones(&self) -> Vec<&Milestone> {
        self.milestones.iter().filter(|m| m.is_overdue()).collect()
    }

    pub fn remove_milestone(&mut self, id: Id) -> Result<()> {
        if let Some(pos) = self.milestones.iter().position(|m| m.id == id) {
            self.milestones.remove(pos);
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Milestone with id {} not found", id)
            ))
        }
    }
    
    // Schedule methods
    pub fn add_schedule(&mut self, schedule: ProjectSchedule) -> Result<()> {
        self.schedules.push(schedule);
        Ok(())
    }
    
    pub fn get_latest_schedule(&self) -> Option<&ProjectSchedule> {
        self.schedules.last()
    }
    
    pub fn get_all_schedules(&self) -> &[ProjectSchedule] {
        &self.schedules
    }
    
    // Analysis methods
    pub fn get_project_health(&self) -> ProjectHealth {
        let total_tasks = self.tasks.len();
        let completed_tasks = self.get_tasks_by_status(&TaskStatus::Completed).len();
        let overdue_tasks = self.get_overdue_tasks().len();
        let overdue_milestones = self.get_overdue_milestones().len();
        
        // Calculate weighted completion percentage based on task progress
        let completion_percentage = if total_tasks > 0 {
            let total_progress: f64 = self.tasks.iter()
                .map(|task| task.progress_percentage)
                .sum();
            total_progress / total_tasks as f64
        } else {
            0.0
        };
        
        ProjectHealth {
            total_tasks,
            completed_tasks,
            overdue_tasks,
            overdue_milestones,
            completion_percentage,
        }
    }

    /// Calculate project completion percentage based on task effort weighting
    pub fn get_effort_weighted_completion(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        let total_effort: f64 = self.tasks.iter()
            .map(|task| task.estimated_hours)
            .sum();

        if total_effort == 0.0 {
            // Fallback to simple average if no effort estimates
            return self.tasks.iter()
                .map(|task| task.progress_percentage)
                .sum::<f64>() / self.tasks.len() as f64;
        }

        let weighted_progress: f64 = self.tasks.iter()
            .map(|task| (task.progress_percentage / 100.0) * task.estimated_hours)
            .sum();

        (weighted_progress / total_effort) * 100.0
    }

    /// Calculate total project cost based on resource rates and task assignments
    pub fn calculate_project_cost(&self) -> ProjectCost {
        let mut total_estimated_cost = 0.0;
        let mut total_actual_cost = 0.0;
        let mut task_costs = Vec::new();

        for task in &self.tasks {
            let task_cost = self.calculate_task_cost(task);
            total_estimated_cost += task_cost.estimated_cost;
            total_actual_cost += task_cost.actual_cost;
            task_costs.push(task_cost);
        }

        ProjectCost {
            total_estimated_cost,
            total_actual_cost,
            task_costs,
            cost_variance: total_actual_cost - total_estimated_cost,
            cost_variance_percentage: if total_estimated_cost > 0.0 {
                ((total_actual_cost - total_estimated_cost) / total_estimated_cost) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Calculate cost for a specific task
    pub fn calculate_task_cost(&self, task: &Task) -> TaskCost {
        let mut estimated_cost = 0.0;
        let mut actual_cost = 0.0;
        let mut resource_costs = Vec::new();

        for assignment in &task.assigned_resources {
            if let Some(resource) = self.find_resource_by_id(assignment.resource_id) {
                let hourly_rate = assignment.rate_override
                    .or(resource.hourly_rate)
                    .unwrap_or(0.0);

                // Calculate estimated hours for this resource based on allocation
                let estimated_hours = if assignment.assigned_hours.is_some() {
                    assignment.assigned_hours.unwrap_or(0.0)
                } else {
                    match task.task_type {
                        TaskType::EffortDriven => {
                            task.estimated_hours * (assignment.allocation_percentage / 100.0)
                        }
                        TaskType::FixedDuration => {
                            if let Some(duration) = task.duration_days {
                                duration * resource.daily_hours * (assignment.allocation_percentage / 100.0)
                            } else {
                                0.0
                            }
                        }
                        TaskType::FixedWork => {
                            task.work_units.unwrap_or(0.0) * (assignment.allocation_percentage / 100.0)
                        }
                        TaskType::Milestone => 0.0,
                    }
                };

                // Calculate actual hours based on progress
                let actual_hours = estimated_hours * (task.progress_percentage / 100.0);

                let resource_estimated_cost = estimated_hours * hourly_rate;
                let resource_actual_cost = actual_hours * hourly_rate;

                estimated_cost += resource_estimated_cost;
                actual_cost += resource_actual_cost;

                resource_costs.push(ResourceCost {
                    resource_id: assignment.resource_id,
                    resource_name: resource.name.clone(),
                    hourly_rate,
                    estimated_hours,
                    actual_hours,
                    estimated_cost: resource_estimated_cost,
                    actual_cost: resource_actual_cost,
                });
            }
        }

        TaskCost {
            task_id: task.id,
            task_name: task.name.clone(),
            estimated_cost,
            actual_cost,
            resource_costs,
            cost_variance: actual_cost - estimated_cost,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectCost {
    pub total_estimated_cost: f64,
    pub total_actual_cost: f64,
    pub task_costs: Vec<TaskCost>,
    pub cost_variance: f64,
    pub cost_variance_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct TaskCost {
    pub task_id: Id,
    pub task_name: String,
    pub estimated_cost: f64,
    pub actual_cost: f64,
    pub resource_costs: Vec<ResourceCost>,
    pub cost_variance: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceCost {
    pub resource_id: Id,
    pub resource_name: String,
    pub hourly_rate: f64,
    pub estimated_hours: f64,
    pub actual_hours: f64,
    pub estimated_cost: f64,
    pub actual_cost: f64,
}

#[derive(Debug)]
pub struct ProjectHealth {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub overdue_tasks: usize,
    pub overdue_milestones: usize,
    pub completion_percentage: f64,
}

// Helper functions for loading/saving RON files
pub fn load_items_from_file<T, P>(path: P) -> Result<Vec<T>>
where
    T: for<'de> serde::Deserialize<'de>,
    P: AsRef<Path>,
{
    let content = std::fs::read_to_string(path)?;
    let items: Vec<T> = ron::from_str(&content)?;
    Ok(items)
}

pub fn save_items_to_file<T, P>(items: &[T], path: P) -> Result<()>
where
    T: serde::Serialize,
    P: AsRef<Path>,
{
    let content = tessera_core::format_ron_pretty(items)?;
    std::fs::write(path, content)?;
    Ok(())
}