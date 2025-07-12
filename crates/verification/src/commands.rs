//! Commands for verification management
//!
//! This module provides the command interface for test procedures and executions.

use crate::data::*;
use crate::repository::VerificationRepository;
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text, Confirm};
use colored::Colorize;

/// Command handler for verification operations
pub struct VerificationCommands {
    repository: VerificationRepository,
    project_context: ProjectContext,
    last_created_procedure: Option<TestProcedure>,
    last_created_execution: Option<TestExecution>,
    last_updated_procedure: Option<TestProcedure>,
    last_updated_execution: Option<TestExecution>,
}

impl VerificationCommands {
    /// Create a new verification command handler
    pub fn new(project_context: ProjectContext) -> Result<Self> {
        let repository = VerificationRepository::load_from_project(&project_context)?;
        Ok(Self {
            repository,
            project_context,
            last_created_procedure: None,
            last_created_execution: None,
            last_updated_procedure: None,
            last_updated_execution: None,
        })
    }

    /// Save changes to persistent storage
    pub fn save(&self) -> Result<()> {
        self.repository.save_to_project(&self.project_context)
    }
    
    /// Get the last created procedure (for impact analysis)
    pub fn get_last_created_procedure(&self) -> Option<&TestProcedure> {
        self.last_created_procedure.as_ref()
    }
    
    /// Get the last created execution (for impact analysis)
    pub fn get_last_created_execution(&self) -> Option<&TestExecution> {
        self.last_created_execution.as_ref()
    }
    
    /// Get the last updated procedure (for impact analysis)
    pub fn get_last_updated_procedure(&self) -> Option<&TestProcedure> {
        self.last_updated_procedure.as_ref()
    }
    
    /// Get the last updated execution (for impact analysis)
    pub fn get_last_updated_execution(&self) -> Option<&TestExecution> {
        self.last_updated_execution.as_ref()
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

    /// Add a new test procedure interactively
    pub fn add_procedure_interactive(&mut self) -> Result<()> {
        println!("{}", "=== Add Test Procedure ===".cyan().bold());
        
        // Basic information
        let name = Text::new("Procedure name:")
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let description = Text::new("Description:")
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        // Procedure type
        let type_options = vec![
            ProcedureType::Unit,
            ProcedureType::Integration,
            ProcedureType::System,
            ProcedureType::Acceptance,
            ProcedureType::Performance,
            ProcedureType::Security,
            ProcedureType::Usability,
            ProcedureType::Regression,
            ProcedureType::Smoke,
        ];
        
        let procedure_type = Select::new("Procedure type:", type_options)
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        // Create procedure
        let mut procedure = TestProcedure::new(name, description, procedure_type);
        
        // Add at least one step
        println!("{}", "\nAdd test steps (at least one required):".yellow());
        let mut step_number = 1;
        
        loop {
            let step_description = Text::new(&format!("Step {} description:", step_number))
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            let expected_result = Text::new(&format!("Step {} expected result:", step_number))
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            let step = TestStep::new(step_number, step_description, expected_result);
            procedure.add_step(step);
            
            step_number += 1;
            
            let add_another = Confirm::new("Add another step?")
                .with_default(false)
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            if !add_another {
                break;
            }
        }
        
        // Add to repository
        self.repository.add_test_procedure(procedure.clone())?;
        self.last_created_procedure = Some(procedure);
        
        // Save to disk
        self.save()?;
        
        println!("{}", "✓ Test procedure created successfully!".green());
        Ok(())
    }

    pub fn list_procedures(&self) -> Result<()> {
        let procedures = self.repository.get_test_procedures();
        
        if procedures.is_empty() {
            println!("{}", "No test procedures found.".yellow());
            return Ok(());
        }
        
        println!("{}", "=== Test Procedures ===".cyan().bold());
        println!();
        
        for procedure in procedures {
            println!("{} {}", "●".green(), procedure.name.bold());
            println!("  ID: {}", procedure.id.to_string().dimmed());
            println!("  Type: {}", procedure.procedure_type);
            println!("  Status: {}", procedure.status);
            println!("  Steps: {}", procedure.steps.len());
            println!("  Description: {}", procedure.description);
            println!();
        }
        
        Ok(())
    }

    pub fn execute_procedure_interactive(&mut self) -> Result<()> {
        let procedures = self.repository.get_test_procedures();
        
        if procedures.is_empty() {
            println!("{}", "No test procedures available for execution.".yellow());
            return Ok(());
        }
        
        println!("{}", "=== Execute Test Procedure ===".cyan().bold());
        
        // Select procedure to execute
        let procedure_names: Vec<String> = procedures.iter()
            .map(|p| format!("{} ({})", p.name, p.procedure_type))
            .collect();
            
        let selected_name = Select::new("Select procedure to execute:", procedure_names.clone())
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let selected_index = procedure_names.iter().position(|name| name == &selected_name).unwrap();
        let selected_procedure = &procedures[selected_index];
        
        // Check if procedure is executable
        if !selected_procedure.is_executable() {
            println!("{}", "Warning: This procedure is not in an approved/active state.".yellow());
            let continue_anyway = Confirm::new("Continue anyway?")
                .with_default(false)
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            if !continue_anyway {
                return Ok(());
            }
        }
        
        // Create execution
        let execution_name = Text::new("Execution name:")
            .with_default(&format!("{} - {}", selected_procedure.name, chrono::Utc::now().format("%Y-%m-%d %H:%M")))
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let executor = Text::new("Executor (optional):")
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let executor = if executor.trim().is_empty() { None } else { Some(executor) };
        
        let mut execution = TestExecution::new(selected_procedure.id, execution_name);
        execution.start(executor);
        
        // Execute steps
        println!("{}", "\nExecuting procedure steps:".yellow());
        
        for step in &selected_procedure.steps {
            println!("{}", format!("\n--- Step {} ---", step.step_number).cyan());
            println!("Description: {}", step.description);
            println!("Expected: {}", step.expected_result);
            
            let actual_result = Text::new("Actual result:")
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            let status_options = vec![
                ExecutionStatus::Passed,
                ExecutionStatus::Failed,
                ExecutionStatus::Skipped,
                ExecutionStatus::Blocked,
            ];
            
            let status = Select::new("Step result:", status_options)
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            let mut step_result = StepResult::new(step.step_number, status);
            step_result.set_actual_result(actual_result);
            
            execution.add_step_result(step_result);
        }
        
        // Overall execution result
        let overall_status_options = vec![
            ExecutionStatus::Passed,
            ExecutionStatus::Failed,
            ExecutionStatus::Cancelled,
        ];
        
        let overall_status = Select::new("\nOverall execution result:", overall_status_options)
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        execution.complete(overall_status);
        
        // Add to repository
        self.repository.add_test_execution(execution.clone())?;
        self.last_created_execution = Some(execution);
        
        // Save to disk
        self.save()?;
        
        println!("{}", "✓ Test execution completed and saved!".green());
        Ok(())
    }

    pub fn list_executions(&self) -> Result<()> {
        let executions = self.repository.get_test_executions();
        
        if executions.is_empty() {
            println!("{}", "No test executions found.".yellow());
            return Ok(());
        }
        
        println!("{}", "=== Test Executions ===".cyan().bold());
        println!();
        
        for execution in executions {
            let procedure = self.repository.get_test_procedure(&execution.procedure_id)
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown procedure");
                
            println!("{} {}", "●".green(), execution.execution_name.bold());
            println!("  ID: {}", execution.id.to_string().dimmed());
            println!("  Procedure: {}", procedure);
            println!("  Status: {}", execution.status);
            if let Some(executor) = &execution.executor {
                println!("  Executor: {}", executor);
            }
            if let (Some(start), Some(end)) = (execution.started_at, execution.completed_at) {
                println!("  Duration: {} seconds", (end - start).num_seconds());
            }
            println!("  Steps: {}/{} passed", 
                execution.step_results.iter().filter(|r| r.status == ExecutionStatus::Passed).count(),
                execution.step_results.len()
            );
            println!();
        }
        
        Ok(())
    }
    
    /// Edit a test procedure interactively
    pub fn edit_procedure_interactive(&mut self) -> Result<()> {
        let procedures = self.repository.get_test_procedures();
        
        if procedures.is_empty() {
            println!("{}", "No test procedures available to edit.".yellow());
            return Ok(());
        }
        
        println!("{}", "=== Edit Test Procedure ===".cyan().bold());
        
        // Select procedure to edit
        let procedure_names: Vec<String> = procedures.iter()
            .map(|p| format!("{} ({})", p.name, p.procedure_type))
            .collect();
            
        let selected_name = Select::new("Select procedure to edit:", procedure_names.clone())
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let selected_index = procedure_names.iter().position(|name| name == &selected_name).unwrap();
        let selected_procedure = procedures[selected_index].clone();
        let mut updated_procedure = selected_procedure.clone();
        
        // Edit fields
        let edit_options = vec![
            "Name",
            "Description", 
            "Status",
            "Execution Method",
            "Required Environment",
            "Add Tag",
            "Save Changes",
        ];
        
        loop {
            let action = Select::new("What would you like to edit?", edit_options.clone())
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            match action {
                "Name" => {
                    let new_name = Text::new("New name:")
                        .with_default(&updated_procedure.name)
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_procedure.name = new_name;
                }
                "Description" => {
                    let new_description = Text::new("New description:")
                        .with_default(&updated_procedure.description)
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_procedure.description = new_description;
                }
                "Status" => {
                    let status_options = vec![
                        ProcedureStatus::Draft,
                        ProcedureStatus::UnderReview,
                        ProcedureStatus::Approved,
                        ProcedureStatus::Active,
                        ProcedureStatus::Deprecated,
                        ProcedureStatus::Retired,
                    ];
                    let new_status = Select::new("New status:", status_options)
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_procedure.update_status(new_status);
                }
                "Execution Method" => {
                    let method_options = vec![
                        ExecutionMethod::Manual,
                        ExecutionMethod::Automated,
                        ExecutionMethod::SemiAutomated,
                        ExecutionMethod::Scripted,
                        ExecutionMethod::External,
                    ];
                    let new_method = Select::new("New execution method:", method_options)
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_procedure.set_execution_method(new_method);
                }
                "Required Environment" => {
                    let new_env = Text::new("Required environment:")
                        .with_default(updated_procedure.required_environment.as_deref().unwrap_or(""))
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_procedure.set_required_environment(new_env);
                }
                "Add Tag" => {
                    let new_tag = Text::new("New tag:")
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_procedure.add_tag(new_tag);
                }
                "Save Changes" => {
                    break;
                }
                _ => unreachable!(),
            }
        }
        
        // Update in repository
        self.repository.update_test_procedure(updated_procedure.clone())?;
        self.last_updated_procedure = Some(updated_procedure);
        
        // Save to disk
        self.save()?;
        
        println!("{}", "✓ Test procedure updated successfully!".green());
        Ok(())
    }
    
    /// Edit a test execution interactively
    pub fn edit_execution_interactive(&mut self) -> Result<()> {
        let executions = self.repository.get_test_executions();
        
        if executions.is_empty() {
            println!("{}", "No test executions available to edit.".yellow());
            return Ok(());
        }
        
        println!("{}", "=== Edit Test Execution ===".cyan().bold());
        
        // Select execution to edit
        let execution_names: Vec<String> = executions.iter()
            .map(|e| format!("{} ({})", e.execution_name, e.status))
            .collect();
            
        let selected_name = Select::new("Select execution to edit:", execution_names.clone())
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            
        let selected_index = execution_names.iter().position(|name| name == &selected_name).unwrap();
        let selected_execution = executions[selected_index].clone();
        let mut updated_execution = selected_execution.clone();
        
        // Edit fields
        let edit_options = vec![
            "Execution Name",
            "Status",
            "Executor",
            "Environment",
            "Overall Result",
            "Add Defect",
            "Save Changes",
        ];
        
        loop {
            let action = Select::new("What would you like to edit?", edit_options.clone())
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                
            match action {
                "Execution Name" => {
                    let new_name = Text::new("New execution name:")
                        .with_default(&updated_execution.execution_name)
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_execution.execution_name = new_name;
                }
                "Status" => {
                    let status_options = vec![
                        ExecutionStatus::Pending,
                        ExecutionStatus::Running,
                        ExecutionStatus::Passed,
                        ExecutionStatus::Failed,
                        ExecutionStatus::Blocked,
                        ExecutionStatus::Skipped,
                        ExecutionStatus::Cancelled,
                    ];
                    let new_status = Select::new("New status:", status_options)
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_execution.status = new_status;
                }
                "Executor" => {
                    let new_executor = Text::new("Executor:")
                        .with_default(updated_execution.executor.as_deref().unwrap_or(""))
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_execution.executor = if new_executor.trim().is_empty() { None } else { Some(new_executor) };
                }
                "Environment" => {
                    let new_env = Text::new("Environment:")
                        .with_default(updated_execution.environment.as_deref().unwrap_or(""))
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_execution.environment = if new_env.trim().is_empty() { None } else { Some(new_env) };
                }
                "Overall Result" => {
                    let new_result = Text::new("Overall result:")
                        .with_default(updated_execution.overall_result.as_deref().unwrap_or(""))
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_execution.set_overall_result(new_result);
                }
                "Add Defect" => {
                    let new_defect = Text::new("Defect description:")
                        .prompt()
                        .map_err(|e| tessera_core::DesignTrackError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    updated_execution.add_defect(new_defect);
                }
                "Save Changes" => {
                    break;
                }
                _ => unreachable!(),
            }
        }
        
        // Update in repository
        self.repository.update_test_execution(updated_execution.clone())?;
        self.last_updated_execution = Some(updated_execution);
        
        // Save to disk
        self.save()?;
        
        println!("{}", "✓ Test execution updated successfully!".green());
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