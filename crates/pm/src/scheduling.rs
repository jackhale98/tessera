use crate::data::*;
use tessera_core::{Id, Result};
use chrono::{DateTime, Duration, Utc};
use petgraph::{Graph, Direction};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

pub struct ProjectScheduler {
    config: SchedulingConfig,
}

#[derive(Debug, Clone)]
pub struct SchedulingConfig {
    pub working_hours_per_day: f64,
    pub working_days_per_week: u32,
    pub buffer_percentage: f64, // Extra time buffer for uncertainty
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        Self {
            working_hours_per_day: 8.0,
            working_days_per_week: 5,
            buffer_percentage: 0.1, // 10% buffer
        }
    }
}

impl ProjectScheduler {
    pub fn new(config: SchedulingConfig) -> Self {
        Self { config }
    }
    
    pub fn compute_schedule(&self, tasks: &[Task], _resources: &[Resource], project_start: DateTime<Utc>) -> Result<ProjectSchedule> {
        if tasks.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Cannot schedule empty task list".to_string()
            ));
        }
        
        // Build dependency graph
        let (graph, task_nodes) = self.build_dependency_graph(tasks)?;
        
        // Perform topological sort to find valid task order
        let sorted_tasks = self.topological_sort(&graph, &task_nodes)?;
        
        // Calculate early start/finish times (forward pass)
        let mut task_schedules = self.calculate_early_times(&sorted_tasks, tasks, project_start)?;
        
        // Calculate late start/finish times (backward pass)
        let project_end = task_schedules.values()
            .map(|ts| ts.earliest_finish)
            .max()
            .unwrap_or(project_start);
        
        self.calculate_late_times(&mut task_schedules, &sorted_tasks, tasks, project_end)?;
        
        // Calculate free float for all tasks
        self.calculate_free_floats(&mut task_schedules, tasks)?;
        
        // Identify critical path
        let critical_path = self.find_critical_path(&task_schedules);
        
        let total_duration_days = (project_end - project_start).num_days();
        
        Ok(ProjectSchedule {
            generated: Utc::now(),
            project_start,
            project_end,
            critical_path,
            total_duration_days,
            task_schedule: task_schedules,
        })
    }
    
    fn build_dependency_graph(&self, tasks: &[Task]) -> Result<(Graph<Id, ()>, HashMap<Id, NodeIndex>)> {
        let mut graph = Graph::new();
        let mut task_nodes = HashMap::new();
        
        // Add all tasks as nodes
        for task in tasks {
            let node_index = graph.add_node(task.id);
            task_nodes.insert(task.id, node_index);
        }
        
        // Add dependency edges
        for task in tasks {
            let task_node = task_nodes[&task.id];
            for dep in &task.dependencies {
                if let Some(&dep_node) = task_nodes.get(&dep.predecessor_id) {
                    // Edge from dependency to task (dependency must finish before task starts)
                    graph.add_edge(dep_node, task_node, ());
                }
            }
        }
        
        Ok((graph, task_nodes))
    }
    
    fn topological_sort(&self, graph: &Graph<Id, ()>, task_nodes: &HashMap<Id, NodeIndex>) -> Result<Vec<Id>> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();
        
        for &node_index in task_nodes.values() {
            if !visited.contains(&node_index) {
                self.dfs_visit(graph, node_index, &mut visited, &mut temp_visited, &mut sorted)?;
            }
        }
        
        // Reverse to get correct topological order
        sorted.reverse();
        Ok(sorted)
    }
    
    fn dfs_visit(
        &self,
        graph: &Graph<Id, ()>,
        node: NodeIndex,
        visited: &mut std::collections::HashSet<NodeIndex>,
        temp_visited: &mut std::collections::HashSet<NodeIndex>,
        sorted: &mut Vec<Id>,
    ) -> Result<()> {
        if temp_visited.contains(&node) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Circular dependency detected in task graph".to_string()
            ));
        }
        
        if visited.contains(&node) {
            return Ok(());
        }
        
        temp_visited.insert(node);
        
        // Visit all neighbors (tasks that depend on this one)
        for edge in graph.edges_directed(node, Direction::Outgoing) {
            self.dfs_visit(graph, edge.target(), visited, temp_visited, sorted)?;
        }
        
        temp_visited.remove(&node);
        visited.insert(node);
        sorted.push(graph[node]);
        
        Ok(())
    }
    
    fn calculate_early_times(
        &self,
        sorted_tasks: &[Id],
        tasks: &[Task],
        project_start: DateTime<Utc>,
    ) -> Result<indexmap::IndexMap<Id, TaskScheduleInfo>> {
        let mut task_schedules: indexmap::IndexMap<Id, TaskScheduleInfo> = indexmap::IndexMap::new();
        let task_map: HashMap<Id, &Task> = tasks.iter().map(|t| (t.id, t)).collect();
        
        for &task_id in sorted_tasks {
            let task = task_map[&task_id];
            
            // Calculate earliest start based on dependencies
            let earliest_start = if task.dependencies.is_empty() {
                project_start
            } else {
                let mut max_constraint_date = project_start;
                
                for dep in &task.dependencies {
                    if let Some(pred_schedule) = task_schedules.get(&dep.predecessor_id) {
                        let constraint_date = match dep.dependency_type {
                            DependencyType::FinishToStart => {
                                // Successor starts after predecessor finishes + lag
                                pred_schedule.earliest_finish + Duration::days(dep.lag_days.ceil() as i64)
                            },
                            DependencyType::StartToStart => {
                                // Successor starts after predecessor starts + lag  
                                pred_schedule.earliest_start + Duration::days(dep.lag_days.ceil() as i64)
                            },
                            DependencyType::FinishToFinish => {
                                // Successor must finish after predecessor finishes + lag
                                // So earliest start = pred finish + lag - task duration
                                let task_duration = self.calculate_task_duration(task);
                                pred_schedule.earliest_finish + Duration::days(dep.lag_days.ceil() as i64) - task_duration
                            },
                            DependencyType::StartToFinish => {
                                // Successor must finish after predecessor starts + lag
                                // So earliest start = pred start + lag - task duration  
                                let task_duration = self.calculate_task_duration(task);
                                pred_schedule.earliest_start + Duration::days(dep.lag_days.ceil() as i64) - task_duration
                            },
                        };
                        
                        if constraint_date > max_constraint_date {
                            max_constraint_date = constraint_date;
                        }
                    }
                }
                
                max_constraint_date
            };
            
            let duration = self.calculate_task_duration(task);
            let schedule_info = TaskScheduleInfo::new(task_id, earliest_start, duration);
            task_schedules.insert(task_id, schedule_info);
        }
        
        Ok(task_schedules)
    }
    
    fn calculate_late_times(
        &self,
        task_schedules: &mut indexmap::IndexMap<Id, TaskScheduleInfo>,
        sorted_tasks: &[Id],
        tasks: &[Task],
        project_end: DateTime<Utc>,
    ) -> Result<()> {
        let task_map: HashMap<Id, &Task> = tasks.iter().map(|t| (t.id, t)).collect();
        
        // Work backwards through the tasks
        for &task_id in sorted_tasks.iter().rev() {
            let task = task_map[&task_id];
            
            // Find tasks that depend on this one
            let dependents: Vec<_> = tasks
                .iter()
                .filter(|t| t.dependencies.iter().any(|dep| dep.predecessor_id == task_id))
                .collect();
            
            let latest_finish = if dependents.is_empty() {
                // No dependents, can finish at project end
                project_end
            } else {
                // Find the minimum constraint from all dependent tasks
                let mut min_constraint_date = project_end;
                
                for dependent in dependents {
                    if let Some(dep_schedule) = task_schedules.get(&dependent.id) {
                        // Find the dependency relationship for this dependent task
                        if let Some(dependency) = dependent.dependencies.iter()
                            .find(|dep| dep.predecessor_id == task_id) {
                            
                            let constraint_date = match dependency.dependency_type {
                                DependencyType::FinishToStart => {
                                    // This task must finish before dependent starts - lag
                                    dep_schedule.latest_start - Duration::days(dependency.lag_days.ceil() as i64)
                                },
                                DependencyType::StartToStart => {
                                    // This task must start before dependent starts - lag
                                    // So latest finish = latest start + task duration
                                    let constraint_start = dep_schedule.latest_start - Duration::days(dependency.lag_days.ceil() as i64);
                                    constraint_start + self.calculate_task_duration(task)
                                },
                                DependencyType::FinishToFinish => {
                                    // This task must finish before dependent finishes - lag
                                    dep_schedule.latest_finish - Duration::days(dependency.lag_days.ceil() as i64)
                                },
                                DependencyType::StartToFinish => {
                                    // This task must start before dependent finishes - lag
                                    // So latest finish = latest start + task duration
                                    let constraint_start = dep_schedule.latest_finish - Duration::days(dependency.lag_days.ceil() as i64);
                                    constraint_start + self.calculate_task_duration(task)
                                },
                            };
                            
                            if constraint_date < min_constraint_date {
                                min_constraint_date = constraint_date;
                            }
                        }
                    }
                }
                
                min_constraint_date
            };
            
            let duration = self.calculate_task_duration(task);
            
            // Update the schedule info
            if let Some(schedule_info) = task_schedules.get_mut(&task_id) {
                schedule_info.latest_finish = latest_finish;
                schedule_info.latest_start = latest_finish - duration;
                
                // Calculate total float (slack)
                let total_float = (schedule_info.latest_start - schedule_info.earliest_start).num_days();
                schedule_info.slack_days = total_float;
                
                // A task is critical if it has zero total float
                schedule_info.is_critical = total_float <= 0;
            }
        }
        
        Ok(())
    }
    
    fn calculate_free_floats(
        &self,
        task_schedules: &mut indexmap::IndexMap<Id, TaskScheduleInfo>,
        tasks: &[Task],
    ) -> Result<()> {
        // Calculate all free floats first, then update the schedule infos
        let task_ids: Vec<Id> = task_schedules.keys().cloned().collect();
        let mut free_floats = HashMap::new();
        
        for task_id in task_ids {
            let free_float = self.calculate_free_float(task_id, task_schedules, tasks);
            free_floats.insert(task_id, free_float);
        }
        
        // Update the schedule infos with calculated free floats
        for (task_id, free_float) in free_floats {
            if let Some(schedule_info) = task_schedules.get_mut(&task_id) {
                schedule_info.free_float_days = free_float;
            }
        }
        
        Ok(())
    }
    
    fn calculate_task_duration(&self, task: &Task) -> Duration {
        let duration_days = match task.task_type {
            TaskType::EffortDriven => {
                // Duration = Effort / (Total Resource Allocation * Hours per Day)
                if !task.assigned_resources.is_empty() {
                    let total_allocation: f64 = task.assigned_resources.iter()
                        .map(|res| res.allocation_percentage / 100.0)
                        .sum();
                    if total_allocation > 0.0 {
                        task.estimated_hours / (total_allocation * self.config.working_hours_per_day)
                    } else {
                        task.estimated_hours / self.config.working_hours_per_day
                    }
                } else {
                    task.estimated_hours / self.config.working_hours_per_day
                }
            },
            TaskType::FixedDuration => {
                // Duration is fixed
                task.duration_days.unwrap_or(1.0)
            },
            TaskType::FixedWork => {
                // Duration = Work Units / (Total Resource Allocation * Hours per Day)
                let work_units = task.work_units.unwrap_or(task.estimated_hours);
                if !task.assigned_resources.is_empty() {
                    let total_allocation: f64 = task.assigned_resources.iter()
                        .map(|res| res.allocation_percentage / 100.0)
                        .sum();
                    if total_allocation > 0.0 {
                        work_units / (total_allocation * self.config.working_hours_per_day)
                    } else {
                        work_units / self.config.working_hours_per_day
                    }
                } else {
                    work_units / self.config.working_hours_per_day
                }
            },
            TaskType::Milestone => {
                // Milestones have zero duration
                0.0
            }
        };
        
        // Apply buffer percentage and convert to Duration
        let final_days = (duration_days * (1.0 + self.config.buffer_percentage)).ceil() as i64;
        Duration::days(final_days.max(0)) // Ensure non-negative duration
    }
    
    fn find_critical_path(&self, task_schedules: &indexmap::IndexMap<Id, TaskScheduleInfo>) -> Vec<Id> {
        // Find all critical tasks (tasks with zero or negative float)
        let critical_tasks: Vec<Id> = task_schedules
            .values()
            .filter(|ts| ts.is_critical)
            .map(|ts| ts.task_id)
            .collect();
        
        // For now, return all critical tasks
        // TODO: Implement path-finding algorithm to find the actual longest path
        // through the critical tasks from project start to end
        critical_tasks
    }
    
    /// Calculate free float for a task (time a task can be delayed without affecting any successor)
    pub fn calculate_free_float(&self, task_id: Id, task_schedules: &indexmap::IndexMap<Id, TaskScheduleInfo>, tasks: &[Task]) -> i64 {
        let task_schedule = match task_schedules.get(&task_id) {
            Some(ts) => ts,
            None => return 0,
        };
        
        // Find the earliest start of immediate successors
        let mut min_successor_start = task_schedule.latest_finish;
        
        for task in tasks {
            for dep in &task.dependencies {
                if dep.predecessor_id == task_id {
                    if let Some(successor_schedule) = task_schedules.get(&task.id) {
                        let successor_constraint = match dep.dependency_type {
                            DependencyType::FinishToStart => {
                                successor_schedule.earliest_start - Duration::days(dep.lag_days.ceil() as i64)
                            },
                            DependencyType::StartToStart => {
                                // Successor starts relative to predecessor start
                                successor_schedule.earliest_start - Duration::days(dep.lag_days.ceil() as i64) - 
                                    (task_schedule.earliest_finish - task_schedule.earliest_start)
                            },
                            DependencyType::FinishToFinish => {
                                successor_schedule.earliest_finish - Duration::days(dep.lag_days.ceil() as i64)
                            },
                            DependencyType::StartToFinish => {
                                successor_schedule.earliest_finish - Duration::days(dep.lag_days.ceil() as i64) -
                                    (task_schedule.earliest_finish - task_schedule.earliest_start)
                            },
                        };
                        
                        if successor_constraint < min_successor_start {
                            min_successor_start = successor_constraint;
                        }
                    }
                }
            }
        }
        
        // Free float = min successor constraint - earliest finish of this task
        (min_successor_start - task_schedule.earliest_finish).num_days()
    }
}

impl Default for ProjectScheduler {
    fn default() -> Self {
        Self::new(SchedulingConfig::default())
    }
}