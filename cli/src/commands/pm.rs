use crate::PmCommands;
use tessera_core::{ProjectContext, Result};
use tessera_pm::ProjectCommands;

pub async fn execute_pm_command(command: PmCommands, project_ctx: ProjectContext) -> Result<()> {
    let mut pm_commands = ProjectCommands::new(project_ctx)?;
    
    match command {
        PmCommands::AddTask => pm_commands.add_task_interactive().await,
        PmCommands::ListTasks => pm_commands.list_tasks(),
        PmCommands::EditTask => pm_commands.edit_task_interactive().await,
        PmCommands::DeleteTask => pm_commands.delete_task_interactive().await,
        PmCommands::AddResource => pm_commands.add_resource_interactive().await,
        PmCommands::ListResources => pm_commands.list_resources(),
        PmCommands::EditResource => pm_commands.edit_resource_interactive().await,
        PmCommands::DeleteResource => pm_commands.delete_resource_interactive().await,
        PmCommands::AddMilestone => pm_commands.add_milestone_interactive().await,
        PmCommands::ListMilestones => pm_commands.list_milestones(),
        PmCommands::EditMilestone => pm_commands.edit_milestone_interactive().await,
        PmCommands::DeleteMilestone => pm_commands.delete_milestone_interactive().await,
        PmCommands::Schedule => pm_commands.compute_schedule(),
        PmCommands::CostAnalysis => pm_commands.show_cost_analysis(),
        PmCommands::Dashboard => pm_commands.show_dashboard(),
        PmCommands::AddRisk => pm_commands.add_risk_interactive().await,
        PmCommands::ListRisks => pm_commands.list_risks(),
        PmCommands::EditRisk => pm_commands.edit_risk_interactive().await,
        PmCommands::AddIssue => pm_commands.add_issue_interactive().await,
        PmCommands::ListIssues => pm_commands.list_issues(),
        PmCommands::EditIssue => pm_commands.edit_issue_interactive().await,
        PmCommands::CreateBaseline => pm_commands.create_baseline_interactive().await,
        PmCommands::ListBaselines => pm_commands.list_baselines(),
        PmCommands::CompareBaselines => pm_commands.compare_baselines_interactive(),
        PmCommands::AddCalendar => pm_commands.add_calendar_interactive().await,
        PmCommands::ListCalendars => pm_commands.list_calendars(),
        PmCommands::EditCalendar => pm_commands.edit_calendar_interactive().await,
        PmCommands::AssignCalendar => pm_commands.assign_calendar_to_resource_interactive().await,
        PmCommands::ListCalendarAssignments => pm_commands.list_resource_calendar_assignments(),
        PmCommands::UnassignCalendar => pm_commands.remove_calendar_assignment_interactive().await,
        PmCommands::CheckMilestoneStatus => pm_commands.check_milestone_status(),
        PmCommands::ShowMilestoneAlerts => pm_commands.show_milestone_alerts(),
        PmCommands::AnalyzeTaskCriticalPath => pm_commands.analyze_task_critical_path_interactive().await,
        PmCommands::AnalyzeMilestoneCriticalPath => pm_commands.analyze_milestone_critical_path_interactive().await,
    }
}