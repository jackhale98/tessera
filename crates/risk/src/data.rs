//! Data structures for risk management
//!
//! This module defines the core data types used throughout the risk management system,
//! including risks, design controls, and their relationships.

use serde::{Deserialize, Serialize};
use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
// OrderedFloat removed as it's not used in this simplified implementation

/// Categories for organizing risks following industry standards
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    Design,
    Process,
    Use,
    Software,
}

impl std::fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskCategory::Design => write!(f, "Design"),
            RiskCategory::Process => write!(f, "Process"),
            RiskCategory::Use => write!(f, "Use"),
            RiskCategory::Software => write!(f, "Software"),
        }
    }
}

/// Status of a risk throughout its lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskStatus {
    Identified,
    Analyzed,
    Mitigated,
    Accepted,
    Transferred,
    Closed,
}

impl std::fmt::Display for RiskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskStatus::Identified => write!(f, "Identified"),
            RiskStatus::Analyzed => write!(f, "Analyzed"),
            RiskStatus::Mitigated => write!(f, "Mitigated"),
            RiskStatus::Accepted => write!(f, "Accepted"),
            RiskStatus::Transferred => write!(f, "Transferred"),
            RiskStatus::Closed => write!(f, "Closed"),
        }
    }
}

/// Risk severity levels based on calculated scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Core risk entity following FMEA methodology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: RiskCategory,
    pub status: RiskStatus,
    
    // FMEA-specific fields
    pub failure_mode: Option<String>,
    pub cause_of_failure: Option<String>,
    pub effect_of_failure: Option<String>,
    
    // Risk scoring
    pub probability: i32,
    pub impact: i32,
    pub detectability: Option<i32>,
    pub risk_score: f64,
    pub risk_level: RiskLevel,
    
    // Management fields
    pub mitigation_strategy: Option<String>,
    pub owner: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub residual_risk: Option<f64>,
    
    // Metadata
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Risk {
    /// Create a new risk with default values
    pub fn new(name: String, description: String, category: RiskCategory) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            category,
            status: RiskStatus::Identified,
            failure_mode: None,
            cause_of_failure: None,
            effect_of_failure: None,
            probability: 1,
            impact: 1,
            detectability: None,
            risk_score: 0.0,
            risk_level: RiskLevel::Low,
            mitigation_strategy: None,
            owner: None,
            due_date: None,
            residual_risk: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    /// Set FMEA fields
    pub fn set_fmea_fields(
        &mut self,
        failure_mode: Option<String>,
        cause_of_failure: Option<String>,
        effect_of_failure: Option<String>,
    ) {
        self.failure_mode = failure_mode;
        self.cause_of_failure = cause_of_failure;
        self.effect_of_failure = effect_of_failure;
        self.updated = Utc::now();
    }

    /// Update risk scores and recalculate risk level
    pub fn update_scores(&mut self, probability: i32, impact: i32, detectability: Option<i32>) {
        self.probability = probability;
        self.impact = impact;
        self.detectability = detectability;
        self.recalculate_risk_score();
        self.updated = Utc::now();
    }

    /// Recalculate risk score based on current values
    pub fn recalculate_risk_score(&mut self) {
        // Use simple probability * impact for base score
        let base_score = (self.probability as f64) * (self.impact as f64);
        
        // Apply detectability if available (lower detectability = higher risk)
        self.risk_score = if let Some(detectability) = self.detectability {
            base_score * (11.0 - detectability as f64) / 10.0
        } else {
            base_score
        };
        
        // Determine risk level based on score
        self.risk_level = self.calculate_risk_level();
    }

    /// Calculate risk level based on risk score
    fn calculate_risk_level(&self) -> RiskLevel {
        // These thresholds can be made configurable
        match self.risk_score {
            score if score >= 16.0 => RiskLevel::Critical,
            score if score >= 9.0 => RiskLevel::High,
            score if score >= 4.0 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Update risk status
    pub fn update_status(&mut self, status: RiskStatus) {
        self.status = status;
        self.updated = Utc::now();
    }

    /// Set mitigation strategy
    pub fn set_mitigation_strategy(&mut self, strategy: String) {
        self.mitigation_strategy = Some(strategy);
        self.updated = Utc::now();
    }

    /// Set owner
    pub fn set_owner(&mut self, owner: String) {
        self.owner = Some(owner);
        self.updated = Utc::now();
    }

    /// Set due date
    pub fn set_due_date(&mut self, due_date: DateTime<Utc>) {
        self.due_date = Some(due_date);
        self.updated = Utc::now();
    }

    /// Set residual risk after mitigation
    pub fn set_residual_risk(&mut self, residual_risk: f64) {
        self.residual_risk = Some(residual_risk);
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

    /// Check if risk is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            Utc::now() > due_date && self.status != RiskStatus::Closed
        } else {
            false
        }
    }

    /// Check if risk needs attention (high/critical and not closed)
    pub fn needs_attention(&self) -> bool {
        matches!(self.risk_level, RiskLevel::High | RiskLevel::Critical)
            && !matches!(self.status, RiskStatus::Closed | RiskStatus::Accepted)
    }

    /// Get risk priority weight for sorting
    pub fn priority_weight(&self) -> u8 {
        match self.risk_level {
            RiskLevel::Critical => 4,
            RiskLevel::High => 3,
            RiskLevel::Medium => 2,
            RiskLevel::Low => 1,
        }
    }

    /// Calculate risk reduction percentage if residual risk is set
    pub fn risk_reduction_percentage(&self) -> Option<f64> {
        self.residual_risk.map(|residual| {
            if self.risk_score > 0.0 {
                ((self.risk_score - residual) / self.risk_score) * 100.0
            } else {
                0.0
            }
        })
    }
}

impl Entity for Risk {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Risk name cannot be empty".to_string(),
            ));
        }
        
        if self.description.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Risk description cannot be empty".to_string(),
            ));
        }

        // Validate probability and impact ranges
        if self.probability < 1 || self.probability > 5 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Probability must be between 1 and 5".to_string(),
            ));
        }

        if self.impact < 1 || self.impact > 5 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Impact must be between 1 and 5".to_string(),
            ));
        }

        // Validate detectability if provided
        if let Some(detectability) = self.detectability {
            if detectability < 1 || detectability > 10 {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Detectability must be between 1 and 10".to_string(),
                ));
            }
        }

        // Validate residual risk if provided
        if let Some(residual) = self.residual_risk {
            if residual < 0.0 || residual > self.risk_score {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Residual risk must be between 0 and original risk score".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Type of design control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlType {
    Preventive,
    Detective,
    Corrective,
    Compensating,
    Directive,
}

impl std::fmt::Display for ControlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlType::Preventive => write!(f, "Preventive"),
            ControlType::Detective => write!(f, "Detective"),
            ControlType::Corrective => write!(f, "Corrective"),
            ControlType::Compensating => write!(f, "Compensating"),
            ControlType::Directive => write!(f, "Directive"),
        }
    }
}

/// Implementation status of design control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlStatus {
    Planned,
    InProgress,
    Implemented,
    Verified,
    Ineffective,
}

impl std::fmt::Display for ControlStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlStatus::Planned => write!(f, "Planned"),
            ControlStatus::InProgress => write!(f, "In Progress"),
            ControlStatus::Implemented => write!(f, "Implemented"),
            ControlStatus::Verified => write!(f, "Verified"),
            ControlStatus::Ineffective => write!(f, "Ineffective"),
        }
    }
}

/// Design control for risk mitigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignControl {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub control_type: ControlType,
    pub status: ControlStatus,
    pub risk_id: Id,
    
    // Implementation details
    pub implementation_approach: Option<String>,
    pub responsible_party: Option<String>,
    pub target_completion: Option<DateTime<Utc>>,
    pub actual_completion: Option<DateTime<Utc>>,
    
    // Effectiveness
    pub effectiveness_rating: Option<i32>,
    pub verification_method: Option<String>,
    pub verification_results: Option<String>,
    
    // Cross-module integration
    pub design_output_id: Option<tessera_core::Id>,
    pub verification_id: Option<tessera_core::Id>,
    
    // Metadata
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl DesignControl {
    /// Create a new design control linked to a risk
    pub fn new(
        name: String,
        description: String,
        control_type: ControlType,
        risk_id: Id,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            control_type,
            status: ControlStatus::Planned,
            risk_id,
            implementation_approach: None,
            responsible_party: None,
            target_completion: None,
            actual_completion: None,
            effectiveness_rating: None,
            verification_method: None,
            verification_results: None,
            design_output_id: None,
            verification_id: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    /// Update control status
    pub fn update_status(&mut self, status: ControlStatus) {
        self.status = status;
        if status == ControlStatus::Implemented {
            self.actual_completion = Some(Utc::now());
        }
        self.updated = Utc::now();
    }

    /// Set implementation details
    pub fn set_implementation_details(
        &mut self,
        approach: Option<String>,
        responsible_party: Option<String>,
        target_completion: Option<DateTime<Utc>>,
    ) {
        self.implementation_approach = approach;
        self.responsible_party = responsible_party;
        self.target_completion = target_completion;
        self.updated = Utc::now();
    }

    /// Set effectiveness rating (1-5 scale)
    pub fn set_effectiveness_rating(&mut self, rating: i32) {
        self.effectiveness_rating = Some(rating);
        self.updated = Utc::now();
    }

    /// Set verification details
    pub fn set_verification_details(
        &mut self,
        method: Option<String>,
        results: Option<String>,
    ) {
        self.verification_method = method;
        self.verification_results = results;
        self.updated = Utc::now();
    }

    /// Link to design output from requirements module
    pub fn link_to_design_output(&mut self, design_output_id: tessera_core::Id) {
        self.design_output_id = Some(design_output_id);
        self.updated = Utc::now();
    }

    /// Link to verification from verification module
    pub fn link_to_verification(&mut self, verification_id: tessera_core::Id) {
        self.verification_id = Some(verification_id);
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

    /// Check if control is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(target) = self.target_completion {
            Utc::now() > target && !matches!(self.status, ControlStatus::Implemented | ControlStatus::Verified)
        } else {
            false
        }
    }

    /// Check if control is effective
    pub fn is_effective(&self) -> bool {
        match self.effectiveness_rating {
            Some(rating) => rating >= 3,
            None => false,
        }
    }

    /// Get implementation duration if completed
    pub fn implementation_duration(&self) -> Option<chrono::Duration> {
        match (self.actual_completion, self.target_completion) {
            (Some(actual), Some(target)) => Some(actual - target),
            _ => None,
        }
    }
}

impl Entity for DesignControl {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design control name cannot be empty".to_string(),
            ));
        }
        
        if self.description.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design control description cannot be empty".to_string(),
            ));
        }

        // Validate effectiveness rating if provided
        if let Some(rating) = self.effectiveness_rating {
            if rating < 1 || rating > 5 {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Effectiveness rating must be between 1 and 5".to_string(),
                ));
            }
        }

        // Validate completion dates
        if let (Some(target), Some(actual)) = (self.target_completion, self.actual_completion) {
            if actual < target {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Actual completion cannot be before target completion".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Risk assessment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentConfig {
    pub probability_scale: (i32, i32),
    pub impact_scale: (i32, i32),
    pub detectability_scale: Option<(i32, i32)>,
    pub thresholds: RiskThresholds,
}

impl RiskAssessmentConfig {
    /// Create default assessment configuration
    pub fn default() -> Self {
        Self {
            probability_scale: (1, 5),
            impact_scale: (1, 5),
            detectability_scale: Some((1, 10)),
            thresholds: RiskThresholds::default(),
        }
    }

    /// Normalize a score to 0-1 range
    pub fn normalize_score(&self, score: i32, scale: (i32, i32)) -> f64 {
        let (min, max) = scale;
        (score - min) as f64 / (max - min) as f64
    }

    /// Validate score against scale
    pub fn validate_score(&self, score: i32, scale: (i32, i32)) -> bool {
        let (min, max) = scale;
        score >= min && score <= max
    }
}

/// Risk level thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    pub low_threshold: f64,
    pub medium_threshold: f64,
    pub high_threshold: f64,
}

impl RiskThresholds {
    /// Create default thresholds
    pub fn default() -> Self {
        Self {
            low_threshold: 4.0,
            medium_threshold: 9.0,
            high_threshold: 16.0,
        }
    }

    /// Determine risk level from score
    pub fn risk_level_from_score(&self, score: f64) -> RiskLevel {
        if score >= self.high_threshold {
            RiskLevel::Critical
        } else if score >= self.medium_threshold {
            RiskLevel::High
        } else if score >= self.low_threshold {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_creation() {
        let risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );

        assert_eq!(risk.name, "Test Risk");
        assert_eq!(risk.description, "A test risk");
        assert_eq!(risk.category, RiskCategory::Technical);
        assert_eq!(risk.status, RiskStatus::Identified);
        assert_eq!(risk.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_risk_scoring() {
        let mut risk = Risk::new(
            "High Risk".to_string(),
            "A high risk".to_string(),
            RiskCategory::Safety,
        );

        risk.update_scores(5, 4, Some(2));
        assert_eq!(risk.probability, 5);
        assert_eq!(risk.impact, 4);
        assert_eq!(risk.detectability, Some(2));
        assert!(risk.risk_score > 0.0);
        assert!(matches!(risk.risk_level, RiskLevel::High | RiskLevel::Critical));
    }

    #[test]
    fn test_risk_validation() {
        let mut risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );

        assert!(risk.validate().is_ok());

        // Test invalid probability
        risk.probability = 10;
        assert!(risk.validate().is_err());

        risk.probability = 3;
        assert!(risk.validate().is_ok());

        // Test invalid impact
        risk.impact = 0;
        assert!(risk.validate().is_err());
    }

    #[test]
    fn test_design_control_creation() {
        let risk_id = Id::new();
        let control = DesignControl::new(
            "Test Control".to_string(),
            "A test control".to_string(),
            ControlType::Preventive,
            risk_id,
        );

        assert_eq!(control.name, "Test Control");
        assert_eq!(control.control_type, ControlType::Preventive);
        assert_eq!(control.risk_id, risk_id);
        assert_eq!(control.status, ControlStatus::Planned);
    }

    #[test]
    fn test_control_status_update() {
        let mut control = DesignControl::new(
            "Test Control".to_string(),
            "A test control".to_string(),
            ControlType::Preventive,
            Id::new(),
        );

        control.update_status(ControlStatus::Implemented);
        assert_eq!(control.status, ControlStatus::Implemented);
        assert!(control.actual_completion.is_some());
    }

    #[test]
    fn test_risk_level_calculation() {
        let thresholds = RiskThresholds::default();
        
        assert_eq!(thresholds.risk_level_from_score(2.0), RiskLevel::Low);
        assert_eq!(thresholds.risk_level_from_score(6.0), RiskLevel::Medium);
        assert_eq!(thresholds.risk_level_from_score(12.0), RiskLevel::High);
        assert_eq!(thresholds.risk_level_from_score(20.0), RiskLevel::Critical);
    }

    #[test]
    fn test_risk_reduction_calculation() {
        let mut risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );

        risk.update_scores(4, 3, None);
        let original_score = risk.risk_score;
        
        risk.set_residual_risk(original_score * 0.5);
        
        let reduction = risk.risk_reduction_percentage().unwrap();
        assert!((reduction - 50.0).abs() < 0.1);
    }
}