use crate::data::*;
use crate::repository::ProjectRepository;
use crate::scheduling::ProjectScheduler;
use crate::task_editor::{PMEntityEditor, EditOptions};
use tessera_core::{ProjectContext, Result, Id};
use inquire::{Select, Text, Confirm, CustomType, DateSelect};
use inquire::validator::Validation;
use chrono::{Utc, TimeZone};

pub struct ProjectCommands {
    repository: ProjectRepository,
    project_context: ProjectContext,
}

impl ProjectCommands {
    pub fn new(project_context: ProjectContext) -> Result<Self> {
        let pm_dir = project_context.module_path("pm");
        let repository = ProjectRepository::load_from_directory(&pm_dir)?;
        
        Ok(Self {
            repository,
            project_context,
        })
    }
    
    pub async fn add_task_interactive(&mut self) -> Result<()> {
        let name = Text::new("Task name:")
            .with_help_message("Enter a name for the task")
            .prompt()?;
        
        let description = Text::new("Description:")
            .with_help_message("Describe what needs to be done")
            .prompt()?;
        
        // Select task type first
        let task_types = vec![
            ("Effort Driven", TaskType::EffortDriven),
            ("Fixed Duration", TaskType::FixedDuration),
            ("Fixed Work", TaskType::FixedWork),
            ("Milestone", TaskType::Milestone),
        ];
        
        let type_names: Vec<&str> = task_types.iter().map(|(name, _)| *name).collect();
        let selected_type_name = Select::new("Task type:", type_names)
            .with_help_message("Effort Driven: effort fixed, duration calculated | Fixed Duration: duration fixed, effort calculated | Fixed Work: work units fixed | Milestone: zero duration")
            .prompt()?;
        
        let task_type = task_types.iter()
            .find(|(name, _)| *name == selected_type_name)
            .map(|(_, t)| *t)
            .unwrap_or(TaskType::EffortDriven);
        
        // Select work type
        let work_types = vec![
            "Design",
            "Analysis",
            "Testing",
            "Documentation",
            "Review",
            "Manufacturing",
            "Other",
        ];
        
        let work_type_str = Select::new("Work type:", work_types).prompt()?;
        let work_type = match work_type_str {
            "Design" => WorkType::Design,
            "Analysis" => WorkType::Analysis,
            "Testing" => WorkType::Testing,
            "Documentation" => WorkType::Documentation,
            "Review" => WorkType::Review,
            "Manufacturing" => WorkType::Manufacturing,
            _ => {
                let other_name = Text::new("Other work type:").prompt()?;
                WorkType::Other(other_name)
            }
        };
        
        // Create task with selected type
        let mut task = Task::with_type(name, description, work_type, task_type);
        
        // Configure effort/duration/work based on task type
        match task_type {
            TaskType::EffortDriven => {
                let effort = CustomType::<f64>::new("Estimated effort (hours):")
                    .with_default(8.0)
                    .with_help_message("Duration will be calculated based on resource assignments")
                    .prompt()?;
                task.estimated_hours = effort;
            }
            TaskType::FixedDuration => {
                let duration = CustomType::<f64>::new("Duration (working days):")
                    .with_default(1.0)
                    .with_help_message("Effort will be calculated based on resource assignments")
                    .prompt()?;
                task.duration_days = Some(duration);
            }
            TaskType::FixedWork => {
                let work_units = CustomType::<f64>::new("Work units:")
                    .with_default(8.0)
                    .with_help_message("Both effort and duration will be calculated based on resource assignments")
                    .prompt()?;
                task.work_units = Some(work_units);
                task.estimated_hours = work_units;
            }
            TaskType::Milestone => {
                // Milestones have zero effort and duration
                println!("Milestone created with zero effort and duration");
            }
        }
        
        // Set priority
        let priority_options = vec!["Low", "Medium", "High", "Critical"];
        let priority_str = Select::new("Priority:", priority_options).prompt()?;
        let priority = match priority_str {
            "Low" => TaskPriority::Low,
            "Medium" => TaskPriority::Medium,
            "High" => TaskPriority::High,
            "Critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        };
        task.priority = priority;
        
        // Add resource assignments
        if !self.repository.get_resources().is_empty() {
            let assign_resources = Confirm::new("Assign resources to this task?")
                .with_default(false)
                .prompt()?;
            
            if assign_resources {
                let resources = self.repository.get_resources().clone();
                Self::add_resource_assignments_static(&mut task, &resources)?;
            }
        }
        
        // Add dependencies
        if !self.repository.get_tasks().is_empty() {
            let add_deps = Confirm::new("Add dependencies to other tasks?")
                .with_default(false)
                .prompt()?;
            
            if add_deps {
                let existing_tasks = self.repository.get_tasks().clone();
                Self::add_task_dependencies_static(&mut task, &existing_tasks)?;
            }
        }
        
        self.repository.add_task(task.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Task '{}' added successfully!", task.name);
        println!("Task Type: {}", task.task_type);
        
        // Show calculated values based on task type
        match task.task_type {
            TaskType::EffortDriven => {
                if let Some(duration) = task.calculate_effective_duration() {
                    println!("Calculated Duration: {:.1} days (based on resource assignments)", duration);
                }
            }
            TaskType::FixedDuration => {
                println!("Calculated Effort: {:.1} hours (based on resource assignments)", task.calculate_effective_effort());
            }
            TaskType::FixedWork => {
                if let Some(duration) = task.calculate_effective_duration() {
                    println!("Calculated Duration: {:.1} days", duration);
                }
                println!("Calculated Effort: {:.1} hours", task.calculate_effective_effort());
            }
            TaskType::Milestone => {}
        }
        
        println!("ID: {}", task.id);
        
        Ok(())
    }
    
    /// Helper method to add resource assignments during task creation
    fn add_resource_assignments_static(task: &mut Task, resources: &[Resource]) -> Result<()> {
        loop {
            let available_resources: Vec<_> = resources.iter()
                .filter(|r| !task.assigned_resources.iter().any(|a| a.resource_id == r.id))
                .collect();
            
            if available_resources.is_empty() {
                println!("All resources have been assigned");
                break;
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
            println!("✓ Resource assigned: {} at {:.1}%", selected_resource.name, allocation);
            
            let continue_adding = Confirm::new("Add another resource?")
                .with_default(false)
                .prompt()?;
            
            if !continue_adding {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Helper method to add task dependencies during task creation
    fn add_task_dependencies_static(task: &mut Task, existing_tasks: &[Task]) -> Result<()> {
        loop {
            let available_tasks: Vec<_> = existing_tasks.iter()
                .filter(|t| t.id != task.id && !task.dependencies.iter().any(|d| d.predecessor_id == t.id))
                .collect();
            
            if available_tasks.is_empty() {
                println!("No more tasks available as dependencies");
                break;
            }
            
            let task_options: Vec<String> = available_tasks.iter()
                .map(|t| format!("{} - {}", t.name, t.status))
                .collect();
            
            let selected = Select::new("Select predecessor task:", task_options.clone()).prompt()?;
            let selected_index = task_options.iter().position(|x| x == &selected).unwrap();
            let predecessor = available_tasks[selected_index];
            
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
                predecessor_id: predecessor.id,
                dependency_type,
                lag_days,
                description: None,
            };
            
            task.dependencies.push(dependency);
            println!("✓ Dependency added: {} -> {} ({}, lag: {:.1} days)", 
                predecessor.name, task.name, dependency_type, lag_days);
            
            let continue_adding = Confirm::new("Add another dependency?")
                .with_default(false)
                .prompt()?;
            
            if !continue_adding {
                break;
            }
        }
        
        Ok(())
    }

    pub async fn delete_task_interactive(&mut self) -> Result<()> {
        let tasks = self.repository.get_tasks().to_vec();
        if tasks.is_empty() {
            println!("No tasks found.");
            return Ok(());
        }

        // Create options for task selection
        let task_options: Vec<String> = tasks.iter()
            .map(|t| format!("{}: {} ({})", t.id, t.name, t.status))
            .collect();

        let selected = Select::new("Select task to delete:", task_options)
            .with_help_message("Use arrow keys to select the task you want to delete")
            .prompt()?;

        // Extract task ID from selection
        let task_id = selected.split(':').next()
            .and_then(|id_str| Id::parse(id_str).ok())
            .ok_or_else(|| tessera_core::DesignTrackError::Validation("Invalid task selection".to_string()))?;

        let task = tasks.iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| tessera_core::DesignTrackError::NotFound("Task not found".to_string()))?;

        // Check for dependencies
        let dependent_tasks: Vec<&Task> = tasks.iter()
            .filter(|t| t.dependencies.iter().any(|dep| dep.predecessor_id == task_id))
            .collect();

        if !dependent_tasks.is_empty() {
            println!("\n⚠️  Warning: This task has dependent tasks:");
            for dep_task in &dependent_tasks {
                println!("  - {} ({})", dep_task.name, dep_task.id);
            }
            println!("Deleting this task will remove these dependencies.");
        }

        // Confirmation prompt
        let confirm_message = format!(
            "Are you sure you want to delete task '{}'?\nThis action cannot be undone.",
            task.name
        );
        
        let confirmed = Confirm::new(&confirm_message)
            .with_default(false)
            .prompt()?;

        if !confirmed {
            println!("Task deletion cancelled.");
            return Ok(());
        }

        let task_name = task.name.clone();

        // Remove the task
        self.repository.remove_task(task_id)?;

        // Remove dependencies from other tasks that reference this task
        let mut updated_tasks = Vec::new();
        for mut other_task in tasks.iter().cloned() {
            if other_task.id != task_id {
                let original_dep_count = other_task.dependencies.len();
                other_task.dependencies.retain(|dep| dep.predecessor_id != task_id);
                if other_task.dependencies.len() != original_dep_count {
                    other_task.updated = Utc::now();
                    updated_tasks.push(other_task);
                }
            }
        }

        // Update tasks that had dependencies removed
        for updated_task in updated_tasks {
            self.repository.update_task(updated_task)?;
        }

        // Save changes
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;

        println!("✓ Task '{}' deleted successfully!", task_name);
        
        Ok(())
    }
    
    pub async fn add_resource_interactive(&mut self) -> Result<()> {
        let name = Text::new("Resource name:")
            .prompt()?;
        
        let role = Text::new("Role:")
            .with_help_message("e.g., Engineer, Designer, Manager")
            .prompt()?;
        
        let email = Text::new("Email (optional):")
            .prompt()?;
        
        let hourly_rate_str = Text::new("Hourly rate (optional):")
            .with_help_message("Leave blank if not applicable")
            .prompt()?;
        
        let availability_str = Text::new("Availability percentage:")
            .with_default("100.0")
            .prompt()?;
        let availability: f64 = availability_str.parse().unwrap_or(100.0);
        
        let mut resource = Resource::new(name, role);
        if !email.is_empty() {
            resource.email = Some(email);
        }
        if !hourly_rate_str.is_empty() {
            if let Ok(rate) = hourly_rate_str.parse::<f64>() {
                resource.hourly_rate = Some(rate);
            }
        }
        resource.availability_percentage = availability.clamp(0.0, 100.0);
        
        self.repository.add_resource(resource.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Resource '{}' added successfully!", resource.name);
        println!("ID: {}", resource.id);
        
        Ok(())
    }

    pub async fn delete_resource_interactive(&mut self) -> Result<()> {
        let resources = self.repository.get_resources().to_vec();
        if resources.is_empty() {
            println!("No resources found.");
            return Ok(());
        }

        // Create options for resource selection
        let resource_options: Vec<String> = resources.iter()
            .map(|r| format!("{}: {} ({})", r.id, r.name, r.role))
            .collect();

        let selected = Select::new("Select resource to delete:", resource_options)
            .with_help_message("Use arrow keys to select the resource you want to delete")
            .prompt()?;

        // Extract resource ID from selection
        let resource_id = selected.split(':').next()
            .and_then(|id_str| Id::parse(id_str).ok())
            .ok_or_else(|| tessera_core::DesignTrackError::Validation("Invalid resource selection".to_string()))?;

        let resource = resources.iter()
            .find(|r| r.id == resource_id)
            .ok_or_else(|| tessera_core::DesignTrackError::NotFound("Resource not found".to_string()))?;

        // Check for tasks assigned to this resource
        let tasks = self.repository.get_tasks().to_vec();
        let assigned_tasks: Vec<&Task> = tasks.iter()
            .filter(|t| t.assigned_resources.iter().any(|res| res.resource_id == resource_id))
            .collect();

        if !assigned_tasks.is_empty() {
            println!("\n⚠️  Warning: This resource is assigned to tasks:");
            for task in &assigned_tasks {
                println!("  - {} ({})", task.name, task.id);
            }
            println!("Deleting this resource will remove these assignments.");
        }

        // Confirmation prompt
        let confirm_message = format!(
            "Are you sure you want to delete resource '{}'?\nThis action cannot be undone.",
            resource.name
        );
        
        let confirmed = Confirm::new(&confirm_message)
            .with_default(false)
            .prompt()?;

        if !confirmed {
            println!("Resource deletion cancelled.");
            return Ok(());
        }

        let resource_name = resource.name.clone();

        // Remove the resource
        self.repository.remove_resource(resource_id)?;

        // Remove assignments from tasks
        let mut updated_tasks = Vec::new();
        for mut task in tasks.iter().cloned() {
            let original_assignment_count = task.assigned_resources.len();
            task.assigned_resources.retain(|res| res.resource_id != resource_id);
            if task.assigned_resources.len() != original_assignment_count {
                task.updated = Utc::now();
                updated_tasks.push(task);
            }
        }

        // Update tasks that had assignments removed
        for updated_task in updated_tasks {
            self.repository.update_task(updated_task)?;
        }

        // Save changes
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;

        println!("✓ Resource '{}' deleted successfully!", resource_name);
        
        Ok(())
    }
    
    pub async fn add_milestone_interactive(&mut self) -> Result<()> {
        let name = Text::new("Milestone name:")
            .prompt()?;
        
        let description = Text::new("Description:")
            .prompt()?;
        
        // Use date picker for target date
        let target_date_naive = DateSelect::new("Target date:")
            .with_help_message("Select the target date for this milestone")
            .with_default(chrono::Local::now().date_naive())
            .prompt()?;
        
        let target_date_utc = Utc.from_utc_datetime(&target_date_naive.and_hms_opt(12, 0, 0).unwrap());
        
        let milestone = Milestone::new(name, description, target_date_utc);
        
        self.repository.add_milestone(milestone.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Milestone '{}' added successfully!", milestone.name);
        println!("ID: {}", milestone.id);
        println!("Target Date: {}", milestone.target_date.format("%Y-%m-%d"));
        
        Ok(())
    }

    pub async fn delete_milestone_interactive(&mut self) -> Result<()> {
        let milestones = self.repository.get_milestones().to_vec();
        if milestones.is_empty() {
            println!("No milestones found.");
            return Ok(());
        }

        // Create options for milestone selection
        let milestone_options: Vec<String> = milestones.iter()
            .map(|m| format!("{}: {} ({})", m.id, m.name, m.target_date.format("%Y-%m-%d")))
            .collect();

        let selected = Select::new("Select milestone to delete:", milestone_options)
            .with_help_message("Use arrow keys to select the milestone you want to delete")
            .prompt()?;

        // Extract milestone ID from selection
        let milestone_id = selected.split(':').next()
            .and_then(|id_str| Id::parse(id_str).ok())
            .ok_or_else(|| tessera_core::DesignTrackError::Validation("Invalid milestone selection".to_string()))?;

        let milestone = milestones.iter()
            .find(|m| m.id == milestone_id)
            .ok_or_else(|| tessera_core::DesignTrackError::NotFound("Milestone not found".to_string()))?;

        // Check for tasks dependent on this milestone
        let tasks = self.repository.get_tasks().to_vec();
        let dependent_tasks: Vec<&Task> = tasks.iter()
            .filter(|t| t.dependencies.iter().any(|dep| dep.predecessor_id == milestone_id))
            .collect();

        if !dependent_tasks.is_empty() {
            println!("\n⚠️  Warning: This milestone has dependent tasks:");
            for task in &dependent_tasks {
                println!("  - {} ({})", task.name, task.id);
            }
            println!("Deleting this milestone will remove these dependencies.");
        }

        // Confirmation prompt
        let confirm_message = format!(
            "Are you sure you want to delete milestone '{}'?\nThis action cannot be undone.",
            milestone.name
        );
        
        let confirmed = Confirm::new(&confirm_message)
            .with_default(false)
            .prompt()?;

        if !confirmed {
            println!("Milestone deletion cancelled.");
            return Ok(());
        }

        let milestone_name = milestone.name.clone();

        // Remove the milestone
        self.repository.remove_milestone(milestone_id)?;

        // Remove dependencies from tasks that reference this milestone
        let mut updated_tasks = Vec::new();
        for mut task in tasks.iter().cloned() {
            let original_dep_count = task.dependencies.len();
            task.dependencies.retain(|dep| dep.predecessor_id != milestone_id);
            if task.dependencies.len() != original_dep_count {
                task.updated = Utc::now();
                updated_tasks.push(task);
            }
        }

        // Update tasks that had dependencies removed
        for updated_task in updated_tasks {
            self.repository.update_task(updated_task)?;
        }

        // Save changes
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;

        println!("✓ Milestone '{}' deleted successfully!", milestone_name);
        
        Ok(())
    }
    
    pub fn compute_schedule(&mut self) -> Result<()> {
        let tasks = self.repository.get_tasks();
        let milestones = self.repository.get_milestones();
        let resources = self.repository.get_resources();
        
        if tasks.is_empty() {
            println!("No tasks found. Add tasks first.");
            return Ok(());
        }
        
        let scheduler = ProjectScheduler::default();
        let project_start = Utc::now();
        
        println!("Computing project schedule...");
        let schedule = scheduler.compute_schedule(tasks, milestones, resources, project_start)?;
        
        self.repository.add_schedule(schedule.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("\nSchedule Results:");
        println!("================");
        println!("Project Start: {}", schedule.project_start.format("%Y-%m-%d"));
        println!("Project End: {}", schedule.project_end.format("%Y-%m-%d"));
        println!("Total Duration: {} days", schedule.total_duration_days);
        println!("Critical Path Tasks: {}", schedule.critical_path.len());
        
        println!("\nCritical Path:");
        for &task_id in &schedule.critical_path {
            if let Some(task) = self.repository.find_task_by_id(task_id) {
                println!("  - {} ({})", task.name, task.description);
                if let Some(schedule_info) = schedule.task_schedule.get(&task_id) {
                    println!("    Duration: {} days, Start: {}, Finish: {}",
                        (schedule_info.earliest_finish - schedule_info.earliest_start).num_days(),
                        schedule_info.earliest_start.format("%Y-%m-%d"),
                        schedule_info.earliest_finish.format("%Y-%m-%d")
                    );
                }
            } else if let Some(milestone) = self.repository.find_milestone_by_id(task_id) {
                println!("  - {} [MILESTONE] ({})", milestone.name, milestone.description);
                println!("    Target Date: {}", milestone.target_date.format("%Y-%m-%d"));
            }
        }
        
        println!("\nTask Schedule Summary:");
        for (task_id, schedule_info) in &schedule.task_schedule {
            if let Some(task) = self.repository.find_task_by_id(*task_id) {
                let critical_marker = if schedule_info.is_critical { " [CRITICAL]" } else { "" };
                println!("  {} ({}) - Start: {}, Finish: {}, Total Float: {} days, Free Float: {} days{}",
                         task.name,
                         task.task_type,
                         schedule_info.earliest_start.format("%Y-%m-%d"),
                         schedule_info.earliest_finish.format("%Y-%m-%d"),
                         schedule_info.slack_days,
                         schedule_info.free_float_days,
                         critical_marker);
            }
        }

        // Show milestone schedule
        if !schedule.milestone_schedule.is_empty() {
            println!("\nMilestone Schedule:");
            for (milestone_id, milestone_info) in &schedule.milestone_schedule {
                if let Some(milestone) = self.repository.find_milestone_by_id(*milestone_id) {
                    let critical_marker = if milestone_info.is_critical { " [CRITICAL]" } else { "" };
                    let status_marker = if milestone_info.earliest_date > milestone_info.target_date {
                        " [AT RISK]"
                    } else {
                        ""
                    };
                    println!("  {} - Target: {}, Earliest: {}, Slack: {} days{}{}",
                             milestone.name,
                             milestone_info.target_date.format("%Y-%m-%d"),
                             milestone_info.earliest_date.format("%Y-%m-%d"),
                             milestone_info.slack_days,
                             critical_marker,
                             status_marker);
                }
            }
        }
        
        Ok(())
    }
    
    pub fn list_tasks(&self) -> Result<()> {
        let tasks = self.repository.get_tasks();
        
        if tasks.is_empty() {
            println!("No tasks found");
            return Ok(());
        }
        
        println!("Tasks:");
        for (i, task) in tasks.iter().enumerate() {
            let status_symbol = match task.status {
                TaskStatus::NotStarted => "○",
                TaskStatus::InProgress => "◐",
                TaskStatus::OnHold => "⏸",
                TaskStatus::Completed => "●",
                TaskStatus::Cancelled => "✗",
            };
            
            let priority_symbol = match task.priority {
                TaskPriority::Low => "▼",
                TaskPriority::Medium => "◆",
                TaskPriority::High => "▲",
                TaskPriority::Critical => "🔥",
            };
            
            println!("{}. {} {} {} - {} ({:.1}h)",
                     i + 1, status_symbol, priority_symbol, task.name, task.description, task.estimated_hours);
            
            if !task.dependencies.is_empty() {
                println!("   Dependencies:");
                for dep in &task.dependencies {
                    let dep_type_str = match dep.dependency_type {
                        DependencyType::FinishToStart => "FS",
                        DependencyType::StartToStart => "SS", 
                        DependencyType::FinishToFinish => "FF",
                        DependencyType::StartToFinish => "SF",
                    };
                    
                    // Try to find the dependency name from tasks and milestones
                    let dep_name = self.repository.find_task_by_id(dep.predecessor_id)
                        .map(|t| format!("{} (Task)", t.name))
                        .or_else(|| self.repository.find_milestone_by_id(dep.predecessor_id)
                                 .map(|m| format!("{} (Milestone)", m.name)))
                        .unwrap_or_else(|| format!("Unknown ({})", dep.predecessor_id));
                    
                    let lag_str = if dep.lag_days != 0.0 {
                        if dep.lag_days > 0.0 {
                            format!(" +{:.1}d lag", dep.lag_days)
                        } else {
                            format!(" {:.1}d lead", dep.lag_days)
                        }
                    } else {
                        String::new()
                    };
                    
                    println!("     → {} [{}]{}", dep_name, dep_type_str, lag_str);
                }
            }
            
            if task.progress_percentage > 0.0 {
                println!("   Progress: {:.1}%", task.progress_percentage);
            }

            // Show cost information if resources are assigned
            if !task.assigned_resources.is_empty() {
                let task_cost = self.repository.calculate_task_cost(task);
                if task_cost.estimated_cost > 0.0 {
                    println!("   Cost: ${:.2} estimated, ${:.2} actual", 
                             task_cost.estimated_cost, task_cost.actual_cost);
                }
            }
            
            println!("   ID: {}", task.id);
            println!();
        }
        
        Ok(())
    }
    
    pub fn show_dashboard(&self) -> Result<()> {
        let health = self.repository.get_project_health();
        
        println!("Project Management Dashboard");
        println!("===========================");
        println!("Total Tasks: {}", health.total_tasks);
        println!("Completed Tasks: {}", health.completed_tasks);
        println!("Completion: {:.1}%", health.completion_percentage);
        println!("Effort-Weighted Completion: {:.1}%", self.repository.get_effort_weighted_completion());
        println!("Overdue Tasks: {}", health.overdue_tasks);
        println!("Overdue Milestones: {}", health.overdue_milestones);
        
        let resources = self.repository.get_resources();
        println!("Resources: {}", resources.len());
        
        let milestones = self.repository.get_milestones();
        println!("Milestones: {}", milestones.len());

        // Cost information
        let project_cost = self.repository.calculate_project_cost();
        println!("\nCost Analysis:");
        println!("  Estimated Cost: ${:.2}", project_cost.total_estimated_cost);
        println!("  Actual Cost: ${:.2}", project_cost.total_actual_cost);
        println!("  Cost Variance: ${:.2} ({:.1}%)", 
                 project_cost.cost_variance, 
                 project_cost.cost_variance_percentage);
        
        if let Some(schedule) = self.repository.get_latest_schedule() {
            println!("\nLatest Schedule:");
            println!("  Generated: {}", schedule.generated.format("%Y-%m-%d %H:%M"));
            println!("  Project Duration: {} days", schedule.total_duration_days);
            println!("  Critical Path: {} tasks", schedule.critical_path.len());
        }
        
        // Show task status breakdown
        let not_started = self.repository.get_tasks_by_status(&TaskStatus::NotStarted).len();
        let in_progress = self.repository.get_tasks_by_status(&TaskStatus::InProgress).len();
        let completed = self.repository.get_tasks_by_status(&TaskStatus::Completed).len();
        let on_hold = self.repository.get_tasks_by_status(&TaskStatus::OnHold).len();
        
        println!("\nTask Status Breakdown:");
        println!("  Not Started: {}", not_started);
        println!("  In Progress: {}", in_progress);
        println!("  Completed: {}", completed);
        println!("  On Hold: {}", on_hold);
        
        Ok(())
    }

    pub fn show_cost_analysis(&self) -> Result<()> {
        let project_cost = self.repository.calculate_project_cost();
        
        println!("Project Cost Analysis");
        println!("====================");
        println!("Total Estimated Cost: ${:.2}", project_cost.total_estimated_cost);
        println!("Total Actual Cost: ${:.2}", project_cost.total_actual_cost);
        println!("Cost Variance: ${:.2} ({:.1}%)", 
                 project_cost.cost_variance, 
                 project_cost.cost_variance_percentage);
        
        println!("\nTask Cost Breakdown:");
        println!("-------------------");
        
        for task_cost in &project_cost.task_costs {
            if task_cost.estimated_cost > 0.0 || task_cost.actual_cost > 0.0 {
                println!("Task: {} ({})", task_cost.task_name, task_cost.task_id);
                println!("  Estimated: ${:.2}", task_cost.estimated_cost);
                println!("  Actual: ${:.2}", task_cost.actual_cost);
                println!("  Variance: ${:.2}", task_cost.cost_variance);
                
                println!("  Resource Breakdown:");
                for resource_cost in &task_cost.resource_costs {
                    println!("    {} - ${:.2}/hr: {:.1}h est, {:.1}h actual (${:.2} est, ${:.2} actual)",
                             resource_cost.resource_name,
                             resource_cost.hourly_rate,
                             resource_cost.estimated_hours,
                             resource_cost.actual_hours,
                             resource_cost.estimated_cost,
                             resource_cost.actual_cost);
                }
                println!();
            }
        }
        
        Ok(())
    }

    /// Edit task interactive with comprehensive functionality
    pub async fn edit_task_interactive(&mut self) -> Result<()> {
        let tasks = self.repository.get_tasks();
        
        if tasks.is_empty() {
            println!("No tasks found. Add tasks first.");
            return Ok(());
        }

        let task_options: Vec<String> = tasks.iter()
            .map(|t| format!("{} - {} ({})", t.name, t.task_type, t.status))
            .collect();

        let selected = Select::new("Select task to edit:", task_options.clone()).prompt()?;
        let selected_index = task_options.iter().position(|x| x == &selected).unwrap();
        let task_id = tasks[selected_index].id;

        PMEntityEditor::edit_task_interactive(&mut self.repository, task_id, EditOptions::default())?;

        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;

        Ok(())
    }

    /// Edit resource interactive
    pub async fn edit_resource_interactive(&mut self) -> Result<()> {
        let resources = self.repository.get_resources();
        
        if resources.is_empty() {
            println!("No resources found. Add resources first.");
            return Ok(());
        }

        let resource_options: Vec<String> = resources.iter()
            .map(|r| format!("{} - {}", r.name, r.role))
            .collect();

        let selected = Select::new("Select resource to edit:", resource_options.clone()).prompt()?;
        let selected_index = resource_options.iter().position(|x| x == &selected).unwrap();
        let resource_id = resources[selected_index].id;

        PMEntityEditor::edit_resource_interactive(&mut self.repository, resource_id)?;

        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;

        Ok(())
    }

    /// Edit milestone interactive
    pub async fn edit_milestone_interactive(&mut self) -> Result<()> {
        let milestones = self.repository.get_milestones();
        
        if milestones.is_empty() {
            println!("No milestones found. Add milestones first.");
            return Ok(());
        }

        let milestone_options: Vec<String> = milestones.iter()
            .map(|m| format!("{} - {}", m.name, m.target_date.format("%Y-%m-%d")))
            .collect();

        let selected = Select::new("Select milestone to edit:", milestone_options.clone()).prompt()?;
        let selected_index = milestone_options.iter().position(|x| x == &selected).unwrap();
        let milestone_id = milestones[selected_index].id;

        PMEntityEditor::edit_milestone_interactive(&mut self.repository, milestone_id)?;

        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;

        Ok(())
    }

    /// List entities with selection for editing
    pub async fn list_and_edit_interactive(&mut self) -> Result<()> {
        let choices = vec![
            "Edit Task",
            "Edit Resource", 
            "Edit Milestone",
            "Back",
        ];

        let choice = Select::new("What would you like to edit?", choices).prompt()?;

        match choice {
            "Edit Task" => self.edit_task_interactive().await?,
            "Edit Resource" => self.edit_resource_interactive().await?,
            "Edit Milestone" => self.edit_milestone_interactive().await?,
            "Back" => return Ok(()),
            _ => {}
        }

        Ok(())
    }
}