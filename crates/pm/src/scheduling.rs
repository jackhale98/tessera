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
            for &dep_id in &task.dependencies {
                if let Some(&dep_node) = task_nodes.get(&dep_id) {
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
                task.dependencies
                    .iter()
                    .filter_map(|&dep_id| task_schedules.get(&dep_id))
                    .map(|ts| ts.earliest_finish)
                    .max()
                    .unwrap_or(project_start)
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
                .filter(|t| t.dependencies.contains(&task_id))
                .collect();
            
            let latest_finish = if dependents.is_empty() {
                // No dependents, can finish at project end
                project_end
            } else {
                // Find the minimum latest start time of dependent tasks
                let mut min_latest_start = project_end;
                for dependent in dependents {
                    if let Some(dep_schedule) = task_schedules.get(&dependent.id) {
                        if dep_schedule.latest_start < min_latest_start {
                            min_latest_start = dep_schedule.latest_start;
                        }
                    }
                }
                min_latest_start
            };
            
            let duration = self.calculate_task_duration(task);
            
            // Update the schedule info
            if let Some(schedule_info) = task_schedules.get_mut(&task_id) {
                schedule_info.latest_finish = latest_finish;
                schedule_info.latest_start = latest_finish - duration;
                schedule_info.slack_days = (schedule_info.latest_start - schedule_info.earliest_start).num_days();
                schedule_info.is_critical = schedule_info.slack_days == 0;
            }
        }
        
        Ok(())
    }
    
    fn calculate_task_duration(&self, task: &Task) -> Duration {
        let base_hours = task.estimated_hours * (1.0 + self.config.buffer_percentage);
        let days = (base_hours / self.config.working_hours_per_day).ceil() as i64;
        Duration::days(days)
    }
    
    fn find_critical_path(&self, task_schedules: &indexmap::IndexMap<Id, TaskScheduleInfo>) -> Vec<Id> {
        task_schedules
            .values()
            .filter(|ts| ts.is_critical)
            .map(|ts| ts.task_id)
            .collect()
    }
}

impl Default for ProjectScheduler {
    fn default() -> Self {
        Self::new(SchedulingConfig::default())
    }
}