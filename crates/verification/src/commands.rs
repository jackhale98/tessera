//! Commands for verification management
//!
//! This module provides the command interface for test procedures and executions.

use crate::data::*;
use crate::repository::VerificationRepository;
use tessera_core::{ProjectContext, Result};

/// Command handler for verification operations
pub struct VerificationCommands {
    repository: VerificationRepository,
    project_context: ProjectContext,
}

impl VerificationCommands {
    /// Create a new verification command handler
    pub fn new(project_context: ProjectContext) -> Result<Self> {
        let repository = VerificationRepository::load_from_project(&project_context)?;
        Ok(Self {
            repository,
            project_context,
        })
    }

    /// Save changes to persistent storage
    pub fn save(&self) -> Result<()> {
        self.repository.save_to_project(&self.project_context)
    }

    /// Show verification dashboard
    pub fn show_dashboard(&self) -> Result<()> {
        let stats = self.repository.get_statistics();
        
        println!("Verification Dashboard");
        println!("=====================");
        println!("Total Procedures: {}", stats.total_procedures);
        println!("Total Executions: {}", stats.total_executions);
        println!("Passed Executions: {}", stats.passed_executions);
        println!("Failed Executions: {}", stats.failed_executions);
        println!("Pass Rate: {:.1}%", stats.pass_rate);

        if !stats.procedures_by_type.is_empty() {
            println!("\nProcedures by Type:");
            for (proc_type, count) in &stats.procedures_by_type {
                println!("  {}: {}", proc_type, count);
            }
        }

        if !stats.procedures_by_status.is_empty() {
            println!("\nProcedures by Status:");
            for (status, count) in &stats.procedures_by_status {
                println!("  {}: {}", status, count);
            }
        }

        if !stats.executions_by_status.is_empty() {
            println!("\nExecutions by Status:");
            for (status, count) in &stats.executions_by_status {
                println!("  {}: {}", status, count);
            }
        }

        Ok(())
    }

    // Placeholder methods for future implementation
    pub fn add_procedure_interactive(&mut self) -> Result<()> {
        println!("Interactive procedure creation not yet implemented");
        Ok(())
    }

    pub fn list_procedures(&self) -> Result<()> {
        println!("Procedure listing not yet implemented");
        Ok(())
    }

    pub fn execute_procedure_interactive(&mut self) -> Result<()> {
        println!("Interactive procedure execution not yet implemented");
        Ok(())
    }

    pub fn list_executions(&self) -> Result<()> {
        println!("Execution listing not yet implemented");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_commands_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_ctx = ProjectContext::new(
            temp_dir.path().to_path_buf(),
            "Test Project".to_string(),
            Some("Test Description".to_string()),
        );

        let commands = VerificationCommands::new(project_ctx);
        assert!(commands.is_ok());
    }
}