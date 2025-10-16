// Modular entity manager implementations
pub mod task_manager;
pub mod requirement_manager;
pub mod risk_manager;
pub mod design_manager;
pub mod testing_manager;
pub mod manufacturing_manager;

pub use task_manager::TaskManager;
pub use requirement_manager::RequirementManager;
pub use risk_manager::RiskManager;
pub use design_manager::DesignManager;
pub use testing_manager::TestingManager;
pub use manufacturing_manager::ManufacturingManager;
