//! Repository for risk management
//!
//! This module provides persistent storage and retrieval for risks and design controls.

use crate::data::*;
use tessera_core::{Id, Result, ProjectContext, Entity};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use ron::ser::{to_string_pretty, PrettyConfig};

/// Repository for managing risks and design controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRepository {
    /// Collection of risks indexed by ID
    pub risks: IndexMap<Id, Risk>,
    /// Collection of design controls indexed by ID
    pub design_controls: IndexMap<Id, DesignControl>,
}

impl RiskRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self {
            risks: IndexMap::new(),
            design_controls: IndexMap::new(),
        }
    }

    /// Load repository from project context
    pub fn load_from_project(project_ctx: &ProjectContext) -> Result<Self> {
        let risk_dir = project_ctx.module_path("risk");
        Self::load_from_directory(&risk_dir)
    }

    /// Load repository from directory
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let mut repo = Self::new();

        // Load risks
        let risks_path = dir.join("risks.ron");
        if risks_path.exists() {
            let content = std::fs::read_to_string(&risks_path)?;
            repo.risks = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        // Load design controls
        let controls_path = dir.join("design_controls.ron");
        if controls_path.exists() {
            let content = std::fs::read_to_string(&controls_path)?;
            repo.design_controls = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        Ok(repo)
    }

    /// Save repository to project context
    pub fn save_to_project(&self, project_ctx: &ProjectContext) -> Result<()> {
        let risk_dir = project_ctx.module_path("risk");
        self.save_to_directory(&risk_dir)
    }

    /// Save repository to directory
    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        // Save risks
        let risks_path = dir.join("risks.ron");
        let content = to_string_pretty(&self.risks, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&risks_path, content)?;

        // Save design controls
        let controls_path = dir.join("design_controls.ron");
        let content = to_string_pretty(&self.design_controls, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&controls_path, content)?;

        Ok(())
    }

    // Risk management methods

    /// Add a new risk
    pub fn add_risk(&mut self, mut risk: Risk) -> Result<()> {
        risk.validate()?;
        risk.recalculate_risk_score();
        self.risks.insert(risk.id, risk);
        Ok(())
    }

    /// Get a risk by ID
    pub fn get_risk(&self, id: &Id) -> Option<&Risk> {
        self.risks.get(id)
    }

    /// Get all risks
    pub fn get_risks(&self) -> Vec<&Risk> {
        self.risks.values().collect()
    }

    /// Update a risk
    pub fn update_risk(&mut self, mut risk: Risk) -> Result<()> {
        risk.validate()?;
        risk.recalculate_risk_score();
        self.risks.insert(risk.id, risk);
        Ok(())
    }

    /// Remove a risk
    pub fn remove_risk(&mut self, id: &Id) -> Result<()> {
        // Check for dependent design controls
        let dependent_controls: Vec<_> = self.design_controls
            .values()
            .filter(|control| control.risk_id == *id)
            .collect();

        if !dependent_controls.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Cannot remove risk: {} dependent design controls exist", dependent_controls.len())
            ));
        }

        self.risks.shift_remove(id);
        Ok(())
    }

    /// Get risks by category
    pub fn get_risks_by_category(&self, category: &RiskCategory) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.category == *category)
            .collect()
    }

    /// Get risks by status
    pub fn get_risks_by_status(&self, status: RiskStatus) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.status == status)
            .collect()
    }

    /// Get risks by level
    pub fn get_risks_by_level(&self, level: RiskLevel) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.risk_level == level)
            .collect()
    }

    /// Get high-priority risks (high and critical)
    pub fn get_high_priority_risks(&self) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| matches!(risk.risk_level, RiskLevel::High | RiskLevel::Critical))
            .collect()
    }

    /// Get overdue risks
    pub fn get_overdue_risks(&self) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.is_overdue())
            .collect()
    }

    /// Get risks needing attention
    pub fn get_risks_needing_attention(&self) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.needs_attention())
            .collect()
    }

    // Design control management methods

    /// Add a new design control
    pub fn add_design_control(&mut self, control: DesignControl) -> Result<()> {
        control.validate()?;
        
        // Validate that the linked risk exists
        if !self.risks.contains_key(&control.risk_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked risk does not exist".to_string()
            ));
        }

        self.design_controls.insert(control.id, control);
        Ok(())
    }

    /// Get a design control by ID
    pub fn get_design_control(&self, id: &Id) -> Option<&DesignControl> {
        self.design_controls.get(id)
    }

    /// Get all design controls
    pub fn get_design_controls(&self) -> Vec<&DesignControl> {
        self.design_controls.values().collect()
    }

    /// Update a design control
    pub fn update_design_control(&mut self, control: DesignControl) -> Result<()> {
        control.validate()?;
        
        // Validate that the linked risk exists
        if !self.risks.contains_key(&control.risk_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked risk does not exist".to_string()
            ));
        }

        self.design_controls.insert(control.id, control);
        Ok(())
    }

    /// Remove a design control
    pub fn remove_design_control(&mut self, id: &Id) -> Result<()> {
        self.design_controls.shift_remove(id);
        Ok(())
    }

    /// Get design controls for a risk
    pub fn get_design_controls_for_risk(&self, risk_id: &Id) -> Vec<&DesignControl> {
        self.design_controls
            .values()
            .filter(|control| control.risk_id == *risk_id)
            .collect()
    }

    /// Get design controls by type
    pub fn get_design_controls_by_type(&self, control_type: &ControlType) -> Vec<&DesignControl> {
        self.design_controls
            .values()
            .filter(|control| control.control_type == *control_type)
            .collect()
    }

    /// Get design controls by status
    pub fn get_design_controls_by_status(&self, status: ControlStatus) -> Vec<&DesignControl> {
        self.design_controls
            .values()
            .filter(|control| control.status == status)
            .collect()
    }

    /// Get overdue design controls
    pub fn get_overdue_design_controls(&self) -> Vec<&DesignControl> {
        self.design_controls
            .values()
            .filter(|control| control.is_overdue())
            .collect()
    }

    /// Get effective design controls
    pub fn get_effective_design_controls(&self) -> Vec<&DesignControl> {
        self.design_controls
            .values()
            .filter(|control| control.is_effective())
            .collect()
    }

    // Analytics and reporting methods

    /// Get repository statistics
    pub fn get_statistics(&self) -> RiskRepositoryStatistics {
        let risks_by_level = self.get_risks_by_level_count();
        let risks_by_status = self.get_risks_by_status_count();
        let risks_by_category = self.get_risks_by_category_count();
        let controls_by_type = self.get_controls_by_type_count();
        let controls_by_status = self.get_controls_by_status_count();
        
        let total_risks = self.risks.len();
        let high_priority_risks = self.get_high_priority_risks().len();
        let overdue_risks = self.get_overdue_risks().len();
        let risks_needing_attention = self.get_risks_needing_attention().len();
        
        let total_controls = self.design_controls.len();
        let effective_controls = self.get_effective_design_controls().len();
        let overdue_controls = self.get_overdue_design_controls().len();
        
        let control_effectiveness_rate = if total_controls > 0 {
            (effective_controls as f64 / total_controls as f64) * 100.0
        } else {
            0.0
        };

        RiskRepositoryStatistics {
            total_risks,
            high_priority_risks,
            overdue_risks,
            risks_needing_attention,
            total_controls,
            effective_controls,
            overdue_controls,
            control_effectiveness_rate,
            risks_by_level,
            risks_by_status,
            risks_by_category,
            controls_by_type,
            controls_by_status,
        }
    }

    /// Get count of risks by level
    fn get_risks_by_level_count(&self) -> IndexMap<RiskLevel, usize> {
        let mut counts = IndexMap::new();
        for risk in self.risks.values() {
            *counts.entry(risk.risk_level).or_insert(0) += 1;
        }
        counts
    }

    /// Get count of risks by status
    fn get_risks_by_status_count(&self) -> IndexMap<RiskStatus, usize> {
        let mut counts = IndexMap::new();
        for risk in self.risks.values() {
            *counts.entry(risk.status).or_insert(0) += 1;
        }
        counts
    }

    /// Get count of risks by category
    fn get_risks_by_category_count(&self) -> IndexMap<String, usize> {
        let mut counts = IndexMap::new();
        for risk in self.risks.values() {
            let category = risk.category.to_string();
            *counts.entry(category).or_insert(0) += 1;
        }
        counts
    }

    /// Get count of controls by type
    fn get_controls_by_type_count(&self) -> IndexMap<ControlType, usize> {
        let mut counts = IndexMap::new();
        for control in self.design_controls.values() {
            *counts.entry(control.control_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Get count of controls by status
    fn get_controls_by_status_count(&self) -> IndexMap<ControlStatus, usize> {
        let mut counts = IndexMap::new();
        for control in self.design_controls.values() {
            *counts.entry(control.status).or_insert(0) += 1;
        }
        counts
    }

    /// Validate referential integrity
    pub fn validate_integrity(&self) -> Result<()> {
        // Check design controls reference valid risks
        for control in self.design_controls.values() {
            if !self.risks.contains_key(&control.risk_id) {
                return Err(tessera_core::DesignTrackError::Validation(
                    format!("Design control '{}' references non-existent risk", control.name)
                ));
            }
        }

        Ok(())
    }

    /// Get risk mitigation coverage
    pub fn get_risk_mitigation_coverage(&self) -> RiskMitigationCoverage {
        let mut coverage = RiskMitigationCoverage::new();
        
        for risk in self.risks.values() {
            let controls = self.get_design_controls_for_risk(&risk.id);
            let has_controls = !controls.is_empty();
            let has_effective_controls = controls.iter().any(|c| c.is_effective());
            
            coverage.add_risk_coverage(
                risk.id,
                risk.name.clone(),
                risk.risk_level,
                has_controls,
                has_effective_controls,
                controls.len(),
            );
        }
        
        coverage
    }

    /// Recalculate all risk scores
    pub fn recalculate_all_risk_scores(&mut self) {
        for risk in self.risks.values_mut() {
            risk.recalculate_risk_score();
        }
    }
}

impl Default for RiskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskRepository {
    /// Validate all entities and referential integrity
    pub fn validate(&self) -> Result<()> {
        // Validate all entities
        for risk in self.risks.values() {
            risk.validate()?;
        }
        
        for control in self.design_controls.values() {
            control.validate()?;
        }

        // Validate referential integrity
        self.validate_integrity()?;

        Ok(())
    }
}

/// Statistics about the risk repository
#[derive(Debug, Clone)]
pub struct RiskRepositoryStatistics {
    pub total_risks: usize,
    pub high_priority_risks: usize,
    pub overdue_risks: usize,
    pub risks_needing_attention: usize,
    pub total_controls: usize,
    pub effective_controls: usize,
    pub overdue_controls: usize,
    pub control_effectiveness_rate: f64,
    pub risks_by_level: IndexMap<RiskLevel, usize>,
    pub risks_by_status: IndexMap<RiskStatus, usize>,
    pub risks_by_category: IndexMap<String, usize>,
    pub controls_by_type: IndexMap<ControlType, usize>,
    pub controls_by_status: IndexMap<ControlStatus, usize>,
}

/// Risk mitigation coverage analysis
#[derive(Debug, Clone)]
pub struct RiskMitigationCoverage {
    pub risks: Vec<RiskCoverage>,
}

impl RiskMitigationCoverage {
    /// Create a new empty coverage report
    pub fn new() -> Self {
        Self {
            risks: Vec::new(),
        }
    }

    /// Add risk coverage information
    pub fn add_risk_coverage(
        &mut self,
        id: Id,
        name: String,
        level: RiskLevel,
        has_controls: bool,
        has_effective_controls: bool,
        control_count: usize,
    ) {
        self.risks.push(RiskCoverage {
            id,
            name,
            level,
            has_controls,
            has_effective_controls,
            control_count,
        });
    }

    /// Get coverage statistics
    pub fn get_statistics(&self) -> RiskCoverageStatistics {
        let total = self.risks.len();
        let with_controls = self.risks.iter().filter(|r| r.has_controls).count();
        let with_effective_controls = self.risks.iter().filter(|r| r.has_effective_controls).count();
        
        let high_priority_risks = self.risks.iter()
            .filter(|r| matches!(r.level, RiskLevel::High | RiskLevel::Critical))
            .count();
        let high_priority_with_controls = self.risks.iter()
            .filter(|r| matches!(r.level, RiskLevel::High | RiskLevel::Critical) && r.has_controls)
            .count();

        RiskCoverageStatistics {
            total_risks: total,
            risks_with_controls: with_controls,
            risks_with_effective_controls: with_effective_controls,
            high_priority_risks,
            high_priority_with_controls,
            control_coverage_percentage: if total > 0 { (with_controls as f64 / total as f64) * 100.0 } else { 0.0 },
            effective_control_coverage_percentage: if total > 0 { (with_effective_controls as f64 / total as f64) * 100.0 } else { 0.0 },
            high_priority_control_coverage_percentage: if high_priority_risks > 0 { (high_priority_with_controls as f64 / high_priority_risks as f64) * 100.0 } else { 0.0 },
        }
    }

    /// Get risks without controls
    pub fn get_unmitigated_risks(&self) -> Vec<&RiskCoverage> {
        self.risks
            .iter()
            .filter(|r| !r.has_controls)
            .collect()
    }

    /// Get risks without effective controls
    pub fn get_inadequately_mitigated_risks(&self) -> Vec<&RiskCoverage> {
        self.risks
            .iter()
            .filter(|r| r.has_controls && !r.has_effective_controls)
            .collect()
    }
}

impl Default for RiskMitigationCoverage {
    fn default() -> Self {
        Self::new()
    }
}

/// Coverage information for a single risk
#[derive(Debug, Clone)]
pub struct RiskCoverage {
    pub id: Id,
    pub name: String,
    pub level: RiskLevel,
    pub has_controls: bool,
    pub has_effective_controls: bool,
    pub control_count: usize,
}

/// Statistics about risk coverage
#[derive(Debug, Clone)]
pub struct RiskCoverageStatistics {
    pub total_risks: usize,
    pub risks_with_controls: usize,
    pub risks_with_effective_controls: usize,
    pub high_priority_risks: usize,
    pub high_priority_with_controls: usize,
    pub control_coverage_percentage: f64,
    pub effective_control_coverage_percentage: f64,
    pub high_priority_control_coverage_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_creation() {
        let repo = RiskRepository::new();
        assert!(repo.risks.is_empty());
        assert!(repo.design_controls.is_empty());
    }

    #[test]
    fn test_risk_crud() {
        let mut repo = RiskRepository::new();
        
        let risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );
        let risk_id = risk.id;

        // Add
        repo.add_risk(risk).unwrap();
        assert_eq!(repo.risks.len(), 1);

        // Get
        let retrieved = repo.get_risk(&risk_id).unwrap();
        assert_eq!(retrieved.name, "Test Risk");

        // Update
        let mut updated = retrieved.clone();
        updated.name = "Updated Risk".to_string();
        repo.update_risk(updated).unwrap();
        assert_eq!(repo.get_risk(&risk_id).unwrap().name, "Updated Risk");

        // Remove
        repo.remove_risk(&risk_id).unwrap();
        assert!(repo.risks.is_empty());
    }

    #[test]
    fn test_design_control_links() {
        let mut repo = RiskRepository::new();
        
        let risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );
        let risk_id = risk.id;
        repo.add_risk(risk).unwrap();

        let control = DesignControl::new(
            "Test Control".to_string(),
            "A test control".to_string(),
            ControlType::Preventive,
            risk_id,
        );

        // Should succeed with valid risk link
        repo.add_design_control(control).unwrap();
        assert_eq!(repo.design_controls.len(), 1);

        // Should fail with invalid risk link
        let invalid_control = DesignControl::new(
            "Invalid Control".to_string(),
            "Invalid control".to_string(),
            ControlType::Preventive,
            Id::new(), // Non-existent risk
        );
        
        assert!(repo.add_design_control(invalid_control).is_err());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create and populate repository
        let mut repo = RiskRepository::new();
        let risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );
        repo.add_risk(risk).unwrap();

        // Save
        repo.save_to_directory(dir_path).unwrap();

        // Load and verify
        let loaded_repo = RiskRepository::load_from_directory(dir_path).unwrap();
        assert_eq!(loaded_repo.risks.len(), 1);
        assert_eq!(loaded_repo.risks.values().next().unwrap().name, "Test Risk");
    }

    #[test]
    fn test_risk_mitigation_coverage() {
        let mut repo = RiskRepository::new();
        
        // Add risk with control
        let risk1 = Risk::new(
            "Mitigated Risk".to_string(),
            "A mitigated risk".to_string(),
            RiskCategory::Technical,
        );
        let risk1_id = risk1.id;
        repo.add_risk(risk1).unwrap();

        let mut control = DesignControl::new(
            "Control 1".to_string(),
            "A control".to_string(),
            ControlType::Preventive,
            risk1_id,
        );
        control.set_effectiveness_rating(4);
        repo.add_design_control(control).unwrap();

        // Add risk without control
        let risk2 = Risk::new(
            "Unmitigated Risk".to_string(),
            "An unmitigated risk".to_string(),
            RiskCategory::Technical,
        );
        repo.add_risk(risk2).unwrap();

        let coverage = repo.get_risk_mitigation_coverage();
        let stats = coverage.get_statistics();

        assert_eq!(stats.total_risks, 2);
        assert_eq!(stats.risks_with_controls, 1);
        assert_eq!(stats.risks_with_effective_controls, 1);
        assert_eq!(stats.control_coverage_percentage, 50.0);
        assert_eq!(stats.effective_control_coverage_percentage, 50.0);
    }

    #[test]
    fn test_dependency_validation() {
        let mut repo = RiskRepository::new();
        
        let risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );
        let risk_id = risk.id;
        repo.add_risk(risk).unwrap();

        let control = DesignControl::new(
            "Test Control".to_string(),
            "A test control".to_string(),
            ControlType::Preventive,
            risk_id,
        );
        repo.add_design_control(control).unwrap();

        // Should not be able to remove risk with dependent controls
        assert!(repo.remove_risk(&risk_id).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut repo = RiskRepository::new();
        
        // Add various risks
        let mut risk1 = Risk::new(
            "High Risk".to_string(),
            "A high risk".to_string(),
            RiskCategory::Safety,
        );
        risk1.update_scores(4, 4, None);
        repo.add_risk(risk1).unwrap();

        let mut risk2 = Risk::new(
            "Low Risk".to_string(),
            "A low risk".to_string(),
            RiskCategory::Technical,
        );
        risk2.update_scores(2, 2, None);
        repo.add_risk(risk2).unwrap();

        let stats = repo.get_statistics();
        assert_eq!(stats.total_risks, 2);
        assert!(stats.risks_by_level.contains_key(&RiskLevel::High));
        assert!(stats.risks_by_level.contains_key(&RiskLevel::Low));
    }
}