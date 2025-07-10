use crate::LinkCommands;
use colored::Colorize;
use comfy_table::Table;
use tessera_core::{ProjectContext, Result, CrossModuleLink, LinkType, Id};
use inquire::{Select, Text};

pub async fn execute_link_command(command: LinkCommands, mut project_ctx: ProjectContext) -> Result<()> {
    match command {
        LinkCommands::Add => add_link_interactive(&mut project_ctx).await,
        LinkCommands::List => list_all_links(&project_ctx),
        LinkCommands::Show => show_entity_links(&project_ctx).await,
        LinkCommands::Remove => remove_link_interactive(&mut project_ctx).await,
        LinkCommands::Validate => validate_all_links(&project_ctx),
    }
}

async fn add_link_interactive(project_ctx: &mut ProjectContext) -> Result<()> {
    println!("{}", "Adding cross-module link".bold().blue());
    
    let modules = vec!["quality", "pm", "tol"];
    
    let source_module = Select::new("Source module:", modules.clone()).prompt()?;
    
    // For simplicity, we'll ask for entity ID directly
    // In a real implementation, we'd list entities from the selected module
    let source_entity_id_str = Text::new("Source entity ID:")
        .with_help_message("Enter the UUID of the source entity")
        .prompt()?;
    let source_entity_id = Id::parse(&source_entity_id_str)
        .map_err(|e| tessera_core::DesignTrackError::Validation(format!("Invalid UUID: {}", e)))?;
    
    let target_module = Select::new("Target module:", modules).prompt()?;
    
    let target_entity_id_str = Text::new("Target entity ID:")
        .with_help_message("Enter the UUID of the target entity")
        .prompt()?;
    let target_entity_id = Id::parse(&target_entity_id_str)
        .map_err(|e| tessera_core::DesignTrackError::Validation(format!("Invalid UUID: {}", e)))?;
    
    let link_types = vec![
        "RequirementToComponent",
        "RequirementToFeature", 
        "RequirementToStackup",
        "OutputToComponent",
        "OutputToFeature",
        "RiskToStackup",
        "RequirementToTask",
        "OutputToTask",
        "RiskToTask",
        "TaskToComponent",
        "TaskToFeature",
        "TaskToStackup",
        "Reference",
        "Verification",
        "Compliance",
        "Mitigation",
        "Other",
    ];
    
    let link_type_str = Select::new("Link type:", link_types).prompt()?;
    let link_type = match link_type_str {
        "RequirementToComponent" => LinkType::RequirementToComponent,
        "RequirementToFeature" => LinkType::RequirementToFeature,
        "RequirementToStackup" => LinkType::RequirementToStackup,
        "OutputToComponent" => LinkType::OutputToComponent,
        "OutputToFeature" => LinkType::OutputToFeature,
        "RiskToStackup" => LinkType::RiskToStackup,
        "RequirementToTask" => LinkType::RequirementToTask,
        "OutputToTask" => LinkType::OutputToTask,
        "RiskToTask" => LinkType::RiskToTask,
        "TaskToComponent" => LinkType::TaskToComponent,
        "TaskToFeature" => LinkType::TaskToFeature,
        "TaskToStackup" => LinkType::TaskToStackup,
        "Reference" => LinkType::Reference,
        "Verification" => LinkType::Verification,
        "Compliance" => LinkType::Compliance,
        "Mitigation" => LinkType::Mitigation,
        _ => {
            let other_name = Text::new("Other link type:").prompt()?;
            LinkType::Other(other_name)
        }
    };
    
    let description = Text::new("Description (optional):")
        .prompt()?;
    
    let mut link = CrossModuleLink::new(
        source_module.to_string(),
        source_entity_id,
        target_module.to_string(),
        target_entity_id,
        link_type,
    );
    
    if !description.is_empty() {
        link = link.with_description(description);
    }
    
    project_ctx.add_link(link.clone())?;
    
    println!("{} Link added successfully!", "✓".green());
    println!("Link ID: {}", link.id);
    println!("From: {} entity {}", link.source_module, link.source_entity_id);
    println!("To: {} entity {}", link.target_module, link.target_entity_id);
    
    Ok(())
}

fn list_all_links(project_ctx: &ProjectContext) -> Result<()> {
    let links = project_ctx.link_registry.get_all_links();
    
    if links.is_empty() {
        println!("{}", "No cross-module links found".yellow());
        return Ok(());
    }
    
    println!("{}", "Cross-Module Links".bold().blue());
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "From", "To", "Type", "Description"]);
    
    for link in links {
        let from = format!("{}:{}", link.source_module, link.source_entity_id);
        let to = format!("{}:{}", link.target_module, link.target_entity_id);
        let link_type = format!("{:?}", link.link_type);
        let description = link.description.as_deref().unwrap_or("-");
        
        table.add_row(vec![
            link.id.to_string(),
            from,
            to,
            link_type,
            description.to_string(),
        ]);
    }
    
    println!("{}", table);
    println!("Total links: {}", links.len());
    
    Ok(())
}

async fn show_entity_links(project_ctx: &ProjectContext) -> Result<()> {
    let modules = vec!["quality", "pm", "tol"];
    let module = Select::new("Module:", modules).prompt()?;
    
    let entity_id_str = Text::new("Entity ID:")
        .with_help_message("Enter the UUID of the entity")
        .prompt()?;
    let entity_id = Id::parse(&entity_id_str)
        .map_err(|e| tessera_core::DesignTrackError::Validation(format!("Invalid UUID: {}", e)))?;
    
    let links_from = project_ctx.get_links_from(module, entity_id);
    let links_to = project_ctx.get_links_to(module, entity_id);
    
    println!("{}", format!("Links for {} entity {}", module, entity_id).bold().blue());
    
    let has_outgoing = !links_from.is_empty();
    let has_incoming = !links_to.is_empty();
    
    if has_outgoing {
        println!("\n{}", "Outgoing Links:".bold());
        for link in &links_from {
            println!("  → {} {} ({:?})", 
                     link.target_module, 
                     link.target_entity_id, 
                     link.link_type);
            if let Some(ref desc) = link.description {
                println!("    {}", desc);
            }
        }
    }
    
    if has_incoming {
        println!("\n{}", "Incoming Links:".bold());
        for link in &links_to {
            println!("  ← {} {} ({:?})", 
                     link.source_module, 
                     link.source_entity_id, 
                     link.link_type);
            if let Some(ref desc) = link.description {
                println!("    {}", desc);
            }
        }
    }
    
    if !has_outgoing && !has_incoming {
        println!("{}", "No links found for this entity".yellow());
    }
    
    Ok(())
}

async fn remove_link_interactive(project_ctx: &mut ProjectContext) -> Result<()> {
    let links = project_ctx.link_registry.get_all_links();
    
    if links.is_empty() {
        println!("{}", "No links to remove".yellow());
        return Ok(());
    }
    
    let link_options: Vec<String> = links.iter()
        .map(|link| format!("{} → {} ({})", 
                          format!("{}:{}", link.source_module, link.source_entity_id),
                          format!("{}:{}", link.target_module, link.target_entity_id),
                          format!("{:?}", link.link_type)))
        .collect();
    
    let selection = Select::new("Select link to remove:", link_options.clone()).prompt()?;
    let link_index = link_options.iter().position(|x| x == &selection).unwrap();
    let selected_link = &links[link_index];
    
    let link_id = selected_link.id;
    project_ctx.link_registry.remove_link(link_id)?;
    project_ctx.save_links()?;
    
    println!("{} Link removed successfully!", "✓".green());
    
    Ok(())
}

fn validate_all_links(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Validating cross-module links...".blue());
    
    let validator = tessera_core::BasicLinkValidator::new(vec![
        "quality".to_string(),
        "pm".to_string(),
        "tol".to_string(),
    ]);
    
    let errors = project_ctx.link_registry.validate_links(&validator)?;
    
    if errors.is_empty() {
        println!("{} All links are valid!", "✓".green());
    } else {
        println!("{} Found {} validation errors:", "⚠".yellow(), errors.len());
        for error in errors {
            println!("  {}", error);
        }
    }
    
    Ok(())
}