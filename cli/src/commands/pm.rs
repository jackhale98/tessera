use crate::PmCommands;
use tessera_core::{ProjectContext, Result};
use tessera_pm::ProjectCommands;

pub async fn execute_pm_command(command: PmCommands, project_ctx: ProjectContext) -> Result<()> {
    let mut pm_commands = ProjectCommands::new(project_ctx)?;
    
    match command {
        PmCommands::AddTask => pm_commands.add_task_interactive().await,
        PmCommands::ListTasks => pm_commands.list_tasks(),
        PmCommands::EditTask => pm_commands.edit_task_interactive().await,
        PmCommands::AddResource => pm_commands.add_resource_interactive().await,
        PmCommands::EditResource => pm_commands.edit_resource_interactive().await,
        PmCommands::AddMilestone => pm_commands.add_milestone_interactive().await,
        PmCommands::EditMilestone => pm_commands.edit_milestone_interactive().await,
        PmCommands::Schedule => pm_commands.compute_schedule(),
        PmCommands::Dashboard => pm_commands.show_dashboard(),
    }
}