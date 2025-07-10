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
            "Add Requirement",
            "List Requirements",
            "Add Design Input",
            "List Design Inputs",
            "Add Design Output",
            "List Design Outputs",
            "Add Design Control",
            "List Design Controls",
            "Add Risk",
            "List Risks",
            "Link Input to Requirement",
            "Link Output to Requirement",
            "Link Output to Input",
            "Link Control to Output",
            "Assess Risks",
            "Risk Scoring Tools",
            "Traceability Matrix",
            "Quality Dashboard",
            "Back to Main Menu",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose a quality management action")
            .prompt()?;
        
        let result = match selection {
            "Add Requirement" => {
                execute_quality_command(QualityCommands::AddRequirement, project_ctx.clone()).await
            },
            "List Requirements" => {
                execute_quality_command(QualityCommands::ListRequirements, project_ctx.clone()).await
            },
            "Add Design Input" => {
                execute_quality_command(QualityCommands::AddInput, project_ctx.clone()).await
            },
            "List Design Inputs" => {
                execute_quality_command(QualityCommands::ListInputs, project_ctx.clone()).await
            },
            "Add Design Output" => {
                execute_quality_command(QualityCommands::AddOutput, project_ctx.clone()).await
            },
            "List Design Outputs" => {
                execute_quality_command(QualityCommands::ListOutputs, project_ctx.clone()).await
            },
            "Add Design Control" => {
                execute_quality_command(QualityCommands::AddControl, project_ctx.clone()).await
            },
            "List Design Controls" => {
                execute_quality_command(QualityCommands::ListControls, project_ctx.clone()).await
            },
            "Add Risk" => {
                execute_quality_command(QualityCommands::AddRisk, project_ctx.clone()).await
            },
            "List Risks" => {
                execute_quality_command(QualityCommands::ListRisks, project_ctx.clone()).await
            },
            "Link Input to Requirement" => {
                execute_quality_command(QualityCommands::LinkInputToRequirement, project_ctx.clone()).await
            },
            "Link Output to Requirement" => {
                execute_quality_command(QualityCommands::LinkOutputToRequirement, project_ctx.clone()).await
            },
            "Link Output to Input" => {
                execute_quality_command(QualityCommands::LinkOutputToInput, project_ctx.clone()).await
            },
            "Link Control to Output" => {
                execute_quality_command(QualityCommands::LinkControlToOutput, project_ctx.clone()).await
            },
            "Assess Risks" => {
                execute_quality_command(QualityCommands::AssessRisks, project_ctx.clone()).await
            },
            "Risk Scoring Tools" => {
                execute_quality_command(QualityCommands::RiskScoring, project_ctx.clone()).await
            },
            "Traceability Matrix" => {
                execute_quality_command(QualityCommands::TraceabilityMatrix, project_ctx.clone()).await
            },
            "Quality Dashboard" => {
                execute_quality_command(QualityCommands::Dashboard, project_ctx.clone()).await
            },
            "Back to Main Menu" => {
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
            "Add Task",
            "List Tasks",
            "Add Resource",
            "Add Milestone",
            "Compute Schedule",
            "PM Dashboard",
            "Back to Main Menu",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose a project management action")
            .prompt()?;
        
        let result = match selection {
            "Add Task" => {
                execute_pm_command(PmCommands::AddTask, project_ctx.clone()).await
            },
            "List Tasks" => {
                execute_pm_command(PmCommands::ListTasks, project_ctx.clone()).await
            },
            "Add Resource" => {
                execute_pm_command(PmCommands::AddResource, project_ctx.clone()).await
            },
            "Add Milestone" => {
                execute_pm_command(PmCommands::AddMilestone, project_ctx.clone()).await
            },
            "Compute Schedule" => {
                execute_pm_command(PmCommands::Schedule, project_ctx.clone()).await
            },
            "PM Dashboard" => {
                execute_pm_command(PmCommands::Dashboard, project_ctx.clone()).await
            },
            "Back to Main Menu" => {
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

async fn run_tol_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Tolerance Analysis".bold().blue());
        
        let options = vec![
            "Add Component",
            "Edit Component",
            "List Components",
            "Add Feature",
            "Edit Feature",
            "Add Mate",
            "Edit Mate",
            "List Mates",
            "Add Stackup",
            "Edit Stackup",
            "Run Analysis",
            "Edit Analysis Settings",
            "Tolerance Dashboard",
            "Back to Main Menu",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose a tolerance analysis action")
            .prompt()?;
        
        let result = match selection {
            "Add Component" => {
                execute_tol_command(TolCommands::AddComponent, project_ctx.clone()).await
            },
            "Edit Component" => {
                execute_tol_command(TolCommands::EditComponent, project_ctx.clone()).await
            },
            "List Components" => {
                execute_tol_command(TolCommands::ListComponents, project_ctx.clone()).await
            },
            "Add Feature" => {
                execute_tol_command(TolCommands::AddFeature, project_ctx.clone()).await
            },
            "Edit Feature" => {
                execute_tol_command(TolCommands::EditFeature, project_ctx.clone()).await
            },
            "Add Mate" => {
                execute_tol_command(TolCommands::AddMate, project_ctx.clone()).await
            },
            "Edit Mate" => {
                execute_tol_command(TolCommands::EditMate, project_ctx.clone()).await
            },
            "List Mates" => {
                execute_tol_command(TolCommands::ListMates, project_ctx.clone()).await
            },
            "Add Stackup" => {
                execute_tol_command(TolCommands::AddStackup, project_ctx.clone()).await
            },
            "Edit Stackup" => {
                execute_tol_command(TolCommands::EditStackup, project_ctx.clone()).await
            },
            "Run Analysis" => {
                execute_tol_command(TolCommands::RunAnalysis, project_ctx.clone()).await
            },
            "Edit Analysis Settings" => {
                execute_tol_command(TolCommands::ConfigureAnalysis, project_ctx.clone()).await
            },
            "Tolerance Dashboard" => {
                execute_tol_command(TolCommands::Dashboard, project_ctx.clone()).await
            },
            "Back to Main Menu" => {
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