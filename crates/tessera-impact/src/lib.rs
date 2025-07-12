pub mod analysis;
pub mod lifecycle;
pub mod git;
pub mod config;
pub mod hooks;

pub use analysis::*;
pub use lifecycle::*;
pub use git::*;
pub use config::*;
pub use hooks::*;

use serde::{Deserialize, Serialize};
use tessera_core::{Id, Result, DesignTrackError};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Helper functions for error conversion (avoiding foreign trait implementation)
pub fn convert_git_error(err: git2::Error) -> DesignTrackError {
    DesignTrackError::Validation(format!("Git error: {}", err))
}

pub fn convert_ron_error(err: ron::Error) -> DesignTrackError {
    DesignTrackError::Validation(format!("RON serialization error: {}", err))
}

pub fn convert_ron_spanned_error(err: ron::error::SpannedError) -> DesignTrackError {
    DesignTrackError::Validation(format!("RON parsing error: {}", err))
}

/// Core entity lifecycle states across all modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityState {
    Draft,
    InReview,
    Approved,
    Released,
}

impl EntityState {
    /// Returns true if this state can transition to the target state
    pub fn can_transition_to(&self, target: EntityState) -> bool {
        match (self, target) {
            (EntityState::Draft, EntityState::InReview) => true,
            (EntityState::InReview, EntityState::Approved) => true,
            (EntityState::InReview, EntityState::Draft) => true, // Back to draft for revisions
            (EntityState::Approved, EntityState::Released) => true,
            (EntityState::Approved, EntityState::InReview) => true, // Back for changes
            _ => false,
        }
    }

    /// Returns approval authority level required for this transition
    pub fn required_approval_level(&self, target: EntityState) -> Option<ApprovalLevel> {
        match (self, target) {
            (EntityState::Draft, EntityState::InReview) => Some(ApprovalLevel::TeamLead),
            (EntityState::InReview, EntityState::Approved) => Some(ApprovalLevel::Manager),
            (EntityState::Approved, EntityState::Released) => Some(ApprovalLevel::Director),
            _ => None,
        }
    }
}

/// Approval authority levels for lifecycle transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ApprovalLevel {
    TeamMember,
    TeamLead,
    Manager,
    Director,
    Executive,
}

/// Impact severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ImpactSeverity {
    Low,        // Cosmetic changes, documentation updates
    Medium,     // Functional changes with limited scope
    High,       // Significant changes affecting multiple areas
    Critical,   // Changes affecting safety, compliance, or major functionality
}

/// Module types in the Tessera ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleType {
    Requirements,
    Risk,
    Verification,
    ProjectManagement,
    ToleranceAnalysis,
    Team,
}

impl std::fmt::Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleType::Requirements => write!(f, "Requirements"),
            ModuleType::Risk => write!(f, "Risk"),
            ModuleType::Verification => write!(f, "Verification"),
            ModuleType::ProjectManagement => write!(f, "Project Management"),
            ModuleType::ToleranceAnalysis => write!(f, "Tolerance Analysis"),
            ModuleType::Team => write!(f, "Team"),
        }
    }
}

/// Entity reference across modules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityReference {
    pub id: Id,
    pub module: ModuleType,
    pub entity_type: String,
    pub name: String,
}

/// Individual change impact on a specific entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeImpact {
    pub id: Id,
    pub target_entity: EntityReference,
    pub impact_type: ImpactType,
    pub severity: ImpactSeverity,
    pub description: String,
    pub affected_attributes: Vec<String>,
    pub propagation_depth: u32,
    pub estimated_effort_hours: Option<f64>,
    pub created: DateTime<Utc>,
}

/// Type of impact relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImpactType {
    DirectLink,          // Direct entity link
    IndirectLink,        // Indirect through other entities
    StateChange,         // Lifecycle state change
    AttributeChange,     // Specific attribute modification
    DependencyChange,    // Dependency relationship change
    RequirementChange,   // Requirements modification
    RiskChange,          // Risk assessment change
    VerificationChange,  // Verification requirement change
}

/// Complete impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub id: Id,
    pub source_entity: EntityReference,
    pub change_description: String,
    pub analysis_timestamp: DateTime<Utc>,
    pub impacts: Vec<ChangeImpact>,
    pub total_affected_entities: usize,
    pub max_severity: ImpactSeverity,
    pub estimated_total_effort_hours: f64,
    pub approval_required: bool,
    pub required_approval_level: Option<ApprovalLevel>,
    pub workflow_id: Option<Id>,
}

impl ImpactAnalysis {
    pub fn new(source_entity: EntityReference, change_description: String) -> Self {
        Self {
            id: Id::new(),
            source_entity,
            change_description,
            analysis_timestamp: Utc::now(),
            impacts: Vec::new(),
            total_affected_entities: 0,
            max_severity: ImpactSeverity::Low,
            estimated_total_effort_hours: 0.0,
            approval_required: false,
            required_approval_level: None,
            workflow_id: None,
        }
    }

    /// Add an impact to this analysis
    pub fn add_impact(&mut self, impact: ChangeImpact) {
        self.total_affected_entities += 1;
        self.estimated_total_effort_hours += impact.estimated_effort_hours.unwrap_or(0.0);
        
        if impact.severity > self.max_severity {
            self.max_severity = impact.severity;
        }

        // Determine if approval is required based on severity
        if impact.severity >= ImpactSeverity::Medium {
            self.approval_required = true;
            let required_level = match impact.severity {
                ImpactSeverity::Low => ApprovalLevel::TeamMember,
                ImpactSeverity::Medium => ApprovalLevel::TeamLead,
                ImpactSeverity::High => ApprovalLevel::Manager,
                ImpactSeverity::Critical => ApprovalLevel::Director,
            };
            
            if self.required_approval_level.is_none() || 
               self.required_approval_level.unwrap() < required_level {
                self.required_approval_level = Some(required_level);
            }
        }

        self.impacts.push(impact);
    }

    /// Get impacts grouped by module
    pub fn impacts_by_module(&self) -> HashMap<ModuleType, Vec<&ChangeImpact>> {
        let mut grouped = HashMap::new();
        for impact in &self.impacts {
            grouped.entry(impact.target_entity.module.clone())
                .or_insert_with(Vec::new)
                .push(impact);
        }
        grouped
    }

    /// Get impacts grouped by severity
    pub fn impacts_by_severity(&self) -> HashMap<ImpactSeverity, Vec<&ChangeImpact>> {
        let mut grouped = HashMap::new();
        for impact in &self.impacts {
            grouped.entry(impact.severity)
                .or_insert_with(Vec::new)
                .push(impact);
        }
        grouped
    }
}

impl std::fmt::Display for EntityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityState::Draft => write!(f, "Draft"),
            EntityState::InReview => write!(f, "In Review"),
            EntityState::Approved => write!(f, "Approved"),
            EntityState::Released => write!(f, "Released"),
        }
    }
}

impl std::fmt::Display for ImpactSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpactSeverity::Low => write!(f, "Low"),
            ImpactSeverity::Medium => write!(f, "Medium"),
            ImpactSeverity::High => write!(f, "High"),
            ImpactSeverity::Critical => write!(f, "Critical"),
        }
    }
}

impl std::fmt::Display for ApprovalLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalLevel::TeamMember => write!(f, "Team Member"),
            ApprovalLevel::TeamLead => write!(f, "Team Lead"),
            ApprovalLevel::Manager => write!(f, "Manager"),
            ApprovalLevel::Director => write!(f, "Director"),
            ApprovalLevel::Executive => write!(f, "Executive"),
        }
    }
}