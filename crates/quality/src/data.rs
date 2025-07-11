use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub source: String, // Source of the requirement (customer, regulation, standard, etc.)
    pub category: String,
    pub priority: Priority,
    pub status: RequirementStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementStatus {
    Draft,
    Approved,
    Verified,
    Closed,
}

impl Entity for Requirement {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Requirement name cannot be empty".to_string()
            ));
        }
        if self.description.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Requirement description cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}


impl Requirement {
    pub fn new(name: String, description: String, source: String, category: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            source,
            category,
            priority: Priority::Medium,
            status: RequirementStatus::Draft,
            created: now,
            updated: now,
        }
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.source.to_lowercase().contains(&query_lower) ||
        self.category.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignInput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub input_type: String,
    pub requirement_id: Id, // Single requirement this input addresses
    pub acceptance_criteria: Vec<String>, // Moved from requirements
    pub linked_outputs: Vec<Id>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}


impl Entity for DesignInput {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design input name cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

impl DesignInput {
    pub fn new(name: String, description: String, input_type: String, requirement_id: Id) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            input_type,
            requirement_id,
            acceptance_criteria: Vec::new(),
            linked_outputs: Vec::new(),
            created: now,
            updated: now,
        }
    }
    
    pub fn add_output(&mut self, output_id: Id) {
        if !self.linked_outputs.contains(&output_id) {
            self.linked_outputs.push(output_id);
            self.updated = Utc::now();
        }
    }
    
    pub fn remove_output(&mut self, output_id: Id) {
        self.linked_outputs.retain(|&id| id != output_id);
        self.updated = Utc::now();
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.input_type.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignOutput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub output_type: String,
    pub file_path: Option<String>,
    pub input_id: Id, // Single design input this output implements
    pub linked_verifications: Vec<Id>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}


impl Entity for DesignOutput {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design output name cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

impl DesignOutput {
    pub fn new(name: String, description: String, output_type: String, input_id: Id) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            output_type,
            file_path: None,
            input_id,
            linked_verifications: Vec::new(),
            created: now,
            updated: now,
        }
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated = Utc::now();
    }
    
    pub fn add_verification(&mut self, verification_id: Id) {
        if !self.linked_verifications.contains(&verification_id) {
            self.linked_verifications.push(verification_id);
            self.updated = Utc::now();
        }
    }
    
    pub fn remove_verification(&mut self, verification_id: Id) {
        self.linked_verifications.retain(|&id| id != verification_id);
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.output_type.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub verification_type: String,
    pub procedure: String,
    pub responsible_party: String,
    pub output_id: Id, // Single design output this verification tests
    pub status: VerificationStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Planned,
    InProgress,
    Passed,
    Failed,
    Deferred,
}


impl Entity for Verification {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Verification name cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

impl Verification {
    pub fn new(name: String, description: String, verification_type: String, output_id: Id) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            verification_type,
            procedure: String::new(),
            responsible_party: String::new(),
            output_id,
            status: VerificationStatus::Planned,
            created: now,
            updated: now,
        }
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.verification_type.to_lowercase().contains(&query_lower) ||
        self.procedure.to_lowercase().contains(&query_lower) ||
        self.responsible_party.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: String,
    pub failure_mode: String,      // What fails
    pub cause_of_failure: String,  // Why it fails  
    pub effect_of_failure: String, // Impact of failure
    pub reference: Option<String>, // Reference to source document/analysis
    pub probability: i32,          // Using configured range values
    pub impact: i32,               // Using configured range values
    pub risk_score: f64,           // Product of normalized probability × impact
    pub mitigation_strategy: String,
    pub status: RiskStatus,
    pub owner: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskStatus {
    Identified,
    Analyzed,
    Mitigated,
    Accepted,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Entity for Risk {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Risk name cannot be empty".to_string()
            ));
        }
        // Validation for probability and impact will be done against configured ranges
        Ok(())
    }
}

impl Risk {
    pub fn new(name: String, description: String, category: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            category,
            failure_mode: String::new(),
            cause_of_failure: String::new(),
            effect_of_failure: String::new(),
            reference: None,
            probability: 1, // Default to minimum value of typical 1-5 range
            impact: 1,      // Default to minimum value of typical 1-5 range
            risk_score: 0.0,
            mitigation_strategy: String::new(),
            status: RiskStatus::Identified,
            owner: String::new(),
            created: now,
            updated: now,
        }
    }
    
    pub fn update_risk_score(&mut self, prob_config: &tessera_core::RiskScoringConfig, impact_config: &tessera_core::RiskScoringConfig) {
        let norm_prob = prob_config.normalize_to_0_1(self.probability);
        let norm_impact = impact_config.normalize_to_0_1(self.impact);
        self.risk_score = norm_prob * norm_impact;
        self.updated = Utc::now();
    }
    
    pub fn update_risk_score_with_category(&mut self, prob_config: &tessera_core::RiskScoringConfig, impact_config: &tessera_core::RiskScoringConfig, thresholds: &tessera_core::RiskToleranceThresholds) -> tessera_core::RiskCategory {
        self.update_risk_score(prob_config, impact_config);
        thresholds.categorize_risk(self.risk_score)
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.failure_mode.to_lowercase().contains(&query_lower) ||
        self.cause_of_failure.to_lowercase().contains(&query_lower) ||
        self.effect_of_failure.to_lowercase().contains(&query_lower) ||
        self.mitigation_strategy.to_lowercase().contains(&query_lower) ||
        self.owner.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignControl {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub control_type: String,
    pub implementation: String,
    pub risk_id: Id, // Risk this control addresses
    pub status: ControlStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlStatus {
    Planned,
    Implemented,
    Verified,
    Active,
    Inactive,
}

impl Entity for DesignControl {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Design control name cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

impl DesignControl {
    pub fn new(name: String, description: String, control_type: String, risk_id: Id) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            control_type,
            implementation: String::new(),
            risk_id,
            status: ControlStatus::Planned,
            created: now,
            updated: now,
        }
    }
    
    pub fn update_timestamp(&mut self) {
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.control_type.to_lowercase().contains(&query_lower) ||
        self.implementation.to_lowercase().contains(&query_lower)
    }
}

