//! Validation utilities for requirements management
//!
//! This module provides validation functions and rules for requirements
//! and related entities to ensure data quality and consistency.

use crate::data::*;
use crate::repository::RequirementsRepository;
use tessera_core::{Id, Result};
use std::collections::HashMap;

/// Validation rule for requirements
pub trait ValidationRule<T> {
    fn validate(&self, entity: &T, repository: &RequirementsRepository) -> Result<()>;
    fn name(&self) -> &'static str;
}

/// Validates that requirement names are unique
pub struct UniqueNameRule;

impl ValidationRule<Requirement> for UniqueNameRule {
    fn validate(&self, entity: &Requirement, repository: &RequirementsRepository) -> Result<()> {
        let duplicate_count = repository.get_requirements()
            .iter()
            .filter(|req| req.name == entity.name && req.id != entity.id)
            .count();
            
        if duplicate_count > 0 {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Requirement name '{}' is not unique", entity.name)
            ));
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "unique_name"
    }
}

/// Validates that requirements have meaningful acceptance criteria
pub struct MeaningfulCriteriaRule;

impl ValidationRule<Requirement> for MeaningfulCriteriaRule {
    fn validate(&self, entity: &Requirement, _repository: &RequirementsRepository) -> Result<()> {
        // Basic requirement validation - description length etc. are handled by Entity::validate()
        // Additional business logic validation can go here
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "meaningful_criteria"
    }
}

/// Validates that design inputs have sufficient detail
pub struct SufficientDetailRule;

impl ValidationRule<DesignInput> for SufficientDetailRule {
    fn validate(&self, entity: &DesignInput, _repository: &RequirementsRepository) -> Result<()> {
        if entity.source.len() < 20 {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Design input '{}' source is too brief", entity.name)
            ));
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "sufficient_detail"
    }
}

/// Validates that design outputs have proper approval workflow
pub struct ApprovalWorkflowRule;

impl ValidationRule<DesignOutput> for ApprovalWorkflowRule {
    fn validate(&self, entity: &DesignOutput, _repository: &RequirementsRepository) -> Result<()> {
        let valid_statuses = vec!["Draft", "Under Review", "Approved", "Released", "Obsolete"];
        
        if !valid_statuses.contains(&entity.approval_status.as_str()) {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Invalid approval status '{}' for design output '{}'", 
                       entity.approval_status, entity.name)
            ));
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "approval_workflow"
    }
}

/// Validates that verifications have proper test methods
pub struct TestMethodRule;

impl ValidationRule<Verification> for TestMethodRule {
    fn validate(&self, entity: &Verification, _repository: &RequirementsRepository) -> Result<()> {
        let valid_methods = vec![
            "Automated Test", "Manual Test", "Code Review", "Design Review",
            "Inspection", "Analysis", "Demonstration", "Simulation"
        ];
        
        if !valid_methods.contains(&entity.method.as_str()) {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Verification method '{}' should be one of: {}", 
                       entity.method, valid_methods.join(", "))
            ));
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "test_method"
    }
}

/// Comprehensive validator for requirements repository
pub struct RequirementsValidator {
    requirement_rules: Vec<Box<dyn ValidationRule<Requirement>>>,
    input_rules: Vec<Box<dyn ValidationRule<DesignInput>>>,
    output_rules: Vec<Box<dyn ValidationRule<DesignOutput>>>,
    verification_rules: Vec<Box<dyn ValidationRule<Verification>>>,
}

impl RequirementsValidator {
    /// Create a new validator with default rules
    pub fn new() -> Self {
        Self {
            requirement_rules: vec![
                Box::new(UniqueNameRule),
                Box::new(MeaningfulCriteriaRule),
            ],
            input_rules: vec![
                Box::new(SufficientDetailRule),
            ],
            output_rules: vec![
                Box::new(ApprovalWorkflowRule),
            ],
            verification_rules: vec![
                Box::new(TestMethodRule),
            ],
        }
    }
    
    /// Add a custom requirement validation rule
    pub fn add_requirement_rule(&mut self, rule: Box<dyn ValidationRule<Requirement>>) {
        self.requirement_rules.push(rule);
    }
    
    /// Add a custom design input validation rule
    pub fn add_input_rule(&mut self, rule: Box<dyn ValidationRule<DesignInput>>) {
        self.input_rules.push(rule);
    }
    
    /// Add a custom design output validation rule
    pub fn add_output_rule(&mut self, rule: Box<dyn ValidationRule<DesignOutput>>) {
        self.output_rules.push(rule);
    }
    
    /// Add a custom verification validation rule
    pub fn add_verification_rule(&mut self, rule: Box<dyn ValidationRule<Verification>>) {
        self.verification_rules.push(rule);
    }
    
    /// Validate all entities in the repository
    pub fn validate_repository(&self, repository: &RequirementsRepository) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Validate requirements
        for requirement in repository.get_requirements() {
            for rule in &self.requirement_rules {
                if let Err(e) = rule.validate(requirement, repository) {
                    report.add_error(requirement.id, rule.name(), e.to_string());
                }
            }
        }
        
        // Validate design inputs
        for input in repository.get_design_inputs() {
            for rule in &self.input_rules {
                if let Err(e) = rule.validate(input, repository) {
                    report.add_error(input.id, rule.name(), e.to_string());
                }
            }
        }
        
        // Validate design outputs
        for output in repository.get_design_outputs() {
            for rule in &self.output_rules {
                if let Err(e) = rule.validate(output, repository) {
                    report.add_error(output.id, rule.name(), e.to_string());
                }
            }
        }
        
        // Validate verifications
        for verification in repository.get_verifications() {
            for rule in &self.verification_rules {
                if let Err(e) = rule.validate(verification, repository) {
                    report.add_error(verification.id, rule.name(), e.to_string());
                }
            }
        }
        
        Ok(report)
    }
}

impl Default for RequirementsValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Report of validation results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: HashMap<Id, Vec<ValidationError>>,
    pub warnings: HashMap<Id, Vec<ValidationWarning>>,
}

impl ValidationReport {
    /// Create a new empty validation report
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            warnings: HashMap::new(),
        }
    }
    
    /// Add a validation error
    pub fn add_error(&mut self, entity_id: Id, rule_name: &str, message: String) {
        let error = ValidationError {
            rule_name: rule_name.to_string(),
            message,
        };
        
        self.errors.entry(entity_id).or_insert_with(Vec::new).push(error);
    }
    
    /// Add a validation warning
    pub fn add_warning(&mut self, entity_id: Id, rule_name: &str, message: String) {
        let warning = ValidationWarning {
            rule_name: rule_name.to_string(),
            message,
        };
        
        self.warnings.entry(entity_id).or_insert_with(Vec::new).push(warning);
    }
    
    /// Check if the report has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Check if the report has any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
    
    /// Get total number of errors
    pub fn error_count(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }
    
    /// Get total number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.values().map(|v| v.len()).sum()
    }
    
    /// Get summary of validation results
    pub fn summary(&self) -> String {
        format!(
            "Validation completed: {} errors, {} warnings",
            self.error_count(),
            self.warning_count()
        )
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// A validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub rule_name: String,
    pub message: String,
}

/// A validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub rule_name: String,
    pub message: String,
}

/// Utility functions for common validation patterns
pub mod utils {
    use super::*;
    
    /// Check if a string contains only valid identifier characters
    pub fn is_valid_identifier(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ')
    }
    
    /// Check if a string is a meaningful description (not just placeholder text)
    pub fn is_meaningful_description(s: &str) -> bool {
        let placeholder_words = vec!["tbd", "todo", "placeholder", "example", "test"];
        let lower_s = s.to_lowercase();
        
        // Check minimum length
        if s.len() < 10 {
            return false;
        }
        
        // Check for placeholder words
        for word in placeholder_words {
            if lower_s.contains(word) {
                return false;
            }
        }
        
        true
    }
    
    /// Extract keywords from text for categorization
    pub fn extract_keywords(text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.to_lowercase())
            .collect()
    }
    
    /// Check if requirement coverage is adequate
    pub fn check_requirement_coverage(repository: &RequirementsRepository) -> CoverageReport {
        let mut report = CoverageReport::new();
        
        for requirement in repository.get_requirements() {
            let inputs = repository.get_design_inputs_for_requirement(&requirement.id);
            let input_coverage = !inputs.is_empty();
            
            let mut verification_coverage = false;
            for input in inputs {
                let verifications = repository.get_verifications_for_input(&input.id);
                if !verifications.is_empty() {
                    verification_coverage = true;
                    break;
                }
            }
            
            report.add_requirement_coverage(
                requirement.id,
                requirement.name.clone(),
                input_coverage,
                verification_coverage,
            );
        }
        
        report
    }
}

/// Report of requirement coverage analysis
#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub requirements: Vec<RequirementCoverage>,
}

impl CoverageReport {
    /// Create a new empty coverage report
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }
    
    /// Add requirement coverage information
    pub fn add_requirement_coverage(
        &mut self,
        id: Id,
        name: String,
        has_inputs: bool,
        has_verifications: bool,
    ) {
        self.requirements.push(RequirementCoverage {
            id,
            name,
            has_inputs,
            has_verifications,
        });
    }
    
    /// Get coverage statistics
    pub fn get_statistics(&self) -> CoverageStatistics {
        let total = self.requirements.len();
        let with_inputs = self.requirements.iter().filter(|r| r.has_inputs).count();
        let with_verifications = self.requirements.iter().filter(|r| r.has_verifications).count();
        
        CoverageStatistics {
            total_requirements: total,
            requirements_with_inputs: with_inputs,
            requirements_with_verifications: with_verifications,
            input_coverage_percentage: if total > 0 { (with_inputs as f64 / total as f64) * 100.0 } else { 0.0 },
            verification_coverage_percentage: if total > 0 { (with_verifications as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }
    
    /// Get requirements that lack coverage
    pub fn get_uncovered_requirements(&self) -> Vec<&RequirementCoverage> {
        self.requirements
            .iter()
            .filter(|r| !r.has_inputs || !r.has_verifications)
            .collect()
    }
}

impl Default for CoverageReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Coverage information for a single requirement
#[derive(Debug, Clone)]
pub struct RequirementCoverage {
    pub id: Id,
    pub name: String,
    pub has_inputs: bool,
    pub has_verifications: bool,
}

/// Statistics about requirement coverage
#[derive(Debug, Clone)]
pub struct CoverageStatistics {
    pub total_requirements: usize,
    pub requirements_with_inputs: usize,
    pub requirements_with_verifications: usize,
    pub input_coverage_percentage: f64,
    pub verification_coverage_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::RequirementsRepository;

    #[test]
    fn test_unique_name_rule() {
        let mut repo = RequirementsRepository::new();
        let rule = UniqueNameRule;
        
        let req1 = Requirement::new(
            "Test Requirement".to_string(),
            "First requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        repo.add_requirement(req1).unwrap();
        
        let req2 = Requirement::new(
            "Test Requirement".to_string(),
            "Second requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        
        // Should fail due to duplicate name
        assert!(rule.validate(&req2, &repo).is_err());
    }
    
    #[test]
    fn test_meaningful_criteria_rule() {
        let repo = RequirementsRepository::new();
        let rule = MeaningfulCriteriaRule;
        
        let mut req = Requirement::new(
            "Critical Requirement".to_string(),
            "A critical requirement".to_string(),
            RequirementCategory::Safety,
            Priority::Critical,
        );
        
        // Should fail without acceptance criteria
        assert!(rule.validate(&req, &repo).is_err());
        
        req.add_acceptance_criterion("System must respond within 100ms".to_string());
        
        // Should pass with meaningful criteria
        assert!(rule.validate(&req, &repo).is_ok());
    }
    
    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        let id = Id::new();
        
        report.add_error(id, "test_rule", "Test error message".to_string());
        report.add_warning(id, "test_rule", "Test warning message".to_string());
        
        assert!(report.has_errors());
        assert!(report.has_warnings());
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 1);
    }
    
    #[test]
    fn test_coverage_report() {
        let mut repo = RequirementsRepository::new();
        
        // Add requirement with full coverage
        let req1 = Requirement::new(
            "Covered Requirement".to_string(),
            "A fully covered requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req1_id = req1.id;
        repo.add_requirement(req1).unwrap();
        
        let input1 = DesignInput::new(
            "Input 1".to_string(),
            "Design input 1".to_string(),
            req1_id,
            "Specification 1".to_string(),
        );
        let input1_id = input1.id;
        repo.add_design_input(input1).unwrap();
        
        let output1 = DesignOutput::new(
            "Output 1".to_string(),
            "Design output 1".to_string(),
            input1_id,
            "Document".to_string(),
            "Test deliverable".to_string(),
        );
        let output1_id = output1.id;
        repo.add_design_output(output1).unwrap();
        
        let verification1 = Verification::new(
            "Verification 1".to_string(),
            "Test verification".to_string(),
            output1_id,
            "Test".to_string(),
            "Automated Test".to_string(),
        );
        repo.add_verification(verification1).unwrap();
        
        // Add requirement without coverage
        let req2 = Requirement::new(
            "Uncovered Requirement".to_string(),
            "An uncovered requirement".to_string(),
            RequirementCategory::Functional,
            Priority::Low,
        );
        repo.add_requirement(req2).unwrap();
        
        let report = utils::check_requirement_coverage(&repo);
        let stats = report.get_statistics();
        
        assert_eq!(stats.total_requirements, 2);
        assert_eq!(stats.requirements_with_inputs, 1);
        assert_eq!(stats.requirements_with_verifications, 1);
        assert_eq!(stats.input_coverage_percentage, 50.0);
        assert_eq!(stats.verification_coverage_percentage, 50.0);
    }
}