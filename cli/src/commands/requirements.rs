use crate::{RequirementsCommands, utils::truncate_string};
use colored::Colorize;
use comfy_table::Table;
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text, Confirm};
use tessera_requirements::*;

pub async fn execute_requirements_command(command: RequirementsCommands, project_ctx: ProjectContext) -> Result<()> {
    match command {
        RequirementsCommands::AddRequirement => add_requirement_interactive(project_ctx).await,
        RequirementsCommands::ListRequirements => list_requirements(project_ctx).await,
        RequirementsCommands::EditRequirement => edit_requirement_interactive(project_ctx).await,
        RequirementsCommands::AddInput => add_input_interactive(project_ctx).await,
        RequirementsCommands::ListInputs => list_inputs(project_ctx).await,
        RequirementsCommands::EditInput => edit_input_interactive(project_ctx).await,
        RequirementsCommands::AddOutput => add_output_interactive(project_ctx).await,
        RequirementsCommands::ListOutputs => list_outputs(project_ctx).await,
        RequirementsCommands::EditOutput => edit_output_interactive(project_ctx).await,
        RequirementsCommands::AddVerification => add_verification_interactive(project_ctx).await,
        RequirementsCommands::ListVerifications => list_verifications(project_ctx).await,
        RequirementsCommands::EditVerification => edit_verification_interactive(project_ctx).await,
        RequirementsCommands::Dashboard => show_dashboard(project_ctx).await,
        RequirementsCommands::TraceabilityMatrix => show_traceability_matrix(project_ctx).await,
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
        "Security",
        "Regulatory",
        "Usability",
        "Reliability",
        "Maintainability",
        "Interface",
        "Custom",
    ];
    
    let category_str = Select::new("Category:", categories).prompt()?;
    let category = match category_str {
        "Functional" => RequirementCategory::Functional,
        "Performance" => RequirementCategory::Performance,
        "Safety" => RequirementCategory::Safety,
        "Security" => RequirementCategory::Security,
        "Regulatory" => RequirementCategory::Regulatory,
        "Usability" => RequirementCategory::Usability,
        "Reliability" => RequirementCategory::Reliability,
        "Maintainability" => RequirementCategory::Maintainability,
        "Interface" => RequirementCategory::Interface,
        _ => {
            let other_name = Text::new("Custom category name:").prompt()?;
            RequirementCategory::Custom(other_name)
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
    
    let requirement = Requirement::new(name, description, category, priority);
    
    let requirements_dir = project_ctx.module_path("requirements");
    if !requirements_dir.exists() {
        std::fs::create_dir_all(&requirements_dir)?;
    }
    
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    repo.add_requirement(requirement.clone())?;
    repo.save_to_directory(&requirements_dir)?;
    
    println!("{} Requirement '{}' added successfully!", "✓".green(), requirement.name);
    println!("ID: {}", requirement.id);
    
    Ok(())
}

async fn list_requirements(project_ctx: ProjectContext) -> Result<()> {
    let requirements_dir = project_ctx.module_path("requirements");
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
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
            RequirementCategory::Security => "Security",
            RequirementCategory::Regulatory => "Regulatory",
            RequirementCategory::Usability => "Usability",
            RequirementCategory::Reliability => "Reliability",
            RequirementCategory::Maintainability => "Maintainability",
            RequirementCategory::Interface => "Interface",
            RequirementCategory::Custom(name) => name,
        };
        
        let priority = match req.priority {
            Priority::Critical => "Critical".red(),
            Priority::High => "High".yellow(),
            Priority::Medium => "Medium".blue(),
            Priority::Low => "Low".green(),
        };
        
        let status = match req.status {
            RequirementStatus::Draft => "Draft".cyan(),
            RequirementStatus::UnderReview => "Under Review".yellow(),
            RequirementStatus::Approved => "Approved".green(),
            RequirementStatus::Implemented => "Implemented".blue(),
            RequirementStatus::Verified => "Verified".green(),
            RequirementStatus::Obsolete => "Obsolete".red(),
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

async fn edit_requirement_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit requirement functionality not yet implemented.".yellow());
    Ok(())
}

async fn add_input_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design input".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    if !requirements_dir.exists() {
        std::fs::create_dir_all(&requirements_dir)?;
    }
    
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let requirements = repo.get_requirements();
    
    if requirements.is_empty() {
        println!("{}", "No requirements found. Please add requirements first before creating design inputs.".yellow());
        println!("Design inputs must be linked to requirements to ensure proper traceability.");
        return Ok(());
    }
    
    // Select requirement to link to FIRST
    println!("{}", "Select Requirement".bold());
    let req_options: Vec<String> = requirements.iter()
        .map(|r| format!("{} - {}", r.name, truncate_string(&r.description, 50)))
        .collect();
    
    let req_selection = Select::new("Select requirement to create a design input for:", req_options.clone())
        .with_help_message("Design inputs must be linked to requirements for traceability")
        .prompt()?;
    
    let req_index = req_options.iter().position(|x| x == &req_selection).unwrap();
    let selected_requirement = &requirements[req_index];
    let requirement_id = selected_requirement.id;
    
    println!("\nCreating design input for requirement: {}", selected_requirement.name.green());
    
    let name = Text::new("Input name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let source = Text::new("Source:")
        .with_help_message("Document reference, URL, or source identifier")
        .prompt()?;
    
    let mut input = DesignInput::new(name, description, requirement_id, source);
    
    let add_criteria = Confirm::new("Add acceptance criteria?")
        .with_default(false)
        .prompt()?;
    
    if add_criteria {
        loop {
            let criterion = Text::new("Acceptance criterion:")
                .with_help_message("Enter acceptance criterion (empty to finish)")
                .prompt()?;
            
            if criterion.is_empty() {
                break;
            }
            
            input.add_acceptance_criterion(criterion);
        }
    }
    
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    repo.add_design_input(input.clone())?;
    repo.save_to_directory(&requirements_dir)?;
    
    println!("{} Design input '{}' added successfully!", "✓".green(), input.name);
    println!("ID: {}", input.id);
    println!("Linked to requirement: {}", selected_requirement.name);
    
    Ok(())
}

async fn list_inputs(project_ctx: ProjectContext) -> Result<()> {
    let requirements_dir = project_ctx.module_path("requirements");
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let inputs = repo.get_design_inputs();
    
    if inputs.is_empty() {
        println!("{}", "No design inputs found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Inputs".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Description", "Source"]);
    
    for input in inputs {
        table.add_row(vec![
            input.id.to_string(),
            truncate_string(&input.name, 25),
            truncate_string(&input.description, 40),
            truncate_string(&input.source, 30),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn edit_input_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit design input functionality not yet implemented.".yellow());
    Ok(())
}

async fn add_output_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new design output".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    if !requirements_dir.exists() {
        std::fs::create_dir_all(&requirements_dir)?;
    }
    
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let inputs = repo.get_design_inputs();
    
    if inputs.is_empty() {
        println!("{}", "No design inputs found. Please add design inputs first before creating design outputs.".yellow());
        println!("Design outputs must be linked to design inputs to ensure proper workflow.");
        return Ok(());
    }
    
    // Select design input to link to FIRST
    println!("{}", "Select Design Input".bold());
    let input_options: Vec<String> = inputs.iter()
        .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
        .collect();
    
    let input_selection = Select::new("Select design input to create a design output for:", input_options.clone())
        .with_help_message("Design outputs must be linked to design inputs for traceability")
        .prompt()?;
    
    let input_index = input_options.iter().position(|x| x == &input_selection).unwrap();
    let selected_input = &inputs[input_index];
    let input_id = selected_input.id;
    
    println!("\nCreating design output for input: {}", selected_input.name.green());
    
    let name = Text::new("Output name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let output_type = Text::new("Output type:")
        .with_help_message("e.g., Drawing, Calculation, Specification, Report, etc.")
        .with_default("Document")
        .prompt()?;
    
    let deliverable = Text::new("Deliverable:")
        .with_help_message("File path, document location, or deliverable description")
        .prompt()?;
    
    let output = DesignOutput::new(name, description, input_id, output_type, deliverable);
    
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    repo.add_design_output(output.clone())?;
    repo.save_to_directory(&requirements_dir)?;
    
    println!("{} Design output '{}' added successfully!", "✓".green(), output.name);
    println!("ID: {}", output.id);
    println!("Linked to design input: {}", selected_input.name);
    
    Ok(())
}

async fn list_outputs(project_ctx: ProjectContext) -> Result<()> {
    let requirements_dir = project_ctx.module_path("requirements");
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let outputs = repo.get_design_outputs();
    
    if outputs.is_empty() {
        println!("{}", "No design outputs found".yellow());
        return Ok(());
    }
    
    println!("{}", "Design Outputs".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Description", "Status"]);
    
    for output in outputs {
        let status = output.approval_status.clone();
        
        table.add_row(vec![
            output.id.to_string(),
            truncate_string(&output.name, 25),
            truncate_string(&output.description, 40),
            status,
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn edit_output_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit design output functionality not yet implemented.".yellow());
    Ok(())
}

async fn add_verification_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new verification".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    if !requirements_dir.exists() {
        std::fs::create_dir_all(&requirements_dir)?;
    }
    
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let outputs = repo.get_design_outputs();
    
    if outputs.is_empty() {
        println!("{}", "No design outputs found. Please add design outputs first before creating verifications.".yellow());
        println!("Verifications must be linked to design outputs to ensure proper workflow.");
        return Ok(());
    }
    
    // Select design output to link to FIRST
    println!("{}", "Select Design Output".bold());
    let output_options: Vec<String> = outputs.iter()
        .map(|o| format!("{} - {}", o.name, truncate_string(&o.description, 50)))
        .collect();
    
    let output_selection = Select::new("Select design output to create a verification for:", output_options.clone())
        .with_help_message("Verifications must be linked to design outputs for traceability")
        .prompt()?;
    
    let output_index = output_options.iter().position(|x| x == &output_selection).unwrap();
    let selected_output = &outputs[output_index];
    let output_id = selected_output.id;
    
    println!("\nCreating verification for output: {}", selected_output.name.green());
    
    let name = Text::new("Verification name:")
        .prompt()?;
    
    let description = Text::new("Description:")
        .prompt()?;
    
    let verification_type = Text::new("Verification type:")
        .with_help_message("e.g., Test, Review, Inspection, Analysis, etc.")
        .with_default("Test")
        .prompt()?;
    
    let method = Text::new("Verification method:")
        .with_help_message("How will this verification be performed?")
        .with_default("Manual Review")
        .prompt()?;
    
    let acceptance_criteria = Text::new("Acceptance criteria:")
        .with_help_message("How will this verification be evaluated?")
        .prompt()?;
    
    let mut verification = Verification::new(name, description, output_id, verification_type, method);
    verification.add_acceptance_criterion(acceptance_criteria);
    
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    repo.add_verification(verification.clone())?;
    repo.save_to_directory(&requirements_dir)?;
    
    println!("{} Verification '{}' added successfully!", "✓".green(), verification.name);
    println!("ID: {}", verification.id);
    println!("Linked to design output: {}", selected_output.name);
    
    Ok(())
}

async fn list_verifications(project_ctx: ProjectContext) -> Result<()> {
    let requirements_dir = project_ctx.module_path("requirements");
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let verifications = repo.get_verifications();
    
    if verifications.is_empty() {
        println!("{}", "No verifications found".yellow());
        return Ok(());
    }
    
    println!("{}", "Verifications".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Description", "Status"]);
    
    for verification in verifications {
        let status = verification.status.clone();
        
        table.add_row(vec![
            verification.id.to_string(),
            truncate_string(&verification.name, 25),
            truncate_string(&verification.description, 40),
            status,
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn edit_verification_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit verification functionality not yet implemented.".yellow());
    Ok(())
}

async fn show_dashboard(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_requirements::RequirementsCommands::new(project_ctx)?;
    commands.show_dashboard()
}

async fn show_traceability_matrix(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Traceability matrix functionality not yet implemented.".yellow());
    Ok(())
}