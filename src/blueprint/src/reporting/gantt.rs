use crate::scheduling::Schedule;
use anyhow::Result;
use std::fmt::Write;
use std::fs;
use std::process::Command;

pub struct GanttGenerator;

impl GanttGenerator {
    pub fn generate_mermaid(schedule: &Schedule) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "gantt")?;
        writeln!(&mut output, "    title {}", schedule.project_name)?;
        writeln!(&mut output, "    dateFormat YYYY-MM-DD")?;
        writeln!(&mut output, "    axisFormat %m-%d")?;
        writeln!(&mut output)?;

        // Group tasks by resource
        let mut tasks_by_resource = std::collections::HashMap::new();
        for (task_id, task) in &schedule.tasks {
            tasks_by_resource
                .entry(&task.assigned_to)
                .or_insert_with(Vec::new)
                .push((task_id, task));
        }

        // Generate sections for each resource
        for (resource_id, tasks) in tasks_by_resource {
            if let Some(util) = schedule.resource_utilization.get(resource_id) {
                writeln!(&mut output, "    section {}", util.name)?;

                for (task_id, task) in tasks {
                    let status = if task.is_critical { "crit" } else { "active" };
                    let duration = (task.end_date - task.start_date).num_days() + 1;

                    writeln!(
                        &mut output,
                        "    {} ({:.0}h) :{}, {}, {}, {}d",
                        task.name,
                        task.effort,
                        status,
                        task_id,
                        task.start_date.format("%Y-%m-%d"),
                        duration
                    )?;
                }
                writeln!(&mut output)?;
            }
        }

        // Add milestones section
        if !schedule.milestones.is_empty() {
            writeln!(&mut output, "    section Milestones")?;
            for (_, milestone) in &schedule.milestones {
                let status = if milestone.is_critical { "crit" } else { "milestone" };
                writeln!(
                    &mut output,
                    "    {} :{}, {}, {}, 0d",
                    milestone.name,
                    status,
                    milestone.milestone_id,
                    milestone.date.format("%Y-%m-%d")
                )?;
            }
        }

        Ok(output)
    }

    pub fn generate_wbs_mermaid(schedule: &Schedule) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "gantt")?;
        writeln!(&mut output, "    title {} (Work Breakdown Structure)", schedule.project_name)?;
        writeln!(&mut output, "    dateFormat YYYY-MM-DD")?;
        writeln!(&mut output, "    axisFormat %m-%d")?;
        writeln!(&mut output)?;

        // Group tasks by base task name (remove resource suffixes) and combine multi-resource tasks
        let mut task_groups = std::collections::HashMap::new();
        for (task_id, task) in &schedule.tasks {
            let base_task_id = if task_id.contains('_') {
                task_id.split('_').next().unwrap_or(task_id)
            } else {
                task_id
            };
            
            task_groups
                .entry(base_task_id.to_string())
                .or_insert_with(Vec::new)
                .push((task_id, task));
        }

        // Convert to combined tasks and sort by dependency order
        let mut combined_tasks = Vec::new();
        for (base_task_id, task_parts) in task_groups {
            if task_parts.len() == 1 {
                let (_, task) = &task_parts[0];
                combined_tasks.push((
                    base_task_id,
                    task.name.clone(),
                    task.start_date,
                    task.end_date,
                    task.effort,
                    task.is_critical,
                ));
            } else {
                // Multi-resource task - combine into single entry
                let total_effort: f32 = task_parts.iter().map(|(_, task)| task.effort).sum();
                let earliest_start = task_parts.iter().map(|(_, task)| task.start_date).min().unwrap();
                let latest_end = task_parts.iter().map(|(_, task)| task.end_date).max().unwrap();
                let is_critical = task_parts.iter().any(|(_, task)| task.is_critical);
                let task_name = &task_parts[0].1.name;

                combined_tasks.push((
                    base_task_id,
                    task_name.clone(),
                    earliest_start,
                    latest_end,
                    total_effort,
                    is_critical,
                ));
            }
        }

        // Sort by start date for dependency-based order
        combined_tasks.sort_by(|a, b| a.2.cmp(&b.2));

        writeln!(&mut output, "    section Project Timeline")?;

        for (task_id, name, start_date, end_date, effort, is_critical) in combined_tasks {
            let status = if is_critical { "crit" } else { "active" };
            let duration = (end_date - start_date).num_days() + 1;

            writeln!(
                &mut output,
                "    {} ({:.0}h) :{}, {}, {}, {}d",
                name,
                effort,
                status,
                task_id,
                start_date.format("%Y-%m-%d"),
                duration
            )?;
        }

        // Add milestones
        if !schedule.milestones.is_empty() {
            writeln!(&mut output)?;
            writeln!(&mut output, "    section Milestones")?;
            for (_, milestone) in &schedule.milestones {
                let status = if milestone.is_critical { "crit" } else { "milestone" };
                writeln!(
                    &mut output,
                    "    {} :{}, {}, {}, 0d",
                    milestone.name,
                    status,
                    milestone.milestone_id,
                    milestone.date.format("%Y-%m-%d")
                )?;
            }
        }

        Ok(output)
    }

    pub fn generate_utilization_mermaid(schedule: &Schedule) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "gantt")?;
        writeln!(&mut output, "    title {} (Resource Utilization)", schedule.project_name)?;
        writeln!(&mut output, "    dateFormat YYYY-MM-DD")?;
        writeln!(&mut output, "    axisFormat %m-%d")?;
        writeln!(&mut output)?;

        // Create utilization timeline for each resource
        for (resource_id, utilization) in &schedule.resource_utilization {
            writeln!(&mut output, "    section {} ({:.0}% utilized)", utilization.name, utilization.utilization_percentage)?;
            
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
                
                writeln!(
                    &mut output,
                    "    {} ({:.0}h) :{}, {}_{}, {}, {}d",
                    assignment.task_name,
                    assignment.hours,
                    utilization_level,
                    assignment.task_id,
                    resource_id,
                    assignment.start_date.format("%Y-%m-%d"),
                    duration
                )?;
            }
            writeln!(&mut output)?;
        }

        Ok(output)
    }

    pub fn generate_image(schedule: &Schedule, output_path: &str, format: &str) -> Result<()> {
        // Generate Mermaid content
        let mermaid_content = Self::generate_mermaid(schedule)?;
        
        // Create temporary file for Mermaid content
        let temp_file = format!("/tmp/blueprint_gantt_{}.mmd", uuid::Uuid::new_v4());
        fs::write(&temp_file, &mermaid_content)?;

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
                    .output()?;

                // Clean up temp file
                let _ = fs::remove_file(&temp_file);

                if output.status.success() {
                    println!("✓ Gantt chart exported to {}", output_path);
                    Ok(())
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("Mermaid CLI error: {}", error);
                }
            }
            Err(_) => {
                // Clean up temp file
                let _ = fs::remove_file(&temp_file);
                
                // Fallback: save Mermaid content as text file
                let fallback_path = output_path.replace(&format!(".{}", format), ".mmd");
                fs::write(&fallback_path, &mermaid_content)?;
                
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

    pub fn generate_markdown(schedule: &Schedule) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "# Project Schedule: {}", schedule.project_name)?;
        writeln!(&mut output)?;
        writeln!(&mut output, "## Timeline")?;
        writeln!(&mut output, "- **Start Date**: {}", schedule.start_date)?;
        writeln!(&mut output, "- **End Date**: {}", schedule.end_date)?;
        writeln!(&mut output, "- **Duration**: {} days", (schedule.end_date - schedule.start_date).num_days())?;
        writeln!(&mut output)?;

        writeln!(&mut output, "## Gantt Chart")?;
        writeln!(&mut output)?;
        writeln!(&mut output, "```mermaid")?;
        write!(&mut output, "{}", Self::generate_mermaid(schedule)?)?;
        writeln!(&mut output, "```")?;
        writeln!(&mut output)?;

        writeln!(&mut output, "## Task Details")?;
        writeln!(&mut output)?;
        writeln!(&mut output, "| Task | Resource | Start | End | Duration | Critical |")?;
        writeln!(&mut output, "|------|----------|-------|-----|----------|----------|")?;

        for (_, task) in &schedule.tasks {
            let duration = (task.end_date - task.start_date).num_days() + 1;
            let critical = if task.is_critical { "Yes" } else { "No" };

            writeln!(
                &mut output,
                "| {} | {} | {} | {} | {} days | {} |",
                task.name,
                task.assigned_to,
                task.start_date.format("%Y-%m-%d"),
                task.end_date.format("%Y-%m-%d"),
                duration,
                critical
            )?;
        }

        Ok(output)
    }
}
