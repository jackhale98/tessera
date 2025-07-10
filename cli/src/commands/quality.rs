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
        QualityCommands::AddInput => add_input_interactive(project_ctx).await,
        QualityCommands::ListInputs => list_inputs(project_ctx).await,
        QualityCommands::LinkInputToRequirement => link_input_to_requirement_interactive(project_ctx).await,
        QualityCommands::AddOutput => add_output_interactive(project_ctx).await,
        QualityCommands::ListOutputs => list_outputs(project_ctx).await,
        QualityCommands::LinkOutputToRequirement => link_output_to_requirement_interactive(project_ctx).await,
        QualityCommands::LinkOutputToInput => link_output_to_input_interactive(project_ctx).await,
        QualityCommands::AddControl => add_control_interactive(project_ctx).await,
        QualityCommands::ListControls => list_controls(project_ctx).await,
        QualityCommands::LinkControlToOutput => link_control_to_output_interactive(project_ctx).await,
        QualityCommands::AddRisk => add_risk_interactive(project_ctx).await,
        QualityCommands::ListRisks => list_risks(project_ctx).await,
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
    
    let category_str = Select::new("Category:", categories).prompt()?;
    let category = match category_str {
        "Functional" => RequirementCategory::Functional,
        "Performance" => RequirementCategory::Performance,
        "Safety" => RequirementCategory::Safety,
        "Regulatory" => RequirementCategory::Regulatory,
        "Usability" => RequirementCategory::Usability,
        "Reliability" => RequirementCategory::Reliability,
        "Maintainability" => RequirementCategory::Maintainability,
        "Environmental" => RequirementCategory::Environmental,
        _ => {
            let other_name = Text::new("Other category name:").prompt()?;
            RequirementCategory::Other(other_name)
        }
    };
    
    let priorities = vec!["Critical", "High", "Medium", "Low"];
    let priority_str = Select::new("Priority:", priorities).prompt()?;
    let priority = match priority_str {
        "Critical" => Priority::Critical,
        "High" => Priority::High,
        "Medium" => Priority::Medium,
        "Low" => Priority::Low,
        _ => Priority::Medium,
    };
    
    let mut requirement = Requirement::new(name, description, category);
    requirement.priority = priority;
    
    let add_criteria = Confirm::new("Add acceptance criteria?")
        .with_default(false)
        .prompt()?;
    
    if add_criteria {
        let mut criteria = Vec::new();
        loop {
            let criterion = Text::new("Acceptance criterion:")
                .with_help_message("Enter acceptance criterion (empty to finish)")
                .prompt()?;
            
            if criterion.is_empty() {
                break;
            }
            
            criteria.push(criterion);
        }
        requirement.acceptance_criteria = criteria;
    }
    
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
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let requirements = repo.get_requirements();
    
    if requirements.is_empty() {
        println!("{}", "No requirements found".yellow());
        return Ok(());
    }
    
    println!("{}", "Requirements".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Category", "Priority", "Status"]);
    
    for req in requirements {
        let category = match &req.category {
            RequirementCategory::Functional => "Functional",
            RequirementCategory::Performance => "Performance",
            RequirementCategory::Safety => "Safety",
            RequirementCategory::Regulatory => "Regulatory",
            RequirementCategory::Usability => "Usability",
            RequirementCategory::Reliability => "Reliability",
            RequirementCategory::Maintainability => "Maintainability",
            RequirementCategory::Environmental => "Environmental",
            RequirementCategory::Other(name) => name,
        };
        
        let priority = match req.priority {
            Priority::Critical => "Critical".red(),
            Priority::High => "High".yellow(),
            Priority::Medium => "Medium".blue(),
            Priority::Low => "Low".green(),
        };
        
        let status = match req.status {
            RequirementStatus::Draft => "Draft".cyan(),
            RequirementStatus::Approved => "Approved".green(),
            RequirementStatus::Implemented => "Implemented".blue(),
            RequirementStatus::Verified => "Verified".green(),
            RequirementStatus::Failed => "Failed".red(),
            RequirementStatus::Deprecated => "Deprecated".red(),
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
    
    let input_type_str = Select::new("Input type:", input_types).prompt()?;
    let input_type = match input_type_str {
        "Specification" => InputType::Specification,
        "Standard" => InputType::Standard,
        "Regulation" => InputType::Regulation,
        "Customer Requirement" => InputType::CustomerRequirement,
        "Market Research" => InputType::MarketResearch,
        "Technical Report" => InputType::TechnicalReport,
        _ => {
            let other_name = Text::new("Other input type:").prompt()?;
            InputType::Other(other_name)
        }
    };
    
    let source = Text::new("Source:")
        .with_help_message("Document reference, URL, or source identifier")
        .prompt()?;
    
    let input = DesignInput::new(name, description, input_type, source);
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_input(input.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Design input '{}' added successfully!", "✓".green(), input.name);
    println!("ID: {}", input.id);
    
    Ok(())
}

async fn link_input_to_requirement_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Linking input to requirement".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let inputs = repo.get_inputs();
    if inputs.is_empty() {
        println!("{}", "No inputs found. Add inputs first.".yellow());
        return Ok(());
    }
    
    let requirements = repo.get_requirements();
    if requirements.is_empty() {
        println!("{}", "No requirements found. Add requirements first.".yellow());
        return Ok(());
    }
    
    let input_options: Vec<String> = inputs.iter()
        .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
        .collect();
    
    let input_selection = Select::new("Select input:", input_options.clone()).prompt()?;
    let input_index = input_options.iter().position(|x| x == &input_selection).unwrap();
    let selected_input = &inputs[input_index];
    let input_id = selected_input.id;
    let input_name = selected_input.name.clone();
    
    let req_options: Vec<String> = requirements.iter()
        .map(|r| format!("{} - {}", r.name, truncate_string(&r.description, 50)))
        .collect();
    
    let req_selection = Select::new("Select requirement:", req_options.clone()).prompt()?;
    let req_index = req_options.iter().position(|x| x == &req_selection).unwrap();
    let selected_requirement = &requirements[req_index];
    let req_id = selected_requirement.id;
    let req_name = selected_requirement.name.clone();
    
    repo.link_input_to_requirement(input_id, req_id)?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Linked input '{}' to requirement '{}'", 
             "✓".green(), input_name, req_name);
    
    Ok(())
}

async fn add_output_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design output".bold().blue());
    
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
    
    let output_type_str = Select::new("Output type:", output_types).prompt()?;
    let output_type = match output_type_str {
        "Drawing" => OutputType::Drawing,
        "Calculation" => OutputType::Calculation,
        "Specification" => OutputType::Specification,
        "Report" => OutputType::Report,
        "Model" => OutputType::Model,
        "Prototype" => OutputType::Prototype,
        "Test Plan" => OutputType::TestPlan,
        _ => {
            let other_name = Text::new("Other output type:").prompt()?;
            OutputType::Other(other_name)
        }
    };
    
    let output = DesignOutput::new(name, description, output_type);
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_output(output.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Design output '{}' added successfully!", "✓".green(), output.name);
    println!("ID: {}", output.id);
    
    Ok(())
}

async fn add_control_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design control".bold().blue());
    
    let name = Text::new("Control name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let control_types = vec![
        "Review",
        "Inspection",
        "Test",
        "Verification",
        "Validation",
        "Approval",
        "Other",
    ];
    
    let control_type_str = Select::new("Control type:", control_types).prompt()?;
    let control_type = match control_type_str {
        "Review" => ControlType::Review,
        "Inspection" => ControlType::Inspection,
        "Test" => ControlType::Test,
        "Verification" => ControlType::Verification,
        "Validation" => ControlType::Validation,
        "Approval" => ControlType::Approval,
        _ => {
            let other_name = Text::new("Other control type:").prompt()?;
            ControlType::Other(other_name)
        }
    };
    
    let control = DesignControl::new(name, description, control_type);
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_control(control.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Design control '{}' added successfully!", "✓".green(), control.name);
    println!("ID: {}", control.id);
    
    Ok(())
}

async fn add_risk_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new risk".bold().blue());
    
    let name = Text::new("Risk name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let risk_categories = vec![
        "Technical",
        "Schedule",
        "Cost",
        "Quality",
        "Safety",
        "Regulatory",
        "Market",
        "Resource",
        "Other",
    ];
    
    let category_str = Select::new("Risk category:", risk_categories).prompt()?;
    let category = match category_str {
        "Technical" => RiskCategory::Technical,
        "Schedule" => RiskCategory::Schedule,
        "Cost" => RiskCategory::Cost,
        "Quality" => RiskCategory::Quality,
        "Safety" => RiskCategory::Safety,
        "Regulatory" => RiskCategory::Regulatory,
        "Market" => RiskCategory::Market,
        "Resource" => RiskCategory::Resource,
        _ => {
            let other_name = Text::new("Other category name:").prompt()?;
            RiskCategory::Other(other_name)
        }
    };
    
    let probability_str = Text::new("Probability (0.0 - 1.0):")
        .with_default("0.5")
        .prompt()?;
    let probability: f64 = probability_str.parse().unwrap_or(0.5);
    
    let impact_str = Text::new("Impact (0.0 - 1.0):")
        .with_default("0.5")
        .prompt()?;
    let impact: f64 = impact_str.parse().unwrap_or(0.5);
    
    let mut risk = Risk::new(name, description, category);
    risk.probability = probability.clamp(0.0, 1.0);
    risk.impact = impact.clamp(0.0, 1.0);
    risk.update_risk_score();
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    repo.add_risk(risk.clone())?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Risk '{}' added successfully!", "✓".green(), risk.name);
    println!("ID: {}", risk.id);
    println!("Risk Score: {:.2}", risk.risk_score);
    
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
    let controls = repo.get_controls();
    let risks = repo.get_risks();
    
    println!("\n{}", "Summary".bold());
    println!("Requirements: {}", requirements.len());
    println!("Design Inputs: {}", inputs.len());
    println!("Design Outputs: {}", outputs.len());
    println!("Design Controls: {}", controls.len());
    println!("Risks: {}", risks.len());
    
    if !requirements.is_empty() {
        println!("\n{}", "Requirements by Status:".bold());
        let mut status_counts = std::collections::HashMap::new();
        for req in requirements {
            let status = match req.status {
                RequirementStatus::Draft => "Draft",
                RequirementStatus::Approved => "Approved",
                RequirementStatus::Implemented => "Implemented",
                RequirementStatus::Verified => "Verified",
                RequirementStatus::Failed => "Failed",
                RequirementStatus::Deprecated => "Deprecated",
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
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let inputs = repo.get_inputs();
    
    if inputs.is_empty() {
        println!("{}", "No design inputs found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Inputs".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Source", "Requirements"]);
    
    for input in inputs {
        let input_type = match &input.input_type {
            InputType::Specification => "Specification",
            InputType::Standard => "Standard",
            InputType::Regulation => "Regulation",
            InputType::CustomerRequirement => "Customer Req",
            InputType::MarketResearch => "Market Research",
            InputType::TechnicalReport => "Tech Report",
            InputType::Other(name) => name,
        };
        
        table.add_row(vec![
            input.id.to_string(),
            truncate_string(&input.name, 25),
            input_type.to_string(),
            truncate_string(&input.source, 20),
            input.requirements.len().to_string(),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn list_outputs(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let outputs = repo.get_outputs();
    
    if outputs.is_empty() {
        println!("{}", "No design outputs found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Outputs".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Requirements", "Inputs"]);
    
    for output in outputs {
        let output_type = match &output.output_type {
            OutputType::Drawing => "Drawing",
            OutputType::Calculation => "Calculation",
            OutputType::Specification => "Specification",
            OutputType::Report => "Report",
            OutputType::Model => "Model",
            OutputType::Prototype => "Prototype",
            OutputType::TestPlan => "Test Plan",
            OutputType::Other(name) => name,
        };
        
        table.add_row(vec![
            output.id.to_string(),
            truncate_string(&output.name, 25),
            output_type.to_string(),
            output.requirements.len().to_string(),
            output.inputs.len().to_string(),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn list_controls(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let controls = repo.get_controls();
    
    if controls.is_empty() {
        println!("{}", "No design controls found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Controls".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Frequency", "Outputs"]);
    
    for control in controls {
        let control_type = match &control.control_type {
            ControlType::Review => "Review",
            ControlType::Inspection => "Inspection",
            ControlType::Test => "Test",
            ControlType::Verification => "Verification",
            ControlType::Validation => "Validation",
            ControlType::Approval => "Approval",
            ControlType::Other(name) => name,
        };
        
        let frequency = match control.frequency {
            ControlFrequency::OneTime => "One Time",
            ControlFrequency::PerBatch => "Per Batch",
            ControlFrequency::Daily => "Daily",
            ControlFrequency::Weekly => "Weekly",
            ControlFrequency::Monthly => "Monthly",
            ControlFrequency::Quarterly => "Quarterly",
            ControlFrequency::Annually => "Annually",
            ControlFrequency::AsNeeded => "As Needed",
        };
        
        table.add_row(vec![
            control.id.to_string(),
            truncate_string(&control.name, 25),
            control_type.to_string(),
            frequency.to_string(),
            control.outputs.len().to_string(),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn list_risks(project_ctx: ProjectContext) -> Result<()> {
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    let risks = repo.get_risks();
    
    if risks.is_empty() {
        println!("{}", "No risks found".yellow());
        return Ok(());
    }
    
    println!("{}", "Risks".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Category", "Probability", "Impact", "Risk Score"]);
    
    for risk in risks {
        let category = match &risk.category {
            RiskCategory::Technical => "Technical",
            RiskCategory::Schedule => "Schedule",
            RiskCategory::Cost => "Cost",
            RiskCategory::Quality => "Quality",
            RiskCategory::Safety => "Safety",
            RiskCategory::Regulatory => "Regulatory",
            RiskCategory::Market => "Market",
            RiskCategory::Resource => "Resource",
            RiskCategory::Other(name) => name,
        };
        
        let risk_score_color = if risk.risk_score >= 0.7 {
            format!("{:.2}", risk.risk_score).red()
        } else if risk.risk_score >= 0.3 {
            format!("{:.2}", risk.risk_score).yellow()
        } else {
            format!("{:.2}", risk.risk_score).green()
        };
        
        table.add_row(vec![
            risk.id.to_string(),
            truncate_string(&risk.name, 25),
            category.to_string(),
            format!("{:.2}", risk.probability),
            format!("{:.2}", risk.impact),
            risk_score_color.to_string(),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn link_output_to_requirement_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Linking output to requirement".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let outputs = repo.get_outputs();
    if outputs.is_empty() {
        println!("{}", "No outputs found. Add outputs first.".yellow());
        return Ok(());
    }
    
    let requirements = repo.get_requirements();
    if requirements.is_empty() {
        println!("{}", "No requirements found. Add requirements first.".yellow());
        return Ok(());
    }
    
    let output_options: Vec<String> = outputs.iter()
        .map(|o| format!("{} - {}", o.name, truncate_string(&o.description, 50)))
        .collect();
    
    let output_selection = Select::new("Select output:", output_options.clone()).prompt()?;
    let output_index = output_options.iter().position(|x| x == &output_selection).unwrap();
    let selected_output = &outputs[output_index];
    let output_id = selected_output.id;
    let output_name = selected_output.name.clone();
    
    let req_options: Vec<String> = requirements.iter()
        .map(|r| format!("{} - {}", r.name, truncate_string(&r.description, 50)))
        .collect();
    
    let req_selection = Select::new("Select requirement:", req_options.clone()).prompt()?;
    let req_index = req_options.iter().position(|x| x == &req_selection).unwrap();
    let selected_requirement = &requirements[req_index];
    let req_id = selected_requirement.id;
    let req_name = selected_requirement.name.clone();
    
    repo.link_output_to_requirement(output_id, req_id)?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Linked output '{}' to requirement '{}'", 
             "✓".green(), output_name, req_name);
    
    Ok(())
}

async fn link_output_to_input_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Linking output to input".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let outputs = repo.get_outputs();
    if outputs.is_empty() {
        println!("{}", "No outputs found. Add outputs first.".yellow());
        return Ok(());
    }
    
    let inputs = repo.get_inputs();
    if inputs.is_empty() {
        println!("{}", "No inputs found. Add inputs first.".yellow());
        return Ok(());
    }
    
    let output_options: Vec<String> = outputs.iter()
        .map(|o| format!("{} - {}", o.name, truncate_string(&o.description, 50)))
        .collect();
    
    let output_selection = Select::new("Select output:", output_options.clone()).prompt()?;
    let output_index = output_options.iter().position(|x| x == &output_selection).unwrap();
    let selected_output = &outputs[output_index];
    let output_id = selected_output.id;
    let output_name = selected_output.name.clone();
    
    let input_options: Vec<String> = inputs.iter()
        .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
        .collect();
    
    let input_selection = Select::new("Select input:", input_options.clone()).prompt()?;
    let input_index = input_options.iter().position(|x| x == &input_selection).unwrap();
    let selected_input = &inputs[input_index];
    let input_id = selected_input.id;
    let input_name = selected_input.name.clone();
    
    repo.link_output_to_input(output_id, input_id)?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Linked output '{}' to input '{}'", 
             "✓".green(), output_name, input_name);
    
    Ok(())
}

async fn link_control_to_output_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Linking control to output".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let mut repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let controls = repo.get_controls();
    if controls.is_empty() {
        println!("{}", "No controls found. Add controls first.".yellow());
        return Ok(());
    }
    
    let outputs = repo.get_outputs();
    if outputs.is_empty() {
        println!("{}", "No outputs found. Add outputs first.".yellow());
        return Ok(());
    }
    
    let control_options: Vec<String> = controls.iter()
        .map(|c| format!("{} - {}", c.name, truncate_string(&c.description, 50)))
        .collect();
    
    let control_selection = Select::new("Select control:", control_options.clone()).prompt()?;
    let control_index = control_options.iter().position(|x| x == &control_selection).unwrap();
    let selected_control = &controls[control_index];
    let control_id = selected_control.id;
    let control_name = selected_control.name.clone();
    
    let output_options: Vec<String> = outputs.iter()
        .map(|o| format!("{} - {}", o.name, truncate_string(&o.description, 50)))
        .collect();
    
    let output_selection = Select::new("Select output:", output_options.clone()).prompt()?;
    let output_index = output_options.iter().position(|x| x == &output_selection).unwrap();
    let selected_output = &outputs[output_index];
    let output_id = selected_output.id;
    let output_name = selected_output.name.clone();
    
    repo.link_control_to_output(control_id, output_id)?;
    repo.save_to_directory(&quality_dir)?;
    
    println!("{} Linked control '{}' to output '{}'", 
             "✓".green(), control_name, output_name);
    
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
    
    let mut menu = ScoringMenuInterface::new();
    menu.show_scoring_menu(&mut repo)
}