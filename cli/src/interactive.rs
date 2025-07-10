use crate::commands::quality::execute_quality_command;
use crate::commands::pm::execute_pm_command;
use crate::commands::tol::execute_tol_command;
use crate::{QualityCommands, PmCommands, TolCommands};
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use inquire::Select;

pub async fn run_interactive_mode(mut project_ctx: ProjectContext, module: Option<String>) -> Result<()> {
    println!("{}", "Welcome to Tessera Interactive Mode".bold().blue());
    println!("Project: {}", project_ctx.metadata.name);
    
    match module {
        Some(ref m) if m == "quality" => {
            project_ctx.set_current_module("quality".to_string());
            run_quality_interactive(project_ctx).await?;
        },
        Some(ref m) if m == "pm" => {
            project_ctx.set_current_module("pm".to_string());
            run_pm_interactive(project_ctx).await?;
        },
        Some(ref m) if m == "tol" => {
            project_ctx.set_current_module("tol".to_string());
            run_tol_interactive(project_ctx).await?;
        },
        Some(ref m) => {
            println!("{} Module '{}' not recognized", "⚠".yellow(), m);
        },
        None => {
            run_main_interactive(project_ctx).await?;
        }
    }
    
    Ok(())
}

async fn run_main_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        let options = vec![
            "Quality Management",
            "Project Management", 
            "Tolerance Analysis",
            "Project Status",
            "Exit",
        ];
        
        let selection = Select::new("Select module:", options)
            .with_help_message("Use arrow keys to navigate, Enter to select")
            .prompt()?;
        
        match selection {
            "Quality Management" => {
                run_quality_interactive(project_ctx.clone()).await?;
            },
            "Project Management" => {
                run_pm_interactive(project_ctx.clone()).await?;
            },
            "Tolerance Analysis" => {
                run_tol_interactive(project_ctx.clone()).await?;
            },
            "Project Status" => {
                show_project_status(&project_ctx)?;
            },
            "Exit" => {
                println!("{}", "Goodbye!".green());
                break;
            },
            _ => {}
        }
    }
    
    Ok(())
}

async fn run_quality_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Quality Management".bold().blue());
        
        let options = vec![
            "📋 Manage Entities",
            "🔗 Link Entities", 
            "📊 Analysis Tools",
            "📈 Dashboard",
            "← Back to Main Menu",
        ];
        
        let selection = Select::new("Select category:", options)
            .with_help_message("Choose a quality management category")
            .prompt()?;
        
        let result = match selection {
            "📋 Manage Entities" => {
                run_quality_manage_menu(project_ctx.clone()).await
            },
            "🔗 Link Entities" => {
                run_quality_link_menu(project_ctx.clone()).await
            },
            "📊 Analysis Tools" => {
                run_quality_analysis_menu(project_ctx.clone()).await
            },
            "📈 Dashboard" => {
                execute_quality_command(QualityCommands::Dashboard, project_ctx.clone()).await
            },
            "← Back to Main Menu" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn run_quality_manage_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Quality Management - Manage Entities".bold().blue());
        
        let options = vec![
            "📝 Requirements",
            "📥 Design Inputs",
            "📤 Design Outputs", 
            "🎯 Design Controls",
            "⚠️  Risks",
            "← Back",
        ];
        
        let selection = Select::new("Select entity type:", options)
            .with_help_message("Choose what to manage")
            .prompt()?;
        
        let result = match selection {
            "📝 Requirements" => {
                run_entity_actions_menu("Requirements", &[
                    ("Add Requirement", QualityCommands::AddRequirement),
                    ("List Requirements", QualityCommands::ListRequirements),
                ], project_ctx.clone()).await
            },
            "📥 Design Inputs" => {
                run_entity_actions_menu("Design Inputs", &[
                    ("Add Design Input", QualityCommands::AddInput),
                    ("List Design Inputs", QualityCommands::ListInputs),
                ], project_ctx.clone()).await
            },
            "📤 Design Outputs" => {
                run_entity_actions_menu("Design Outputs", &[
                    ("Add Design Output", QualityCommands::AddOutput),
                    ("List Design Outputs", QualityCommands::ListOutputs),
                ], project_ctx.clone()).await
            },
            "🎯 Design Controls" => {
                run_entity_actions_menu("Design Controls", &[
                    ("Add Design Control", QualityCommands::AddControl),
                    ("List Design Controls", QualityCommands::ListControls),
                ], project_ctx.clone()).await
            },
            "⚠️  Risks" => {
                run_entity_actions_menu("Risks", &[
                    ("Add Risk", QualityCommands::AddRisk),
                    ("List Risks", QualityCommands::ListRisks),
                ], project_ctx.clone()).await
            },
            "← Back" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn run_entity_actions_menu(entity_type: &str, actions: &[(&str, QualityCommands)], project_ctx: ProjectContext) -> Result<()> {
    println!("\n{}", format!("Quality Management - {}", entity_type).bold().blue());
    
    let mut options: Vec<String> = actions.iter().map(|(name, _)| name.to_string()).collect();
    options.push("← Back".to_string());
    
    let selection = Select::new("Select action:", options)
        .with_help_message(&format!("Choose action for {}", entity_type.to_lowercase()))
        .prompt()?;
    
    if selection == "← Back" {
        return Ok(());
    }
    
    for (action_name, command) in actions {
        if selection == *action_name {
            execute_quality_command(command.clone(), project_ctx).await?;
            break;
        }
    }
    
    Ok(())
}

async fn run_quality_link_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Quality Management - Link Entities".bold().blue());
        
        let options = vec![
            "📥➡️📝 Link Input to Requirement",
            "📤➡️📝 Link Output to Requirement",
            "📤➡️📥 Link Output to Input",
            "🎯➡️📤 Link Control to Output",
            "← Back",
        ];
        
        let selection = Select::new("Select linking action:", options)
            .with_help_message("Choose entities to link")
            .prompt()?;
        
        let result = match selection {
            "📥➡️📝 Link Input to Requirement" => {
                execute_quality_command(QualityCommands::LinkInputToRequirement, project_ctx.clone()).await
            },
            "📤➡️📝 Link Output to Requirement" => {
                execute_quality_command(QualityCommands::LinkOutputToRequirement, project_ctx.clone()).await
            },
            "📤➡️📥 Link Output to Input" => {
                execute_quality_command(QualityCommands::LinkOutputToInput, project_ctx.clone()).await
            },
            "🎯➡️📤 Link Control to Output" => {
                execute_quality_command(QualityCommands::LinkControlToOutput, project_ctx.clone()).await
            },
            "← Back" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn run_quality_analysis_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Quality Management - Analysis Tools".bold().blue());
        
        let options = vec![
            "⚠️  Assess Risks",
            "📊 Risk Scoring Tools",
            "🔍 Traceability Matrix",
            "← Back",
        ];
        
        let selection = Select::new("Select analysis tool:", options)
            .with_help_message("Choose analysis to run")
            .prompt()?;
        
        let result = match selection {
            "⚠️  Assess Risks" => {
                execute_quality_command(QualityCommands::AssessRisks, project_ctx.clone()).await
            },
            "📊 Risk Scoring Tools" => {
                execute_quality_command(QualityCommands::RiskScoring, project_ctx.clone()).await
            },
            "🔍 Traceability Matrix" => {
                execute_quality_command(QualityCommands::TraceabilityMatrix, project_ctx.clone()).await
            },
            "← Back" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

fn show_project_status(project_ctx: &ProjectContext) -> Result<()> {
    println!("\n{}", "Project Status".bold().blue());
    println!("Name: {}", project_ctx.metadata.name);
    println!("Description: {}", project_ctx.metadata.description);
    println!("Version: {}", project_ctx.metadata.version);
    println!("Created: {}", project_ctx.metadata.created.format("%Y-%m-%d %H:%M:%S"));
    println!("Location: {}", project_ctx.root_path.display());
    
    println!("\n{}", "Available Modules:".bold());
    for module in &project_ctx.metadata.modules {
        let module_path = project_ctx.module_path(module);
        let status = if module_path.exists() {
            "✓".green()
        } else {
            "✗".red()
        };
        println!("  {} {}", status, module);
    }
    
    Ok(())
}

async fn run_pm_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Project Management".bold().blue());
        
        let options = vec![
            "📋 Manage Project",
            "📅 Scheduling",
            "📈 Dashboard",
            "← Back to Main Menu",
        ];
        
        let selection = Select::new("Select category:", options)
            .with_help_message("Choose a project management category")
            .prompt()?;
        
        let result = match selection {
            "📋 Manage Project" => {
                run_pm_manage_menu(project_ctx.clone()).await
            },
            "📅 Scheduling" => {
                execute_pm_command(PmCommands::Schedule, project_ctx.clone()).await
            },
            "📈 Dashboard" => {
                execute_pm_command(PmCommands::Dashboard, project_ctx.clone()).await
            },
            "← Back to Main Menu" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn run_pm_manage_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Project Management - Manage Project".bold().blue());
        
        let options = vec![
            "✅ Tasks",
            "👥 Resources",
            "🏁 Milestones",
            "← Back",
        ];
        
        let selection = Select::new("Select what to manage:", options)
            .with_help_message("Choose project elements to manage")
            .prompt()?;
        
        let result = match selection {
            "✅ Tasks" => {
                run_pm_entity_actions_menu("Tasks", &[
                    ("Add Task", PmCommands::AddTask),
                    ("List Tasks", PmCommands::ListTasks),
                ], project_ctx.clone()).await
            },
            "👥 Resources" => {
                execute_pm_command(PmCommands::AddResource, project_ctx.clone()).await
            },
            "🏁 Milestones" => {
                execute_pm_command(PmCommands::AddMilestone, project_ctx.clone()).await
            },
            "← Back" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn run_pm_entity_actions_menu(entity_type: &str, actions: &[(&str, PmCommands)], project_ctx: ProjectContext) -> Result<()> {
    println!("\n{}", format!("Project Management - {}", entity_type).bold().blue());
    
    let mut options: Vec<String> = actions.iter().map(|(name, _)| name.to_string()).collect();
    options.push("← Back".to_string());
    
    let selection = Select::new("Select action:", options)
        .with_help_message(&format!("Choose action for {}", entity_type.to_lowercase()))
        .prompt()?;
    
    if selection == "← Back" {
        return Ok(());
    }
    
    for (action_name, command) in actions {
        if selection == *action_name {
            execute_pm_command(command.clone(), project_ctx).await?;
            break;
        }
    }
    
    Ok(())
}

async fn run_tol_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Tolerance Analysis".bold().blue());
        
        let options = vec![
            "🔧 Manage Model",
            "📊 Run Analysis",
            "📈 Dashboard",
            "← Back to Main Menu",
        ];
        
        let selection = Select::new("Select category:", options)
            .with_help_message("Choose a tolerance analysis category")
            .prompt()?;
        
        let result = match selection {
            "🔧 Manage Model" => {
                run_tol_manage_menu(project_ctx.clone()).await
            },
            "📊 Run Analysis" => {
                run_tol_analysis_menu(project_ctx.clone()).await
            },
            "📈 Dashboard" => {
                execute_tol_command(TolCommands::Dashboard, project_ctx.clone()).await
            },
            "← Back to Main Menu" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn run_tol_manage_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Tolerance Analysis - Manage Model".bold().blue());
        
        let options = vec![
            "📜 Components",
            "📎 Features",
            "🔗 Mates",
            "📊 Stackups",
            "← Back",
        ];
        
        let selection = Select::new("Select what to manage:", options)
            .with_help_message("Choose model elements to manage")
            .prompt()?;
        
        let result = match selection {
            "📜 Components" => {
                run_tol_entity_actions_menu("Components", &[
                    ("Add Component", TolCommands::AddComponent),
                    ("Edit Component", TolCommands::EditComponent),
                    ("List Components", TolCommands::ListComponents),
                ], project_ctx.clone()).await
            },
            "📎 Features" => {
                run_tol_entity_actions_menu("Features", &[
                    ("Add Feature", TolCommands::AddFeature),
                    ("Edit Feature", TolCommands::EditFeature),
                ], project_ctx.clone()).await
            },
            "🔗 Mates" => {
                run_tol_entity_actions_menu("Mates", &[
                    ("Add Mate", TolCommands::AddMate),
                    ("Edit Mate", TolCommands::EditMate),
                    ("List Mates", TolCommands::ListMates),
                ], project_ctx.clone()).await
            },
            "📊 Stackups" => {
                run_tol_entity_actions_menu("Stackups", &[
                    ("Add Stackup", TolCommands::AddStackup),
                    ("Edit Stackup", TolCommands::EditStackup),
                ], project_ctx.clone()).await
            },
            "← Back" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn run_tol_entity_actions_menu(entity_type: &str, actions: &[(&str, TolCommands)], project_ctx: ProjectContext) -> Result<()> {
    println!("\n{}", format!("Tolerance Analysis - {}", entity_type).bold().blue());
    
    let mut options: Vec<String> = actions.iter().map(|(name, _)| name.to_string()).collect();
    options.push("← Back".to_string());
    
    let selection = Select::new("Select action:", options)
        .with_help_message(&format!("Choose action for {}", entity_type.to_lowercase()))
        .prompt()?;
    
    if selection == "← Back" {
        return Ok(());
    }
    
    for (action_name, command) in actions {
        if selection == *action_name {
            execute_tol_command(command.clone(), project_ctx).await?;
            break;
        }
    }
    
    Ok(())
}

async fn run_tol_analysis_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Tolerance Analysis - Run Analysis".bold().blue());
        
        let options = vec![
            "🏃 Run Analysis",
            "⚙️  Configure Analysis Settings",
            "← Back",
        ];
        
        let selection = Select::new("Select analysis action:", options)
            .with_help_message("Choose analysis to run or configure")
            .prompt()?;
        
        let result = match selection {
            "🏃 Run Analysis" => {
                execute_tol_command(TolCommands::RunAnalysis, project_ctx.clone()).await
            },
            "⚙️  Configure Analysis Settings" => {
                execute_tol_command(TolCommands::ConfigureAnalysis, project_ctx.clone()).await
            },
            "← Back" => {
                break;
            },
            _ => Ok(()),
        };
        
        if let Err(e) = result {
            println!("{} Error: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}