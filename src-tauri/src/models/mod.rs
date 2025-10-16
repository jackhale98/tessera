// Data models and entity definitions
pub mod entity;
pub mod task;
pub mod requirement;
pub mod risk;
pub mod link;
pub mod config;
pub mod design;
pub mod testing;
pub mod manufacturing;

pub use entity::{EntityMetadata, EntityStatus, EntityType};
pub use task::{
    Task, TaskType, SchedulingMode, ResourceAssignment, TaskDependency,
    DependencyType, TaskBaseline, Milestone, Resource, ResourceType,
    Calendar, Baseline,
};
pub use requirement::Requirement;
pub use risk::{Hazard, Risk, RiskControl};
pub use link::{Link, LinkType, LinkMetadata};
pub use config::ProjectConfig;
pub use design::{
    Assembly, Component, Feature, FeatureType, DistributionType,
    Mate, MateType, MateAnalysisResult,
    Stackup, AnalysisType, StackupResult, MonteCarloResult,
    ContributionSign, StackupFeatureContribution,
    Supplier, Quote, CostDistribution,
    BomItem, BomResult, CostEstimate,
};
pub use testing::{
    Verification, Validation, TestStatus, TestPriority, TestStep,
};
pub use manufacturing::{
    Manufacturing, ProcessStatus, QualityStatus,
    WorkInstructionStep, QualityCheckpoint, ProductionBatch,
};
