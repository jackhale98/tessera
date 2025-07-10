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
        
        let completion_percentage = if total_tasks > 0 {
            (completed_tasks as f64 / total_tasks as f64) * 100.0
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