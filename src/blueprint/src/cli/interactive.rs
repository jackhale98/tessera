use crate::{Project, Task, Resource, Milestone, SchedulingEngine, Schedule};
use crate::core::{TaskPriority, Calendar, Holiday, WorkingHours, ResourceCalendar, TaskType, TaskDependency, DependencyType, BaselineManager, BaselineType, BaselineInfo, RiskCategory, RiskProbability, RiskImpact, RiskStatus, RiskResponseType, ActionStatus, CommentType, BusinessImpact, IssuePriority, IssueSeverity, IssueStatus, IssueCategory, EscalationLevel, ProgressSnapshot, TaskProgress, TaskStatus, ProjectStatus, EarnedValueMetrics};
use anyhow::Result;
use inquire::{
    Confirm, CustomType, Select, Text, MultiSelect, DateSelect, InquireError,
    validator::{Validation, CustomTypeValidator},
};
use chrono::Weekday;
use std::path::PathBuf;
use colored::Colorize;

#[derive(Debug, Clone)]
struct PositiveFloatValidator;

/// Custom result type for menu navigation
#[derive(Debug)]
enum MenuResult<T> {
    Selection(T),
    GoBack,
    Exit,
}

/// Main menu function that handles Esc with exit confirmation
fn show_main_menu<T: Clone>(prompt: &str, choices: Vec<T>) -> Result<MenuResult<T>> 
where 
    T: std::fmt::Display,
{
    let select = Select::new(prompt, choices.clone());
    
    match select.prompt() {
        Ok(choice) => Ok(MenuResult::Selection(choice)),
        Err(InquireError::OperationInterrupted) => {
            // ESC was pressed - ask for exit confirmation in main menu
            if confirm_exit()? {
                Ok(MenuResult::Exit)
            } else {
                show_main_menu(prompt, choices) // Try again
            }
        },
        Err(InquireError::OperationCanceled) => {
            // Ctrl+C was pressed - ask for confirmation before exiting
            if confirm_exit()? {
                Ok(MenuResult::Exit)
            } else {
                show_main_menu(prompt, choices) // Try again
            }
        },
        Err(e) => Err(e.into()),
    }
}

/// Submenu function that handles Esc as "go back" without exit confirmation
fn show_submenu<T: Clone>(prompt: &str, choices: Vec<T>) -> Result<MenuResult<T>> 
where 
    T: std::fmt::Display,
{
    let select = Select::new(prompt, choices.clone());
    
    match select.prompt() {
        Ok(choice) => Ok(MenuResult::Selection(choice)),
        Err(InquireError::OperationInterrupted) => {
            // ESC was pressed - go back to parent menu
            Ok(MenuResult::GoBack)
        },
        Err(InquireError::OperationCanceled) => {
            // Also treat as go back - some terminals send this for ESC
            Ok(MenuResult::GoBack)
        },
        Err(e) => Err(e.into()),
    }
}

/// Confirms if the user really wants to exit
fn confirm_exit() -> Result<bool> {
    match Confirm::new("Are you sure you want to exit Blueprint?")
        .with_default(false)
        .prompt() 
    {
        Ok(confirmed) => Ok(confirmed),
        Err(InquireError::OperationInterrupted) => Ok(false), // ESC means "no, don't exit"
        Err(InquireError::OperationCanceled) => Ok(true),     // Ctrl+C means "yes, exit"
        Err(e) => Err(e.into()),
    }
}

/// Helper function for text input that handles Esc gracefully
fn prompt_text(prompt: &str) -> Result<Option<String>> {
    match Text::new(prompt).prompt() {
        Ok(text) => Ok(Some(text)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for confirmation prompts that handles Esc gracefully  
fn prompt_confirm(prompt: &str, default: bool) -> Result<Option<bool>> {
    match Confirm::new(prompt).with_default(default).prompt() {
        Ok(confirmed) => Ok(Some(confirmed)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for select prompts that handles Esc gracefully
fn prompt_select<T: Clone + std::fmt::Display>(prompt: &str, choices: Vec<T>) -> Result<Option<T>> {
    match Select::new(prompt, choices.clone()).prompt() {
        Ok(selection) => Ok(Some(selection)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

/// Helper function for custom type prompts that handles Esc gracefully
fn prompt_custom_type<T: Clone + std::str::FromStr + std::fmt::Display>(
    prompt: &str, 
    validator: impl CustomTypeValidator<T> + Clone + 'static
) -> Result<Option<T>> 
where 
    <T as std::str::FromStr>::Err: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    match CustomType::<T>::new(prompt).with_validator(validator.clone()).prompt() {
        Ok(value) => Ok(Some(value)),
        Err(InquireError::OperationInterrupted) => Ok(None), // ESC pressed - cancel operation
        Err(InquireError::OperationCanceled) => Ok(None), // Also treat as cancel - some terminals send this for ESC
        Err(e) => Err(e.into()),
    }
}

impl CustomTypeValidator<f32> for PositiveFloatValidator {
    fn validate(&self, input: &f32) -> Result<Validation, inquire::CustomUserError> {
        if *input > 0.0 {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Value must be positive".into()))
        }
    }
}


pub fn run(project_path: Option<PathBuf>) -> Result<()> {
    println!("{}", "Blueprint Interactive Mode".bold().cyan());
    println!("{}", "─".repeat(50));

    let mut project = if let Some(path) = project_path {
        Project::load_from_file(path)?
    } else if PathBuf::from("project.ron").exists() {
        Project::load_from_file("project.ron")?
    } else {
        create_new_project()?
    };

    // Auto-compute schedule on startup
    let mut current_schedule = compute_and_update_schedule(&mut project)?;
    
    if !project.tasks.is_empty() {
        println!("\n{}", "✓ Schedule computed automatically".green());
        println!("  Duration: {} → {}", 
            current_schedule.start_date.format("%Y-%m-%d"),
            current_schedule.end_date.format("%Y-%m-%d")
        );
        println!("  Total Cost: {} {:.2}", project.currency, current_schedule.total_cost);
    }

    loop {
        let choices = vec![
            "View Project Overview",
            "Manage Tasks",
            "Manage Resources",
            "Manage Milestones",
            "Manage Calendars",
            "Manage Baselines",
            "Track Progress",
            "Manage Issues & Risks",
            "Compute Schedule",
            "Schedule Analysis",
            "Generate Reports",
            "Save Project",
            "Exit",
        ];

        let choice_result = show_main_menu("\nWhat would you like to do?", choices)?;
        
        let choice = match choice_result {
            MenuResult::Selection(selection) => selection,
            MenuResult::GoBack => {
                // At main menu, go back means exit with confirmation
                if confirm_exit()? {
                    if Confirm::new("Save project before exiting?")
                        .with_default(true)
                        .prompt()
                        .unwrap_or(false) 
                    {
                        save_project(&project)?;
                    }
                    break;
                } else {
                    continue; // Stay in main menu
                }
            },
            MenuResult::Exit => {
                if Confirm::new("Save project before exiting?")
                    .with_default(true)
                    .prompt()
                    .unwrap_or(false) 
                {
                    save_project(&project)?;
                }
                break;
            }
        };

        match choice {
            "View Project Overview" => view_project_overview(&project, &current_schedule)?,
            "Manage Tasks" => {
                manage_tasks(&mut project)?;
                current_schedule = compute_and_update_schedule(&mut project)?;
            },
            "Manage Resources" => {
                manage_resources(&mut project)?;
                current_schedule = compute_and_update_schedule(&mut project)?;
            },
            "Manage Milestones" => {
                manage_milestones(&mut project)?;
                current_schedule = compute_and_update_schedule(&mut project)?;
            },
            "Manage Calendars" => {
                manage_calendars(&mut project)?;
                current_schedule = compute_and_update_schedule(&mut project)?;
            },
            "Manage Baselines" => manage_baselines(&mut project)?,
            "Track Progress" => manage_progress(&mut project)?,
            "Manage Issues & Risks" => manage_issues_risks(&mut project)?,
            "Compute Schedule" => {
                current_schedule = compute_and_update_schedule(&mut project)?;
                compute_schedule_interactive(&project)?;
            },
            "Schedule Analysis" => schedule_analysis_interactive(&project)?,
            "Generate Reports" => generate_reports_interactive(&project)?,
            "Save Project" => save_project(&project)?,
            "Exit" => {
                if confirm_exit()? {
                    if Confirm::new("Save project before exiting?")
                        .with_default(true)
                        .prompt()
                        .unwrap_or(false) 
                    {
                        save_project(&project)?;
                    }
                    break;
                }
                // If they don't want to exit, continue the loop
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// Computes the schedule and updates milestone dates in the project
fn compute_and_update_schedule(project: &mut Project) -> Result<Schedule> {
    if project.tasks.is_empty() {
        // Return empty schedule if no tasks
        return Ok(Schedule {
            project_name: project.name.clone(),
            start_date: project.start_date,
            end_date: project.start_date,
            tasks: Default::default(),
            milestones: Default::default(),
            critical_path: Vec::new(),
            total_cost: 0.0,
            resource_utilization: Default::default(),
        });
    }

    let engine = SchedulingEngine::new();
    let schedule = engine.compute_schedule(project)?;
    
    // Update milestone dates in the project with computed values
    for (milestone_id, scheduled_milestone) in &schedule.milestones {
        if let Some(project_milestone) = project.milestones.get_mut(milestone_id) {
            // Update the milestone's target date with the computed date
            project_milestone.target_date = Some(scheduled_milestone.date);
        }
    }
    
    Ok(schedule)
}

fn create_new_project() -> Result<Project> {
    println!("\n{}", "Creating New Project".bold());

    let name = Text::new("Project name:")
        .prompt()?;

    let start_date = DateSelect::new("Project start date:")
        .with_default(chrono::Local::now().date_naive())
        .prompt()?;

    let currency = Text::new("Currency:")
        .with_default("USD")
        .prompt()?;

    let mut project = Project::new(name, start_date);
    project.currency = currency;

    Ok(project)
}

fn view_project_overview(project: &Project, schedule: &Schedule) -> Result<()> {
    println!("\n{}", "Project Overview".bold());
    println!("{}", "─".repeat(50));
    println!("Name: {}", project.name.cyan());
    println!("Start Date: {}", project.start_date);
    println!("End Date: {}", schedule.end_date);
    println!("Duration: {} days", (schedule.end_date - schedule.start_date).num_days());
    println!("Currency: {}", project.currency);
    println!("Total Cost: {} {:.2}", project.currency, schedule.total_cost);
    println!("Resources: {}", project.resources.len());
    println!("Tasks: {}", project.tasks.len());
    println!("Milestones: {}", project.milestones.len());

    if !project.resources.is_empty() {
        println!("\n{}", "Resources:".bold());
        for (id, resource) in &project.resources {
            println!("  • {} - {} ({}/hr)",
                id.to_string().yellow(),
                resource.name,
                resource.hourly_rate
            );
        }
    }

    if !project.tasks.is_empty() {
        println!("\n{}", "Tasks:".bold());
        for (id, task) in &project.tasks {
            println!("  • {} - {} ({}h)",
                id.to_string().yellow(),
                task.name,
                task.total_effort_hours()
            );
        }
    }

    if !project.milestones.is_empty() {
        println!("\n{}", "Milestones:".bold());
        for (id, milestone) in &project.milestones {
            let target_str = if let Some(date) = milestone.target_date {
                format!(" (target: {})", date.format("%Y-%m-%d"))
            } else {
                " (no target date)".to_string()
            };
            
            // Show expected date from schedule if available
            let expected_str = if let Some(scheduled_milestone) = schedule.milestones.get(id) {
                let critical_marker = if scheduled_milestone.is_critical { " (CRITICAL)".red() } else { "".normal() };
                format!(" → expected: {}{}", scheduled_milestone.date.format("%Y-%m-%d"), critical_marker)
            } else {
                "".to_string()
            };
            
            println!("  • {} - {}{}{}",
                id.to_string().yellow(),
                milestone.name,
                target_str,
                expected_str
            );
        }
    }

    Ok(())
}

fn manage_tasks(project: &mut Project) -> Result<()> {
    loop {
        let choices = vec![
            "Add Task",
            "Edit Task",
            "Delete Task",
            "View Tasks",
            "Back",
        ];

        let choice_result = show_submenu("\nTask Management:", choices)?;
        
        let choice = match choice_result {
            MenuResult::Selection(selection) => selection,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Add Task" => add_task(project)?,
            "Edit Task" => edit_task(project)?,
            "Delete Task" => delete_task(project)?,
            "View Tasks" => view_tasks(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn add_task(project: &mut Project) -> Result<()> {
    println!("\n{}", "Adding New Task".bold());

    let id = match prompt_text("Task ID:")? {
        Some(id) => {
            if id.is_empty() {
                super::print_error("ID cannot be empty");
                return Ok(());
            }
            if id.contains(' ') {
                super::print_error("ID cannot contain spaces");
                return Ok(());
            }
            id
        },
        None => {
            println!("Task creation cancelled");
            return Ok(());
        }
    };

    if project.tasks.contains_key(&id) {
        super::print_error(&format!("Task '{}' already exists", id));
        return Ok(());
    }

    let name = match prompt_text("Task name:")? {
        Some(name) => name,
        None => {
            println!("Task creation cancelled");
            return Ok(());
        }
    };

    let mut task = Task::new(name);

    // Task type selection
    let task_types = vec![
        TaskType::EffortDriven,
        TaskType::FixedDuration,
        TaskType::FixedWork,
    ];

    task.task_type = match prompt_select("Task type:", task_types)? {
        Some(task_type) => task_type,
        None => {
            println!("Task creation cancelled");
            return Ok(());
        }
    };

    // Work units for FixedWork tasks
    if task.task_type == TaskType::FixedWork {
        let work_units = match prompt_custom_type("Work units:", PositiveFloatValidator)? {
            Some(work_units) => work_units,
            None => {
                println!("Task creation cancelled");
                return Ok(());
            }
        };
        task.work_units = Some(work_units);
    }

    // Skills required
    if let Some(true) = prompt_confirm("Does this task require specific skills?", false)? {
        let skills = match prompt_text("Skills (comma-separated):")? {
            Some(skills) => skills,
            None => {
                println!("Task creation cancelled");
                return Ok(());
            }
        };
        task.skills_required = skills.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Dependencies with fuzzy search
    if !project.tasks.is_empty() &&
       prompt_confirm("Does this task have dependencies?", false)? == Some(true) {
        
        loop {
            let task_list: Vec<String> = project.tasks.keys()
                .filter(|k| !task.dependencies.iter().any(|dep| dep.task_id == **k))
                .map(|k| format!("{} - {}", k, project.tasks[k].name))
                .collect();

            if task_list.is_empty() {
                super::print_info("All tasks are already dependencies");
                break;
            }

            let selected = match prompt_select("Select dependency:", task_list)? {
                Some(selected) => selected,
                None => {
                    println!("Dependency selection cancelled");
                    break;
                }
            };

            let dependency_task_id = selected.split(" - ").next().unwrap().to_string();
            
            // Select dependency type
            let dependency_types = vec![
                DependencyType::FinishToStart,
                DependencyType::StartToStart,
                DependencyType::FinishToFinish,
                DependencyType::StartToFinish,
            ];

            let dependency_type = match prompt_select("Dependency type:", dependency_types)? {
                Some(dependency_type) => dependency_type,
                None => {
                    println!("Dependency creation cancelled");
                    break;
                }
            };

            // Ask for lag/lead time
            let lag_days = if prompt_confirm("Add lag or lead time?", false)? == Some(true) {
                match prompt_custom_type("Lag days (positive for lag, negative for lead):", |input: &f32| {
                    Ok(Validation::Valid)
                })? {
                    Some(lag_days) => lag_days,
                    None => {
                        println!("Dependency creation cancelled");
                        break;
                    }
                }
            } else {
                0.0
            };

            let dependency = TaskDependency::new(dependency_task_id)
                .with_type(dependency_type)
                .with_lag(lag_days);

            task.dependencies.push(dependency);

            if prompt_confirm("Add another dependency?", false)? != Some(true) {
                break;
            }
        }
    }

    // Priority
    let priorities = vec![
        TaskPriority::Critical,
        TaskPriority::High,
        TaskPriority::Medium,
        TaskPriority::Low,
    ];

    task.priority = match prompt_select("Priority:", priorities)? {
        Some(priority) => priority,
        None => {
            println!("Task creation cancelled");
            return Ok(());
        }
    };

    // Resource-based estimation (now the only option)
    if project.resources.is_empty() {
        super::print_error("No resources defined. Please add resources first.");
        return Ok(());
    }

    // Duration is required for resource-based estimation
    let duration = match prompt_custom_type("Task duration (days):", |input: &i32| {
        if *input > 0 {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Duration must be positive".into()))
        }
    })? {
        Some(duration) => duration,
        None => {
            println!("Task creation cancelled");
            return Ok(());
        }
    };
    task.duration_days = Some(duration);

    // Resource assignments
    loop {
        let resource_list: Vec<String> = project.resources.iter()
            .filter(|(id, _)| !task.resource_assignments.contains_key(*id))
            .map(|(id, r)| format!("{} - {} ({}/hr)", id, r.name, r.hourly_rate))
            .collect();

        if resource_list.is_empty() {
            super::print_info("All resources are already assigned to this task");
            break;
        }

        let selected = match prompt_select("Select resource to assign:", resource_list)? {
            Some(selected) => selected,
            None => {
                println!("Resource assignment cancelled");
                break;
            }
        };

        let resource_id = selected.split(" - ").next().unwrap().to_string();

        // Ask for allocation type
        let allocation_choices = vec!["Hours", "Percentage", "Full Time"];
        let allocation_type = match prompt_select("Allocation type:", allocation_choices)? {
            Some(allocation_type) => allocation_type,
            None => {
                println!("Resource assignment cancelled");
                break;
            }
        };

        match allocation_type {
            "Hours" => {
                let hours = match prompt_custom_type("Hours for this resource:", PositiveFloatValidator)? {
                    Some(hours) => hours,
                    None => {
                        println!("Resource assignment cancelled");
                        break;
                    }
                };
                task.add_resource_hours(resource_id, hours);
            }
            "Percentage" => {
                let percentage = match prompt_custom_type("Percentage of resource capacity (0-100):", |input: &f32| {
                    if *input >= 0.0 && *input <= 100.0 {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid("Must be between 0 and 100".into()))
                    }
                })? {
                    Some(percentage) => percentage,
                    None => {
                        println!("Resource assignment cancelled");
                        break;
                    }
                };
                task.add_resource_percentage(resource_id, percentage);
            }
            "Full Time" => {
                task.add_resource_full_time(resource_id);
            }
            _ => unreachable!(),
        }

        if prompt_confirm("Add another resource assignment?", false)? != Some(true) {
            break;
        }
    }

    if task.resource_assignments.is_empty() {
        super::print_error("At least one resource assignment is required");
        return Ok(());
    }

    project.add_task(id.clone(), task);
    super::print_success(&format!("Added task '{}'", id));

    Ok(())
}

fn edit_task(project: &mut Project) -> Result<()> {
    if project.tasks.is_empty() {
        super::print_info("No tasks to edit");
        return Ok(());
    }

    let task_list: Vec<String> = project.tasks.keys()
        .map(|k| format!("{} - {}", k, project.tasks[k].name))
        .collect();

    let selected = match prompt_select("Select task to edit:", task_list)? {
        Some(selected) => selected,
        None => {
            println!("Task editing cancelled");
            return Ok(());
        }
    };

    let task_id = selected.split(" - ").next().unwrap();

    // Show current task info
    if let Some(task) = project.tasks.get(task_id) {
        println!("\n{}", format!("Editing Task: {}", task.name).bold());
        println!("{}", "─".repeat(50));
        println!("Name: {}", task.name);
        println!("Priority: {:?}", task.priority);
        println!("Completion: {:.0}%", task.completion * 100.0);
        if let Some(duration) = task.duration_days {
            println!("Duration: {} days", duration);
        }
        if !task.dependencies.is_empty() {
            println!("Dependencies: {}", task.dependencies.iter().map(|d| d.task_id.clone()).collect::<Vec<_>>().join(", "));
        }
        if !task.resource_assignments.is_empty() {
            println!("Resource assignments: {}", task.resource_assignments.len());
        }
        println!();
    }

    // Selective editing menu
    loop {
        let edit_choices = vec![
            "Edit Name",
            "Edit Priority",
            "Edit Task Type",
            "Edit Duration",
            "Edit Work Units",
            "Edit Dependencies",
            "Edit Resource Assignments",
            "Edit Completion",
            "Edit Fixed Dates",
            "Done Editing",
        ];

        let choice = match prompt_select("What would you like to edit?", edit_choices)? {
            Some(choice) => choice,
            None => break, // ESC pressed - exit editing
        };

        match choice {
            "Edit Name" => edit_task_name(project, task_id)?,
            "Edit Priority" => edit_task_priority(project, task_id)?,
            "Edit Task Type" => edit_task_type(project, task_id)?,
            "Edit Duration" => edit_task_duration(project, task_id)?,
            "Edit Work Units" => edit_task_work_units(project, task_id)?,
            "Edit Dependencies" => edit_task_dependencies(project, task_id)?,
            "Edit Resource Assignments" => edit_task_resources(project, task_id)?,
            "Edit Completion" => edit_task_completion(project, task_id)?,
            "Edit Fixed Dates" => edit_task_fixed_dates(project, task_id)?,
            "Done Editing" => break,
            _ => unreachable!(),
        }
    }

    super::print_success(&format!("Finished editing task '{}'", task_id));
    Ok(())
}

fn edit_task_name(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        let new_name = match prompt_text("Task name:")? {
            Some(name) => name,
            None => {
                println!("Task name editing cancelled");
                return Ok(());
            }
        };
        task.name = new_name;
        super::print_success("Task name updated");
    }
    Ok(())
}

fn edit_task_priority(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        let priorities = vec![
            TaskPriority::Critical,
            TaskPriority::High,
            TaskPriority::Medium,
            TaskPriority::Low,
        ];

        let current_index = priorities.iter().position(|p| *p == task.priority).unwrap_or(2);
        
        let new_priority = match prompt_select("Priority:", priorities)? {
            Some(priority) => priority,
            None => {
                println!("Task priority editing cancelled");
                return Ok(());
            }
        };
        task.priority = new_priority;
        super::print_success("Task priority updated");
    }
    Ok(())
}

fn edit_task_type(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        let task_types = vec![
            TaskType::EffortDriven,
            TaskType::FixedDuration,
            TaskType::FixedWork,
        ];

        let current_index = task_types.iter().position(|t| *t == task.task_type).unwrap_or(0);
        
        let new_task_type = match prompt_select("Task type:", task_types)? {
            Some(task_type) => task_type,
            None => {
                println!("Task type editing cancelled");
                return Ok(());
            }
        };
        
        task.task_type = new_task_type;
        
        // If changed to FixedWork and no work units set, prompt for them
        if task.task_type == TaskType::FixedWork && task.work_units.is_none() {
            let work_units = CustomType::<f32>::new("Work units:")
                .with_validator(PositiveFloatValidator)
                .prompt()?;
            task.work_units = Some(work_units);
        }
        
        super::print_success("Task type updated");
    }
    Ok(())
}

fn edit_task_duration(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        let current_duration = task.duration_days.unwrap_or(1);
        let new_duration = match prompt_custom_type("Task duration (days):", |input: &i32| {
            if *input > 0 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Duration must be positive".into()))
            }
        })? {
            Some(duration) => duration,
            None => {
                println!("Task duration editing cancelled");
                return Ok(());
            }
        };
        task.duration_days = Some(new_duration);
        super::print_success("Task duration updated");
    }
    Ok(())
}

fn edit_task_work_units(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        if task.task_type != TaskType::FixedWork {
            super::print_error("Work units can only be set for Fixed Work tasks");
            return Ok(());
        }
        
        let current_work_units = task.work_units.unwrap_or(1.0);
        let new_work_units = CustomType::<f32>::new("Work units:")
            .with_default(current_work_units)
            .with_validator(PositiveFloatValidator)
            .prompt()?;
        task.work_units = Some(new_work_units);
        super::print_success("Work units updated");
    }
    Ok(())
}

fn edit_task_dependencies(project: &mut Project, task_id: &str) -> Result<()> {
    loop {
        // Show current dependencies
        if let Some(task) = project.tasks.get(task_id) {
            if !task.dependencies.is_empty() {
                super::print_info("Current dependencies:");
                for dep in &task.dependencies {
                    let dep_type_str = match dep.dependency_type {
                        DependencyType::FinishToStart => "FS",
                        DependencyType::StartToStart => "SS", 
                        DependencyType::FinishToFinish => "FF",
                        DependencyType::StartToFinish => "SF",
                    };
                    let lag_str = if dep.lag_days != 0.0 {
                        format!(" ({:+} days)", dep.lag_days)
                    } else {
                        String::new()
                    };
                    println!("    - {} ({}){}", dep.task_id, dep_type_str, lag_str);
                }
            }
        }

        let choices = vec![
            "Add dependency",
            "Remove dependency",
            "Modify existing dependency",
            "Clear all dependencies",
            "Done",
        ];

        let choice = Select::new("Dependency editing options:", choices)
            .prompt()?;

        match choice {
            "Add dependency" => add_task_dependency(project, task_id)?,
            "Remove dependency" => remove_task_dependency(project, task_id)?,
            "Modify existing dependency" => modify_task_dependency(project, task_id)?,
            "Clear all dependencies" => {
                if Confirm::new("Remove all dependencies?").prompt()? {
                    if let Some(task) = project.tasks.get_mut(task_id) {
                        task.dependencies.clear();
                        super::print_success("Cleared all dependencies");
                    }
                }
            }
            "Done" => break,
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn add_task_dependency(project: &mut Project, task_id: &str) -> Result<()> {
    // First collect the available tasks without borrowing mutably
    let current_dependencies: Vec<String> = if let Some(task) = project.tasks.get(task_id) {
        task.dependencies.iter().map(|dep| dep.task_id.clone()).collect()
    } else {
        return Ok(());
    };

    let available_tasks: Vec<String> = project.tasks.keys()
        .filter(|&k| k != task_id && !current_dependencies.contains(k))
        .map(|k| format!("{} - {}", k, project.tasks[k].name))
        .collect();

    if available_tasks.is_empty() {
        super::print_info("No available tasks for dependencies");
        return Ok(());
    }

    let selected = match prompt_select("Select task to add as dependency:", available_tasks)? {
        Some(selected) => selected,
        None => {
            println!("Dependency creation cancelled");
            return Ok(());
        }
    };

    let dependency_task_id = selected.split(" - ").next().unwrap().to_string();
    
    // Select dependency type
    let dependency_types = vec![
        DependencyType::FinishToStart,
        DependencyType::StartToStart,
        DependencyType::FinishToFinish,
        DependencyType::StartToFinish,
    ];

    let dependency_type = match prompt_select("Dependency type:", dependency_types)? {
        Some(dependency_type) => dependency_type,
        None => {
            println!("Dependency creation cancelled");
            return Ok(());
        }
    };

    // Ask for lag/lead time
    let lag_days = if let Some(true) = prompt_confirm("Add lag or lead time?", false)? {
        match prompt_custom_type("Lag days (positive for lag, negative for lead):", |input: &f32| {
            Ok(Validation::Valid)
        })? {
            Some(lag_days) => lag_days,
            None => {
                println!("Dependency creation cancelled");
                return Ok(());
            }
        }
    } else {
        0.0
    };

    let dependency = TaskDependency::new(dependency_task_id)
        .with_type(dependency_type)
        .with_lag(lag_days);

    // Now borrow mutably to add the dependency
    if let Some(task) = project.tasks.get_mut(task_id) {
        task.dependencies.push(dependency);
        super::print_success("Dependency added");
    }
    Ok(())
}

fn remove_task_dependency(project: &mut Project, task_id: &str) -> Result<()> {
    // First collect the dependencies
    let dependency_list: Vec<String> = if let Some(task) = project.tasks.get(task_id) {
        if task.dependencies.is_empty() {
            super::print_info("No dependencies to remove");
            return Ok(());
        }

        task.dependencies.iter()
            .map(|dep| {
                let dep_type_str = match dep.dependency_type {
                    DependencyType::FinishToStart => "FS",
                    DependencyType::StartToStart => "SS", 
                    DependencyType::FinishToFinish => "FF",
                    DependencyType::StartToFinish => "SF",
                };
                let lag_str = if dep.lag_days != 0.0 {
                    format!(" ({:+} days)", dep.lag_days)
                } else {
                    String::new()
                };
                format!("{} ({}){}", dep.task_id, dep_type_str, lag_str)
            })
            .collect()
    } else {
        return Ok(());
    };

    let selected = Select::new("Select dependency to remove:", dependency_list)
        .with_vim_mode(true)
        .prompt()?;

    let dependency_task_id = selected.split(" (").next().unwrap();
    
    // Now borrow mutably to remove the dependency
    if let Some(task) = project.tasks.get_mut(task_id) {
        task.dependencies.retain(|dep| dep.task_id != dependency_task_id);
        super::print_success(&format!("Removed dependency '{}'", dependency_task_id));
    }
    Ok(())
}

fn modify_task_dependency(project: &mut Project, task_id: &str) -> Result<()> {
    // First collect the dependencies
    let (dependency_list, current_dep_info) = if let Some(task) = project.tasks.get(task_id) {
        if task.dependencies.is_empty() {
            super::print_info("No dependencies to modify");
            return Ok(());
        }

        let dependency_list: Vec<String> = task.dependencies.iter()
            .map(|dep| {
                let dep_type_str = match dep.dependency_type {
                    DependencyType::FinishToStart => "FS",
                    DependencyType::StartToStart => "SS", 
                    DependencyType::FinishToFinish => "FF",
                    DependencyType::StartToFinish => "SF",
                };
                let lag_str = if dep.lag_days != 0.0 {
                    format!(" ({:+} days)", dep.lag_days)
                } else {
                    String::new()
                };
                format!("{} ({}){}", dep.task_id, dep_type_str, lag_str)
            })
            .collect();

        let current_dep_info: Vec<(String, DependencyType, f32)> = task.dependencies.iter()
            .map(|dep| (dep.task_id.clone(), dep.dependency_type, dep.lag_days))
            .collect();

        (dependency_list, current_dep_info)
    } else {
        return Ok(());
    };

    let selected = Select::new("Select dependency to modify:", dependency_list)
        .with_vim_mode(true)
        .prompt()?;

    let dependency_task_id = selected.split(" (").next().unwrap();
    
    // Find the current dependency info
    let current_dep = current_dep_info.iter()
        .find(|(id, _, _)| id == dependency_task_id)
        .cloned();

    if let Some((_, current_type, current_lag)) = current_dep {
        // Select new dependency type
        let dependency_types = vec![
            DependencyType::FinishToStart,
            DependencyType::StartToStart,
            DependencyType::FinishToFinish,
            DependencyType::StartToFinish,
        ];

        let current_index = dependency_types.iter().position(|t| *t == current_type).unwrap_or(0);
        
        let new_dependency_type = Select::new("Dependency type:", dependency_types)
            .with_starting_cursor(current_index)
            .prompt()?;

        // Update lag/lead time
        let new_lag_days = CustomType::<f32>::new("Lag days (positive for lag, negative for lead):")
            .with_default(current_lag)
            .prompt()?;

        // Now borrow mutably to update the dependency
        if let Some(task) = project.tasks.get_mut(task_id) {
            if let Some(dep) = task.dependencies.iter_mut().find(|d| d.task_id == dependency_task_id) {
                dep.dependency_type = new_dependency_type;
                dep.lag_days = new_lag_days;
                super::print_success("Dependency modified");
            }
        }
    }
    Ok(())
}

fn edit_task_resources(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        if project.resources.is_empty() {
            super::print_error("No resources defined. Cannot update resource assignments.");
            return Ok(());
        }

        // Show current assignments
        if !task.resource_assignments.is_empty() {
            super::print_info("Current resource assignments:");
            for (resource_id, assignment) in &task.resource_assignments {
                let allocation_str = match &assignment.allocation_type {
                    crate::core::AllocationType::Hours(h) => format!("{:.1} hours", h),
                    crate::core::AllocationType::Percentage(p) => format!("{:.1}%", p),
                    crate::core::AllocationType::FullTime => "Full time".to_string(),
                };
                println!("    - {}: {}", resource_id, allocation_str);
            }
        }

        let choices = vec![
            "Add resource assignment",
            "Remove resource assignment",
            "Modify existing assignment",
            "Clear all assignments",
            "Done",
        ];

        loop {
            let choice = Select::new("Resource assignment options:", choices.clone())
                .prompt()?;

            match choice {
                "Add resource assignment" => add_resource_assignment(project, task_id)?,
                "Remove resource assignment" => remove_resource_assignment(project, task_id)?,
                "Modify existing assignment" => modify_resource_assignment(project, task_id)?,
                "Clear all assignments" => {
                    if Confirm::new("Remove all resource assignments?").prompt()? {
                        if let Some(task) = project.tasks.get_mut(task_id) {
                            task.resource_assignments.clear();
                            super::print_success("Cleared all resource assignments");
                        }
                    }
                }
                "Done" => break,
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}

fn add_resource_assignment(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        let available_resources: Vec<String> = project.resources.iter()
            .filter(|(id, _)| !task.resource_assignments.contains_key(*id))
            .map(|(id, r)| format!("{} - {} ({}/hr)", id, r.name, r.hourly_rate))
            .collect();

        if available_resources.is_empty() {
            super::print_info("All resources are already assigned to this task");
            return Ok(());
        }

        let selected = Select::new("Select resource to assign:", available_resources)
            .with_vim_mode(true)
            .prompt()?;

        let resource_id = selected.split(" - ").next().unwrap().to_string();

        // Ask for allocation type
        let allocation_choices = vec!["Hours", "Percentage", "Full Time"];
        let allocation_type = Select::new("Allocation type:", allocation_choices)
            .prompt()?;

        match allocation_type {
            "Hours" => {
                let hours = CustomType::<f32>::new("Hours for this resource:")
                    .with_validator(PositiveFloatValidator)
                    .prompt()?;
                task.add_resource_hours(resource_id, hours);
            }
            "Percentage" => {
                let percentage = CustomType::<f32>::new("Percentage of resource capacity (0-100):")
                    .with_validator(|input: &f32| {
                        if *input >= 0.0 && *input <= 100.0 {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid("Must be between 0 and 100".into()))
                        }
                    })
                    .prompt()?;
                task.add_resource_percentage(resource_id, percentage);
            }
            "Full Time" => {
                task.add_resource_full_time(resource_id);
            }
            _ => unreachable!(),
        }
        super::print_success("Resource assignment added");
    }
    Ok(())
}

fn remove_resource_assignment(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        if task.resource_assignments.is_empty() {
            super::print_info("No resource assignments to remove");
            return Ok(());
        }

        let resource_list: Vec<String> = task.resource_assignments.keys()
            .map(|id| {
                if let Some(resource) = project.resources.get(id) {
                    format!("{} - {}", id, resource.name)
                } else {
                    id.clone()
                }
            })
            .collect();

        let selected = Select::new("Select resource to remove:", resource_list)
            .with_vim_mode(true)
            .prompt()?;

        let resource_id = selected.split(" - ").next().unwrap();
        task.resource_assignments.remove(resource_id);
        super::print_success(&format!("Removed resource assignment for '{}'", resource_id));
    }
    Ok(())
}

fn modify_resource_assignment(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        if task.resource_assignments.is_empty() {
            super::print_info("No resource assignments to modify");
            return Ok(());
        }

        let resource_list: Vec<String> = task.resource_assignments.keys()
            .map(|id| {
                if let Some(resource) = project.resources.get(id) {
                    format!("{} - {}", id, resource.name)
                } else {
                    id.clone()
                }
            })
            .collect();

        let selected = Select::new("Select resource to modify:", resource_list)
            .with_vim_mode(true)
            .prompt()?;

        let resource_id = selected.split(" - ").next().unwrap().to_string();

        // Ask for new allocation type
        let allocation_choices = vec!["Hours", "Percentage", "Full Time"];
        let allocation_type = Select::new("New allocation type:", allocation_choices)
            .prompt()?;

        match allocation_type {
            "Hours" => {
                let hours = CustomType::<f32>::new("Hours for this resource:")
                    .with_validator(PositiveFloatValidator)
                    .prompt()?;
                task.add_resource_hours(resource_id, hours);
            }
            "Percentage" => {
                let percentage = CustomType::<f32>::new("Percentage of resource capacity (0-100):")
                    .with_validator(|input: &f32| {
                        if *input >= 0.0 && *input <= 100.0 {
                            Ok(Validation::Valid)
                        } else {
                            Ok(Validation::Invalid("Must be between 0 and 100".into()))
                        }
                    })
                    .prompt()?;
                task.add_resource_percentage(resource_id, percentage);
            }
            "Full Time" => {
                task.add_resource_full_time(resource_id);
            }
            _ => unreachable!(),
        }
        super::print_success("Resource assignment modified");
    }
    Ok(())
}

fn edit_task_completion(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        let new_completion = CustomType::<f32>::new("Completion (0.0-1.0):")
            .with_default(task.completion)
            .with_validator(|input: &f32| {
                if *input >= 0.0 && *input <= 1.0 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Must be between 0.0 and 1.0".into()))
                }
            })
            .prompt()?;
        task.completion = new_completion;
        super::print_success("Task completion updated");
    }
    Ok(())
}

fn edit_task_fixed_dates(project: &mut Project, task_id: &str) -> Result<()> {
    if let Some(task) = project.tasks.get_mut(task_id) {
        // Show current dates if any
        if task.start_date.is_some() || task.end_date.is_some() {
            super::print_info("Current fixed dates:");
            if let Some(start) = task.start_date {
                println!("  Start: {}", start.format("%Y-%m-%d"));
            }
            if let Some(end) = task.end_date {
                println!("  End: {}", end.format("%Y-%m-%d"));
            }
        }

        let choices = vec![
            "Set start date",
            "Set end date", 
            "Set both dates",
            "Clear fixed dates",
            "Skip",
        ];

        let choice = match prompt_select("Fixed date options:", choices)? {
            Some(choice) => choice,
            None => return Ok(()), // ESC pressed - exit fixed date editing
        };

        match choice {
            "Set start date" => {
                let start_date = DateSelect::new("Fixed start date:")
                    .with_default(task.start_date.unwrap_or_else(|| chrono::Local::now().date_naive()))
                    .prompt()?;
                task.start_date = Some(start_date);
                super::print_success("Start date set");
            }
            "Set end date" => {
                let end_date = DateSelect::new("Fixed end date:")
                    .with_default(task.end_date.unwrap_or_else(|| chrono::Local::now().date_naive()))
                    .prompt()?;
                task.end_date = Some(end_date);
                super::print_success("End date set");
            }
            "Set both dates" => {
                let start_date = DateSelect::new("Fixed start date:")
                    .with_default(task.start_date.unwrap_or_else(|| chrono::Local::now().date_naive()))
                    .prompt()?;
                let end_date = DateSelect::new("Fixed end date:")
                    .with_default(task.end_date.unwrap_or(start_date))
                    .prompt()?;
                
                if end_date < start_date {
                    super::print_error("End date cannot be before start date");
                } else {
                    task.start_date = Some(start_date);
                    task.end_date = Some(end_date);
                    super::print_success("Both dates set");
                }
            }
            "Clear fixed dates" => {
                task.start_date = None;
                task.end_date = None;
                super::print_success("Cleared fixed dates - task will use automatic scheduling");
            }
            "Skip" => {}
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn delete_task(project: &mut Project) -> Result<()> {
    if project.tasks.is_empty() {
        super::print_info("No tasks to delete");
        return Ok(());
    }

    let task_list: Vec<String> = project.tasks.keys()
        .map(|k| format!("{} - {}", k, project.tasks[k].name))
        .collect();

    let selected = Select::new("Select task to delete:", task_list)
        .with_vim_mode(true)
        .prompt()?;

    let task_id = selected.split(" - ").next().unwrap();

    if Confirm::new(&format!("Delete task '{}'?", task_id)).prompt()? {
        project.tasks.shift_remove(task_id);
        super::print_success(&format!("Deleted task '{}'", task_id));
    }

    Ok(())
}

fn view_tasks(project: &Project) -> Result<()> {
    if project.tasks.is_empty() {
        super::print_info("No tasks defined");
        return Ok(());
    }
    
    // Compute current schedule to get start/end dates
    let engine = SchedulingEngine::new();
    let schedule = engine.compute_schedule(project).unwrap_or_else(|_| Schedule {
        project_name: project.name.clone(),
        start_date: project.start_date,
        end_date: project.start_date,
        tasks: Default::default(),
        milestones: Default::default(),
        critical_path: Vec::new(),
        total_cost: 0.0,
        resource_utilization: Default::default(),
    });
    
    println!("\n{}", "Task Summary".bold());
    println!("{}", "─".repeat(80));
    
    // Show summary of all tasks
    for (id, task) in &project.tasks {
        let effort_hours = task.total_effort_hours();
        let duration_str = if let Some(duration) = task.duration_days {
            format!("{}d", duration)
        } else {
            "auto".to_string()
        };
        
        let schedule_str = if let Some(scheduled_task) = schedule.tasks.get(id) {
            format!(" | {} → {}", 
                scheduled_task.start_date.format("%m/%d"),
                scheduled_task.end_date.format("%m/%d")
            )
        } else {
            "".to_string()
        };
        
        let critical_marker = if schedule.tasks.get(id).map_or(false, |t| t.is_critical) {
            " (CRITICAL)".red()
        } else {
            "".normal()
        };
        
        println!("  • {} - {} | {:.0}h | {} | {:.0}%{}{}",
            id.to_string().yellow(),
            task.name,
            effort_hours,
            duration_str,
            task.completion * 100.0,
            schedule_str,
            critical_marker
        );
    }
    
    // Task selection menu
    loop {
        let mut choices: Vec<_> = project.tasks.keys()
            .map(|id| format!("{} - {}", id, project.tasks[id].name))
            .collect();
        choices.push("← Back".to_string());
        
        let choice = Select::new("\nSelect task to view details:", choices)
            .prompt()?;
            
        if choice == "← Back" {
            break;
        }
        
        // Extract task ID from choice
        let task_id = choice.split(" - ").next().unwrap();
        if let Some(task) = project.tasks.get(task_id) {
            view_task_details(task_id, task, &schedule)?;
        }
    }
    
    Ok(())
}

fn view_task_details(task_id: &str, task: &Task, schedule: &Schedule) -> Result<()> {
    println!("\n{}", format!("Task Details: {}", task_id).bold());
    println!("{}", "─".repeat(80));
    
    println!("Name: {}", task.name.bold());
    println!("Type: {}", task.task_type);
    println!("Effort: {} hours", task.total_effort_hours());
    
    if let Some(duration) = task.duration_days {
        println!("Duration: {} days", duration);
    } else {
        println!("Duration: Calculated from effort");
    }
    
    if let Some(work_units) = task.work_units {
        println!("Work Units: {}", work_units);
    }
    
    println!("Priority: {:?}", task.priority);
    println!("Completion: {:.0}%", task.completion * 100.0);
    
    // Show scheduling information
    if let Some(scheduled_task) = schedule.tasks.get(task_id) {
        println!("\n{}", "Schedule Information:".bold());
        println!("  Start Date: {}", scheduled_task.start_date.format("%Y-%m-%d"));
        println!("  End Date: {}", scheduled_task.end_date.format("%Y-%m-%d"));
        println!("  Assigned to: {}", scheduled_task.assigned_to);
        println!("  Cost: {:.2}", scheduled_task.cost);
        println!("  Critical Path: {}", if scheduled_task.is_critical { "Yes".red() } else { "No".green() });
        println!("  Slack: {} days", scheduled_task.slack);
    }
    
    if !task.skills_required.is_empty() {
        println!("\nSkills Required: {}", task.skills_required.join(", "));
    }
    
    if !task.dependencies.is_empty() {
        println!("\n{}", "Dependencies:".bold());
        for dep in &task.dependencies {
            let dep_type_str = match dep.dependency_type {
                DependencyType::FinishToStart => "FS",
                DependencyType::StartToStart => "SS", 
                DependencyType::FinishToFinish => "FF",
                DependencyType::StartToFinish => "SF",
            };
            let lag_str = if dep.lag_days != 0.0 {
                format!(" ({:+} days)", dep.lag_days)
            } else {
                String::new()
            };
            println!("    - {} ({}){}", dep.task_id, dep_type_str, lag_str);
        }
    }

    // Resource assignments
    if !task.resource_assignments.is_empty() {
        println!("\n{}", "Resource Assignments:".bold());
        for (resource_id, assignment) in &task.resource_assignments {
            let allocation_str = match &assignment.allocation_type {
                crate::core::AllocationType::Hours(h) => format!("{:.1} hours", h),
                crate::core::AllocationType::Percentage(p) => format!("{:.1}%", p),
                crate::core::AllocationType::FullTime => "Full time".to_string(),
            };
            println!("    - {}: {}", resource_id, allocation_str);
        }
    }

    // Show fixed dates if set
    if task.start_date.is_some() || task.end_date.is_some() {
        println!("\n{}", "Fixed Dates:".bold());
        if let Some(start) = task.start_date {
            println!("  Start: {}", start.format("%Y-%m-%d"));
        }
        if let Some(end) = task.end_date {
            println!("  End: {}", end.format("%Y-%m-%d"));
        }
    }

    println!("\n{}", "Press Enter to continue...".normal().dimmed());
    let _ = Text::new("").prompt();
    
    Ok(())
}

fn manage_resources(project: &mut Project) -> Result<()> {
    loop {
        let choices = vec![
            "Add Resource",
            "Edit Resource",
            "Delete Resource",
            "View Resources",
            "Back",
        ];

        let choice_result = show_submenu("\nResource Management:", choices)?;
        
        let choice = match choice_result {
            MenuResult::Selection(selection) => selection,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Add Resource" => add_resource(project)?,
            "Edit Resource" => edit_resource(project)?,
            "Delete Resource" => delete_resource(project)?,
            "View Resources" => view_resources(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn add_resource(project: &mut Project) -> Result<()> {
    println!("\n{}", "Adding New Resource".bold());

    let id = match prompt_text("Resource ID:")? {
        Some(id) => {
            if id.is_empty() {
                super::print_error("ID cannot be empty");
                return Ok(());
            }
            if id.contains(' ') {
                super::print_error("ID cannot contain spaces");
                return Ok(());
            }
            id
        },
        None => {
            println!("Resource creation cancelled");
            return Ok(());
        }
    };

    if project.resources.contains_key(&id) {
        super::print_error(&format!("Resource '{}' already exists", id));
        return Ok(());
    }

    let name = match prompt_text("Full name:")? {
        Some(name) => name,
        None => {
            println!("Resource creation cancelled");
            return Ok(());
        }
    };

    let hourly_rate = match prompt_custom_type("Hourly rate:", PositiveFloatValidator)? {
        Some(hourly_rate) => hourly_rate,
        None => {
            println!("Resource creation cancelled");
            return Ok(());
        }
    };

    let mut resource = Resource::new(name, hourly_rate);

    // Optional fields
    resource.role = Text::new("Role:")
        .with_default("")
        .prompt()
        .ok()
        .filter(|s| !s.is_empty());

    resource.capacity = CustomType::<f32>::new("Capacity (0.0-1.0, where 1.0 is full time):")
        .with_default(1.0)
        .with_validator(|input: &f32| {
            if *input > 0.0 && *input <= 1.0 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Must be between 0.0 and 1.0".into()))
            }
        })
        .prompt()?;

    // Skills
    if let Some(true) = prompt_confirm("Add skills?", false)? {
        let skills = match prompt_text("Skills (comma-separated):")? {
            Some(skills) => skills,
            None => {
                println!("Resource creation cancelled");
                return Ok(());
            }
        };
        resource.skills = skills.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    project.add_resource(id.clone(), resource);
    super::print_success(&format!("Added resource '{}'", id));

    Ok(())
}

fn edit_resource(project: &mut Project) -> Result<()> {
    if project.resources.is_empty() {
        super::print_info("No resources to edit");
        return Ok(());
    }

    let resource_list: Vec<String> = project.resources.iter()
        .map(|(id, r)| format!("{} - {} ({}/hr)", id, r.name, r.hourly_rate))
        .collect();

    let selected = match prompt_select("Select resource to edit:", resource_list)? {
        Some(selected) => selected,
        None => {
            println!("Resource editing cancelled");
            return Ok(());
        }
    };

    let resource_id = selected.split(" - ").next().unwrap();

    if let Some(resource) = project.resources.get_mut(resource_id) {
        resource.name = match prompt_text("Full name:")? {
            Some(name) => name,
            None => {
                println!("Resource editing cancelled");
                return Ok(());
            }
        };

        resource.hourly_rate = match prompt_custom_type("Hourly rate:", PositiveFloatValidator)? {
            Some(rate) => rate,
            None => {
                println!("Resource editing cancelled");
                return Ok(());
            }
        };

        resource.capacity = match prompt_custom_type("Capacity (0.0-1.0):", |input: &f32| {
            if *input > 0.0 && *input <= 1.0 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Must be between 0.0 and 1.0".into()))
            }
        })? {
            Some(capacity) => capacity,
            None => {
                println!("Resource editing cancelled");
                return Ok(());
            }
        };

        super::print_success(&format!("Updated resource '{}'", resource_id));
    }

    Ok(())
}

fn delete_resource(project: &mut Project) -> Result<()> {
    if project.resources.is_empty() {
        super::print_info("No resources to delete");
        return Ok(());
    }

    let resource_list: Vec<String> = project.resources.iter()
        .map(|(id, r)| format!("{} - {} ({}/hr)", id, r.name, r.hourly_rate))
        .collect();

    let selected = Select::new("Select resource to delete:", resource_list)
        .with_vim_mode(true)
        .prompt()?;

    let resource_id = selected.split(" - ").next().unwrap();

    // Check if resource is assigned to any tasks
    let assigned_tasks: Vec<&str> = project.tasks.iter()
        .filter(|(_, task)| task.resource_assignments.contains_key(resource_id))
        .map(|(id, _)| id.as_str())
        .collect();

    if !assigned_tasks.is_empty() {
        super::print_error(&format!(
            "Cannot delete resource '{}' - assigned to tasks: {}",
            resource_id,
            assigned_tasks.join(", ")
        ));
        return Ok(());
    }

    if Confirm::new(&format!("Delete resource '{}'?", resource_id)).prompt()? {
        project.resources.shift_remove(resource_id);
        super::print_success(&format!("Deleted resource '{}'", resource_id));
    }

    Ok(())
}

fn view_resources(project: &Project) -> Result<()> {
    if project.resources.is_empty() {
        super::print_info("No resources defined");
        return Ok(());
    }

    println!("\n{}", "Resources".bold());
    println!("{}", "─".repeat(80));

    for (id, resource) in &project.resources {
        println!("\n{}: {}", id.yellow(), resource.name.bold());
        if let Some(role) = &resource.role {
            println!("  Role: {}", role);
        }
        println!("  Hourly Rate: {:.2}", resource.hourly_rate);
        println!("  Capacity: {:.0}% ({} hrs/week)",
            resource.capacity * 100.0,
            resource.weekly_hours()
        );

        if !resource.skills.is_empty() {
            println!("  Skills: {}", resource.skills.join(", "));
        }

        // Show assigned tasks
        let assigned_tasks: Vec<&str> = project.tasks.iter()
            .filter(|(_, task)| task.resource_assignments.contains_key(id))
            .map(|(_, task)| task.name.as_str())
            .collect();

        if !assigned_tasks.is_empty() {
            println!("  Assigned Tasks: {}", assigned_tasks.join(", "));
        }
    }

    Ok(())
}

fn manage_milestones(project: &mut Project) -> Result<()> {
    loop {
        let choices = vec![
            "Add Milestone",
            "Edit Milestone",
            "Delete Milestone",
            "View Milestones",
            "Back",
        ];

        let choice = match show_submenu("\nMilestone Management:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Add Milestone" => add_milestone(project)?,
            "Edit Milestone" => edit_milestone(project)?,
            "Delete Milestone" => delete_milestone(project)?,
            "View Milestones" => view_milestones(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn add_milestone(project: &mut Project) -> Result<()> {
    println!("\n{}", "Adding New Milestone".bold());

    let id = match prompt_text("Milestone ID:")? {
        Some(id) => {
            if id.is_empty() {
                super::print_error("ID cannot be empty");
                return Ok(());
            }
            if id.contains(' ') {
                super::print_error("ID cannot contain spaces");
                return Ok(());
            }
            id
        },
        None => {
            println!("Milestone creation cancelled");
            return Ok(());
        }
    };

    if project.milestones.contains_key(&id) {
        super::print_error(&format!("Milestone '{}' already exists", id));
        return Ok(());
    }

    let name = match prompt_text("Milestone name:")? {
        Some(name) => name,
        None => {
            println!("Milestone creation cancelled");
            return Ok(());
        }
    };

    let mut milestone = Milestone::new(name);

    // Dependencies with fuzzy search
    if (!project.tasks.is_empty() || !project.milestones.is_empty()) &&
       Confirm::new("Does this milestone have dependencies?").prompt()? {
        let mut dep_list: Vec<String> = project.tasks.keys()
            .map(|k| format!("task: {} - {}", k, project.tasks[k].name))
            .collect();

        let milestone_deps: Vec<String> = project.milestones.keys()
            .filter(|k| **k != id)
            .map(|k| format!("milestone: {} - {}", k, project.milestones[k].name))
            .collect();

        dep_list.extend(milestone_deps);

        if !dep_list.is_empty() {
            let dependencies = MultiSelect::new("Select dependencies:", dep_list)
                .with_vim_mode(true)
                .prompt()?;

            milestone.dependencies = dependencies.iter()
                .map(|d| d.split(": ").nth(1).unwrap().split(" - ").next().unwrap().to_string())
                .collect();
        }
    }

    // Target date
    if Confirm::new("Set target date?").prompt()? {
        let date = DateSelect::new("Target date:")
            .with_default(chrono::Local::now().date_naive())
            .prompt()?;
        milestone.target_date = Some(date);
    }

    project.add_milestone(id.clone(), milestone);
    super::print_success(&format!("Added milestone '{}'", id));

    Ok(())
}

fn edit_milestone(project: &mut Project) -> Result<()> {
    if project.milestones.is_empty() {
        super::print_info("No milestones to edit");
        return Ok(());
    }

    let milestone_list: Vec<String> = project.milestones.keys()
        .map(|k| format!("{} - {}", k, project.milestones[k].name))
        .collect();

    let selected = match prompt_select("Select milestone to edit:", milestone_list)? {
        Some(selected) => selected,
        None => {
            println!("Milestone editing cancelled");
            return Ok(());
        }
    };

    let milestone_id = selected.split(" - ").next().unwrap();

    // Collect data before getting mutable references
    let current_milestone_name = project.milestones[milestone_id].name.clone();
    let current_milestone_dependencies = project.milestones[milestone_id].dependencies.clone();
    
    let mut dep_list: Vec<String> = project.tasks.keys()
        .map(|k| format!("task: {} - {}", k, project.tasks[k].name))
        .collect();

    let milestone_deps: Vec<String> = project.milestones.keys()
        .filter(|k| **k != milestone_id)
        .map(|k| format!("milestone: {} - {}", k, project.milestones[k].name))
        .collect();

    dep_list.extend(milestone_deps);

    // Pre-calculate dependency indices
    let current_dep_indices: Vec<usize> = current_milestone_dependencies.iter()
        .filter_map(|dep| {
            let dep_string = if project.tasks.contains_key(dep) {
                format!("task: {} - {}", dep, project.tasks[dep].name)
            } else if project.milestones.contains_key(dep) {
                format!("milestone: {} - {}", dep, project.milestones[dep].name)
            } else {
                return None;
            };
            dep_list.iter().position(|item| item == &dep_string)
        })
        .collect();

    if let Some(milestone) = project.milestones.get_mut(milestone_id) {
        milestone.name = Text::new("Milestone name:")
            .with_default(&current_milestone_name)
            .prompt()?;

        // Edit dependencies
        if Confirm::new("Update dependencies?").prompt()? {
            if !dep_list.is_empty() {
                let dependencies = MultiSelect::new("Select dependencies:", dep_list)
                    .with_default(&current_dep_indices)
                    .with_vim_mode(true)
                    .prompt()?;

                milestone.dependencies = dependencies.iter()
                    .map(|d| d.split(": ").nth(1).unwrap().split(" - ").next().unwrap().to_string())
                    .collect();
            } else {
                milestone.dependencies.clear();
            }
        }

        if let Some(current_date) = milestone.target_date {
            if Confirm::new("Update target date?").prompt()? {
                let date = DateSelect::new("Target date:")
                    .with_default(current_date)
                    .prompt()?;
                milestone.target_date = Some(date);
            }
        } else if Confirm::new("Set target date?").prompt()? {
            let date = DateSelect::new("Target date:")
                .with_default(chrono::Local::now().date_naive())
                .prompt()?;
            milestone.target_date = Some(date);
        }

        super::print_success(&format!("Updated milestone '{}'", milestone_id));
    }

    Ok(())
}

fn delete_milestone(project: &mut Project) -> Result<()> {
    if project.milestones.is_empty() {
        super::print_info("No milestones to delete");
        return Ok(());
    }

    let milestone_list: Vec<String> = project.milestones.keys()
        .map(|k| format!("{} - {}", k, project.milestones[k].name))
        .collect();

    let selected = Select::new("Select milestone to delete:", milestone_list)
        .with_vim_mode(true)
        .prompt()?;

    let milestone_id = selected.split(" - ").next().unwrap();

    if Confirm::new(&format!("Delete milestone '{}'?", milestone_id)).prompt()? {
        project.milestones.shift_remove(milestone_id);
        super::print_success(&format!("Deleted milestone '{}'", milestone_id));
    }

    Ok(())
}

fn view_milestones(project: &Project) -> Result<()> {
    if project.milestones.is_empty() {
        super::print_info("No milestones defined");
        return Ok(());
    }

    println!("\n{}", "Milestones".bold());
    println!("{}", "─".repeat(80));

    // Compute schedule to get expected dates
    let engine = SchedulingEngine::new();
    let schedule = engine.compute_schedule(project).unwrap_or_else(|_| Schedule {
        project_name: project.name.clone(),
        start_date: project.start_date,
        end_date: project.start_date,
        tasks: Default::default(),
        milestones: Default::default(),
        critical_path: Vec::new(),
        total_cost: 0.0,
        resource_utilization: Default::default(),
    });

    for (id, milestone) in &project.milestones {
        println!("\n{}: {}", id.yellow(), milestone.name.bold());

        if let Some(date) = milestone.target_date {
            println!("  Target Date: {}", date.format("%Y-%m-%d"));
        } else {
            println!("  Target Date: Not set");
        }

        // Show expected date from schedule
        if let Some(scheduled_milestone) = schedule.milestones.get(id) {
            let critical_marker = if scheduled_milestone.is_critical { " (CRITICAL)".red() } else { "".normal() };
            println!("  Expected Date: {}{}", 
                scheduled_milestone.date.format("%Y-%m-%d"),
                critical_marker
            );
            
            // Show variance if both target and expected dates exist
            if let Some(target_date) = milestone.target_date {
                let variance_days = (scheduled_milestone.date - target_date).num_days();
                if variance_days != 0 {
                    let variance_color = if variance_days > 0 { "red" } else { "green" };
                    let variance_text = if variance_days > 0 {
                        format!("  Variance: {} days LATE", variance_days)
                    } else {
                        format!("  Variance: {} days EARLY", variance_days.abs())
                    };
                    println!("{}", variance_text.color(variance_color));
                }
            }
        } else {
            println!("  Expected Date: Not scheduled");
        }

        if !milestone.dependencies.is_empty() {
            println!("  Dependencies: {}", milestone.dependencies.join(", "));
        }

        if let Some(desc) = &milestone.description {
            println!("  Description: {}", desc);
        }
    }

    Ok(())
}

fn compute_schedule_interactive(project: &Project) -> Result<()> {
    let engine = SchedulingEngine::new();

    println!("\n{}", "Computing schedule...".yellow());
    let schedule = engine.compute_schedule(project)?;

    println!("\n{}", "Schedule Computed!".bold().green());
    println!("{}", "─".repeat(50));
    println!("Duration: {} → {}",
        schedule.start_date.format("%Y-%m-%d"),
        schedule.end_date.format("%Y-%m-%d")
    );
    println!("Total Days: {}", (schedule.end_date - schedule.start_date).num_days());
    println!("Total Cost: {} {:.2}", project.currency, schedule.total_cost);
    
    // Show critical path information
    if !schedule.critical_path.is_empty() {
        println!("\n{}", "Critical Path:".bold().red());
        println!("  Tasks: {}", schedule.critical_path.join(" → "));
        let critical_tasks: Vec<&String> = schedule.tasks.iter()
            .filter(|(_, task)| task.is_critical)
            .map(|(id, _)| id)
            .collect();
        println!("  Critical Tasks: {}", critical_tasks.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    }

    // Show milestones status
    if !schedule.milestones.is_empty() {
        println!("\n{}", "Milestones:".bold());
        for (id, milestone) in &schedule.milestones {
            let critical_marker = if milestone.is_critical { " (CRITICAL)".red() } else { "".normal() };
            println!("  • {}: {} - {}{}",
                id.to_string().yellow(),
                milestone.name,
                milestone.date.format("%Y-%m-%d"),
                critical_marker
            );
        }
    }

    // Single streamlined report selection
    let report_choices = vec![
        "Gantt Chart (Mermaid)",
        "Resource Utilization Table", 
        "Cost Report",
        "Critical Path Analysis",
        "Task Slack Analysis",
        "All Reports",
        "Save Reports to Files",
        "Skip Reports",
    ];

    let choice = match prompt_select("\nWhat would you like to see?", report_choices)? {
        Some(choice) => choice,
        None => {
            println!("Schedule analysis cancelled");
            return Ok(());
        }
    };

    match choice {
        "Gantt Chart (Mermaid)" => {
            let gantt = crate::reporting::GanttGenerator::generate_mermaid(&schedule)?;
            println!("\n```mermaid");
            println!("{}", gantt);
            println!("```");
        }
        "Resource Utilization Table" => {
            let table = crate::reporting::ResourceReporter::generate_table(&schedule)?;
            println!("\n{}", table);
        }
        "Cost Report" => {
            let report = crate::reporting::CostReporter::generate_markdown(&schedule, &project.currency)?;
            println!("\n{}", report);
        }
        "Critical Path Analysis" => {
            show_critical_path_analysis(&schedule)?;
        }
        "Task Slack Analysis" => {
            show_task_slack_analysis(&schedule)?;
        }
        "All Reports" => {
            println!("\n{}", "=== GANTT CHART ===".bold());
            let gantt = crate::reporting::GanttGenerator::generate_mermaid(&schedule)?;
            println!("```mermaid");
            println!("{}", gantt);
            println!("```");
            
            println!("\n{}", "=== RESOURCE UTILIZATION ===".bold());
            let table = crate::reporting::ResourceReporter::generate_table(&schedule)?;
            println!("{}", table);
            
            println!("\n{}", "=== COST REPORT ===".bold());
            let cost_report = crate::reporting::CostReporter::generate_markdown(&schedule, &project.currency)?;
            println!("{}", cost_report);

            println!("\n{}", "=== CRITICAL PATH ANALYSIS ===".bold());
            show_critical_path_analysis(&schedule)?;

            println!("\n{}", "=== TASK SLACK ANALYSIS ===".bold());
            show_task_slack_analysis(&schedule)?;
        }
        "Save Reports to Files" => {
            save_all_reports(project, &schedule)?;
        }
        "Skip Reports" => {}
        _ => unreachable!(),
    }

    Ok(())
}

fn schedule_analysis_interactive(project: &Project) -> Result<()> {
    let engine = SchedulingEngine::new();
    
    println!("\n{}", "Computing schedule for analysis...".yellow());
    let schedule = match engine.compute_schedule(project) {
        Ok(schedule) => schedule,
        Err(e) => {
            super::print_error(&format!("Failed to compute schedule: {}", e));
            return Ok(());
        }
    };

    loop {
        let choices = vec![
            "Critical Path Analysis",
            "Task Slack Analysis",
            "Resource Loading",
            "Milestone Schedule",
            "Task Schedule Details",
            "Back",
        ];

        let choice = match show_submenu("\nSchedule Analysis:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Critical Path Analysis" => show_critical_path_analysis(&schedule)?,
            "Task Slack Analysis" => show_task_slack_analysis(&schedule)?,
            "Resource Loading" => {
                let table = crate::reporting::ResourceReporter::generate_table(&schedule)?;
                println!("\n{}", table);
            }
            "Milestone Schedule" => show_milestone_schedule(&schedule)?,
            "Task Schedule Details" => show_task_schedule_details(&schedule)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn show_milestone_schedule(schedule: &Schedule) -> Result<()> {
    println!("\n{}", "Milestone Schedule".bold());
    println!("{}", "─".repeat(80));

    if schedule.milestones.is_empty() {
        super::print_info("No milestones in schedule");
        return Ok(());
    }

    // Sort milestones by date
    let mut milestones: Vec<_> = schedule.milestones.iter().collect();
    milestones.sort_by_key(|(_, milestone)| milestone.date);

    println!("{:<20} {:<30} {:<12} {:<10}",
        "Milestone ID", "Name", "Date", "Critical"
    );
    println!("{}", "─".repeat(72));

    for (id, milestone) in milestones {
        let critical_marker = if milestone.is_critical { "YES".red() } else { "NO".green() };
        let name_truncated = if milestone.name.len() > 28 {
            format!("{}...", &milestone.name[..25])
        } else {
            milestone.name.clone()
        };

        println!("{:<20} {:<30} {:<12} {}",
            id,
            name_truncated,
            milestone.date.format("%Y-%m-%d"),
            critical_marker
        );
    }

    Ok(())
}

fn show_task_schedule_details(schedule: &Schedule) -> Result<()> {
    println!("\n{}", "Task Schedule Details".bold());
    println!("{}", "─".repeat(120));

    if schedule.tasks.is_empty() {
        super::print_info("No tasks in schedule");
        return Ok(());
    }

    // Sort tasks by start date
    let mut tasks: Vec<_> = schedule.tasks.iter().collect();
    tasks.sort_by_key(|(_, task)| task.start_date);

    println!("{:<15} {:<25} {:<12} {:<12} {:<8} {:<8} {:<8} {:<10}",
        "Task ID", "Name", "Start", "End", "Duration", "Effort", "Slack", "Critical"
    );
    println!("{}", "─".repeat(106));

    for (task_id, task) in tasks {
        let duration = (task.end_date - task.start_date).num_days() + 1;
        let critical_marker = if task.is_critical { "YES".red() } else { "NO".green() };
        let slack_color = if task.slack == 0 {
            format!("{}", task.slack).red()
        } else if task.slack <= 2 {
            format!("{}", task.slack).yellow()
        } else {
            format!("{}", task.slack).green()
        };

        let name_truncated = if task.name.len() > 23 {
            format!("{}...", &task.name[..20])
        } else {
            task.name.clone()
        };

        println!("{:<15} {:<25} {:<12} {:<12} {:<8} {:<8.1} {} {}",
            task_id,
            name_truncated,
            task.start_date.format("%Y-%m-%d"),
            task.end_date.format("%Y-%m-%d"),
            format!("{}d", duration),
            task.effort,
            format!("{:<8}", slack_color),
            critical_marker
        );
    }

    Ok(())
}

fn generate_reports_interactive(project: &Project) -> Result<()> {
    let engine = SchedulingEngine::new();
    
    println!("\n{}", "Computing schedule for reports...".yellow());
    let schedule = engine.compute_schedule(project)?;

    let choices = vec![
        "Gantt Chart (Mermaid)",
        "Gantt Chart (WBS Timeline)",
        "Gantt Chart (Resource Utilization)",
        "Gantt Chart (Markdown)",
        "Cost Report",
        "Resource Utilization Report",
        "All Reports (Display)",
        "Save All Reports to Files",
    ];

    let choice = match prompt_select("\nSelect report to generate:", choices)? {
        Some(choice) => choice,
        None => {
            println!("Report generation cancelled");
            return Ok(());
        }
    };

    match choice {
        "Gantt Chart (Mermaid)" => {
            let gantt = crate::reporting::GanttGenerator::generate_mermaid(&schedule)?;
            println!("\n```mermaid");
            println!("{}", gantt);
            println!("```");
        }
        "Gantt Chart (WBS Timeline)" => {
            let gantt = crate::reporting::GanttGenerator::generate_wbs_mermaid(&schedule)?;
            println!("\n```mermaid");
            println!("{}", gantt);
            println!("```");
        }
        "Gantt Chart (Resource Utilization)" => {
            let gantt = crate::reporting::GanttGenerator::generate_utilization_mermaid(&schedule)?;
            println!("\n```mermaid");
            println!("{}", gantt);
            println!("```");
        }
        "Gantt Chart (Markdown)" => {
            let gantt = crate::reporting::GanttGenerator::generate_markdown(&schedule)?;
            println!("\n{}", gantt);
        }
        "Cost Report" => {
            let report = crate::reporting::CostReporter::generate_markdown(&schedule, &project.currency)?;
            println!("\n{}", report);
        }
        "Resource Utilization Report" => {
            let report = crate::reporting::ResourceReporter::generate_markdown(&schedule)?;
            println!("\n{}", report);
        }
        "All Reports (Display)" => {
            println!("\n{}", "=== GANTT CHART ===".bold());
            let gantt = crate::reporting::GanttGenerator::generate_mermaid(&schedule)?;
            println!("```mermaid");
            println!("{}", gantt);
            println!("```");
            
            println!("\n{}", "=== RESOURCE UTILIZATION ===".bold());
            let resource_report = crate::reporting::ResourceReporter::generate_markdown(&schedule)?;
            println!("{}", resource_report);
            
            println!("\n{}", "=== COST REPORT ===".bold());
            let cost_report = crate::reporting::CostReporter::generate_markdown(&schedule, &project.currency)?;
            println!("{}", cost_report);
        }
        "Save All Reports to Files" => {
            save_all_reports(project, &schedule)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn save_all_reports(project: &Project, schedule: &Schedule) -> Result<()> {
    let dir = Text::new("Output directory:")
        .with_default("reports")
        .prompt()?;

    std::fs::create_dir_all(&dir)?;

    // Save Gantt chart
    let gantt = crate::reporting::GanttGenerator::generate_markdown(schedule)?;
    std::fs::write(format!("{}/gantt.md", dir), gantt)?;

    // Save cost report
    let costs = crate::reporting::CostReporter::generate_markdown(schedule, &project.currency)?;
    std::fs::write(format!("{}/costs.md", dir), costs)?;

    // Save resource report
    let resources = crate::reporting::ResourceReporter::generate_markdown(schedule)?;
    std::fs::write(format!("{}/resources.md", dir), resources)?;

    // Save schedule JSON
    let json = serde_json::to_string_pretty(schedule)?;
    std::fs::write(format!("{}/schedule.json", dir), json)?;

    super::print_success(&format!("Reports saved to {}/", dir));

    Ok(())
}

fn save_project(project: &Project) -> Result<()> {
    let path = Text::new("Save to:")
        .with_default("project.ron")
        .prompt()?;

    project.save_to_file(&path)?;
    super::print_success(&format!("Project saved to {}", path));

    Ok(())
}

// Calendar Management Functions
fn manage_calendars(project: &mut Project) -> Result<()> {
    loop {
        let choices = vec![
            "Add Calendar",
            "Edit Calendar",
            "Delete Calendar",
            "View Calendars",
            "Manage Resource Calendars",
            "Set Default Calendar",
            "Back",
        ];

        let choice = match show_submenu("\nCalendar Management:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Add Calendar" => add_calendar(project)?,
            "Edit Calendar" => edit_calendar(project)?,
            "Delete Calendar" => delete_calendar(project)?,
            "View Calendars" => view_calendars(project)?,
            "Manage Resource Calendars" => manage_resource_calendars(project)?,
            "Set Default Calendar" => set_default_calendar(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn add_calendar(project: &mut Project) -> Result<()> {
    println!("\n{}", "Adding New Calendar".bold());

    let id = match prompt_text("Calendar ID:")? {
        Some(id) => {
            if id.is_empty() {
                super::print_error("ID cannot be empty");
                return Ok(());
            }
            if id.contains(' ') {
                super::print_error("ID cannot contain spaces");
                return Ok(());
            }
            id
        },
        None => {
            println!("Calendar creation cancelled");
            return Ok(());
        }
    };

    if project.calendars.contains_key(&id) {
        super::print_error(&format!("Calendar '{}' already exists", id));
        return Ok(());
    }

    let name = Text::new("Calendar name:")
        .prompt()?;

    let mut calendar = Calendar::new(name);

    // Configure working days
    let all_weekdays = vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    let weekday_strings: Vec<String> = all_weekdays.iter()
        .map(|day| format!("{}", day))
        .collect();

    let default_indices = vec![0, 1, 2, 3, 4]; // Mon-Fri

    let working_days = MultiSelect::new("Select working days:", weekday_strings)
        .with_default(&default_indices)
        .prompt()?;

    calendar.working_days = working_days.iter()
        .map(|day_str| match day_str.as_str() {
            "Mon" => Weekday::Mon,
            "Tue" => Weekday::Tue,
            "Wed" => Weekday::Wed,
            "Thu" => Weekday::Thu,
            "Fri" => Weekday::Fri,
            "Sat" => Weekday::Sat,
            "Sun" => Weekday::Sun,
            _ => unreachable!(),
        })
        .collect();

    // Configure working hours
    let start_time = CustomType::<u8>::new("Start time (hour, 0-23):")
        .with_default(9)
        .with_validator(|input: &u8| {
            if *input <= 23 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Must be between 0 and 23".into()))
            }
        })
        .prompt()?;

    let end_time = CustomType::<u8>::new("End time (hour, 0-23):")
        .with_default(17)
        .with_validator(|input: &u8| {
            if *input <= 23 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Must be between 0 and 23".into()))
            }
        })
        .prompt()?;

    let daily_hours = CustomType::<f32>::new("Daily working hours:")
        .with_default(8.0)
        .with_validator(PositiveFloatValidator)
        .prompt()?;

    calendar.working_hours = WorkingHours {
        start_time,
        end_time,
        daily_hours,
    };

    // Add holidays
    if Confirm::new("Add holidays?").prompt()? {
        loop {
            let holiday_name = Text::new("Holiday name:")
                .prompt()?;

            let holiday_date = DateSelect::new("Holiday date:")
                .with_default(chrono::Local::now().date_naive())
                .prompt()?;

            let recurring = Confirm::new("Is this a recurring holiday?")
                .with_default(false)
                .prompt()?;

            let mut holiday = Holiday::new(holiday_name, holiday_date);
            if recurring {
                holiday = holiday.recurring();
            }

            calendar.add_holiday(holiday);

            if !Confirm::new("Add another holiday?").prompt()? {
                break;
            }
        }
    }

    project.calendars.insert(id.clone(), calendar);
    super::print_success(&format!("Added calendar '{}'", id));

    Ok(())
}

fn edit_calendar(project: &mut Project) -> Result<()> {
    if project.calendars.is_empty() {
        super::print_info("No calendars to edit");
        return Ok(());
    }

    let calendar_list: Vec<String> = project.calendars.keys()
        .map(|k| format!("{} - {}", k, project.calendars[k].name))
        .collect();

    let selected = match prompt_select("Select calendar to edit:", calendar_list)? {
        Some(selected) => selected,
        None => {
            println!("Calendar editing cancelled");
            return Ok(());
        }
    };

    let calendar_id = selected.split(" - ").next().unwrap();

    if let Some(calendar) = project.calendars.get_mut(calendar_id) {
        calendar.name = match prompt_text("Calendar name:")? {
            Some(name) => name,
            None => {
                println!("Calendar editing cancelled");
                return Ok(());
            }
        };

        super::print_success(&format!("Updated calendar '{}'", calendar_id));
    }

    Ok(())
}

fn delete_calendar(project: &mut Project) -> Result<()> {
    if project.calendars.is_empty() {
        super::print_info("No calendars to delete");
        return Ok(());
    }

    let calendar_list: Vec<String> = project.calendars.keys()
        .map(|k| format!("{} - {}", k, project.calendars[k].name))
        .collect();

    let selected = Select::new("Select calendar to delete:", calendar_list)
        .with_vim_mode(true)
        .prompt()?;

    let calendar_id = selected.split(" - ").next().unwrap();

    if Some(calendar_id) == project.default_calendar_id.as_deref() {
        super::print_error("Cannot delete the default calendar");
        return Ok(());
    }

    if Confirm::new(&format!("Delete calendar '{}'?", calendar_id)).prompt()? {
        project.calendars.shift_remove(calendar_id);
        super::print_success(&format!("Deleted calendar '{}'", calendar_id));
    }

    Ok(())
}

fn view_calendars(project: &Project) -> Result<()> {
    if project.calendars.is_empty() {
        super::print_info("No calendars defined");
        return Ok(());
    }

    println!("\n{}", "Calendars".bold());
    println!("{}", "─".repeat(80));

    for (id, calendar) in &project.calendars {
        let is_default = Some(id) == project.default_calendar_id.as_ref();
        let default_marker = if is_default { " (Default)" } else { "" };
        
        println!("\n{}: {}{}", id.yellow(), calendar.name.bold(), default_marker);
        println!("  Working Days: {}", calendar.working_days.iter()
            .map(|day| format!("{}", day))
            .collect::<Vec<_>>()
            .join(", "));
        println!("  Working Hours: {:02}:00 - {:02}:00 ({:.1}h/day)", 
            calendar.working_hours.start_time, 
            calendar.working_hours.end_time,
            calendar.working_hours.daily_hours);
        
        if !calendar.holidays.is_empty() {
            println!("  Holidays: {}", calendar.holidays.len());
            for holiday in &calendar.holidays {
                let recurring = if holiday.recurring { " (recurring)" } else { "" };
                println!("    - {}: {}{}", holiday.name, holiday.date.format("%Y-%m-%d"), recurring);
            }
        }
    }

    Ok(())
}

fn manage_resource_calendars(project: &mut Project) -> Result<()> {
    if project.resources.is_empty() {
        super::print_info("No resources available");
        return Ok(());
    }

    if project.calendars.is_empty() {
        super::print_info("No calendars available");
        return Ok(());
    }

    let resource_list: Vec<String> = project.resources.keys()
        .map(|k| format!("{} - {}", k, project.resources[k].name))
        .collect();

    let selected = Select::new("Select resource to assign calendar:", resource_list)
        .with_vim_mode(true)
        .prompt()?;

    let resource_id = selected.split(" - ").next().unwrap().to_string();

    let calendar_list: Vec<String> = project.calendars.keys()
        .map(|k| format!("{} - {}", k, project.calendars[k].name))
        .collect();

    let selected_calendar = Select::new("Select calendar:", calendar_list)
        .with_vim_mode(true)
        .prompt()?;

    let calendar_id = selected_calendar.split(" - ").next().unwrap().to_string();

    // Remove existing calendar assignment for this resource
    project.resource_calendars.retain(|rc| rc.resource_id != resource_id);

    // Add new assignment
    let resource_calendar = ResourceCalendar::new(resource_id.clone(), calendar_id.clone());
    project.resource_calendars.push(resource_calendar);

    super::print_success(&format!("Assigned calendar '{}' to resource '{}'", calendar_id, resource_id));

    Ok(())
}

fn set_default_calendar(project: &mut Project) -> Result<()> {
    if project.calendars.is_empty() {
        super::print_info("No calendars available");
        return Ok(());
    }

    let calendar_list: Vec<String> = project.calendars.keys()
        .map(|k| format!("{} - {}", k, project.calendars[k].name))
        .collect();

    let selected = Select::new("Select default calendar:", calendar_list)
        .with_vim_mode(true)
        .prompt()?;

    let calendar_id = selected.split(" - ").next().unwrap().to_string();
    project.default_calendar_id = Some(calendar_id.clone());

    super::print_success(&format!("Set '{}' as default calendar", calendar_id));

    Ok(())
}

// Baseline Management Functions
fn manage_baselines(project: &mut Project) -> Result<()> {
    let baseline_manager = BaselineManager::new(std::path::PathBuf::from("project.ron"));

    loop {
        let choices = vec![
            "Create Baseline",
            "View Baselines",
            "Compare Baselines",
            "Set Current Baseline",
            "Baseline Metrics",
            "Archive Baseline",
            "Delete Baseline",
            "Back",
        ];

        let choice = match show_submenu("\nBaseline Management:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Create Baseline" => create_baseline(project, &baseline_manager)?,
            "View Baselines" => view_baselines_enhanced(&baseline_manager)?,
            "Compare Baselines" => compare_baselines(&baseline_manager)?,
            "Set Current Baseline" => set_current_baseline_enhanced(&baseline_manager)?,
            "Baseline Metrics" => show_baseline_metrics(&baseline_manager)?,
            "Archive Baseline" => archive_baseline(&baseline_manager)?,
            "Delete Baseline" => delete_baseline_enhanced(&baseline_manager)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn create_baseline(project: &Project, baseline_manager: &BaselineManager) -> Result<()> {
    println!("\n{}", "Creating New Baseline".bold());

    // First compute the current schedule
    let engine = SchedulingEngine::new();
    let schedule = match engine.compute_schedule(project) {
        Ok(schedule) => schedule,
        Err(e) => {
            super::print_error(&format!("Cannot create baseline: Failed to compute schedule - {}", e));
            return Ok(());
        }
    };

    // Get baseline information from user
    let name = Text::new("Baseline name:")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Ok(Validation::Invalid("Name cannot be empty".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let baseline_types = vec![
        BaselineType::Initial,
        BaselineType::Approved,
        BaselineType::Working,
        BaselineType::Archived,
    ];

    let baseline_type = match prompt_select("Baseline type:", baseline_types)? {
        Some(baseline_type) => baseline_type,
        None => {
            println!("Baseline creation cancelled");
            return Ok(());
        }
    };

    let description = if let Some(true) = prompt_confirm("Add description?", false)? {
        match prompt_text("Description:")? {
            Some(desc) => Some(desc),
            None => {
                println!("Baseline creation cancelled");
                return Ok(());
            }
        }
    } else {
        None
    };

    let created_by = match prompt_text("Created by:")? {
        Some(created_by) => created_by,
        None => {
            println!("Baseline creation cancelled");
            return Ok(());
        }
    };

    // Create the baseline
    match baseline_manager.create_baseline(
        project,
        &schedule,
        baseline_type,
        name.clone(),
        description,
        created_by,
    ) {
        Ok(baseline) => {
            super::print_success(&format!("Created baseline '{}' (ID: {})", name, baseline.baseline_id));
            
            // Show baseline summary
            println!("\n{}", "Baseline Summary:".bold());
            println!("  Type: {}", baseline.baseline_type);
            println!("  Duration: {} → {}", 
                baseline.project_snapshot.start_date.format("%Y-%m-%d"),
                baseline.project_snapshot.end_date.format("%Y-%m-%d")
            );
            println!("  Total Cost: {:.2}", baseline.project_snapshot.total_cost);
            println!("  Total Tasks: {}", baseline.project_snapshot.tasks.len());
            println!("  Total Milestones: {}", baseline.project_snapshot.milestones.len());
            println!("  Created: {}", baseline.created_date.format("%Y-%m-%d %H:%M:%S"));
        }
        Err(e) => {
            super::print_error(&format!("Failed to create baseline: {}", e));
        }
    }

    Ok(())
}

fn view_baselines_enhanced(baseline_manager: &BaselineManager) -> Result<()> {
    let baselines = match baseline_manager.list_baselines() {
        Ok(baselines) => baselines,
        Err(e) => {
            super::print_error(&format!("Failed to load baselines: {}", e));
            return Ok(());
        }
    };

    if baselines.is_empty() {
        super::print_info("No baselines found");
        return Ok(());
    }

    println!("\n{}", "Project Baselines".bold());
    println!("{}", "─".repeat(120));

    println!("{:<25} {:<20} {:<12} {:<12} {:<15} {:<12} {:<10}",
        "Baseline ID", "Name", "Type", "Created", "End Date", "Cost", "Current"
    );
    println!("{}", "─".repeat(120));

    for baseline in &baselines {
        let current_marker = if baseline.is_current { "YES".green() } else { "NO".normal() };
        let name_truncated = if baseline.name.len() > 18 {
            format!("{}...", &baseline.name[..15])
        } else {
            baseline.name.clone()
        };

        println!("{:<25} {:<20} {:<12} {:<12} {:<15} {:<12.2} {}",
            &baseline.baseline_id[..25.min(baseline.baseline_id.len())],
            name_truncated,
            baseline.baseline_type,
            baseline.created_date.format("%Y-%m-%d"),
            baseline.project_end_date.format("%Y-%m-%d"),
            baseline.total_cost,
            current_marker
        );
    }

    // Show baseline statistics
    let current_count = baselines.iter().filter(|b| b.is_current).count();
    let initial_count = baselines.iter().filter(|b| b.baseline_type == BaselineType::Initial).count();
    let approved_count = baselines.iter().filter(|b| b.baseline_type == BaselineType::Approved).count();
    let working_count = baselines.iter().filter(|b| b.baseline_type == BaselineType::Working).count();
    let archived_count = baselines.iter().filter(|b| b.baseline_type == BaselineType::Archived).count();

    println!("\n{}", "Baseline Summary:".bold());
    println!("  Total: {} | Current: {} | Initial: {} | Approved: {} | Working: {} | Archived: {}",
        baselines.len(), current_count, initial_count, approved_count, working_count, archived_count
    );

    Ok(())
}

fn compare_baselines(baseline_manager: &BaselineManager) -> Result<()> {
    let baselines = baseline_manager.list_baselines()?;
    
    if baselines.len() < 2 {
        super::print_info("Need at least 2 baselines to compare");
        return Ok(());
    }

    let baseline_list: Vec<String> = baselines.iter()
        .map(|b| format!("{} - {}", b.baseline_id, b.name))
        .collect();

    let current_baseline = Select::new("Select current baseline:", baseline_list.clone())
        .with_vim_mode(true)
        .prompt()?;

    let compare_baseline = Select::new("Select baseline to compare against:", baseline_list)
        .with_vim_mode(true)
        .prompt()?;

    let current_id = current_baseline.split(" - ").next().unwrap();
    let compare_id = compare_baseline.split(" - ").next().unwrap();

    if current_id == compare_id {
        super::print_error("Cannot compare a baseline to itself");
        return Ok(());
    }

    match baseline_manager.compare_baselines(current_id, compare_id) {
        Ok(comparison) => show_baseline_comparison(&comparison)?,
        Err(e) => super::print_error(&format!("Failed to compare baselines: {}", e)),
    }

    Ok(())
}

fn show_baseline_comparison(comparison: &crate::core::BaselineComparison) -> Result<()> {
    println!("\n{}", "Baseline Comparison".bold());
    println!("{}", "─".repeat(80));

    println!("Current Baseline: {}", comparison.current_baseline.yellow());
    println!("Comparison Baseline: {}", comparison.comparison_baseline.yellow());
    println!("Generated: {}", comparison.generated_date.format("%Y-%m-%d %H:%M:%S"));

    println!("\n{}", "Overall Variance:".bold());
    let schedule_variance_color = if comparison.schedule_variance_days > 0 {
        format!("+{} days", comparison.schedule_variance_days).red()
    } else if comparison.schedule_variance_days < 0 {
        format!("{} days", comparison.schedule_variance_days).green()
    } else {
        "No variance".green()
    };

    let cost_variance_color = if comparison.cost_variance > 0.0 {
        format!("+{:.2}", comparison.cost_variance).red()
    } else if comparison.cost_variance < 0.0 {
        format!("{:.2}", comparison.cost_variance).green()
    } else {
        "No variance".green()
    };

    println!("  Schedule Variance: {}", schedule_variance_color);
    println!("  Cost Variance: {}", cost_variance_color);
    println!("  Effort Variance: {:.1} hours", comparison.effort_variance_hours);

    // Show project health
    let health_color = match comparison.summary.overall_health {
        crate::core::ProjectHealth::Green => "Green".green(),
        crate::core::ProjectHealth::Yellow => "Yellow".yellow(),
        crate::core::ProjectHealth::Red => "Red".red(),
    };
    println!("  Project Health: {}", health_color);

    // Show task changes
    if !comparison.task_changes.is_empty() {
        println!("\n{}", "Task Changes:".bold());
        println!("{:<20} {:<25} {:<15} {:<12} {:<12}",
            "Task ID", "Name", "Change Type", "Schedule Δ", "Cost Δ"
        );
        println!("{}", "─".repeat(84));

        for task in &comparison.task_changes {
            let variance_type_str = match task.variance_type {
                crate::core::VarianceType::NoChange => "No Change".green(),
                crate::core::VarianceType::ScheduleVariance => "Schedule".yellow(),
                crate::core::VarianceType::CostVariance => "Cost".yellow(),
                crate::core::VarianceType::ScopeChange => "Scope".yellow(),
                crate::core::VarianceType::TaskAdded => "Added".blue(),
                crate::core::VarianceType::TaskRemoved => "Removed".red(),
            };

            let name_truncated = if task.task_name.len() > 23 {
                format!("{}...", &task.task_name[..20])
            } else {
                task.task_name.clone()
            };

            println!("{:<20} {:<25} {} {:<12} {:<12.2}",
                &task.task_id[..20.min(task.task_id.len())],
                name_truncated,
                format!("{:<15}", variance_type_str),
                format!("{:+} days", task.schedule_variance_days),
                task.cost_variance
            );
        }
    }

    // Show milestone changes
    if !comparison.milestone_changes.is_empty() {
        println!("\n{}", "Milestone Changes:".bold());
        println!("{:<20} {:<25} {:<12} {:<12} {:<10}",
            "Milestone ID", "Name", "Baseline", "Current", "Status"
        );
        println!("{}", "─".repeat(79));

        for milestone in &comparison.milestone_changes {
            let status_color = match milestone.status {
                crate::core::MilestoneStatus::OnTrack => "On Track".green(),
                crate::core::MilestoneStatus::AtRisk => "At Risk".yellow(),
                crate::core::MilestoneStatus::Delayed => "Delayed".red(),
                crate::core::MilestoneStatus::Completed => "Complete".blue(),
                crate::core::MilestoneStatus::Cancelled => "Cancelled".red(),
            };

            let name_truncated = if milestone.milestone_name.len() > 23 {
                format!("{}...", &milestone.milestone_name[..20])
            } else {
                milestone.milestone_name.clone()
            };

            let current_date_str = if let Some(date) = milestone.current_date {
                date.format("%Y-%m-%d").to_string()
            } else {
                "N/A".to_string()
            };

            println!("{:<20} {:<25} {:<12} {:<12} {}",
                &milestone.milestone_id[..20.min(milestone.milestone_id.len())],
                name_truncated,
                milestone.baseline_date.format("%Y-%m-%d"),
                current_date_str,
                status_color
            );
        }
    }

    println!("\n{}", "Summary:".bold());
    println!("  Tasks Changed: {}", comparison.summary.total_tasks_changed);
    println!("  Milestones At Risk: {}", comparison.summary.total_milestones_at_risk);

    Ok(())
}

fn set_current_baseline_enhanced(baseline_manager: &BaselineManager) -> Result<()> {
    let baselines = baseline_manager.list_baselines()?;
    
    if baselines.is_empty() {
        super::print_info("No baselines available");
        return Ok(());
    }

    let baseline_list: Vec<String> = baselines.iter()
        .map(|b| {
            let current_marker = if b.is_current { " (Current)" } else { "" };
            format!("{} - {}{}", b.baseline_id, b.name, current_marker)
        })
        .collect();

    let selected = Select::new("Select baseline to set as current:", baseline_list)
        .with_vim_mode(true)
        .prompt()?;

    let baseline_id = selected.split(" - ").next().unwrap();

    match baseline_manager.set_current_baseline(baseline_id) {
        Ok(()) => super::print_success(&format!("Set '{}' as current baseline", baseline_id)),
        Err(e) => super::print_error(&format!("Failed to set current baseline: {}", e)),
    }

    Ok(())
}

fn show_baseline_metrics(baseline_manager: &BaselineManager) -> Result<()> {
    let baselines = baseline_manager.list_baselines()?;
    
    if baselines.is_empty() {
        super::print_info("No baselines available");
        return Ok(());
    }

    let baseline_list: Vec<String> = baselines.iter()
        .map(|b| format!("{} - {}", b.baseline_id, b.name))
        .collect();

    let selected = Select::new("Select baseline for metrics:", baseline_list)
        .with_vim_mode(true)
        .prompt()?;

    let baseline_id = selected.split(" - ").next().unwrap();

    match baseline_manager.generate_baseline_metrics(baseline_id) {
        Ok(metrics) => {
            println!("\n{}", "Baseline Metrics".bold());
            println!("{}", "─".repeat(80));
            println!("Baseline: {} ({})", metrics.name.yellow(), metrics.baseline_id);
            println!("Type: {}", metrics.baseline_type);
            println!("Created: {}", metrics.created_date.format("%Y-%m-%d %H:%M:%S"));
            println!();
            println!("Project Scope:");
            println!("  Tasks: {}", metrics.total_tasks);
            println!("  Milestones: {}", metrics.total_milestones);
            println!("  Resources: {}", metrics.total_resources);
            println!();
            println!("Schedule & Cost:");
            println!("  Duration: {} days", metrics.duration_days);
            println!("  Total Cost: {:.2}", metrics.total_cost);
            println!("  Total Effort: {:.1} hours", metrics.total_effort_hours);
            println!("  Avg Resource Utilization: {:.1} hours", metrics.avg_resource_utilization);
        }
        Err(e) => super::print_error(&format!("Failed to generate metrics: {}", e)),
    }

    Ok(())
}

fn archive_baseline(baseline_manager: &BaselineManager) -> Result<()> {
    let baselines = baseline_manager.list_baselines()?;
    
    if baselines.is_empty() {
        super::print_info("No baselines available");
        return Ok(());
    }

    // Filter out already archived baselines
    let active_baselines: Vec<&BaselineInfo> = baselines.iter()
        .filter(|b| b.baseline_type != BaselineType::Archived)
        .collect();

    if active_baselines.is_empty() {
        super::print_info("No active baselines to archive");
        return Ok(());
    }

    let baseline_list: Vec<String> = active_baselines.iter()
        .map(|b| format!("{} - {} ({})", b.baseline_id, b.name, b.baseline_type))
        .collect();

    let selected = Select::new("Select baseline to archive:", baseline_list)
        .with_vim_mode(true)
        .prompt()?;

    let baseline_id = selected.split(" - ").next().unwrap();

    if Confirm::new(&format!("Archive baseline '{}'? This will make it read-only.", baseline_id)).prompt()? {
        match baseline_manager.archive_baseline(baseline_id) {
            Ok(()) => super::print_success(&format!("Archived baseline '{}'", baseline_id)),
            Err(e) => super::print_error(&format!("Failed to archive baseline: {}", e)),
        }
    }

    Ok(())
}

fn delete_baseline_enhanced(baseline_manager: &BaselineManager) -> Result<()> {
    let baselines = baseline_manager.list_baselines()?;
    
    if baselines.is_empty() {
        super::print_info("No baselines to delete");
        return Ok(());
    }

    let baseline_list: Vec<String> = baselines.iter()
        .map(|b| {
            let current_marker = if b.is_current { " (Current)" } else { "" };
            format!("{} - {} ({}){}", b.baseline_id, b.name, b.baseline_type, current_marker)
        })
        .collect();

    let selected = Select::new("Select baseline to delete:", baseline_list)
        .with_vim_mode(true)
        .prompt()?;

    let baseline_id = selected.split(" - ").next().unwrap();

    if Confirm::new(&format!("PERMANENTLY delete baseline '{}'? This cannot be undone.", baseline_id)).prompt()? {
        match baseline_manager.delete_baseline(baseline_id) {
            Ok(()) => super::print_success(&format!("Deleted baseline '{}'", baseline_id)),
            Err(e) => super::print_error(&format!("Failed to delete baseline: {}", e)),
        }
    }

    Ok(())
}

// Progress Management Functions
fn manage_progress(project: &mut Project) -> Result<()> {
    loop {
        let choices = vec![
            "Create Progress Snapshot",
            "Update Task Progress",
            "View Progress Dashboard",
            "View Progress Snapshots",
            "Generate Earned Value Report",
            "View SPI/CPI Metrics",
            "Delete Progress Snapshot",
            "Back",
        ];

        let choice = match show_submenu("\nProgress Management:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Create Progress Snapshot" => create_progress_snapshot(project)?,
            "Update Task Progress" => update_task_progress(project)?,
            "View Progress Dashboard" => view_progress_dashboard(project)?,
            "View Progress Snapshots" => view_progress_snapshots(project)?,
            "Generate Earned Value Report" => generate_earned_value_report(project)?,
            "View SPI/CPI Metrics" => view_spi_cpi_metrics(project)?,
            "Delete Progress Snapshot" => delete_progress_snapshot(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn create_progress_snapshot(project: &mut Project) -> Result<()> {
    if project.tasks.is_empty() {
        super::print_info("No tasks available to create progress snapshot");
        return Ok(());
    }

    println!("\n{}", "Create Progress Snapshot".bold().cyan());
    println!("{}", "─".repeat(50));

    let snapshot_id = Text::new("Enter snapshot ID (e.g., 'snapshot_2024_01_15'):")
        .prompt()?;

    let status_date = DateSelect::new("Select status date:")
        .with_default(chrono::Utc::now().date_naive())
        .prompt()?;

    let recorded_by = Text::new("Enter your name:")
        .prompt()?;

    let mut snapshot = crate::core::ProgressSnapshot::new(
        project.name.clone(),
        status_date,
        recorded_by,
    );

    // Collect task progress for all tasks
    println!("\n{}", "Enter progress for each task:".bold());
    for (task_id, task) in &project.tasks {
        let task_uuid = uuid::Uuid::new_v4(); // Generate UUID for task progress
        let mut task_progress = crate::core::TaskProgress::new(
            task_uuid,
            task.name.clone(),
            snapshot.recorded_by.clone(),
        );

        println!("\n{}", format!("Task: {}", task.name).cyan());
        
        // Get task status
        let status_choices = vec![
            "Not Started",
            "In Progress", 
            "On Hold",
            "Completed",
            "Cancelled",
        ];
        let status_choice = Select::new("Task Status:", status_choices)
            .prompt()?;
        
        task_progress.status = match status_choice {
            "Not Started" => crate::core::TaskStatus::NotStarted,
            "In Progress" => crate::core::TaskStatus::InProgress,
            "On Hold" => crate::core::TaskStatus::OnHold,
            "Completed" => crate::core::TaskStatus::Completed,
            "Cancelled" => crate::core::TaskStatus::Cancelled,
            _ => crate::core::TaskStatus::NotStarted,
        };

        // Get percent complete
        let percent_complete = if task_progress.status == crate::core::TaskStatus::Completed {
            100.0
        } else if task_progress.status == crate::core::TaskStatus::NotStarted {
            0.0
        } else {
            CustomType::<f32>::new("Percent Complete (0-100):")
                .with_default(0.0)
                .with_validator(|val: &f32| {
                    if *val >= 0.0 && *val <= 100.0 {
                        Ok(inquire::validator::Validation::Valid)
                    } else {
                        Ok(inquire::validator::Validation::Invalid("Value must be between 0 and 100".into()))
                    }
                })
                .prompt()?
        };

        task_progress.percent_complete = percent_complete;

        // Get actual start date if task is started
        if task_progress.status != crate::core::TaskStatus::NotStarted {
            let has_start_date = Confirm::new("Has this task actually started?")
                .with_default(true)
                .prompt()?;
            
            if has_start_date {
                let start_date = DateSelect::new("Actual start date:")
                    .with_default(status_date)
                    .prompt()?;
                task_progress.actual_start = Some(start_date);
            }
        }

        // Get actual end date if task is completed
        if task_progress.status == crate::core::TaskStatus::Completed {
            let end_date = DateSelect::new("Actual completion date:")
                .with_default(status_date)
                .prompt()?;
            task_progress.actual_end = Some(end_date);
        }

        // Get actual effort hours
        let actual_effort = CustomType::<f32>::new("Actual effort hours spent:")
            .with_default(0.0)
            .with_validator(PositiveFloatValidator)
            .prompt()?;
        task_progress.actual_effort_hours = actual_effort;

        // Get actual cost
        let actual_cost = CustomType::<f32>::new("Actual cost incurred:")
            .with_default(0.0)
            .with_validator(PositiveFloatValidator)
            .prompt()?;
        task_progress.actual_cost = actual_cost;

        // Get remaining effort hours
        let remaining_effort = if task_progress.status == crate::core::TaskStatus::Completed {
            0.0
        } else {
            CustomType::<f32>::new("Remaining effort hours:")
                .with_default(0.0)
                .with_validator(PositiveFloatValidator)
                .prompt()?
        };
        task_progress.remaining_effort_hours = remaining_effort;

        // Get notes
        let notes = Text::new("Notes (optional):")
            .with_default("")
            .prompt()?;
        task_progress.notes = if notes.trim().is_empty() { None } else { Some(notes) };

        snapshot.add_task_progress(task_progress);
    }

    // Calculate overall project status
    let overall_completion = snapshot.calculate_overall_completion();
    snapshot.overall_status = if overall_completion >= 95.0 {
        crate::core::ProjectStatus::Green
    } else if overall_completion >= 75.0 {
        crate::core::ProjectStatus::Yellow
    } else {
        crate::core::ProjectStatus::Red
    };

    // Add snapshot to project
    project.record_progress_snapshot(snapshot_id.clone(), snapshot);

    println!("\n{}", "✓ Progress snapshot created successfully".green());
    println!("  Snapshot ID: {}", snapshot_id);
    println!("  Overall Completion: {:.1}%", overall_completion);
    
    Ok(())
}

fn view_progress_snapshots(project: &Project) -> Result<()> {
    if project.progress_snapshots.is_empty() {
        super::print_info("No progress snapshots defined");
        return Ok(());
    }

    println!("\n{}", "Progress Snapshots".bold());
    println!("{}", "─".repeat(80));

    for (id, _snapshot) in &project.progress_snapshots {
        println!("  • {}", id.yellow());
    }

    Ok(())
}

fn delete_progress_snapshot(project: &mut Project) -> Result<()> {
    if project.progress_snapshots.is_empty() {
        super::print_info("No progress snapshots to delete");
        return Ok(());
    }

    let snapshot_list: Vec<String> = project.progress_snapshots.keys().cloned().collect();

    let selected = Select::new("Select progress snapshot to delete:", snapshot_list)
        .with_vim_mode(true)
        .prompt()?;

    if Confirm::new(&format!("Delete progress snapshot '{}'?", selected)).prompt()? {
        project.progress_snapshots.shift_remove(&selected);
        super::print_success(&format!("Deleted progress snapshot '{}'", selected));
    }

    Ok(())
}

fn update_task_progress(project: &mut Project) -> Result<()> {
    if project.tasks.is_empty() {
        super::print_info("No tasks available to update progress");
        return Ok(());
    }

    if project.progress_snapshots.is_empty() {
        super::print_info("No progress snapshots available. Create a progress snapshot first.");
        return Ok(());
    }

    println!("\n{}", "Update Task Progress".bold().cyan());
    println!("{}", "─".repeat(50));

    // Select progress snapshot to update
    let snapshot_list: Vec<String> = project.progress_snapshots.keys().cloned().collect();
    let selected_snapshot = Select::new("Select progress snapshot to update:", snapshot_list)
        .prompt()?;

    let snapshot = project.progress_snapshots.get_mut(&selected_snapshot)
        .ok_or_else(|| anyhow::anyhow!("Snapshot not found"))?;

    // Select task to update
    let task_list: Vec<String> = snapshot.task_progress.keys().map(|uuid| uuid.to_string()).collect();
    let selected_task = Select::new("Select task to update:", task_list)
        .prompt()?;

    let task_uuid = uuid::Uuid::parse_str(&selected_task).map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
    if let Some(task_progress) = snapshot.task_progress.get_mut(&task_uuid) {
        println!("\n{}", format!("Updating Task: {}", task_progress.name).cyan());
        println!("Current Status: {}", task_progress.status.to_string().yellow());
        println!("Current Completion: {:.1}%", task_progress.percent_complete);

        // Update task status
        let status_choices = vec![
            "Not Started",
            "In Progress",
            "On Hold", 
            "Completed",
            "Cancelled",
        ];
        let status_choice = Select::new("New Task Status:", status_choices)
            .prompt()?;

        task_progress.status = match status_choice {
            "Not Started" => crate::core::TaskStatus::NotStarted,
            "In Progress" => crate::core::TaskStatus::InProgress,
            "On Hold" => crate::core::TaskStatus::OnHold,
            "Completed" => crate::core::TaskStatus::Completed,
            "Cancelled" => crate::core::TaskStatus::Cancelled,
            _ => crate::core::TaskStatus::NotStarted,
        };

        // Update percent complete
        let percent_complete = if task_progress.status == crate::core::TaskStatus::Completed {
            100.0
        } else if task_progress.status == crate::core::TaskStatus::NotStarted {
            0.0
        } else {
            CustomType::<f32>::new("Percent Complete (0-100):")
                .with_default(task_progress.percent_complete)
                .with_validator(|val: &f32| {
                    if *val >= 0.0 && *val <= 100.0 {
                        Ok(inquire::validator::Validation::Valid)
                    } else {
                        Ok(inquire::validator::Validation::Invalid("Value must be between 0 and 100".into()))
                    }
                })
                .prompt()?
        };

        // Update actual effort hours
        let actual_effort = CustomType::<f32>::new("Actual effort hours spent:")
            .with_default(task_progress.actual_effort_hours)
            .with_validator(PositiveFloatValidator)
            .prompt()?;

        // Update actual cost
        let actual_cost = CustomType::<f32>::new("Actual cost incurred:")
            .with_default(task_progress.actual_cost)
            .with_validator(PositiveFloatValidator)
            .prompt()?;

        // Update remaining effort hours
        let remaining_effort = if task_progress.status == crate::core::TaskStatus::Completed {
            0.0
        } else {
            CustomType::<f32>::new("Remaining effort hours:")
                .with_default(task_progress.remaining_effort_hours)
                .with_validator(PositiveFloatValidator)
                .prompt()?
        };

        // Update notes
        let current_notes = task_progress.notes.as_deref().unwrap_or("");
        let notes = Text::new("Notes:")
            .with_default(current_notes)
            .prompt()?;

        // Apply updates
        task_progress.update_progress(percent_complete, remaining_effort, Some(notes));
        task_progress.actual_effort_hours = actual_effort;
        task_progress.actual_cost = actual_cost;
        task_progress.updated_by = "User".to_string();

        println!("\n{}", "✓ Task progress updated successfully".green());
        println!("  Status: {}", task_progress.status.to_string().yellow());
        println!("  Completion: {:.1}%", task_progress.percent_complete);
    } else {
        super::print_error("Task not found in progress snapshot");
    }

    Ok(())
}

fn view_progress_dashboard(project: &Project) -> Result<()> {
    if project.progress_snapshots.is_empty() {
        super::print_info("No progress snapshots available");
        return Ok(());
    }

    println!("\n{}", "Progress Dashboard".bold().cyan());
    println!("{}", "─".repeat(80));

    // Get latest snapshot
    let latest_snapshot = project.progress_snapshots.values()
        .max_by_key(|s| s.status_date)
        .ok_or_else(|| anyhow::anyhow!("No snapshots found"))?;

    // Overall project status
    let overall_completion = latest_snapshot.calculate_overall_completion();
    let status_color = match latest_snapshot.overall_status {
        crate::core::ProjectStatus::Green => "green",
        crate::core::ProjectStatus::Yellow => "yellow",
        crate::core::ProjectStatus::Red => "red",
        _ => "white",
    };

    println!("\n{}", "Project Overview".bold());
    println!("  Status Date: {}", latest_snapshot.status_date.format("%Y-%m-%d"));
    println!("  Overall Status: {}", latest_snapshot.overall_status.to_string().color(status_color));
    println!("  Overall Completion: {:.1}%", overall_completion);

    // Task statistics
    let not_started = latest_snapshot.get_tasks_by_status(crate::core::TaskStatus::NotStarted).len();
    let in_progress = latest_snapshot.get_tasks_by_status(crate::core::TaskStatus::InProgress).len();
    let on_hold = latest_snapshot.get_tasks_by_status(crate::core::TaskStatus::OnHold).len();
    let completed = latest_snapshot.get_tasks_by_status(crate::core::TaskStatus::Completed).len();
    let cancelled = latest_snapshot.get_tasks_by_status(crate::core::TaskStatus::Cancelled).len();

    println!("\n{}", "Task Status Summary".bold());
    println!("  Not Started: {}", not_started.to_string().yellow());
    println!("  In Progress: {}", in_progress.to_string().cyan());
    println!("  On Hold: {}", on_hold.to_string().red());
    println!("  Completed: {}", completed.to_string().green());
    println!("  Cancelled: {}", cancelled.to_string().red());

    // Overdue tasks
    let overdue_tasks = latest_snapshot.get_overdue_tasks();
    if !overdue_tasks.is_empty() {
        println!("\n{}", "Overdue Tasks".bold().red());
        for task in overdue_tasks {
            println!("  • {} ({:.1}% complete)", task.name.red(), task.percent_complete);
        }
    }

    // Cost summary
    let total_actual_cost = latest_snapshot.calculate_total_actual_cost();
    println!("\n{}", "Cost Summary".bold());
    println!("  Total Actual Cost: {} {:.2}", project.currency, total_actual_cost);

    // Top tasks by completion
    let mut task_progress_vec: Vec<_> = latest_snapshot.task_progress.values().collect();
    task_progress_vec.sort_by(|a, b| b.percent_complete.partial_cmp(&a.percent_complete).unwrap());
    
    println!("\n{}", "Task Progress".bold());
    for (_i, task) in task_progress_vec.iter().take(10).enumerate() {
        let progress_bar = if task.percent_complete >= 100.0 {
            "█████████████████████".green()
        } else if task.percent_complete >= 75.0 {
            "██████████████████░░░".cyan()
        } else if task.percent_complete >= 50.0 {
            "███████████░░░░░░░░░░".yellow()
        } else if task.percent_complete >= 25.0 {
            "██████░░░░░░░░░░░░░░░".yellow()
        } else {
            "░░░░░░░░░░░░░░░░░░░░░".red()
        };
        
        let task_name = task.name.chars().take(30).collect::<String>();
        let padded_name = format!("{:<30}", task_name);
        println!("  {}: {} {:.1}%", 
            padded_name,
            progress_bar,
            task.percent_complete
        );
    }

    Ok(())
}

fn generate_earned_value_report(project: &Project) -> Result<()> {
    if project.progress_snapshots.is_empty() {
        super::print_info("No progress snapshots available");
        return Ok(());
    }

    if project.baselines.is_empty() {
        super::print_info("No baselines available. Create a baseline first.");
        return Ok(());
    }

    println!("\n{}", "Earned Value Report".bold().cyan());
    println!("{}", "─".repeat(80));

    // Select baseline and progress snapshot
    let baseline_list: Vec<String> = project.baselines.keys().cloned().collect();
    let selected_baseline = Select::new("Select baseline:", baseline_list)
        .prompt()?;

    let snapshot_list: Vec<String> = project.progress_snapshots.keys().cloned().collect();
    let selected_snapshot = Select::new("Select progress snapshot:", snapshot_list)
        .prompt()?;

    let baseline = project.baselines.get(&selected_baseline)
        .ok_or_else(|| anyhow::anyhow!("Baseline not found"))?;

    let progress = project.progress_snapshots.get(&selected_snapshot)
        .ok_or_else(|| anyhow::anyhow!("Progress snapshot not found"))?;

    // Calculate earned value metrics
    let ev_metrics = crate::core::EarnedValueMetrics::calculate(
        progress.status_date,
        project.name.clone(),
        baseline,
        progress,
    );

    // Display metrics
    println!("\n{}", "Earned Value Analysis".bold());
    println!("  Status Date: {}", ev_metrics.status_date.format("%Y-%m-%d"));
    println!("  Budget at Completion (BAC): {} {:.2}", project.currency, ev_metrics.budget_at_completion);
    println!("  Planned Value (PV): {} {:.2}", project.currency, ev_metrics.planned_value);
    println!("  Earned Value (EV): {} {:.2}", project.currency, ev_metrics.earned_value);
    println!("  Actual Cost (AC): {} {:.2}", project.currency, ev_metrics.actual_cost);

    println!("\n{}", "Variance Analysis".bold());
    let sv_color = if ev_metrics.schedule_variance >= 0.0 { "green" } else { "red" };
    let cv_color = if ev_metrics.cost_variance >= 0.0 { "green" } else { "red" };
    
    println!("  Schedule Variance (SV): {} {:.2}", 
        project.currency, 
        ev_metrics.schedule_variance.to_string().color(sv_color)
    );
    println!("  Cost Variance (CV): {} {:.2}", 
        project.currency, 
        ev_metrics.cost_variance.to_string().color(cv_color)
    );

    println!("\n{}", "Performance Indices".bold());
    let spi_color = if ev_metrics.schedule_performance_index >= 1.0 { "green" } else { "red" };
    let cpi_color = if ev_metrics.cost_performance_index >= 1.0 { "green" } else { "red" };
    
    println!("  Schedule Performance Index (SPI): {}", 
        format!("{:.3}", ev_metrics.schedule_performance_index).color(spi_color)
    );
    println!("  Cost Performance Index (CPI): {}", 
        format!("{:.3}", ev_metrics.cost_performance_index).color(cpi_color)
    );

    println!("\n{}", "Forecasting".bold());
    println!("  Estimate at Completion (EAC): {} {:.2}", project.currency, ev_metrics.estimate_at_completion);
    println!("  Estimate to Complete (ETC): {} {:.2}", project.currency, ev_metrics.estimate_to_complete);
    println!("  Variance at Completion (VAC): {} {:.2}", project.currency, ev_metrics.variance_at_completion);

    println!("\n{}", "Progress Indicators".bold());
    println!("  Percent Complete: {:.1}%", ev_metrics.percent_complete);
    println!("  Percent Spent: {:.1}%", ev_metrics.percent_spent);

    // Health indicators
    let schedule_health = ev_metrics.schedule_health();
    let cost_health = ev_metrics.cost_health();
    
    println!("\n{}", "Health Status".bold());
    println!("  Schedule Health: {}", schedule_health.to_string().color(match schedule_health {
        crate::core::ProjectStatus::Green => "green",
        crate::core::ProjectStatus::Yellow => "yellow",
        crate::core::ProjectStatus::Red => "red",
        _ => "white",
    }));
    println!("  Cost Health: {}", cost_health.to_string().color(match cost_health {
        crate::core::ProjectStatus::Green => "green",
        crate::core::ProjectStatus::Yellow => "yellow",
        crate::core::ProjectStatus::Red => "red",
        _ => "white",
    }));

    Ok(())
}

fn view_spi_cpi_metrics(project: &Project) -> Result<()> {
    if project.progress_snapshots.is_empty() {
        super::print_info("No progress snapshots available");
        return Ok(());
    }

    if project.baselines.is_empty() {
        super::print_info("No baselines available. Create a baseline first.");
        return Ok(());
    }

    println!("\n{}", "SPI/CPI Metrics Dashboard".bold().cyan());
    println!("{}", "─".repeat(80));

    // Select baseline
    let baseline_list: Vec<String> = project.baselines.keys().cloned().collect();
    let selected_baseline = Select::new("Select baseline:", baseline_list)
        .prompt()?;

    let baseline = project.baselines.get(&selected_baseline)
        .ok_or_else(|| anyhow::anyhow!("Baseline not found"))?;

    // Calculate metrics for all snapshots
    let mut metrics_data = Vec::new();
    for (snapshot_id, progress) in &project.progress_snapshots {
        let ev_metrics = crate::core::EarnedValueMetrics::calculate(
            progress.status_date,
            project.name.clone(),
            baseline,
            progress,
        );
        metrics_data.push((snapshot_id.clone(), ev_metrics));
    }

    // Sort by date
    metrics_data.sort_by(|a, b| a.1.status_date.cmp(&b.1.status_date));

    // Display trend table
    println!("\n{}", "SPI/CPI Trend Analysis".bold());
    println!("{:<15} {:<12} {:<8} {:<8} {:<10} {:<10}", 
        "Date", "Snapshot", "SPI", "CPI", "% Complete", "% Spent"
    );
    println!("{}", "─".repeat(80));

    for (snapshot_id, metrics) in &metrics_data {
        let spi_color = if metrics.schedule_performance_index >= 1.0 { "green" } else { "red" };
        let cpi_color = if metrics.cost_performance_index >= 1.0 { "green" } else { "red" };
        
        println!("{:<15} {:<12} {:<8} {:<8} {:<10} {:<10}",
            metrics.status_date.format("%Y-%m-%d"),
            snapshot_id.chars().take(12).collect::<String>(),
            format!("{:.3}", metrics.schedule_performance_index).color(spi_color),
            format!("{:.3}", metrics.cost_performance_index).color(cpi_color),
            format!("{:.1}%", metrics.percent_complete),
            format!("{:.1}%", metrics.percent_spent)
        );
    }

    // Show latest metrics in detail
    if let Some((_, latest_metrics)) = metrics_data.last() {
        println!("\n{}", "Latest Performance Analysis".bold());
        
        // SPI Analysis
        println!("\n{}", "Schedule Performance Index (SPI)".cyan());
        println!("  Current SPI: {:.3}", latest_metrics.schedule_performance_index);
        if latest_metrics.schedule_performance_index > 1.0 {
            println!("  Status: {} (Ahead of schedule)", "AHEAD".green());
        } else if latest_metrics.schedule_performance_index < 1.0 {
            println!("  Status: {} (Behind schedule)", "BEHIND".red());
        } else {
            println!("  Status: {} (On schedule)", "ON TRACK".green());
        }

        // CPI Analysis
        println!("\n{}", "Cost Performance Index (CPI)".cyan());
        println!("  Current CPI: {:.3}", latest_metrics.cost_performance_index);
        if latest_metrics.cost_performance_index > 1.0 {
            println!("  Status: {} (Under budget)", "UNDER BUDGET".green());
        } else if latest_metrics.cost_performance_index < 1.0 {
            println!("  Status: {} (Over budget)", "OVER BUDGET".red());
        } else {
            println!("  Status: {} (On budget)", "ON BUDGET".green());
        }

        // Trend analysis
        if metrics_data.len() > 1 {
            let previous_metrics = &metrics_data[metrics_data.len() - 2].1;
            let spi_trend = latest_metrics.schedule_performance_index - previous_metrics.schedule_performance_index;
            let cpi_trend = latest_metrics.cost_performance_index - previous_metrics.cost_performance_index;

            println!("\n{}", "Trend Analysis".cyan());
            println!("  SPI Trend: {:.3} ({})", spi_trend, 
                if spi_trend > 0.0 { "Improving".green() } else { "Declining".red() }
            );
            println!("  CPI Trend: {:.3} ({})", cpi_trend,
                if cpi_trend > 0.0 { "Improving".green() } else { "Declining".red() }
            );
        }
    }

    Ok(())
}

// Issues & Risks Management Functions
fn manage_issues_risks(project: &mut Project) -> Result<()> {
    loop {
        let choices = vec![
            "Manage Issues",
            "Manage Risks", 
            "Back",
        ];

        let choice = match show_submenu("\nIssues & Risks Management:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to main menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "Manage Issues" => manage_issues(project)?,
            "Manage Risks" => manage_risks(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn manage_issues(project: &mut Project) -> Result<()> {
    loop {
        let choices = vec![
            "View Issues",
            "Add New Issue",
            "Edit Issue",
            "Update Issue Status",
            "Add Comment",
            "Assign Issue",
            "View Issue Metrics",
            "View by Priority",
            "View by Status",
            "Back",
        ];

        let choice = match show_submenu("\nIssue Management:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to issues & risks menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "View Issues" => view_issues(project)?,
            "Add New Issue" => add_new_issue(project)?,
            "Edit Issue" => edit_issue(project)?,
            "Update Issue Status" => update_issue_status(project)?,
            "Add Comment" => add_issue_comment(project)?,
            "Assign Issue" => assign_issue(project)?,
            "View Issue Metrics" => view_issue_metrics(project)?,
            "View by Priority" => view_issues_by_priority(project)?,
            "View by Status" => view_issues_by_status(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn view_issues(project: &Project) -> Result<()> {
    if project.issue_registry.issues.is_empty() {
        super::print_info("No issues reported");
        return Ok(());
    }

    println!("\n{}", "Issue Registry".bold());
    println!("{}", "─".repeat(80));

    for (id, issue) in &project.issue_registry.issues {
        let priority_color = match issue.priority {
            IssuePriority::Critical => "red",
            IssuePriority::High => "yellow",
            IssuePriority::Medium => "blue",
            IssuePriority::Low => "green",
        };

        let severity_marker = match issue.severity {
            crate::core::IssueSeverity::Blocker => " [BLOCKER]".red(),
            crate::core::IssueSeverity::Major => " [MAJOR]".yellow(),
            crate::core::IssueSeverity::Minor => " [MINOR]".blue(),
            crate::core::IssueSeverity::Trivial => "".normal(),
        };

        let status_color = match issue.status {
            IssueStatus::Open => "red",
            IssueStatus::InProgress => "yellow",
            IssueStatus::PendingReview => "blue",
            IssueStatus::Resolved => "green",
            IssueStatus::Closed => "bright_black",
            IssueStatus::Deferred => "magenta",
            IssueStatus::Duplicate => "bright_black",
            IssueStatus::CannotReproduce => "bright_black",
        };

        println!("  • {} - {} | {} | {}{}",
            id.to_string().yellow(),
            issue.title,
            issue.priority.to_string().color(priority_color),
            issue.status.to_string().color(status_color),
            severity_marker
        );

        if let Some(ref assigned_to) = issue.assigned_to {
            println!("    Assigned to: {}", assigned_to);
        }

        if let Some(due_date) = issue.due_date {
            println!("    Due: {}", due_date.format("%Y-%m-%d"));
        }

        if !issue.related_tasks.is_empty() {
            println!("    Related tasks: {}", issue.related_tasks.join(", "));
        }
    }

    // Issue selection for details
    loop {
        let mut choices: Vec<_> = project.issue_registry.issues.keys()
            .map(|id| format!("{} - {}", id, project.issue_registry.issues[id].title))
            .collect();
        choices.push("← Back".to_string());

        let choice = Select::new("\nSelect issue to view details:", choices)
            .prompt()?;

        if choice == "← Back" {
            break;
        }

        let issue_id = choice.split(" - ").next().unwrap();
        if let Ok(uuid) = uuid::Uuid::parse_str(issue_id) {
            if let Some(issue) = project.issue_registry.issues.get(&uuid) {
                view_issue_details(issue_id, issue)?;
            }
        }
    }

    Ok(())
}

fn view_issue_details(issue_id: &str, issue: &crate::core::Issue) -> Result<()> {
    println!("\n{}", format!("Issue Details: {}", issue_id).bold());
    println!("{}", "─".repeat(80));

    println!("Title: {}", issue.title.bold());
    println!("Description: {}", issue.description);
    println!("Priority: {}", issue.priority);
    println!("Severity: {}", issue.severity);
    println!("Status: {}", issue.status);
    println!("Category: {}", issue.category);

    if let Some(ref assigned_to) = issue.assigned_to {
        println!("Assigned to: {}", assigned_to);
    }

    println!("Reported by: {} on {}", issue.reported_by, issue.reported_date.format("%Y-%m-%d"));

    if let Some(due_date) = issue.due_date {
        println!("Due date: {}", due_date.format("%Y-%m-%d"));
    }

    if let Some(resolution_date) = issue.resolution_date {
        println!("Resolved: {}", resolution_date.format("%Y-%m-%d %H:%M"));
    }

    if let Some(ref resolution) = issue.resolution_description {
        println!("Resolution: {}", resolution);
    }

    if let Some(estimated_effort) = issue.estimated_effort_hours {
        println!("Estimated effort: {:.1} hours", estimated_effort);
    }

    if let Some(actual_effort) = issue.actual_effort_hours {
        println!("Actual effort: {:.1} hours", actual_effort);
    }

    if !issue.related_tasks.is_empty() {
        println!("Related tasks: {}", issue.related_tasks.join(", "));
    }

    if !issue.related_risks.is_empty() {
        println!("Related risks: {}", issue.related_risks.join(", "));
    }

    if !issue.comments.is_empty() {
        println!("\n{}", "Recent Comments:".bold());
        for comment in issue.comments.iter().rev().take(3) {
            let resolution_marker = if comment.is_resolution { " [RESOLUTION]".green() } else { "".normal() };
            println!("  • {} ({}): {}{}",
                comment.author,
                comment.created_date.format("%Y-%m-%d"),
                comment.content,
                resolution_marker
            );
        }
    }

    println!("\n{}", "Press Enter to continue...".normal().dimmed());
    let _ = Text::new("").prompt();

    Ok(())
}

fn add_new_issue(project: &mut Project) -> Result<()> {
    println!("\n{}", "Add New Issue".bold());
    println!("{}", "─".repeat(50));

    let title = match prompt_text("Issue title:")? {
        Some(title) => title,
        None => {
            println!("Issue creation cancelled");
            return Ok(());
        }
    };

    let description = match prompt_text("Issue description:")? {
        Some(description) => description,
        None => {
            println!("Issue creation cancelled");
            return Ok(());
        }
    };

    let reported_by = match prompt_text("Reported by:")? {
        Some(reported_by) => reported_by,
        None => {
            println!("Issue creation cancelled");
            return Ok(());
        }
    };

    let priorities = vec![
        IssuePriority::Critical,
        IssuePriority::High,
        IssuePriority::Medium,
        IssuePriority::Low,
    ];

    let priority = match prompt_select("Priority:", priorities)? {
        Some(priority) => priority,
        None => {
            println!("Issue creation cancelled");
            return Ok(());
        }
    };

    let severities = vec![
        crate::core::IssueSeverity::Blocker,
        crate::core::IssueSeverity::Major,
        crate::core::IssueSeverity::Minor,
        crate::core::IssueSeverity::Trivial,
    ];

    let severity = match prompt_select("Severity:", severities)? {
        Some(severity) => severity,
        None => {
            println!("Issue creation cancelled");
            return Ok(());
        }
    };

    let categories = vec![
        IssueCategory::Technical,
        IssueCategory::Process,
        IssueCategory::Resource,
        IssueCategory::Scope,
        IssueCategory::Quality,
        IssueCategory::Communication,
        IssueCategory::External,
        IssueCategory::Requirements,
        IssueCategory::Environment,
    ];

    let category = match prompt_select("Category:", categories)? {
        Some(category) => category,
        None => {
            println!("Issue creation cancelled");
            return Ok(());
        }
    };

    let issue_id = project.issue_registry.create_issue(title, description, reported_by);

    if let Some(issue) = project.issue_registry.get_issue_mut(&issue_id) {
        issue.priority = priority;
        issue.severity = severity;
        issue.category = category;

        // Optional fields
        if Confirm::new("Set due date?").prompt()? {
            let due_date = DateSelect::new("Due date:")
                .prompt()?;
            issue.due_date = Some(due_date);
        }

        if Confirm::new("Assign to someone?").prompt()? {
            let assigned_to = Text::new("Assigned to:")
                .prompt()?;
            issue.assigned_to = Some(assigned_to);
        }

        if Confirm::new("Add estimated effort?").prompt()? {
            let effort = CustomType::<f32>::new("Estimated effort (hours):")
                .prompt()?;
            issue.estimated_effort_hours = Some(effort);
        }

        // Add related tasks
        if !project.tasks.is_empty() && Confirm::new("Link to tasks?").prompt()? {
            let task_choices: Vec<_> = project.tasks.iter()
                .map(|(id, task)| format!("{} - {}", id, task.name))
                .collect();

            let selected_tasks = MultiSelect::new("Select related tasks:", task_choices)
                .prompt()?;

            for task_choice in selected_tasks {
                let task_id = task_choice.split(" - ").next().unwrap();
                issue.related_tasks.push(task_id.to_string());
            }
        }

        super::print_success(&format!("Issue '{}' created successfully", issue_id));
    }

    Ok(())
}

fn edit_issue(project: &mut Project) -> Result<()> {
    if project.issue_registry.issues.is_empty() {
        super::print_info("No issues to edit");
        return Ok(());
    }

    let issue_choices: Vec<_> = project.issue_registry.issues.iter()
        .map(|(id, issue)| format!("{} - {}", id, issue.title))
        .collect();

    let selected = match prompt_select("Select issue to edit:", issue_choices)? {
        Some(selected) => selected,
        None => {
            println!("Issue editing cancelled");
            return Ok(());
        }
    };

    let issue_id = selected.split(" - ").next().unwrap();

    loop {
        let choices = vec![
            "Edit Title",
            "Edit Description",
            "Edit Priority",
            "Edit Severity",
            "Edit Category",
            "Edit Due Date",
            "Edit Estimated Effort",
            "Done",
        ];

        let choice = Select::new(&format!("Edit Issue {}:", issue_id), choices)
            .prompt()?;

        match choice {
            "Edit Title" => {
                if let Ok(uuid) = uuid::Uuid::parse_str(issue_id) {
                    if let Some(issue) = project.issue_registry.get_issue_mut(&uuid) {
                        let new_title = Text::new("New title:")
                            .with_default(&issue.title)
                            .prompt()?;
                        issue.title = new_title;
                    }
                }
                super::print_success("Title updated");
            },
            "Edit Description" => {
                // TODO: Fix UUID parsing for interactive CLI
                super::print_info("Edit Description feature temporarily disabled during UUID migration");
            },
            "Edit Priority" => {
                // TODO: Fix UUID parsing for interactive CLI  
                super::print_info("Edit Priority feature temporarily disabled during UUID migration");
            },
            "Done" => break,
            _ => {
                super::print_info("Feature not yet implemented");
            }
        }
    }

    Ok(())
}

fn update_issue_status(project: &mut Project) -> Result<()> {
    // TODO: Reimplement with UUID support
    super::print_info("Issue status update temporarily disabled during UUID migration");
    Ok(())
    
    /*
    if project.issue_registry.issues.is_empty() {
        super::print_info("No issues to update");
        return Ok(());
    }

    let issue_choices: Vec<_> = project.issue_registry.issues.iter()
        .map(|(id, issue)| format!("{} - {} ({})", id, issue.title, issue.status))
        .collect();

    let selected = Select::new("Select issue to update:", issue_choices)
        .prompt()?;

    let issue_id = selected.split(" - ").next().unwrap();

    let statuses = vec![
        IssueStatus::Open,
        IssueStatus::InProgress,
        IssueStatus::PendingReview,
        IssueStatus::Resolved,
        IssueStatus::Closed,
        IssueStatus::Deferred,
        IssueStatus::Duplicate,
        IssueStatus::CannotReproduce,
    ];

    let new_status = Select::new("New status:", statuses)
        .prompt()?;

    if let Some(issue) = project.issue_registry.get_issue_mut(issue_id) {
        issue.status = new_status;

        if new_status == IssueStatus::Resolved || new_status == IssueStatus::Closed {
            if Confirm::new("Add resolution description?").prompt()? {
                let resolution = Text::new("Resolution description:")
                    .prompt()?;
                issue.resolution_description = Some(resolution);
            }

            if Confirm::new("Record actual effort spent?").prompt()? {
                let effort = CustomType::<f32>::new("Actual effort (hours):")
                    .prompt()?;
                issue.actual_effort_hours = Some(effort);
            }

            issue.resolution_date = Some(chrono::Utc::now().naive_utc());
        }

        super::print_success("Issue status updated");
    }

    Ok(())
    */
}

fn add_issue_comment(project: &mut Project) -> Result<()> {
    // TODO: Reimplement with UUID support
    super::print_info("Issue comment feature temporarily disabled during UUID migration");
    Ok(())
    
    /*
    if project.issue_registry.issues.is_empty() {
        super::print_info("No issues available for comments");
        return Ok(());
    }

    let issue_choices: Vec<_> = project.issue_registry.issues.iter()
        .map(|(id, issue)| format!("{} - {}", id, issue.title))
        .collect();

    let selected = Select::new("Select issue to comment on:", issue_choices)
        .prompt()?;

    let issue_id = selected.split(" - ").next().unwrap();

    let author = Text::new("Author:")
        .with_default("Project Manager")
        .prompt()?;

    let content = Text::new("Comment:")
        .prompt()?;

    let is_resolution = Confirm::new("Is this a resolution comment?")
        .with_default(false)
        .prompt()?;

    if let Some(issue) = project.issue_registry.get_issue_mut(issue_id) {
        let comment_id = issue.add_comment(author, content);
        super::print_success(&format!("Comment '{}' added", comment_id));
    }

    Ok(())
    */
}

fn assign_issue(project: &mut Project) -> Result<()> {
    // TODO: Reimplement with UUID support
    super::print_info("Issue assignment feature temporarily disabled during UUID migration");
    Ok(())
    
    /*
    if project.issue_registry.issues.is_empty() {
        super::print_info("No issues to assign");
        return Ok(());
    }

    let issue_choices: Vec<_> = project.issue_registry.issues.iter()
        .map(|(id, issue)| format!("{} - {}", id, issue.title))
        .collect();

    let selected = Select::new("Select issue to assign:", issue_choices)
        .prompt()?;

    let issue_id = selected.split(" - ").next().unwrap();

    let assigned_to = Text::new("Assign to:")
        .prompt()?;

    if let Some(issue) = project.issue_registry.get_issue_mut(issue_id) {
        issue.assigned_to = Some(assigned_to);
        if issue.status == IssueStatus::Open {
            issue.status = IssueStatus::InProgress;
        }
        super::print_success("Issue assigned successfully");
    }

    Ok(())
    */
}

fn view_issue_metrics(project: &Project) -> Result<()> {
    let metrics = project.issue_registry.calculate_metrics();

    println!("\n{}", "Issue Metrics".bold());
    println!("{}", "─".repeat(50));

    println!("Total Issues: {}", metrics.total_issues);
    println!("Open Issues: {}", metrics.open_issues);
    println!("Escalated Issues: {}", metrics.escalated_issues);
    println!("Average Resolution Time: {:.1} hours", metrics.average_resolution_time_hours);

    println!("\n{}", "Issues by Priority:".bold());
    for (priority, count) in &metrics.issues_by_priority {
        println!("  {}: {}", priority, count);
    }

    println!("\n{}", "Issues by Status:".bold());
    for (status, count) in &metrics.issues_by_status {
        println!("  {}: {}", status, count);
    }

    println!("\n{}", "Issues by Category:".bold());
    for (category, count) in &metrics.issues_by_category {
        println!("  {}: {}", category, count);
    }

    if metrics.overdue_issues > 0 {
        println!("\n{}: {}", "Overdue Issues".red(), metrics.overdue_issues);
    }

    Ok(())
}

fn view_issues_by_priority(project: &Project) -> Result<()> {
    let priorities = vec![
        IssuePriority::Critical,
        IssuePriority::High,
        IssuePriority::Medium,
        IssuePriority::Low,
    ];

    let selected_priority = Select::new("Select priority:", priorities)
        .prompt()?;

    let issues = project.issue_registry.get_issues_by_priority(selected_priority);

    if issues.is_empty() {
        super::print_info(&format!("No {} priority issues found", selected_priority));
        return Ok(());
    }

    println!("\n{} Priority Issues", selected_priority.to_string().bold());
    println!("{}", "─".repeat(50));

    for issue in issues {
        println!("  • {} - {} | {}",
            issue.issue_id.to_string().yellow(),
            issue.title,
            issue.status
        );
    }

    Ok(())
}

fn view_issues_by_status(project: &Project) -> Result<()> {
    let statuses = vec![
        IssueStatus::Open,
        IssueStatus::InProgress,
        IssueStatus::PendingReview,
        IssueStatus::Resolved,
        IssueStatus::Closed,
        IssueStatus::Deferred,
    ];

    let selected_status = Select::new("Select status:", statuses)
        .prompt()?;

    let issues = project.issue_registry.get_issues_by_status(selected_status);

    if issues.is_empty() {
        super::print_info(&format!("No {} issues found", selected_status));
        return Ok(());
    }

    println!("\n{} Issues", selected_status.to_string().bold());
    println!("{}", "─".repeat(50));

    for issue in issues {
        println!("  • {} - {} | {}",
            issue.issue_id.to_string().yellow(),
            issue.title,
            issue.priority
        );
    }

    Ok(())
}

fn manage_risks(project: &mut Project) -> Result<()> {
    
    loop {
        let choices = vec![
            "View Risks",
            "Add New Risk",
            "Edit Risk",
            "Add Mitigation Action",
            "Update Risk Status",
            "View Risk Metrics",
            "View by Category",
            "View High Priority Risks",
            "Back",
        ];

        let choice = match show_submenu("\nRisk Management:", choices)? {
            MenuResult::Selection(choice) => choice,
            MenuResult::GoBack => break, // ESC pressed - go back to issues & risks menu
            MenuResult::Exit => return Ok(()), // Let main menu handle exit
        };

        match choice {
            "View Risks" => view_risks(project)?,
            "Add New Risk" => add_new_risk(project)?,
            "Edit Risk" => edit_risk(project)?,
            "Add Mitigation Action" => add_mitigation_action(project)?,
            "Update Risk Status" => update_risk_status(project)?,
            "View Risk Metrics" => view_risk_metrics(project)?,
            "View by Category" => view_risks_by_category(project)?,
            "View High Priority Risks" => view_high_priority_risks(project)?,
            "Back" => break,
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn view_risks(project: &Project) -> Result<()> {
    if project.risk_registry.risks.is_empty() {
        super::print_info("No risks registered");
        return Ok(());
    }

    println!("\n{}", "Risk Registry".bold());
    println!("{}", "─".repeat(80));

    for (id, risk) in &project.risk_registry.risks {
        let priority_marker = if risk.is_high_priority() { " (HIGH PRIORITY)".red() } else { "".normal() };
        let status_color = match risk.status {
            crate::core::RiskStatus::Identified => "yellow",
            crate::core::RiskStatus::Analyzing => "blue",
            crate::core::RiskStatus::Planning => "cyan",
            crate::core::RiskStatus::Mitigating => "green",
            crate::core::RiskStatus::Monitoring => "magenta",
            crate::core::RiskStatus::Realized => "red",
            crate::core::RiskStatus::Closed => "bright_black",
            crate::core::RiskStatus::Transferred => "bright_blue",
        };

        println!("  • {} - {} | {} | Score: {:.1} | {}{}",
            id.to_string().yellow(),
            risk.title,
            risk.category.to_string(),
            risk.risk_score,
            risk.status.to_string().color(status_color),
            priority_marker
        );

        if let Some(ref owner) = risk.owner {
            println!("    Owner: {}", owner);
        }

        if !risk.mitigation_actions.is_empty() {
            let completed_actions = risk.mitigation_actions.iter()
                .filter(|a| a.status == ActionStatus::Completed)
                .count();
            println!("    Mitigation Actions: {}/{} completed", completed_actions, risk.mitigation_actions.len());
        }

        if let Some(cost_impact) = risk.cost_impact {
            println!("    Cost Impact: {:.2}", cost_impact);
        }

        if let Some(schedule_impact) = risk.schedule_impact_days {
            println!("    Schedule Impact: {} days", schedule_impact);
        }
    }

    // Risk selection for details
    loop {
        let mut choices: Vec<_> = project.risk_registry.risks.keys()
            .map(|id| format!("{} - {}", id, project.risk_registry.risks[id].title))
            .collect();
        choices.push("← Back".to_string());

        let choice = Select::new("\nSelect risk to view details:", choices)
            .prompt()?;

        if choice == "← Back" {
            break;
        }

        // TODO: Fix UUID parsing
        super::print_info("Risk management features temporarily disabled during UUID migration");
        break;
    }
    
    Ok(())
}

fn view_risk_details(risk_id: &str, risk: &crate::core::Risk) -> Result<()> {
    println!("\n{}", format!("Risk Details: {}", risk_id).bold());
    println!("{}", "─".repeat(80));

    println!("Title: {}", risk.title.bold());
    println!("Description: {}", risk.description);
    println!("Category: {}", risk.category.to_string());
    println!("Probability: {} ({:.1}%)", risk.probability, risk.probability.numeric_value() * 100.0);
    println!("Impact: {} ({:.1})", risk.impact, risk.impact.numeric_value());
    println!("Risk Score: {:.1}", risk.risk_score);
    println!("Status: {}", risk.status);

    if let Some(ref owner) = risk.owner {
        println!("Owner: {}", owner);
    }

    println!("Identified by: {} on {}", risk.identified_by, risk.identified_date.format("%Y-%m-%d"));

    if let Some(last_reviewed) = risk.last_reviewed {
        println!("Last reviewed: {}", last_reviewed.format("%Y-%m-%d"));
    }

    if let Some(cost_impact) = risk.cost_impact {
        println!("Cost Impact: {:.2}", cost_impact);
    }

    if let Some(schedule_impact) = risk.schedule_impact_days {
        println!("Schedule Impact: {} days", schedule_impact);
    }

    if !risk.related_tasks.is_empty() {
        println!("Related Tasks: {}", risk.related_tasks.join(", "));
    }

    if !risk.mitigation_actions.is_empty() {
        println!("\n{}", "Mitigation Actions:".bold());
        for action in &risk.mitigation_actions {
            let status_color = match action.status {
                ActionStatus::NotStarted => "yellow",
                ActionStatus::InProgress => "blue",
                ActionStatus::Completed => "green",
                ActionStatus::OnHold => "magenta",
                ActionStatus::Cancelled => "red",
            };
            println!("  • {} - {} | Due: {} | Status: {}",
                action.action_id,
                action.description,
                action.due_date.format("%Y-%m-%d"),
                action.status.to_string().color(status_color)
            );
        }
    }

    if !risk.comments.is_empty() {
        println!("\n{}", "Recent Comments:".bold());
        for comment in risk.comments.iter().rev().take(3) {
            println!("  • {} ({}): {}",
                comment.author,
                comment.created_date.format("%Y-%m-%d"),
                comment.content
            );
        }
    }

    println!("\n{}", "Press Enter to continue...".normal().dimmed());
    let _ = Text::new("").prompt();

    Ok(())
}

fn add_new_risk(project: &mut Project) -> Result<()> {

    println!("\n{}", "Add New Risk".bold());
    println!("{}", "─".repeat(50));

    let title = match prompt_text("Risk title:")? {
        Some(title) => title,
        None => {
            println!("Risk creation cancelled");
            return Ok(());
        }
    };

    let description = match prompt_text("Risk description:")? {
        Some(description) => description,
        None => {
            println!("Risk creation cancelled");
            return Ok(());
        }
    };

    let identified_by = match prompt_text("Identified by:")? {
        Some(identified_by) => identified_by,
        None => {
            println!("Risk creation cancelled");
            return Ok(());
        }
    };

    let categories = vec![
        RiskCategory::Technical,
        RiskCategory::Schedule,
        RiskCategory::Cost,
        RiskCategory::Resource,
        RiskCategory::External,
        RiskCategory::Organizational,
        RiskCategory::Quality,
        RiskCategory::Security,
        RiskCategory::Environmental,
        RiskCategory::Market,
    ];

    let category = match prompt_select("Risk category:", categories)? {
        Some(category) => category,
        None => {
            println!("Risk creation cancelled");
            return Ok(());
        }
    };

    let probabilities = vec![
        RiskProbability::VeryLow,
        RiskProbability::Low,
        RiskProbability::Medium,
        RiskProbability::High,
        RiskProbability::VeryHigh,
        RiskProbability::Certain,
    ];

    let probability = match prompt_select("Probability:", probabilities)? {
        Some(probability) => probability,
        None => {
            println!("Risk creation cancelled");
            return Ok(());
        }
    };

    let impacts = vec![
        RiskImpact::Negligible,
        RiskImpact::Minor,
        RiskImpact::Moderate,
        RiskImpact::Major,
        RiskImpact::Severe,
    ];

    let impact = match prompt_select("Impact:", impacts)? {
        Some(impact) => impact,
        None => {
            println!("Risk creation cancelled");
            return Ok(());
        }
    };

    let risk_id = project.risk_registry.create_risk(title, description, identified_by);

    if let Some(risk) = project.risk_registry.get_risk_mut(&risk_id) {
        *risk = risk.clone()
            .with_category(category)
            .with_probability_and_impact(probability, impact);

        // Optional fields
        if Confirm::new("Add cost impact estimate?").prompt()? {
            let cost_impact = CustomType::<f32>::new("Cost impact:")
                .prompt()?;
            risk.cost_impact = Some(cost_impact);
        }

        if Confirm::new("Add schedule impact estimate?").prompt()? {
            let schedule_impact = CustomType::<i32>::new("Schedule impact (days):")
                .prompt()?;
            risk.schedule_impact_days = Some(schedule_impact);
        }

        if Confirm::new("Assign owner?").prompt()? {
            let owner = Text::new("Owner:")
                .prompt()?;
            risk.assign_owner(owner);
        }

        // Add related tasks
        if !project.tasks.is_empty() && Confirm::new("Link to tasks?").prompt()? {
            let task_choices: Vec<_> = project.tasks.iter()
                .map(|(id, task)| format!("{} - {}", id, task.name))
                .collect();

            let selected_tasks = MultiSelect::new("Select related tasks:", task_choices)
                .prompt()?;

            for task_choice in selected_tasks {
                let task_id = task_choice.split(" - ").next().unwrap();
                risk.related_tasks.push(task_id.to_string());
            }
        }

        super::print_success(&format!("Risk '{}' created successfully", risk_id));
    }

    Ok(())
}

fn edit_risk(project: &mut Project) -> Result<()> {
    // TODO: Reimplement with UUID support
    super::print_info("Risk editing feature temporarily disabled during UUID migration");
    Ok(())
}
fn add_mitigation_action(project: &mut Project) -> Result<()> {
    // TODO: Reimplement with UUID support
    super::print_info("Mitigation action feature temporarily disabled during UUID migration");
    Ok(())
}

fn update_risk_status(project: &mut Project) -> Result<()> {
    // TODO: Reimplement with UUID support
    super::print_info("Risk status update feature temporarily disabled during UUID migration");
    Ok(())
}
fn view_risk_metrics(project: &Project) -> Result<()> {
    let metrics = project.risk_registry.calculate_metrics();

    println!("\n{}", "Risk Metrics".bold());
    println!("{}", "─".repeat(50));

    println!("Total Risks: {}", metrics.total_risks);
    println!("Active Risks: {}", metrics.active_risks);
    println!("Average Risk Score: {:.1}", metrics.average_risk_score);
    println!("Highest Risk Score: {:.1}", metrics.highest_risk_score);
    println!("Risks Requiring Action: {}", metrics.risks_requiring_action);
    println!("Overdue Mitigation Actions: {}", metrics.overdue_mitigation_actions);
    println!("Mitigation Effectiveness: {:.1}%", metrics.mitigation_effectiveness);

    println!("\n{}", "Risks by Status:".bold());
    for (status, count) in &metrics.risks_by_status {
        println!("  {}: {}", status, count);
    }

    println!("\n{}", "Risks by Category:".bold());
    for (category, count) in &metrics.risks_by_category {
        println!("  {}: {}", category.to_string(), count);
    }

    println!("\n{}", "Risks by Level:".bold());
    for (level, count) in &metrics.risks_by_level {
        println!("  {}: {}", level, count);
    }

    Ok(())
}

fn view_risks_by_category(project: &Project) -> Result<()> {
    let categories = vec![
        crate::core::RiskCategory::Technical,
        crate::core::RiskCategory::Schedule,
        crate::core::RiskCategory::Cost,
        crate::core::RiskCategory::Resource,
        crate::core::RiskCategory::External,
        crate::core::RiskCategory::Organizational,
        crate::core::RiskCategory::Quality,
        crate::core::RiskCategory::Security,
        crate::core::RiskCategory::Environmental,
        crate::core::RiskCategory::Market,
    ];

    let selected_category = Select::new("Select category:", categories)
        .prompt()?;

    let risks = project.risk_registry.get_risks_by_category(selected_category);

    if risks.is_empty() {
        super::print_info(&format!("No risks found in {} category", selected_category.to_string()));
        return Ok(());
    }

    println!("\n{} Risks", selected_category.to_string().bold());
    println!("{}", "─".repeat(50));

    for risk in risks {
        println!("  • {} - {} | Score: {:.1} | {}",
            risk.risk_id.to_string().yellow(),
            risk.title,
            risk.risk_score,
            risk.status
        );
    }

    Ok(())
}

fn view_high_priority_risks(project: &Project) -> Result<()> {
    let high_priority_risks = project.risk_registry.get_high_priority_risks();

    if high_priority_risks.is_empty() {
        super::print_info("No high priority risks found");
        return Ok(());
    }

    println!("\n{}", "High Priority Risks".bold());
    println!("{}", "─".repeat(50));

    for risk in high_priority_risks {
        println!("  • {} - {} | Score: {:.1} | {}",
            risk.risk_id.to_string().yellow(),
            risk.title,
            risk.risk_score,
            risk.status.to_string().red()
        );
    }

    Ok(())
}

// Critical Path and Slack Analysis Functions
fn show_critical_path_analysis(schedule: &Schedule) -> Result<()> {
    println!("\n{}", "Critical Path Analysis".bold());
    println!("{}", "─".repeat(80));

    if schedule.critical_path.is_empty() {
        super::print_info("No critical path found");
        return Ok(());
    }

    println!("Critical Path Sequence: {}", schedule.critical_path.join(" → "));
    println!("Critical Path Length: {} tasks", schedule.critical_path.len());

    let critical_duration: i64 = schedule.tasks.iter()
        .filter(|(_, task)| task.is_critical)
        .map(|(_, task)| (task.end_date - task.start_date).num_days() + 1)
        .sum();

    println!("Critical Path Duration: {} days", critical_duration);

    println!("\n{}", "Critical Tasks:".bold());
    for task_id in &schedule.critical_path {
        if let Some(task) = schedule.tasks.get(task_id) {
            println!("  • {} - {} ({} → {})",
                task_id.yellow(),
                task.name,
                task.start_date.format("%Y-%m-%d"),
                task.end_date.format("%Y-%m-%d")
            );
            println!("    Effort: {:.1}h | Cost: {:.2} | Slack: {} days",
                task.effort,
                task.cost,
                task.slack
            );
        }
    }

    // Show critical milestones
    let critical_milestones: Vec<_> = schedule.milestones.iter()
        .filter(|(_, milestone)| milestone.is_critical)
        .collect();

    if !critical_milestones.is_empty() {
        println!("\n{}", "Critical Milestones:".bold());
        for (id, milestone) in critical_milestones {
            println!("  • {} - {} ({})",
                id.to_string().yellow(),
                milestone.name,
                milestone.date.format("%Y-%m-%d")
            );
        }
    }

    Ok(())
}

fn show_task_slack_analysis(schedule: &Schedule) -> Result<()> {
    println!("\n{}", "Task Slack Analysis".bold());
    println!("{}", "─".repeat(80));

    if schedule.tasks.is_empty() {
        super::print_info("No tasks found");
        return Ok(());
    }

    // Sort tasks by slack (ascending - critical tasks first)
    let mut tasks_by_slack: Vec<_> = schedule.tasks.iter().collect();
    tasks_by_slack.sort_by_key(|(_, task)| task.slack);

    println!("{:<20} {:<30} {:<10} {:<12} {:<12} {:<8}",
        "Task ID", "Name", "Duration", "Start", "End", "Slack"
    );
    println!("{}", "─".repeat(98));

    for (task_id, task) in &tasks_by_slack {
        let duration = (task.end_date - task.start_date).num_days() + 1;
        let slack_color = if task.slack == 0 {
            format!("{}", task.slack).red()
        } else if task.slack <= 2 {
            format!("{}", task.slack).yellow()
        } else {
            format!("{}", task.slack).green()
        };

        let name_truncated = if task.name.len() > 28 {
            format!("{}...", &task.name[..25])
        } else {
            task.name.clone()
        };

        println!("{:<20} {:<30} {:<10} {:<12} {:<12} {}",
            task_id,
            name_truncated,
            format!("{}d", duration),
            task.start_date.format("%Y-%m-%d"),
            task.end_date.format("%Y-%m-%d"),
            slack_color
        );
    }

    // Summary statistics
    let critical_tasks = tasks_by_slack.iter().filter(|(_, task)| task.slack == 0).count();
    let near_critical_tasks = tasks_by_slack.iter().filter(|(_, task)| task.slack > 0 && task.slack <= 2).count();
    let flexible_tasks = tasks_by_slack.iter().filter(|(_, task)| task.slack > 2).count();

    println!("\n{}", "Slack Summary:".bold());
    println!("  • Critical Tasks (0 slack): {} tasks", critical_tasks.to_string().red());
    println!("  • Near-Critical Tasks (1-2 days slack): {} tasks", near_critical_tasks.to_string().yellow());
    println!("  • Flexible Tasks (>2 days slack): {} tasks", flexible_tasks.to_string().green());

    if critical_tasks > 0 {
        println!("\n{}", "⚠️  Warning: Critical tasks have no slack - any delay will impact project completion!".red());
    }

    if near_critical_tasks > 0 {
        println!("{}", "⚠️  Caution: Near-critical tasks need close monitoring.".yellow());
    }

    Ok(())
}
