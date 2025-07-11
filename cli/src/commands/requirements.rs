use crate::{RequirementsCommands, utils::truncate_string};
use colored::Colorize;
use comfy_table::Table;
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text, Confirm};
use tessera_requirements::*;
use chrono::{DateTime, Utc};

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

async fn edit_requirement_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit Requirement".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let requirements = repo.get_requirements();
    
    if requirements.is_empty() {
        println!("{}", "No requirements found. Add requirements first.".yellow());
        return Ok(());
    }
    
    // Select requirement to edit
    let req_options: Vec<String> = requirements.iter()
        .map(|r| format!("{} - {}", r.name, truncate_string(&r.description, 50)))
        .collect();
    
    let req_selection = Select::new("Select requirement to edit:", req_options.clone())
        .prompt()?;
    
    let req_index = req_options.iter().position(|x| x == &req_selection).unwrap();
    let mut requirement = requirements[req_index].clone();
    
    loop {
        println!("\n{}", format!("Editing: {}", requirement.name).bold());
        
        let edit_options = vec![
            "📝 Edit name",
            "📄 Edit description", 
            "🏷️  Edit category",
            "⚡ Edit priority",
            "📊 Edit status",
            "💭 Edit rationale",
            "📖 Edit source",
            "👤 Edit stakeholder",
            "🏷️  Manage tags",
            "🗂️  Manage metadata",
            "💾 Save changes",
            "❌ Cancel",
        ];
        
        let choice = Select::new("What would you like to edit?", edit_options)
            .prompt()?;
        
        match choice {
            "📝 Edit name" => {
                let new_name = Text::new("New name:")
                    .with_default(&requirement.name)
                    .prompt()?;
                requirement.name = new_name;
                requirement.updated = Utc::now();
            },
            "📄 Edit description" => {
                let new_description = Text::new("New description:")
                    .with_default(&requirement.description)
                    .prompt()?;
                requirement.description = new_description;
                requirement.updated = Utc::now();
            },
            "🏷️  Edit category" => {
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
                
                let current_category = match &requirement.category {
                    RequirementCategory::Functional => "Functional",
                    RequirementCategory::Performance => "Performance",
                    RequirementCategory::Safety => "Safety",
                    RequirementCategory::Security => "Security",
                    RequirementCategory::Regulatory => "Regulatory",
                    RequirementCategory::Usability => "Usability",
                    RequirementCategory::Reliability => "Reliability",
                    RequirementCategory::Maintainability => "Maintainability",
                    RequirementCategory::Interface => "Interface",
                    RequirementCategory::Custom(_) => "Custom",
                };
                
                let category_str = Select::new("Category:", categories.clone())
                    .with_starting_cursor(categories.iter().position(|&x| x == current_category).unwrap_or(0))
                    .prompt()?;
                    
                requirement.category = match category_str {
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
                requirement.updated = Utc::now();
            },
            "⚡ Edit priority" => {
                let priorities = vec!["Critical", "High", "Medium", "Low"];
                let current_priority = match requirement.priority {
                    Priority::Critical => "Critical",
                    Priority::High => "High", 
                    Priority::Medium => "Medium",
                    Priority::Low => "Low",
                };
                
                let priority_str = Select::new("Priority:", priorities.clone())
                    .with_starting_cursor(priorities.iter().position(|&x| x == current_priority).unwrap())
                    .prompt()?;
                    
                requirement.priority = match priority_str {
                    "Critical" => Priority::Critical,
                    "High" => Priority::High,
                    "Medium" => Priority::Medium,
                    "Low" => Priority::Low,
                    _ => Priority::Medium,
                };
                requirement.updated = Utc::now();
            },
            "📊 Edit status" => {
                let statuses = vec!["Draft", "Under Review", "Approved", "Implemented", "Verified", "Obsolete"];
                let current_status = match requirement.status {
                    RequirementStatus::Draft => "Draft",
                    RequirementStatus::UnderReview => "Under Review",
                    RequirementStatus::Approved => "Approved",
                    RequirementStatus::Implemented => "Implemented",
                    RequirementStatus::Verified => "Verified",
                    RequirementStatus::Obsolete => "Obsolete",
                };
                
                let status_str = Select::new("Status:", statuses.clone())
                    .with_starting_cursor(statuses.iter().position(|&x| x == current_status).unwrap())
                    .prompt()?;
                    
                requirement.status = match status_str {
                    "Draft" => RequirementStatus::Draft,
                    "Under Review" => RequirementStatus::UnderReview,
                    "Approved" => RequirementStatus::Approved,
                    "Implemented" => RequirementStatus::Implemented,
                    "Verified" => RequirementStatus::Verified,
                    "Obsolete" => RequirementStatus::Obsolete,
                    _ => RequirementStatus::Draft,
                };
                requirement.updated = Utc::now();
            },
            "💭 Edit rationale" => {
                let current_rationale = requirement.rationale.as_deref().unwrap_or("");
                let new_rationale = Text::new("Rationale:")
                    .with_help_message("Why is this requirement needed?")
                    .with_default(current_rationale)
                    .prompt()?;
                requirement.rationale = if new_rationale.trim().is_empty() { 
                    None 
                } else { 
                    Some(new_rationale) 
                };
                requirement.updated = Utc::now();
            },
            "📖 Edit source" => {
                let current_source = requirement.source.as_deref().unwrap_or("");
                let new_source = Text::new("Source:")
                    .with_help_message("Source document, standard, or stakeholder")
                    .with_default(current_source)
                    .prompt()?;
                requirement.source = if new_source.trim().is_empty() { 
                    None 
                } else { 
                    Some(new_source) 
                };
                requirement.updated = Utc::now();
            },
            "👤 Edit stakeholder" => {
                let current_stakeholder = requirement.stakeholder.as_deref().unwrap_or("");
                let new_stakeholder = Text::new("Stakeholder:")
                    .with_help_message("Who requested or owns this requirement?")
                    .with_default(current_stakeholder)
                    .prompt()?;
                requirement.stakeholder = if new_stakeholder.trim().is_empty() { 
                    None 
                } else { 
                    Some(new_stakeholder) 
                };
                requirement.updated = Utc::now();
            },
            "🏷️  Manage tags" => {
                manage_tags(&mut requirement.tags, &mut requirement.updated)?;
            },
            "🗂️  Manage metadata" => {
                manage_metadata(&mut requirement.metadata, &mut requirement.updated)?;
            },
            "💾 Save changes" => {
                repo.update_requirement(requirement.clone())?;
                repo.save_to_directory(&requirements_dir)?;
                println!("{} Requirement '{}' updated successfully!", "✓".green(), requirement.name);
                break;
            },
            "❌ Cancel" => {
                println!("Changes cancelled.");
                break;
            },
            _ => {}
        }
    }
    
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

async fn edit_input_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit Design Input".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let inputs = repo.get_design_inputs();
    
    if inputs.is_empty() {
        println!("{}", "No design inputs found. Add design inputs first.".yellow());
        return Ok(());
    }
    
    // Select design input to edit
    let input_options: Vec<String> = inputs.iter()
        .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
        .collect();
    
    let input_selection = Select::new("Select design input to edit:", input_options.clone())
        .prompt()?;
    
    let input_index = input_options.iter().position(|x| x == &input_selection).unwrap();
    let mut input = inputs[input_index].clone();
    
    loop {
        println!("\n{}", format!("Editing: {}", input.name).bold());
        
        let edit_options = vec![
            "📝 Edit name",
            "📄 Edit description",
            "📖 Edit source",
            "🔗 Change linked requirement",
            "✅ Manage acceptance criteria",
            "🚧 Manage constraints",
            "💭 Manage assumptions",
            "📚 Manage references",
            "🏷️  Manage tags",
            "🗂️  Manage metadata",
            "💾 Save changes",
            "❌ Cancel",
        ];
        
        let choice = Select::new("What would you like to edit?", edit_options)
            .prompt()?;
        
        match choice {
            "📝 Edit name" => {
                let new_name = Text::new("New name:")
                    .with_default(&input.name)
                    .prompt()?;
                input.name = new_name;
                input.updated = Utc::now();
            },
            "📄 Edit description" => {
                let new_description = Text::new("New description:")
                    .with_default(&input.description)
                    .prompt()?;
                input.description = new_description;
                input.updated = Utc::now();
            },
            "📖 Edit source" => {
                let new_source = Text::new("New source:")
                    .with_help_message("Document reference, URL, or source identifier")
                    .with_default(&input.source)
                    .prompt()?;
                input.source = new_source;
                input.updated = Utc::now();
            },
            "🔗 Change linked requirement" => {
                let requirements = repo.get_requirements();
                if requirements.is_empty() {
                    println!("{} No requirements available to link to", "⚠".yellow());
                    continue;
                }
                
                let req_options: Vec<String> = requirements.iter()
                    .map(|r| format!("{} - {}", r.name, truncate_string(&r.description, 50)))
                    .collect();
                
                let req_selection = Select::new("Select new requirement to link to:", req_options.clone())
                    .prompt()?;
                
                let req_index = req_options.iter().position(|x| x == &req_selection).unwrap();
                let selected_requirement = &requirements[req_index];
                
                input.requirement_id = selected_requirement.id;
                input.updated = Utc::now();
                println!("{} Linked to requirement: {}", "✓".green(), selected_requirement.name);
            },
            "✅ Manage acceptance criteria" => {
                manage_list_items(&mut input.acceptance_criteria, "acceptance criteria", &mut input.updated)?;
            },
            "🚧 Manage constraints" => {
                manage_list_items(&mut input.constraints, "constraints", &mut input.updated)?;
            },
            "💭 Manage assumptions" => {
                manage_list_items(&mut input.assumptions, "assumptions", &mut input.updated)?;
            },
            "📚 Manage references" => {
                manage_list_items(&mut input.references, "references", &mut input.updated)?;
            },
            "🏷️  Manage tags" => {
                // input doesn't have tags - would need to add to data structure
                println!("{} Tags not available for design inputs", "ℹ".blue());
            },
            "🗂️  Manage metadata" => {
                manage_metadata(&mut input.metadata, &mut input.updated)?;
            },
            "💾 Save changes" => {
                repo.update_design_input(input.clone())?;
                repo.save_to_directory(&requirements_dir)?;
                println!("{} Design input '{}' updated successfully!", "✓".green(), input.name);
                break;
            },
            "❌ Cancel" => {
                println!("Changes cancelled.");
                break;
            },
            _ => {}
        }
    }
    
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
    
    // Select design inputs to link to FIRST (allow multiple)
    println!("{}", "Select Design Inputs".bold());
    let input_options: Vec<String> = inputs.iter()
        .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
        .collect();
    
    let mut selected_input_ids = Vec::new();
    loop {
        let mut available_options = input_options.clone();
        available_options.push("✅ Done selecting inputs".to_string());
        
        if selected_input_ids.is_empty() {
            available_options.remove(available_options.len() - 1);
        }
        
        let input_selection = Select::new("Select design input (or Done when finished):", available_options.clone())
            .prompt()?;
        
        if input_selection == "✅ Done selecting inputs" {
            break;
        }
        
        let input_index = input_options.iter().position(|x| x == &input_selection).unwrap();
        let selected_input = &inputs[input_index];
        
        if !selected_input_ids.contains(&selected_input.id) {
            selected_input_ids.push(selected_input.id);
            println!("{} Added: {}", "✓".green(), selected_input.name);
        } else {
            println!("{} Already selected: {}", "ℹ".blue(), selected_input.name);
        }
        
        if selected_input_ids.len() >= inputs.len() {
            break;
        }
    }
    
    if selected_input_ids.is_empty() {
        println!("{} No design inputs selected. Operation cancelled.", "⚠".yellow());
        return Ok(());
    }
    
    println!("\nCreating design output for {} selected input(s)", selected_input_ids.len().to_string().green());
    
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
    
    let output = DesignOutput::new(name, description, selected_input_ids.clone(), output_type, deliverable);
    
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    repo.add_design_output(output.clone())?;
    repo.save_to_directory(&requirements_dir)?;
    
    println!("{} Design output '{}' added successfully!", "✓".green(), output.name);
    println!("ID: {}", output.id);
    println!("Linked to {} design input(s)", selected_input_ids.len());
    
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

async fn edit_output_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit Design Output".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let outputs = repo.get_design_outputs();
    
    if outputs.is_empty() {
        println!("{}", "No design outputs found. Add design outputs first.".yellow());
        return Ok(());
    }
    
    // Select design output to edit
    let output_options: Vec<String> = outputs.iter()
        .map(|o| format!("{} - {}", o.name, truncate_string(&o.description, 50)))
        .collect();
    
    let output_selection = Select::new("Select design output to edit:", output_options.clone())
        .prompt()?;
    
    let output_index = output_options.iter().position(|x| x == &output_selection).unwrap();
    let mut output = outputs[output_index].clone();
    
    loop {
        println!("\n{}", format!("Editing: {}", output.name).bold());
        
        let edit_options = vec![
            "📝 Edit name",
            "📄 Edit description",
            "🏷️  Edit output type",
            "📦 Edit deliverable",
            "📍 Edit location",
            "🔗 Manage linked design inputs",
            "📊 Edit approval status",
            "🏷️  Edit version",
            "🗂️  Manage metadata",
            "💾 Save changes",
            "❌ Cancel",
        ];
        
        let choice = Select::new("What would you like to edit?", edit_options)
            .prompt()?;
        
        match choice {
            "📝 Edit name" => {
                let new_name = Text::new("New name:")
                    .with_default(&output.name)
                    .prompt()?;
                output.name = new_name;
                output.updated = Utc::now();
            },
            "📄 Edit description" => {
                let new_description = Text::new("New description:")
                    .with_default(&output.description)
                    .prompt()?;
                output.description = new_description;
                output.updated = Utc::now();
            },
            "🏷️  Edit output type" => {
                let new_output_type = Text::new("New output type:")
                    .with_help_message("e.g., Drawing, Calculation, Specification, Report, etc.")
                    .with_default(&output.output_type)
                    .prompt()?;
                output.output_type = new_output_type;
                output.updated = Utc::now();
            },
            "📦 Edit deliverable" => {
                let new_deliverable = Text::new("New deliverable:")
                    .with_help_message("File path, document location, or deliverable description")
                    .with_default(&output.deliverable)
                    .prompt()?;
                output.deliverable = new_deliverable;
                output.updated = Utc::now();
            },
            "📍 Edit location" => {
                let current_location = output.location.as_deref().unwrap_or("");
                let new_location = Text::new("Location:")
                    .with_help_message("Physical or digital location of the deliverable")
                    .with_default(current_location)
                    .prompt()?;
                output.location = if new_location.trim().is_empty() { 
                    None 
                } else { 
                    Some(new_location) 
                };
                output.updated = Utc::now();
            },
            "🔗 Manage linked design inputs" => {
                let inputs = repo.get_design_inputs();
                if inputs.is_empty() {
                    println!("{} No design inputs available to link to", "⚠".yellow());
                    continue;
                }
                
                // Manage multiple linked design inputs
                loop {
                    println!("\nCurrently linked to {} design input(s)", output.input_ids.len());
                    
                    let mut action_options = vec!["Add design input", "Remove design input", "Done"];
                    
                    let action = Select::new("What would you like to do?", action_options)
                        .prompt()?;
                    
                    match action {
                        "Add design input" => {
                            let available_inputs: Vec<_> = inputs.iter()
                                .filter(|i| !output.input_ids.contains(&i.id))
                                .collect();
                            
                            if available_inputs.is_empty() {
                                println!("{} All inputs are already linked", "ℹ".blue());
                                continue;
                            }
                            
                            let input_options: Vec<String> = available_inputs.iter()
                                .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
                                .collect();
                            
                            let input_selection = Select::new("Select design input to add:", input_options)
                                .prompt()?;
                            
                            let input_index = available_inputs.iter()
                                .position(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)) == input_selection)
                                .unwrap();
                            let selected_input = available_inputs[input_index];
                            
                            output.input_ids.push(selected_input.id);
                            output.updated = Utc::now();
                            println!("{} Added: {}", "✓".green(), selected_input.name);
                        },
                        "Remove design input" => {
                            if output.input_ids.is_empty() {
                                println!("{} No inputs linked", "ℹ".blue());
                                continue;
                            }
                            
                            let linked_inputs: Vec<_> = output.input_ids.iter()
                                .filter_map(|id| inputs.iter().find(|i| i.id == *id))
                                .collect();
                            
                            let input_options: Vec<String> = linked_inputs.iter()
                                .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
                                .collect();
                            
                            let input_selection = Select::new("Select design input to remove:", input_options)
                                .prompt()?;
                            
                            let input_index = linked_inputs.iter()
                                .position(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)) == input_selection)
                                .unwrap();
                            let selected_input = linked_inputs[input_index];
                            
                            output.input_ids.retain(|&id| id != selected_input.id);
                            output.updated = Utc::now();
                            println!("{} Removed: {}", "✓".green(), selected_input.name);
                        },
                        "Done" => break,
                        _ => {}
                    }
                }
            },
            "📊 Edit approval status" => {
                let statuses = vec!["Draft", "Under Review", "Approved", "Released", "Obsolete"];
                let status_str = Select::new("Approval status:", statuses.clone())
                    .with_starting_cursor(statuses.iter().position(|&x| x == output.approval_status.as_str()).unwrap_or(0))
                    .prompt()?;
                output.approval_status = status_str.to_string();
                output.updated = Utc::now();
            },
            "🏷️  Edit version" => {
                let current_version = output.version.as_deref().unwrap_or("");
                let new_version = Text::new("Version:")
                    .with_help_message("Version number or identifier")
                    .with_default(current_version)
                    .prompt()?;
                output.version = if new_version.trim().is_empty() { 
                    None 
                } else { 
                    Some(new_version) 
                };
                output.updated = Utc::now();
            },
            "🗂️  Manage metadata" => {
                manage_metadata(&mut output.metadata, &mut output.updated)?;
            },
            "💾 Save changes" => {
                repo.update_design_output(output.clone())?;
                repo.save_to_directory(&requirements_dir)?;
                println!("{} Design output '{}' updated successfully!", "✓".green(), output.name);
                break;
            },
            "❌ Cancel" => {
                println!("Changes cancelled.");
                break;
            },
            _ => {}
        }
    }
    
    Ok(())
}

async fn add_verification_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Adding new verification".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    if !requirements_dir.exists() {
        std::fs::create_dir_all(&requirements_dir)?;
    }
    
    let repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let inputs = repo.get_design_inputs();
    
    if inputs.is_empty() {
        println!("{}", "No design inputs found. Please add design inputs first before creating verifications.".yellow());
        println!("Verifications must be linked to design inputs to ensure proper workflow.");
        return Ok(());
    }
    
    // Select design inputs to link to FIRST (allow multiple)
    println!("{}", "Select Design Inputs".bold());
    let input_options: Vec<String> = inputs.iter()
        .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
        .collect();
    
    let mut selected_input_ids = Vec::new();
    loop {
        let mut available_options = input_options.clone();
        available_options.push("✅ Done selecting inputs".to_string());
        
        if selected_input_ids.is_empty() {
            available_options.remove(available_options.len() - 1);
        }
        
        let input_selection = Select::new("Select design input (or Done when finished):", available_options.clone())
            .prompt()?;
        
        if input_selection == "✅ Done selecting inputs" {
            break;
        }
        
        let input_index = input_options.iter().position(|x| x == &input_selection).unwrap();
        let selected_input = &inputs[input_index];
        
        if !selected_input_ids.contains(&selected_input.id) {
            selected_input_ids.push(selected_input.id);
            println!("{} Added: {}", "✓".green(), selected_input.name);
        } else {
            println!("{} Already selected: {}", "ℹ".blue(), selected_input.name);
        }
        
        if selected_input_ids.len() >= inputs.len() {
            break;
        }
    }
    
    if selected_input_ids.is_empty() {
        println!("{} No design inputs selected. Operation cancelled.", "⚠".yellow());
        return Ok(());
    }
    
    println!("\nCreating verification for {} selected input(s)", selected_input_ids.len().to_string().green());
    
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
    
    let mut verification = Verification::new(name, description, selected_input_ids.clone(), verification_type, method);
    verification.add_acceptance_criterion(acceptance_criteria);
    
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    repo.add_verification(verification.clone())?;
    repo.save_to_directory(&requirements_dir)?;
    
    println!("{} Verification '{}' added successfully!", "✓".green(), verification.name);
    println!("ID: {}", verification.id);
    println!("Linked to {} design input(s)", selected_input_ids.len());
    
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

async fn edit_verification_interactive(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit Verification".bold().blue());
    
    let requirements_dir = project_ctx.module_path("requirements");
    let mut repo = RequirementsRepository::load_from_directory(&requirements_dir)?;
    let verifications = repo.get_verifications();
    
    if verifications.is_empty() {
        println!("{}", "No verifications found. Add verifications first.".yellow());
        return Ok(());
    }
    
    // Select verification to edit
    let verification_options: Vec<String> = verifications.iter()
        .map(|v| format!("{} - {}", v.name, truncate_string(&v.description, 50)))
        .collect();
    
    let verification_selection = Select::new("Select verification to edit:", verification_options.clone())
        .prompt()?;
    
    let verification_index = verification_options.iter().position(|x| x == &verification_selection).unwrap();
    let mut verification = verifications[verification_index].clone();
    
    loop {
        println!("\n{}", format!("Editing: {}", verification.name).bold());
        
        let edit_options = vec![
            "📝 Edit name",
            "📄 Edit description",
            "🏷️  Edit verification type",
            "🔧 Edit method",
            "🔗 Manage linked design inputs",
            "✅ Manage acceptance criteria",
            "📊 Edit status",
            "📝 Edit results",
            "📁 Manage evidence",
            "🗂️  Manage metadata",
            "💾 Save changes",
            "❌ Cancel",
        ];
        
        let choice = Select::new("What would you like to edit?", edit_options)
            .prompt()?;
        
        match choice {
            "📝 Edit name" => {
                let new_name = Text::new("New name:")
                    .with_default(&verification.name)
                    .prompt()?;
                verification.name = new_name;
                verification.updated = Utc::now();
            },
            "📄 Edit description" => {
                let new_description = Text::new("New description:")
                    .with_default(&verification.description)
                    .prompt()?;
                verification.description = new_description;
                verification.updated = Utc::now();
            },
            "🏷️  Edit verification type" => {
                let new_verification_type = Text::new("New verification type:")
                    .with_help_message("e.g., Test, Review, Inspection, Analysis, etc.")
                    .with_default(&verification.verification_type)
                    .prompt()?;
                verification.verification_type = new_verification_type;
                verification.updated = Utc::now();
            },
            "🔧 Edit method" => {
                let new_method = Text::new("New verification method:")
                    .with_help_message("How will this verification be performed?")
                    .with_default(&verification.method)
                    .prompt()?;
                verification.method = new_method;
                verification.updated = Utc::now();
            },
            "🔗 Manage linked design inputs" => {
                let inputs = repo.get_design_inputs();
                if inputs.is_empty() {
                    println!("{} No design inputs available to link to", "⚠".yellow());
                    continue;
                }
                
                // Manage multiple linked design inputs
                loop {
                    println!("\nCurrently linked to {} design input(s)", verification.input_ids.len());
                    
                    let action_options = vec!["Add design input", "Remove design input", "Done"];
                    
                    let action = Select::new("What would you like to do?", action_options)
                        .prompt()?;
                    
                    match action {
                        "Add design input" => {
                            let available_inputs: Vec<_> = inputs.iter()
                                .filter(|i| !verification.input_ids.contains(&i.id))
                                .collect();
                            
                            if available_inputs.is_empty() {
                                println!("{} All inputs are already linked", "ℹ".blue());
                                continue;
                            }
                            
                            let input_options: Vec<String> = available_inputs.iter()
                                .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
                                .collect();
                            
                            let input_selection = Select::new("Select design input to add:", input_options)
                                .prompt()?;
                            
                            let input_index = available_inputs.iter()
                                .position(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)) == input_selection)
                                .unwrap();
                            let selected_input = available_inputs[input_index];
                            
                            verification.input_ids.push(selected_input.id);
                            verification.updated = Utc::now();
                            println!("{} Added: {}", "✓".green(), selected_input.name);
                        },
                        "Remove design input" => {
                            if verification.input_ids.is_empty() {
                                println!("{} No inputs linked", "ℹ".blue());
                                continue;
                            }
                            
                            let linked_inputs: Vec<_> = verification.input_ids.iter()
                                .filter_map(|id| inputs.iter().find(|i| i.id == *id))
                                .collect();
                            
                            let input_options: Vec<String> = linked_inputs.iter()
                                .map(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)))
                                .collect();
                            
                            let input_selection = Select::new("Select design input to remove:", input_options)
                                .prompt()?;
                            
                            let input_index = linked_inputs.iter()
                                .position(|i| format!("{} - {}", i.name, truncate_string(&i.description, 50)) == input_selection)
                                .unwrap();
                            let selected_input = linked_inputs[input_index];
                            
                            verification.input_ids.retain(|&id| id != selected_input.id);
                            verification.updated = Utc::now();
                            println!("{} Removed: {}", "✓".green(), selected_input.name);
                        },
                        "Done" => break,
                        _ => {}
                    }
                }
            },
            "✅ Manage acceptance criteria" => {
                manage_list_items(&mut verification.acceptance_criteria, "acceptance criteria", &mut verification.updated)?;
            },
            "📊 Edit status" => {
                let statuses = vec!["Planned", "In Progress", "Passed", "Failed", "Blocked", "Cancelled"];
                let status_str = Select::new("Status:", statuses.clone())
                    .with_starting_cursor(statuses.iter().position(|&x| x == verification.status.as_str()).unwrap_or(0))
                    .prompt()?;
                verification.status = status_str.to_string();
                verification.updated = Utc::now();
            },
            "📝 Edit results" => {
                let current_results = verification.results.as_deref().unwrap_or("");
                let new_results = Text::new("Results:")
                    .with_help_message("Verification test results or outcome")
                    .with_default(current_results)
                    .prompt()?;
                verification.results = if new_results.trim().is_empty() { 
                    None 
                } else { 
                    Some(new_results) 
                };
                verification.updated = Utc::now();
            },
            "📁 Manage evidence" => {
                manage_list_items(&mut verification.evidence, "evidence files", &mut verification.updated)?;
            },
            "🗂️  Manage metadata" => {
                manage_metadata(&mut verification.metadata, &mut verification.updated)?;
            },
            "💾 Save changes" => {
                repo.update_verification(verification.clone())?;
                repo.save_to_directory(&requirements_dir)?;
                println!("{} Verification '{}' updated successfully!", "✓".green(), verification.name);
                break;
            },
            "❌ Cancel" => {
                println!("Changes cancelled.");
                break;
            },
            _ => {}
        }
    }
    
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

// Helper functions for editing

fn manage_tags(tags: &mut Vec<String>, updated: &mut DateTime<Utc>) -> Result<()> {
    loop {
        println!("\n{}", "Current tags:".bold());
        if tags.is_empty() {
            println!("  No tags");
        } else {
            for (i, tag) in tags.iter().enumerate() {
                println!("  {}: {}", i + 1, tag);
            }
        }
        
        let tag_options = vec![
            "➕ Add tag",
            "🗑️  Remove tag",
            "← Back",
        ];
        
        let choice = Select::new("Tag management:", tag_options).prompt()?;
        
        match choice {
            "➕ Add tag" => {
                let new_tag = Text::new("New tag:").prompt()?;
                if !new_tag.trim().is_empty() && !tags.contains(&new_tag) {
                    tags.push(new_tag);
                    *updated = Utc::now();
                    println!("{} Tag added!", "✓".green());
                }
            },
            "🗑️  Remove tag" => {
                if tags.is_empty() {
                    println!("{} No tags to remove", "ℹ".blue());
                    continue;
                }
                
                let remove_selection = Select::new("Select tag to remove:", tags.clone()).prompt()?;
                tags.retain(|tag| tag != &remove_selection);
                *updated = Utc::now();
                println!("{} Tag '{}' removed!", "✓".green(), remove_selection);
            },
            "← Back" => break,
            _ => {}
        }
    }
    Ok(())
}

fn manage_metadata(metadata: &mut std::collections::HashMap<String, String>, updated: &mut DateTime<Utc>) -> Result<()> {
    loop {
        println!("\n{}", "Current metadata:".bold());
        if metadata.is_empty() {
            println!("  No metadata");
        } else {
            for (key, value) in metadata.iter() {
                println!("  {}: {}", key, truncate_string(value, 40));
            }
        }
        
        let metadata_options = vec![
            "➕ Add metadata",
            "✏️  Edit metadata", 
            "🗑️  Remove metadata",
            "← Back",
        ];
        
        let choice = Select::new("Metadata management:", metadata_options).prompt()?;
        
        match choice {
            "➕ Add metadata" => {
                let key = Text::new("Metadata key:").prompt()?;
                if !key.trim().is_empty() {
                    let value = Text::new("Metadata value:").prompt()?;
                    metadata.insert(key.clone(), value);
                    *updated = Utc::now();
                    println!("{} Metadata '{}' added!", "✓".green(), key);
                }
            },
            "✏️  Edit metadata" => {
                if metadata.is_empty() {
                    println!("{} No metadata to edit", "ℹ".blue());
                    continue;
                }
                
                let keys: Vec<String> = metadata.keys().cloned().collect();
                let key_selection = Select::new("Select metadata to edit:", keys).prompt()?;
                
                let current_value = metadata.get(&key_selection).unwrap();
                let new_value = Text::new("New value:")
                    .with_default(current_value)
                    .prompt()?;
                    
                metadata.insert(key_selection.clone(), new_value);
                *updated = Utc::now();
                println!("{} Metadata '{}' updated!", "✓".green(), key_selection);
            },
            "🗑️  Remove metadata" => {
                if metadata.is_empty() {
                    println!("{} No metadata to remove", "ℹ".blue());
                    continue;
                }
                
                let keys: Vec<String> = metadata.keys().cloned().collect();
                let key_selection = Select::new("Select metadata to remove:", keys).prompt()?;
                
                metadata.remove(&key_selection);
                *updated = Utc::now();
                println!("{} Metadata '{}' removed!", "✓".green(), key_selection);
            },
            "← Back" => break,
            _ => {}
        }
    }
    Ok(())
}

fn manage_list_items(items: &mut Vec<String>, item_type: &str, updated: &mut DateTime<Utc>) -> Result<()> {
    loop {
        println!("\n{}", format!("Current {}:", item_type).bold());
        if items.is_empty() {
            println!("  No {}", item_type);
        } else {
            for (i, item) in items.iter().enumerate() {
                println!("  {}: {}", i + 1, item);
            }
        }
        
        let list_options = vec![
            format!("➕ Add {}", item_type),
            format!("✏️  Edit {}", item_type),
            format!("🗑️  Remove {}", item_type),
            "← Back".to_string(),
        ];
        
        let choice = Select::new(&format!("{} management:", item_type), list_options).prompt()?;
        
        if choice.starts_with("➕") {
            let new_item = Text::new(&format!("New {}:", item_type)).prompt()?;
            if !new_item.trim().is_empty() {
                items.push(new_item);
                *updated = Utc::now();
                println!("{} {} added!", "✓".green(), item_type.trim_end_matches('s'));
            }
        } else if choice.starts_with("✏️") {
            if items.is_empty() {
                println!("{} No {} to edit", "ℹ".blue(), item_type);
                continue;
            }
            
            let item_selection = Select::new(&format!("Select {} to edit:", item_type), items.clone()).prompt()?;
            let item_index = items.iter().position(|x| x == &item_selection).unwrap();
            
            let new_value = Text::new(&format!("Edit {}:", item_type))
                .with_default(&items[item_index])
                .prompt()?;
                
            items[item_index] = new_value;
            *updated = Utc::now();
            println!("{} {} updated!", "✓".green(), item_type.trim_end_matches('s'));
        } else if choice.starts_with("🗑️") {
            if items.is_empty() {
                println!("{} No {} to remove", "ℹ".blue(), item_type);
                continue;
            }
            
            let remove_selection = Select::new(&format!("Select {} to remove:", item_type), items.clone()).prompt()?;
            items.retain(|item| item != &remove_selection);
            *updated = Utc::now();
            println!("{} {} removed!", "✓".green(), item_type.trim_end_matches('s'));
        } else if choice == "← Back" {
            break;
        }
    }
    Ok(())
}