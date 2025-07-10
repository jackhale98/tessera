use tessera_core::{Entity, Id, Link, Linkable, Result};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: RequirementCategory,
    pub priority: Priority,
    pub status: RequirementStatus,
    pub acceptance_criteria: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub links: Vec<Link>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementCategory {
    Functional,
    Performance,
    Safety,
    Regulatory,
    Usability,
    Reliability,
    Maintainability,
    Environmental,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementStatus {
    Draft,
    Approved,
    Implemented,
    Verified,
    Deprecated,
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

impl Linkable for Requirement {
    fn get_links(&self) -> Vec<Id> {
        self.links.iter().map(|link| link.target_id).collect()
    }
    
    fn add_link(&mut self, target_id: Id) -> Result<()> {
        let link = Link::new(target_id, "reference".to_string());
        self.links.push(link);
        self.updated = Utc::now();
        Ok(())
    }
    
    fn remove_link(&mut self, target_id: Id) -> Result<()> {
        self.links.retain(|link| link.target_id != target_id);
        self.updated = Utc::now();
        Ok(())
    }
    
    fn validate_links(&self, _resolver: &dyn tessera_core::LinkResolver) -> Result<()> {
        Ok(())
    }
}

impl Requirement {
    pub fn new(name: String, description: String, category: RequirementCategory) -> Self {
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
            links: Vec::new(),
            metadata: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignInput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub input_type: InputType,
    pub source: String,
    pub requirements: Vec<Id>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    Specification,
    Standard,
    Regulation,
    CustomerRequirement,
    MarketResearch,
    TechnicalReport,
    Other(String),
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
    pub fn new(name: String, description: String, input_type: InputType, source: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            input_type,
            source,
            requirements: Vec::new(),
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignOutput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub output_type: OutputType,
    pub file_path: Option<String>,
    pub inputs: Vec<Id>,
    pub requirements: Vec<Id>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Drawing,
    Calculation,
    Specification,
    Report,
    Model,
    Prototype,
    TestPlan,
    Other(String),
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
    pub fn new(name: String, description: String, output_type: OutputType) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            output_type,
            file_path: None,
            inputs: Vec::new(),
            requirements: Vec::new(),
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignControl {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub control_type: ControlType,
    pub frequency: ControlFrequency,
    pub responsible_party: String,
    pub procedure: String,
    pub outputs: Vec<Id>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlType {
    Review,
    Inspection,
    Test,
    Verification,
    Validation,
    Approval,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlFrequency {
    OneTime,
    PerBatch,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    AsNeeded,
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
    pub fn new(name: String, description: String, control_type: ControlType) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            control_type,
            frequency: ControlFrequency::AsNeeded,
            responsible_party: String::new(),
            procedure: String::new(),
            outputs: Vec::new(),
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: RiskCategory,
    pub probability: f64,
    pub impact: f64,
    pub risk_score: f64,
    pub mitigation_strategy: String,
    pub status: RiskStatus,
    pub owner: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    Technical,
    Schedule,
    Cost,
    Quality,
    Safety,
    Regulatory,
    Market,
    Resource,
    Other(String),
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
    pub fn new(name: String, description: String, category: RiskCategory) -> Self {
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
            metadata: IndexMap::new(),
        }
    }
    
    pub fn update_risk_score(&mut self) {
        self.risk_score = self.probability * self.impact;
        self.updated = Utc::now();
    }
}