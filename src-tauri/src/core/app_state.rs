use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::core::{EdtResult, EntityManager, LinkManager, CalculationEngine};
use crate::storage::RonStorage;

/// Application state shared across Tauri commands
pub struct AppState {
    pub entity_manager: Arc<EntityManager>,
    pub link_manager: Arc<Mutex<LinkManager>>,
    pub calculation_engine: Arc<CalculationEngine>,
    pub storage: Arc<RonStorage>,
    pub project_root: PathBuf,
}

impl AppState {
    /// Create a new AppState for a project
    pub fn new(project_root: PathBuf) -> EdtResult<Self> {
        let storage = Arc::new(RonStorage::new(&project_root)?);
        let entity_manager = Arc::new(EntityManager::new(Arc::clone(&storage)));
        let link_manager = Arc::new(Mutex::new(LinkManager::new()));
        let calculation_engine = Arc::new(CalculationEngine::new(
            Arc::clone(&entity_manager),
            Arc::clone(&link_manager)
        ));

        Ok(Self {
            entity_manager,
            link_manager,
            calculation_engine,
            storage,
            project_root,
        })
    }

    /// Get the project root path
    pub fn project_root(&self) -> &PathBuf {
        &self.project_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_state_creation() {
        let temp_dir = TempDir::new().unwrap();
        let app_state = AppState::new(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(app_state.project_root(), temp_dir.path());
    }

    #[test]
    fn test_app_state_entity_manager() {
        let temp_dir = TempDir::new().unwrap();
        let app_state = AppState::new(temp_dir.path().to_path_buf()).unwrap();

        // Test that we can use the entity manager
        let task_ids = app_state.entity_manager.list_task_ids().unwrap();
        assert_eq!(task_ids.len(), 0);
    }

    #[test]
    fn test_app_state_link_manager() {
        let temp_dir = TempDir::new().unwrap();
        let app_state = AppState::new(temp_dir.path().to_path_buf()).unwrap();

        // Test that we can use the link manager
        let link_manager = app_state.link_manager.lock().unwrap();
        assert_eq!(link_manager.link_count(), 0);
    }
}
