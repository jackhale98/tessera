// Core services and business logic
pub mod error;
pub mod entity_manager;
pub mod link_manager;
pub mod app_state;
pub mod calculation_engine;
pub mod managers;

pub use error::{EdtError, EdtResult};
pub use entity_manager::EntityManager;
pub use link_manager::LinkManager;
pub use app_state::AppState;
pub use calculation_engine::{CalculationEngine, CriticalPathResult, EvmMetrics};
