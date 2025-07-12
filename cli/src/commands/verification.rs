use crate::VerificationCommands;
use crate::impact_service::get_impact_service;
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use tessera_verification::*;
use tessera_impact::{ModuleType, ChangeType};

pub async fn execute_verification_command(command: VerificationCommands, project_ctx: ProjectContext) -> Result<()> {
    match command {
        VerificationCommands::AddProcedure => add_procedure_interactive(project_ctx).await,
        VerificationCommands::ListProcedures => list_procedures(project_ctx).await,
        VerificationCommands::EditProcedure => edit_procedure_interactive(project_ctx).await,
        VerificationCommands::AddExecution => add_execution_interactive(project_ctx).await,
        VerificationCommands::ListExecutions => list_executions(project_ctx).await,
        VerificationCommands::EditExecution => edit_execution_interactive(project_ctx).await,
        VerificationCommands::Dashboard => show_dashboard(project_ctx).await,
        VerificationCommands::GenerateReport => generate_report(project_ctx).await,
    }
}

async fn add_procedure_interactive(project_ctx: ProjectContext) -> Result<()> {
    let mut commands = tessera_verification::VerificationCommands::new(project_ctx.clone())?;
    let result = commands.add_procedure_interactive();
    
    // Trigger automatic impact analysis if procedure was created
    if result.is_ok() {
        if let Some(procedure) = commands.get_last_created_procedure() {
            if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                let _ = service.on_entity_changed(
                    procedure,
                    ModuleType::Verification,
                    "TestProcedure".to_string(),
                    ChangeType::Created,
                    &project_ctx,
                ).await;
            }
        }
    }
    
    result
}

async fn list_procedures(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.list_procedures()
}

async fn edit_procedure_interactive(project_ctx: ProjectContext) -> Result<()> {
    let mut commands = tessera_verification::VerificationCommands::new(project_ctx.clone())?;
    let result = commands.edit_procedure_interactive();
    
    // Trigger automatic impact analysis if procedure was updated
    if result.is_ok() {
        if let Some(procedure) = commands.get_last_updated_procedure() {
            if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                let _ = service.on_entity_changed(
                    procedure,
                    ModuleType::Verification,
                    "TestProcedure".to_string(),
                    ChangeType::Updated,
                    &project_ctx,
                ).await;
            }
        }
    }
    
    result
}

async fn add_execution_interactive(project_ctx: ProjectContext) -> Result<()> {
    let mut commands = tessera_verification::VerificationCommands::new(project_ctx.clone())?;
    let result = commands.execute_procedure_interactive();
    
    // Trigger automatic impact analysis if execution was created
    if result.is_ok() {
        if let Some(execution) = commands.get_last_created_execution() {
            if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                let _ = service.on_entity_changed(
                    execution,
                    ModuleType::Verification,
                    "TestExecution".to_string(),
                    ChangeType::Created,
                    &project_ctx,
                ).await;
            }
        }
    }
    
    result
}

async fn list_executions(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.list_executions()
}

async fn edit_execution_interactive(project_ctx: ProjectContext) -> Result<()> {
    let mut commands = tessera_verification::VerificationCommands::new(project_ctx.clone())?;
    let result = commands.edit_execution_interactive();
    
    // Trigger automatic impact analysis if execution was updated
    if result.is_ok() {
        if let Some(execution) = commands.get_last_updated_execution() {
            if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                let _ = service.on_entity_changed(
                    execution,
                    ModuleType::Verification,
                    "TestExecution".to_string(),
                    ChangeType::Updated,
                    &project_ctx,
                ).await;
            }
        }
    }
    
    result
}

async fn show_dashboard(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.show_dashboard()
}

async fn generate_report(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Verification report generation functionality not yet implemented.".yellow());
    Ok(())
}