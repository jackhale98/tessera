use crate::{ImpactAnalyzer, EntityReference, ModuleType, ImpactAnalysis};
use tessera_core::{Id, Result, ProjectContext, Entity};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::Path;

/// Change event that triggers impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub id: Id,
    pub entity_reference: EntityReference,
    pub change_type: ChangeType,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub previous_state: Option<String>,
    pub new_state: Option<String>,
}

/// Types of changes that can trigger impact analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
    StateTransition,
    LinkAdded,
    LinkRemoved,
}

/// Background impact analysis engine that monitors for changes
pub struct ImpactEngine {
    analyzer: ImpactAnalyzer,
    enabled: bool,
    auto_analyze_threshold: f64, // Only auto-analyze if estimated effort is below this threshold
}

impl ImpactEngine {
    pub fn new() -> Self {
        Self {
            analyzer: ImpactAnalyzer::new(),
            enabled: true,
            auto_analyze_threshold: 8.0, // 8 hours
        }
    }

    /// Hook that should be called after any entity save operation
    pub async fn on_entity_saved<T: Entity>(
        &mut self,
        entity: &T,
        module: ModuleType,
        entity_type: String,
        change_type: ChangeType,
        project_ctx: &ProjectContext,
    ) -> Result<Option<ImpactAnalysis>> {
        if !self.enabled {
            return Ok(None);
        }

        // Create entity reference
        let entity_ref = EntityReference {
            id: entity.id(),
            module,
            entity_type: entity_type.clone(),
            name: entity.name().to_string(),
        };

        // Generate change description
        let change_description = match change_type {
            ChangeType::Created => format!("Created new {}: {}", entity_type, entity.name()),
            ChangeType::Updated => format!("Updated {}: {}", entity_type, entity.name()),
            ChangeType::Deleted => format!("Deleted {}: {}", entity_type, entity.name()),
            ChangeType::StateTransition => format!("State transition for {}: {}", entity_type, entity.name()),
            ChangeType::LinkAdded => format!("Added link to {}: {}", entity_type, entity.name()),
            ChangeType::LinkRemoved => format!("Removed link from {}: {}", entity_type, entity.name()),
        };

        // Load cross-module relationships
        self.analyzer.load_cross_module_relationships(project_ctx).await?;

        // Perform impact analysis
        let analysis = self.analyzer.analyze_impact(entity_ref, change_description, project_ctx).await?;

        // Only proceed with automatic actions if below threshold
        if analysis.estimated_total_effort_hours <= self.auto_analyze_threshold {
            // Save the analysis
            self.save_analysis(&analysis, project_ctx)?;

            // Log the change event
            self.log_change_event(&analysis, change_type, project_ctx)?;
        }

        Ok(Some(analysis))
    }

    /// Save impact analysis to project directory
    fn save_analysis(&self, analysis: &ImpactAnalysis, project_ctx: &ProjectContext) -> Result<()> {
        let impact_dir = project_ctx.root_path.join("impact").join("analyses");
        tessera_core::ensure_directory(&impact_dir)?;

        let analysis_file = impact_dir.join(format!("impact_analysis_{}.ron", analysis.id));
        let content = ron::ser::to_string_pretty(analysis, ron::ser::PrettyConfig::default())
            .map_err(crate::convert_ron_error)?;
        
        std::fs::write(analysis_file, content)?;
        Ok(())
    }

    /// Log change event for audit trail
    fn log_change_event(&self, analysis: &ImpactAnalysis, change_type: ChangeType, project_ctx: &ProjectContext) -> Result<()> {
        let event = ChangeEvent {
            id: Id::new(),
            entity_reference: analysis.source_entity.clone(),
            change_type,
            timestamp: analysis.analysis_timestamp,
            description: analysis.change_description.clone(),
            previous_state: None,
            new_state: None,
        };

        let events_dir = project_ctx.root_path.join("impact").join("events");
        tessera_core::ensure_directory(&events_dir)?;

        let event_file = events_dir.join(format!("change_event_{}.ron", event.id));
        let content = ron::ser::to_string_pretty(&event, ron::ser::PrettyConfig::default())
            .map_err(crate::convert_ron_error)?;
        
        std::fs::write(event_file, content)?;
        Ok(())
    }

    /// Enable or disable automatic impact analysis
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set the effort threshold for automatic analysis
    pub fn set_auto_analyze_threshold(&mut self, threshold: f64) {
        self.auto_analyze_threshold = threshold;
    }

    /// Get all saved impact analyses
    pub fn get_saved_analyses(&self, project_ctx: &ProjectContext) -> Result<Vec<ImpactAnalysis>> {
        let analyses_dir = project_ctx.root_path.join("impact").join("analyses");
        
        if !analyses_dir.exists() {
            return Ok(Vec::new());
        }

        let mut analyses = Vec::new();
        for entry in std::fs::read_dir(analyses_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("ron") &&
               path.file_stem().and_then(|s| s.to_str()).unwrap_or("").starts_with("impact_analysis_") {
                
                let content = std::fs::read_to_string(path)?;
                let analysis: ImpactAnalysis = ron::de::from_str(&content)
                    .map_err(crate::convert_ron_spanned_error)?;
                analyses.push(analysis);
            }
        }

        // Sort by timestamp, newest first
        analyses.sort_by(|a, b| b.analysis_timestamp.cmp(&a.analysis_timestamp));
        Ok(analyses)
    }

    /// Get all change events
    pub fn get_change_events(&self, project_ctx: &ProjectContext) -> Result<Vec<ChangeEvent>> {
        let events_dir = project_ctx.root_path.join("impact").join("events");
        
        if !events_dir.exists() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        for entry in std::fs::read_dir(events_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("ron") &&
               path.file_stem().and_then(|s| s.to_str()).unwrap_or("").starts_with("change_event_") {
                
                let content = std::fs::read_to_string(path)?;
                let event: ChangeEvent = ron::de::from_str(&content)
                    .map_err(crate::convert_ron_spanned_error)?;
                events.push(event);
            }
        }

        // Sort by timestamp, newest first
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(events)
    }
}

impl Default for ImpactEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Global impact engine instance
static mut IMPACT_ENGINE: Option<ImpactEngine> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Get the global impact engine instance
pub fn get_impact_engine() -> &'static mut ImpactEngine {
    unsafe {
        INIT.call_once(|| {
            IMPACT_ENGINE = Some(ImpactEngine::new());
        });
        IMPACT_ENGINE.as_mut().unwrap()
    }
}

/// Macro to easily hook impact analysis into save operations
#[macro_export]
macro_rules! impact_hook {
    ($entity:expr, $module:expr, $entity_type:expr, $change_type:expr, $project_ctx:expr) => {
        if let Ok(engine) = std::panic::catch_unwind(|| {
            $crate::hooks::get_impact_engine()
        }) {
            let _ = engine.on_entity_saved($entity, $module, $entity_type, $change_type, $project_ctx).await;
        }
    };
}