//! Data structures for requirements management
//!
//! This module defines the core data types used throughout the requirements system,
//! including requirements, design inputs, outputs, and verifications.

use serde::{Deserialize, Serialize};
use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Priority levels for requirements and related entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "Low"),
            Priority::Medium => write!(f, "Medium"),
            Priority::High => write!(f, "High"),
            Priority::Critical => write!(f, "Critical"),
        }
    }
}

/// Categories for organizing requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementCategory {
    Functional,
    Performance,
    Safety,
    Security,
    Usability,
    Reliability,
    Maintainability,
    Regulatory,
    Interface,
    Custom(String),
}

impl std::fmt::Display for RequirementCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequirementCategory::Functional => write!(f, "Functional"),
            RequirementCategory::Performance => write!(f, "Performance"),
            RequirementCategory::Safety => write!(f, "Safety"),
            RequirementCategory::Security => write!(f, "Security"),
            RequirementCategory::Usability => write!(f, "Usability"),
            RequirementCategory::Reliability => write!(f, "Reliability"),
            RequirementCategory::Maintainability => write!(f, "Maintainability"),
            RequirementCategory::Regulatory => write!(f, "Regulatory"),
            RequirementCategory::Interface => write!(f, "Interface"),
            RequirementCategory::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Status of a requirement throughout its lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequirementStatus {
    Draft,
    UnderReview,
    Approved,
    Implemented,
    Verified,
    Obsolete,
}

impl std::fmt::Display for RequirementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequirementStatus::Draft => write!(f, "Draft"),
            RequirementStatus::UnderReview => write!(f, "Under Review"),
            RequirementStatus::Approved => write!(f, "Approved"),
            RequirementStatus::Implemented => write!(f, "Implemented"),
            RequirementStatus::Verified => write!(f, "Verified"),
            RequirementStatus::Obsolete => write!(f, "Obsolete"),
        }
    }
}

/// Core requirement entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: RequirementCategory,
    pub priority: Priority,
    pub status: RequirementStatus,
    pub rationale: Option<String>,
    pub source: Option<String>,
    pub stakeholder: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Requirement {
    /// Create a new requirement with default values
    pub fn new(
        name: String,
        description: String,
        category: RequirementCategory,
        priority: Priority,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            category,
            priority,
            status: RequirementStatus::Draft,
            rationale: None,
            source: None,
            stakeholder: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }


    /// Update the requirement status
    pub fn update_status(&mut self, status: RequirementStatus) {
        self.status = status;
        self.updated = Utc::now();
    }

    /// Add a tag for categorization
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated = Utc::now();
        }
    }

    /// Set metadata field
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated = Utc::now();
    }

    /// Check if requirement is complete (has description)
    pub fn is_complete(&self) -> bool {
        !self.description.trim().is_empty()
    }

    /// Get priority weight for sorting
    pub fn priority_weight(&self) -> u8 {
        match self.priority {
            Priority::Critical => 4,
            Priority::High => 3,
            Priority::Medium => 2,
            Priority::Low => 1,
        }
    }
}

impl Entity for Requirement {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Requirement name cannot be empty".to_string(),
            ));
        }
        
        if self.description.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Requirement description cannot be empty".to_string(),
            ));
        }


        Ok(())
    }
}

/// Design input derived from requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignInput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub requirement_id: Id,
    pub source: String,
    pub acceptance_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub references: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl DesignInput {
    /// Create a new design input linked to a requirement
    pub fn new(
        name: String,
        description: String,
        requirement_id: Id,
        source: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            requirement_id,
            source,
            acceptance_criteria: Vec::new(),
            constraints: Vec::new(),
            assumptions: Vec::new(),
            references: Vec::new(),
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    /// Add a design constraint
    pub fn add_constraint(&mut self, constraint: String) {
        self.constraints.push(constraint);
        self.updated = Utc::now();
    }

    /// Add an assumption
    pub fn add_assumption(&mut self, assumption: String) {
        self.assumptions.push(assumption);
        self.updated = Utc::now();
    }

    /// Add a reference document
    pub fn add_reference(&mut self, reference: String) {
        self.references.push(reference);
        self.updated = Utc::now();
    }

    /// Add acceptance criteria
    pub fn add_acceptance_criterion(&mut self, criterion: String) {
        self.acceptance_criteria.push(criterion);
        self.updated = Utc::now();
    }
}

impl Entity for DesignInput {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design input name cannot be empty".to_string(),
            ));
        }
        
        if self.description.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design input description cannot be empty".to_string(),
            ));
        }

        if self.source.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design input source cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Design output that satisfies design inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignOutput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub input_ids: Vec<Id>,
    pub output_type: String,
    pub deliverable: String,
    pub location: Option<String>,
    pub version: Option<String>,
    pub approval_status: String,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl DesignOutput {
    /// Create a new design output linked to design inputs
    pub fn new(
        name: String,
        description: String,
        input_ids: Vec<Id>,
        output_type: String,
        deliverable: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            input_ids,
            output_type,
            deliverable,
            location: None,
            version: None,
            approval_status: "Draft".to_string(),
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    /// Update approval status
    pub fn update_approval_status(&mut self, status: String) {
        self.approval_status = status;
        self.updated = Utc::now();
    }

    /// Set the location/path of the deliverable
    pub fn set_location(&mut self, location: String) {
        self.location = Some(location);
        self.updated = Utc::now();
    }

    /// Set the version of the deliverable
    pub fn set_version(&mut self, version: String) {
        self.version = Some(version);
        self.updated = Utc::now();
    }
}

impl Entity for DesignOutput {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design output name cannot be empty".to_string(),
            ));
        }
        
        if self.description.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design output description cannot be empty".to_string(),
            ));
        }

        if self.deliverable.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design output deliverable cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Verification activity that validates design inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub input_ids: Vec<Id>,
    pub verification_type: String,
    pub method: String,
    pub acceptance_criteria: Vec<String>,
    pub status: String,
    pub results: Option<String>,
    pub evidence: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Verification {
    /// Create a new verification activity
    pub fn new(
        name: String,
        description: String,
        input_ids: Vec<Id>,
        verification_type: String,
        method: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            input_ids,
            verification_type,
            method,
            acceptance_criteria: Vec::new(),
            status: "Planned".to_string(),
            results: None,
            evidence: Vec::new(),
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    /// Add acceptance criteria
    pub fn add_acceptance_criterion(&mut self, criterion: String) {
        self.acceptance_criteria.push(criterion);
        self.updated = Utc::now();
    }

    /// Update verification status
    pub fn update_status(&mut self, status: String) {
        self.status = status;
        self.updated = Utc::now();
    }

    /// Set verification results
    pub fn set_results(&mut self, results: String) {
        self.results = Some(results);
        self.updated = Utc::now();
    }

    /// Add evidence document
    pub fn add_evidence(&mut self, evidence: String) {
        self.evidence.push(evidence);
        self.updated = Utc::now();
    }

    /// Check if verification is complete
    pub fn is_complete(&self) -> bool {
        self.status == "Passed" || self.status == "Failed"
    }
}

impl Entity for Verification {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Verification name cannot be empty".to_string(),
            ));
        }
        
        if self.description.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Verification description cannot be empty".to_string(),
            ));
        }

        if self.method.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Verification method cannot be empty".to_string(),
            ));
        }

        // Validate acceptance criteria exist for complete verifications
        if self.is_complete() && self.acceptance_criteria.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Completed verification must have acceptance criteria".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requirement_creation() {
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );

        assert_eq!(req.name, "Test Requirement");
        assert_eq!(req.description, "A test requirement");
        assert_eq!(req.category, RequirementCategory::Functional);
        assert_eq!(req.priority, Priority::High);
        assert_eq!(req.status, RequirementStatus::Draft);
        assert!(!req.is_complete());
    }

    #[test]
    fn test_requirement_validation() {
        let req = Requirement::new(
            "Test".to_string(),
            "Description".to_string(),
            RequirementCategory::Functional,
            Priority::Low,
        );

        assert!(req.validate().is_ok());

        let invalid_req = Requirement::new(
            "".to_string(),
            "Description".to_string(),
            RequirementCategory::Functional,
            Priority::Low,
        );

        assert!(invalid_req.validate().is_err());
    }

    #[test]
    fn test_design_input_creation() {
        let input = DesignInput::new(
            "Test Input".to_string(),
            "A test input".to_string(),
            Id::new(),
            "Test specification".to_string(),
        );

        assert_eq!(input.name, "Test Input");
        assert_eq!(input.source, "Test specification");
        assert!(input.constraints.is_empty());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical.priority_weight() > Priority::High.priority_weight());
        assert!(Priority::High.priority_weight() > Priority::Medium.priority_weight());
        assert!(Priority::Medium.priority_weight() > Priority::Low.priority_weight());
    }
}