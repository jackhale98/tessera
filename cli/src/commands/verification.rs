use crate::VerificationCommands;
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use tessera_verification::*;

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
    let mut commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.add_procedure_interactive()
}

async fn list_procedures(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.list_procedures()
}

async fn edit_procedure_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit procedure functionality not yet implemented.".yellow());
    Ok(())
}

async fn add_execution_interactive(project_ctx: ProjectContext) -> Result<()> {
    let mut commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.execute_procedure_interactive()
}

async fn list_executions(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.list_executions()
}

async fn edit_execution_interactive(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Edit execution functionality not yet implemented.".yellow());
    Ok(())
}

async fn show_dashboard(project_ctx: ProjectContext) -> Result<()> {
    let commands = tessera_verification::VerificationCommands::new(project_ctx)?;
    commands.show_dashboard()
}

async fn generate_report(_project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Verification report generation functionality not yet implemented.".yellow());
    Ok(())
}