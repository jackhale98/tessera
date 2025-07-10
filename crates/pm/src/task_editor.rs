use crate::{Task, Resource, TaskStatus, TaskPriority, ProjectRepository, Project};
use tessera_core::{Id, Result, DesignTrackError};
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;
use inquire::{
    Confirm, CustomType, Select, Text, MultiSelect, DateSelect,
    validator::{Validation, CustomTypeValidator},
};

/// Task editor for comprehensive task management and duration editing
pub struct TaskEditor;

#[derive(Debug)]
pub struct TaskEditOptions {
    pub allow_status_change: bool,
    pub allow_date_change: bool,
    pub allow_resource_change: bool,
    pub allow_dependency_change: bool,
}

impl Default for TaskEditOptions {
    fn default() -> Self {
        Self {
            allow_status_change: true,
            allow_date_change: true,
            allow_resource_change: true,
            allow_dependency_change: true,
        }
    }
}

impl TaskEditor {
    /// Interactive task editing with comprehensive options
    pub fn edit_task_interactive(
        project: &mut Project,
        repository: &mut ProjectRepository,
        task_id: Id,
        options: TaskEditOptions,
    ) -> Result<()> {
        let task = repository.find_task_by_id(task_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task with ID {}", task_id)))?;

        Self::display_task_info(&task);

        loop {
            let mut choices = vec![
                "Edit Name",
                "Edit Description", 
                "Edit Priority",
                "Edit Progress",
                "Edit Effort Estimates",
            ];

            if options.allow_status_change {
                choices.push("Edit Status");
            }
            if options.allow_date_change {
                choices.push("Edit Dates");
            }
            if options.allow_resource_change {
                choices.push("Edit Resource Assignments");
            }
            if options.allow_dependency_change {
                choices.push("Edit Dependencies");
            }

            choices.extend_from_slice(&[
                "Recalculate Duration",
                "Validate Task",
                "Done Editing",
            ]);

            let choice = Select::new("What would you like to edit?", choices)
                .prompt()
                .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

            match choice {
                "Edit Name" => Self::edit_task_name(repository, task_id)?,
                "Edit Description" => Self::edit_task_description(repository, task_id)?,
                "Edit Priority" => Self::edit_task_priority(repository, task_id)?,
                "Edit Status" => Self::edit_task_status(repository, task_id)?,
                "Edit Progress" => Self::edit_task_progress(repository, task_id)?,
                "Edit Effort Estimates" => Self::edit_task_effort(repository, task_id)?,
                "Edit Dates" => Self::edit_task_dates(repository, task_id)?,
                "Edit Resource Assignments" => Self::edit_task_resources(project, repository, task_id)?,
                "Edit Dependencies" => Self::edit_task_dependencies(project, repository, task_id)?,
                "Recalculate Duration" => Self::recalculate_task_duration(project, repository, task_id)?,
                "Validate Task" => Self::validate_task(repository, task_id)?,
                "Done Editing" => break,
                _ => unreachable!(),
            }
        }

        println!("✓ Finished editing task");
        Ok(())
    }

    /// Display comprehensive task information
    fn display_task_info(task: &Task) {
        println!("\n{}", format!("=== Task: {} ===", task.name));
        println!("ID: {}", task.id);
        println!("Status: {:?}", task.status);
        println!("Priority: {:?}", task.priority);
        println!("Progress: {:.1}%", task.progress_percentage);
        
        if !task.description.is_empty() {
            println!("Description: {}", task.description);
        }

        println!("Effort - Estimated: {:.1}h, Actual: {:.1}h", 
            task.estimated_hours, task.actual_hours);

        if let Some(start) = task.start_date {
            println!("Start Date: {}", start.format("%Y-%m-%d"));
        }
        if let Some(due) = task.due_date {
            println!("Due Date: {}", due.format("%Y-%m-%d"));
        }
        if let Some(completed) = task.completion_date {
            println!("Completion Date: {}", completed.format("%Y-%m-%d"));
        }

        if !task.assigned_resources.is_empty() {
            println!("Assigned Resources: {} resource(s)", task.assigned_resources.len());
        }
        if !task.dependencies.is_empty() {
            println!("Dependencies: {} task(s)", task.dependencies.len());
        }

        println!("{}", "─".repeat(50));
    }

    /// Edit task name with validation
    fn edit_task_name(repository: &mut ProjectRepository, task_id: Id) -> Result<()> {
        let task = repository.find_task_by_id(task_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;
        let mut task = task.clone();

        let new_name = Text::new("Task name:")
            .with_default(&task.name)
            .with_validator(|input: &str| {
                if input.trim().is_empty() {
                    Ok(Validation::Invalid("Name cannot be empty".into()))
                } else if input.len() > 200 {
                    Ok(Validation::Invalid("Name too long (max 200 characters)".into()))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        task.name = new_name.trim().to_string();
        repository.update_task(task)?;
        println!("✓ Task name updated");
        Ok(())
    }

    /// Edit task description
    fn edit_task_description(repository: &mut ProjectRepository, task_id: Id) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        let new_description = Text::new("Task description:")
            .with_default(&task.description)
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        task.description = new_description;
        repository.update_task(task)?;
        println!("✓ Task description updated");
        Ok(())
    }

    /// Edit task priority
    fn edit_task_priority(repository: &mut ProjectRepository, task_id: Id) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        let priorities = vec![
            TaskPriority::Critical,
            TaskPriority::High,
            TaskPriority::Medium,
            TaskPriority::Low,
        ];

        let current_index = priorities.iter()
            .position(|p| *p == task.priority)
            .unwrap_or(2); // Default to Medium

        let new_priority = Select::new("Task priority:", priorities)
            .with_starting_cursor(current_index)
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        task.priority = new_priority;
        repository.update_task(task)?;
        println!("✓ Task priority updated");
        Ok(())
    }

    /// Edit task status with validation
    fn edit_task_status(repository: &mut ProjectRepository, task_id: Id) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        let statuses = vec![
            TaskStatus::NotStarted,
            TaskStatus::InProgress,
            TaskStatus::OnHold,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ];

        let current_index = statuses.iter()
            .position(|s| *s == task.status)
            .unwrap_or(0);

        let new_status = Select::new("Task status:", statuses)
            .with_starting_cursor(current_index)
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        // Handle status transitions
        match new_status {
            TaskStatus::InProgress => {
                if task.start_date.is_none() {
                    task.start_date = Some(Utc::now());
                    println!("ℹ Start date set to current time");
                }
                if task.progress_percentage == 0.0 {
                    task.progress_percentage = 1.0;
                    println!("ℹ Progress set to 1%");
                }
            }
            TaskStatus::Completed => {
                task.progress_percentage = 100.0;
                task.completion_date = Some(Utc::now());
                println!("ℹ Progress set to 100% and completion date set");
            }
            TaskStatus::NotStarted => {
                task.start_date = None;
                task.progress_percentage = 0.0;
                task.completion_date = None;
                println!("ℹ Reset start date, progress, and completion date");
            }
            _ => {}
        }

        task.status = new_status;
        repository.update_task(task)?;
        println!("✓ Task status updated");
        Ok(())
    }

    /// Edit task progress with validation
    fn edit_task_progress(repository: &mut ProjectRepository, task_id: Id) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        let new_progress = CustomType::<f32>::new("Progress percentage (0-100):")
            .with_default(task.progress_percentage)
            .with_validator(|input: &f32| {
                if *input >= 0.0 && *input <= 100.0 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Progress must be between 0 and 100".into()))
                }
            })
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        task.progress_percentage = new_progress;

        // Auto-update status based on progress
        if new_progress == 0.0 && task.status != TaskStatus::NotStarted {
            task.status = TaskStatus::NotStarted;
            println!("ℹ Status changed to Not Started");
        } else if new_progress > 0.0 && new_progress < 100.0 && task.status == TaskStatus::NotStarted {
            task.status = TaskStatus::InProgress;
            if task.start_date.is_none() {
                task.start_date = Some(Utc::now());
            }
            println!("ℹ Status changed to In Progress");
        } else if new_progress == 100.0 && task.status != TaskStatus::Completed {
            task.status = TaskStatus::Completed;
            task.completion_date = Some(Utc::now());
            println!("ℹ Status changed to Completed");
        }

        repository.update_task(task)?;
        println!("✓ Task progress updated");
        Ok(())
    }

    /// Edit task effort estimates with duration recalculation
    fn edit_task_effort(repository: &mut ProjectRepository, task_id: Id) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        println!("Current estimates - Estimated: {:.1}h, Actual: {:.1}h", 
            task.estimated_hours, task.actual_hours);

        let edit_choice = Select::new("What would you like to edit?", vec![
            "Estimated hours",
            "Actual hours", 
            "Both estimates",
        ])
        .prompt()
        .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        match edit_choice {
            "Estimated hours" => {
                let new_estimated = CustomType::<f32>::new("Estimated hours:")
                    .with_default(task.estimated_hours)
                    .with_validator(|input: &f32| {
                        if *input >= 0.0 {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid("Hours must be non-negative".into()))
                        }
                    })
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                task.estimated_hours = new_estimated;
            }
            "Actual hours" => {
                let new_actual = CustomType::<f32>::new("Actual hours:")
                    .with_default(task.actual_hours)
                    .with_validator(|input: &f32| {
                        if *input >= 0.0 {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid("Hours must be non-negative".into()))
                        }
                    })
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                task.actual_hours = new_actual;
            }
            "Both estimates" => {
                let new_estimated = CustomType::<f32>::new("Estimated hours:")
                    .with_default(task.estimated_hours)
                    .with_validator(|input: &f32| {
                        if *input >= 0.0 { Ok(Validation::Valid) } 
                        else { Ok(Validation::Invalid("Hours must be non-negative".into())) }
                    })
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                let new_actual = CustomType::<f32>::new("Actual hours:")
                    .with_default(task.actual_hours)
                    .with_validator(|input: &f32| {
                        if *input >= 0.0 { Ok(Validation::Valid) } 
                        else { Ok(Validation::Invalid("Hours must be non-negative".into())) }
                    })
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                task.estimated_hours = new_estimated;
                task.actual_hours = new_actual;
            }
            _ => unreachable!(),
        }

        repository.update_task(task)?;
        println!("✓ Task effort estimates updated");
        Ok(())
    }

    /// Edit task dates with validation
    fn edit_task_dates(repository: &mut ProjectRepository, task_id: Id) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        let date_choices = vec![
            "Set start date",
            "Set due date", 
            "Clear start date",
            "Clear due date",
            "Set both dates",
        ];

        let choice = Select::new("Date editing options:", date_choices)
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        match choice {
            "Set start date" => {
                let default_date = task.start_date
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| Utc::now().date_naive());

                let new_date = DateSelect::new("Start date:")
                    .with_default(default_date)
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                task.start_date = Some(new_date.and_hms_opt(9, 0, 0).unwrap().and_utc());
                println!("✓ Start date set to {}", new_date);
            }
            "Set due date" => {
                let default_date = task.due_date
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| Utc::now().date_naive());

                let new_date = DateSelect::new("Due date:")
                    .with_default(default_date)
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                task.due_date = Some(new_date.and_hms_opt(17, 0, 0).unwrap().and_utc());
                println!("✓ Due date set to {}", new_date);
            }
            "Clear start date" => {
                task.start_date = None;
                println!("✓ Start date cleared");
            }
            "Clear due date" => {
                task.due_date = None;
                println!("✓ Due date cleared");
            }
            "Set both dates" => {
                let start_default = task.start_date
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| Utc::now().date_naive());

                let start_date = DateSelect::new("Start date:")
                    .with_default(start_default)
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                let due_default = task.due_date
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| start_date);

                let due_date = DateSelect::new("Due date:")
                    .with_default(due_default)
                    .prompt()
                    .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                if due_date < start_date {
                    return Err(DesignTrackError::Validation(
                        "Due date cannot be before start date".to_string()
                    ));
                }

                task.start_date = Some(start_date.and_hms_opt(9, 0, 0).unwrap().and_utc());
                task.due_date = Some(due_date.and_hms_opt(17, 0, 0).unwrap().and_utc());
                println!("✓ Dates set: {} to {}", start_date, due_date);
            }
            _ => unreachable!(),
        }

        repository.update_task(task)?;
        Ok(())
    }

    /// Edit task resource assignments
    fn edit_task_resources(
        project: &Project,
        repository: &mut ProjectRepository,
        task_id: Id,
    ) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        if project.resources.is_empty() {
            println!("ℹ No resources available to assign");
            return Ok(());
        }

        // Display current assignments
        if !task.assigned_resources.is_empty() {
            println!("Current resource assignments:");
            for resource_id in &task.assigned_resources {
                if let Some(resource) = project.resources.get(resource_id) {
                    println!("  - {} ({})", resource.name, resource.role);
                }
            }
        }

        let choices = vec![
            "Add resource",
            "Remove resource",
            "Clear all resources",
            "Done",
        ];

        loop {
            let choice = Select::new("Resource assignment options:", choices.clone())
                .prompt()
                .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

            match choice {
                "Add resource" => {
                    let available_resources: Vec<_> = project.resources.iter()
                        .filter(|(id, _)| !task.assigned_resources.contains(id))
                        .map(|(id, resource)| format!("{} - {} ({})", id, resource.name, resource.role))
                        .collect();

                    if available_resources.is_empty() {
                        println!("ℹ All available resources are already assigned");
                        continue;
                    }

                    let selected = Select::new("Select resource to assign:", available_resources)
                        .prompt()
                        .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                    let resource_id = selected.split(" - ").next().unwrap().parse::<Id>()
                        .map_err(|e| DesignTrackError::Validation(format!("Invalid resource ID: {}", e)))?;

                    task.assigned_resources.push(resource_id);
                    println!("✓ Resource assigned");
                }
                "Remove resource" => {
                    if task.assigned_resources.is_empty() {
                        println!("ℹ No resources to remove");
                        continue;
                    }

                    let resource_list: Vec<_> = task.assigned_resources.iter()
                        .filter_map(|id| {
                            project.resources.get(id)
                                .map(|r| format!("{} - {} ({})", id, r.name, r.role))
                        })
                        .collect();

                    let selected = Select::new("Select resource to remove:", resource_list)
                        .prompt()
                        .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                    let resource_id = selected.split(" - ").next().unwrap().parse::<Id>()
                        .map_err(|e| DesignTrackError::Validation(format!("Invalid resource ID: {}", e)))?;

                    task.assigned_resources.retain(|&id| id != resource_id);
                    println!("✓ Resource removed");
                }
                "Clear all resources" => {
                    if Confirm::new("Remove all resource assignments?")
                        .with_default(false)
                        .prompt()
                        .map_err(|e| DesignTrackError::Ui(e.to_string()))?
                    {
                        task.assigned_resources.clear();
                        println!("✓ All resource assignments cleared");
                    }
                }
                "Done" => break,
                _ => unreachable!(),
            }
        }

        repository.update_task(task)?;
        Ok(())
    }

    /// Edit task dependencies
    fn edit_task_dependencies(
        project: &Project,
        repository: &mut ProjectRepository,
        task_id: Id,
    ) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        // Display current dependencies
        if !task.dependencies.is_empty() {
            println!("Current dependencies:");
            for dep_id in &task.dependencies {
                if let Some(dep_task) = project.tasks.get(dep_id) {
                    println!("  - {} ({})", dep_task.name, dep_id);
                }
            }
        }

        let choices = vec![
            "Add dependency",
            "Remove dependency",
            "Clear all dependencies",
            "Done",
        ];

        loop {
            let choice = Select::new("Dependency options:", choices.clone())
                .prompt()
                .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

            match choice {
                "Add dependency" => {
                    let available_tasks: Vec<_> = project.tasks.iter()
                        .filter(|(id, _)| **id != task_id && !task.dependencies.contains(id))
                        .map(|(id, task)| format!("{} - {}", id, task.name))
                        .collect();

                    if available_tasks.is_empty() {
                        println!("ℹ No available tasks to add as dependencies");
                        continue;
                    }

                    let selected = Select::new("Select task dependency:", available_tasks)
                        .prompt()
                        .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                    let dep_id = selected.split(" - ").next().unwrap().parse::<Id>()
                        .map_err(|e| DesignTrackError::Validation(format!("Invalid task ID: {}", e)))?;

                    // Check for circular dependency
                    if Self::would_create_circular_dependency(project, dep_id, task_id) {
                        println!("⚠ Cannot add dependency: would create circular dependency");
                        continue;
                    }

                    task.dependencies.push(dep_id);
                    println!("✓ Dependency added");
                }
                "Remove dependency" => {
                    if task.dependencies.is_empty() {
                        println!("ℹ No dependencies to remove");
                        continue;
                    }

                    let dep_list: Vec<_> = task.dependencies.iter()
                        .filter_map(|id| {
                            project.tasks.get(id)
                                .map(|t| format!("{} - {}", id, t.name))
                        })
                        .collect();

                    let selected = Select::new("Select dependency to remove:", dep_list)
                        .prompt()
                        .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                    let dep_id = selected.split(" - ").next().unwrap().parse::<Id>()
                        .map_err(|e| DesignTrackError::Validation(format!("Invalid task ID: {}", e)))?;

                    task.dependencies.retain(|&id| id != dep_id);
                    println!("✓ Dependency removed");
                }
                "Clear all dependencies" => {
                    if Confirm::new("Remove all dependencies?")
                        .with_default(false)
                        .prompt()
                        .map_err(|e| DesignTrackError::Ui(e.to_string()))?
                    {
                        task.dependencies.clear();
                        println!("✓ All dependencies cleared");
                    }
                }
                "Done" => break,
                _ => unreachable!(),
            }
        }

        repository.update_task(task)?;
        Ok(())
    }

    /// Recalculate task duration based on effort and resource assignments
    fn recalculate_task_duration(
        project: &Project,
        repository: &mut ProjectRepository,
        task_id: Id,
    ) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        if task.assigned_resources.is_empty() {
            println!("ℹ No resources assigned - cannot calculate duration");
            return Ok(());
        }

        if task.estimated_hours == 0.0 {
            println!("ℹ No estimated hours - cannot calculate duration");
            return Ok(());
        }

        // Calculate total daily capacity from assigned resources
        let total_daily_hours: f32 = task.assigned_resources.iter()
            .filter_map(|resource_id| project.resources.get(resource_id))
            .map(|resource| resource.daily_hours)
            .sum();

        if total_daily_hours == 0.0 {
            println!("ℹ No resource capacity available");
            return Ok(());
        }

        let calculated_days = (task.estimated_hours / total_daily_hours).ceil();

        println!("Duration calculation:");
        println!("  Estimated effort: {:.1} hours", task.estimated_hours);
        println!("  Total daily capacity: {:.1} hours", total_daily_hours);
        println!("  Calculated duration: {:.0} days", calculated_days);

        // If start date is set, calculate new due date
        if let Some(start_date) = task.start_date {
            let new_due_date = start_date + chrono::Duration::days(calculated_days as i64);
            
            if Confirm::new(&format!("Update due date to {}?", new_due_date.format("%Y-%m-%d")))
                .with_default(true)
                .prompt()
                .map_err(|e| DesignTrackError::Ui(e.to_string()))?
            {
                task.due_date = Some(new_due_date);
                repository.update_task(task)?;
                println!("✓ Duration recalculated and due date updated");
            }
        } else {
            println!("ℹ Set start date to automatically calculate due date");
        }

        Ok(())
    }

    /// Validate task for consistency and completeness
    fn validate_task(repository: &ProjectRepository, task_id: Id) -> Result<()> {
        let task = repository.find_task_by_id(task_id)?
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Required field validation
        if task.name.trim().is_empty() {
            issues.push("Task name is empty");
        }

        // Progress validation
        if task.progress_percentage < 0.0 || task.progress_percentage > 100.0 {
            issues.push("Progress percentage is out of range (0-100)");
        }

        // Effort validation
        if task.estimated_hours < 0.0 {
            issues.push("Estimated hours cannot be negative");
        }
        if task.actual_hours < 0.0 {
            issues.push("Actual hours cannot be negative");
        }

        // Date validation
        if let (Some(start), Some(due)) = (task.start_date, task.due_date) {
            if due < start {
                issues.push("Due date is before start date");
            }
        }

        // Status consistency validation
        match task.status {
            TaskStatus::NotStarted => {
                if task.progress_percentage > 0.0 {
                    warnings.push("Task marked as Not Started but has progress");
                }
                if task.start_date.is_some() {
                    warnings.push("Task marked as Not Started but has start date");
                }
            }
            TaskStatus::InProgress => {
                if task.progress_percentage == 0.0 {
                    warnings.push("Task in progress but no progress recorded");
                }
                if task.progress_percentage == 100.0 {
                    warnings.push("Task in progress but shows 100% complete");
                }
            }
            TaskStatus::Completed => {
                if task.progress_percentage < 100.0 {
                    warnings.push("Task marked completed but progress < 100%");
                }
                if task.completion_date.is_none() {
                    warnings.push("Task marked completed but no completion date");
                }
            }
            _ => {}
        }

        // Resource assignment warnings
        if task.assigned_resources.is_empty() {
            warnings.push("No resources assigned to task");
        }

        // Effort vs actual warnings
        if task.actual_hours > 0.0 && task.estimated_hours > 0.0 {
            let variance = (task.actual_hours - task.estimated_hours) / task.estimated_hours * 100.0;
            if variance > 20.0 {
                warnings.push(&format!("Actual effort exceeds estimate by {:.1}%", variance));
            }
        }

        // Display results
        println!("\n=== Task Validation Results ===");
        
        if issues.is_empty() && warnings.is_empty() {
            println!("✓ Task validation passed - no issues found");
        } else {
            if !issues.is_empty() {
                println!("❌ Issues found:");
                for issue in issues {
                    println!("  - {}", issue);
                }
            }

            if !warnings.is_empty() {
                println!("⚠ Warnings:");
                for warning in warnings {
                    println!("  - {}", warning);
                }
            }
        }

        Ok(())
    }

    /// Check if adding a dependency would create a circular dependency
    fn would_create_circular_dependency(
        project: &Project,
        from_task_id: Id,
        to_task_id: Id,
    ) -> bool {
        fn has_path_to(
            project: &Project,
            current: Id,
            target: Id,
            visited: &mut std::collections::HashSet<Id>,
        ) -> bool {
            if current == target {
                return true;
            }
            
            if visited.contains(&current) {
                return false;
            }
            
            visited.insert(current);
            
            if let Some(task) = project.tasks.get(&current) {
                for dep_id in &task.dependencies {
                    if has_path_to(project, *dep_id, target, visited) {
                        return true;
                    }
                }
            }
            
            false
        }

        let mut visited = std::collections::HashSet::new();
        has_path_to(project, to_task_id, from_task_id, &mut visited)
    }

    /// Bulk edit multiple tasks
    pub fn bulk_edit_tasks(
        project: &mut Project,
        repository: &mut ProjectRepository,
        task_ids: Vec<Id>,
    ) -> Result<()> {
        if task_ids.is_empty() {
            return Ok(());
        }

        println!("Bulk editing {} tasks", task_ids.len());

        let edit_options = vec![
            "Change Priority",
            "Change Status", 
            "Update Progress",
            "Add Resource",
            "Remove Resource",
            "Set Due Date",
            "Clear Due Date",
        ];

        let choice = Select::new("Select bulk edit operation:", edit_options)
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

        match choice {
            "Change Priority" => {
                let priority = Select::new("New priority:", vec![
                    TaskPriority::Critical,
                    TaskPriority::High,
                    TaskPriority::Medium,
                    TaskPriority::Low,
                ])
                .prompt()
                .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                for task_id in task_ids {
                    if let Ok(Some(mut task)) = repository.find_task_by_id(task_id) {
                        task.priority = priority;
                        let _ = repository.update_task(task);
                    }
                }
                println!("✓ Priority updated for {} tasks", task_ids.len());
            }
            "Change Status" => {
                let status = Select::new("New status:", vec![
                    TaskStatus::NotStarted,
                    TaskStatus::InProgress,
                    TaskStatus::OnHold,
                    TaskStatus::Completed,
                    TaskStatus::Cancelled,
                ])
                .prompt()
                .map_err(|e| DesignTrackError::Ui(e.to_string()))?;

                for task_id in task_ids {
                    if let Ok(Some(mut task)) = repository.find_task_by_id(task_id) {
                        task.status = status;
                        let _ = repository.update_task(task);
                    }
                }
                println!("✓ Status updated for {} tasks", task_ids.len());
            }
            // Add other bulk operations as needed...
            _ => {
                println!("Bulk operation not yet implemented");
            }
        }

        Ok(())
    }
}