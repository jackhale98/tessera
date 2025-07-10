use crate::data::*;
use crate::repository::ProjectRepository;
use crate::scheduling::ProjectScheduler;
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text, Confirm};
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
        
        let estimated_hours_str = Text::new("Estimated hours:")
            .with_default("8.0")
            .prompt()?;
        let estimated_hours: f64 = estimated_hours_str.parse().unwrap_or(8.0);
        
        let priority_options = vec!["Low", "Medium", "High", "Critical"];
        let priority_str = Select::new("Priority:", priority_options).prompt()?;
        let priority = match priority_str {
            "Low" => TaskPriority::Low,
            "Medium" => TaskPriority::Medium,
            "High" => TaskPriority::High,
            "Critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        };
        
        let mut task = Task::new(name, description, work_type);
        task.estimated_hours = estimated_hours;
        task.priority = priority;
        
        // Add dependencies if there are existing tasks
        let existing_tasks = self.repository.get_tasks();
        if !existing_tasks.is_empty() {
            let add_deps = Confirm::new("Add dependencies to other tasks?")
                .with_default(false)
                .prompt()?;
            
            if add_deps {
                loop {
                    let task_options: Vec<String> = existing_tasks.iter()
                        .map(|t| format!("{} - {}", t.name, t.description))
                        .collect();
                    
                    let dep_selection = Select::new("Select dependency:", task_options.clone()).prompt()?;
                    let dep_index = task_options.iter().position(|x| x == &dep_selection).unwrap();
                    let selected_task = &existing_tasks[dep_index];
                    
                    task.dependencies.push(selected_task.id);
                    println!("Added dependency: {}", selected_task.name);
                    
                    let continue_adding = Confirm::new("Add another dependency?")
                        .with_default(false)
                        .prompt()?;
                    
                    if !continue_adding {
                        break;
                    }
                }
            }
        }
        
        self.repository.add_task(task.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Task '{}' added successfully!", task.name);
        println!("ID: {}", task.id);
        
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
    
    pub async fn add_milestone_interactive(&mut self) -> Result<()> {
        let name = Text::new("Milestone name:")
            .prompt()?;
        
        let description = Text::new("Description:")
            .prompt()?;
        
        // For simplicity, use a text input for date instead of date picker
        let target_date_str = Text::new("Target date (YYYY-MM-DD):")
            .with_help_message("Enter target date in format: 2024-12-31")
            .prompt()?;
        
        let target_date = chrono::NaiveDate::parse_from_str(&target_date_str, "%Y-%m-%d")
            .map_err(|e| tessera_core::DesignTrackError::Validation(format!("Invalid date format: {}", e)))?
            .and_hms_opt(12, 0, 0)
            .ok_or_else(|| tessera_core::DesignTrackError::Validation("Invalid time".to_string()))?;
        
        let target_date_utc = Utc.from_utc_datetime(&target_date);
        
        let milestone = Milestone::new(name, description, target_date_utc);
        
        self.repository.add_milestone(milestone.clone())?;
        
        let pm_dir = self.project_context.module_path("pm");
        self.repository.save_to_directory(&pm_dir)?;
        
        println!("✓ Milestone '{}' added successfully!", milestone.name);
        println!("ID: {}", milestone.id);
        println!("Target Date: {}", milestone.target_date.format("%Y-%m-%d"));
        
        Ok(())
    }
    
    pub fn compute_schedule(&mut self) -> Result<()> {
        let tasks = self.repository.get_tasks();
        let resources = self.repository.get_resources();
        
        if tasks.is_empty() {
            println!("No tasks found. Add tasks first.");
            return Ok(());
        }
        
        let scheduler = ProjectScheduler::default();
        let project_start = Utc::now();
        
        println!("Computing project schedule...");
        let schedule = scheduler.compute_schedule(tasks, resources, project_start)?;
        
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
                println!("  - {}", task.name);
            }
        }
        
        println!("\nTask Schedule Summary:");
        for (task_id, schedule_info) in &schedule.task_schedule {
            if let Some(task) = self.repository.find_task_by_id(*task_id) {
                let critical_marker = if schedule_info.is_critical { " [CRITICAL]" } else { "" };
                println!("  {} - Start: {}, Finish: {}, Slack: {} days{}",
                         task.name,
                         schedule_info.earliest_start.format("%Y-%m-%d"),
                         schedule_info.earliest_finish.format("%Y-%m-%d"),
                         schedule_info.slack_days,
                         critical_marker);
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
                println!("   Dependencies: {} tasks", task.dependencies.len());
            }
            
            if task.progress_percentage > 0.0 {
                println!("   Progress: {:.1}%", task.progress_percentage);
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
        println!("Overdue Tasks: {}", health.overdue_tasks);
        println!("Overdue Milestones: {}", health.overdue_milestones);
        
        let resources = self.repository.get_resources();
        println!("Resources: {}", resources.len());
        
        let milestones = self.repository.get_milestones();
        println!("Milestones: {}", milestones.len());
        
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
}