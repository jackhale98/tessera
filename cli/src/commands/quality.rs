use crate::{QualityCommands, utils::*};
use colored::Colorize;
use comfy_table::Table;
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text, Confirm};
use tessera_quality::*;

pub async fn execute_quality_command(command: QualityCommands, project_ctx: ProjectContext) -> Result<()> {
    match command {
        QualityCommands::AddRequirement => add_requirement_interactive(project_ctx).await,
        QualityCommands::ListRequirements => list_requirements(project_ctx).await,
        QualityCommands::EditRequirement => edit_requirement_interactive(project_ctx).await,
        QualityCommands::AddInput => add_input_interactive(project_ctx).await,
        QualityCommands::ListInputs => list_inputs(project_ctx).await,
        QualityCommands::EditInput => edit_input_interactive(project_ctx).await,
        QualityCommands::AddOutput => add_output_interactive(project_ctx).await,
        QualityCommands::ListOutputs => list_outputs(project_ctx).await,
        QualityCommands::EditOutput => edit_output_interactive(project_ctx).await,
        QualityCommands::AddVerification => add_verification_interactive(project_ctx).await,
        QualityCommands::ListVerifications => list_verifications(project_ctx).await,
        QualityCommands::EditVerification => edit_verification_interactive(project_ctx).await,
        QualityCommands::AddControl => add_control_interactive(project_ctx).await,
        QualityCommands::ListControls => list_controls(project_ctx).await,
        QualityCommands::EditControl => edit_control_interactive(project_ctx).await,
        QualityCommands::AddRisk => add_risk_interactive(project_ctx).await,
        QualityCommands::ListRisks => list_risks(project_ctx).await,
        QualityCommands::EditRisk => edit_risk_interactive(project_ctx).await,
        QualityCommands::AssessRisks => assess_risks(project_ctx).await,
        QualityCommands::TraceabilityMatrix => run_traceability_matrix(project_ctx).await,
        QualityCommands::RiskScoring => run_risk_scoring(project_ctx).await,
        QualityCommands::Dashboard => show_quality_dashboard(project_ctx).await,
    }
}

async fn add_requirement_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new requirement".bold().blue());
    
    let name = Text::new("Requirement name:")
        .with_help_message("Enter a concise name for the requirement")
        .prompt()?;
    
    let description = Text::new("Description:")
        .with_help_message("Detailed description of the requirement")
        .prompt()?;
    
    let categories = vec![
        "Functional",
        "Performance", 
        "Safety",
        "Regulatory",
        "Usability",
        "Reliability",
        "Maintainability",
        "Environmental",
        "Other",
    ];
    
    let category = Select::new("Category:", categories).prompt()?.to_string();
    
    let priorities = vec!["Critical", "High", "Medium", "Low"];
    let priority_str = Select::new("Priority:", priorities).prompt()?;
    let priority = match priority_str {
        "Critical" => Priority::Critical,
        "High" => Priority::High,
        "Medium" => Priority::Medium,
        "Low" => Priority::Low,
        _ => Priority::Medium,
    };
    
    let source = Text::new("Source:")
        .with_help_message("Source of requirement (customer, regulation, standard, etc.)")
        .prompt()?;

    let mut requirement = Requirement::new(name, description, source, category);
    requirement.priority = priority;
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_requirement(requirement.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Requirement '{}' added successfully!", "✓".green(), requirement.name);
    println!("ID: {}", requirement.id);
    
    Ok(())
}

async fn list_requirements(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = match QualityRepository::load_from_directory(&quality_dir) {
        Ok(repo) => repo,
        Err(_) => {
            // Try migration if loading fails
            tessera_quality::migrate_quality_data(&quality_dir)?;
            QualityRepository::load_from_directory(&quality_dir)?
        }
    };
    let requirements = repo.get_requirements();
    
    if requirements.is_empty() {
        println!("{}", "No requirements found".yellow());
        return Ok(());
    }
    
    println!("{}", "Requirements".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Category", "Priority", "Status"]);
    
    for req in requirements {
        let category = &req.category;
        
        let priority = match req.priority {
            Priority::Critical => "Critical".red(),
            Priority::High => "High".yellow(),
            Priority::Medium => "Medium".blue(),
            Priority::Low => "Low".green(),
        };
        
        let status = match req.status {
            RequirementStatus::Draft => "Draft".cyan(),
            RequirementStatus::Approved => "Approved".green(),
            RequirementStatus::Verified => "Verified".green(),
            RequirementStatus::Closed => "Closed".red(),
        };
        
        table.add_row(vec![
            req.id.to_string(),
            truncate_string(&req.name, 30),
            category.to_string(),
            priority.to_string(),
            status.to_string(),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn add_input_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design input".bold().blue());
    
    // First, load requirements to ensure we have some to link to
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let requirements = repo.get_requirements();
    
    if requirements.is_empty() {
        println!("{}", "No requirements found. You must create requirements before adding design inputs.".yellow());
        println!("Design inputs implement specific requirements - this ensures traceability.");
        return Ok(());
    }
    
    // FIRST: Select the requirement this design input implements
    println!("{}", "Select the requirement this design input implements:".bold().blue());
    let req_options: Vec<String> = requirements.iter()
        .map(|r| format!("{} - {} ({})", r.name, truncate_string(&r.description, 40), r.source))
        .collect();
    
    let req_selection = Select::new("Requirement:", req_options.clone()).prompt()?;
    let req_index = req_options.iter().position(|x| x == &req_selection).unwrap();
    let selected_requirement = &requirements[req_index];
    let requirement_id = selected_requirement.id;
    
    println!("\n{}", format!("Creating design input for requirement: {}", selected_requirement.name).bold().green());
    
    // THEN: Get details about the input
    let name = Text::new("Input name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let input_types = vec![
        "Specification",
        "Standard", 
        "Regulation",
        "Customer Requirement",
        "Market Research",
        "Technical Report",
        "Other",
    ];
    
    let input_type = Select::new("Input type:", input_types).prompt()?.to_string();
    
    // Get acceptance criteria for this design input
    let add_criteria = Confirm::new("Add acceptance criteria for this design input?")
        .with_default(true)
        .prompt()?;
    
    let mut acceptance_criteria = Vec::new();
    if add_criteria {
        loop {
            let criterion = Text::new("Acceptance criterion:")
                .with_help_message("Enter criterion (empty to finish)")
                .prompt()?;
            
            if criterion.is_empty() {
                break;
            }
            
            acceptance_criteria.push(criterion);
        }
    }
    
    let mut input = DesignInput::new(name, description, input_type, requirement_id);
    input.acceptance_criteria = acceptance_criteria;
    
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_input(input.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Design input '{}' added successfully!", "✓".green(), input.name);
    println!("ID: {}", input.id);
    println!("Implements requirement: {}", selected_requirement.name);
    
    Ok(())
}


async fn add_output_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design output".bold().blue());
    
    // First, load design inputs to ensure we have some to link to
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let inputs = repo.get_inputs();
    
    if inputs.is_empty() {
        println!("{}", "No design inputs found. You must create design inputs before adding design outputs.".yellow());
        println!("Design outputs satisfy specific design inputs - this ensures traceability.");
        return Ok(());
    }
    
    // FIRST: Select the design input this output satisfies
    println!("{}", "Select the design input this output satisfies:".bold().blue());
    let input_options: Vec<String> = inputs.iter()
        .map(|i| format!("{} - {} ({})", i.name, truncate_string(&i.description, 40), i.input_type))
        .collect();
    
    let input_selection = Select::new("Design Input:", input_options.clone()).prompt()?;
    let input_index = input_options.iter().position(|x| x == &input_selection).unwrap();
    let selected_input = &inputs[input_index];
    let input_id = selected_input.id;
    
    println!("\n{}", format!("Creating design output for input: {}", selected_input.name).bold().green());
    
    // THEN: Get details about the output
    let name = Text::new("Output name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let output_types = vec![
        "Drawing",
        "Calculation", 
        "Specification",
        "Report",
        "Model",
        "Prototype",
        "Test Plan",
        "Other",
    ];
    
    let output_type = Select::new("Output type:", output_types).prompt()?.to_string();
    
    // Optional file path for the output
    let file_path = Text::new("File path (optional):")
        .with_help_message("Path to the actual design output file")
        .prompt()
        .ok()
        .filter(|s| !s.is_empty());
    
    let mut output = DesignOutput::new(name, description, output_type, input_id);
    output.file_path = file_path;
    
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_output(output.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Design output '{}' added successfully!", "✓".green(), output.name);
    println!("ID: {}", output.id);
    println!("Satisfies design input: {}", selected_input.name);
    
    Ok(())
}

async fn add_verification_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new verification".bold().blue());
    
    // First, load design outputs to ensure we have some to link to
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let outputs = repo.get_outputs();
    
    if outputs.is_empty() {
        println!("{}", "No design outputs found. You must create design outputs before adding verifications.".yellow());
        println!("Verifications validate specific design outputs - this ensures traceability.");
        return Ok(());
    }
    
    // FIRST: Select the design output this verification validates
    println!("{}", "Select the design output this verification validates:".bold().blue());
    let output_options: Vec<String> = outputs.iter()
        .map(|o| format!("{} - {} ({})", o.name, truncate_string(&o.description, 40), o.output_type))
        .collect();
    
    let output_selection = Select::new("Design Output:", output_options.clone()).prompt()?;
    let output_index = output_options.iter().position(|x| x == &output_selection).unwrap();
    let selected_output = &outputs[output_index];
    let output_id = selected_output.id;
    
    println!("\n{}", format!("Creating verification for output: {}", selected_output.name).bold().green());
    
    // THEN: Get details about the verification
    let name = Text::new("Verification name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let verification_types = vec![
        "Review",
        "Inspection", 
        "Test",
        "Verification",
        "Validation",
        "Approval",
        "Other",
    ];
    
    let verification_type = Select::new("Verification type:", verification_types).prompt()?.to_string();
    
    // Additional verification details
    let procedure = Text::new("Verification procedure:")
        .with_help_message("How will this verification be performed?")
        .prompt()?;
    
    let responsible_party = Text::new("Responsible party:")
        .with_help_message("Who is responsible for performing this verification?")
        .prompt()?;
    
    let mut verification = Verification::new(name, description, verification_type, output_id);
    verification.procedure = procedure;
    verification.responsible_party = responsible_party;
    
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_verification(verification.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Verification '{}' added successfully!", "✓".green(), verification.name);
    println!("ID: {}", verification.id);
    println!("Validates design output: {}", selected_output.name);
    
    Ok(())
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
    ];
    
    let category = Select::new("Risk category:", risk_categories).prompt()?.to_string();
    
    // Get risk scoring configuration from project
    let prob_config = &project_ctx.metadata.quality_settings.risk_probability_range;
    let impact_config = &project_ctx.metadata.quality_settings.risk_impact_range;
    let risk_thresholds = &project_ctx.metadata.quality_settings.risk_tolerance_thresholds;
    
    let prob_values = prob_config.values();
    let impact_values = impact_config.values();
    
    let probability_str = Text::new(&format!("Probability [values available: {:?}]:", prob_values))
        .with_default(&prob_values[0].to_string())
        .prompt()?;
    let probability: i32 = probability_str.parse().unwrap_or(prob_values[0]);
    
    let impact_str = Text::new(&format!("Impact [values available: {:?}]:", impact_values))
        .with_default(&impact_values[0].to_string())
        .prompt()?;
    let impact: i32 = impact_str.parse().unwrap_or(impact_values[0]);
    
    let mut risk = Risk::new(name, description, category);
    risk.probability = probability;
    risk.impact = impact;
    let risk_category = risk.update_risk_score_with_category(prob_config, impact_config, risk_thresholds);
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_risk(risk.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Risk '{}' added successfully!", "✓".green(), risk.name);
    println!("ID: {}", risk.id);
    println!("Risk Score: {:.3} ({})", risk.risk_score, risk_category);
    
    Ok(())
}

async fn assess_risks(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Assessing project risks".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found. Add risks first.".yellow());
        return Ok(());
    }
    
    let config = RiskAnalysisConfig::default();
    let analyzer = RiskAnalyzer::new(config);
    
    println!("Running Monte Carlo risk analysis...");
    let analysis = analyzer.analyze_project_risks(risks)?;
    
    println!("\n{}", "Risk Analysis Results".bold().green());
    println!("Overall Risk Score: {:.2}", analysis.overall_risk_score);
    println!("High Risk Items: {}", analysis.high_risk_items.len());
    
    println!("\n{}", "Individual Risk Analysis:".bold());
    let mut table = Table::new();
    table.set_header(vec!["Risk", "Monte Carlo Score", "95th Percentile", "Recommendation"]);
    
    for result in &analysis.individual_risks {
        let recommendation = match result.recommendation {
            RiskRecommendation::Accept => "Accept".green(),
            RiskRecommendation::Monitor => "Monitor".yellow(),
            RiskRecommendation::Mitigate => "Mitigate".red(),
            RiskRecommendation::Avoid => "Avoid".red(),
        };
        
        table.add_row(vec![
            truncate_string(&result.risk_name, 25),
            format!("{:.3}", result.monte_carlo_score),
            format!("{:.3}", result.percentile_95),
            recommendation.to_string(),
        ]);
    }
    
    println!("{}", table);
    
    println!("\n{}", "Recommendations:".bold());
    for recommendation in &analysis.recommendations {
        println!("• {}", recommendation);
    }
    
    Ok(())
}

async fn show_quality_dashboard(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Quality Dashboard".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let requirements = repo.get_requirements();
    let inputs = repo.get_inputs();
    let outputs = repo.get_outputs();
    let verifications = repo.get_verifications();
    let risks = repo.get_risks();
    
    println!("\n{}", "Summary".bold());
    println!("Requirements: {}", requirements.len());
    println!("Design Inputs: {}", inputs.len());
    println!("Design Outputs: {}", outputs.len());
    println!("Verifications: {}", verifications.len());
    println!("Risks: {}", risks.len());
    
    if !requirements.is_empty() {
        println!("\n{}", "Requirements by Status:".bold());
        let mut status_counts = std::collections::HashMap::new();
        for req in requirements {
            let status = match req.status {
                RequirementStatus::Draft => "Draft",
                RequirementStatus::Approved => "Approved",
                RequirementStatus::Verified => "Verified",
                RequirementStatus::Closed => "Closed",
            };
            *status_counts.entry(status).or_insert(0) += 1;
        }
        
        for (status, count) in status_counts {
            println!("  {}: {}", status, count);
        }
    }
    
    if !risks.is_empty() {
        println!("\n{}", "Risk Summary:".bold());
        let high_risk_count = risks.iter().filter(|r| r.risk_score >= 0.7).count();
        let medium_risk_count = risks.iter().filter(|r| r.risk_score >= 0.3 && r.risk_score < 0.7).count();
        let low_risk_count = risks.iter().filter(|r| r.risk_score < 0.3).count();
        
        println!("  High Risk (≥0.7): {}", high_risk_count);
        println!("  Medium Risk (0.3-0.7): {}", medium_risk_count);
        println!("  Low Risk (<0.3): {}", low_risk_count);
    }
    
    Ok(())
}

async fn list_inputs(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = match QualityRepository::load_from_directory(&quality_dir) {
        Ok(repo) => repo,
        Err(_) => {
            // Try migration if loading fails
            tessera_quality::migrate_quality_data(&quality_dir)?;
            QualityRepository::load_from_directory(&quality_dir)?
        }
    };
    let inputs = repo.get_inputs();
    let requirements = repo.get_requirements();
    
    if inputs.is_empty() {
        println!("{}", "No design inputs found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Inputs".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Requirement"]);
    
    for input in inputs {
        let input_type = &input.input_type;
        
        // Find the requirement name
        let requirement_name = requirements.iter()
            .find(|r| r.id == input.requirement_id)
            .map(|r| truncate_string(&r.name, 30))
            .unwrap_or_else(|| "[Not Found]".to_string());
        
        table.add_row(vec![
            input.id.to_string(),
            truncate_string(&input.name, 25),
            input_type.to_string(),
            requirement_name,
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn list_outputs(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = match QualityRepository::load_from_directory(&quality_dir) {
        Ok(repo) => repo,
        Err(_) => {
            // Try migration if loading fails
            tessera_quality::migrate_quality_data(&quality_dir)?;
            QualityRepository::load_from_directory(&quality_dir)?
        }
    };
    let outputs = repo.get_outputs();
    let inputs = repo.get_inputs();
    
    if outputs.is_empty() {
        println!("{}", "No design outputs found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Outputs".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Design Input"]);
    
    for output in outputs {
        let output_type = &output.output_type;
        
        // Find the input name
        let input_name = inputs.iter()
            .find(|i| i.id == output.input_id)
            .map(|i| truncate_string(&i.name, 30))
            .unwrap_or_else(|| "[Not Found]".to_string());
        
        table.add_row(vec![
            output.id.to_string(),
            truncate_string(&output.name, 25),
            output_type.to_string(),
            input_name,
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn list_verifications(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = match QualityRepository::load_from_directory(&quality_dir) {
        Ok(repo) => repo,
        Err(_) => {
            // Try migration if loading fails
            tessera_quality::migrate_quality_data(&quality_dir)?;
            QualityRepository::load_from_directory(&quality_dir)?
        }
    };
    let verifications = repo.get_verifications();
    let outputs = repo.get_outputs();
    
    if verifications.is_empty() {
        println!("{}", "No verifications found".yellow());
        return Ok(());
    }
    
    println!("{}", "Verifications".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Status", "Design Output"]);
    
    for verification in verifications {
        // Find the output name
        let output_name = outputs.iter()
            .find(|o| o.id == verification.output_id)
            .map(|o| truncate_string(&o.name, 25))
            .unwrap_or_else(|| "[Not Found]".to_string());
        
        table.add_row(vec![
            verification.id.to_string(),
            truncate_string(&verification.name, 25),
            truncate_string(&verification.verification_type, 15),
            format!("{:?}", verification.status),
            output_name,
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn list_risks(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = match QualityRepository::load_from_directory(&quality_dir) {
        Ok(repo) => repo,
        Err(_) => {
            // Try migration if loading fails
            tessera_quality::migrate_quality_data(&quality_dir)?;
            QualityRepository::load_from_directory(&quality_dir)?
        }
    };
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found".yellow());
        return Ok(());
    }
    
    println!("{}", "Risks".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Category", "Probability", "Impact", "Risk Score", "Risk Level"]);
    
    let risk_thresholds = &project_ctx.metadata.quality_settings.risk_tolerance_thresholds;
    
    for risk in risks {
        let category = &risk.category;
        let risk_level = risk_thresholds.categorize_risk(risk.risk_score);
        
        let risk_score_color = match risk_level {
            tessera_core::RiskCategory::BroadlyAcceptable => format!("{:.3}", risk.risk_score).green(),
            tessera_core::RiskCategory::TolerableWithReduction => format!("{:.3}", risk.risk_score).blue(),
            tessera_core::RiskCategory::AsFarAsPracticable => format!("{:.3}", risk.risk_score).yellow(),
            tessera_core::RiskCategory::Intolerable => format!("{:.3}", risk.risk_score).red(),
        };
        
        let risk_level_display = match risk_level {
            tessera_core::RiskCategory::BroadlyAcceptable => "BAR".green(),
            tessera_core::RiskCategory::TolerableWithReduction => "TOLERABLE".blue(),
            tessera_core::RiskCategory::AsFarAsPracticable => "AFAP".yellow(),
            tessera_core::RiskCategory::Intolerable => "INT".red(),
        };
        
        table.add_row(vec![
            risk.id.to_string(),
            truncate_string(&risk.name, 25),
            category.to_string(),
            risk.probability.to_string(),
            risk.impact.to_string(),
            risk_score_color.to_string(),
            risk_level_display.to_string(),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}


async fn run_traceability_matrix(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let mut menu = TraceabilityMenuInterface::new();
    menu.show_traceability_menu(&mut repo)
}

async fn run_risk_scoring(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let mut menu = ScoringMenu::new();
    menu.run(&repo)?;
    
    Ok(())
}

async fn add_control_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design control".bold().blue());
    
    // First, load risks to ensure we have some to link to
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found. You must create risks before adding design controls.".yellow());
        println!("Design controls are used to mitigate or control specific risks.");
        return Ok(());
    }
    
    let name = Text::new("Control name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let control_types = vec![
        "Administrative",
        "Engineering",
        "Physical",
        "Procedural",
        "Technical",
        "Training",
        "Other",
    ];
    
    let control_type = Select::new("Control type:", control_types).prompt()?.to_string();
    
    // Automatically prompt to select risk - this ensures controls are linked to risks
    println!("\n{}", "Select the risk this control mitigates:".bold().blue());
    let risk_options: Vec<String> = risks.iter()
        .map(|r| format!("{} - {} (Score: {:.3})", r.name, truncate_string(&r.description, 40), r.risk_score))
        .collect();
    
    let risk_selection = Select::new("Risk:", risk_options.clone()).prompt()?;
    let risk_index = risk_options.iter().position(|x| x == &risk_selection).unwrap();
    let selected_risk = &risks[risk_index];
    let risk_id = selected_risk.id;
    
    // Additional control details
    let implementation = Text::new("Implementation details:")
        .with_help_message("How will this control be implemented?")
        .prompt()?;
    
    let mut control = DesignControl::new(name, description, control_type, risk_id);
    control.implementation = implementation;
    
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_control(control.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Design control '{}' added successfully!", "✓".green(), control.name);
    println!("ID: {}", control.id);
    println!("Mitigates risk: {}", selected_risk.name);
    
    Ok(())
}

async fn list_controls(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = match QualityRepository::load_from_directory(&quality_dir) {
        Ok(repo) => repo,
        Err(_) => {
            // Try migration if loading fails
            tessera_quality::migrate_quality_data(&quality_dir)?;
            QualityRepository::load_from_directory(&quality_dir)?
        }
    };
    let controls = repo.get_controls();
    let risks = repo.get_risks();
    
    if controls.is_empty() {
        println!("{}", "No design controls found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Controls".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Risk", "Implementation"]);
    
    for control in controls {
        // Find the risk name
        let risk_name = risks.iter()
            .find(|r| r.id == control.risk_id)
            .map(|r| truncate_string(&r.name, 25))
            .unwrap_or_else(|| "[Not Found]".to_string());
        
        table.add_row(vec![
            control.id.to_string(),
            truncate_string(&control.name, 25),
            control.control_type.to_string(),
            risk_name,
            truncate_string(&control.implementation, 30),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}
async fn edit_requirement_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Editing requirement".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let requirements = repo.get_requirements();
    
    if requirements.is_empty() {
        println!("{}", "No requirements found to edit".yellow());
        return Ok(());
    }
    
    // Select requirement to edit
    let req_options: Vec<String> = requirements.iter()
        .map(|r| format!("{} - {} ({})", r.name, truncate_string(&r.description, 40), r.source))
        .collect();
    
    let req_selection = Select::new("Select requirement to edit:", req_options.clone()).prompt()?;
    let req_index = req_options.iter().position(|x| x == &req_selection).unwrap();
    let mut requirement = requirements[req_index].clone();
    
    // Show current values and let user choose what to edit
    loop {
        println!("\n{}", format!("Editing requirement: {}", requirement.name).bold().green());
        println!("Current values:");
        println!("  Name: {}", requirement.name);
        println!("  Description: {}", requirement.description);
        println!("  Source: {}", requirement.source);
        println!("  Category: {}", requirement.category);
        println!("  Priority: {:?}", requirement.priority);
        
        let field_options = vec![
            "📝 Name",
            "📄 Description", 
            "📍 Source",
            "📂 Category",
            "⚡ Priority",
            "💾 Save Changes",
            "❌ Cancel",
        ];
        
        let field_selection = Select::new("Select field to edit:", field_options)
            .with_help_message("Choose which field to modify")
            .prompt()?;
        
        match field_selection {
            "📝 Name" => {
                let new_name = Text::new("Name:")
                    .with_default(&requirement.name)
                    .prompt()?;
                requirement.name = new_name;
            },
            "📄 Description" => {
                let new_description = Text::new("Description:")
                    .with_default(&requirement.description)
                    .prompt()?;
                requirement.description = new_description;
            },
            "📍 Source" => {
                let new_source = Text::new("Source:")
                    .with_default(&requirement.source)
                    .prompt()?;
                requirement.source = new_source;
            },
            "📂 Category" => {
                let categories = vec![
                    "Functional", "Performance", "Safety", "Regulatory",
                    "Usability", "Reliability", "Maintainability", "Environmental", "Other",
                ];
                let category_index = categories.iter().position(|&c| c == requirement.category).unwrap_or(0);
                let new_category = Select::new("Category:", categories)
                    .with_starting_cursor(category_index)
                    .prompt()?.to_string();
                requirement.category = new_category;
            },
            "⚡ Priority" => {
                let priority_names = vec!["Critical", "High", "Medium", "Low"];
                let current_priority_index = match requirement.priority {
                    Priority::Critical => 0,
                    Priority::High => 1,
                    Priority::Medium => 2,
                    Priority::Low => 3,
                };
                let priority_str = Select::new("Priority:", priority_names)
                    .with_starting_cursor(current_priority_index)
                    .prompt()?;
                requirement.priority = match priority_str {
                    "Critical" => Priority::Critical,
                    "High" => Priority::High,
                    "Medium" => Priority::Medium,
                    "Low" => Priority::Low,
                    _ => Priority::Medium,
                };
            },
            "💾 Save Changes" => {
                requirement.update_timestamp();
                let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
                repo.update_requirement(requirement.clone())?;
                repo.save_to_directory(&quality_dir)?;
                println!("{} Requirement '{}' updated successfully!", "✓".green(), requirement.name);
                break;
            },
            "❌ Cancel" => {
                println!("{} Edit cancelled", "ℹ".blue());
                break;
            },
            _ => {}
        }
    }
    
    Ok(())
}

// Placeholder edit functions
async fn edit_input_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit functionality for design inputs coming soon...".yellow());
    Ok(())
}

async fn edit_output_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit functionality for design outputs coming soon...".yellow());
    Ok(())
}

async fn edit_verification_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit functionality for verifications coming soon...".yellow());
    Ok(())
}

async fn edit_control_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit functionality for design controls coming soon...".yellow());
    Ok(())
}

async fn edit_risk_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Editing risk".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found to edit".yellow());
        return Ok(());
    }
    
    // Select risk to edit
    let risk_options: Vec<String> = risks.iter()
        .map(|r| format!("{} - {} (Score: {:.3})", r.name, truncate_string(&r.description, 40), r.risk_score))
        .collect();
    
    let risk_selection = Select::new("Select risk to edit:", risk_options.clone()).prompt()?;
    let risk_index = risk_options.iter().position(|x| x == &risk_selection).unwrap();
    let mut risk = risks[risk_index].clone();
    
    // Get risk scoring configuration
    let prob_config = &project_ctx.metadata.quality_settings.risk_probability_range;
    let impact_config = &project_ctx.metadata.quality_settings.risk_impact_range;
    let risk_thresholds = &project_ctx.metadata.quality_settings.risk_tolerance_thresholds;
    
    // Show current values and let user choose what to edit
    loop {
        println!("\n{}", format!("Editing risk: {}", risk.name).bold().green());
        println!("Current values:");
        println!("  Name: {}", risk.name);
        println!("  Description: {}", risk.description);
        println!("  Category: {}", risk.category);
        println!("  Probability: {}", risk.probability);
        println!("  Impact: {}", risk.impact);
        println!("  Risk Score: {:.3}", risk.risk_score);
        println!("  Failure Mode: {}", risk.failure_mode);
        println!("  Cause of Failure: {}", risk.cause_of_failure);
        println!("  Effect of Failure: {}", risk.effect_of_failure);
        println!("  Mitigation Strategy: {}", risk.mitigation_strategy);
        println!("  Owner: {}", risk.owner);
        
        let field_options = vec![
            "📝 Name",
            "📄 Description",
            "📂 Category",
            "📊 Probability",
            "💥 Impact",
            "⚠️  Failure Mode",
            "🔍 Cause of Failure",
            "💥 Effect of Failure",
            "🛡️  Mitigation Strategy",
            "👤 Owner",
            "💾 Save Changes",
            "❌ Cancel",
        ];
        
        let field_selection = Select::new("Select field to edit:", field_options)
            .with_help_message("Choose which field to modify")
            .prompt()?;
        
        match field_selection {
            "📝 Name" => {
                let new_name = Text::new("Name:")
                    .with_default(&risk.name)
                    .prompt()?;
                risk.name = new_name;
            },
            "📄 Description" => {
                let new_description = Text::new("Description:")
                    .with_default(&risk.description)
                    .prompt()?;
                risk.description = new_description;
            },
            "📂 Category" => {
                let categories = vec!["Design", "Process", "Use"];
                let category_index = categories.iter().position(|&c| c == risk.category).unwrap_or(0);
                let new_category = Select::new("Category:", categories)
                    .with_starting_cursor(category_index)
                    .prompt()?.to_string();
                risk.category = new_category;
            },
            "📊 Probability" => {
                let prob_values = prob_config.values();
                let probability_str = Text::new(&format!("Probability [values available: {:?}]:", prob_values))
                    .with_default(&risk.probability.to_string())
                    .prompt()?;
                risk.probability = probability_str.parse().unwrap_or(risk.probability);
                risk.update_risk_score(prob_config, impact_config);
            },
            "💥 Impact" => {
                let impact_values = impact_config.values();
                let impact_str = Text::new(&format!("Impact [values available: {:?}]:", impact_values))
                    .with_default(&risk.impact.to_string())
                    .prompt()?;
                risk.impact = impact_str.parse().unwrap_or(risk.impact);
                risk.update_risk_score(prob_config, impact_config);
            },
            "⚠️  Failure Mode" => {
                let new_failure_mode = Text::new("Failure Mode:")
                    .with_default(&risk.failure_mode)
                    .prompt()?;
                risk.failure_mode = new_failure_mode;
            },
            "🔍 Cause of Failure" => {
                let new_cause = Text::new("Cause of Failure:")
                    .with_default(&risk.cause_of_failure)
                    .prompt()?;
                risk.cause_of_failure = new_cause;
            },
            "💥 Effect of Failure" => {
                let new_effect = Text::new("Effect of Failure:")
                    .with_default(&risk.effect_of_failure)
                    .prompt()?;
                risk.effect_of_failure = new_effect;
            },
            "🛡️  Mitigation Strategy" => {
                let new_mitigation = Text::new("Mitigation Strategy:")
                    .with_default(&risk.mitigation_strategy)
                    .prompt()?;
                risk.mitigation_strategy = new_mitigation;
            },
            "👤 Owner" => {
                let new_owner = Text::new("Owner:")
                    .with_default(&risk.owner)
                    .prompt()?;
                risk.owner = new_owner;
            },
            "💾 Save Changes" => {
                risk.update_timestamp();
                let risk_category = risk_thresholds.categorize_risk(risk.risk_score);
                
                let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
                repo.update_risk(risk.clone())?;
                repo.save_to_directory(&quality_dir)?;
                
                println!("{} Risk '{}' updated successfully!", "✓".green(), risk.name);
                println!("Risk Score: {:.3} ({})", risk.risk_score, risk_category);
                break;
            },
            "❌ Cancel" => {
                println!("{} Edit cancelled", "ℹ".blue());
                break;
            },
            _ => {}
        }
    }
    
    Ok(())
}
