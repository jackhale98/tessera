use crate::commands::quality::execute_quality_command;
use crate::commands::pm::execute_pm_command;
use crate::commands::tol::execute_tol_command;
use crate::{QualityCommands, PmCommands, TolCommands};
use colored::Colorize;
use tessera_core::{ProjectContext, Result, RiskScoringConfig};
use inquire::{Select, Text, Confirm};

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
            "📊 Analysis Tools",
            "⚙️  Settings",
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
            "📊 Analysis Tools" => {
                run_quality_analysis_menu(project_ctx.clone()).await
            },
            "⚙️  Settings" => {
                run_quality_settings_menu(project_ctx.clone()).await
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
            "✅ Verifications",
            "🛡️  Design Controls",
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
                    ("Edit Requirement", QualityCommands::EditRequirement),
                ], project_ctx.clone()).await
            },
            "📥 Design Inputs" => {
                run_entity_actions_menu("Design Inputs", &[
                    ("Add Design Input", QualityCommands::AddInput),
                    ("List Design Inputs", QualityCommands::ListInputs),
                    ("Edit Design Input", QualityCommands::EditInput),
                ], project_ctx.clone()).await
            },
            "📤 Design Outputs" => {
                run_entity_actions_menu("Design Outputs", &[
                    ("Add Design Output", QualityCommands::AddOutput),
                    ("List Design Outputs", QualityCommands::ListOutputs),
                    ("Edit Design Output", QualityCommands::EditOutput),
                ], project_ctx.clone()).await
            },
            "✅ Verifications" => {
                run_entity_actions_menu("Verifications", &[
                    ("Add Verification", QualityCommands::AddVerification),
                    ("List Verifications", QualityCommands::ListVerifications),
                    ("Edit Verification", QualityCommands::EditVerification),
                ], project_ctx.clone()).await
            },
            "🛡️  Design Controls" => {
                run_entity_actions_menu("Design Controls", &[
                    ("Add Design Control", QualityCommands::AddControl),
                    ("List Design Controls", QualityCommands::ListControls),
                    ("Edit Design Control", QualityCommands::EditControl),
                ], project_ctx.clone()).await
            },
            "⚠️  Risks" => {
                run_entity_actions_menu("Risks", &[
                    ("Add Risk", QualityCommands::AddRisk),
                    ("List Risks", QualityCommands::ListRisks),
                    ("Edit Risk", QualityCommands::EditRisk),
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

async fn run_quality_settings_menu(mut project_ctx: ProjectContext) -> Result<()> {
    loop {
        println!("\n{}", "Quality Management - Settings".bold().blue());
        
        let options = vec![
            "📊 Risk Scoring Configuration",
            "🎯 Risk Tolerance Thresholds", 
            "📋 View Current Settings",
            "← Back",
        ];
        
        let selection = Select::new("Select setting to configure:", options)
            .with_help_message("Choose settings to modify")
            .prompt()?;
        
        let result = match selection {
            "📊 Risk Scoring Configuration" => {
                configure_risk_scoring(&mut project_ctx).await
            },
            "🎯 Risk Tolerance Thresholds" => {
                configure_risk_thresholds(&mut project_ctx).await
            },
            "📋 View Current Settings" => {
                view_current_settings(&project_ctx).await
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

async fn configure_risk_scoring(project_ctx: &mut ProjectContext) -> Result<()> {
    println!("\n{}", "Risk Scoring Configuration".bold().blue());
    
    let configure_prob = Confirm::new("Configure probability range?")
        .with_default(true)
        .prompt()?;
    
    if configure_prob {
        println!("\n{}", "Probability Range Configuration".bold());
        println!("Current: [{}, {}, {}]", 
            project_ctx.metadata.quality_settings.risk_probability_range.range[0],
            project_ctx.metadata.quality_settings.risk_probability_range.range[1],
            project_ctx.metadata.quality_settings.risk_probability_range.range[2]);
        
        let start: i32 = Text::new("Start value:")
            .with_default(&project_ctx.metadata.quality_settings.risk_probability_range.range[0].to_string())
            .prompt()?
            .parse()
            .unwrap_or(1);
        
        let end: i32 = Text::new("End value:")
            .with_default(&project_ctx.metadata.quality_settings.risk_probability_range.range[1].to_string())
            .prompt()?
            .parse()
            .unwrap_or(5);
        
        let step: i32 = Text::new("Step size:")
            .with_default(&project_ctx.metadata.quality_settings.risk_probability_range.range[2].to_string())
            .prompt()?
            .parse()
            .unwrap_or(1);
        
        project_ctx.metadata.quality_settings.risk_probability_range = RiskScoringConfig::new(start, end, step);
        println!("{} Probability range updated to: [{}, {}, {}]", "✓".green(), start, end, step);
    }
    
    let configure_impact = Confirm::new("Configure impact range?")
        .with_default(true)
        .prompt()?;
    
    if configure_impact {
        println!("\n{}", "Impact Range Configuration".bold());
        println!("Current: [{}, {}, {}]", 
            project_ctx.metadata.quality_settings.risk_impact_range.range[0],
            project_ctx.metadata.quality_settings.risk_impact_range.range[1],
            project_ctx.metadata.quality_settings.risk_impact_range.range[2]);
        
        let start: i32 = Text::new("Start value:")
            .with_default(&project_ctx.metadata.quality_settings.risk_impact_range.range[0].to_string())
            .prompt()?
            .parse()
            .unwrap_or(1);
        
        let end: i32 = Text::new("End value:")
            .with_default(&project_ctx.metadata.quality_settings.risk_impact_range.range[1].to_string())
            .prompt()?
            .parse()
            .unwrap_or(5);
        
        let step: i32 = Text::new("Step size:")
            .with_default(&project_ctx.metadata.quality_settings.risk_impact_range.range[2].to_string())
            .prompt()?
            .parse()
            .unwrap_or(1);
        
        project_ctx.metadata.quality_settings.risk_impact_range = RiskScoringConfig::new(start, end, step);
        println!("{} Impact range updated to: [{}, {}, {}]", "✓".green(), start, end, step);
    }
    
    // Save the updated settings
    let project_file = project_ctx.root_path.join("project.ron");
    project_ctx.metadata.save_to_file(project_file)?;
    println!("{} Settings saved to project file", "✓".green());
    
    Ok(())
}

async fn configure_risk_thresholds(project_ctx: &mut ProjectContext) -> Result<()> {
    println!("\n{}", "Risk Tolerance Thresholds Configuration".bold().blue());
    println!("These thresholds determine risk categories based on normalized risk scores (0.0 to 1.0)");
    
    let current = &project_ctx.metadata.quality_settings.risk_tolerance_thresholds;
    println!("Current thresholds:");
    println!("  BAR (Broadly Acceptable): < {:.2}", current.bar_threshold);
    println!("  Tolerable (with reduction): {:.2} - {:.2}", current.bar_threshold, current.afap_threshold);
    println!("  AFAP (As Far As Practicable): {:.2} - {:.2}", current.afap_threshold, current.int_threshold);
    println!("  Intolerable: > {:.2}", current.int_threshold);
    
    let bar: f64 = Text::new("BAR threshold (0.0-1.0):")
        .with_default(&current.bar_threshold.to_string())
        .prompt()?
        .parse()
        .unwrap_or(0.25);
    
    let afap: f64 = Text::new("AFAP threshold (0.0-1.0):")
        .with_default(&current.afap_threshold.to_string())
        .prompt()?
        .parse()
        .unwrap_or(0.50);
    
    let int: f64 = Text::new("Intolerable threshold (0.0-1.0):")
        .with_default(&current.int_threshold.to_string())
        .prompt()?
        .parse()
        .unwrap_or(0.75);
    
    match tessera_core::RiskToleranceThresholds::new(bar, afap, int) {
        Ok(new_thresholds) => {
            project_ctx.metadata.quality_settings.risk_tolerance_thresholds = new_thresholds;
            println!("{} Risk tolerance thresholds updated", "✓".green());
            
            // Save the updated settings
            let project_file = project_ctx.root_path.join("project.ron");
            project_ctx.metadata.save_to_file(project_file)?;
            println!("{} Settings saved to project file", "✓".green());
        },
        Err(e) => {
            println!("{} Invalid thresholds: {}", "✗".red(), e);
        }
    }
    
    Ok(())
}

async fn view_current_settings(project_ctx: &ProjectContext) -> Result<()> {
    println!("\n{}", "Current Quality Settings".bold().blue());
    
    let prob_range = &project_ctx.metadata.quality_settings.risk_probability_range;
    let impact_range = &project_ctx.metadata.quality_settings.risk_impact_range;
    let thresholds = &project_ctx.metadata.quality_settings.risk_tolerance_thresholds;
    
    println!("\n{}", "Risk Scoring Ranges:".bold());
    println!("  Probability: [{}, {}, {}] -> values: {:?}", 
        prob_range.range[0], prob_range.range[1], prob_range.range[2], prob_range.values());
    println!("  Impact: [{}, {}, {}] -> values: {:?}", 
        impact_range.range[0], impact_range.range[1], impact_range.range[2], impact_range.values());
    
    println!("\n{}", "Risk Tolerance Thresholds:".bold());
    println!("  BAR (Broadly Acceptable): < {:.2}", thresholds.bar_threshold);
    println!("  Tolerable (with reduction): {:.2} - {:.2}", thresholds.bar_threshold, thresholds.afap_threshold);
    println!("  AFAP (As Far As Practicable): {:.2} - {:.2}", thresholds.afap_threshold, thresholds.int_threshold);
    println!("  Intolerable: > {:.2}", thresholds.int_threshold);
    
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