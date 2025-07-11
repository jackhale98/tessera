//! Integration with requirements management
//!
//! This module provides integration capabilities with the tessera-requirements crate
//! for bidirectional linking and status updates.

use crate::data::*;
use tessera_core::{Id, Result};
use tessera_requirements::{RequirementsCommands, VerificationStatus};

/// Integration manager for requirements-verification linking
pub struct RequirementsIntegration {
    requirements_commands: RequirementsCommands,
}

impl RequirementsIntegration {
    /// Create a new integration manager
    pub fn new(requirements_commands: RequirementsCommands) -> Self {
        Self {
            requirements_commands,
        }
    }

    /// Update verification status in requirements module
    pub fn update_verification_status(
        &mut self,
        verification_id: &Id,
        execution: &TestExecution,
    ) -> Result<()> {
        let status = match execution.status {
            ExecutionStatus::Passed => "Passed",
            ExecutionStatus::Failed => "Failed",
            ExecutionStatus::Blocked => "Blocked",
            ExecutionStatus::Cancelled => "Cancelled",
            ExecutionStatus::Skipped => "Skipped",
            _ => "In Progress",
        };

        let results = if execution.is_complete() {
            execution.overall_result.clone()
        } else {
            None
        };

        self.requirements_commands.update_verification_status(
            verification_id,
            status.to_string(),
            results,
        )?;

        Ok(())
    }

    /// Get verification status from requirements module
    pub fn get_verification_status(&self, verification_id: &Id) -> Option<VerificationStatus> {
        self.requirements_commands.get_verification_status(verification_id)
    }

    /// Get unverified design inputs from requirements module
    pub fn get_unverified_design_inputs(&self) -> Vec<tessera_requirements::UnverifiedInput> {
        self.requirements_commands.get_unverified_design_inputs()
    }

    /// Create test procedures for unverified design inputs
    pub fn create_procedures_for_unverified_inputs(&self) -> Result<Vec<TestProcedure>> {
        let unverified = self.get_unverified_design_inputs();
        let mut procedures = Vec::new();

        for input in unverified {
            let procedure = TestProcedure::new(
                format!("Verify {}", input.input_name),
                format!("Verification procedure for design input: {}", input.input_name),
                ProcedureType::Integration,
            );
            procedures.push(procedure);
        }

        Ok(procedures)
    }

    /// Synchronize verification statuses
    pub fn synchronize_verification_statuses(&mut self, executions: &[TestExecution]) -> Result<()> {
        for execution in executions {
            // Find associated verification IDs from procedure links
            // This is a simplified implementation - in practice, you'd need
            // more sophisticated linking logic
            if let Some(verification_id) = self.find_verification_id_for_execution(execution) {
                self.update_verification_status(&verification_id, execution)?;
            }
        }

        Ok(())
    }

    /// Find verification ID for an execution (placeholder implementation)
    fn find_verification_id_for_execution(&self, _execution: &TestExecution) -> Option<Id> {
        // Placeholder implementation
        // In practice, this would look up the verification ID based on
        // the procedure's linked requirements/design outputs
        None
    }
}

/// Integration statistics
#[derive(Debug, Clone)]
pub struct IntegrationStatistics {
    pub total_linked_verifications: usize,
    pub completed_verifications: usize,
    pub failed_verifications: usize,
    pub pending_verifications: usize,
    pub completion_rate: f64,
}

impl IntegrationStatistics {
    /// Calculate completion rate
    pub fn calculate_completion_rate(completed: usize, total: usize) -> f64 {
        if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tessera_core::ProjectContext;

    #[test]
    fn test_integration_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_ctx = ProjectContext::new(
            temp_dir.path().to_path_buf(),
            "Test Project".to_string(),
            Some("Test Description".to_string()),
        );

        let requirements_commands = RequirementsCommands::new(project_ctx).unwrap();
        let integration = RequirementsIntegration::new(requirements_commands);

        // Basic test that integration can be created
        let unverified = integration.get_unverified_design_inputs();
        assert!(unverified.is_empty()); // Should be empty for new project
    }

    #[test]
    fn test_completion_rate_calculation() {
        let rate = IntegrationStatistics::calculate_completion_rate(7, 10);
        assert_eq!(rate, 70.0);

        let rate_zero = IntegrationStatistics::calculate_completion_rate(0, 0);
        assert_eq!(rate_zero, 0.0);
    }
}