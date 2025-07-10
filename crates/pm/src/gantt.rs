use crate::{Task, Milestone, Resource};
use tessera_core::{Id, Result, DesignTrackError};
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub id: Id,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub effort_hours: f32,
    pub assigned_to: Vec<String>,
    pub is_critical: bool,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub struct ScheduledMilestone {
    pub id: Id,
    pub name: String,
    pub date: NaiveDate,
    pub is_critical: bool,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ProjectSchedule {
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub tasks: HashMap<Id, ScheduledTask>,
    pub milestones: HashMap<Id, ScheduledMilestone>,
    pub critical_path: Vec<Id>,
    pub total_cost: f32,
}

#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    pub resource_id: Id,
    pub name: String,
    pub total_hours: f32,
    pub utilization_percentage: f32,
    pub task_assignments: Vec<TaskAssignment>,
}

#[derive(Debug, Clone)]
pub struct TaskAssignment {
    pub task_id: Id,
    pub task_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub hours: f32,
    pub allocation_percentage: f32,
}

pub struct GanttGenerator;

impl GanttGenerator {
    pub fn generate_mermaid(schedule: &ProjectSchedule) -> String {
        let mut output = String::new();

        output.push_str("gantt\n");
        output.push_str(&format!("    title {}\n", schedule.project_name));
        output.push_str("    dateFormat YYYY-MM-DD\n");
        output.push_str("    axisFormat %m-%d\n");
        output.push_str("\n");

        // Group tasks by assigned resources
        let mut tasks_by_resource: HashMap<String, Vec<&ScheduledTask>> = HashMap::new();
        for task in schedule.tasks.values() {
            let resource_name = if task.assigned_to.is_empty() {
                "Unassigned".to_string()
            } else {
                task.assigned_to.join(", ")
            };
            
            tasks_by_resource
                .entry(resource_name)
                .or_insert_with(Vec::new)
                .push(task);
        }

        // Generate sections for each resource
        for (resource_name, tasks) in &tasks_by_resource {
            output.push_str(&format!("    section {}\n", resource_name));

            let mut sorted_tasks = tasks.clone();
            sorted_tasks.sort_by(|a, b| a.start_date.cmp(&b.start_date));

            for task in sorted_tasks {
                let status = if task.is_critical { 
                    "crit" 
                } else if task.progress >= 100.0 {
                    "done"
                } else if task.progress > 0.0 {
                    "active"
                } else {
                    "active"
                };
                
                let duration = (task.end_date - task.start_date).num_days() + 1;

                output.push_str(&format!(
                    "    {} ({:.0}h) :{}, {}, {}, {}d\n",
                    task.name,
                    task.effort_hours,
                    status,
                    task.id,
                    task.start_date.format("%Y-%m-%d"),
                    duration
                ));
            }
            output.push_str("\n");
        }

        // Add milestones section
        if !schedule.milestones.is_empty() {
            output.push_str("    section Milestones\n");
            
            let mut sorted_milestones: Vec<_> = schedule.milestones.values().collect();
            sorted_milestones.sort_by(|a, b| a.date.cmp(&b.date));
            
            for milestone in sorted_milestones {
                let status = if milestone.is_critical { "crit" } else { "milestone" };
                output.push_str(&format!(
                    "    {} :{}, {}, {}, 0d\n",
                    milestone.name,
                    status,
                    milestone.id,
                    milestone.date.format("%Y-%m-%d")
                ));
            }
        }

        output
    }

    pub fn generate_wbs_mermaid(schedule: &ProjectSchedule) -> String {
        let mut output = String::new();

        output.push_str("gantt\n");
        output.push_str(&format!("    title {} (Work Breakdown Structure)\n", schedule.project_name));
        output.push_str("    dateFormat YYYY-MM-DD\n");
        output.push_str("    axisFormat %m-%d\n");
        output.push_str("\n");

        output.push_str("    section Project Timeline\n");

        // Sort tasks by start date for dependency-based order
        let mut sorted_tasks: Vec<_> = schedule.tasks.values().collect();
        sorted_tasks.sort_by(|a, b| a.start_date.cmp(&b.start_date));

        for task in sorted_tasks {
            let status = if task.is_critical { 
                "crit" 
            } else if task.progress >= 100.0 {
                "done"
            } else if task.progress > 0.0 {
                "active"
            } else {
                "active"
            };
            
            let duration = (task.end_date - task.start_date).num_days() + 1;

            output.push_str(&format!(
                "    {} ({:.0}h) :{}, {}, {}, {}d\n",
                task.name,
                task.effort_hours,
                status,
                task.id,
                task.start_date.format("%Y-%m-%d"),
                duration
            ));
        }

        // Add milestones
        if !schedule.milestones.is_empty() {
            output.push_str("\n");
            output.push_str("    section Milestones\n");
            
            let mut sorted_milestones: Vec<_> = schedule.milestones.values().collect();
            sorted_milestones.sort_by(|a, b| a.date.cmp(&b.date));
            
            for milestone in sorted_milestones {
                let status = if milestone.is_critical { "crit" } else { "milestone" };
                output.push_str(&format!(
                    "    {} :{}, {}, {}, 0d\n",
                    milestone.name,
                    status,
                    milestone.id,
                    milestone.date.format("%Y-%m-%d")
                ));
            }
        }

        output
    }

    pub fn generate_utilization_mermaid(
        schedule: &ProjectSchedule, 
        resource_utilization: &HashMap<Id, ResourceUtilization>
    ) -> String {
        let mut output = String::new();

        output.push_str("gantt\n");
        output.push_str(&format!("    title {} (Resource Utilization)\n", schedule.project_name));
        output.push_str("    dateFormat YYYY-MM-DD\n");
        output.push_str("    axisFormat %m-%d\n");
        output.push_str("\n");

        // Create utilization timeline for each resource
        for utilization in resource_utilization.values() {
            output.push_str(&format!(
                "    section {} ({:.0}% utilized)\n", 
                utilization.name, 
                utilization.utilization_percentage
            ));
            
            // Sort assignments by start date
            let mut assignments = utilization.task_assignments.clone();
            assignments.sort_by(|a, b| a.start_date.cmp(&b.start_date));
            
            for assignment in assignments {
                let duration = (assignment.end_date - assignment.start_date).num_days() + 1;
                let utilization_level = if utilization.utilization_percentage > 90.0 {
                    "crit"  // Overutilized
                } else if utilization.utilization_percentage > 70.0 {
                    "active"  // Well utilized
                } else {
                    "done"  // Underutilized
                };
                
                output.push_str(&format!(
                    "    {} ({:.0}h) :{}, {}_{}, {}, {}d\n",
                    assignment.task_name,
                    assignment.hours,
                    utilization_level,
                    assignment.task_id,
                    utilization.resource_id,
                    assignment.start_date.format("%Y-%m-%d"),
                    duration
                ));
            }
            output.push_str("\n");
        }

        output
    }

    pub fn generate_markdown(schedule: &ProjectSchedule) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Project Schedule: {}\n", schedule.project_name));
        output.push_str("\n");
        output.push_str("## Timeline\n");
        output.push_str(&format!("- **Start Date**: {}\n", schedule.start_date));
        output.push_str(&format!("- **End Date**: {}\n", schedule.end_date));
        output.push_str(&format!("- **Duration**: {} days\n", (schedule.end_date - schedule.start_date).num_days()));
        output.push_str(&format!("- **Total Cost**: ${:.2}\n", schedule.total_cost));
        output.push_str("\n");

        output.push_str("## Gantt Chart\n");
        output.push_str("\n");
        output.push_str("```mermaid\n");
        output.push_str(&Self::generate_mermaid(schedule));
        output.push_str("```\n");
        output.push_str("\n");

        output.push_str("## Task Details\n");
        output.push_str("\n");
        output.push_str("| Task | Resources | Start | End | Duration | Progress | Critical |\n");
        output.push_str("|------|-----------|-------|-----|----------|----------|----------|\n");

        let mut sorted_tasks: Vec<_> = schedule.tasks.values().collect();
        sorted_tasks.sort_by(|a, b| a.start_date.cmp(&b.start_date));

        for task in sorted_tasks {
            let duration = (task.end_date - task.start_date).num_days() + 1;
            let critical = if task.is_critical { "Yes" } else { "No" };
            let resources = if task.assigned_to.is_empty() {
                "Unassigned".to_string()
            } else {
                task.assigned_to.join(", ")
            };

            output.push_str(&format!(
                "| {} | {} | {} | {} | {} days | {:.1}% | {} |\n",
                task.name,
                resources,
                task.start_date.format("%Y-%m-%d"),
                task.end_date.format("%Y-%m-%d"),
                duration,
                task.progress,
                critical
            ));
        }

        output.push_str("\n");

        if !schedule.milestones.is_empty() {
            output.push_str("## Milestones\n");
            output.push_str("\n");
            output.push_str("| Milestone | Date | Status | Critical |\n");
            output.push_str("|-----------|------|--------|----------|\n");

            let mut sorted_milestones: Vec<_> = schedule.milestones.values().collect();
            sorted_milestones.sort_by(|a, b| a.date.cmp(&b.date));

            for milestone in sorted_milestones {
                let critical = if milestone.is_critical { "Yes" } else { "No" };
                output.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    milestone.name,
                    milestone.date.format("%Y-%m-%d"),
                    milestone.status,
                    critical
                ));
            }
        }

        output
    }

    pub fn generate_image(schedule: &ProjectSchedule, output_path: &str, format: &str) -> Result<()> {
        // Generate Mermaid content
        let mermaid_content = Self::generate_mermaid(schedule);
        
        // Create temporary file for Mermaid content
        let temp_file = format!("/tmp/tessera_gantt_{}.mmd", uuid::Uuid::new_v4());
        fs::write(&temp_file, &mermaid_content)
            .map_err(|e| DesignTrackError::Io(e))?;

        // Check if Mermaid CLI is available
        let mmdc_check = Command::new("mmdc")
            .arg("--version")
            .output();

        match mmdc_check {
            Ok(_) => {
                // Use Mermaid CLI to generate image
                let output = Command::new("mmdc")
                    .arg("-i")
                    .arg(&temp_file)
                    .arg("-o")
                    .arg(output_path)
                    .arg("-f")
                    .arg(format)
                    .output()
                    .map_err(|e| DesignTrackError::Io(e))?;

                // Clean up temp file
                let _ = fs::remove_file(&temp_file);

                if output.status.success() {
                    println!("✓ Gantt chart exported to {}", output_path);
                    Ok(())
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    Err(DesignTrackError::Module(format!("Mermaid CLI error: {}", error)))
                }
            }
            Err(_) => {
                // Clean up temp file
                let _ = fs::remove_file(&temp_file);
                
                // Fallback: save Mermaid content as text file
                let fallback_path = output_path.replace(&format!(".{}", format), ".mmd");
                fs::write(&fallback_path, &mermaid_content)
                    .map_err(|e| DesignTrackError::Io(e))?;
                
                println!("ℹ Mermaid CLI not found. Install with: npm install -g @mermaid-js/mermaid-cli");
                println!("ℹ Saved Mermaid source to: {}", fallback_path);
                println!("ℹ You can render it online at: https://mermaid.live/");
                
                Ok(())
            }
        }
    }

    pub fn check_mermaid_cli() -> bool {
        Command::new("mmdc")
            .arg("--version")
            .output()
            .is_ok()
    }

    pub fn create_schedule_from_tasks_and_milestones(
        project_name: String,
        tasks: &[Task],
        milestones: &[Milestone],
        resources: &[Resource],
    ) -> ProjectSchedule {
        let mut scheduled_tasks = HashMap::new();
        let mut scheduled_milestones = HashMap::new();
        let mut start_date = None;
        let mut end_date = None;
        let mut total_cost = 0.0;

        // Convert tasks to scheduled tasks
        for task in tasks {
            if let (Some(start), Some(end)) = (task.start_date, task.due_date) {
                let scheduled_task = ScheduledTask {
                    id: task.id,
                    name: task.name.clone(),
                    start_date: start.date_naive(),
                    end_date: end.date_naive(),
                    effort_hours: task.estimated_hours as f32,
                    assigned_to: task.assigned_resources.iter()
                        .filter_map(|resource_id| {
                            resources.iter()
                                .find(|r| r.id == *resource_id)
                                .map(|r| r.name.clone())
                        })
                        .collect(),
                    is_critical: false, // Would need critical path analysis
                    progress: task.progress_percentage as f32,
                };

                let start_naive = start.date_naive();
                let end_naive = end.date_naive();

                if start_date.is_none() || start_naive < start_date.unwrap() {
                    start_date = Some(start_naive);
                }
                if end_date.is_none() || end_naive > end_date.unwrap() {
                    end_date = Some(end_naive);
                }

                // Estimate cost (basic calculation)
                let estimated_cost = scheduled_task.effort_hours * 100.0; // $100/hour default
                total_cost += estimated_cost;

                scheduled_tasks.insert(task.id, scheduled_task);
            }
        }

        // Convert milestones to scheduled milestones
        for milestone in milestones {
            let scheduled_milestone = ScheduledMilestone {
                id: milestone.id,
                name: milestone.name.clone(),
                date: milestone.target_date.date_naive(),
                is_critical: false, // Would need analysis
                status: format!("{:?}", milestone.status),
            };

            scheduled_milestones.insert(milestone.id, scheduled_milestone);
        }

        ProjectSchedule {
            project_name,
            start_date: start_date.unwrap_or_else(|| Utc::now().date_naive()),
            end_date: end_date.unwrap_or_else(|| Utc::now().date_naive()),
            tasks: scheduled_tasks,
            milestones: scheduled_milestones,
            critical_path: Vec::new(), // Would need critical path analysis
            total_cost,
        }
    }

    pub fn calculate_resource_utilization(
        schedule: &ProjectSchedule,
        resources: &[Resource],
    ) -> HashMap<Id, ResourceUtilization> {
        let mut utilization_map = HashMap::new();

        for resource in resources {
            let mut task_assignments = Vec::new();
            let mut total_hours = 0.0;

            // Find all tasks assigned to this resource
            for task in schedule.tasks.values() {
                if task.assigned_to.contains(&resource.name) {
                    let assignment = TaskAssignment {
                        task_id: task.id,
                        task_name: task.name.clone(),
                        start_date: task.start_date,
                        end_date: task.end_date,
                        hours: task.effort_hours,
                        allocation_percentage: 100.0 / task.assigned_to.len() as f32, // Simple split
                    };
                    
                    total_hours += assignment.hours;
                    task_assignments.push(assignment);
                }
            }

            // Calculate utilization percentage (simplified)
            let working_days = if let Some(start) = schedule.tasks.values().map(|t| t.start_date).min() {
                if let Some(end) = schedule.tasks.values().map(|t| t.end_date).max() {
                    let duration = (end - start).num_days() as f32;
                    let available_hours = duration * 8.0; // 8 hours per day
                    if available_hours > 0.0 {
                        (total_hours / available_hours) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let resource_utilization = ResourceUtilization {
                resource_id: resource.id,
                name: resource.name.clone(),
                total_hours,
                utilization_percentage: working_days.min(100.0),
                task_assignments,
            };

            utilization_map.insert(resource.id, resource_utilization);
        }

        utilization_map
    }
}