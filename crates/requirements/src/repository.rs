//! Repository for requirements management
//!
//! This module provides persistent storage and retrieval for requirements,
//! design inputs, outputs, and verifications.

use crate::data::*;
use tessera_core::{Id, Result, ProjectContext, Entity};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use ron::ser::{to_string_pretty, PrettyConfig};

/// Repository for managing requirements and related entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementsRepository {
    /// Collection of requirements indexed by ID
    pub requirements: IndexMap<Id, Requirement>,
    /// Collection of design inputs indexed by ID
    pub design_inputs: IndexMap<Id, DesignInput>,
    /// Collection of design outputs indexed by ID
    pub design_outputs: IndexMap<Id, DesignOutput>,
    /// Collection of verifications indexed by ID
    pub verifications: IndexMap<Id, Verification>,
}

impl RequirementsRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self {
            requirements: IndexMap::new(),
            design_inputs: IndexMap::new(),
            design_outputs: IndexMap::new(),
            verifications: IndexMap::new(),
        }
    }

    /// Load repository from project context
    pub fn load_from_project(project_ctx: &ProjectContext) -> Result<Self> {
        let requirements_dir = project_ctx.module_path("requirements");
        Self::load_from_directory(&requirements_dir)
    }

    /// Load repository from directory
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let mut repo = Self::new();

        // Load requirements
        let requirements_path = dir.join("requirements.ron");
        if requirements_path.exists() {
            let content = std::fs::read_to_string(&requirements_path)?;
            repo.requirements = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        // Load design inputs
        let inputs_path = dir.join("design_inputs.ron");
        if inputs_path.exists() {
            let content = std::fs::read_to_string(&inputs_path)?;
            repo.design_inputs = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        // Load design outputs
        let outputs_path = dir.join("design_outputs.ron");
        if outputs_path.exists() {
            let content = std::fs::read_to_string(&outputs_path)?;
            repo.design_outputs = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        // Load verifications
        let verifications_path = dir.join("verifications.ron");
        if verifications_path.exists() {
            let content = std::fs::read_to_string(&verifications_path)?;
            repo.verifications = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        Ok(repo)
    }

    /// Save repository to project context
    pub fn save_to_project(&self, project_ctx: &ProjectContext) -> Result<()> {
        let requirements_dir = project_ctx.module_path("requirements");
        self.save_to_directory(&requirements_dir)
    }

    /// Save repository to directory
    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        // Save requirements
        let requirements_path = dir.join("requirements.ron");
        let content = to_string_pretty(&self.requirements, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&requirements_path, content)?;

        // Save design inputs
        let inputs_path = dir.join("design_inputs.ron");
        let content = to_string_pretty(&self.design_inputs, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&inputs_path, content)?;

        // Save design outputs
        let outputs_path = dir.join("design_outputs.ron");
        let content = to_string_pretty(&self.design_outputs, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&outputs_path, content)?;

        // Save verifications
        let verifications_path = dir.join("verifications.ron");
        let content = to_string_pretty(&self.verifications, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&verifications_path, content)?;

        Ok(())
    }

    // Requirements management
    
    /// Add a new requirement
    pub fn add_requirement(&mut self, requirement: Requirement) -> Result<()> {
        requirement.validate()?;
        self.requirements.insert(requirement.id, requirement);
        Ok(())
    }

    /// Get a requirement by ID
    pub fn get_requirement(&self, id: &Id) -> Option<&Requirement> {
        self.requirements.get(id)
    }

    /// Get all requirements
    pub fn get_requirements(&self) -> Vec<&Requirement> {
        self.requirements.values().collect()
    }

    /// Update a requirement
    pub fn update_requirement(&mut self, requirement: Requirement) -> Result<()> {
        requirement.validate()?;
        self.requirements.insert(requirement.id, requirement);
        Ok(())
    }

    /// Remove a requirement
    pub fn remove_requirement(&mut self, id: &Id) -> Result<()> {
        // Check for dependent design inputs
        let dependent_inputs: Vec<_> = self.design_inputs
            .values()
            .filter(|input| input.requirement_id == *id)
            .collect();

        if !dependent_inputs.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Cannot remove requirement: {} dependent design inputs exist", dependent_inputs.len())
            ));
        }

        self.requirements.shift_remove(id);
        Ok(())
    }

    /// Get requirements by category
    pub fn get_requirements_by_category(&self, category: &RequirementCategory) -> Vec<&Requirement> {
        self.requirements
            .values()
            .filter(|req| req.category == *category)
            .collect()
    }

    /// Get requirements by priority
    pub fn get_requirements_by_priority(&self, priority: Priority) -> Vec<&Requirement> {
        self.requirements
            .values()
            .filter(|req| req.priority == priority)
            .collect()
    }

    /// Get requirements by status
    pub fn get_requirements_by_status(&self, status: RequirementStatus) -> Vec<&Requirement> {
        self.requirements
            .values()
            .filter(|req| req.status == status)
            .collect()
    }

    // Design inputs management

    /// Add a new design input
    pub fn add_design_input(&mut self, input: DesignInput) -> Result<()> {
        input.validate()?;
        
        // Validate that the linked requirement exists
        if !self.requirements.contains_key(&input.requirement_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked requirement does not exist".to_string()
            ));
        }

        self.design_inputs.insert(input.id, input);
        Ok(())
    }

    /// Get a design input by ID
    pub fn get_design_input(&self, id: &Id) -> Option<&DesignInput> {
        self.design_inputs.get(id)
    }

    /// Get all design inputs
    pub fn get_design_inputs(&self) -> Vec<&DesignInput> {
        self.design_inputs.values().collect()
    }

    /// Update a design input
    pub fn update_design_input(&mut self, input: DesignInput) -> Result<()> {
        input.validate()?;
        
        // Validate that the linked requirement exists
        if !self.requirements.contains_key(&input.requirement_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked requirement does not exist".to_string()
            ));
        }

        self.design_inputs.insert(input.id, input);
        Ok(())
    }

    /// Remove a design input
    pub fn remove_design_input(&mut self, id: &Id) -> Result<()> {
        // Check for dependent design outputs
        let dependent_outputs: Vec<_> = self.design_outputs
            .values()
            .filter(|output| output.input_id == *id)
            .collect();

        if !dependent_outputs.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Cannot remove design input: {} dependent design outputs exist", dependent_outputs.len())
            ));
        }

        self.design_inputs.shift_remove(id);
        Ok(())
    }

    /// Get design inputs for a requirement
    pub fn get_design_inputs_for_requirement(&self, requirement_id: &Id) -> Vec<&DesignInput> {
        self.design_inputs
            .values()
            .filter(|input| input.requirement_id == *requirement_id)
            .collect()
    }

    // Design outputs management

    /// Add a new design output
    pub fn add_design_output(&mut self, output: DesignOutput) -> Result<()> {
        output.validate()?;
        
        // Validate that the linked design input exists
        if !self.design_inputs.contains_key(&output.input_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked design input does not exist".to_string()
            ));
        }

        self.design_outputs.insert(output.id, output);
        Ok(())
    }

    /// Get a design output by ID
    pub fn get_design_output(&self, id: &Id) -> Option<&DesignOutput> {
        self.design_outputs.get(id)
    }

    /// Get all design outputs
    pub fn get_design_outputs(&self) -> Vec<&DesignOutput> {
        self.design_outputs.values().collect()
    }

    /// Update a design output
    pub fn update_design_output(&mut self, output: DesignOutput) -> Result<()> {
        output.validate()?;
        
        // Validate that the linked design input exists
        if !self.design_inputs.contains_key(&output.input_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked design input does not exist".to_string()
            ));
        }

        self.design_outputs.insert(output.id, output);
        Ok(())
    }

    /// Remove a design output
    pub fn remove_design_output(&mut self, id: &Id) -> Result<()> {
        // Check for dependent verifications
        let dependent_verifications: Vec<_> = self.verifications
            .values()
            .filter(|verification| verification.output_id == *id)
            .collect();

        if !dependent_verifications.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Cannot remove design output: {} dependent verifications exist", dependent_verifications.len())
            ));
        }

        self.design_outputs.shift_remove(id);
        Ok(())
    }

    /// Get design outputs for a design input
    pub fn get_design_outputs_for_input(&self, input_id: &Id) -> Vec<&DesignOutput> {
        self.design_outputs
            .values()
            .filter(|output| output.input_id == *input_id)
            .collect()
    }

    // Verifications management

    /// Add a new verification
    pub fn add_verification(&mut self, verification: Verification) -> Result<()> {
        verification.validate()?;
        
        // Validate that the linked design output exists
        if !self.design_outputs.contains_key(&verification.output_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked design output does not exist".to_string()
            ));
        }

        self.verifications.insert(verification.id, verification);
        Ok(())
    }

    /// Get a verification by ID
    pub fn get_verification(&self, id: &Id) -> Option<&Verification> {
        self.verifications.get(id)
    }

    /// Get all verifications
    pub fn get_verifications(&self) -> Vec<&Verification> {
        self.verifications.values().collect()
    }

    /// Update a verification
    pub fn update_verification(&mut self, verification: Verification) -> Result<()> {
        verification.validate()?;
        
        // Validate that the linked design output exists
        if !self.design_outputs.contains_key(&verification.output_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked design output does not exist".to_string()
            ));
        }

        self.verifications.insert(verification.id, verification);
        Ok(())
    }

    /// Remove a verification
    pub fn remove_verification(&mut self, id: &Id) -> Result<()> {
        self.verifications.shift_remove(id);
        Ok(())
    }

    /// Get verifications for a design output
    pub fn get_verifications_for_output(&self, output_id: &Id) -> Vec<&Verification> {
        self.verifications
            .values()
            .filter(|verification| verification.output_id == *output_id)
            .collect()
    }

    // Analytics and reporting

    /// Get repository statistics
    pub fn get_statistics(&self) -> RepositoryStatistics {
        let requirements_by_status = self.get_requirements_by_status_count();
        let requirements_by_priority = self.get_requirements_by_priority_count();
        let verification_completion_rate = self.get_verification_completion_rate();
        
        RepositoryStatistics {
            total_requirements: self.requirements.len(),
            total_design_inputs: self.design_inputs.len(),
            total_design_outputs: self.design_outputs.len(),
            total_verifications: self.verifications.len(),
            requirements_by_status,
            requirements_by_priority,
            verification_completion_rate,
        }
    }

    /// Get count of requirements by status
    fn get_requirements_by_status_count(&self) -> IndexMap<RequirementStatus, usize> {
        let mut counts = IndexMap::new();
        for req in self.requirements.values() {
            *counts.entry(req.status).or_insert(0) += 1;
        }
        counts
    }

    /// Get count of requirements by priority
    fn get_requirements_by_priority_count(&self) -> IndexMap<Priority, usize> {
        let mut counts = IndexMap::new();
        for req in self.requirements.values() {
            *counts.entry(req.priority).or_insert(0) += 1;
        }
        counts
    }

    /// Get verification completion rate
    fn get_verification_completion_rate(&self) -> f64 {
        if self.verifications.is_empty() {
            return 0.0;
        }
        
        let completed = self.verifications.values()
            .filter(|v| v.is_complete())
            .count();
        
        completed as f64 / self.verifications.len() as f64 * 100.0
    }

    /// Validate referential integrity
    pub fn validate_integrity(&self) -> Result<()> {
        // Check design inputs reference valid requirements
        for input in self.design_inputs.values() {
            if !self.requirements.contains_key(&input.requirement_id) {
                return Err(tessera_core::DesignTrackError::Validation(
                    format!("Design input '{}' references non-existent requirement", input.name)
                ));
            }
        }

        // Check design outputs reference valid design inputs
        for output in self.design_outputs.values() {
            if !self.design_inputs.contains_key(&output.input_id) {
                return Err(tessera_core::DesignTrackError::Validation(
                    format!("Design output '{}' references non-existent design input", output.name)
                ));
            }
        }

        // Check verifications reference valid design outputs
        for verification in self.verifications.values() {
            if !self.design_outputs.contains_key(&verification.output_id) {
                return Err(tessera_core::DesignTrackError::Validation(
                    format!("Verification '{}' references non-existent design output", verification.name)
                ));
            }
        }

        Ok(())
    }
}

impl Default for RequirementsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl RequirementsRepository {
    /// Validate all entities and referential integrity
    pub fn validate(&self) -> Result<()> {
        // Validate all entities
        for req in self.requirements.values() {
            req.validate()?;
        }
        
        for input in self.design_inputs.values() {
            input.validate()?;
        }
        
        for output in self.design_outputs.values() {
            output.validate()?;
        }
        
        for verification in self.verifications.values() {
            verification.validate()?;
        }

        // Validate referential integrity
        self.validate_integrity()?;

        Ok(())
    }
}

/// Statistics about the requirements repository
#[derive(Debug, Clone)]
pub struct RepositoryStatistics {
    pub total_requirements: usize,
    pub total_design_inputs: usize,
    pub total_design_outputs: usize,
    pub total_verifications: usize,
    pub requirements_by_status: IndexMap<RequirementStatus, usize>,
    pub requirements_by_priority: IndexMap<Priority, usize>,
    pub verification_completion_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_creation() {
        let repo = RequirementsRepository::new();
        assert!(repo.requirements.is_empty());
        assert!(repo.design_inputs.is_empty());
        assert!(repo.design_outputs.is_empty());
        assert!(repo.verifications.is_empty());
    }

    #[test]
    fn test_requirement_crud() {
        let mut repo = RequirementsRepository::new();
        
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req_id = req.id;

        // Add
        repo.add_requirement(req).unwrap();
        assert_eq!(repo.requirements.len(), 1);

        // Get
        let retrieved = repo.get_requirement(&req_id).unwrap();
        assert_eq!(retrieved.name, "Test Requirement");

        // Update
        let mut updated = retrieved.clone();
        updated.name = "Updated Requirement".to_string();
        repo.update_requirement(updated).unwrap();
        assert_eq!(repo.get_requirement(&req_id).unwrap().name, "Updated Requirement");

        // Remove
        repo.remove_requirement(&req_id).unwrap();
        assert!(repo.requirements.is_empty());
    }

    #[test]
    fn test_design_input_links() {
        let mut repo = RequirementsRepository::new();
        
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req_id = req.id;
        repo.add_requirement(req).unwrap();

        let input = DesignInput::new(
            "Test Input".to_string(),
            "A test input".to_string(),
            req_id,
            "Test specification".to_string(),
        );

        // Should succeed with valid requirement link
        repo.add_design_input(input).unwrap();
        assert_eq!(repo.design_inputs.len(), 1);

        // Should fail with invalid requirement link
        let invalid_input = DesignInput::new(
            "Invalid Input".to_string(),
            "Invalid input".to_string(),
            Id::new(), // Non-existent requirement
            "Test specification".to_string(),
        );
        
        assert!(repo.add_design_input(invalid_input).is_err());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create and populate repository
        let mut repo = RequirementsRepository::new();
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        repo.add_requirement(req).unwrap();

        // Save
        repo.save_to_directory(dir_path).unwrap();

        // Load and verify
        let loaded_repo = RequirementsRepository::load_from_directory(dir_path).unwrap();
        assert_eq!(loaded_repo.requirements.len(), 1);
        assert_eq!(loaded_repo.requirements.values().next().unwrap().name, "Test Requirement");
    }

    #[test]
    fn test_dependency_validation() {
        let mut repo = RequirementsRepository::new();
        
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req_id = req.id;
        repo.add_requirement(req).unwrap();

        let input = DesignInput::new(
            "Test Input".to_string(),
            "A test input".to_string(),
            req_id,
            "Test specification".to_string(),
        );
        repo.add_design_input(input).unwrap();

        // Should not be able to remove requirement with dependent inputs
        assert!(repo.remove_requirement(&req_id).is_err());
    }
}