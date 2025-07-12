use tessera_impact::{ImpactEngine, ChangeType, ModuleType};
use tessera_core::{Entity, Result, ProjectContext};

/// Service wrapper that provides impact-aware entity operations at the CLI level
/// This avoids circular dependencies by coordinating at the highest level
pub struct ImpactAwareService {
    engine: ImpactEngine,
}

impl ImpactAwareService {
    pub fn new() -> Self {
        Self {
            engine: ImpactEngine::new(),
        }
    }

    /// Trigger impact analysis after an entity operation
    pub async fn on_entity_changed<T: Entity>(
        &mut self,
        entity: &T,
        module: ModuleType,
        entity_type: String,
        change_type: ChangeType,
        project_ctx: &ProjectContext,
    ) -> Result<()> {
        // Trigger impact analysis
        self.engine.on_entity_saved(
            entity,
            module,
            entity_type,
            change_type,
            project_ctx,
        ).await?;

        Ok(())
    }

    /// Get the underlying impact engine for direct access
    pub fn engine(&mut self) -> &mut ImpactEngine {
        &mut self.engine
    }
}

impl Default for ImpactAwareService {
    fn default() -> Self {
        Self::new()
    }
}

/// Global impact service instance for the CLI
static mut IMPACT_SERVICE: Option<ImpactAwareService> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Get the global impact service instance
pub fn get_impact_service() -> &'static mut ImpactAwareService {
    unsafe {
        INIT.call_once(|| {
            IMPACT_SERVICE = Some(ImpactAwareService::new());
        });
        IMPACT_SERVICE.as_mut().unwrap()
    }
}

/// Macro to easily trigger impact analysis from CLI commands
#[macro_export]
macro_rules! trigger_impact_analysis {
    ($entity:expr, $module:expr, $entity_type:expr, $change_type:expr, $project_ctx:expr) => {
        if let Ok(service) = std::panic::catch_unwind(|| {
            $crate::impact_service::get_impact_service()
        }) {
            let _ = service.on_entity_changed($entity, $module, $entity_type, $change_type, $project_ctx).await;
        }
    };
}