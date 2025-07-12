use crate::TolCommands;
use tessera_core::{ProjectContext, Result};
use tessera_tol::ToleranceCommands;

pub async fn execute_tol_command(command: TolCommands, project_ctx: ProjectContext) -> Result<()> {
    let mut tol_commands = ToleranceCommands::new(project_ctx)?;
    
    match command {
        TolCommands::AddComponent => tol_commands.add_component_interactive().await,
        TolCommands::EditComponent => tol_commands.edit_component_interactive().await,
        TolCommands::ListComponents => tol_commands.list_components(),
        TolCommands::AddFeature => tol_commands.add_feature_interactive().await,
        TolCommands::EditFeature => tol_commands.edit_feature_interactive().await,
        TolCommands::ListFeatures => tol_commands.list_features_interactive().await,
        TolCommands::AddMate => tol_commands.add_mate_interactive().await,
        TolCommands::EditMate => tol_commands.edit_mate_interactive().await,
        TolCommands::ListMates => tol_commands.list_mates_interactive().await,
        TolCommands::AddStackup => tol_commands.add_stackup_interactive().await,
        TolCommands::EditStackup => tol_commands.edit_stackup_interactive().await,
        TolCommands::DeleteStackup => tol_commands.delete_stackup_interactive().await,
        TolCommands::RunAnalysis => tol_commands.run_analysis_interactive().await,
        TolCommands::ListAnalysis => tol_commands.list_analysis_interactive().await,
        TolCommands::DeleteAnalysis => tol_commands.delete_analysis_interactive().await,
        TolCommands::Dashboard => tol_commands.show_dashboard(),
    }
}