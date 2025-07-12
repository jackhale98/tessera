use tessera_core::{ProjectContext, Result, Id};
use tessera_impact::{
    ImpactAnalyzer, LifecycleManager, GitWorkflowOrchestrator, ConfigurationManager,
    EntityReference, ModuleType, EntityState, ApprovalDecision, ImpactSeverity,
    ImpactEngine, ChangeType
};
use colored::Colorize;
use comfy_table::{Table, Cell, Attribute, Color};
use inquire::{Text, Select, Confirm};
use chrono::Utc;

use crate::ImpactCommands;

pub async fn execute_impact_command(command: ImpactCommands, project_ctx: ProjectContext) -> Result<()> {
    match command {
        ImpactCommands::ListAnalyses => {
            list_impact_analyses(&project_ctx).await?;
        },
        
        ImpactCommands::ShowAnalysis { analysis_id } => {
            show_impact_analysis(&project_ctx, analysis_id).await?;
        },
        
        ImpactCommands::ShowEvents => {
            show_change_events(&project_ctx).await?;
        },
        
        ImpactCommands::ShowEntityImpacts { entity_id } => {
            show_entity_impacts(&project_ctx, entity_id).await?;
        },
        
        ImpactCommands::ListApprovals => {
            list_pending_approvals(&project_ctx).await?;
        },
        
        ImpactCommands::ProcessApproval { workflow_id, decision, comments } => {
            process_approval(&project_ctx, workflow_id, decision, comments).await?;
        },
        
        ImpactCommands::ListWorkflows => {
            list_git_workflows(&project_ctx).await?;
        },
        
        ImpactCommands::ShowWorkflow { workflow_id } => {
            show_git_workflow(&project_ctx, workflow_id).await?;
        },
        
        ImpactCommands::Configure => {
            configure_impact_analysis(&project_ctx).await?;
        },
        
        ImpactCommands::Dashboard => {
            show_impact_dashboard(&project_ctx).await?;
        },
    }
    
    Ok(())
}

async fn show_change_events(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Recent Change Events".bold().blue());
    
    let engine = ImpactEngine::new();
    let events = engine.get_change_events(project_ctx)?;
    
    if events.is_empty() {
        println!("No change events found.");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Timestamp").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Entity").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Change Type").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Description").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    for event in events.iter().take(20) { // Show last 20 events
        let change_type_cell = match event.change_type {
            ChangeType::Created => Cell::new("Created").fg(Color::Green),
            ChangeType::Updated => Cell::new("Updated").fg(Color::Yellow),
            ChangeType::Deleted => Cell::new("Deleted").fg(Color::Red),
            ChangeType::StateTransition => Cell::new("State Change").fg(Color::Blue),
            ChangeType::LinkAdded => Cell::new("Link Added").fg(Color::Cyan),
            ChangeType::LinkRemoved => Cell::new("Link Removed").fg(Color::Magenta),
        };
        
        table.add_row(vec![
            Cell::new(event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::new(format!("{} {}", event.entity_reference.module, event.entity_reference.name)),
            change_type_cell,
            Cell::new(&event.description),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn list_impact_analyses(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Automatically Generated Impact Analyses".bold().blue());
    
    let engine = ImpactEngine::new();
    let analyses = engine.get_saved_analyses(project_ctx)?;
    
    if analyses.is_empty() {
        println!("No impact analyses found.");
        println!("Impact analyses are automatically generated when entities are created, updated, or deleted.");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Source Entity").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Severity").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Affected").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Effort (hrs)").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Date").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    for analysis in analyses.iter().take(20) { // Show most recent 20
        let severity_cell = match analysis.max_severity {
            ImpactSeverity::Critical => Cell::new("Critical").fg(Color::Red),
            ImpactSeverity::High => Cell::new("High").fg(Color::Yellow),
            ImpactSeverity::Medium => Cell::new("Medium").fg(Color::Blue),
            ImpactSeverity::Low => Cell::new("Low").fg(Color::Green),
        };
        
        table.add_row(vec![
            Cell::new(analysis.id.to_string()),
            Cell::new(format!("{} {}", analysis.source_entity.module, analysis.source_entity.name)),
            severity_cell,
            Cell::new(analysis.total_affected_entities.to_string()),
            Cell::new(format!("{:.1}", analysis.estimated_total_effort_hours)),
            Cell::new(analysis.analysis_timestamp.format("%Y-%m-%d %H:%M").to_string()),
        ]);
    }
    
    println!("{}", table);
    
    if analyses.len() > 20 {
        println!("\nShowing 20 most recent analyses. Total: {}", analyses.len());
    }
    
    Ok(())
}

async fn show_impact_analysis(project_ctx: &ProjectContext, analysis_id: String) -> Result<()> {
    let analysis_id = Id::parse(&analysis_id)
        .map_err(|_| tessera_core::DesignTrackError::Validation("Invalid analysis ID format".to_string()))?;
    
    let engine = ImpactEngine::new();
    let analyses = engine.get_saved_analyses(project_ctx)?;
    
    let analysis = analyses.iter()
        .find(|a| a.id == analysis_id)
        .ok_or_else(|| tessera_core::DesignTrackError::Validation(
            "Impact analysis not found".to_string()
        ))?;
    
    display_impact_analysis(analysis);
    Ok(())
}

async fn show_entity_impacts(project_ctx: &ProjectContext, entity_id: String) -> Result<()> {
    println!("{}", "Entity Impact Analysis".bold().blue());
    
    let entity_id = Id::parse(&entity_id)
        .map_err(|_| tessera_core::DesignTrackError::Validation("Invalid entity ID format".to_string()))?;
    
    let engine = ImpactEngine::new();
    let analyses = engine.get_saved_analyses(project_ctx)?;
    
    // Filter analyses that involve this entity (either as source or target)
    let relevant_analyses: Vec<_> = analyses.iter()
        .filter(|analysis| {
            analysis.source_entity.id == entity_id ||
            analysis.impacts.iter().any(|impact| impact.target_entity.id == entity_id)
        })
        .collect();
    
    if relevant_analyses.is_empty() {
        println!("No impact analyses found for entity {}", entity_id);
        return Ok(());
    }
    
    println!("Found {} impact analyses involving this entity:\n", relevant_analyses.len());
    
    for analysis in relevant_analyses {
        println!("📊 {} Analysis: {} ({})", 
            "Impact".bold(),
            analysis.id,
            analysis.analysis_timestamp.format("%Y-%m-%d %H:%M")
        );
        println!("   Source: {} {}", analysis.source_entity.module, analysis.source_entity.name);
        println!("   Change: {}", analysis.change_description);
        println!("   Max Severity: {:?}", analysis.max_severity);
        println!("   Affected Entities: {}", analysis.total_affected_entities);
        println!("   Estimated Effort: {:.1} hours\n", analysis.estimated_total_effort_hours);
    }
    
    Ok(())
}

async fn list_pending_approvals(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Pending Approvals".bold().blue());
    
    let lifecycle_manager = LifecycleManager::load_from_project(project_ctx)?;
    let pending_approvals = lifecycle_manager.get_pending_approvals();
    
    if pending_approvals.is_empty() {
        println!("No pending approvals.");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Workflow ID").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Entity").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Transition").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Required Level").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Created").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    for approval in pending_approvals {
        table.add_row(vec![
            Cell::new(approval.id.to_string()),
            Cell::new(approval.entity_id.to_string()),
            Cell::new(format!("{:?} → {:?}", 
                approval.requested_transition.from_state,
                approval.requested_transition.to_state
            )),
            Cell::new(format!("{:?}", approval.required_approval_level)),
            Cell::new(format!("{:?}", approval.status)),
            Cell::new(approval.created.format("%Y-%m-%d %H:%M").to_string()),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn process_approval(
    project_ctx: &ProjectContext,
    workflow_id: String,
    decision: String,
    comments: Option<String>
) -> Result<()> {
    println!("{}", "Process Approval".bold().blue());
    
    // Parse workflow ID and decision
    let workflow_id = Id::parse(&workflow_id)
        .map_err(|_| tessera_core::DesignTrackError::Validation("Invalid workflow ID format".to_string()))?;
    
    let approval_decision = match decision.to_lowercase().as_str() {
        "approved" | "approve" => ApprovalDecision::Approved,
        "rejected" | "reject" => ApprovalDecision::Rejected,
        "changes" | "request-changes" => ApprovalDecision::RequestChanges,
        _ => return Err(tessera_core::DesignTrackError::Validation(
            "Invalid decision. Use: approved, rejected, or changes".to_string()
        )),
    };
    
    // Load lifecycle manager
    let mut lifecycle_manager = LifecycleManager::load_from_project(project_ctx)?;
    
    // Process approval (simulate approver ID)
    let approver_id = Id::new(); // In real implementation, get from current user
    lifecycle_manager.process_approval(
        workflow_id,
        approver_id,
        approval_decision,
        comments,
        project_ctx
    ).await?;
    
    // Save changes
    lifecycle_manager.save_to_project(project_ctx)?;
    
    println!("{} Approval processed", "✓".green());
    Ok(())
}

async fn list_git_workflows(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Git Workflows".bold().blue());
    
    let config = ConfigurationManager::load_from_project(project_ctx)?;
    let mut orchestrator = GitWorkflowOrchestrator::new(project_ctx, config)?;
    orchestrator.load_workflows(project_ctx)?;
    
    let workflows = orchestrator.get_active_workflows();
    
    if workflows.is_empty() {
        println!("No active Git workflows.");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Workflow ID").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Branch").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("PR URL").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Created").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    for workflow in workflows {
        table.add_row(vec![
            Cell::new(workflow.id.to_string()),
            Cell::new(&workflow.branch_name),
            Cell::new(format!("{:?}", workflow.status)),
            Cell::new(workflow.pull_request_url.as_deref().unwrap_or("-")),
            Cell::new(workflow.created.format("%Y-%m-%d %H:%M").to_string()),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn show_git_workflow(project_ctx: &ProjectContext, workflow_id: String) -> Result<()> {
    let workflow_id = Id::parse(&workflow_id)
        .map_err(|_| tessera_core::DesignTrackError::Validation("Invalid workflow ID format".to_string()))?;
    
    let config = ConfigurationManager::load_from_project(project_ctx)?;
    let mut orchestrator = GitWorkflowOrchestrator::new(project_ctx, config)?;
    orchestrator.load_workflows(project_ctx)?;
    
    if let Some(workflow) = orchestrator.get_workflow(&workflow_id) {
        println!("{}", "Git Workflow Details".bold().blue());
        println!("ID: {}", workflow.id);
        println!("Branch: {}", workflow.branch_name);
        println!("Status: {:?}", workflow.status);
        println!("Created: {}", workflow.created.format("%Y-%m-%d %H:%M:%S"));
        
        if let Some(pr_url) = &workflow.pull_request_url {
            println!("Pull Request: {}", pr_url);
        }
        
        if !workflow.commits.is_empty() {
            println!("\nCommits:");
            for commit in &workflow.commits {
                println!("  {} - {} ({})", 
                    &commit.commit_hash[..8], 
                    commit.message.lines().next().unwrap_or(""),
                    commit.author
                );
            }
        }
    } else {
        println!("Workflow not found");
    }
    
    Ok(())
}

async fn configure_impact_analysis(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Configure Impact Analysis".bold().blue());
    println!("Impact analysis configuration is stored in RON files.");
    println!("Configuration files are located at:");
    println!("  {}/impact/config/", project_ctx.root_path.display());
    println!("");
    println!("Configuration files:");
    println!("  - impact_config.ron    - Impact analysis rules");
    println!("  - workflow_config.ron  - Git workflow settings");
    println!("  - team_config.ron      - Team-specific settings");
    
    // Create default configuration if it doesn't exist
    let config = ConfigurationManager::load_from_project(project_ctx)?;
    config.save_to_project(project_ctx)?;
    
    println!("\n{} Configuration files created/updated", "✓".green());
    Ok(())
}

async fn show_impact_dashboard(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Automatic Impact Analysis Dashboard".bold().blue());
    println!();
    
    let engine = ImpactEngine::new();
    let analyses = engine.get_saved_analyses(project_ctx)?;
    let events = engine.get_change_events(project_ctx)?;
    
    // Statistics
    println!("{}", "📊 Statistics:".bold());
    println!("  Total Impact Analyses: {}", analyses.len());
    println!("  Total Change Events: {}", events.len());
    
    if !analyses.is_empty() {
        let recent_analyses = analyses.iter().take(5).count();
        println!("  Recent Analyses (last 5): {}", recent_analyses);
        
        // Severity distribution
        let critical_count = analyses.iter().filter(|a| a.max_severity == ImpactSeverity::Critical).count();
        let high_count = analyses.iter().filter(|a| a.max_severity == ImpactSeverity::High).count();
        let medium_count = analyses.iter().filter(|a| a.max_severity == ImpactSeverity::Medium).count();
        let low_count = analyses.iter().filter(|a| a.max_severity == ImpactSeverity::Low).count();
        
        println!();
        println!("{}", "🎯 Severity Distribution:".bold());
        println!("  🔴 Critical: {}", critical_count);
        println!("  🟡 High: {}", high_count);
        println!("  🔵 Medium: {}", medium_count);
        println!("  🟢 Low: {}", low_count);
    }
    
    if !events.is_empty() {
        println!();
        println!("{}", "📅 Recent Activity:".bold());
        for event in events.iter().take(5) {
            let change_icon = match event.change_type {
                ChangeType::Created => "➕",
                ChangeType::Updated => "✏️",
                ChangeType::Deleted => "🗑️",
                ChangeType::StateTransition => "🔄",
                ChangeType::LinkAdded => "🔗",
                ChangeType::LinkRemoved => "💔",
            };
            println!("  {} {} - {} {} ({})", 
                change_icon,
                event.timestamp.format("%m/%d %H:%M"),
                event.entity_reference.module,
                event.entity_reference.name,
                format!("{:?}", event.change_type)
            );
        }
    }
    
    // Check for lifecycle manager data
    if let Ok(lifecycle_manager) = LifecycleManager::load_from_project(project_ctx) {
        let pending_count = lifecycle_manager.get_pending_approvals().len();
        if pending_count > 0 {
            println!();
            println!("{}", "⚠️  Pending Actions:".bold());
            println!("  Pending Approvals: {}", pending_count);
        }
    }
    
    println!();
    println!("{}", "📋 Available Commands:".bold());
    println!("  tessera impact list            - View generated impact analyses");
    println!("  tessera impact events          - View recent change events");
    println!("  tessera impact entity <id>     - View impacts for specific entity");
    println!("  tessera impact approvals       - List pending approvals");
    println!("  tessera impact workflows       - List Git workflows");
    println!("  tessera impact config          - Configure automatic analysis");
    
    println!();
    println!("{}", "💡 How it works:".bold());
    println!("  Impact analysis runs automatically when you create, update, or delete entities.");
    println!("  Use these commands to view the results and manage approval workflows.");
    
    Ok(())
}

// Helper functions
fn display_impact_analysis(analysis: &tessera_impact::ImpactAnalysis) {
    println!();
    println!("{}", "Impact Analysis Results".bold().green());
    println!("Analysis ID: {}", analysis.id);
    println!("Source Entity: {} {}", analysis.source_entity.module, analysis.source_entity.name);
    println!("Change: {}", analysis.change_description);
    println!("Timestamp: {}", analysis.analysis_timestamp.format("%Y-%m-%d %H:%M:%S"));
    println!();
    
    // Summary
    let severity_color = match analysis.max_severity {
        ImpactSeverity::Critical => "red",
        ImpactSeverity::High => "yellow", 
        ImpactSeverity::Medium => "blue",
        ImpactSeverity::Low => "green",
    };
    
    println!("📊 {} Summary:", "Impact".bold());
    println!("  Maximum Severity: {}", format!("{:?}", analysis.max_severity).color(severity_color));
    println!("  Total Affected Entities: {}", analysis.total_affected_entities);
    println!("  Estimated Effort: {:.1} hours", analysis.estimated_total_effort_hours);
    println!("  Approval Required: {}", if analysis.approval_required { "Yes" } else { "No" });
    
    if let Some(level) = analysis.required_approval_level {
        println!("  Required Approval Level: {:?}", level);
    }
    
    // Impacts by module
    if !analysis.impacts.is_empty() {
        println!();
        println!("🎯 {} Affected Entities:", "Detailed".bold());
        
        let impacts_by_module = analysis.impacts_by_module();
        for (module, impacts) in impacts_by_module {
            println!("  {}", format!("{:?}", module).bold());
            for impact in impacts {
                let severity_icon = match impact.severity {
                    ImpactSeverity::Critical => "🔴",
                    ImpactSeverity::High => "🟡",
                    ImpactSeverity::Medium => "🔵",
                    ImpactSeverity::Low => "🟢",
                };
                println!("    {} {} - {} ({:?})",
                    severity_icon,
                    impact.target_entity.name,
                    impact.description,
                    impact.severity
                );
            }
        }
    }
}

async fn create_approval_workflow(
    project_ctx: &ProjectContext,
    analysis: &tessera_impact::ImpactAnalysis
) -> Result<()> {
    println!("{}", "Creating Approval Workflow".bold().blue());
    
    // Load managers
    let mut lifecycle_manager = LifecycleManager::load_from_project(project_ctx)?;
    let config = ConfigurationManager::load_from_project(project_ctx)?;
    let mut git_orchestrator = GitWorkflowOrchestrator::new(project_ctx, config)?;
    
    // Create approval workflow (simplified for demo)
    let workflow_id = Id::new();
    
    // Create Git workflow
    let git_workflow_id = git_orchestrator.create_workflow(
        analysis,
        // Need to create a dummy approval workflow for the Git orchestrator
        &tessera_impact::ApprovalWorkflow {
            id: workflow_id,
            entity_id: analysis.source_entity.id,
            requested_transition: tessera_impact::StateTransition {
                from_state: EntityState::Draft,
                to_state: EntityState::InReview,
                timestamp: Utc::now(),
                approver_id: None,
                approval_workflow_id: Some(workflow_id),
                reason: "Impact analysis approval".to_string(),
                impact_analysis_id: Some(analysis.id),
            },
            required_approval_level: analysis.required_approval_level.unwrap_or(tessera_impact::ApprovalLevel::TeamLead),
            status: tessera_impact::ApprovalStatus::Pending,
            approvals: Vec::new(),
            impact_analysis: Some(analysis.clone()),
            created: Utc::now(),
            updated: Utc::now(),
            deadline: Some(Utc::now() + chrono::Duration::days(7)),
        },
        project_ctx
    ).await?;
    
    // Save workflows
    git_orchestrator.save_workflows(project_ctx)?;
    lifecycle_manager.save_to_project(project_ctx)?;
    
    println!("{} Approval workflow created", "✓".green());
    println!("Git workflow ID: {}", git_workflow_id);
    
    Ok(())
}

fn parse_module_type(module: &str) -> Result<ModuleType> {
    match module.to_lowercase().as_str() {
        "requirements" => Ok(ModuleType::Requirements),
        "risk" => Ok(ModuleType::Risk),
        "verification" => Ok(ModuleType::Verification),
        "pm" | "project-management" => Ok(ModuleType::ProjectManagement),
        "tol" | "tolerance" => Ok(ModuleType::ToleranceAnalysis),
        "team" => Ok(ModuleType::Team),
        _ => Err(tessera_core::DesignTrackError::Validation(
            format!("Unknown module type: {}", module)
        )),
    }
}

fn parse_entity_state(state: &str) -> Result<EntityState> {
    match state.to_lowercase().as_str() {
        "draft" => Ok(EntityState::Draft),
        "inreview" | "in-review" => Ok(EntityState::InReview),
        "approved" => Ok(EntityState::Approved),
        "released" => Ok(EntityState::Released),
        _ => Err(tessera_core::DesignTrackError::Validation(
            format!("Unknown entity state: {}", state)
        )),
    }
}