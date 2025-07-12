use crate::TolCommands;
use crate::impact_service::get_impact_service;
use tessera_core::{ProjectContext, Result};
use tessera_impact::{ModuleType, ChangeType};
use tessera_tol::ToleranceCommands;

pub async fn execute_tol_command(command: TolCommands, project_ctx: ProjectContext) -> Result<()> {
    let mut tol_commands = ToleranceCommands::new(project_ctx.clone())?;
    
    let result = match &command {
        TolCommands::AddComponent => {
            let result = tol_commands.add_component_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for component creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let components = tol_commands.get_components();
                    if let Some(last_component) = components.last() {
                        let _ = service.on_entity_changed(
                            last_component,
                            ModuleType::ToleranceAnalysis,
                            "Component".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::EditComponent => {
            let result = tol_commands.edit_component_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for component update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let components = tol_commands.get_components();
                    if let Some(last_component) = components.last() {
                        let _ = service.on_entity_changed(
                            last_component,
                            ModuleType::ToleranceAnalysis,
                            "Component".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::ListComponents => tol_commands.list_components(),
        TolCommands::AddFeature => {
            let result = tol_commands.add_feature_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for feature creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let features = tol_commands.get_features();
                    if let Some(last_feature) = features.last() {
                        let _ = service.on_entity_changed(
                            last_feature,
                            ModuleType::ToleranceAnalysis,
                            "Feature".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::EditFeature => {
            let result = tol_commands.edit_feature_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for feature update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let features = tol_commands.get_features();
                    if let Some(last_feature) = features.last() {
                        let _ = service.on_entity_changed(
                            last_feature,
                            ModuleType::ToleranceAnalysis,
                            "Feature".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::ListFeatures => tol_commands.list_features_interactive().await,
        TolCommands::AddMate => {
            let result = tol_commands.add_mate_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for mate creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let mates = tol_commands.get_mates();
                    if let Some(last_mate) = mates.last() {
                        let _ = service.on_entity_changed(
                            last_mate,
                            ModuleType::ToleranceAnalysis,
                            "Mate".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::EditMate => {
            let result = tol_commands.edit_mate_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for mate update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let mates = tol_commands.get_mates();
                    if let Some(last_mate) = mates.last() {
                        let _ = service.on_entity_changed(
                            last_mate,
                            ModuleType::ToleranceAnalysis,
                            "Mate".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::ListMates => tol_commands.list_mates_interactive().await,
        TolCommands::AddStackup => {
            let result = tol_commands.add_stackup_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for stackup creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let stackups = tol_commands.get_stackups();
                    if let Some(last_stackup) = stackups.last() {
                        let _ = service.on_entity_changed(
                            last_stackup,
                            ModuleType::ToleranceAnalysis,
                            "Stackup".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::EditStackup => {
            let result = tol_commands.edit_stackup_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for stackup update
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let stackups = tol_commands.get_stackups();
                    if let Some(last_stackup) = stackups.last() {
                        let _ = service.on_entity_changed(
                            last_stackup,
                            ModuleType::ToleranceAnalysis,
                            "Stackup".to_string(),
                            ChangeType::Updated,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::DeleteStackup => {
            let result = tol_commands.delete_stackup_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for stackup deletion
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    // For deletion, we'll trigger analysis for each remaining stackup
                    let stackups = tol_commands.get_stackups();
                    for stackup in stackups {
                        let _ = service.on_entity_changed(
                            stackup,
                            ModuleType::ToleranceAnalysis,
                            "Stackup".to_string(),
                            ChangeType::Deleted,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::RunAnalysis => {
            let result = tol_commands.run_analysis_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for analysis creation
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    let analyses = tol_commands.get_analyses();
                    if let Some(last_analysis) = analyses.last() {
                        let _ = service.on_entity_changed(
                            last_analysis,
                            ModuleType::ToleranceAnalysis,
                            "Analysis".to_string(),
                            ChangeType::Created,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::ListAnalysis => tol_commands.list_analysis_interactive().await,
        TolCommands::DeleteAnalysis => {
            let result = tol_commands.delete_analysis_interactive().await;
            if result.is_ok() {
                // Trigger automatic impact analysis for analysis deletion
                if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
                    // For deletion, we'll trigger analysis for each remaining analysis
                    let analyses = tol_commands.get_analyses();
                    for analysis in analyses {
                        let _ = service.on_entity_changed(
                            analysis,
                            ModuleType::ToleranceAnalysis,
                            "Analysis".to_string(),
                            ChangeType::Deleted,
                            &project_ctx,
                        ).await;
                    }
                }
            }
            result
        },
        TolCommands::Dashboard => tol_commands.show_dashboard(),
    };

    result
}