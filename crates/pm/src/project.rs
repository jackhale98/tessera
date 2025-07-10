use crate::{Task, Resource, Milestone, ProjectRepository};
use tessera_core::Id;
use std::collections::HashMap;

/// Project wrapper that provides a consolidated view of project data
#[derive(Debug)]
pub struct Project {
    pub tasks: HashMap<Id, Task>,
    pub resources: HashMap<Id, Resource>, 
    pub milestones: HashMap<Id, Milestone>,
}

impl Project {
    /// Create project from repository data
    pub fn from_repository(repository: &ProjectRepository) -> Self {
        let tasks = repository.get_tasks()
            .iter()
            .map(|task| (task.id, task.clone()))
            .collect();
            
        let resources = repository.get_resources()
            .iter()
            .map(|resource| (resource.id, resource.clone()))
            .collect();
            
        let milestones = repository.get_milestones()
            .iter()
            .map(|milestone| (milestone.id, milestone.clone()))
            .collect();

        Self {
            tasks,
            resources,
            milestones,
        }
    }

    /// Update repository with project changes
    pub fn update_repository(&self, repository: &mut ProjectRepository) -> tessera_core::Result<()> {
        // Update all tasks
        for task in self.tasks.values() {
            repository.update_task(task.clone())?;
        }
        
        // Update all resources  
        for resource in self.resources.values() {
            repository.update_resource(resource.clone())?;
        }
        
        // Update all milestones
        for milestone in self.milestones.values() {
            repository.update_milestone(milestone.clone())?;
        }
        
        Ok(())
    }
}