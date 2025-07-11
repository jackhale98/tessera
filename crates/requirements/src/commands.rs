//! Commands for requirements management
//!
//! This module provides the command interface for interacting with requirements,
//! design inputs, outputs, and verifications.

use crate::data::*;
use crate::repository::RequirementsRepository;
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text, Confirm};
use colored::Colorize;
use comfy_table::Table;

/// Command handler for requirements operations
pub struct RequirementsCommands {
    repository: RequirementsRepository,
    project_context: ProjectContext,
}

impl RequirementsCommands {
    /// Create a new requirements command handler
    pub fn new(project_context: ProjectContext) -> Result<Self> {
        let repository = RequirementsRepository::load_from_project(&project_context)?;
        Ok(Self {
            repository,
            project_context,
        })
    }

    /// Save changes to persistent storage
    pub fn save(&self) -> Result<()> {
        self.repository.save_to_project(&self.project_context)
    }

    // Requirements commands

    /// Add a new requirement interactively
    pub fn add_requirement_interactive(&mut self) -> Result<()> {
        println!("{}", "Adding new requirement".bold().blue());

        let name = Text::new("Requirement name:").prompt()?;
        let description = Text::new("Description:").prompt()?;
        
        let category_options = vec![
            "Functional", "Performance", "Safety", "Security", 
            "Usability", "Reliability", "Maintainability", "Regulatory", "Interface"
        ];
        let category_str = Select::new("Category:", category_options).prompt()?;
        let category = match category_str {
            "Functional" => RequirementCategory::Functional,
            "Performance" => RequirementCategory::Performance,
            "Safety" => RequirementCategory::Safety,
            "Security" => RequirementCategory::Security,
            "Usability" => RequirementCategory::Usability,
            "Reliability" => RequirementCategory::Reliability,
            "Maintainability" => RequirementCategory::Maintainability,
            "Regulatory" => RequirementCategory::Regulatory,
            "Interface" => RequirementCategory::Interface,
            _ => RequirementCategory::Custom(category_str.to_string()),
        };

        let priority_options = vec!["Low", "Medium", "High", "Critical"];
        let priority_str = Select::new("Priority:", priority_options).prompt()?;
        let priority = match priority_str {
            "Low" => Priority::Low,
            "Medium" => Priority::Medium,
            "High" => Priority::High,
            "Critical" => Priority::Critical,
            _ => Priority::Medium,
        };

        let requirement = Requirement::new(name, description, category, priority);
        self.repository.add_requirement(requirement)?;

        self.save()?;
        println!("{}", "✓ Requirement added successfully!".green());
        Ok(())
    }

    /// List all requirements
    pub fn list_requirements(&self) -> Result<()> {
        let requirements = self.repository.get_requirements();
        
        if requirements.is_empty() {
            println!("{}", "No requirements found.".yellow());
            return Ok(());
        }

        let mut table = Table::new();
        table.set_header(vec!["Name", "Category", "Priority", "Status", "Created"]);

        for req in requirements {
            table.add_row(vec![
                req.name.clone(),
                req.category.to_string(),
                req.priority.to_string(),
                req.status.to_string(),
                req.created.format("%Y-%m-%d").to_string(),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    /// Show requirements dashboard
    pub fn show_dashboard(&self) -> Result<()> {
        let stats = self.repository.get_statistics();
        
        println!("{}", "Requirements Dashboard".bold().blue());
        println!("======================");
        println!("Total Requirements: {}", stats.total_requirements);
        println!("Total Design Inputs: {}", stats.total_design_inputs);
        println!("Total Design Outputs: {}", stats.total_design_outputs);
        println!("Total Verifications: {}", stats.total_verifications);
        println!("Verification Completion: {:.1}%", stats.verification_completion_rate);

        if !stats.requirements_by_status.is_empty() {
            println!("\n{}", "Requirements by Status:".bold());
            for (status, count) in &stats.requirements_by_status {
                println!("  {}: {}", status, count);
            }
        }

        if !stats.requirements_by_priority.is_empty() {
            println!("\n{}", "Requirements by Priority:".bold());
            for (priority, count) in &stats.requirements_by_priority {
                println!("  {}: {}", priority, count);
            }
        }

        Ok(())
    }

    // Design inputs commands

    /// Add a new design input interactively
    pub fn add_design_input_interactive(&mut self) -> Result<()> {
        println!("{}", "Adding new design input".bold().blue());

        let requirements = self.repository.get_requirements();
        if requirements.is_empty() {
            println!("{}", "No requirements found. Create requirements first.".yellow());
            return Ok(());
        }

        // Select requirement
        let req_options: Vec<String> = requirements.iter()
            .map(|req| format!("{} - {}", req.name, req.category))
            .collect();
        let req_selection = Select::new("Select requirement:", req_options).prompt()?;
        let req_index = requirements.iter()
            .position(|req| format!("{} - {}", req.name, req.category) == req_selection)
            .unwrap();
        let selected_req = requirements.get(req_index).unwrap();

        let name = Text::new("Design input name:").prompt()?;
        let description = Text::new("Description:").prompt()?;
        let source = Text::new("Source:").prompt()?;

        let input = DesignInput::new(name, description, selected_req.id, source);
        self.repository.add_design_input(input)?;
        self.save()?;

        println!("{}", "✓ Design input added successfully!".green());
        Ok(())
    }

    /// List design inputs
    pub fn list_design_inputs(&self) -> Result<()> {
        let inputs = self.repository.get_design_inputs();
        
        if inputs.is_empty() {
            println!("{}", "No design inputs found.".yellow());
            return Ok(());
        }

        let mut table = Table::new();
        table.set_header(vec!["Name", "Requirement", "Created"]);

        for input in inputs {
            let req_name = self.repository.get_requirement(&input.requirement_id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
                
            table.add_row(vec![
                input.name.clone(),
                req_name,
                input.created.format("%Y-%m-%d").to_string(),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    // Design outputs commands

    /// Add a new design output interactively
    pub fn add_design_output_interactive(&mut self) -> Result<()> {
        println!("{}", "Adding new design output".bold().blue());

        let inputs = self.repository.get_design_inputs();
        if inputs.is_empty() {
            println!("{}", "No design inputs found. Create design inputs first.".yellow());
            return Ok(());
        }

        // Select design input
        let input_options: Vec<String> = inputs.iter()
            .map(|input| format!("{} - {}", input.name, input.source))
            .collect();
        let input_selection = Select::new("Select design input:", input_options).prompt()?;
        let input_index = inputs.iter()
            .position(|input| format!("{} - {}", input.name, input.source) == input_selection)
            .unwrap();
        let selected_input = inputs.get(input_index).unwrap();

        let name = Text::new("Design output name:").prompt()?;
        let description = Text::new("Description:").prompt()?;
        let output_type = Text::new("Output type (e.g., Document, Code, Drawing):").prompt()?;
        let deliverable = Text::new("Deliverable description:").prompt()?;

        let output = DesignOutput::new(name, description, vec![selected_input.id], output_type, deliverable);
        self.repository.add_design_output(output)?;
        self.save()?;

        println!("{}", "✓ Design output added successfully!".green());
        Ok(())
    }

    /// List design outputs
    pub fn list_design_outputs(&self) -> Result<()> {
        let outputs = self.repository.get_design_outputs();
        
        if outputs.is_empty() {
            println!("{}", "No design outputs found.".yellow());
            return Ok(());
        }

        let mut table = Table::new();
        table.set_header(vec!["Name", "Type", "Input", "Status", "Created"]);

        for output in outputs {
            let input_names: Vec<String> = output.input_ids.iter()
                .filter_map(|id| self.repository.get_design_input(id))
                .map(|i| i.name.clone())
                .collect();
            let input_name = if input_names.is_empty() {
                "Unknown".to_string()
            } else {
                input_names.join(", ")
            };
                
            table.add_row(vec![
                output.name.clone(),
                output.output_type.clone(),
                input_name,
                output.approval_status.clone(),
                output.created.format("%Y-%m-%d").to_string(),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    // Verifications commands

    /// Add a new verification interactively
    pub fn add_verification_interactive(&mut self) -> Result<()> {
        println!("{}", "Adding new verification".bold().blue());

        let inputs = self.repository.get_design_inputs();
        if inputs.is_empty() {
            println!("{}", "No design inputs found. Create design inputs first.".yellow());
            return Ok(());
        }

        // Select design input
        let input_options: Vec<String> = inputs.iter()
            .map(|input| format!("{} - {}", input.name, input.source))
            .collect();
        let input_selection = Select::new("Select design input:", input_options).prompt()?;
        let input_index = inputs.iter()
            .position(|input| format!("{} - {}", input.name, input.source) == input_selection)
            .unwrap();
        let selected_input = inputs.get(input_index).unwrap();

        let name = Text::new("Verification name:").prompt()?;
        let description = Text::new("Description:").prompt()?;
        let verification_type = Text::new("Verification type (e.g., Test, Review, Analysis):").prompt()?;
        let method = Text::new("Verification method:").prompt()?;

        let verification = Verification::new(name, description, vec![selected_input.id], verification_type, method);
        self.repository.add_verification(verification)?;
        self.save()?;

        println!("{}", "✓ Verification added successfully!".green());
        Ok(())
    }

    /// List verifications
    pub fn list_verifications(&self) -> Result<()> {
        let verifications = self.repository.get_verifications();
        
        if verifications.is_empty() {
            println!("{}", "No verifications found.".yellow());
            return Ok(());
        }

        let mut table = Table::new();
        table.set_header(vec!["Name", "Type", "Method", "Status", "Input", "Created"]);

        for verification in verifications {
            let input_names: Vec<String> = verification.input_ids.iter()
                .filter_map(|id| self.repository.get_design_input(id))
                .map(|i| i.name.clone())
                .collect();
            let input_name = if input_names.is_empty() {
                "Unknown".to_string()
            } else {
                input_names.join(", ")
            };
                
            table.add_row(vec![
                verification.name.clone(),
                verification.verification_type.clone(),
                verification.method.clone(),
                verification.status.clone(),
                input_name,
                verification.created.format("%Y-%m-%d").to_string(),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    // Cross-crate integration support

    /// Get verification status for external linking
    /// This will be used by the verification crate to show completion status
    pub fn get_verification_status(&self, verification_id: &tessera_core::Id) -> Option<VerificationStatus> {
        self.repository.get_verification(verification_id)
            .map(|v| VerificationStatus {
                id: v.id,
                name: v.name.clone(),
                status: v.status.clone(),
                is_complete: v.is_complete(),
                output_id: tessera_core::Id::new(), // deprecated field
            })
    }

    /// Update verification status from external source
    /// This will be called by the verification crate when tests complete
    pub fn update_verification_status(&mut self, verification_id: &tessera_core::Id, status: String, results: Option<String>) -> Result<()> {
        if let Some(mut verification) = self.repository.get_verification(verification_id).cloned() {
            verification.update_status(status);
            if let Some(results) = results {
                verification.set_results(results);
            }
            self.repository.update_verification(verification)?;
            self.save()?;
        }
        Ok(())
    }

    /// Get design inputs that need verification
    /// This helps identify which inputs still need verification activities
    pub fn get_unverified_design_inputs(&self) -> Vec<UnverifiedInput> {
        let mut unverified = Vec::new();
        
        for input in self.repository.get_design_inputs() {
            let outputs = self.repository.get_design_outputs_for_input(&input.id);
            let mut has_verification = false;
            
            for output in outputs {
                let verifications = self.repository.get_verifications_for_output(&output.id);
                if !verifications.is_empty() {
                    has_verification = true;
                    break;
                }
            }
            
            if !has_verification {
                unverified.push(UnverifiedInput {
                    input_id: input.id,
                    input_name: input.name.clone(),
                    requirement_id: input.requirement_id,
                    requirement_name: self.repository.get_requirement(&input.requirement_id)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                });
            }
        }
        
        unverified
    }
}

/// Status information for cross-crate verification linking
#[derive(Debug, Clone)]
pub struct VerificationStatus {
    pub id: tessera_core::Id,
    pub name: String,
    pub status: String,
    pub is_complete: bool,
    pub output_id: tessera_core::Id,
}

/// Information about design inputs that lack verification
#[derive(Debug, Clone)]
pub struct UnverifiedInput {
    pub input_id: tessera_core::Id,
    pub input_name: String,
    pub requirement_id: tessera_core::Id,
    pub requirement_name: String,
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

        let commands = RequirementsCommands::new(project_ctx);
        assert!(commands.is_ok());
    }

    #[test]
    fn test_verification_status_integration() {
        let temp_dir = TempDir::new().unwrap();
        let project_ctx = ProjectContext::new(
            temp_dir.path().to_path_buf(),
            "Test Project".to_string(),
            Some("Test Description".to_string()),
        );

        let mut commands = RequirementsCommands::new(project_ctx).unwrap();
        
        // Add a complete chain: requirement -> input -> output -> verification
        let mut req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req_id = req.id;
        commands.repository.add_requirement(req).unwrap();

        let input = DesignInput::new(
            "Test Input".to_string(),
            "A test input".to_string(),
            req_id,
            "Test source".to_string(),
        );
        let input_id = input.id;
        commands.repository.add_design_input(input).unwrap();

        let output = DesignOutput::new(
            "Test Output".to_string(),
            "A test output".to_string(),
            input_id,
            "Document".to_string(),
            "Test deliverable".to_string(),
        );
        let output_id = output.id;
        commands.repository.add_design_output(output).unwrap();

        let verification = Verification::new(
            "Test Verification".to_string(),
            "A test verification".to_string(),
            output_id,
            "Test".to_string(),
            "Automated test".to_string(),
        );
        let verification_id = verification.id;
        commands.repository.add_verification(verification).unwrap();

        // Test verification status retrieval
        let status = commands.get_verification_status(&verification_id).unwrap();
        assert_eq!(status.name, "Test Verification");
        assert_eq!(status.status, "Planned");
        assert!(!status.is_complete);

        // Test status update
        commands.update_verification_status(&verification_id, "Passed".to_string(), Some("All tests passed".to_string())).unwrap();
        
        let updated_status = commands.get_verification_status(&verification_id).unwrap();
        assert_eq!(updated_status.status, "Passed");
        assert!(updated_status.is_complete);
    }
}