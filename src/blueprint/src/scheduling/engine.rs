use crate::core::{Project, Task, Calendar, DependencyType, TaskType};
use anyhow::Result;
use chrono::{Datelike, NaiveDate, Weekday};
use indexmap::IndexMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Schedule {
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub tasks: IndexMap<String, ScheduledTask>,
    pub milestones: IndexMap<String, ScheduledMilestone>,
    pub critical_path: Vec<String>,
    pub total_cost: f32,
    pub resource_utilization: HashMap<String, ResourceUtilization>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScheduledTask {
    pub task_id: String,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub assigned_to: String,
    pub effort: f32,
    pub cost: f32,
    pub is_critical: bool,
    pub slack: i64, // days
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScheduledMilestone {
    pub milestone_id: String,
    pub name: String,
    pub date: NaiveDate,
    pub is_critical: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceUtilization {
    pub resource_id: String,
    pub name: String,
    pub total_hours: f32,
    pub utilization_percentage: f32,
    pub task_assignments: Vec<TaskAssignment>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskAssignment {
    pub task_id: String,
    pub task_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub hours: f32,
}

pub struct SchedulingEngine {
    optimizer: Box<dyn crate::SchedulingAlgorithm>,
}

impl Default for SchedulingEngine {
    fn default() -> Self {
        Self {
            optimizer: Box::new(crate::scheduling::HeuristicOptimizer::new()),
        }
    }
}

impl SchedulingEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compute_schedule(&self, project: &Project) -> Result<Schedule> {
        // Build dependency graph
        let (graph, node_map) = self.build_dependency_graph(project)?;

        // Topological sort to get execution order
        let sorted_nodes = toposort(&graph, None)
            .map_err(|_| anyhow::anyhow!("Circular dependency detected"))?;

        // Assign resources and dates
        let mut scheduled_tasks = IndexMap::new();
        let mut resource_calendars: HashMap<String, NaiveDate> = HashMap::new();

        // Initialize resource calendars with project start date
        for (resource_id, _) in &project.resources {
            resource_calendars.insert(resource_id.clone(), project.start_date);
        }

        for node_idx in sorted_nodes {
            if let Some(task_id) = node_map.iter()
                .find(|(_, &idx)| idx == node_idx)
                .map(|(id, _)| id)
            {
                if let Some(task) = project.tasks.get(task_id) {
                    let scheduled_parts = self.schedule_task(
                        task_id,
                        task,
                        project,
                        &scheduled_tasks,
                        &mut resource_calendars,
                    )?;
                    
                    // Insert all scheduled task parts
                    for scheduled in scheduled_parts {
                        scheduled_tasks.insert(scheduled.task_id.clone(), scheduled);
                    }
                }
            }
        }

        // Calculate critical path and slack using proper algorithm
        let (critical_path, slack_map) = crate::scheduling::critical_path::CriticalPathAnalyzer::perform_critical_path_analysis(
            project,
            &scheduled_tasks,
            &graph,
            &node_map,
        )?;
        
        // Update scheduled tasks with critical path and slack information
        let mut updated_scheduled_tasks = scheduled_tasks.clone();
        for (task_id, task) in updated_scheduled_tasks.iter_mut() {
            task.is_critical = critical_path.contains(task_id);
            task.slack = slack_map.get(task_id).copied().unwrap_or(0);
        }
        let scheduled_tasks = updated_scheduled_tasks;

        // Calculate project end date
        let end_date = scheduled_tasks.values()
            .map(|t| t.end_date)
            .max()
            .unwrap_or(project.start_date);

        // Schedule milestones
        let scheduled_milestones = self.schedule_milestones(project, &scheduled_tasks)?;

        // Calculate total cost and resource utilization
        let (total_cost, resource_utilization) = self.calculate_metrics(
            &scheduled_tasks,
            project,
            project.start_date,
            end_date,
        )?;
        
        Ok(Schedule {
            project_name: project.name.clone(),
            start_date: project.start_date,
            end_date,
            tasks: scheduled_tasks,
            milestones: scheduled_milestones,
            critical_path,
            total_cost,
            resource_utilization,
        })
    }

    fn build_dependency_graph(
        &self,
        project: &Project,
    ) -> Result<(DiGraph<String, ()>, HashMap<String, NodeIndex>)> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // Add all tasks as nodes
        for task_id in project.tasks.keys() {
            let node = graph.add_node(task_id.clone());
            node_map.insert(task_id.clone(), node);
        }

        // Add dependency edges
        for (task_id, task) in &project.tasks {
            let task_node = node_map[task_id];
            for dep in &task.dependencies {
                if let Some(&dep_node) = node_map.get(&dep.task_id) {
                    graph.add_edge(dep_node, task_node, ());
                }
            }
        }

        Ok((graph, node_map))
    }

    fn schedule_task(
        &self,
        task_id: &str,
        task: &Task,
        project: &Project,
        scheduled_tasks: &IndexMap<String, ScheduledTask>,
        resource_calendars: &mut HashMap<String, NaiveDate>,
    ) -> Result<Vec<ScheduledTask>> {
        // Find the earliest start date based on dependencies
        let mut earliest_start = project.start_date;
        for dep in &task.dependencies {
            if let Some(dep_task) = scheduled_tasks.get(&dep.task_id) {
                let constraint_date = self.calculate_dependency_constraint_date(
                    dep_task,
                    &dep.dependency_type,
                    dep.lag_days,
                    project,
                )?;
                earliest_start = earliest_start.max(constraint_date);
            }
        }

        let mut scheduled_task_parts = Vec::new();

        // All tasks must have resource assignments
        if task.resource_assignments.is_empty() {
            anyhow::bail!("Task '{}' has no resource assignments. All tasks must have at least one resource assignment.", task_id);
        }

        for (resource_id, _assignment) in &task.resource_assignments {
            if let Some(resource) = project.resources.get(resource_id) {
                // Get calendar for this resource
                let calendar = project.get_calendar_for_resource(resource_id)
                    .or_else(|| project.get_default_calendar())
                    .ok_or_else(|| anyhow::anyhow!("No calendar available for resource '{}'", resource_id))?;

                // Calculate effort, duration, and dates based on task type
                let (effort_hours, _duration_days, start_date, end_date) = self.calculate_task_schedule(
                    task,
                    resource_id,
                    resource,
                    calendar,
                    earliest_start,
                    resource_calendars,
                )?;

                if effort_hours > 0.0 {
                    // Update resource calendar
                    resource_calendars.insert(
                        resource_id.clone(),
                        calendar.next_working_day(end_date),
                    );

                    // Calculate cost
                    let cost = effort_hours * resource.hourly_rate;

                    // Create unique task ID for this resource assignment
                    let subtask_id = if task.resource_assignments.len() > 1 {
                        format!("{}_{}", task_id, resource_id)
                    } else {
                        task_id.to_string()
                    };

                    scheduled_task_parts.push(ScheduledTask {
                        task_id: subtask_id,
                        name: task.name.clone(),
                        start_date,
                        end_date,
                        assigned_to: resource_id.clone(),
                        effort: effort_hours,
                        cost,
                        is_critical: false,
                        slack: 0,
                    });
                }
            }
        }

        if scheduled_task_parts.is_empty() {
            anyhow::bail!("No valid resource assignments found for task '{}'", task_id);
        }

        Ok(scheduled_task_parts)
    }


    fn calculate_dependency_constraint_date(
        &self,
        predecessor: &ScheduledTask,
        dependency_type: &DependencyType,
        lag_days: f32,
        project: &Project,
    ) -> Result<NaiveDate> {
        let base_date = match dependency_type {
            DependencyType::FinishToStart => predecessor.end_date,
            DependencyType::StartToStart => predecessor.start_date,
            DependencyType::FinishToFinish => predecessor.end_date,
            DependencyType::StartToFinish => predecessor.start_date,
        };

        let calendar = project.get_default_calendar()
            .ok_or_else(|| anyhow::anyhow!("No default calendar available"))?;

        // Apply lag/lead time
        if lag_days != 0.0 {
            if lag_days > 0.0 {
                // Positive lag - add working days
                Ok(calendar.add_working_days(base_date, lag_days))
            } else {
                // Negative lag (lead) - subtract working days
                let mut result_date = base_date;
                let mut remaining_days = -lag_days;
                
                while remaining_days > 0.0 {
                    result_date = calendar.previous_working_day(result_date);
                    remaining_days -= 1.0;
                }
                
                Ok(result_date)
            }
        } else {
            Ok(base_date)
        }
    }

    fn calculate_task_schedule(
        &self,
        task: &Task,
        resource_id: &str,
        resource: &crate::core::Resource,
        calendar: &Calendar,
        earliest_start: NaiveDate,
        resource_calendars: &HashMap<String, NaiveDate>,
    ) -> Result<(f32, f32, NaiveDate, NaiveDate)> {
        // Get resource's next available date
        let resource_available = resource_calendars.get(resource_id)
            .copied()
            .unwrap_or(earliest_start);
        let start_date = earliest_start.max(resource_available);

        // Determine effective task type based on available data
        let effective_task_type = if task.duration_days.is_some() && task.work_units.is_none() {
            // If duration is specified but work units are not, treat as FixedDuration
            TaskType::FixedDuration
        } else if task.work_units.is_some() {
            // If work units are specified, treat as FixedWork
            TaskType::FixedWork
        } else {
            // Otherwise use the specified task type
            task.task_type
        };
        
        
        // Calculate based on effective task type
        match effective_task_type {
            TaskType::EffortDriven => {
                // Effort is fixed, calculate duration
                let effort_hours = task.calculate_effort_hours_for_resource(resource_id, resource);
                let daily_hours = calendar.get_working_hours(start_date).min(resource.daily_hours());
                let duration_days = if daily_hours > 0.0 {
                    (effort_hours / daily_hours).ceil()
                } else {
                    1.0
                };
                
                // Debug large duration values
                if duration_days > 50.0 {
                    eprintln!("Warning: Large duration_days: {} for task (effort: {}, daily: {})", duration_days, effort_hours, daily_hours);
                }
                
                let end_date = calendar.add_working_days(start_date, duration_days);
                Ok((effort_hours, duration_days, start_date, end_date))
            }
            TaskType::FixedDuration => {
                // Duration is fixed, calculate effort based on resource assignment
                let duration_days = task.duration_days.unwrap_or(1) as f32;
                let effort_hours = task.calculate_effort_hours_for_resource(resource_id, resource);
                
                
                // Use the actual assignment effort if it exists, otherwise calculate from duration
                let final_effort_hours = if effort_hours > 0.0 {
                    effort_hours
                } else {
                    let daily_hours = calendar.get_working_hours(start_date).min(resource.daily_hours());
                    duration_days * daily_hours
                };
                
                let end_date = calendar.add_working_days(start_date, duration_days);
                
                Ok((final_effort_hours, duration_days, start_date, end_date))
            }
            TaskType::FixedWork => {
                // Work units are fixed, calculate effort and duration
                let work_units = task.work_units.unwrap_or(1.0);
                let daily_capacity = calendar.get_working_hours(start_date).min(resource.daily_hours());
                let duration_days = if daily_capacity > 0.0 {
                    (work_units / daily_capacity).ceil()
                } else {
                    1.0
                };
                
                let end_date = calendar.add_working_days(start_date, duration_days);
                Ok((work_units, duration_days, start_date, end_date))
            }
        }
    }



    fn calculate_metrics(
        &self,
        scheduled_tasks: &IndexMap<String, ScheduledTask>,
        project: &Project,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(f32, HashMap<String, ResourceUtilization>)> {
        let mut total_cost = 0.0;
        let mut resource_utilization = HashMap::new();

        // Initialize resource utilization
        for (resource_id, resource) in &project.resources {
            resource_utilization.insert(
                resource_id.clone(),
                ResourceUtilization {
                    resource_id: resource_id.clone(),
                    name: resource.name.clone(),
                    total_hours: 0.0,
                    utilization_percentage: 0.0,
                    task_assignments: Vec::new(),
                },
            );
        }

        // Calculate metrics from scheduled tasks
        for (task_id, scheduled_task) in scheduled_tasks {
            total_cost += scheduled_task.cost;

            if let Some(utilization) = resource_utilization.get_mut(&scheduled_task.assigned_to) {
                utilization.total_hours += scheduled_task.effort;
                utilization.task_assignments.push(TaskAssignment {
                    task_id: task_id.clone(),
                    task_name: scheduled_task.name.clone(),
                    start_date: scheduled_task.start_date,
                    end_date: scheduled_task.end_date,
                    hours: scheduled_task.effort,
                });
            }
        }

        // Calculate utilization percentages
        let project_duration_days = (end_date - start_date).num_days() as f32;
        let working_days = project_duration_days * 5.0 / 7.0; // Rough estimate

        for (resource_id, utilization) in resource_utilization.iter_mut() {
            if let Some(resource) = project.resources.get(resource_id) {
                let available_hours = working_days * resource.daily_hours();
                utilization.utilization_percentage =
                    (utilization.total_hours / available_hours * 100.0).min(100.0);
            }
        }

        Ok((total_cost, resource_utilization))
    }

    fn schedule_milestones(
        &self,
        project: &Project,
        scheduled_tasks: &IndexMap<String, ScheduledTask>,
    ) -> Result<IndexMap<String, ScheduledMilestone>> {
        let mut scheduled_milestones: IndexMap<String, ScheduledMilestone> = IndexMap::new();

        // Process milestones in dependency order (simple implementation)
        let mut remaining_milestones: Vec<_> = project.milestones.iter().collect();
        let mut processed = std::collections::HashSet::new();

        while !remaining_milestones.is_empty() {
            let mut made_progress = false;

            remaining_milestones.retain(|(milestone_id, milestone)| {
                // Check if all dependencies are satisfied
                let deps_satisfied = milestone.dependencies.iter().all(|dep_id| {
                    scheduled_tasks.contains_key(dep_id) || processed.contains(dep_id)
                });

                if deps_satisfied {
                    // Calculate milestone date based on dependencies
                    let mut milestone_date = if let Some(target_date) = milestone.target_date {
                        target_date
                    } else {
                        project.start_date
                    };

                    // Find latest dependency completion date
                    for dep_id in &milestone.dependencies {
                        if let Some(dep_task) = scheduled_tasks.get(dep_id) {
                            milestone_date = milestone_date.max(dep_task.end_date);
                        } else if let Some(dep_milestone) = scheduled_milestones.get(dep_id) {
                            milestone_date = milestone_date.max(dep_milestone.date);
                        }
                    }

                    // Create scheduled milestone
                    let is_critical = if let Some(target_date) = milestone.target_date {
                        milestone_date > target_date
                    } else {
                        false
                    };

                    let scheduled_milestone = ScheduledMilestone {
                        milestone_id: milestone_id.to_string(),
                        name: milestone.name.clone(),
                        date: milestone_date,
                        is_critical,
                    };

                    scheduled_milestones.insert(milestone_id.to_string(), scheduled_milestone);
                    processed.insert(milestone_id.to_string());
                    made_progress = true;
                    false // Remove from remaining list
                } else {
                    true // Keep in remaining list
                }
            });

            if !made_progress && !remaining_milestones.is_empty() {
                // Circular dependency or missing dependency - process remaining with warning
                for (milestone_id, milestone) in remaining_milestones {
                    let milestone_date = milestone.target_date.unwrap_or(project.start_date);
                    let scheduled_milestone = ScheduledMilestone {
                        milestone_id: milestone_id.to_string(),
                        name: milestone.name.clone(),
                        date: milestone_date,
                        is_critical: true, // Mark as critical due to dependency issues
                    };
                    scheduled_milestones.insert(milestone_id.to_string(), scheduled_milestone);
                }
                break;
            }
        }

        Ok(scheduled_milestones)
    }
}
