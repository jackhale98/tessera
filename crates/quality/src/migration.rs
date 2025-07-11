use crate::data::*;
use tessera_core::{Id, Result, DesignTrackError};
use std::path::Path;
use std::fs;
use ron::de::from_str;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyRequirement {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: LegacyRequirementCategory,
    pub priority: Priority,
    pub status: RequirementStatus,
    pub due_date: Option<chrono::NaiveDate>,
    pub acceptance_criteria: Vec<String>,
    pub risk_score: Option<f64>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub links: Vec<Id>,
    pub traced_to: Vec<Id>,
    pub traced_from: Vec<Id>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegacyRequirementCategory {
    Functional,
    Performance,
    Safety,
    Regulatory,
    Usability,
    Reliability,
    Maintainability,
    Environmental,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyDesignInput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub input_type: LegacyInputType,
    pub source: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub requirements: Vec<Id>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegacyInputType {
    Specification,
    Standard,
    Regulation,
    CustomerRequirement,
    MarketResearch,
    TechnicalReport,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyDesignOutput {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub output_type: LegacyOutputType,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub linked_inputs: Vec<Id>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegacyOutputType {
    Drawing,
    Calculation,
    Specification,
    Report,
    Model,
    Prototype,
    TestPlan,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyDesignControl {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub control_type: LegacyControlType,
    pub status: VerificationStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub linked_outputs: Vec<Id>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegacyControlType {
    Review,
    Inspection,
    Test,
    Verification,
    Validation,
    Approval,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyRisk {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub category: LegacyRiskCategory,
    pub probability: f64,
    pub impact: f64,
    pub risk_score: f64,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegacyRiskCategory {
    Technical,
    Schedule,
    Cost,
    Quality,
    Safety,
    Regulatory,
    Market,
    Resource,
    Other,
}

impl From<LegacyRequirementCategory> for String {
    fn from(category: LegacyRequirementCategory) -> Self {
        match category {
            LegacyRequirementCategory::Functional => "Functional".to_string(),
            LegacyRequirementCategory::Performance => "Performance".to_string(),
            LegacyRequirementCategory::Safety => "Safety".to_string(),
            LegacyRequirementCategory::Regulatory => "Regulatory".to_string(),
            LegacyRequirementCategory::Usability => "Usability".to_string(),
            LegacyRequirementCategory::Reliability => "Reliability".to_string(),
            LegacyRequirementCategory::Maintainability => "Maintainability".to_string(),
            LegacyRequirementCategory::Environmental => "Environmental".to_string(),
            LegacyRequirementCategory::Other => "Other".to_string(),
        }
    }
}

impl From<LegacyInputType> for String {
    fn from(input_type: LegacyInputType) -> Self {
        match input_type {
            LegacyInputType::Specification => "Specification".to_string(),
            LegacyInputType::Standard => "Standard".to_string(),
            LegacyInputType::Regulation => "Regulation".to_string(),
            LegacyInputType::CustomerRequirement => "Customer Requirement".to_string(),
            LegacyInputType::MarketResearch => "Market Research".to_string(),
            LegacyInputType::TechnicalReport => "Technical Report".to_string(),
            LegacyInputType::Other => "Other".to_string(),
        }
    }
}

impl From<LegacyOutputType> for String {
    fn from(output_type: LegacyOutputType) -> Self {
        match output_type {
            LegacyOutputType::Drawing => "Drawing".to_string(),
            LegacyOutputType::Calculation => "Calculation".to_string(),
            LegacyOutputType::Specification => "Specification".to_string(),
            LegacyOutputType::Report => "Report".to_string(),
            LegacyOutputType::Model => "Model".to_string(),
            LegacyOutputType::Prototype => "Prototype".to_string(),
            LegacyOutputType::TestPlan => "Test Plan".to_string(),
            LegacyOutputType::Other => "Other".to_string(),
        }
    }
}

impl From<LegacyControlType> for String {
    fn from(control_type: LegacyControlType) -> Self {
        match control_type {
            LegacyControlType::Review => "Review".to_string(),
            LegacyControlType::Inspection => "Inspection".to_string(),
            LegacyControlType::Test => "Test".to_string(),
            LegacyControlType::Verification => "Verification".to_string(),
            LegacyControlType::Validation => "Validation".to_string(),
            LegacyControlType::Approval => "Approval".to_string(),
            LegacyControlType::Other => "Other".to_string(),
        }
    }
}

impl From<LegacyRiskCategory> for String {
    fn from(category: LegacyRiskCategory) -> Self {
        match category {
            LegacyRiskCategory::Technical => "Technical".to_string(),
            LegacyRiskCategory::Schedule => "Schedule".to_string(),
            LegacyRiskCategory::Cost => "Cost".to_string(),
            LegacyRiskCategory::Quality => "Quality".to_string(),
            LegacyRiskCategory::Safety => "Safety".to_string(),
            LegacyRiskCategory::Regulatory => "Regulatory".to_string(),
            LegacyRiskCategory::Market => "Market".to_string(),
            LegacyRiskCategory::Resource => "Resource".to_string(),
            LegacyRiskCategory::Other => "Other".to_string(),
        }
    }
}

impl From<LegacyRequirement> for Requirement {
    fn from(legacy: LegacyRequirement) -> Self {
        Requirement {
            id: legacy.id,
            name: legacy.name,
            description: legacy.description,
            source: "Legacy Migration".to_string(), // Default source for migrated data
            category: legacy.category.into(),
            priority: legacy.priority,
            status: legacy.status,
            created: legacy.created,
            updated: legacy.updated,
        }
    }
}

impl From<LegacyDesignInput> for DesignInput {
    fn from(legacy: LegacyDesignInput) -> Self {
        // For migration, we'll assign to the first requirement if any exist
        let requirement_id = legacy.requirements.first().copied()
            .unwrap_or_else(|| tessera_core::Id::new()); // Create placeholder if none
        
        DesignInput {
            id: legacy.id,
            name: legacy.name,
            description: legacy.description,
            input_type: legacy.input_type.into(),
            requirement_id,
            acceptance_criteria: Vec::new(), // Will be set separately in migration
            linked_outputs: Vec::new(), // Initialize empty - will be rebuilt from bidirectional links
            created: legacy.created,
            updated: legacy.updated,
        }
    }
}

impl From<LegacyDesignOutput> for DesignOutput {
    fn from(legacy: LegacyDesignOutput) -> Self {
        // For migration, we'll assign to the first input if any exist
        let input_id = legacy.linked_inputs.first().copied()
            .unwrap_or_else(|| tessera_core::Id::new()); // Create placeholder if none
        
        DesignOutput {
            id: legacy.id,
            name: legacy.name,
            description: legacy.description,
            output_type: legacy.output_type.into(),
            file_path: None, // Initialize empty - not available in legacy data
            input_id,
            linked_verifications: Vec::new(), // Initialize empty - will be rebuilt from bidirectional links
            created: legacy.created,
            updated: legacy.updated,
        }
    }
}

impl From<LegacyDesignControl> for Verification {
    fn from(legacy: LegacyDesignControl) -> Self {
        // For migration, we'll assign to the first output if any exist
        let output_id = legacy.linked_outputs.first().copied()
            .unwrap_or_else(|| tessera_core::Id::new()); // Create placeholder if none
        
        Verification {
            id: legacy.id,
            name: legacy.name,
            description: legacy.description,
            verification_type: legacy.control_type.into(),
            procedure: String::new(), // Initialize empty - not available in legacy data
            responsible_party: String::new(), // Initialize empty - not available in legacy data
            output_id,
            status: legacy.status,
            created: legacy.created,
            updated: legacy.updated,
        }
    }
}

impl From<LegacyRisk> for Risk {
    fn from(legacy: LegacyRisk) -> Self {
        // Convert legacy 0.0-1.0 range to 1-5 range for migration
        let probability = ((legacy.probability * 4.0) + 1.0).round() as i32;
        let impact = ((legacy.impact * 4.0) + 1.0).round() as i32;
        
        Risk {
            id: legacy.id,
            name: legacy.name,
            description: legacy.description,
            category: legacy.category.into(),
            failure_mode: String::new(), // Initialize empty - not available in legacy data
            cause_of_failure: String::new(), // Initialize empty - not available in legacy data
            effect_of_failure: String::new(), // Initialize empty - not available in legacy data
            reference: None, // Initialize empty - not available in legacy data
            probability: probability.clamp(1, 5),
            impact: impact.clamp(1, 5),
            risk_score: legacy.risk_score,
            mitigation_strategy: String::new(), // Initialize empty - not available in legacy data
            owner: String::new(), // Initialize empty - not available in legacy data
            status: RiskStatus::Identified, // Default status for legacy risks
            created: legacy.created,
            updated: legacy.updated,
        }
    }
}

pub fn migrate_quality_data(quality_dir: &Path) -> Result<()> {
    println!("Migrating quality data from legacy format...");
    
    let requirements_file = quality_dir.join("requirements.ron");
    let inputs_file = quality_dir.join("inputs.ron");
    let outputs_file = quality_dir.join("outputs.ron");
    let controls_file = quality_dir.join("controls.ron");
    let verifications_file = quality_dir.join("verifications.ron");
    let risks_file = quality_dir.join("risks.ron");
    
    // Migrate requirements
    if requirements_file.exists() {
        if let Ok(content) = fs::read_to_string(&requirements_file) {
            if let Ok(legacy_requirements) = from_str::<Vec<LegacyRequirement>>(&content) {
                let new_requirements: Vec<Requirement> = legacy_requirements.into_iter().map(Into::into).collect();
                let new_content = ron::ser::to_string_pretty(&new_requirements, ron::ser::PrettyConfig::default())
                    .map_err(|e| DesignTrackError::Module(format!("Serialization error: {}", e)))?;
                fs::write(&requirements_file, new_content)?;
                println!("Migrated {} requirements", new_requirements.len());
            }
        }
    }
    
    // Migrate inputs
    if inputs_file.exists() {
        if let Ok(content) = fs::read_to_string(&inputs_file) {
            if let Ok(legacy_inputs) = from_str::<Vec<LegacyDesignInput>>(&content) {
                let new_inputs: Vec<DesignInput> = legacy_inputs.into_iter().map(Into::into).collect();
                let new_content = ron::ser::to_string_pretty(&new_inputs, ron::ser::PrettyConfig::default())
                    .map_err(|e| DesignTrackError::Module(format!("Serialization error: {}", e)))?;
                fs::write(&inputs_file, new_content)?;
                println!("Migrated {} design inputs", new_inputs.len());
            }
        }
    }
    
    // Migrate outputs
    if outputs_file.exists() {
        if let Ok(content) = fs::read_to_string(&outputs_file) {
            if let Ok(legacy_outputs) = from_str::<Vec<LegacyDesignOutput>>(&content) {
                let new_outputs: Vec<DesignOutput> = legacy_outputs.into_iter().map(Into::into).collect();
                let new_content = ron::ser::to_string_pretty(&new_outputs, ron::ser::PrettyConfig::default())
                    .map_err(|e| DesignTrackError::Module(format!("Serialization error: {}", e)))?;
                fs::write(&outputs_file, new_content)?;
                println!("Migrated {} design outputs", new_outputs.len());
            }
        }
    }
    
    // Migrate controls to verifications
    if controls_file.exists() {
        if let Ok(content) = fs::read_to_string(&controls_file) {
            if let Ok(legacy_controls) = from_str::<Vec<LegacyDesignControl>>(&content) {
                let new_verifications: Vec<Verification> = legacy_controls.into_iter().map(Into::into).collect();
                let new_content = ron::ser::to_string_pretty(&new_verifications, ron::ser::PrettyConfig::default())
                    .map_err(|e| DesignTrackError::Module(format!("Serialization error: {}", e)))?;
                fs::write(&verifications_file, new_content)?;
                // Remove old controls file
                fs::remove_file(&controls_file)?;
                println!("Migrated {} controls to verifications", new_verifications.len());
            }
        }
    }
    
    // Migrate risks
    if risks_file.exists() {
        if let Ok(content) = fs::read_to_string(&risks_file) {
            if let Ok(legacy_risks) = from_str::<Vec<LegacyRisk>>(&content) {
                let new_risks: Vec<Risk> = legacy_risks.into_iter().map(Into::into).collect();
                let new_content = ron::ser::to_string_pretty(&new_risks, ron::ser::PrettyConfig::default())
                    .map_err(|e| DesignTrackError::Module(format!("Serialization error: {}", e)))?;
                fs::write(&risks_file, new_content)?;
                println!("Migrated {} risks", new_risks.len());
            }
        }
    }
    
    println!("Migration completed successfully!");
    Ok(())
}