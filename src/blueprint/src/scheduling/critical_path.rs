use crate::scheduling::{Schedule, ScheduledTask};
use crate::core::{Project, DependencyType};
use indexmap::IndexMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use chrono::NaiveDate;
use anyhow::Result;

#[derive(Debug, Clone)]
struct TaskTimes {
    early_start: NaiveDate,
    early_finish: NaiveDate,
    late_start: NaiveDate,
    late_finish: NaiveDate,
    total_float: i64,
    free_float: i64,
}

pub struct CriticalPathAnalyzer;

impl CriticalPathAnalyzer {
    pub fn analyze(schedule: &Schedule) -> Vec<String> {
        // Return the already calculated critical path
        schedule.critical_path.clone()
    }

    pub fn calculate_slack(tasks: &IndexMap<String, ScheduledTask>) -> IndexMap<String, i64> {
        let mut slack_map = IndexMap::new();
        
        // Extract slack values already calculated during scheduling
        for (task_id, task) in tasks {
            slack_map.insert(task_id.clone(), task.slack);
        }
        
        slack_map
    }
    
    /// Performs complete critical path analysis with forward and backward pass
    pub fn perform_critical_path_analysis(
        project: &Project,
        scheduled_tasks: &IndexMap<String, ScheduledTask>,
        graph: &DiGraph<String, ()>,
        node_map: &HashMap<String, NodeIndex>,
    ) -> Result<(Vec<String>, IndexMap<String, i64>)> {
        // Build task times map
        let mut task_times = HashMap::new();
        
        // Initialize early times with actual scheduled dates
        for (task_id, scheduled_task) in scheduled_tasks {
            task_times.insert(task_id.clone(), TaskTimes {
                early_start: scheduled_task.start_date,
                early_finish: scheduled_task.end_date,
                late_start: scheduled_task.start_date,
                late_finish: scheduled_task.end_date,
                total_float: 0,
                free_float: 0,
            });
        }
        
        // Find project end date
        let project_end = scheduled_tasks.values()
            .map(|t| t.end_date)
            .max()
            .unwrap_or(project.start_date);
        
        // Perform backward pass to calculate late times
        let sorted_nodes = toposort(graph, None)
            .map_err(|_| anyhow::anyhow!("Circular dependency detected"))?;
        
        // Initialize late times for tasks with no successors
        for (task_id, times) in task_times.iter_mut() {
            if let Some(&node_idx) = node_map.get(task_id) {
                let has_successors = graph.edges(node_idx).count() > 0;
                if !has_successors {
                    times.late_finish = times.early_finish;
                    times.late_start = times.early_start;
                }
            }
        }
        
        // Backward pass: calculate late start and late finish times
        for &node_idx in sorted_nodes.iter().rev() {
            if let Some(task_id) = node_map.iter()
                .find(|(_, &idx)| idx == node_idx)
                .map(|(id, _)| id)
            {
                if let Some(task) = project.tasks.get(task_id) {
                    let mut earliest_late_start = project_end;
                    
                    // Find the earliest late start among successors
                    for successor_edge in graph.edges(node_idx) {
                        let successor_node = successor_edge.target();
                        if let Some(successor_id) = node_map.iter()
                            .find(|(_, &idx)| idx == successor_node)
                            .map(|(id, _)| id)
                        {
                            if let Some(successor_task) = project.tasks.get(successor_id) {
                                if let Some(successor_times) = task_times.get(successor_id) {
                                    // Find the dependency relationship
                                    let dep = successor_task.dependencies.iter()
                                        .find(|d| d.task_id == *task_id);
                                    
                                    if let Some(dep) = dep {
                                        let constraint_date = Self::calculate_backward_constraint(
                                            successor_times.late_start,
                                            successor_times.late_finish,
                                            &dep.dependency_type,
                                            dep.lag_days,
                                            project,
                                        )?;
                                        earliest_late_start = earliest_late_start.min(constraint_date);
                                    }
                                }
                            }
                        }
                    }
                    
                    // Update late times for this task
                    if let Some(times) = task_times.get_mut(task_id) {
                        times.late_finish = earliest_late_start;
                        let duration = (times.early_finish - times.early_start).num_days();
                        times.late_start = if duration > 0 {
                            project.get_default_calendar()
                                .map(|cal| cal.subtract_working_days(times.late_finish, duration as f32))
                                .unwrap_or(times.late_finish - chrono::Duration::days(duration))
                        } else {
                            times.late_finish
                        };
                    }
                }
            }
        }
        
        // Calculate float values - first calculate total float
        let mut total_floats = HashMap::new();
        for (task_id, times) in &task_times {
            let total_float = (times.late_start - times.early_start).num_days();
            total_floats.insert(task_id.clone(), total_float);
        }
        
        // Update total float and calculate free float
        for (task_id, times) in task_times.iter_mut() {
            times.total_float = total_floats[task_id];
            
            // Calculate free float (minimum float of immediate successors)
            if let Some(&node_idx) = node_map.get(task_id) {
                let mut min_successor_float = times.total_float;
                for successor_edge in graph.edges(node_idx) {
                    let successor_node = successor_edge.target();
                    if let Some(successor_id) = node_map.iter()
                        .find(|(_, &idx)| idx == successor_node)
                        .map(|(id, _)| id)
                    {
                        if let Some(&successor_total_float) = total_floats.get(successor_id) {
                            min_successor_float = min_successor_float.min(successor_total_float);
                        }
                    }
                }
                times.free_float = min_successor_float;
            }
        }
        
        // Identify critical path (tasks with zero or minimal total float)
        let mut critical_path = Vec::new();
        let min_float = task_times.values()
            .map(|times| times.total_float)
            .min()
            .unwrap_or(0);
        
        // Find critical tasks (those with minimum float)
        for (task_id, times) in &task_times {
            if times.total_float == min_float {
                critical_path.push(task_id.clone());
            }
        }
        
        // Sort critical path in dependency order
        critical_path.sort_by(|a, b| {
            let a_times = &task_times[a];
            let b_times = &task_times[b];
            a_times.early_start.cmp(&b_times.early_start)
                .then_with(|| a_times.early_finish.cmp(&b_times.early_finish))
        });
        
        // Build slack map
        let mut slack_map = IndexMap::new();
        for (task_id, times) in task_times {
            slack_map.insert(task_id, times.total_float);
        }
        
        Ok((critical_path, slack_map))
    }
    
    fn calculate_backward_constraint(
        successor_late_start: NaiveDate,
        successor_late_finish: NaiveDate,
        dependency_type: &DependencyType,
        lag_days: f32,
        project: &Project,
    ) -> Result<NaiveDate> {
        let base_date = match dependency_type {
            DependencyType::FinishToStart => successor_late_start,
            DependencyType::StartToStart => successor_late_start,
            DependencyType::FinishToFinish => successor_late_finish,
            DependencyType::StartToFinish => successor_late_finish,
        };
        
        let calendar = project.get_default_calendar()
            .ok_or_else(|| anyhow::anyhow!("No default calendar available"))?;
        
        // Apply lag/lead time in reverse
        if lag_days != 0.0 {
            if lag_days > 0.0 {
                // Positive lag - subtract working days for backward pass
                Ok(calendar.subtract_working_days(base_date, lag_days))
            } else {
                // Negative lag (lead) - add working days for backward pass
                Ok(calendar.add_working_days(base_date, -lag_days))
            }
        } else {
            Ok(base_date)
        }
    }
}
