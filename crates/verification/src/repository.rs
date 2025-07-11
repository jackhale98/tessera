//! Repository for verification management
//!
//! This module provides persistent storage and retrieval for test procedures,
//! executions, and results.

use crate::data::*;
use tessera_core::{Id, Result, ProjectContext, Entity};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use ron::ser::{to_string_pretty, PrettyConfig};

/// Repository for managing test procedures and executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRepository {
    /// Collection of test procedures indexed by ID
    pub procedures: IndexMap<Id, TestProcedure>,
    /// Collection of test executions indexed by ID
    pub executions: IndexMap<Id, TestExecution>,
}

impl VerificationRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self {
            procedures: IndexMap::new(),
            executions: IndexMap::new(),
        }
    }

    /// Load repository from project context
    pub fn load_from_project(project_ctx: &ProjectContext) -> Result<Self> {
        let verification_dir = project_ctx.module_path("verification");
        Self::load_from_directory(&verification_dir)
    }

    /// Load repository from directory
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let mut repo = Self::new();

        // Load procedures
        let procedures_path = dir.join("procedures.ron");
        if procedures_path.exists() {
            let content = std::fs::read_to_string(&procedures_path)?;
            repo.procedures = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        // Load executions
        let executions_path = dir.join("executions.ron");
        if executions_path.exists() {
            let content = std::fs::read_to_string(&executions_path)?;
            repo.executions = ron::from_str(&content)
                .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e.into()))?;
        }

        Ok(repo)
    }

    /// Save repository to project context
    pub fn save_to_project(&self, project_ctx: &ProjectContext) -> Result<()> {
        let verification_dir = project_ctx.module_path("verification");
        self.save_to_directory(&verification_dir)
    }

    /// Save repository to directory
    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        // Save procedures
        let procedures_path = dir.join("procedures.ron");
        let content = to_string_pretty(&self.procedures, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&procedures_path, content)?;

        // Save executions
        let executions_path = dir.join("executions.ron");
        let content = to_string_pretty(&self.executions, PrettyConfig::default())
            .map_err(|e| tessera_core::DesignTrackError::RonSerialization(e))?;
        std::fs::write(&executions_path, content)?;

        Ok(())
    }

    // Test procedure management

    /// Add a new test procedure
    pub fn add_test_procedure(&mut self, procedure: TestProcedure) -> Result<()> {
        procedure.validate()?;
        self.procedures.insert(procedure.id, procedure);
        Ok(())
    }

    /// Get a test procedure by ID
    pub fn get_test_procedure(&self, id: &Id) -> Option<&TestProcedure> {
        self.procedures.get(id)
    }

    /// Get all test procedures
    pub fn get_test_procedures(&self) -> Vec<&TestProcedure> {
        self.procedures.values().collect()
    }

    /// Update a test procedure
    pub fn update_test_procedure(&mut self, procedure: TestProcedure) -> Result<()> {
        procedure.validate()?;
        self.procedures.insert(procedure.id, procedure);
        Ok(())
    }

    /// Remove a test procedure
    pub fn remove_test_procedure(&mut self, id: &Id) -> Result<()> {
        // Check for dependent executions
        let dependent_executions: Vec<_> = self.executions
            .values()
            .filter(|execution| execution.procedure_id == *id)
            .collect();

        if !dependent_executions.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Cannot remove procedure: {} dependent executions exist", dependent_executions.len())
            ));
        }

        self.procedures.shift_remove(id);
        Ok(())
    }

    // Test execution management

    /// Add a new test execution
    pub fn add_test_execution(&mut self, execution: TestExecution) -> Result<()> {
        execution.validate()?;
        
        // Validate that the linked procedure exists
        if !self.procedures.contains_key(&execution.procedure_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked procedure does not exist".to_string()
            ));
        }

        self.executions.insert(execution.id, execution);
        Ok(())
    }

    /// Get a test execution by ID
    pub fn get_test_execution(&self, id: &Id) -> Option<&TestExecution> {
        self.executions.get(id)
    }

    /// Get all test executions
    pub fn get_test_executions(&self) -> Vec<&TestExecution> {
        self.executions.values().collect()
    }

    /// Update a test execution
    pub fn update_test_execution(&mut self, execution: TestExecution) -> Result<()> {
        execution.validate()?;
        
        // Validate that the linked procedure exists
        if !self.procedures.contains_key(&execution.procedure_id) {
            return Err(tessera_core::DesignTrackError::Validation(
                "Linked procedure does not exist".to_string()
            ));
        }

        self.executions.insert(execution.id, execution);
        Ok(())
    }

    /// Remove a test execution
    pub fn remove_test_execution(&mut self, id: &Id) -> Result<()> {
        self.executions.shift_remove(id);
        Ok(())
    }

    /// Get executions for a procedure
    pub fn get_executions_for_procedure(&self, procedure_id: &Id) -> Vec<&TestExecution> {
        self.executions
            .values()
            .filter(|execution| execution.procedure_id == *procedure_id)
            .collect()
    }

    // Query methods

    /// Get procedures by type
    pub fn get_procedures_by_type(&self, procedure_type: &ProcedureType) -> Vec<&TestProcedure> {
        self.procedures
            .values()
            .filter(|proc| proc.procedure_type == *procedure_type)
            .collect()
    }

    /// Get procedures by status
    pub fn get_procedures_by_status(&self, status: ProcedureStatus) -> Vec<&TestProcedure> {
        self.procedures
            .values()
            .filter(|proc| proc.status == status)
            .collect()
    }

    /// Get executions by status
    pub fn get_executions_by_status(&self, status: ExecutionStatus) -> Vec<&TestExecution> {
        self.executions
            .values()
            .filter(|exec| exec.status == status)
            .collect()
    }

    /// Get recent executions
    pub fn get_recent_executions(&self, limit: usize) -> Vec<&TestExecution> {
        let mut executions: Vec<_> = self.executions.values().collect();
        executions.sort_by(|a, b| b.created.cmp(&a.created));
        executions.into_iter().take(limit).collect()
    }

    /// Get repository statistics
    pub fn get_statistics(&self) -> VerificationStatistics {
        let total_procedures = self.procedures.len();
        let total_executions = self.executions.len();
        
        let procedures_by_type = self.get_procedures_by_type_count();
        let procedures_by_status = self.get_procedures_by_status_count();
        let executions_by_status = self.get_executions_by_status_count();
        
        let passed_executions = self.get_executions_by_status(ExecutionStatus::Passed).len();
        let failed_executions = self.get_executions_by_status(ExecutionStatus::Failed).len();
        
        let pass_rate = if total_executions > 0 {
            (passed_executions as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        VerificationStatistics {
            total_procedures,
            total_executions,
            passed_executions,
            failed_executions,
            pass_rate,
            procedures_by_type,
            procedures_by_status,
            executions_by_status,
        }
    }

    /// Get procedures by type count
    fn get_procedures_by_type_count(&self) -> IndexMap<String, usize> {
        let mut counts = IndexMap::new();
        for proc in self.procedures.values() {
            let type_name = proc.procedure_type.to_string();
            *counts.entry(type_name).or_insert(0) += 1;
        }
        counts
    }

    /// Get procedures by status count
    fn get_procedures_by_status_count(&self) -> IndexMap<ProcedureStatus, usize> {
        let mut counts = IndexMap::new();
        for proc in self.procedures.values() {
            *counts.entry(proc.status).or_insert(0) += 1;
        }
        counts
    }

    /// Get executions by status count
    fn get_executions_by_status_count(&self) -> IndexMap<ExecutionStatus, usize> {
        let mut counts = IndexMap::new();
        for exec in self.executions.values() {
            *counts.entry(exec.status).or_insert(0) += 1;
        }
        counts
    }

    /// Validate referential integrity
    pub fn validate_integrity(&self) -> Result<()> {
        // Check executions reference valid procedures
        for execution in self.executions.values() {
            if !self.procedures.contains_key(&execution.procedure_id) {
                return Err(tessera_core::DesignTrackError::Validation(
                    format!("Execution '{}' references non-existent procedure", execution.execution_name)
                ));
            }
        }

        Ok(())
    }
}

impl Default for VerificationRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationRepository {
    /// Validate all entities and referential integrity
    pub fn validate(&self) -> Result<()> {
        // Validate all entities
        for procedure in self.procedures.values() {
            procedure.validate()?;
        }
        
        for execution in self.executions.values() {
            execution.validate()?;
        }

        // Validate referential integrity
        self.validate_integrity()?;

        Ok(())
    }
}

/// Statistics about the verification repository
#[derive(Debug, Clone)]
pub struct VerificationStatistics {
    pub total_procedures: usize,
    pub total_executions: usize,
    pub passed_executions: usize,
    pub failed_executions: usize,
    pub pass_rate: f64,
    pub procedures_by_type: IndexMap<String, usize>,
    pub procedures_by_status: IndexMap<ProcedureStatus, usize>,
    pub executions_by_status: IndexMap<ExecutionStatus, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_creation() {
        let repo = VerificationRepository::new();
        assert!(repo.procedures.is_empty());
        assert!(repo.executions.is_empty());
    }

    #[test]
    fn test_procedure_crud() {
        let mut repo = VerificationRepository::new();
        
        let mut procedure = TestProcedure::new(
            "Test Procedure".to_string(),
            "A test procedure".to_string(),
            ProcedureType::Integration,
        );
        
        // Add a step to make it valid
        let step = TestStep::new(1, "Test step".to_string(), "Expected result".to_string());
        procedure.add_step(step);
        
        let procedure_id = procedure.id;

        // Add
        repo.add_test_procedure(procedure).unwrap();
        assert_eq!(repo.procedures.len(), 1);

        // Get
        let retrieved = repo.get_test_procedure(&procedure_id).unwrap();
        assert_eq!(retrieved.name, "Test Procedure");

        // Update
        let mut updated = retrieved.clone();
        updated.name = "Updated Procedure".to_string();
        repo.update_test_procedure(updated).unwrap();
        assert_eq!(repo.get_test_procedure(&procedure_id).unwrap().name, "Updated Procedure");

        // Remove
        repo.remove_test_procedure(&procedure_id).unwrap();
        assert!(repo.procedures.is_empty());
    }

    #[test]
    fn test_execution_links() {
        let mut repo = VerificationRepository::new();
        
        let mut procedure = TestProcedure::new(
            "Test Procedure".to_string(),
            "A test procedure".to_string(),
            ProcedureType::Integration,
        );
        
        // Add a step to make it valid
        let step = TestStep::new(1, "Test step".to_string(), "Expected result".to_string());
        procedure.add_step(step);
        
        let procedure_id = procedure.id;
        repo.add_test_procedure(procedure).unwrap();

        let execution = TestExecution::new(
            procedure_id,
            "Test Execution".to_string(),
        );

        // Should succeed with valid procedure link
        repo.add_test_execution(execution).unwrap();
        assert_eq!(repo.executions.len(), 1);

        // Should fail with invalid procedure link
        let invalid_execution = TestExecution::new(
            Id::new(), // Non-existent procedure
            "Invalid Execution".to_string(),
        );
        
        assert!(repo.add_test_execution(invalid_execution).is_err());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create and populate repository
        let mut repo = VerificationRepository::new();
        let mut procedure = TestProcedure::new(
            "Test Procedure".to_string(),
            "A test procedure".to_string(),
            ProcedureType::Integration,
        );
        
        let step = TestStep::new(1, "Test step".to_string(), "Expected result".to_string());
        procedure.add_step(step);
        
        repo.add_test_procedure(procedure).unwrap();

        // Save
        repo.save_to_directory(dir_path).unwrap();

        // Load and verify
        let loaded_repo = VerificationRepository::load_from_directory(dir_path).unwrap();
        assert_eq!(loaded_repo.procedures.len(), 1);
        assert_eq!(loaded_repo.procedures.values().next().unwrap().name, "Test Procedure");
    }

    #[test]
    fn test_statistics_calculation() {
        let mut repo = VerificationRepository::new();
        
        // Add procedures
        let mut proc1 = TestProcedure::new(
            "Unit Test".to_string(),
            "A unit test".to_string(),
            ProcedureType::Unit,
        );
        let step1 = TestStep::new(1, "Test step".to_string(), "Expected result".to_string());
        proc1.add_step(step1);
        let proc1_id = proc1.id;
        repo.add_test_procedure(proc1).unwrap();

        let mut proc2 = TestProcedure::new(
            "Integration Test".to_string(),
            "An integration test".to_string(),
            ProcedureType::Integration,
        );
        let step2 = TestStep::new(1, "Test step".to_string(), "Expected result".to_string());
        proc2.add_step(step2);
        let proc2_id = proc2.id;
        repo.add_test_procedure(proc2).unwrap();

        // Add executions
        let mut exec1 = TestExecution::new(proc1_id, "Execution 1".to_string());
        exec1.complete(ExecutionStatus::Passed);
        repo.add_test_execution(exec1).unwrap();

        let mut exec2 = TestExecution::new(proc2_id, "Execution 2".to_string());
        exec2.complete(ExecutionStatus::Failed);
        repo.add_test_execution(exec2).unwrap();

        let stats = repo.get_statistics();
        assert_eq!(stats.total_procedures, 2);
        assert_eq!(stats.total_executions, 2);
        assert_eq!(stats.passed_executions, 1);
        assert_eq!(stats.failed_executions, 1);
        assert_eq!(stats.pass_rate, 50.0);
    }
}