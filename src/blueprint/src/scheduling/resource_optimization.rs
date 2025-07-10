use crate::core::Project;
use crate::scheduling::{Schedule, ScheduledTask};
use anyhow::Result;
use chrono::NaiveDate;
use indexmap::IndexMap;

pub trait ResourceLevelingAlgorithm: Send + Sync {
    fn level_resources(&self, schedule: &mut Schedule, project: &Project) -> Result<()>;
}

pub trait ResourceSmoothingAlgorithm: Send + Sync {
    fn smooth_resources(&self, schedule: &mut Schedule, project: &Project) -> Result<()>;
}

pub trait ResourceOptimizer: Send + Sync {
    fn optimize(&self, schedule: &mut Schedule, project: &Project) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ResourceConflict {
    pub resource_id: String,
    pub date: NaiveDate,
    pub required_capacity: f32,
    pub available_capacity: f32,
    pub conflicting_tasks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceUtilizationMetrics {
    pub resource_id: String,
    pub peak_utilization: f32,
    pub average_utilization: f32,
    pub underutilized_days: u32,
    pub overallocated_days: u32,
    pub utilization_variance: f32,
}

pub struct BasicResourceLeveling;

impl ResourceLevelingAlgorithm for BasicResourceLeveling {
    fn level_resources(&self, schedule: &mut Schedule, project: &Project) -> Result<()> {
        // Placeholder implementation - this will be implemented in the future
        // Resource leveling adjusts task start dates to resolve resource conflicts
        // while maintaining the critical path
        
        let conflicts = self.detect_resource_conflicts(schedule, project)?;
        
        for conflict in conflicts {
            // Defer non-critical tasks to resolve conflicts
            self.resolve_conflict(schedule, project, &conflict)?;
        }
        
        Ok(())
    }
}

impl BasicResourceLeveling {
    pub fn new() -> Self {
        Self
    }

    fn detect_resource_conflicts(
        &self,
        schedule: &Schedule,
        project: &Project,
    ) -> Result<Vec<ResourceConflict>> {
        let mut conflicts = Vec::new();
        
        // Group tasks by resource and check for overlapping periods
        for (resource_id, resource) in &project.resources {
            let resource_tasks: Vec<_> = schedule.tasks.values()
                .filter(|task| task.assigned_to == *resource_id)
                .collect();
            
            // Check for date overlaps and capacity violations
            for task in &resource_tasks {
                let mut current_date = task.start_date;
                while current_date <= task.end_date {
                    let daily_demand = self.calculate_daily_resource_demand(
                        current_date,
                        resource_id,
                        &schedule.tasks,
                    );
                    
                    if daily_demand > resource.capacity {
                        let conflicting_tasks = self.get_conflicting_tasks(
                            current_date,
                            resource_id,
                            &schedule.tasks,
                        );
                        
                        conflicts.push(ResourceConflict {
                            resource_id: resource_id.clone(),
                            date: current_date,
                            required_capacity: daily_demand,
                            available_capacity: resource.capacity,
                            conflicting_tasks,
                        });
                    }
                    
                    current_date = current_date.succ_opt().unwrap();
                }
            }
        }
        
        Ok(conflicts)
    }

    fn resolve_conflict(
        &self,
        _schedule: &mut Schedule,
        _project: &Project,
        _conflict: &ResourceConflict,
    ) -> Result<()> {
        // Placeholder for conflict resolution logic
        // This would implement actual task rescheduling
        Ok(())
    }

    fn calculate_daily_resource_demand(
        &self,
        date: NaiveDate,
        resource_id: &str,
        tasks: &IndexMap<String, ScheduledTask>,
    ) -> f32 {
        tasks.values()
            .filter(|task| {
                task.assigned_to == resource_id
                    && task.start_date <= date
                    && task.end_date >= date
            })
            .map(|task| {
                // Calculate daily allocation for this task
                let task_duration = (task.end_date - task.start_date).num_days() as f32 + 1.0;
                task.effort / task_duration / 8.0 // Assuming 8-hour days
            })
            .sum()
    }

    fn get_conflicting_tasks(
        &self,
        date: NaiveDate,
        resource_id: &str,
        tasks: &IndexMap<String, ScheduledTask>,
    ) -> Vec<String> {
        tasks.values()
            .filter(|task| {
                task.assigned_to == resource_id
                    && task.start_date <= date
                    && task.end_date >= date
            })
            .map(|task| task.task_id.clone())
            .collect()
    }
}

pub struct BasicResourceSmoothing;

impl ResourceSmoothingAlgorithm for BasicResourceSmoothing {
    fn smooth_resources(&self, schedule: &mut Schedule, project: &Project) -> Result<()> {
        // Placeholder implementation - this will be implemented in the future
        // Resource smoothing adjusts resource allocations to minimize utilization variance
        // while keeping the original schedule dates
        
        let metrics = self.calculate_utilization_metrics(schedule, project)?;
        
        for metric in metrics {
            if metric.utilization_variance > 0.5 {
                // High variance - smooth the allocation
                self.smooth_resource_allocation(schedule, &metric.resource_id)?;
            }
        }
        
        Ok(())
    }
}

impl BasicResourceSmoothing {
    pub fn new() -> Self {
        Self
    }

    fn calculate_utilization_metrics(
        &self,
        schedule: &Schedule,
        project: &Project,
    ) -> Result<Vec<ResourceUtilizationMetrics>> {
        let mut metrics = Vec::new();
        
        for (resource_id, resource) in &project.resources {
            let resource_tasks: Vec<_> = schedule.tasks.values()
                .filter(|task| task.assigned_to == *resource_id)
                .collect();
            
            if resource_tasks.is_empty() {
                continue;
            }
            
            // Calculate daily utilizations
            let start_date = resource_tasks.iter().map(|t| t.start_date).min().unwrap();
            let end_date = resource_tasks.iter().map(|t| t.end_date).max().unwrap();
            
            let mut daily_utilizations = Vec::new();
            let mut current_date = start_date;
            
            while current_date <= end_date {
                let daily_demand = self.calculate_daily_resource_demand(
                    current_date,
                    resource_id,
                    &schedule.tasks,
                );
                let utilization = daily_demand / resource.capacity;
                daily_utilizations.push(utilization);
                current_date = current_date.succ_opt().unwrap();
            }
            
            // Calculate metrics
            let peak = daily_utilizations.iter().cloned().fold(0.0f32, f32::max);
            let average = daily_utilizations.iter().sum::<f32>() / daily_utilizations.len() as f32;
            let underutilized = daily_utilizations.iter().filter(|&&u| u < 0.5).count() as u32;
            let overallocated = daily_utilizations.iter().filter(|&&u| u > 1.0).count() as u32;
            
            let variance = daily_utilizations.iter()
                .map(|u| (u - average).powi(2))
                .sum::<f32>() / daily_utilizations.len() as f32;
            
            metrics.push(ResourceUtilizationMetrics {
                resource_id: resource_id.clone(),
                peak_utilization: peak,
                average_utilization: average,
                underutilized_days: underutilized,
                overallocated_days: overallocated,
                utilization_variance: variance.sqrt(),
            });
        }
        
        Ok(metrics)
    }

    fn smooth_resource_allocation(
        &self,
        _schedule: &mut Schedule,
        _resource_id: &str,
    ) -> Result<()> {
        // Placeholder for allocation smoothing logic
        Ok(())
    }

    fn calculate_daily_resource_demand(
        &self,
        date: NaiveDate,
        resource_id: &str,
        tasks: &IndexMap<String, ScheduledTask>,
    ) -> f32 {
        tasks.values()
            .filter(|task| {
                task.assigned_to == resource_id
                    && task.start_date <= date
                    && task.end_date >= date
            })
            .map(|task| {
                let task_duration = (task.end_date - task.start_date).num_days() as f32 + 1.0;
                task.effort / task_duration / 8.0
            })
            .sum()
    }
}

pub struct CompositeResourceOptimizer {
    leveling: Box<dyn ResourceLevelingAlgorithm>,
    smoothing: Box<dyn ResourceSmoothingAlgorithm>,
}

impl CompositeResourceOptimizer {
    pub fn new(
        leveling: Box<dyn ResourceLevelingAlgorithm>,
        smoothing: Box<dyn ResourceSmoothingAlgorithm>,
    ) -> Self {
        Self { leveling, smoothing }
    }

    pub fn with_defaults() -> Self {
        Self {
            leveling: Box::new(BasicResourceLeveling::new()),
            smoothing: Box::new(BasicResourceSmoothing::new()),
        }
    }
}

impl ResourceOptimizer for CompositeResourceOptimizer {
    fn optimize(&self, schedule: &mut Schedule, project: &Project) -> Result<()> {
        // First level resources to resolve conflicts
        self.leveling.level_resources(schedule, project)?;
        
        // Then smooth resources to optimize utilization
        self.smoothing.smooth_resources(schedule, project)?;
        
        Ok(())
    }
}

impl Default for CompositeResourceOptimizer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Project, Resource, Task};
    use chrono::NaiveDate;

    #[test]
    fn test_resource_conflict_detection() {
        let leveling = BasicResourceLeveling::new();
        let mut project = Project::new("Test".to_string(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        
        // Add a resource with capacity 1.0
        let resource = Resource::new("dev1".to_string(), 100.0);
        project.add_resource("dev1".to_string(), resource);
        
        // Create a schedule with overlapping tasks
        let mut schedule = Schedule {
            project_name: "Test".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            tasks: IndexMap::new(),
            milestones: IndexMap::new(),
            critical_path: Vec::new(),
            total_cost: 0.0,
            resource_utilization: std::collections::HashMap::new(),
        };
        
        // This test verifies the conflict detection structure is in place
        let conflicts = leveling.detect_resource_conflicts(&schedule, &project).unwrap();
        assert!(conflicts.is_empty()); // No tasks yet, so no conflicts
    }

    #[test]
    fn test_utilization_metrics_calculation() {
        let smoothing = BasicResourceSmoothing::new();
        let project = Project::new("Test".to_string(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        
        let schedule = Schedule {
            project_name: "Test".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            tasks: IndexMap::new(),
            milestones: IndexMap::new(),
            critical_path: Vec::new(),
            total_cost: 0.0,
            resource_utilization: std::collections::HashMap::new(),
        };
        
        let metrics = smoothing.calculate_utilization_metrics(&schedule, &project).unwrap();
        assert!(metrics.is_empty()); // No resources or tasks
    }
}