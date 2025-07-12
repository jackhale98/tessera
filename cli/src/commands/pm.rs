use crate::PmCommands;
use crate::impact_service::get_impact_service;
use tessera_core::{ProjectContext, Result};
use tessera_pm::ProjectCommands;
use tessera_impact::{ModuleType, ChangeType};

pub async fn execute_pm_command(command: PmCommands, project_ctx: ProjectContext) -> Result<()> {
    let mut pm_commands = ProjectCommands::new(project_ctx.clone())?;
    
    let result = match &command {
        PmCommands::AddTask => {
            let result = pm_commands.add_task_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for task creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let tasks = pm_commands.get_tasks();
                    if let Some(last_task) = tasks.last() {
                        let _ = service.on_entity_changed(
                            last_task,
                            ModuleType::ProjectManagement,
                            "Task".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::ListTasks => pm_commands.list_tasks(),
        PmCommands::EditTask => {
            let result = pm_commands.edit_task_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for task update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let tasks = pm_commands.get_tasks();
                    if let Some(last_task) = tasks.last() {
                        let _ = service.on_entity_changed(
                            last_task,
                            ModuleType::ProjectManagement,
                            "Task".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::DeleteTask => {
            let result = pm_commands.delete_task_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for task deletion
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let tasks = pm_commands.get_tasks();
                    if let Some(last_task) = tasks.last() {
                        let _ = service.on_entity_changed(
                            last_task,
                            ModuleType::ProjectManagement,
                            "Task".to_string(),
                            ChangeType::Deleted,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::AddResource => {
            let result = pm_commands.add_resource_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for resource creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let resources = pm_commands.get_resources();
                    if let Some(last_resource) = resources.last() {
                        let _ = service.on_entity_changed(
                            last_resource,
                            ModuleType::ProjectManagement,
                            "Resource".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::ListResources => pm_commands.list_resources(),
        PmCommands::EditResource => {
            let result = pm_commands.edit_resource_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for resource update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let resources = pm_commands.get_resources();
                    if let Some(last_resource) = resources.last() {
                        let _ = service.on_entity_changed(
                            last_resource,
                            ModuleType::ProjectManagement,
                            "Resource".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::DeleteResource => {
            let result = pm_commands.delete_resource_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for resource deletion
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let resources = pm_commands.get_resources();
                    if let Some(last_resource) = resources.last() {
                        let _ = service.on_entity_changed(
                            last_resource,
                            ModuleType::ProjectManagement,
                            "Resource".to_string(),
                            ChangeType::Deleted,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::AddMilestone => {
            let result = pm_commands.add_milestone_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for milestone creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let milestones = pm_commands.get_milestones();
                    if let Some(last_milestone) = milestones.last() {
                        let _ = service.on_entity_changed(
                            last_milestone,
                            ModuleType::ProjectManagement,
                            "Milestone".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::ListMilestones => pm_commands.list_milestones(),
        PmCommands::EditMilestone => {
            let result = pm_commands.edit_milestone_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for milestone update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let milestones = pm_commands.get_milestones();
                    if let Some(last_milestone) = milestones.last() {
                        let _ = service.on_entity_changed(
                            last_milestone,
                            ModuleType::ProjectManagement,
                            "Milestone".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::DeleteMilestone => {
            let result = pm_commands.delete_milestone_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for milestone deletion
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let milestones = pm_commands.get_milestones();
                    if let Some(last_milestone) = milestones.last() {
                        let _ = service.on_entity_changed(
                            last_milestone,
                            ModuleType::ProjectManagement,
                            "Milestone".to_string(),
                            ChangeType::Deleted,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::Schedule => pm_commands.compute_schedule(),
        PmCommands::CostAnalysis => pm_commands.show_cost_analysis(),
        PmCommands::Dashboard => pm_commands.show_dashboard(),
        PmCommands::AddRisk => {
            let result = pm_commands.add_risk_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for risk creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let risks = pm_commands.get_risks();
                    if let Some(last_risk) = risks.last() {
                        let _ = service.on_entity_changed(
                            last_risk,
                            ModuleType::ProjectManagement,
                            "ProjectRisk".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::ListRisks => pm_commands.list_risks(),
        PmCommands::EditRisk => {
            let result = pm_commands.edit_risk_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for risk update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let risks = pm_commands.get_risks();
                    if let Some(last_risk) = risks.last() {
                        let _ = service.on_entity_changed(
                            last_risk,
                            ModuleType::ProjectManagement,
                            "ProjectRisk".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::AddIssue => {
            let result = pm_commands.add_issue_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for issue creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let issues = pm_commands.get_issues();
                    if let Some(last_issue) = issues.last() {
                        let _ = service.on_entity_changed(
                            last_issue,
                            ModuleType::ProjectManagement,
                            "Issue".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::ListIssues => pm_commands.list_issues(),
        PmCommands::EditIssue => {
            let result = pm_commands.edit_issue_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for issue update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let issues = pm_commands.get_issues();
                    if let Some(last_issue) = issues.last() {
                        let _ = service.on_entity_changed(
                            last_issue,
                            ModuleType::ProjectManagement,
                            "Issue".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::CreateBaseline => {
            let result = pm_commands.create_baseline_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for baseline creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let baselines = pm_commands.get_baselines();
                    if let Some(last_baseline) = baselines.last() {
                        let _ = service.on_entity_changed(
                            last_baseline,
                            ModuleType::ProjectManagement,
                            "ProjectBaseline".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::ListBaselines => pm_commands.list_baselines(),
        PmCommands::CompareBaselines => pm_commands.compare_baselines_interactive(),
        PmCommands::AddCalendar => {
            let result = pm_commands.add_calendar_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for calendar creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let calendars = pm_commands.get_calendars();
                    if let Some(last_calendar) = calendars.last() {
                        let _ = service.on_entity_changed(
                            last_calendar,
                            ModuleType::ProjectManagement,
                            "Calendar".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::ListCalendars => pm_commands.list_calendars(),
        PmCommands::EditCalendar => {
            let result = pm_commands.edit_calendar_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for calendar update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let calendars = pm_commands.get_calendars();
                    if let Some(last_calendar) = calendars.last() {
                        let _ = service.on_entity_changed(
                            last_calendar,
                            ModuleType::ProjectManagement,
                            "Calendar".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        PmCommands::AssignCalendar => pm_commands.assign_calendar_to_resource_interactive().await,
        PmCommands::ListCalendarAssignments => pm_commands.list_resource_calendar_assignments(),
        PmCommands::UnassignCalendar => pm_commands.remove_calendar_assignment_interactive().await,
        PmCommands::CheckMilestoneStatus => pm_commands.check_milestone_status(),
        PmCommands::ShowMilestoneAlerts => pm_commands.show_milestone_alerts(),
        PmCommands::AnalyzeTaskCriticalPath => pm_commands.analyze_task_critical_path_interactive().await,
        PmCommands::AnalyzeMilestoneCriticalPath => pm_commands.analyze_milestone_critical_path_interactive().await,
    };

    result
}