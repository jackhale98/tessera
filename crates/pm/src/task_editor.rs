use crate::{Task, Resource, Milestone, TaskStatus, TaskPriority, TaskType, DependencyType, WorkType, TaskDependency, ResourceAssignment, ProjectRepository};
use tessera_core::{Id, Result, DesignTrackError};
use chrono::{Utc, TimeZone};
use inquire::{
    Confirm, CustomType, Select, Text, DateSelect,
    validator::Validation,
};

/// Comprehensive editor for all PM entities with enhanced functionality
pub struct PMEntityEditor;

#[derive(Debug)]
pub struct EditOptions {
    pub allow_status_change: bool,
    pub allow_date_change: bool,
    pub allow_resource_change: bool,
    pub allow_dependency_change: bool,
}

impl Default for EditOptions {
    fn default() -> Self {
        Self {
            allow_status_change: true,
            allow_date_change: true,
            allow_resource_change: true,
            allow_dependency_change: true,
        }
    }
}

impl PMEntityEditor {
    /// Interactive task editing with comprehensive options including new task types
    pub fn edit_task_interactive(
        repository: &mut ProjectRepository,
        task_id: Id,
        options: EditOptions,
    ) -> Result<()> {
        let mut task = repository.find_task_by_id(task_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task with ID {}", task_id)))?
            .clone();

        Self::display_task_info(&task, repository);

        loop {
            let mut choices = vec![
                "Edit Name",
                "Edit Description",
                "Edit Task Type",
                "Edit Priority",
                "Edit Work Type",
                "Edit Effort/Duration/Work",
                "Edit Progress",
                "Edit Notes",
            ];

            if options.allow_status_change {
                choices.push("Edit Status");
            }
            if options.allow_date_change {
                choices.push("Edit Dates");
            }
            if options.allow_resource_change {
                choices.push("Manage Resources");
            }
            if options.allow_dependency_change {
                choices.push("Manage Dependencies");
            }

            choices.extend_from_slice(&["Save & Exit", "Exit without Saving"]);

            let choice = Select::new("What would you like to edit?", choices).prompt()?;

            match choice {
                "Edit Name" => {
                    let new_name = Text::new("Task name:")
                        .with_initial_value(&task.name)
                        .prompt()?;
                    task.name = new_name;
                    task.updated = Utc::now();
                }
                "Edit Description" => {
                    let new_description = Text::new("Description:")
                        .with_initial_value(&task.description)
                        .prompt()?;
                    task.description = new_description;
                    task.updated = Utc::now();
                }
                "Edit Task Type" => {
                    Self::edit_task_type(&mut task)?;
                }
                "Edit Priority" => {
                    Self::edit_priority(&mut task)?;
                }
                "Edit Work Type" => {
                    Self::edit_work_type(&mut task)?;
                }
                "Edit Effort/Duration/Work" => {
                    Self::edit_effort_duration_work(&mut task)?;
                }
                "Edit Progress" => {
                    Self::edit_progress(&mut task)?;
                }
                "Edit Notes" => {
                    let current_notes = task.notes.as_deref().unwrap_or("");
                    let new_notes = Text::new("Notes:")
                        .with_initial_value(current_notes)
                        .prompt()?;
                    task.notes = if new_notes.trim().is_empty() {
                        None
                    } else {
                        Some(new_notes)
                    };
                    task.updated = Utc::now();
                }
                "Edit Status" => {
                    Self::edit_status(&mut task)?;
                }
                "Edit Dates" => {
                    Self::edit_dates(&mut task)?;
                }
                "Manage Resources" => {
                    Self::manage_task_resources(&mut task, repository)?;
                }
                "Manage Dependencies" => {
                    Self::manage_task_dependencies(&mut task, repository)?;
                }
                "Save & Exit" => {
                    repository.update_task(task)?;
                    println!("✓ Task updated successfully!");
                    break;
                }
                "Exit without Saving" => {
                    let confirm = Confirm::new("Are you sure you want to exit without saving changes?")
                        .with_default(false)
                        .prompt()?;
                    if confirm {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Edit task type with proper validation and defaults
    fn edit_task_type(task: &mut Task) -> Result<()> {
        let task_types = vec![
            ("Effort Driven", TaskType::EffortDriven),
            ("Fixed Duration", TaskType::FixedDuration),
            ("Fixed Work", TaskType::FixedWork),
            ("Milestone", TaskType::Milestone),
        ];

        let current_index = task_types.iter()
            .position(|(_, t)| *t == task.task_type)
            .unwrap_or(0);

        let type_names: Vec<&str> = task_types.iter().map(|(name, _)| *name).collect();
        let selected = Select::new("Task type:", type_names)
            .with_starting_cursor(current_index)
            .with_help_message("Effort Driven: effort fixed, duration calculated | Fixed Duration: duration fixed, effort calculated | Fixed Work: work units fixed | Milestone: zero duration")
            .prompt()?;

        let selected_type = task_types.iter()
            .find(|(name, _)| *name == selected)
            .map(|(_, t)| *t)
            .unwrap_or(TaskType::EffortDriven);

        if selected_type != task.task_type {
            task.task_type = selected_type;
            
            // Adjust defaults based on new task type
            match selected_type {
                TaskType::EffortDriven => {
                    task.duration_days = None; // Will be calculated
                    if task.estimated_hours == 0.0 {
                        task.estimated_hours = 8.0;
                    }
                }
                TaskType::FixedDuration => {
                    if task.duration_days.is_none() {
                        task.duration_days = Some(1.0);
                    }
                }
                TaskType::FixedWork => {
                    if task.work_units.is_none() {
                        task.work_units = Some(task.estimated_hours);
                    }
                }
                TaskType::Milestone => {
                    task.estimated_hours = 0.0;
                    task.duration_days = Some(0.0);
                    task.work_units = None;
                }
            }
            task.updated = Utc::now();
            println!("✓ Task type changed to: {}", selected_type);
        }

        Ok(())
    }

    /// Edit effort, duration, and work units based on task type
    fn edit_effort_duration_work(task: &mut Task) -> Result<()> {
        match task.task_type {
            TaskType::EffortDriven => {
                let effort = CustomType::<f64>::new("Estimated effort (hours):")
                    .with_default(task.estimated_hours)
                    .with_help_message("Duration will be calculated based on resource assignments")
                    .prompt()?;
                task.estimated_hours = effort;
            }
            TaskType::FixedDuration => {
                let duration = CustomType::<f64>::new("Duration (working days):")
                    .with_default(task.duration_days.unwrap_or(1.0))
                    .with_help_message("Effort will be calculated based on resource assignments")
                    .prompt()?;
                task.duration_days = Some(duration);
            }
            TaskType::FixedWork => {
                let work_units = CustomType::<f64>::new("Work units:")
                    .with_default(task.work_units.unwrap_or(8.0))
                    .with_help_message("Both effort and duration will be calculated based on resource assignments")
                    .prompt()?;
                task.work_units = Some(work_units);
                task.estimated_hours = work_units; // Update estimated hours to match
            }
            TaskType::Milestone => {
                println!("Milestones have zero effort and duration");
                return Ok(());
            }
        }

        task.updated = Utc::now();
        println!("✓ Updated effort/duration/work parameters");
        Ok(())
    }

    /// Edit task priority
    fn edit_priority(task: &mut Task) -> Result<()> {
        let priorities = vec![
            ("Low", TaskPriority::Low),
            ("Medium", TaskPriority::Medium),
            ("High", TaskPriority::High),
            ("Critical", TaskPriority::Critical),
        ];

        let current_index = priorities.iter()
            .position(|(_, p)| *p == task.priority)
            .unwrap_or(1);

        let priority_names: Vec<&str> = priorities.iter().map(|(name, _)| *name).collect();
        let selected = Select::new("Priority:", priority_names)
            .with_starting_cursor(current_index)
            .prompt()?;

        let selected_priority = priorities.iter()
            .find(|(name, _)| *name == selected)
            .map(|(_, p)| *p)
            .unwrap_or(TaskPriority::Medium);

        task.priority = selected_priority;
        task.updated = Utc::now();
        Ok(())
    }

    /// Edit work type
    fn edit_work_type(task: &mut Task) -> Result<()> {
        let work_types = vec![
            "Design",
            "Analysis", 
            "Testing",
            "Documentation",
            "Review",
            "Manufacturing",
            "Other",
        ];

        let selected = Select::new("Work type:", work_types).prompt()?;
        
        let work_type = match selected {
            "Design" => WorkType::Design,
            "Analysis" => WorkType::Analysis,
            "Testing" => WorkType::Testing,
            "Documentation" => WorkType::Documentation,
            "Review" => WorkType::Review,
            "Manufacturing" => WorkType::Manufacturing,
            "Other" => {
                let other_type = Text::new("Specify work type:").prompt()?;
                WorkType::Other(other_type)
            }
            _ => WorkType::Design,
        };

        task.work_type = work_type;
        task.updated = Utc::now();
        Ok(())
    }

    /// Edit task progress
    fn edit_progress(task: &mut Task) -> Result<()> {
        let progress = CustomType::<f64>::new("Progress percentage (0-100):")
            .with_default(task.progress_percentage)
            .with_validator(|val: &f64| {
                if *val >= 0.0 && *val <= 100.0 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Progress must be between 0 and 100".into()))
                }
            })
            .prompt()?;

        task.update_progress(progress)?;
        println!("✓ Progress updated to {:.1}%", progress);
        Ok(())
    }

    /// Edit task status
    fn edit_status(task: &mut Task) -> Result<()> {
        let statuses = vec![
            ("Not Started", TaskStatus::NotStarted),
            ("In Progress", TaskStatus::InProgress),
            ("On Hold", TaskStatus::OnHold),
            ("Completed", TaskStatus::Completed),
            ("Cancelled", TaskStatus::Cancelled),
        ];

        let current_index = statuses.iter()
            .position(|(_, s)| *s == task.status)
            .unwrap_or(0);

        let status_names: Vec<&str> = statuses.iter().map(|(name, _)| *name).collect();
        let selected = Select::new("Status:", status_names)
            .with_starting_cursor(current_index)
            .prompt()?;

        let selected_status = statuses.iter()
            .find(|(name, _)| *name == selected)
            .map(|(_, s)| *s)
            .unwrap_or(TaskStatus::NotStarted);

        task.status = selected_status;
        
        // Auto-update related fields
        match selected_status {
            TaskStatus::InProgress => {
                if task.start_date.is_none() {
                    task.start_date = Some(Utc::now());
                }
            }
            TaskStatus::Completed => {
                task.completion_date = Some(Utc::now());
                task.progress_percentage = 100.0;
            }
            _ => {}
        }

        task.updated = Utc::now();
        Ok(())
    }

    /// Edit task dates
    fn edit_dates(task: &mut Task) -> Result<()> {
        let choices = vec![
            "Set Start Date",
            "Set Due Date",
            "Clear Start Date",
            "Clear Due Date",
            "Back",
        ];

        let choice = Select::new("Date options:", choices).prompt()?;

        match choice {
            "Set Start Date" => {
                let current_date = task.start_date
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| chrono::Local::now().date_naive());
                
                let naive_date = DateSelect::new("Start date:")
                    .with_help_message("Select the task start date")
                    .with_default(current_date)
                    .prompt()?;
                
                task.start_date = Some(Utc.from_utc_datetime(&naive_date.and_hms_opt(9, 0, 0).unwrap()));
            }
            "Set Due Date" => {
                let current_date = task.due_date
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| chrono::Local::now().date_naive());
                
                let naive_date = DateSelect::new("Due date:")
                    .with_help_message("Select the task due date")
                    .with_default(current_date)
                    .prompt()?;
                
                task.due_date = Some(Utc.from_utc_datetime(&naive_date.and_hms_opt(17, 0, 0).unwrap()));
            }
            "Clear Start Date" => {
                task.start_date = None;
            }
            "Clear Due Date" => {
                task.due_date = None;
            }
            "Back" => return Ok(()),
            _ => {}
        }

        task.updated = Utc::now();
        Ok(())
    }

    /// Manage task resource assignments with allocation percentages
    fn manage_task_resources(task: &mut Task, repository: &ProjectRepository) -> Result<()> {
        let resources = repository.get_resources();
        
        if resources.is_empty() {
            println!("No resources available. Add resources first.");
            return Ok(());
        }

        loop {
            println!("\nCurrent Resource Assignments:");
            if task.assigned_resources.is_empty() {
                println!("  No resources assigned");
            } else {
                for assignment in &task.assigned_resources {
                    if let Some(resource) = repository.find_resource_by_id(assignment.resource_id) {
                        println!("  {} - {:.1}% allocation", resource.name, assignment.allocation_percentage);
                        if let Some(role) = &assignment.role_in_task {
                            println!("    Role: {}", role);
                        }
                    }
                }
            }

            let choices = vec![
                "Add Resource",
                "Remove Resource", 
                "Edit Resource Allocation",
                "Back",
            ];

            let choice = Select::new("Resource management:", choices).prompt()?;

            match choice {
                "Add Resource" => {
                    Self::add_resource_assignment(task, resources)?;
                }
                "Remove Resource" => {
                    Self::remove_resource_assignment(task, repository)?;
                }
                "Edit Resource Allocation" => {
                    Self::edit_resource_allocation(task, repository)?;
                }
                "Back" => break,
                _ => {}
            }
        }

        Ok(())
    }

    fn add_resource_assignment(task: &mut Task, resources: &[Resource]) -> Result<()> {
        let available_resources: Vec<_> = resources.iter()
            .filter(|r| !task.assigned_resources.iter().any(|a| a.resource_id == r.id))
            .collect();

        if available_resources.is_empty() {
            println!("All resources are already assigned to this task");
            return Ok(());
        }

        let resource_options: Vec<String> = available_resources.iter()
            .map(|r| format!("{} - {}", r.name, r.role))
            .collect();

        let selected = Select::new("Select resource to assign:", resource_options.clone()).prompt()?;
        let selected_index = resource_options.iter().position(|x| x == &selected).unwrap();
        let selected_resource = available_resources[selected_index];

        let allocation = CustomType::<f64>::new("Allocation percentage (0-100):")
            .with_default(100.0)
            .with_validator(|val: &f64| {
                if *val > 0.0 && *val <= 100.0 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Allocation must be between 0 and 100".into()))
                }
            })
            .prompt()?;

        let role = Text::new("Role in this task (optional):")
            .prompt()?;

        let assignment = ResourceAssignment {
            resource_id: selected_resource.id,
            allocation_percentage: allocation,
            assigned_hours: None,
            rate_override: None,
            role_in_task: if role.trim().is_empty() { None } else { Some(role) },
        };

        task.assigned_resources.push(assignment);
        task.updated = Utc::now();
        println!("✓ Resource assigned: {} at {:.1}%", selected_resource.name, allocation);

        Ok(())
    }

    fn remove_resource_assignment(task: &mut Task, repository: &ProjectRepository) -> Result<()> {
        if task.assigned_resources.is_empty() {
            println!("No resources assigned to this task");
            return Ok(());
        }

        let resource_options: Vec<String> = task.assigned_resources.iter()
            .filter_map(|assignment| {
                repository.find_resource_by_id(assignment.resource_id)
                    .map(|r| format!("{} - {:.1}%", r.name, assignment.allocation_percentage))
            })
            .collect();

        let selected = Select::new("Select resource to remove:", resource_options.clone()).prompt()?;
        let selected_index = resource_options.iter().position(|x| x == &selected).unwrap();
        let assignment_to_remove = &task.assigned_resources[selected_index];
        let resource_id = assignment_to_remove.resource_id;

        task.assigned_resources.retain(|a| a.resource_id != resource_id);
        task.updated = Utc::now();
        println!("✓ Resource assignment removed");

        Ok(())
    }

    fn edit_resource_allocation(task: &mut Task, repository: &ProjectRepository) -> Result<()> {
        if task.assigned_resources.is_empty() {
            println!("No resources assigned to this task");
            return Ok(());
        }

        let resource_options: Vec<String> = task.assigned_resources.iter()
            .filter_map(|assignment| {
                repository.find_resource_by_id(assignment.resource_id)
                    .map(|r| format!("{} - {:.1}%", r.name, assignment.allocation_percentage))
            })
            .collect();

        let selected = Select::new("Select resource to edit:", resource_options.clone()).prompt()?;
        let selected_index = resource_options.iter().position(|x| x == &selected).unwrap();
        
        let current_allocation = task.assigned_resources[selected_index].allocation_percentage;
        let new_allocation = CustomType::<f64>::new("New allocation percentage (0-100):")
            .with_default(current_allocation)
            .with_validator(|val: &f64| {
                if *val > 0.0 && *val <= 100.0 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Allocation must be between 0 and 100".into()))
                }
            })
            .prompt()?;

        task.assigned_resources[selected_index].allocation_percentage = new_allocation;
        task.updated = Utc::now();
        println!("✓ Allocation updated to {:.1}%", new_allocation);

        Ok(())
    }

    /// Manage task dependencies with relationship types and lag/lead
    fn manage_task_dependencies(task: &mut Task, repository: &ProjectRepository) -> Result<()> {
        loop {
            println!("\nCurrent Dependencies:");
            if task.dependencies.is_empty() {
                println!("  No dependencies");
            } else {
                for dep in &task.dependencies {
                    if let Some(pred_task) = repository.find_task_by_id(dep.predecessor_id) {
                        let lag_text = if dep.lag_days != 0.0 {
                            format!(" ({} lag: {:.1} days)", 
                                if dep.lag_days > 0.0 { "+" } else { "" }, dep.lag_days)
                        } else {
                            String::new()
                        };
                        println!("  {} -> {} ({}){}", 
                            pred_task.name, task.name, dep.dependency_type, lag_text);
                    }
                }
            }

            let choices = vec![
                "Add Dependency",
                "Remove Dependency",
                "Edit Dependency",
                "Back",
            ];

            let choice = Select::new("Dependency management:", choices).prompt()?;

            match choice {
                "Add Dependency" => {
                    Self::add_task_dependency(task, repository)?;
                }
                "Remove Dependency" => {
                    Self::remove_task_dependency(task, repository)?;
                }
                "Edit Dependency" => {
                    Self::edit_task_dependency(task, repository)?;
                }
                "Back" => break,
                _ => {}
            }
        }

        Ok(())
    }

    fn add_task_dependency(task: &mut Task, repository: &ProjectRepository) -> Result<()> {
        let all_tasks = repository.get_tasks();
        let available_tasks: Vec<_> = all_tasks.iter()
            .filter(|t| t.id != task.id && !task.dependencies.iter().any(|d| d.predecessor_id == t.id))
            .collect();

        let all_milestones = repository.get_milestones();
        let available_milestones: Vec<_> = all_milestones.iter()
            .filter(|m| !task.dependencies.iter().any(|d| d.predecessor_id == m.id))
            .collect();

        if available_tasks.is_empty() && available_milestones.is_empty() {
            println!("No tasks or milestones available to add as dependencies");
            return Ok(());
        }

        // Create combined options list
        let mut all_options: Vec<(String, Id)> = Vec::new();
        
        // Add tasks
        for t in &available_tasks {
            all_options.push((format!("TASK: {} - {}", t.name, t.status), t.id));
        }
        
        // Add milestones  
        for m in &available_milestones {
            all_options.push((format!("MILESTONE: {} - {}", m.name, m.target_date.format("%Y-%m-%d")), m.id));
        }

        let option_labels: Vec<String> = all_options.iter().map(|(label, _)| label.clone()).collect();
        let selected = Select::new("Select predecessor (task or milestone):", option_labels.clone()).prompt()?;
        let selected_index = option_labels.iter().position(|x| x == &selected).unwrap();
        let predecessor_id = all_options[selected_index].1;

        // Select dependency type
        let dep_types = vec![
            ("Finish-to-Start", DependencyType::FinishToStart),
            ("Start-to-Start", DependencyType::StartToStart),
            ("Finish-to-Finish", DependencyType::FinishToFinish),
            ("Start-to-Finish", DependencyType::StartToFinish),
        ];

        let type_names: Vec<&str> = dep_types.iter().map(|(name, _)| *name).collect();
        let selected_type = Select::new("Dependency type:", type_names)
            .with_help_message("FS: predecessor finishes before this starts | SS: predecessor starts before this starts | FF: predecessor finishes before this finishes | SF: predecessor starts before this finishes")
            .prompt()?;

        let dependency_type = dep_types.iter()
            .find(|(name, _)| *name == selected_type)
            .map(|(_, t)| *t)
            .unwrap_or(DependencyType::FinishToStart);

        // Get lag/lead time
        let lag_days = CustomType::<f32>::new("Lag/Lead time (days):")
            .with_default(0.0)
            .with_help_message("Positive for lag (delay), negative for lead (overlap)")
            .prompt()?;

        let dependency = TaskDependency {
            predecessor_id,
            dependency_type,
            lag_days,
            description: None,
        };

        task.dependencies.push(dependency);
        task.updated = Utc::now();
        
        // Get name for confirmation message
        let predecessor_name = repository.find_task_by_id(predecessor_id)
            .map(|t| t.name.clone())
            .or_else(|| repository.find_milestone_by_id(predecessor_id)
                     .map(|m| m.name.clone()))
            .unwrap_or_else(|| "Unknown".to_string());
        
        println!("✓ Dependency added: {} -> {} ({})", 
            predecessor_name, task.name, dependency_type);

        Ok(())
    }

    fn remove_task_dependency(task: &mut Task, repository: &ProjectRepository) -> Result<()> {
        if task.dependencies.is_empty() {
            println!("No dependencies to remove");
            return Ok(());
        }

        let dep_options: Vec<String> = task.dependencies.iter()
            .filter_map(|dep| {
                repository.find_task_by_id(dep.predecessor_id)
                    .map(|t| format!("{} ({})", t.name, dep.dependency_type))
            })
            .collect();

        let selected = Select::new("Select dependency to remove:", dep_options.clone()).prompt()?;
        let selected_index = dep_options.iter().position(|x| x == &selected).unwrap();
        
        task.dependencies.remove(selected_index);
        task.updated = Utc::now();
        println!("✓ Dependency removed");

        Ok(())
    }

    fn edit_task_dependency(task: &mut Task, repository: &ProjectRepository) -> Result<()> {
        if task.dependencies.is_empty() {
            println!("No dependencies to edit");
            return Ok(());
        }

        let dep_options: Vec<String> = task.dependencies.iter()
            .filter_map(|dep| {
                repository.find_task_by_id(dep.predecessor_id)
                    .map(|t| format!("{} ({}, lag: {:.1})", t.name, dep.dependency_type, dep.lag_days))
            })
            .collect();

        let selected = Select::new("Select dependency to edit:", dep_options.clone()).prompt()?;
        let selected_index = dep_options.iter().position(|x| x == &selected).unwrap();

        let choices = vec![
            "Change Dependency Type",
            "Change Lag/Lead Time",
            "Back",
        ];

        let choice = Select::new("What to edit:", choices).prompt()?;

        match choice {
            "Change Dependency Type" => {
                let dep_types = vec![
                    ("Finish-to-Start", DependencyType::FinishToStart),
                    ("Start-to-Start", DependencyType::StartToStart),
                    ("Finish-to-Finish", DependencyType::FinishToFinish),
                    ("Start-to-Finish", DependencyType::StartToFinish),
                ];

                let current_type = task.dependencies[selected_index].dependency_type;
                let current_index = dep_types.iter()
                    .position(|(_, t)| *t == current_type)
                    .unwrap_or(0);

                let type_names: Vec<&str> = dep_types.iter().map(|(name, _)| *name).collect();
                let selected_type = Select::new("New dependency type:", type_names)
                    .with_starting_cursor(current_index)
                    .prompt()?;

                let new_type = dep_types.iter()
                    .find(|(name, _)| *name == selected_type)
                    .map(|(_, t)| *t)
                    .unwrap_or(DependencyType::FinishToStart);

                task.dependencies[selected_index].dependency_type = new_type;
                task.updated = Utc::now();
                println!("✓ Dependency type updated to: {}", new_type);
            }
            "Change Lag/Lead Time" => {
                let current_lag = task.dependencies[selected_index].lag_days;
                let new_lag = CustomType::<f32>::new("New lag/lead time (days):")
                    .with_default(current_lag)
                    .with_help_message("Positive for lag (delay), negative for lead (overlap)")
                    .prompt()?;

                task.dependencies[selected_index].lag_days = new_lag;
                task.updated = Utc::now();
                println!("✓ Lag/lead time updated to: {:.1} days", new_lag);
            }
            "Back" => {}
            _ => {}
        }

        Ok(())
    }

    /// Display comprehensive task information
    fn display_task_info(task: &Task, repository: &ProjectRepository) {
        println!("\n=== Task Information ===");
        println!("ID: {}", task.id);
        println!("Name: {}", task.name);
        println!("Description: {}", task.description);
        println!("Task Type: {}", task.task_type);
        println!("Status: {}", task.status);
        println!("Priority: {}", task.priority);
        println!("Work Type: {:?}", task.work_type);
        println!("Progress: {:.1}%", task.progress_percentage);

        // Show effort/duration/work based on task type
        match task.task_type {
            TaskType::EffortDriven => {
                println!("Estimated Effort: {:.1} hours", task.estimated_hours);
                if let Some(duration) = task.calculate_effective_duration() {
                    println!("Calculated Duration: {:.1} days", duration);
                }
            }
            TaskType::FixedDuration => {
                if let Some(duration) = task.duration_days {
                    println!("Fixed Duration: {:.1} days", duration);
                }
                println!("Calculated Effort: {:.1} hours", task.calculate_effective_effort());
            }
            TaskType::FixedWork => {
                if let Some(work_units) = task.work_units {
                    println!("Fixed Work Units: {:.1}", work_units);
                }
                if let Some(duration) = task.calculate_effective_duration() {
                    println!("Calculated Duration: {:.1} days", duration);
                }
            }
            TaskType::Milestone => {
                println!("Milestone: Zero effort and duration");
            }
        }

        if let Some(start) = task.start_date {
            println!("Start Date: {}", start.format("%Y-%m-%d"));
        }
        if let Some(due) = task.due_date {
            println!("Due Date: {}", due.format("%Y-%m-%d"));
        }

        if !task.assigned_resources.is_empty() {
            println!("Assigned Resources:");
            for assignment in &task.assigned_resources {
                if let Some(resource) = repository.find_resource_by_id(assignment.resource_id) {
                    println!("  {} - {:.1}% allocation", resource.name, assignment.allocation_percentage);
                }
            }
        }

        if !task.dependencies.is_empty() {
            println!("Dependencies:");
            for dep in &task.dependencies {
                if let Some(pred_task) = repository.find_task_by_id(dep.predecessor_id) {
                    println!("  {} ({}, lag: {:.1} days)", pred_task.name, dep.dependency_type, dep.lag_days);
                }
            }
        }

        if let Some(notes) = &task.notes {
            println!("Notes: {}", notes);
        }

        println!("Created: {}", task.created.format("%Y-%m-%d %H:%M"));
        println!("Updated: {}", task.updated.format("%Y-%m-%d %H:%M"));
        println!("========================\n");
    }

    /// Edit resource interactive
    pub fn edit_resource_interactive(
        repository: &mut ProjectRepository,
        resource_id: Id,
    ) -> Result<()> {
        let mut resource = repository.find_resource_by_id(resource_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Resource with ID {}", resource_id)))?
            .clone();

        println!("\n=== Resource Information ===");
        println!("ID: {}", resource.id);
        println!("Name: {}", resource.name);
        println!("Role: {}", resource.role);
        if let Some(email) = &resource.email {
            println!("Email: {}", email);
        }
        if let Some(rate) = resource.hourly_rate {
            println!("Hourly Rate: ${:.2}", rate);
        }
        println!("Daily Hours: {:.1}", resource.daily_hours);
        println!("Availability: {:.1}%", resource.availability_percentage);
        println!("Skills: {}", resource.skills.join(", "));
        println!("=============================\n");

        loop {
            let choices = vec![
                "Edit Name",
                "Edit Role",
                "Edit Email",
                "Edit Hourly Rate",
                "Edit Daily Hours",
                "Edit Availability",
                "Edit Skills",
                "Save & Exit",
                "Exit without Saving",
            ];

            let choice = Select::new("What would you like to edit?", choices).prompt()?;

            match choice {
                "Edit Name" => {
                    let new_name = Text::new("Resource name:")
                        .with_initial_value(&resource.name)
                        .prompt()?;
                    resource.name = new_name;
                    resource.updated = Utc::now();
                }
                "Edit Role" => {
                    let new_role = Text::new("Role:")
                        .with_initial_value(&resource.role)
                        .prompt()?;
                    resource.role = new_role;
                    resource.updated = Utc::now();
                }
                "Edit Email" => {
                    let current_email = resource.email.as_deref().unwrap_or("");
                    let new_email = Text::new("Email:")
                        .with_initial_value(current_email)
                        .prompt()?;
                    resource.email = if new_email.trim().is_empty() {
                        None
                    } else {
                        Some(new_email)
                    };
                    resource.updated = Utc::now();
                }
                "Edit Hourly Rate" => {
                    let current_rate = resource.hourly_rate.unwrap_or(0.0);
                    let new_rate = CustomType::<f64>::new("Hourly rate:")
                        .with_default(current_rate)
                        .prompt()?;
                    resource.hourly_rate = if new_rate > 0.0 { Some(new_rate) } else { None };
                    resource.updated = Utc::now();
                }
                "Edit Daily Hours" => {
                    let new_hours = CustomType::<f64>::new("Daily hours:")
                        .with_default(resource.daily_hours)
                        .prompt()?;
                    resource.daily_hours = new_hours;
                    resource.updated = Utc::now();
                }
                "Edit Availability" => {
                    let new_availability = CustomType::<f64>::new("Availability percentage (0-100):")
                        .with_default(resource.availability_percentage)
                        .with_validator(|val: &f64| {
                            if *val >= 0.0 && *val <= 100.0 {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid("Availability must be between 0 and 100".into()))
                            }
                        })
                        .prompt()?;
                    resource.availability_percentage = new_availability;
                    resource.updated = Utc::now();
                }
                "Edit Skills" => {
                    let current_skills = resource.skills.join(", ");
                    let new_skills_str = Text::new("Skills (comma-separated):")
                        .with_initial_value(&current_skills)
                        .prompt()?;
                    resource.skills = new_skills_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    resource.updated = Utc::now();
                }
                "Save & Exit" => {
                    repository.update_resource(resource)?;
                    println!("✓ Resource updated successfully!");
                    break;
                }
                "Exit without Saving" => {
                    let confirm = Confirm::new("Are you sure you want to exit without saving changes?")
                        .with_default(false)
                        .prompt()?;
                    if confirm {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Edit milestone interactive
    pub fn edit_milestone_interactive(
        repository: &mut ProjectRepository,
        milestone_id: Id,
    ) -> Result<()> {
        let mut milestone = repository.find_milestone_by_id(milestone_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Milestone with ID {}", milestone_id)))?
            .clone();

        println!("\n=== Milestone Information ===");
        println!("ID: {}", milestone.id);
        println!("Name: {}", milestone.name);
        println!("Description: {}", milestone.description);
        println!("Target Date: {}", milestone.target_date.format("%Y-%m-%d"));
        println!("Status: {:?}", milestone.status);
        println!("Dependent Tasks: {}", milestone.dependent_tasks.len());
        println!("==============================\n");

        loop {
            let choices = vec![
                "Edit Name",
                "Edit Description",
                "Edit Target Date",
                "Manage Dependent Tasks",
                "Save & Exit",
                "Exit without Saving",
            ];

            let choice = Select::new("What would you like to edit?", choices).prompt()?;

            match choice {
                "Edit Name" => {
                    let new_name = Text::new("Milestone name:")
                        .with_initial_value(&milestone.name)
                        .prompt()?;
                    milestone.name = new_name;
                    milestone.updated = Utc::now();
                }
                "Edit Description" => {
                    let new_description = Text::new("Description:")
                        .with_initial_value(&milestone.description)
                        .prompt()?;
                    milestone.description = new_description;
                    milestone.updated = Utc::now();
                }
                "Edit Target Date" => {
                    let current_date = milestone.target_date.date_naive();
                    
                    let naive_date = DateSelect::new("Target date:")
                        .with_help_message("Select the milestone target date")
                        .with_default(current_date)
                        .prompt()?;
                    
                    milestone.target_date = Utc.from_utc_datetime(&naive_date.and_hms_opt(12, 0, 0).unwrap());
                    milestone.updated = Utc::now();
                }
                "Manage Dependent Tasks" => {
                    Self::manage_milestone_dependencies(&mut milestone, repository)?;
                }
                "Save & Exit" => {
                    repository.update_milestone(milestone)?;
                    println!("✓ Milestone updated successfully!");
                    break;
                }
                "Exit without Saving" => {
                    let confirm = Confirm::new("Are you sure you want to exit without saving changes?")
                        .with_default(false)
                        .prompt()?;
                    if confirm {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn manage_milestone_dependencies(milestone: &mut Milestone, repository: &ProjectRepository) -> Result<()> {
        loop {
            println!("\nDependent Tasks:");
            if milestone.dependent_tasks.is_empty() {
                println!("  No dependent tasks");
            } else {
                for task_id in &milestone.dependent_tasks {
                    if let Some(task) = repository.find_task_by_id(*task_id) {
                        println!("  {} - {}", task.name, task.status);
                    }
                }
            }

            let choices = vec![
                "Add Dependent Task",
                "Remove Dependent Task",
                "Back",
            ];

            let choice = Select::new("Dependency management:", choices).prompt()?;

            match choice {
                "Add Dependent Task" => {
                    let all_tasks = repository.get_tasks();
                    let available_tasks: Vec<_> = all_tasks.iter()
                        .filter(|t| !milestone.dependent_tasks.contains(&t.id))
                        .collect();

                    if available_tasks.is_empty() {
                        println!("No tasks available to add as dependencies");
                        continue;
                    }

                    let task_options: Vec<String> = available_tasks.iter()
                        .map(|t| format!("{} - {}", t.name, t.status))
                        .collect();

                    let selected = Select::new("Select task:", task_options.clone()).prompt()?;
                    let selected_index = task_options.iter().position(|x| x == &selected).unwrap();
                    let selected_task = available_tasks[selected_index];

                    milestone.dependent_tasks.push(selected_task.id);
                    milestone.updated = Utc::now();
                    println!("✓ Task added as dependency: {}", selected_task.name);
                }
                "Remove Dependent Task" => {
                    if milestone.dependent_tasks.is_empty() {
                        println!("No dependent tasks to remove");
                        continue;
                    }

                    let task_options: Vec<String> = milestone.dependent_tasks.iter()
                        .filter_map(|task_id| {
                            repository.find_task_by_id(*task_id)
                                .map(|t| format!("{} - {}", t.name, t.status))
                        })
                        .collect();

                    let selected = Select::new("Select task to remove:", task_options.clone()).prompt()?;
                    let selected_index = task_options.iter().position(|x| x == &selected).unwrap();
                    
                    milestone.dependent_tasks.remove(selected_index);
                    milestone.updated = Utc::now();
                    println!("✓ Dependent task removed");
                }
                "Back" => break,
                _ => {}
            }
        }

        Ok(())
    }
}