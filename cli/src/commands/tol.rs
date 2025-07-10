use crate::TolCommands;
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use tessera_tol::ToleranceCommands;

pub async fn execute_tol_command(command: TolCommands, project_ctx: ProjectContext) -> Result<()> {
    let mut tol_commands = ToleranceCommands::new(project_ctx)?;
    
    match command {
        TolCommands::AddComponent => tol_commands.add_component_interactive().await,
        TolCommands::ListComponents => tol_commands.list_components(),
        TolCommands::AddFeature => tol_commands.add_feature_interactive().await,
        TolCommands::AddStackup => tol_commands.add_stackup_interactive().await,
        TolCommands::RunAnalysis => tol_commands.run_analysis(None),
        TolCommands::Dashboard => tol_commands.show_dashboard(),
    }
}