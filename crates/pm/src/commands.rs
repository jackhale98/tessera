use crate::data::*;
use crate::repository::ProjectRepository;
use crate::scheduling::ProjectScheduler;
use crate::task_editor::{PMEntityEditor, EditOptions};
use tessera_core::{ProjectContext, Result, Id};
use inquire::{Select, Text, Confirm, CustomType, DateSelect};
use inquire::validator::Validation;
use chrono::{Utc, TimeZone, DateTime};

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

    pub fn list_resources(&self) -> Result<()> {
        let resources = self.repository.get_resources();
        
        if resources.is_empty() {
            println!("No resources found");
            return Ok(());
        }
        
        println!("Resources:");
        for (i, resource) in resources.iter().enumerate() {
            let rate_str = resource.hourly_rate
                .map(|rate| format!(" (${:.2}/hr)", rate))
                .unwrap_or_default();
            
            println!("{}. {} - {}{} - {:.0}% available",
                     i + 1, 
                     resource.name, 
                     resource.role,
                     rate_str,
                     resource.availability_percentage);
            
            if !resource.skills.is_empty() {
                println!("   Skills: {}", resource.skills.join(", "));
            }
            
            println!("   ID: {}", resource.id);
            println!();
        }
        
        Ok(())
    }

    pub fn list_milestones(&self) -> Result<()> {
        let milestones = self.repository.get_milestones();
        
        if milestones.is_empty() {
            println!("No milestones found");
            return Ok(());
        }
        
        println!("Milestones:");
        for (i, milestone) in milestones.iter().enumerate() {
            let status_symbol = match milestone.status {
                MilestoneStatus::Pending => "○",
                MilestoneStatus::AtRisk => "⚠",
                MilestoneStatus::Achieved => "●",
                MilestoneStatus::Missed => "✗",
            };
            
            let overdue_marker = if milestone.is_overdue() { " [OVERDUE]" } else { "" };
            
            println!("{}. {} {} - {} ({}){}", 
                     i + 1, 
                     status_symbol, 
                     milestone.name, 
                     milestone.description,
                     milestone.target_date.format("%Y-%m-%d"),
                     overdue_marker);
            
            if !milestone.dependencies.is_empty() {
                println!("   Dependencies:");
                for dep in &milestone.dependencies {
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
            
            println!("   ID: {}", milestone.id);
            println!();
        }
        
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

        let selected = Select::new("Select resource to delete:", resource_options.clone())
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

    /// Add a project risk interactively
    pub async fn add_risk_interactive(&mut self) -> Result<()> {
        let name = Text::new("Risk name:")
            .with_help_message("Describe the risk")
            .prompt()?;
        
        let description = Text::new("Risk description:")
            .with_help_message("Detailed description of the risk")
            .prompt()?;
        
        // Select risk category
        let categories = vec![
            "Technical",
            "Schedule",
            "Cost", 
            "Resource",
            "External",
            "Quality",
            "Organizational",
            "Security",
            "Environmental",
            "Market",
        ];
        
        let category_str = Select::new("Risk category:", categories).prompt()?;
        let category = match category_str {
            "Technical" => crate::risk::RiskCategory::Technical,
            "Schedule" => crate::risk::RiskCategory::Schedule,
            "Cost" => crate::risk::RiskCategory::Cost,
            "Resource" => crate::risk::RiskCategory::Resource,
            "External" => crate::risk::RiskCategory::External,
            "Quality" => crate::risk::RiskCategory::Quality,
            "Organizational" => crate::risk::RiskCategory::Organizational,
            "Security" => crate::risk::RiskCategory::Security,
            "Environmental" => crate::risk::RiskCategory::Environmental,
            _ => crate::risk::RiskCategory::Market,
        };
        
        // Risk probability
        let probability_options = vec![
            "Very Low",
            "Low",
            "Medium",
            "High", 
            "Very High",
            "Certain",
        ];
        
        let prob_selection = Select::new("Probability:", probability_options).prompt()?;
        let probability = match prob_selection {
            "Very Low" => crate::risk::RiskProbability::VeryLow,
            "Low" => crate::risk::RiskProbability::Low,
            "Medium" => crate::risk::RiskProbability::Medium,
            "High" => crate::risk::RiskProbability::High,
            "Very High" => crate::risk::RiskProbability::VeryHigh,
            _ => crate::risk::RiskProbability::Certain,
        };
        
        // Risk impact
        let impact_options = vec![
            "Negligible",
            "Minor",
            "Moderate",
            "Major",
            "Severe",
        ];
        
        let impact_selection = Select::new("Impact:", impact_options).prompt()?;
        let impact = match impact_selection {
            "Negligible" => crate::risk::RiskImpact::Negligible,
            "Minor" => crate::risk::RiskImpact::Minor,
            "Moderate" => crate::risk::RiskImpact::Moderate,
            "Major" => crate::risk::RiskImpact::Major,
            _ => crate::risk::RiskImpact::Severe,
        };
        
        // Owner
        let owner = Text::new("Risk owner:")
            .with_help_message("Person responsible for managing this risk")
            .prompt()?;
        
        let mut risk = crate::risk::ProjectRisk::new(name, description, owner);
        risk.category = category;
        risk.probability = probability;
        risk.impact = impact;
        risk.calculate_risk_score();
        
        self.repository.add_risk(risk.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Project risk '{}' added successfully!", risk.title);
        println!("ID: {}", risk.id);
        println!("Risk Score: {:.1} ({:?} x {:?})", risk.risk_score, risk.probability, risk.impact);
        
        Ok(())
    }

    /// List project risks
    pub fn list_risks(&self) -> Result<()> {
        let risks = self.repository.get_risks();
        
        if risks.is_empty() {
            println!("No project risks found");
            return Ok(());
        }
        
        println!("Project Risks:");
        println!("=============");
        
        for (i, risk) in risks.iter().enumerate() {
            let status_symbol = match risk.status {
                crate::risk::RiskStatus::Identified => "🔍",
                crate::risk::RiskStatus::Analyzing => "🔬",
                crate::risk::RiskStatus::Planning => "📋",
                crate::risk::RiskStatus::Mitigating => "⚠️",
                crate::risk::RiskStatus::Monitoring => "👁️",
                crate::risk::RiskStatus::Realized => "💥",
                crate::risk::RiskStatus::Closed => "✅",
                crate::risk::RiskStatus::Transferred => "📤",
            };
            
            println!("{}. {} {} - {}", 
                     i + 1, status_symbol, risk.title, risk.description);
            println!("   Category: {:?}, Score: {:.1} (P:{:?} × I:{:?})",
                     risk.category, risk.risk_score, risk.probability, risk.impact);
            
            if let Some(ref owner) = risk.owner {
                println!("   Owner: {}", owner);
            }
            
            if !risk.mitigation_actions.is_empty() {
                println!("   Mitigations:");
                for action in &risk.mitigation_actions {
                    println!("     - {}", action.description);
                }
            }
            
            println!("   ID: {}", risk.id);
            println!();
        }
        
        Ok(())
    }

    /// Add an issue interactively
    pub async fn add_issue_interactive(&mut self) -> Result<()> {
        let title = Text::new("Issue title:")
            .with_help_message("Brief description of the issue")
            .prompt()?;
        
        let description = Text::new("Issue description:")
            .with_help_message("Detailed description of the issue")
            .prompt()?;
        
        // Select issue category
        let categories = vec![
            "Technical",
            "Process",
            "Resource",
            "Scope",
            "Quality",
            "Communication",
            "External",
            "Requirements",
            "Environment",
        ];
        
        let category_str = Select::new("Issue category:", categories).prompt()?;
        let category = match category_str {
            "Technical" => crate::issue::IssueCategory::Technical,
            "Process" => crate::issue::IssueCategory::Process,
            "Resource" => crate::issue::IssueCategory::Resource,
            "Scope" => crate::issue::IssueCategory::Scope,
            "Quality" => crate::issue::IssueCategory::Quality,
            "Communication" => crate::issue::IssueCategory::Communication,
            "External" => crate::issue::IssueCategory::External,
            "Requirements" => crate::issue::IssueCategory::Requirements,
            _ => crate::issue::IssueCategory::Environment,
        };
        
        // Select priority
        let priorities = vec![
            "Low",
            "Medium",
            "High",
            "Critical",
        ];
        
        let priority_str = Select::new("Priority:", priorities).prompt()?;
        let priority = match priority_str {
            "Low" => crate::issue::IssuePriority::Low,
            "Medium" => crate::issue::IssuePriority::Medium,
            "High" => crate::issue::IssuePriority::High,
            "Critical" => crate::issue::IssuePriority::Critical,
            _ => crate::issue::IssuePriority::Medium,
        };
        
        // Select severity
        let severities = vec![
            "Trivial",
            "Minor",
            "Major",
            "Blocker",
        ];
        
        let severity_str = Select::new("Severity:", severities).prompt()?;
        let severity = match severity_str {
            "Trivial" => crate::issue::IssueSeverity::Trivial,
            "Minor" => crate::issue::IssueSeverity::Minor,
            "Major" => crate::issue::IssueSeverity::Major,
            "Blocker" => crate::issue::IssueSeverity::Blocker,
            _ => crate::issue::IssueSeverity::Minor,
        };
        
        // Reporter
        let reported_by = Text::new("Reported by:")
            .with_help_message("Person who reported this issue")
            .prompt()?;
        
        let mut issue = crate::issue::Issue::new(title, description, reported_by);
        issue.category = category;
        issue.priority = priority;
        issue.severity = severity;
        
        self.repository.add_issue(issue.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Issue '{}' added successfully!", issue.title);
        println!("ID: {}", issue.id);
        
        Ok(())
    }

    /// List issues
    pub fn list_issues(&self) -> Result<()> {
        let issues = self.repository.get_issues();
        
        if issues.is_empty() {
            println!("No issues found");
            return Ok(());
        }
        
        println!("Issues:");
        println!("======");
        
        for (i, issue) in issues.iter().enumerate() {
            let status_symbol = match issue.status {
                crate::issue::IssueStatus::Open => "🔓",
                crate::issue::IssueStatus::InProgress => "⚠️",
                crate::issue::IssueStatus::PendingReview => "👀",
                crate::issue::IssueStatus::Resolved => "✅",
                crate::issue::IssueStatus::Closed => "🔒",
                crate::issue::IssueStatus::Deferred => "⏸️",
                crate::issue::IssueStatus::Duplicate => "📋",
                crate::issue::IssueStatus::CannotReproduce => "❓",
            };
            
            let priority_symbol = match issue.priority {
                crate::issue::IssuePriority::Low => "▼",
                crate::issue::IssuePriority::Medium => "◆",
                crate::issue::IssuePriority::High => "▲",
                crate::issue::IssuePriority::Critical => "🔥",
            };
            
            let severity_symbol = match issue.severity {
                crate::issue::IssueSeverity::Trivial => "🟢",
                crate::issue::IssueSeverity::Minor => "🟡",
                crate::issue::IssueSeverity::Major => "🟠",
                crate::issue::IssueSeverity::Blocker => "🔴",
            };
            
            println!("{}. {} {} {} {} - {}", 
                     i + 1, status_symbol, priority_symbol, severity_symbol, issue.title, issue.description);
            println!("   Category: {:?}, Reporter: {}", issue.category, issue.reported_by);
            
            if let Some(ref assignee) = issue.assigned_to {
                println!("   Assignee: {}", assignee);
            }
            
            if let Some(due_date) = issue.due_date {
                println!("   Due: {}", due_date.format("%Y-%m-%d"));
            }
            
            println!("   ID: {}", issue.id);
            println!();
        }
        
        Ok(())
    }

    /// Create a baseline interactively
    pub async fn create_baseline_interactive(&mut self) -> Result<()> {
        let name = Text::new("Baseline name:")
            .with_help_message("Name for this baseline snapshot")
            .prompt()?;
        
        let description = Text::new("Baseline description:")
            .with_help_message("Description of what this baseline represents")
            .prompt()?;
        
        // Select baseline type
        let types = vec![
            "Initial",
            "Approved",
            "Working",
            "Archived",
        ];
        
        let type_str = Select::new("Baseline type:", types).prompt()?;
        let baseline_type = match type_str {
            "Initial" => crate::baseline::BaselineType::Initial,
            "Approved" => crate::baseline::BaselineType::Approved,
            "Working" => crate::baseline::BaselineType::Working,
            "Archived" => crate::baseline::BaselineType::Archived,
            _ => crate::baseline::BaselineType::Working,
        };
        
        // Created by
        let created_by = Text::new("Created by:")
            .with_help_message("Person creating this baseline")
            .prompt()?;
        
        // Get current project name
        let project_name = self.project_context.metadata.name.clone();
        
        // Get current state snapshots
        let tasks = self.repository.get_tasks();
        let milestones = self.repository.get_milestones();
        let resources = self.repository.get_resources();
        
        let mut baseline = crate::baseline::ProjectBaseline::new(
            name,
            created_by,
            baseline_type,
            project_name,
            tasks,
            milestones,
            resources,
        );
        baseline.description = Some(description);
        
        self.repository.add_baseline(baseline.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Baseline '{}' created successfully!", baseline.name);
        println!("ID: {}", baseline.id);
        println!("Type: {:?}", baseline.baseline_type);
        
        Ok(())
    }

    /// List baselines
    pub fn list_baselines(&self) -> Result<()> {
        let baselines = self.repository.get_baselines();
        
        if baselines.is_empty() {
            println!("No baselines found");
            return Ok(());
        }
        
        println!("Project Baselines:");
        println!("=================");
        
        for (i, baseline) in baselines.iter().enumerate() {
            let description = baseline.description.as_ref().map(|s| s.as_str()).unwrap_or("No description");
            println!("{}. {} - {}", i + 1, baseline.name, description);
            println!("   Type: {:?}", baseline.baseline_type);
            println!("   Created: {} by {}", baseline.created_date.format("%Y-%m-%d"), baseline.created_by);
            println!("   Current: {}", if baseline.is_current { "Yes" } else { "No" });
            println!("   ID: {}", baseline.id);
            println!();
        }
        
        Ok(())
    }

    /// Add a calendar interactively
    pub async fn add_calendar_interactive(&mut self) -> Result<()> {
        let name = Text::new("Calendar name:")
            .with_help_message("Name for this calendar")
            .prompt()?;
        
        let description = Text::new("Calendar description:")
            .with_help_message("Description of this calendar")
            .prompt()?;
        
        // Working hours
        let start_hour = CustomType::<u8>::new("Start hour (0-23):")
            .with_default(9)
            .with_help_message("Hour when work starts (0-23)")
            .prompt()?;
        
        let end_hour = CustomType::<u8>::new("End hour (0-23):")
            .with_default(17)
            .with_help_message("Hour when work ends (0-23)")
            .prompt()?;
        
        let daily_hours = CustomType::<f32>::new("Daily working hours:")
            .with_default(8.0)
            .with_help_message("Total working hours per day")
            .prompt()?;
        
        let working_hours = crate::calendar::WorkingHours {
            start_time: start_hour,
            end_time: end_hour,
            daily_hours,
        };
        
        let mut calendar = crate::calendar::Calendar::new(name);
        calendar.description = Some(description);
        calendar.working_hours = working_hours;
        
        self.repository.add_calendar(calendar.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Calendar '{}' added successfully!", calendar.name);
        println!("ID: {}", calendar.id);
        println!("Working hours: {}:00 to {}:00, {:.1} hours/day", 
                 calendar.working_hours.start_time,
                 calendar.working_hours.end_time,
                 calendar.working_hours.daily_hours);
        
        Ok(())
    }

    /// List calendars
    pub fn list_calendars(&self) -> Result<()> {
        let calendars = self.repository.get_calendars();
        
        if calendars.is_empty() {
            println!("No calendars found");
            return Ok(());
        }
        
        println!("Calendars:");
        println!("=========");
        
        for (i, calendar) in calendars.iter().enumerate() {
            let description = calendar.description.as_ref().map(|s| s.as_str()).unwrap_or("No description");
            println!("{}. {} - {}", i + 1, calendar.name, description);
            println!("   Working Hours: {}:00 to {}:00, {:.1} hours/day",
                     calendar.working_hours.start_time,
                     calendar.working_hours.end_time,
                     calendar.working_hours.daily_hours);
            
            if !calendar.holidays.is_empty() {
                println!("   Holidays: {} defined", calendar.holidays.len());
            }
            
            if !calendar.exceptions.is_empty() {
                println!("   Exceptions: {} defined", calendar.exceptions.len());
            }
            
            println!("   ID: {}", calendar.id);
            println!();
        }
        
        Ok(())
    }

    /// Edit a project risk interactively
    pub async fn edit_risk_interactive(&mut self) -> Result<()> {
        let risks = self.repository.get_risks().to_vec();
        
        if risks.is_empty() {
            println!("No project risks found. Add risks first.");
            return Ok(());
        }

        // Create risk options
        let risk_options: Vec<String> = risks.iter()
            .map(|r| format!("{} - {} ({:?})", r.title, r.description, r.status))
            .collect();

        let selected = Select::new("Select risk to edit:", risk_options.clone())
            .with_help_message("Choose the risk you want to edit")
            .prompt()?;

        let risk_index = risk_options.iter().position(|x| x == &selected).unwrap();
        let mut risk = risks[risk_index].clone();

        loop {
            println!("\nEditing Risk: {}", risk.title);
            
            let edit_options = vec![
                "Title",
                "Description", 
                "Category",
                "Probability",
                "Impact",
                "Status",
                "Owner",
                "Target Resolution Date",
                "Mitigation Strategy",
                "Add Mitigation Action",
                "Done",
            ];

            let choice = Select::new("What would you like to edit?", edit_options).prompt()?;

            match choice {
                "Title" => {
                    let new_title = Text::new("New title:")
                        .with_default(&risk.title)
                        .prompt()?;
                    risk.title = new_title;
                },
                "Description" => {
                    let new_description = Text::new("New description:")
                        .with_default(&risk.description)
                        .prompt()?;
                    risk.description = new_description;
                },
                "Category" => {
                    let categories = vec![
                        "Technical", "Schedule", "Cost", "Resource", "External",
                        "Quality", "Organizational", "Security", "Environmental", "Market",
                    ];
                    let category_str = Select::new("Risk category:", categories).prompt()?;
                    risk.category = match category_str {
                        "Technical" => crate::risk::RiskCategory::Technical,
                        "Schedule" => crate::risk::RiskCategory::Schedule,
                        "Cost" => crate::risk::RiskCategory::Cost,
                        "Resource" => crate::risk::RiskCategory::Resource,
                        "External" => crate::risk::RiskCategory::External,
                        "Quality" => crate::risk::RiskCategory::Quality,
                        "Organizational" => crate::risk::RiskCategory::Organizational,
                        "Security" => crate::risk::RiskCategory::Security,
                        "Environmental" => crate::risk::RiskCategory::Environmental,
                        _ => crate::risk::RiskCategory::Market,
                    };
                },
                "Probability" => {
                    let probability_options = vec![
                        "Very Low", "Low", "Medium", "High", "Very High", "Certain",
                    ];
                    let prob_selection = Select::new("Probability:", probability_options).prompt()?;
                    risk.probability = match prob_selection {
                        "Very Low" => crate::risk::RiskProbability::VeryLow,
                        "Low" => crate::risk::RiskProbability::Low,
                        "Medium" => crate::risk::RiskProbability::Medium,
                        "High" => crate::risk::RiskProbability::High,
                        "Very High" => crate::risk::RiskProbability::VeryHigh,
                        _ => crate::risk::RiskProbability::Certain,
                    };
                },
                "Impact" => {
                    let impact_options = vec![
                        "Negligible", "Minor", "Moderate", "Major", "Severe",
                    ];
                    let impact_selection = Select::new("Impact:", impact_options).prompt()?;
                    risk.impact = match impact_selection {
                        "Negligible" => crate::risk::RiskImpact::Negligible,
                        "Minor" => crate::risk::RiskImpact::Minor,
                        "Moderate" => crate::risk::RiskImpact::Moderate,
                        "Major" => crate::risk::RiskImpact::Major,
                        _ => crate::risk::RiskImpact::Severe,
                    };
                },
                "Status" => {
                    let status_options = vec![
                        "Identified", "Analyzing", "Planning", "Mitigating", 
                        "Monitoring", "Realized", "Closed", "Transferred",
                    ];
                    let status_selection = Select::new("Status:", status_options).prompt()?;
                    risk.status = match status_selection {
                        "Identified" => crate::risk::RiskStatus::Identified,
                        "Analyzing" => crate::risk::RiskStatus::Analyzing,
                        "Planning" => crate::risk::RiskStatus::Planning,
                        "Mitigating" => crate::risk::RiskStatus::Mitigating,
                        "Monitoring" => crate::risk::RiskStatus::Monitoring,
                        "Realized" => crate::risk::RiskStatus::Realized,
                        "Closed" => crate::risk::RiskStatus::Closed,
                        _ => crate::risk::RiskStatus::Transferred,
                    };
                },
                "Owner" => {
                    let default_owner = "".to_string();
                    let current_owner = risk.owner.as_ref().unwrap_or(&default_owner);
                    let new_owner = Text::new("Risk owner:")
                        .with_default(current_owner)
                        .prompt()?;
                    if !new_owner.is_empty() {
                        risk.owner = Some(new_owner);
                    } else {
                        risk.owner = None;
                    }
                },
                "Target Resolution Date" => {
                    let use_date = Confirm::new("Set target resolution date?")
                        .with_default(risk.target_resolution_date.is_some())
                        .prompt()?;
                    
                    if use_date {
                        let default_date = risk.target_resolution_date
                            .unwrap_or_else(|| chrono::Local::now().date_naive());
                        let target_date = DateSelect::new("Target resolution date:")
                            .with_default(default_date)
                            .prompt()?;
                        risk.target_resolution_date = Some(target_date);
                    } else {
                        risk.target_resolution_date = None;
                    }
                },
                "Mitigation Strategy" => {
                    let default_strategy = "".to_string();
                    let current_strategy = risk.mitigation_strategy.as_ref().unwrap_or(&default_strategy);
                    let new_strategy = Text::new("Mitigation strategy:")
                        .with_default(current_strategy)
                        .prompt()?;
                    if !new_strategy.is_empty() {
                        risk.mitigation_strategy = Some(new_strategy);
                    } else {
                        risk.mitigation_strategy = None;
                    }
                },
                "Add Mitigation Action" => {
                    let action_description = Text::new("Action description:")
                        .prompt()?;
                    let assigned_to = Text::new("Assigned to:")
                        .prompt()?;
                    let due_date = DateSelect::new("Due date:")
                        .with_default(chrono::Local::now().date_naive())
                        .prompt()?;
                    
                    let action = crate::risk::MitigationAction {
                        id: tessera_core::Id::new(),
                        description: action_description,
                        assigned_to,
                        due_date,
                        status: crate::risk::ActionStatus::NotStarted,
                        estimated_cost: None,
                        actual_cost: None,
                        completion_date: None,
                        notes: None,
                        effectiveness_rating: None,
                    };
                    
                    risk.mitigation_actions.push(action);
                    println!("✓ Mitigation action added!");
                },
                "Done" => break,
                _ => {}
            }

            // Recalculate risk score
            risk.calculate_risk_score();
            risk.updated = Utc::now();
        }

        // Save the updated risk
        self.repository.update_risk(risk.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Risk '{}' updated successfully!", risk.title);
        
        Ok(())
    }

    /// Edit an issue interactively
    pub async fn edit_issue_interactive(&mut self) -> Result<()> {
        let issues = self.repository.get_issues().to_vec();
        
        if issues.is_empty() {
            println!("No issues found. Add issues first.");
            return Ok(());
        }

        // Create issue options
        let issue_options: Vec<String> = issues.iter()
            .map(|i| format!("{} - {} ({:?})", i.title, i.description, i.status))
            .collect();

        let selected = Select::new("Select issue to edit:", issue_options.clone())
            .with_help_message("Choose the issue you want to edit")
            .prompt()?;

        let issue_index = issue_options.iter().position(|x| x == &selected).unwrap();
        let mut issue = issues[issue_index].clone();

        loop {
            println!("\nEditing Issue: {}", issue.title);
            
            let edit_options = vec![
                "Title",
                "Description", 
                "Category",
                "Priority",
                "Severity",
                "Status",
                "Assigned To",
                "Due Date",
                "Resolution Description",
                "Add Comment",
                "Done",
            ];

            let choice = Select::new("What would you like to edit?", edit_options).prompt()?;

            match choice {
                "Title" => {
                    let new_title = Text::new("New title:")
                        .with_default(&issue.title)
                        .prompt()?;
                    issue.title = new_title;
                },
                "Description" => {
                    let new_description = Text::new("New description:")
                        .with_default(&issue.description)
                        .prompt()?;
                    issue.description = new_description;
                },
                "Category" => {
                    let categories = vec![
                        "Technical", "Process", "Resource", "Scope", "Quality",
                        "Communication", "External", "Requirements", "Environment",
                    ];
                    let category_str = Select::new("Issue category:", categories).prompt()?;
                    issue.category = match category_str {
                        "Technical" => crate::issue::IssueCategory::Technical,
                        "Process" => crate::issue::IssueCategory::Process,
                        "Resource" => crate::issue::IssueCategory::Resource,
                        "Scope" => crate::issue::IssueCategory::Scope,
                        "Quality" => crate::issue::IssueCategory::Quality,
                        "Communication" => crate::issue::IssueCategory::Communication,
                        "External" => crate::issue::IssueCategory::External,
                        "Requirements" => crate::issue::IssueCategory::Requirements,
                        _ => crate::issue::IssueCategory::Environment,
                    };
                },
                "Priority" => {
                    let priorities = vec!["Low", "Medium", "High", "Critical"];
                    let priority_str = Select::new("Priority:", priorities).prompt()?;
                    issue.priority = match priority_str {
                        "Low" => crate::issue::IssuePriority::Low,
                        "Medium" => crate::issue::IssuePriority::Medium,
                        "High" => crate::issue::IssuePriority::High,
                        _ => crate::issue::IssuePriority::Critical,
                    };
                },
                "Severity" => {
                    let severities = vec!["Trivial", "Minor", "Major", "Blocker"];
                    let severity_str = Select::new("Severity:", severities).prompt()?;
                    issue.severity = match severity_str {
                        "Trivial" => crate::issue::IssueSeverity::Trivial,
                        "Minor" => crate::issue::IssueSeverity::Minor,
                        "Major" => crate::issue::IssueSeverity::Major,
                        _ => crate::issue::IssueSeverity::Blocker,
                    };
                },
                "Status" => {
                    let status_options = vec![
                        "Open", "InProgress", "PendingReview", "Resolved", 
                        "Closed", "Deferred", "Duplicate", "CannotReproduce",
                    ];
                    let status_selection = Select::new("Status:", status_options).prompt()?;
                    issue.status = match status_selection {
                        "Open" => crate::issue::IssueStatus::Open,
                        "InProgress" => crate::issue::IssueStatus::InProgress,
                        "PendingReview" => crate::issue::IssueStatus::PendingReview,
                        "Resolved" => crate::issue::IssueStatus::Resolved,
                        "Closed" => crate::issue::IssueStatus::Closed,
                        "Deferred" => crate::issue::IssueStatus::Deferred,
                        "Duplicate" => crate::issue::IssueStatus::Duplicate,
                        _ => crate::issue::IssueStatus::CannotReproduce,
                    };
                    
                    // If marking as resolved, set resolution date
                    if matches!(issue.status, crate::issue::IssueStatus::Resolved | crate::issue::IssueStatus::Closed) 
                        && issue.resolution_date.is_none() {
                        issue.resolution_date = Some(Utc::now());
                    }
                },
                "Assigned To" => {
                    let default_assignee = "".to_string();
                    let current_assignee = issue.assigned_to.as_ref().unwrap_or(&default_assignee);
                    let new_assignee = Text::new("Assigned to:")
                        .with_default(current_assignee)
                        .prompt()?;
                    if !new_assignee.is_empty() {
                        issue.assigned_to = Some(new_assignee);
                    } else {
                        issue.assigned_to = None;
                    }
                },
                "Due Date" => {
                    let use_date = Confirm::new("Set due date?")
                        .with_default(issue.due_date.is_some())
                        .prompt()?;
                    
                    if use_date {
                        let default_date = issue.due_date
                            .unwrap_or_else(|| chrono::Local::now().date_naive());
                        let due_date = DateSelect::new("Due date:")
                            .with_default(default_date)
                            .prompt()?;
                        issue.due_date = Some(due_date);
                    } else {
                        issue.due_date = None;
                    }
                },
                "Resolution Description" => {
                    let default_resolution = "".to_string();
                    let current_resolution = issue.resolution_description.as_ref().unwrap_or(&default_resolution);
                    let new_resolution = Text::new("Resolution description:")
                        .with_default(current_resolution)
                        .prompt()?;
                    if !new_resolution.is_empty() {
                        issue.resolution_description = Some(new_resolution);
                    } else {
                        issue.resolution_description = None;
                    }
                },
                "Add Comment" => {
                    let content = Text::new("Comment:")
                        .prompt()?;
                    let author = Text::new("Author:")
                        .with_default(&issue.reported_by)
                        .prompt()?;
                    
                    let is_resolution = if matches!(issue.status, crate::issue::IssueStatus::Resolved | crate::issue::IssueStatus::Closed) {
                        Confirm::new("Is this a resolution comment?")
                            .with_default(false)
                            .prompt()?
                    } else {
                        false
                    };
                    
                    let comment = crate::issue::IssueComment {
                        id: tessera_core::Id::new(),
                        author,
                        content,
                        created_date: Utc::now(),
                        is_resolution,
                    };
                    
                    issue.comments.push(comment);
                    println!("✓ Comment added!");
                },
                "Done" => break,
                _ => {}
            }

            issue.updated = Utc::now();
        }

        // Save the updated issue
        self.repository.update_issue(issue.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Issue '{}' updated successfully!", issue.title);
        
        Ok(())
    }

    /// Edit a calendar interactively
    pub async fn edit_calendar_interactive(&mut self) -> Result<()> {
        let calendars = self.repository.get_calendars().to_vec();
        
        if calendars.is_empty() {
            println!("No calendars found. Add calendars first.");
            return Ok(());
        }

        // Create calendar options
        let default_desc = "No description".to_string();
        let calendar_options: Vec<String> = calendars.iter()
            .map(|c| format!("{} - {}", c.name, 
                            c.description.as_ref().unwrap_or(&default_desc)))
            .collect();

        let selected = Select::new("Select calendar to edit:", calendar_options.clone())
            .with_help_message("Choose the calendar you want to edit")
            .prompt()?;

        let calendar_index = calendar_options.iter().position(|x| x == &selected).unwrap();
        let mut calendar = calendars[calendar_index].clone();

        loop {
            println!("\nEditing Calendar: {}", calendar.name);
            
            let edit_options = vec![
                "Name",
                "Description", 
                "Working Hours",
                "Working Days",
                "Add Holiday",
                "List Holidays",
                "Remove Holiday",
                "Add Exception",
                "List Exceptions",
                "Remove Exception",
                "Done",
            ];

            let choice = Select::new("What would you like to edit?", edit_options).prompt()?;

            match choice {
                "Name" => {
                    let new_name = Text::new("New name:")
                        .with_default(&calendar.name)
                        .prompt()?;
                    calendar.name = new_name;
                },
                "Description" => {
                    let default_desc = "".to_string();
                    let current_desc = calendar.description.as_ref().unwrap_or(&default_desc);
                    let new_description = Text::new("New description:")
                        .with_default(current_desc)
                        .prompt()?;
                    if !new_description.is_empty() {
                        calendar.description = Some(new_description);
                    } else {
                        calendar.description = None;
                    }
                },
                "Working Hours" => {
                    let start_hour = CustomType::<u8>::new("Start hour (0-23):")
                        .with_default(calendar.working_hours.start_time)
                        .prompt()?;
                    
                    let end_hour = CustomType::<u8>::new("End hour (0-23):")
                        .with_default(calendar.working_hours.end_time)
                        .prompt()?;
                    
                    let daily_hours = CustomType::<f32>::new("Daily working hours:")
                        .with_default(calendar.working_hours.daily_hours)
                        .prompt()?;
                    
                    calendar.working_hours = crate::calendar::WorkingHours {
                        start_time: start_hour,
                        end_time: end_hour,
                        daily_hours,
                    };
                },
                "Working Days" => {
                    println!("Current working days: {:?}", calendar.working_days);
                    
                    let all_days = vec![
                        ("Monday", chrono::Weekday::Mon),
                        ("Tuesday", chrono::Weekday::Tue),
                        ("Wednesday", chrono::Weekday::Wed),
                        ("Thursday", chrono::Weekday::Thu),
                        ("Friday", chrono::Weekday::Fri),
                        ("Saturday", chrono::Weekday::Sat),
                        ("Sunday", chrono::Weekday::Sun),
                    ];
                    
                    let mut new_working_days = Vec::new();
                    
                    for (day_name, weekday) in all_days {
                        let is_working = Confirm::new(&format!("Include {}?", day_name))
                            .with_default(calendar.working_days.contains(&weekday))
                            .prompt()?;
                        
                        if is_working {
                            new_working_days.push(weekday);
                        }
                    }
                    
                    calendar.working_days = new_working_days;
                },
                "Add Holiday" => {
                    let name = Text::new("Holiday name:")
                        .prompt()?;
                    
                    let description = Text::new("Holiday description (optional):")
                        .prompt()?;
                    
                    let date = DateSelect::new("Holiday date:")
                        .with_default(chrono::Local::now().date_naive())
                        .prompt()?;
                    
                    let recurring = Confirm::new("Is this a recurring holiday?")
                        .with_default(false)
                        .prompt()?;
                    
                    let holiday = crate::calendar::Holiday {
                        id: tessera_core::Id::new(),
                        name,
                        date,
                        description: if description.is_empty() { None } else { Some(description) },
                        recurring,
                    };
                    
                    calendar.holidays.push(holiday);
                    println!("✓ Holiday added!");
                },
                "List Holidays" => {
                    if calendar.holidays.is_empty() {
                        println!("No holidays defined.");
                    } else {
                        println!("Holidays:");
                        for (i, holiday) in calendar.holidays.iter().enumerate() {
                            let recurring_text = if holiday.recurring { " (recurring)" } else { "" };
                            println!("  {}. {} - {}{}", 
                                     i + 1, 
                                     holiday.name, 
                                     holiday.date.format("%Y-%m-%d"),
                                     recurring_text);
                            if let Some(ref desc) = holiday.description {
                                println!("     {}", desc);
                            }
                        }
                    }
                },
                "Remove Holiday" => {
                    if calendar.holidays.is_empty() {
                        println!("No holidays to remove.");
                    } else {
                        let holiday_options: Vec<String> = calendar.holidays.iter()
                            .map(|h| format!("{} - {}", h.name, h.date.format("%Y-%m-%d")))
                            .collect();
                        
                        let selected_holiday = Select::new("Select holiday to remove:", holiday_options.clone())
                            .prompt()?;
                        
                        let holiday_index = holiday_options.iter().position(|x| x == &selected_holiday).unwrap();
                        let removed_holiday = calendar.holidays.remove(holiday_index);
                        println!("✓ Holiday '{}' removed!", removed_holiday.name);
                    }
                },
                "Add Exception" => {
                    let date = DateSelect::new("Exception date:")
                        .with_default(chrono::Local::now().date_naive())
                        .prompt()?;
                    
                    let exception_types = vec!["Working", "NonWorking", "HalfDay"];
                    let exception_type_str = Select::new("Exception type:", exception_types).prompt()?;
                    let exception_type = match exception_type_str {
                        "Working" => crate::calendar::ExceptionType::Working,
                        "NonWorking" => crate::calendar::ExceptionType::NonWorking,
                        _ => crate::calendar::ExceptionType::HalfDay,
                    };
                    
                    let description = Text::new("Exception description (optional):")
                        .prompt()?;
                    
                    let exception = crate::calendar::CalendarException {
                        id: tessera_core::Id::new(),
                        date,
                        exception_type,
                        description: if description.is_empty() { None } else { Some(description) },
                    };
                    
                    calendar.exceptions.push(exception);
                    println!("✓ Exception added!");
                },
                "List Exceptions" => {
                    if calendar.exceptions.is_empty() {
                        println!("No exceptions defined.");
                    } else {
                        println!("Exceptions:");
                        for (i, exception) in calendar.exceptions.iter().enumerate() {
                            println!("  {}. {} - {:?}", 
                                     i + 1, 
                                     exception.date.format("%Y-%m-%d"),
                                     exception.exception_type);
                            if let Some(ref desc) = exception.description {
                                println!("     {}", desc);
                            }
                        }
                    }
                },
                "Remove Exception" => {
                    if calendar.exceptions.is_empty() {
                        println!("No exceptions to remove.");
                    } else {
                        let exception_options: Vec<String> = calendar.exceptions.iter()
                            .map(|e| format!("{} - {:?}", e.date.format("%Y-%m-%d"), e.exception_type))
                            .collect();
                        
                        let selected_exception = Select::new("Select exception to remove:", exception_options.clone())
                            .prompt()?;
                        
                        let exception_index = exception_options.iter().position(|x| x == &selected_exception).unwrap();
                        let removed_exception = calendar.exceptions.remove(exception_index);
                        println!("✓ Exception for {} removed!", removed_exception.date.format("%Y-%m-%d"));
                    }
                },
                "Done" => break,
                _ => {}
            }

            calendar.updated = Utc::now();
        }

        // Save the updated calendar
        self.repository.update_calendar(calendar.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Calendar '{}' updated successfully!", calendar.name);
        
        Ok(())
    }

    /// Assign calendar to resource interactively
    pub async fn assign_calendar_to_resource_interactive(&mut self) -> Result<()> {
        let resources = self.repository.get_resources().to_vec();
        let calendars = self.repository.get_calendars().to_vec();
        
        if resources.is_empty() {
            println!("No resources found. Add resources first.");
            return Ok(());
        }
        
        if calendars.is_empty() {
            println!("No calendars found. Add calendars first.");
            return Ok(());
        }

        // Select resource
        let resource_options: Vec<String> = resources.iter()
            .map(|r| format!("{} - {} ({})", r.name, r.role, r.id))
            .collect();

        let selected_resource = Select::new("Select resource:", resource_options.clone())
            .with_help_message("Choose the resource to assign a calendar to")
            .prompt()?;

        let resource_index = resource_options.iter().position(|x| x == &selected_resource).unwrap();
        let resource = &resources[resource_index];

        // Select calendar
        let default_desc = "No description".to_string();
        let calendar_options: Vec<String> = calendars.iter()
            .map(|c| format!("{} - {}", c.name, 
                            c.description.as_ref().unwrap_or(&default_desc)))
            .collect();

        let selected_calendar = Select::new("Select calendar:", calendar_options.clone())
            .with_help_message("Choose the calendar to assign")
            .prompt()?;

        let calendar_index = calendar_options.iter().position(|x| x == &selected_calendar).unwrap();
        let calendar = &calendars[calendar_index];

        // Create resource calendar assignment
        let resource_calendar = crate::calendar::ResourceCalendar::new(resource.id, calendar.id);
        
        // Store this in resource metadata for now since we don't have a separate repository
        let mut updated_resource = resource.clone();
        updated_resource.metadata.insert("assigned_calendar_id".to_string(), calendar.id.to_string());
        updated_resource.metadata.insert("assigned_calendar_name".to_string(), calendar.name.clone());
        updated_resource.updated = Utc::now();

        self.repository.update_resource(updated_resource)?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Calendar '{}' assigned to resource '{}'!", calendar.name, resource.name);
        
        Ok(())
    }

    /// List resource calendar assignments
    pub fn list_resource_calendar_assignments(&self) -> Result<()> {
        let resources = self.repository.get_resources();
        let calendars = self.repository.get_calendars();
        
        println!("Resource Calendar Assignments:");
        println!("=============================");
        
        let mut has_assignments = false;
        
        for resource in resources {
            if let Some(calendar_id_str) = resource.metadata.get("assigned_calendar_id") {
                if let Ok(calendar_id) = tessera_core::Id::parse(calendar_id_str) {
                    let default_calendar_name = "Unknown Calendar".to_string();
                    let calendar_name = resource.metadata.get("assigned_calendar_name")
                        .unwrap_or(&default_calendar_name);
                    
                    println!("Resource: {} ({})", resource.name, resource.role);
                    println!("  Calendar: {} ({})", calendar_name, calendar_id);
                    println!();
                    has_assignments = true;
                }
            }
        }
        
        if !has_assignments {
            println!("No calendar assignments found.");
        }
        
        Ok(())
    }

    /// Remove calendar assignment from resource
    pub async fn remove_calendar_assignment_interactive(&mut self) -> Result<()> {
        let resources = self.repository.get_resources().to_vec();
        
        // Filter resources that have calendar assignments
        let assigned_resources: Vec<_> = resources.iter()
            .filter(|r| r.metadata.contains_key("assigned_calendar_id"))
            .collect();
        
        if assigned_resources.is_empty() {
            println!("No resources have calendar assignments.");
            return Ok(());
        }

        // Create options for assigned resources
        let resource_options: Vec<String> = assigned_resources.iter()
            .map(|r| {
                let default_calendar_name = "Unknown Calendar".to_string();
                let calendar_name = r.metadata.get("assigned_calendar_name")
                    .unwrap_or(&default_calendar_name);
                format!("{} - {} (assigned to: {})", r.name, r.role, calendar_name)
            })
            .collect();

        let selected = Select::new("Select resource to remove calendar assignment:", resource_options.clone())
            .with_help_message("Choose the resource to remove calendar assignment from")
            .prompt()?;

        let resource_index = resource_options.iter().position(|x| x == &selected).unwrap();
        let resource = assigned_resources[resource_index];

        // Confirm removal
        let default_calendar_name = "Unknown Calendar".to_string();
        let calendar_name = resource.metadata.get("assigned_calendar_name")
            .unwrap_or(&default_calendar_name);
        
        let confirm = Confirm::new(&format!("Remove calendar '{}' assignment from resource '{}'?", 
                                           calendar_name, resource.name))
            .with_default(false)
            .prompt()?;

        if !confirm {
            println!("Calendar assignment removal cancelled.");
            return Ok(());
        }

        // Remove the assignment
        let mut updated_resource = resource.clone();
        updated_resource.metadata.remove("assigned_calendar_id");
        updated_resource.metadata.remove("assigned_calendar_name");
        updated_resource.updated = Utc::now();

        self.repository.update_resource(updated_resource)?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Calendar assignment removed from resource '{}'!", resource.name);
        
        Ok(())
    }

    /// Check milestone status and show warnings for early/late milestones
    pub fn check_milestone_status(&self) -> Result<()> {
        let milestones = self.repository.get_milestones();
        let tasks = self.repository.get_tasks();
        
        if milestones.is_empty() {
            println!("No milestones found.");
            return Ok(());
        }

        println!("Milestone Status Report");
        println!("======================");
        
        let mut on_track = 0;
        let mut at_risk = 0;
        let mut early = 0;
        let mut late = 0;
        
        for milestone in milestones {
            let now = Utc::now();
            let days_to_target = (milestone.target_date - now).num_days();
            
            // Calculate dependency-driven date
            let dependency_driven_date = self.calculate_milestone_dependency_date(milestone, tasks);
            
            let status = if matches!(milestone.status, crate::data::MilestoneStatus::Achieved) {
                if let Some(actual_date) = milestone.actual_date {
                    if actual_date <= milestone.target_date {
                        early += 1;
                        "🟢 EARLY"
                    } else {
                        late += 1;
                        "🔴 LATE"
                    }
                } else {
                    on_track += 1;
                    "✅ ACHIEVED"
                }
            } else if let Some(dependency_date) = dependency_driven_date {
                let dependency_days_diff = (dependency_date - milestone.target_date).num_days();
                
                if dependency_days_diff > 7 {
                    late += 1;
                    "🔴 LATE"
                } else if dependency_days_diff > 0 {
                    at_risk += 1;
                    "⚠️ AT RISK"
                } else if dependency_days_diff < -7 {
                    early += 1;
                    "🟢 EARLY"
                } else {
                    on_track += 1;
                    "✅ ON TRACK"
                }
            } else if days_to_target < 0 {
                late += 1;
                "🔴 OVERDUE"
            } else if days_to_target < 7 {
                at_risk += 1;
                "⚠️ AT RISK"
            } else {
                on_track += 1;
                "✅ ON TRACK"
            };
            
            println!("{} {}", status, milestone.name);
            println!("  Target: {}", milestone.target_date.format("%Y-%m-%d"));
            
            if let Some(dependency_date) = dependency_driven_date {
                println!("  Dependency-driven: {}", dependency_date.format("%Y-%m-%d"));
                let variance_days = (dependency_date - milestone.target_date).num_days();
                if variance_days != 0 {
                    println!("  Variance: {} days", variance_days);
                }
            }
            
            if let Some(actual_date) = milestone.actual_date {
                println!("  Actual: {}", actual_date.format("%Y-%m-%d"));
                let actual_variance = (actual_date - milestone.target_date).num_days();
                if actual_variance != 0 {
                    println!("  Actual Variance: {} days", actual_variance);
                }
            }
            
            if !milestone.dependencies.is_empty() {
                println!("  Dependencies:");
                for dep in &milestone.dependencies {
                    let dep_name = self.repository.find_task_by_id(dep.predecessor_id)
                        .map(|t| format!("{} (Task)", t.name))
                        .or_else(|| self.repository.find_milestone_by_id(dep.predecessor_id)
                                 .map(|m| format!("{} (Milestone)", m.name)))
                        .unwrap_or_else(|| format!("Unknown ({})", dep.predecessor_id));
                    
                    let lag_str = if dep.lag_days != 0.0 {
                        format!(" +{:.1}d", dep.lag_days)
                    } else {
                        String::new()
                    };
                    
                    println!("    - {} [{:?}]{}", dep_name, dep.dependency_type, lag_str);
                }
            }
            
            println!();
        }
        
        println!("Summary:");
        println!("  On Track: {}", on_track);
        println!("  At Risk: {}", at_risk);
        println!("  Early: {}", early);
        println!("  Late: {}", late);
        
        Ok(())
    }

    /// Calculate when a milestone should complete based on its dependencies
    fn calculate_milestone_dependency_date(&self, milestone: &crate::data::Milestone, tasks: &[crate::data::Task]) -> Option<DateTime<Utc>> {
        if milestone.dependencies.is_empty() {
            return None;
        }
        
        let mut latest_date: Option<DateTime<Utc>> = None;
        
        for dependency in &milestone.dependencies {
            let predecessor_end_date = if let Some(task) = self.repository.find_task_by_id(dependency.predecessor_id) {
                // For tasks, use due date or estimate completion based on progress
                if let Some(due_date) = task.due_date {
                    Some(due_date)
                } else {
                    // Estimate completion based on start date and progress
                    task.start_date.map(|start| {
                        let estimated_duration_days = task.duration_days()
                            .unwrap_or_else(|| (task.estimated_hours / 8.0).ceil() as i64);
                        let remaining_work = (100.0 - task.progress_percentage) / 100.0;
                        let remaining_days = (estimated_duration_days as f64 * remaining_work).ceil() as i64;
                        start + chrono::Duration::days(remaining_days)
                    })
                }
            } else if let Some(milestone_dep) = self.repository.find_milestone_by_id(dependency.predecessor_id) {
                // For milestone dependencies, use target date
                Some(milestone_dep.target_date)
            } else {
                None
            };
            
            if let Some(end_date) = predecessor_end_date {
                // Apply lag/lead time
                let adjusted_date = if dependency.lag_days != 0.0 {
                    end_date + chrono::Duration::days(dependency.lag_days.ceil() as i64)
                } else {
                    end_date
                };
                
                match dependency.dependency_type {
                    crate::data::DependencyType::FinishToStart => {
                        // Milestone can start after predecessor finishes
                        if latest_date.is_none() || adjusted_date > latest_date.unwrap() {
                            latest_date = Some(adjusted_date);
                        }
                    },
                    crate::data::DependencyType::FinishToFinish => {
                        // Milestone must finish when predecessor finishes
                        if latest_date.is_none() || adjusted_date > latest_date.unwrap() {
                            latest_date = Some(adjusted_date);
                        }
                    },
                    _ => {
                        // For other dependency types, use finish-to-start logic
                        if latest_date.is_none() || adjusted_date > latest_date.unwrap() {
                            latest_date = Some(adjusted_date);
                        }
                    }
                }
            }
        }
        
        latest_date
    }

    /// Show milestone alerts for items that are at risk or overdue
    pub fn show_milestone_alerts(&self) -> Result<()> {
        let milestones = self.repository.get_milestones();
        let tasks = self.repository.get_tasks();
        
        let mut alerts = Vec::new();
        
        for milestone in milestones {
            let now = Utc::now();
            let days_to_target = (milestone.target_date - now).num_days();
            
            // Check if overdue
            if milestone.is_overdue() {
                alerts.push(format!("🔴 OVERDUE: '{}' was due {} days ago", 
                                  milestone.name, -days_to_target));
            }
            // Check if at risk (within 7 days)
            else if days_to_target <= 7 && days_to_target > 0 {
                alerts.push(format!("⚠️ AT RISK: '{}' is due in {} days", 
                                  milestone.name, days_to_target));
            }
            // Check dependency-driven schedule
            else if let Some(dependency_date) = self.calculate_milestone_dependency_date(milestone, tasks) {
                let dependency_variance = (dependency_date - milestone.target_date).num_days();
                if dependency_variance > 0 {
                    alerts.push(format!("📅 DEPENDENCY RISK: '{}' dependencies suggest {} days late", 
                                      milestone.name, dependency_variance));
                } else if dependency_variance < -7 {
                    alerts.push(format!("🟢 EARLY OPPORTUNITY: '{}' could complete {} days early", 
                                      milestone.name, -dependency_variance));
                }
            }
        }
        
        if alerts.is_empty() {
            println!("✅ All milestones are on track!");
        } else {
            println!("Milestone Alerts:");
            println!("================");
            for alert in alerts {
                println!("{}", alert);
            }
        }
        
        Ok(())
    }

    /// Compare baselines interactively
    pub fn compare_baselines_interactive(&self) -> Result<()> {
        let baselines = self.repository.get_baselines();
        
        if baselines.len() < 2 {
            println!("Need at least 2 baselines to compare.");
            return Ok(());
        }

        // Create baseline options
        let default_desc = "No description".to_string();
        let baseline_options: Vec<String> = baselines.iter()
            .map(|b| format!("{} - {} ({})", b.name, 
                            b.description.as_ref().unwrap_or(&default_desc),
                            b.baseline_type))
            .collect();

        let selected1 = Select::new("Select first baseline:", baseline_options.clone())
            .with_help_message("Choose the baseline to compare from")
            .prompt()?;

        let baseline1_index = baseline_options.iter().position(|x| x == &selected1).unwrap();
        let baseline1_id = baselines[baseline1_index].id;

        let selected2 = Select::new("Select second baseline:", baseline_options.clone())
            .with_help_message("Choose the baseline to compare to")
            .prompt()?;

        let baseline2_index = baseline_options.iter().position(|x| x == &selected2).unwrap();
        let baseline2_id = baselines[baseline2_index].id;

        if baseline1_id == baseline2_id {
            println!("Cannot compare a baseline to itself.");
            return Ok(());
        }

        // Perform comparison
        let baseline1 = &baselines[baseline1_index];
        let baseline2 = &baselines[baseline2_index];
        let comparison = baseline1.compare_to(baseline2);

        // Display comparison results
        println!("\nBaseline Comparison Report");
        println!("=========================");
        println!("From: {} ({:?})", baseline1.name, baseline1.baseline_type);
        println!("To: {} ({:?})", baseline2.name, baseline2.baseline_type);
        println!("Generated: {}", comparison.generated_date.format("%Y-%m-%d %H:%M"));
        
        println!("\nSummary:");
        println!("  Schedule Variance: {} days", comparison.schedule_variance_days);
        println!("  Cost Variance: ${:.2}", comparison.cost_variance);
        println!("  Effort Variance: {:.1} hours", comparison.effort_variance_hours);

        // Task changes
        if !comparison.task_changes.is_empty() {
            println!("\nTask Changes:");
            for task_change in &comparison.task_changes {
                let variance_symbol = match task_change.variance_type {
                    crate::baseline::VarianceType::NoChange => "✓",
                    crate::baseline::VarianceType::ScheduleVariance => "📅",
                    crate::baseline::VarianceType::CostVariance => "💰",
                    crate::baseline::VarianceType::ScopeChange => "🎯",
                    crate::baseline::VarianceType::TaskAdded => "➕",
                    crate::baseline::VarianceType::TaskRemoved => "➖",
                };

                println!("  {} {} - {:?}", variance_symbol, task_change.task_name, task_change.variance_type);
                
                if task_change.schedule_variance_days != 0 {
                    println!("    Schedule: {} days", task_change.schedule_variance_days);
                }
                if task_change.cost_variance != 0.0 {
                    println!("    Cost: ${:.2}", task_change.cost_variance);
                }
                if task_change.effort_variance_hours != 0.0 {
                    println!("    Effort: {:.1} hours", task_change.effort_variance_hours);
                }
            }
        }

        // Milestone changes
        if !comparison.milestone_changes.is_empty() {
            println!("\nMilestone Changes:");
            for milestone_change in &comparison.milestone_changes {
                let status_symbol = match milestone_change.status {
                    crate::baseline::MilestoneStatus::OnTrack => "✅",
                    crate::baseline::MilestoneStatus::AtRisk => "⚠️",
                    crate::baseline::MilestoneStatus::Delayed => "🔴",
                    crate::baseline::MilestoneStatus::Completed => "🟢",
                    crate::baseline::MilestoneStatus::Cancelled => "❌",
                };

                println!("  {} {} - {:?}", status_symbol, milestone_change.milestone_name, milestone_change.status);
                println!("    Baseline: {}", milestone_change.baseline_date.format("%Y-%m-%d"));
                if let Some(current_date) = milestone_change.current_date {
                    println!("    Current: {}", current_date.format("%Y-%m-%d"));
                    println!("    Variance: {} days", milestone_change.variance_days);
                } else {
                    println!("    Current: Not scheduled");
                }
            }
        }

        Ok(())
    }
}