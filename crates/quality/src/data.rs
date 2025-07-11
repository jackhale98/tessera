use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: Priority,
    pub status: RequirementStatus,
    pub acceptance_criteria: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub linked_inputs: Vec<Id>,
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
    pub fn new(name: String, description: String, category: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            category,
            priority: Priority::Medium,
            status: RequirementStatus::Draft,
            acceptance_criteria: Vec::new(),
            created: now,
            updated: now,
            linked_inputs: Vec::new(),
        }
    }
    
    pub fn add_input(&mut self, input_id: Id) {
        if !self.linked_inputs.contains(&input_id) {
            self.linked_inputs.push(input_id);
            self.updated = Utc::now();
        }
    }
    
    pub fn remove_input(&mut self, input_id: Id) {
        self.linked_inputs.retain(|&id| id != input_id);
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.category.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignInput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub input_type: String,
    pub source: String,
    pub requirements: Vec<Id>,
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
    pub fn new(name: String, description: String, input_type: String, source: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            input_type,
            source,
            requirements: Vec::new(),
            linked_outputs: Vec::new(),
            created: now,
            updated: now,
        }
    }
    
    pub fn add_requirement(&mut self, req_id: Id) {
        if !self.requirements.contains(&req_id) {
            self.requirements.push(req_id);
            self.updated = Utc::now();
        }
    }
    
    pub fn remove_requirement(&mut self, req_id: Id) {
        self.requirements.retain(|&id| id != req_id);
        self.updated = Utc::now();
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
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.input_type.to_lowercase().contains(&query_lower) ||
        self.source.to_lowercase().contains(&query_lower)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignOutput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub output_type: String,
    pub file_path: Option<String>,
    pub linked_inputs: Vec<Id>,
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
    pub fn new(name: String, description: String, output_type: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            output_type,
            file_path: None,
            linked_inputs: Vec::new(),
            linked_verifications: Vec::new(),
            created: now,
            updated: now,
        }
    }
    
    pub fn add_input(&mut self, input_id: Id) {
        if !self.linked_inputs.contains(&input_id) {
            self.linked_inputs.push(input_id);
            self.updated = Utc::now();
        }
    }
    
    pub fn remove_input(&mut self, input_id: Id) {
        self.linked_inputs.retain(|&id| id != input_id);
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
    pub linked_outputs: Vec<Id>,
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
    pub fn new(name: String, description: String, verification_type: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            verification_type,
            procedure: String::new(),
            responsible_party: String::new(),
            linked_outputs: Vec::new(),
            status: VerificationStatus::Planned,
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
    pub probability: f64,
    pub impact: f64,
    pub risk_score: f64,
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
        if self.probability < 0.0 || self.probability > 1.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Risk probability must be between 0.0 and 1.0".to_string()
            ));
        }
        if self.impact < 0.0 || self.impact > 1.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Risk impact must be between 0.0 and 1.0".to_string()
            ));
        }
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
            probability: 0.5,
            impact: 0.5,
            risk_score: 0.25,
            mitigation_strategy: String::new(),
            status: RiskStatus::Identified,
            owner: String::new(),
            created: now,
            updated: now,
        }
    }
    
    pub fn update_risk_score(&mut self) {
        self.risk_score = self.probability * self.impact;
        self.updated = Utc::now();
    }
    
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.mitigation_strategy.to_lowercase().contains(&query_lower) ||
        self.owner.to_lowercase().contains(&query_lower)
    }
}