use crate::{RiskCommands, utils::truncate_string};
use crate::impact_service::get_impact_service;
use colored::Colorize;
use comfy_table::Table;
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text};
use tessera_impact::{ModuleType, ChangeType};
use tessera_risk::*;

pub async fn execute_risk_command(command: RiskCommands, project_ctx: ProjectContext) -> Result<()> {
    match command {
        RiskCommands::AddRisk => add_risk_interactive(project_ctx).await,
        RiskCommands::ListRisks => list_risks(project_ctx).await,
        RiskCommands::EditRisk => edit_risk_interactive(project_ctx).await,
        RiskCommands::AssessRisks => assess_risks(project_ctx).await,
        RiskCommands::AddControl => add_control_interactive(project_ctx).await,
        RiskCommands::ListControls => list_controls(project_ctx).await,
        RiskCommands::EditControl => edit_control_interactive(project_ctx).await,
        RiskCommands::Dashboard => show_dashboard(project_ctx).await,
        RiskCommands::RiskScoring => show_risk_scoring(project_ctx).await,
    }
}

async fn add_risk_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new risk".bold().blue());
    
    let name = Text::new("Risk name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let risk_categories = vec![
        "Design",
        "Process",
        "Use",
        "Software",
    ];
    
    let category_str = Select::new("Risk category:", risk_categories).prompt()?;
    let category = match category_str {
        "Design" => RiskCategory::Design,
        "Process" => RiskCategory::Process,
        "Use" => RiskCategory::Use,
        "Software" => RiskCategory::Software,
        _ => RiskCategory::Design, // Default fallback
    };
    
    let probability_str = Text::new("Probability (1-5):")
        .with_default("3")
        .prompt()?;
    let probability: i32 = probability_str.parse().unwrap_or(3).clamp(1, 5);
    
    let impact_str = Text::new("Impact (1-5):")
        .with_default("3")
        .prompt()?;
    let impact: i32 = impact_str.parse().unwrap_or(3).clamp(1, 5);
    
    let mut risk = Risk::new(name, description, category);
    risk.update_scores(probability, impact, None);
    
    let risk_dir = project_ctx.module_path("risk");
    if !risk_dir.exists() {
        std::fs::create_dir_all(&risk_dir)?;
    }
    
    let mut repo = RiskRepository::load_from_directory(&risk_dir)?;
    repo.add_risk(risk.clone())?;
    repo.save_to_directory(&risk_dir)?;
    
    // Trigger automatic impact analysis
    if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
        let _ = service.on_entity_changed(
            &risk,
            ModuleType::Risk,
            "Risk".to_string(),
            ChangeType::Created,
            &project_ctx,
        ).await;
    }
    
    println!("{} Risk '{}' added successfully!", "✓".green(), risk.name);
    println!("ID: {}", risk.id);
    
    Ok(())
}

async fn list_risks(project_ctx: ProjectContext) -> Result<()> {
    let risk_dir = project_ctx.module_path("risk");
    let repo = RiskRepository::load_from_directory(&risk_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found".yellow());
        return Ok(());
    }
    
    println!("{}", "Risks".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Category", "Status", "Level"]);
    
    for risk in risks {
        let category = match &risk.category {
            RiskCategory::Design => "Design",
            RiskCategory::Process => "Process",
            RiskCategory::Use => "Use",
            RiskCategory::Software => "Software",
        };
        
        let status = match risk.status {
            RiskStatus::Identified => "Identified".cyan(),
            RiskStatus::Analyzed => "Analyzed".yellow(),
            RiskStatus::Mitigated => "Mitigated".green(),
            RiskStatus::Accepted => "Accepted".blue(),
            RiskStatus::Transferred => "Transferred".blue(),
            RiskStatus::Closed => "Closed".green(),
        };
        
        let level = match risk.risk_level {
            RiskLevel::Low => "Low".green(),
            RiskLevel::Medium => "Medium".blue(),
            RiskLevel::High => "High".yellow(),
            RiskLevel::Critical => "Critical".red(),
        };
        
        table.add_row(vec![
            risk.id.to_string(),
            truncate_string(&risk.name, 25),
            category.to_string(),
            status.to_string(),
            level.to_string(),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn edit_risk_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Editing risk".bold().blue());
    
    let risk_dir = project_ctx.module_path("risk");
    let mut repo = RiskRepository::load_from_directory(&risk_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found. Add risks first.".yellow());
        return Ok(());
    }
    
    let risk_options: Vec<String> = risks.iter()
        .map(|r| format!("{} - {}", r.name, truncate_string(&r.description, 50)))
        .collect();
    
    let risk_selection = Select::new("Select risk to edit:", risk_options.clone()).prompt()?;
    let risk_index = risk_options.iter().position(|x| x == &risk_selection).unwrap();
    let selected_risk = &risks[risk_index];
    let mut risk: tessera_risk::Risk = (*selected_risk).clone();
    
    let edit_options = vec![
        "Update status",
        "Update scores (probability/impact)",
        "Set mitigation strategy",
        "Set owner",
        "Set due date",
        "Add tag", 
        "Update FMEA fields",
        "Cancel",
    ];
    
    let edit_choice = Select::new("What would you like to edit?", edit_options).prompt()?;
    
    match edit_choice {
        "Update status" => {
            let status_options = vec!["Identified", "Analyzed", "Mitigated", "Accepted", "Transferred", "Closed"];
            let status_choice = Select::new("New status:", status_options).prompt()?;
            let new_status = match status_choice {
                "Identified" => RiskStatus::Identified,
                "Analyzed" => RiskStatus::Analyzed,
                "Mitigated" => RiskStatus::Mitigated,
                "Accepted" => RiskStatus::Accepted,
                "Transferred" => RiskStatus::Transferred,
                "Closed" => RiskStatus::Closed,
                _ => RiskStatus::Identified,
            };
            risk.update_status(new_status);
        },
        "Update scores (probability/impact)" => {
            let prob_str = Text::new("Probability (1-5):")
                .with_default(&risk.probability.to_string())
                .prompt()?;
            let impact_str = Text::new("Impact (1-5):")
                .with_default(&risk.impact.to_string())
                .prompt()?;
            
            let probability: i32 = prob_str.parse().unwrap_or(risk.probability).clamp(1, 5);
            let impact: i32 = impact_str.parse().unwrap_or(risk.impact).clamp(1, 5);
            
            let detectability_str = Text::new("Detectability (1-10, optional):")
                .with_help_message("Leave empty to keep current value")
                .prompt()?;
            
            let detectability = if detectability_str.is_empty() {
                risk.detectability
            } else {
                detectability_str.parse().ok().map(|d: i32| d.clamp(1, 10))
            };
            
            risk.update_scores(probability, impact, detectability);
        },
        "Set mitigation strategy" => {
            let strategy = Text::new("Mitigation strategy:")
                .with_help_message("Describe how this risk will be mitigated")
                .prompt()?;
            risk.set_mitigation_strategy(strategy);
        },
        "Set owner" => {
            let owner = Text::new("Risk owner:")
                .with_help_message("Who is responsible for managing this risk?")
                .prompt()?;
            risk.set_owner(owner);
        },
        "Add tag" => {
            let tag = Text::new("Tag to add:").prompt()?;
            risk.add_tag(tag);
        },
        "Update FMEA fields" => {
            let failure_mode = Text::new("Failure mode:")
                .with_help_message("What could go wrong?")
                .prompt()?;
            let cause = Text::new("Cause of failure:")
                .with_help_message("What could cause this failure?")
                .prompt()?;
            let effect = Text::new("Effect of failure:")
                .with_help_message("What would be the impact?")
                .prompt()?;
            
            risk.set_fmea_fields(Some(failure_mode), Some(cause), Some(effect));
        },
        _ => {
            println!("Cancelled.");
            return Ok(());
        }
    }
    
    // Drop the immutable borrow before the mutable borrow
    let risk_name = risk.name.clone();
    drop(risks);
    
    repo.update_risk(risk.clone())?;
    repo.save_to_directory(&risk_dir)?;
    
    // Trigger automatic impact analysis
    if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
        let _ = service.on_entity_changed(
            &risk,
            ModuleType::Risk,
            "Risk".to_string(),
            ChangeType::Updated,
            &project_ctx,
        ).await;
    }
    
    println!("{} Risk '{}' updated successfully!", "✓".green(), risk_name);
    Ok(())
}

async fn assess_risks(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_risk::RiskCommands::new(project_ctx)?;
    commands.assess_risks()
}

async fn add_control_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design control".bold().blue());
    
    let name = Text::new("Control name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let control_types = vec![
        "Preventive",
        "Detective",
        "Corrective",
        "Compensating",
        "Directive",
    ];
    
    let control_type_str = Select::new("Control type:", control_types).prompt()?;
    let control_type = match control_type_str {
        "Preventive" => ControlType::Preventive,
        "Detective" => ControlType::Detective,
        "Corrective" => ControlType::Corrective,
        "Compensating" => ControlType::Compensating,
        "Directive" => ControlType::Directive,
        _ => ControlType::Preventive,
    };
    
    // First, let's select a risk to link this control to
    let risk_dir = project_ctx.module_path("risk");
    if !risk_dir.exists() {
        std::fs::create_dir_all(&risk_dir)?;
    }
    
    let repo = RiskRepository::load_from_directory(&risk_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found. Create a risk first.".yellow());
        return Ok(());
    }
    
    let risk_options: Vec<String> = risks.iter()
        .map(|r| format!("{} - {}", r.name, truncate_string(&r.description, 50)))
        .collect();
    
    let risk_selection = Select::new("Select risk to link control to:", risk_options.clone()).prompt()?;
    let risk_index = risk_options.iter().position(|x| x == &risk_selection).unwrap();
    let selected_risk = &risks[risk_index];
    let risk_id = selected_risk.id;
    
    let control = DesignControl::new(name, description, control_type, risk_id);
    
    let mut repo = RiskRepository::load_from_directory(&risk_dir)?;
    repo.add_design_control(control.clone())?;
    repo.save_to_directory(&risk_dir)?;
    
    // Trigger automatic impact analysis
    if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
        let _ = service.on_entity_changed(
            &control,
            ModuleType::Risk,
            "DesignControl".to_string(),
            ChangeType::Created,
            &project_ctx,
        ).await;
    }
    
    println!("{} Design control '{}' added successfully!", "✓".green(), control.name);
    println!("ID: {}", control.id);
    
    Ok(())
}

async fn list_controls(project_ctx: ProjectContext) -> Result<()> {
    let risk_dir = project_ctx.module_path("risk");
    let repo = RiskRepository::load_from_directory(&risk_dir)?;
    let controls = repo.get_design_controls();
    
    if controls.is_empty() {
        println!("{}", "No design controls found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Controls".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Status", "Effectiveness"]);
    
    for control in controls {
        let control_type = match &control.control_type {
            ControlType::Preventive => "Preventive",
            ControlType::Detective => "Detective",
            ControlType::Corrective => "Corrective",
            ControlType::Compensating => "Compensating",
            ControlType::Directive => "Directive",
        };
        
        let status = match control.status {
            ControlStatus::Planned => "Planned".cyan(),
            ControlStatus::InProgress => "In Progress".yellow(),
            ControlStatus::Implemented => "Implemented".green(),
            ControlStatus::Verified => "Verified".green(),
            ControlStatus::Ineffective => "Ineffective".red(),
        };
        
        let effectiveness = if let Some(rating) = control.effectiveness_rating {
            format!("{}/10", rating)
        } else {
            "N/A".to_string()
        };
        
        table.add_row(vec![
            control.id.to_string(),
            truncate_string(&control.name, 25),
            control_type.to_string(),
            status.to_string(),
            effectiveness,
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn edit_control_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Editing design control".bold().blue());
    
    let risk_dir = project_ctx.module_path("risk");
    let mut repo = RiskRepository::load_from_directory(&risk_dir)?;
    let controls = repo.get_design_controls();
    
    if controls.is_empty() {
        println!("{}", "No design controls found. Add controls first.".yellow());
        return Ok(());
    }
    
    let control_options: Vec<String> = controls.iter()
        .map(|c| format!("{} - {}", c.name, truncate_string(&c.description, 50)))
        .collect();
    
    let control_selection = Select::new("Select control to edit:", control_options.clone()).prompt()?;
    let control_index = control_options.iter().position(|x| x == &control_selection).unwrap();
    let selected_control = &controls[control_index];
    let mut control: tessera_risk::DesignControl = (*selected_control).clone();
    
    let edit_options = vec![
        "Update status",
        "Set implementation approach",
        "Set responsible party", 
        "Set target completion date",
        "Set effectiveness rating",
        "Set verification method",
        "Add tag",
        "Link to design output",
        "Cancel",
    ];
    
    let edit_choice = Select::new("What would you like to edit?", edit_options).prompt()?;
    
    match edit_choice {
        "Update status" => {
            let status_options = vec!["Planned", "In Progress", "Implemented", "Verified", "Ineffective"];
            let status_choice = Select::new("New status:", status_options).prompt()?;
            let new_status = match status_choice {
                "Planned" => ControlStatus::Planned,
                "In Progress" => ControlStatus::InProgress,
                "Implemented" => ControlStatus::Implemented,
                "Verified" => ControlStatus::Verified,
                "Ineffective" => ControlStatus::Ineffective,
                _ => ControlStatus::Planned,
            };
            control.update_status(new_status);
        },
        "Set implementation approach" => {
            let approach = Text::new("Implementation approach:")
                .with_help_message("Describe how this control will be implemented")
                .prompt()?;
            control.set_implementation_details(Some(approach), None, None);
        },
        "Set responsible party" => {
            let party = Text::new("Responsible party:")
                .with_help_message("Who is responsible for implementing this control?")
                .prompt()?;
            control.set_implementation_details(None, Some(party), None);
        },
        "Set effectiveness rating" => {
            let rating_str = Text::new("Effectiveness rating (1-5):")
                .with_help_message("Rate the effectiveness of this control")
                .prompt()?;
            if let Ok(rating) = rating_str.parse::<i32>() {
                if rating >= 1 && rating <= 5 {
                    control.set_effectiveness_rating(rating);
                }
            }
        },
        "Set verification method" => {
            let method = Text::new("Verification method:")
                .with_help_message("How will this control be verified?")
                .prompt()?;
            control.set_verification_details(Some(method), None);
        },
        "Add tag" => {
            let tag = Text::new("Tag to add:").prompt()?;
            control.add_tag(tag);
        },
        "Link to design output" => {
            println!("{}", "Design output linking not yet implemented - requires integration with requirements module".yellow());
        },
        _ => {
            println!("Cancelled.");
            return Ok(());
        }
    }
    
    // Drop the immutable borrow before the mutable borrow
    let control_name = control.name.clone();
    drop(controls);
    
    repo.update_design_control(control.clone())?;
    repo.save_to_directory(&risk_dir)?;
    
    // Trigger automatic impact analysis
    if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
        let _ = service.on_entity_changed(
            &control,
            ModuleType::Risk,
            "DesignControl".to_string(),
            ChangeType::Updated,
            &project_ctx,
        ).await;
    }
    
    println!("{} Design control '{}' updated successfully!", "✓".green(), control_name);
    Ok(())
}

async fn show_dashboard(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_risk::RiskCommands::new(project_ctx)?;
    commands.show_dashboard()
}

async fn show_risk_scoring(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Risk Scoring Analysis".bold().blue());
    
    let risk_dir = project_ctx.module_path("risk");
    let repo = RiskRepository::load_from_directory(&risk_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found. Add risks first.".yellow());
        return Ok(());
    }
    
    println!("\n{}", "Risk Scoring Summary".bold());
    
    let mut total_score = 0.0;
    let mut high_risk_count = 0;
    let mut medium_risk_count = 0;
    let mut low_risk_count = 0;
    
    let mut table = Table::new();
    table.set_header(vec!["Risk", "Category", "Probability", "Impact", "Score", "Level", "Controls"]);
    
    for risk in &risks {
        total_score += risk.risk_score;
        
        match risk.risk_level {
            RiskLevel::Critical | RiskLevel::High => high_risk_count += 1,
            RiskLevel::Medium => medium_risk_count += 1,
            RiskLevel::Low => low_risk_count += 1,
        }
        
        let category = match &risk.category {
            RiskCategory::Design => "Design",
            RiskCategory::Process => "Process", 
            RiskCategory::Use => "Use",
            RiskCategory::Software => "Software",
        };
        
        let level = match risk.risk_level {
            RiskLevel::Low => "Low".green(),
            RiskLevel::Medium => "Medium".blue(),
            RiskLevel::High => "High".yellow(),
            RiskLevel::Critical => "Critical".red(),
        };
        
        let controls_count = repo.get_design_controls_for_risk(&risk.id).len();
        
        table.add_row(vec![
            truncate_string(&risk.name, 20),
            category.to_string(),
            risk.probability.to_string(),
            risk.impact.to_string(),
            format!("{:.2}", risk.risk_score),
            level.to_string(),
            controls_count.to_string(),
        ]);
    }
    
    println!("{}", table);
    
    let average_score = if risks.is_empty() { 0.0 } else { total_score / risks.len() as f64 };
    
    println!("\n{}", "Analysis Summary:".bold());
    println!("Total Risks: {}", risks.len());
    println!("Average Risk Score: {:.2}", average_score);
    println!("High/Critical Risk Items: {}", high_risk_count);
    println!("Medium Risk Items: {}", medium_risk_count);
    println!("Low Risk Items: {}", low_risk_count);
    
    let total_controls = repo.get_design_controls().len();
    println!("Total Design Controls: {}", total_controls);
    
    if high_risk_count > 0 {
        println!("\n{}", "Recommendations:".bold().yellow());
        println!("• {} high/critical risks require immediate attention", high_risk_count);
        println!("• Consider implementing additional design controls for high-risk items");
        println!("• Review control effectiveness for existing mitigations");
    }
    
    Ok(())
}