use crate::commands::{
    execute_requirements_command, execute_risk_command, execute_verification_command,
    execute_pm_command, execute_tol_command, execute_team_command
};
use crate::{RequirementsCommands, RiskCommands, VerificationCommands, PmCommands, TolCommands, TeamCommands};
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use inquire::Select;

pub async fn run_interactive_mode(mut project_ctx: ProjectContext, module: Option<String>) -> Result<()> {
    println!("{}", "Welcome to Tessera Interactive Mode".bold().blue());
    println!("Project: {}", project_ctx.metadata.name);
    
    match module {
        Some(ref m) if m == "requirements" => {
            project_ctx.set_current_module("requirements".to_string());
            run_requirements_interactive(project_ctx).await?;
        },
        Some(ref m) if m == "risk" => {
            project_ctx.set_current_module("risk".to_string());
            run_risk_interactive(project_ctx).await?;
        },
        Some(ref m) if m == "verification" => {
            project_ctx.set_current_module("verification".to_string());
            run_verification_interactive(project_ctx).await?;
        },
        Some(ref m) if m == "pm" => {
            project_ctx.set_current_module("pm".to_string());
            run_pm_interactive(project_ctx).await?;
        },
        Some(ref m) if m == "tol" => {
            project_ctx.set_current_module("tol".to_string());
            run_tol_interactive(project_ctx).await?;
        },
        Some(ref m) if m == "team" => {
            project_ctx.set_current_module("team".to_string());
            run_team_interactive(project_ctx).await?;
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
            "Team Management",
            "Project Management",
            "Requirements Management",
            "Risk Management",
            "Tolerance Analysis",
            "Verification & Testing",
            "Program Status",
            "Exit",
        ];
        
        let selection = Select::new("Select module:", options)
            .with_help_message("Use arrow keys to navigate, Enter to select")
            .prompt()?;
        
        match selection {
            "Team Management" => {
                run_team_interactive(project_ctx.clone()).await?;
            },
            "Project Management" => {
                run_pm_interactive(project_ctx.clone()).await?;
            },
            "Requirements Management" => {
                run_requirements_interactive(project_ctx.clone()).await?;
            },
            "Risk Management" => {
                run_risk_interactive(project_ctx.clone()).await?;
            },
            "Tolerance Analysis" => {
                run_tol_interactive(project_ctx.clone()).await?;
            },
            "Verification & Testing" => {
                run_verification_interactive(project_ctx.clone()).await?;
            },
            "Program Status" => {
                show_program_status(&project_ctx)?;
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

async fn run_requirements_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Requirements Management".bold().blue());
        
        let options = vec![
            "📝 Requirements",
            "📥 Design Inputs",
            "📤 Design Outputs",
            "✅ Verifications",
            "📊 Dashboard",
            "← Back to Main Menu",
        ];
        
        let selection = Select::new("Select category:", options)
            .with_help_message("Choose a requirements management category")
            .prompt()?;
        
        let result = match selection {
            "📝 Requirements" => {
                run_requirements_submenu(project_ctx.clone()).await
            },
            "📥 Design Inputs" => {
                run_design_inputs_submenu(project_ctx.clone()).await
            },
            "📤 Design Outputs" => {
                run_design_outputs_submenu(project_ctx.clone()).await
            },
            "✅ Verifications" => {
                run_verifications_submenu(project_ctx.clone()).await
            },
            "📊 Dashboard" => {
                execute_requirements_command(RequirementsCommands::Dashboard, project_ctx.clone()).await
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

async fn run_risk_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Risk Management".bold().blue());
        
        let options = vec![
            "⚠️  Risks",
            "🛡️  Design Controls",
            "📊 Risk Assessment",
            "📈 Dashboard",
            "← Back to Main Menu",
        ];
        
        let selection = Select::new("Select category:", options)
            .with_help_message("Choose a risk management category")
            .prompt()?;
        
        let result = match selection {
            "⚠️  Risks" => {
                run_risks_submenu(project_ctx.clone()).await
            },
            "🛡️  Design Controls" => {
                run_design_controls_submenu(project_ctx.clone()).await
            },
            "📊 Risk Assessment" => {
                execute_risk_command(RiskCommands::AssessRisks, project_ctx.clone()).await
            },
            "📈 Dashboard" => {
                execute_risk_command(RiskCommands::Dashboard, project_ctx.clone()).await
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

async fn run_verification_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Verification & Testing".bold().blue());
        
        let options = vec![
            "📋 Test Procedures",
            "🔄 Test Executions",
            "📊 Dashboard",
            "📄 Generate Report",
            "← Back to Main Menu",
        ];
        
        let selection = Select::new("Select category:", options)
            .with_help_message("Choose a verification category")
            .prompt()?;
        
        let result = match selection {
            "📋 Test Procedures" => {
                execute_verification_command(VerificationCommands::ListProcedures, project_ctx.clone()).await
            },
            "🔄 Test Executions" => {
                execute_verification_command(VerificationCommands::ListExecutions, project_ctx.clone()).await
            },
            "📊 Dashboard" => {
                execute_verification_command(VerificationCommands::Dashboard, project_ctx.clone()).await
            },
            "📄 Generate Report" => {
                execute_verification_command(VerificationCommands::GenerateReport, project_ctx.clone()).await
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



fn show_program_status(project_ctx: &ProjectContext) -> Result<()> {
    println!("\n{}", "Program Status".bold().blue());
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
            "⚠️  Project Risk Management",
            "🐛 Issue Tracking", 
            "📊 Baselines",
            "📅 Calendars",
            "🔍 Critical Path Analysis",
            "💰 Cost Analysis",
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
            "⚠️  Project Risk Management" => {
                run_pm_risk_menu(project_ctx.clone()).await
            },
            "🐛 Issue Tracking" => {
                run_pm_issue_menu(project_ctx.clone()).await
            },
            "📊 Baselines" => {
                run_pm_baseline_menu(project_ctx.clone()).await
            },
            "📅 Calendars" => {
                run_pm_calendar_menu(project_ctx.clone()).await
            },
            "🔍 Critical Path Analysis" => {
                run_pm_critical_path_menu(project_ctx.clone()).await
            },
            "💰 Cost Analysis" => {
                execute_pm_command(PmCommands::CostAnalysis, project_ctx.clone()).await
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
                    ("Edit Task", PmCommands::EditTask),
                    ("Delete Task", PmCommands::DeleteTask),
                ], project_ctx.clone()).await
            },
            "👥 Resources" => {
                run_pm_entity_actions_menu("Resources", &[
                    ("Add Resource", PmCommands::AddResource),
                    ("List Resources", PmCommands::ListResources),
                    ("Edit Resource", PmCommands::EditResource),
                    ("Delete Resource", PmCommands::DeleteResource),
                ], project_ctx.clone()).await
            },
            "🏁 Milestones" => {
                run_pm_entity_actions_menu("Milestones", &[
                    ("Add Milestone", PmCommands::AddMilestone),
                    ("List Milestones", PmCommands::ListMilestones),
                    ("Edit Milestone", PmCommands::EditMilestone),
                    ("Delete Milestone", PmCommands::DeleteMilestone),
                    ("Check Milestone Status", PmCommands::CheckMilestoneStatus),
                    ("Show Milestone Alerts", PmCommands::ShowMilestoneAlerts),
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
                    ("List Features", TolCommands::ListFeatures),
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
                    ("Delete Stackup", TolCommands::DeleteStackup),
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
        println!("\n{}", "Tolerance Analysis - Analysis Tools".bold().blue());
        
        let options = vec![
            "🚀 Run New Analysis",
            "📋 List Previous Results",
            "🗑️ Delete Analysis Results",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose analysis action")
            .prompt()?;
        
        let result = match selection {
            "🚀 Run New Analysis" => {
                execute_tol_command(TolCommands::RunAnalysis, project_ctx.clone()).await
            },
            "📋 List Previous Results" => {
                execute_tol_command(TolCommands::ListAnalysis, project_ctx.clone()).await
            },
            "🗑️ Delete Analysis Results" => {
                execute_tol_command(TolCommands::DeleteAnalysis, project_ctx.clone()).await
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

// Requirements Management Submenus

async fn run_requirements_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Requirements Management - Requirements".bold().blue());
        
        let options = vec![
            "➕ Add Requirement",
            "📋 List Requirements", 
            "✏️  Edit Requirement",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with requirements")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Requirement" => {
                execute_requirements_command(RequirementsCommands::AddRequirement, project_ctx.clone()).await
            },
            "📋 List Requirements" => {
                execute_requirements_command(RequirementsCommands::ListRequirements, project_ctx.clone()).await
            },
            "✏️  Edit Requirement" => {
                execute_requirements_command(RequirementsCommands::EditRequirement, project_ctx.clone()).await
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

async fn run_design_inputs_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Requirements Management - Design Inputs".bold().blue());
        
        let options = vec![
            "➕ Add Design Input",
            "📋 List Design Inputs",
            "✏️  Edit Design Input", 
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with design inputs")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Design Input" => {
                execute_requirements_command(RequirementsCommands::AddInput, project_ctx.clone()).await
            },
            "📋 List Design Inputs" => {
                execute_requirements_command(RequirementsCommands::ListInputs, project_ctx.clone()).await
            },
            "✏️  Edit Design Input" => {
                execute_requirements_command(RequirementsCommands::EditInput, project_ctx.clone()).await
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

async fn run_design_outputs_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Requirements Management - Design Outputs".bold().blue());
        
        let options = vec![
            "➕ Add Design Output",
            "📋 List Design Outputs",
            "✏️  Edit Design Output",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with design outputs")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Design Output" => {
                execute_requirements_command(RequirementsCommands::AddOutput, project_ctx.clone()).await
            },
            "📋 List Design Outputs" => {
                execute_requirements_command(RequirementsCommands::ListOutputs, project_ctx.clone()).await
            },
            "✏️  Edit Design Output" => {
                execute_requirements_command(RequirementsCommands::EditOutput, project_ctx.clone()).await
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

async fn run_verifications_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Requirements Management - Verifications".bold().blue());
        
        let options = vec![
            "➕ Add Verification",
            "📋 List Verifications",
            "✏️  Edit Verification",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with verifications")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Verification" => {
                execute_requirements_command(RequirementsCommands::AddVerification, project_ctx.clone()).await
            },
            "📋 List Verifications" => {
                execute_requirements_command(RequirementsCommands::ListVerifications, project_ctx.clone()).await
            },
            "✏️  Edit Verification" => {
                execute_requirements_command(RequirementsCommands::EditVerification, project_ctx.clone()).await
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

// Risk Management Submenus

async fn run_risks_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Risk Management - Risks".bold().blue());
        
        let options = vec![
            "➕ Add Risk",
            "📋 List Risks",
            "✏️  Edit Risk",
            "📊 Risk Scoring",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with risks")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Risk" => {
                execute_risk_command(RiskCommands::AddRisk, project_ctx.clone()).await
            },
            "📋 List Risks" => {
                execute_risk_command(RiskCommands::ListRisks, project_ctx.clone()).await
            },
            "✏️  Edit Risk" => {
                execute_risk_command(RiskCommands::EditRisk, project_ctx.clone()).await
            },
            "📊 Risk Scoring" => {
                execute_risk_command(RiskCommands::RiskScoring, project_ctx.clone()).await
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

async fn run_design_controls_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Risk Management - Design Controls".bold().blue());
        
        let options = vec![
            "➕ Add Design Control",
            "📋 List Design Controls",
            "✏️  Edit Design Control",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with design controls")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Design Control" => {
                execute_risk_command(RiskCommands::AddControl, project_ctx.clone()).await
            },
            "📋 List Design Controls" => {
                execute_risk_command(RiskCommands::ListControls, project_ctx.clone()).await
            },
            "✏️  Edit Design Control" => {
                execute_risk_command(RiskCommands::EditControl, project_ctx.clone()).await
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

// Team Management Functions
async fn run_team_interactive(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Team Management".bold().blue());
        
        let options = vec![
            "👥 Team Members",
            "🎭 Roles",
            "🏢 Teams",
            "✅ Git Integration",
            "📊 Dashboard",
            "← Back to Main Menu",
        ];
        
        let selection = Select::new("Select category:", options)
            .with_help_message("Choose a team management category")
            .prompt()?;
        
        let result = match selection {
            "👥 Team Members" => {
                run_team_members_submenu(project_ctx.clone()).await
            },
            "🎭 Roles" => {
                run_roles_submenu(project_ctx.clone()).await
            },
            "🏢 Teams" => {
                run_teams_submenu(project_ctx.clone()).await
            },
            "✅ Git Integration" => {
                run_git_integration_submenu(project_ctx.clone()).await
            },
            "📊 Dashboard" => {
                execute_team_command(TeamCommands::Dashboard, project_ctx.clone()).await
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

async fn run_team_members_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Team Management - Team Members".bold().blue());
        
        let options = vec![
            "➕ Add Team Member",
            "📋 List Team Members",
            "✏️  Edit Team Member",
            "🗑️  Remove Team Member",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with team members")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Team Member" => {
                execute_team_command(TeamCommands::AddMember, project_ctx.clone()).await
            },
            "📋 List Team Members" => {
                execute_team_command(TeamCommands::ListMembers, project_ctx.clone()).await
            },
            "✏️  Edit Team Member" => {
                execute_team_command(TeamCommands::EditMember, project_ctx.clone()).await
            },
            "🗑️  Remove Team Member" => {
                execute_team_command(TeamCommands::DeactivateMember, project_ctx.clone()).await
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

async fn run_roles_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Team Management - Roles".bold().blue());
        
        let options = vec![
            "➕ Add Role",
            "📋 List Roles",
            "✏️  Edit Role",
            "🗑️  Remove Role",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with roles")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Role" => {
                execute_team_command(TeamCommands::AddRole, project_ctx.clone()).await
            },
            "📋 List Roles" => {
                execute_team_command(TeamCommands::ListRoles, project_ctx.clone()).await
            },
            "✏️  Edit Role" => {
                execute_team_command(TeamCommands::EditRole, project_ctx.clone()).await
            },
            "🗑️  Remove Role" => {
                execute_team_command(TeamCommands::RemoveRole, project_ctx.clone()).await
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

async fn run_teams_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Team Management - Teams".bold().blue());
        
        let options = vec![
            "➕ Create Team",
            "📋 List Teams",
            "✏️  Edit Team",
            "🗑️  Remove Team",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with teams")
            .prompt()?;
        
        let result = match selection {
            "➕ Create Team" => {
                execute_team_command(TeamCommands::AddTeam, project_ctx.clone()).await
            },
            "📋 List Teams" => {
                execute_team_command(TeamCommands::ListTeams, project_ctx.clone()).await
            },
            "✏️  Edit Team" => {
                execute_team_command(TeamCommands::EditTeam, project_ctx.clone()).await
            },
            "🗑️  Remove Team" => {
                execute_team_command(TeamCommands::RemoveMemberFromTeam, project_ctx.clone()).await
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

async fn run_git_integration_submenu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Team Management - Git Integration".bold().blue());
        
        let options = vec![
            "📝 Generate CODEOWNERS",
            "🔄 Sync with Git Teams",
            "✅ Validate Git Setup",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose git integration actions")
            .prompt()?;
        
        let result = match selection {
            "📝 Generate CODEOWNERS" => {
                execute_team_command(TeamCommands::GenerateCodeowners, project_ctx.clone()).await
            },
            "🔄 Sync with Git Teams" => {
                execute_team_command(TeamCommands::GitSync, project_ctx.clone()).await
            },
            "✅ Validate Git Setup" => {
                execute_team_command(TeamCommands::Validate, project_ctx.clone()).await
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


// PM Project Risk Management Menu
async fn run_pm_risk_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Project Management - Project Risk Management".bold().blue());
        
        let options = vec![
            "➕ Add Project Risk",
            "📋 List Project Risks",
            "✏️  Edit Project Risk",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with project risks (schedule/cost risks)")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Project Risk" => {
                execute_pm_command(PmCommands::AddRisk, project_ctx.clone()).await
            },
            "📋 List Project Risks" => {
                execute_pm_command(PmCommands::ListRisks, project_ctx.clone()).await
            },
            "✏️  Edit Project Risk" => {
                execute_pm_command(PmCommands::EditRisk, project_ctx.clone()).await
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

// PM Issue Tracking Menu
async fn run_pm_issue_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Project Management - Issue Tracking".bold().blue());
        
        let options = vec![
            "➕ Add Issue",
            "📋 List Issues",
            "✏️  Edit Issue",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with project issues")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Issue" => {
                execute_pm_command(PmCommands::AddIssue, project_ctx.clone()).await
            },
            "📋 List Issues" => {
                execute_pm_command(PmCommands::ListIssues, project_ctx.clone()).await
            },
            "✏️  Edit Issue" => {
                execute_pm_command(PmCommands::EditIssue, project_ctx.clone()).await
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

// PM Baseline Management Menu
async fn run_pm_baseline_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Project Management - Baseline Management".bold().blue());
        
        let options = vec![
            "📊 Create Baseline",
            "📋 List Baselines",
            "🔄 Compare Baselines",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with project baselines")
            .prompt()?;
        
        let result = match selection {
            "📊 Create Baseline" => {
                execute_pm_command(PmCommands::CreateBaseline, project_ctx.clone()).await
            },
            "📋 List Baselines" => {
                execute_pm_command(PmCommands::ListBaselines, project_ctx.clone()).await
            },
            "🔄 Compare Baselines" => {
                execute_pm_command(PmCommands::CompareBaselines, project_ctx.clone()).await
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

// PM Calendar Management Menu
async fn run_pm_calendar_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Project Management - Calendar Management".bold().blue());
        
        let options = vec![
            "➕ Add Calendar",
            "📋 List Calendars",
            "✏️  Edit Calendar",
            "🔗 Assign Calendar to Resource",
            "📋 List Calendar Assignments",
            "🔓 Remove Calendar Assignment",
            "← Back",
        ];
        
        let selection = Select::new("Select action:", options)
            .with_help_message("Choose what to do with project calendars")
            .prompt()?;
        
        let result = match selection {
            "➕ Add Calendar" => {
                execute_pm_command(PmCommands::AddCalendar, project_ctx.clone()).await
            },
            "📋 List Calendars" => {
                execute_pm_command(PmCommands::ListCalendars, project_ctx.clone()).await
            },
            "✏️  Edit Calendar" => {
                execute_pm_command(PmCommands::EditCalendar, project_ctx.clone()).await
            },
            "🔗 Assign Calendar to Resource" => {
                execute_pm_command(PmCommands::AssignCalendar, project_ctx.clone()).await
            },
            "📋 List Calendar Assignments" => {
                execute_pm_command(PmCommands::ListCalendarAssignments, project_ctx.clone()).await
            },
            "🔓 Remove Calendar Assignment" => {
                execute_pm_command(PmCommands::UnassignCalendar, project_ctx.clone()).await
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


// PM Critical Path Analysis Menu
async fn run_pm_critical_path_menu(project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Project Management - Critical Path Analysis".bold().blue());
        
        let options = vec![
            "📋 Analyze Task Critical Path",
            "🏁 Analyze Milestone Critical Path",
            "← Back",
        ];
        
        let selection = Select::new("Select analysis type:", options)
            .with_help_message("Choose what to analyze the critical path for")
            .prompt()?;
        
        let result = match selection {
            "📋 Analyze Task Critical Path" => {
                execute_pm_command(PmCommands::AnalyzeTaskCriticalPath, project_ctx.clone()).await
            },
            "🏁 Analyze Milestone Critical Path" => {
                execute_pm_command(PmCommands::AnalyzeMilestoneCriticalPath, project_ctx.clone()).await
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
