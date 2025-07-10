use crate::{QualityRepository, TraceabilityMatrix, TraceabilityLink, TraceabilityRelation, SuggestedLink};
use tessera_core::{Id, Result, DesignTrackError};
use inquire::{Confirm, CustomType, Select, Text, InquireError, validator::Validation};
use colored::Colorize;
use std::fs;

/// Menu result type for ESC navigation
#[derive(Debug)]
enum MenuResult<T> {
    Selection(T),
    GoBack,
    Exit,
}

/// Menu interface for traceability matrix management
pub struct TraceabilityMenuInterface {
    matrix: Option<TraceabilityMatrix>,
}

impl TraceabilityMenuInterface {
    pub fn new() -> Self {
        Self {
            matrix: None,
        }
    }

    /// Main traceability menu with ESC navigation
    pub fn show_traceability_menu(&mut self, repository: &mut QualityRepository) -> Result<()> {
        loop {
            println!("\n{}", "=== Quality Traceability Matrix ===".bold().cyan());
            
            // Show matrix status
            if let Some(ref matrix) = self.matrix {
                println!("{}", "Matrix loaded ✓".green());
            } else {
                println!("{}", "No matrix loaded".yellow());
            }

            let choices = vec![
                "Generate Matrix from Repository",
                "View Matrix Summary",
                "Analyze Traceability Gaps", 
                "Add Traceability Link",
                "Remove Traceability Link",
                "Suggest Missing Links",
                "Generate Detailed Report",
                "Export Matrix Data",
                "Back to Main Menu",
            ];

            match self.show_submenu("Traceability Matrix Options:", choices)? {
                MenuResult::Selection(choice) => {
                    match choice {
                        "Generate Matrix from Repository" => self.generate_matrix(repository)?,
                        "View Matrix Summary" => self.view_matrix_summary(repository)?,
                        "Analyze Traceability Gaps" => self.analyze_gaps(repository)?,
                        "Add Traceability Link" => self.add_link(repository)?,
                        "Remove Traceability Link" => self.remove_link(repository)?,
                        "Suggest Missing Links" => self.suggest_links(repository)?,
                        "Generate Detailed Report" => self.generate_report(repository)?,
                        "Export Matrix Data" => self.export_data(repository)?,
                        "Back to Main Menu" => break,
                        _ => unreachable!(),
                    }
                }
                MenuResult::GoBack | MenuResult::Exit => break,
            }
        }
        Ok(())
    }

    /// Generate matrix from repository data
    fn generate_matrix(&mut self, repository: &QualityRepository) -> Result<()> {
        println!("\n{}", "Generating Traceability Matrix...".bold().yellow());
        
        self.matrix = Some(TraceabilityMatrix::from_repository(repository));
        
        if let Some(ref matrix) = self.matrix {
            let req_count = matrix.requirements.len();
            let input_count = matrix.inputs.len();
            let output_count = matrix.outputs.len();
            let control_count = matrix.controls.len();
            let risk_count = matrix.risks.len();
            let link_count = matrix.links.len();

            println!("{}", "✓ Matrix generated successfully".green());
            println!("Entities loaded:");
            println!("  Requirements: {}", req_count);
            println!("  Inputs: {}", input_count);
            println!("  Outputs: {}", output_count);
            println!("  Controls: {}", control_count);
            println!("  Risks: {}", risk_count);
            println!("  Total Links: {}", link_count);
        }

        Ok(())
    }

    /// View matrix summary
    fn view_matrix_summary(&self, repository: &QualityRepository) -> Result<()> {
        let matrix = self.ensure_matrix_loaded()?;
        
        println!("\n{}", "=== Matrix Summary ===".bold().cyan());
        let ascii_matrix = matrix.generate_ascii_matrix(repository);
        println!("{}", ascii_matrix);
        
        Ok(())
    }

    /// Analyze traceability gaps
    fn analyze_gaps(&self, repository: &QualityRepository) -> Result<()> {
        let matrix = self.ensure_matrix_loaded()?;
        
        println!("\n{}", "Analyzing Traceability Gaps...".bold().yellow());
        
        let analysis = matrix.analyze_traceability(repository)?;
        
        println!("\n{}", "=== Gap Analysis Results ===".bold().cyan());
        println!("Overall Completeness: {:.1}%", analysis.completeness_score);
        println!("Requirement Coverage: {:.1}%", analysis.coverage_percentage);
        println!("Total Gaps: {}", analysis.gaps.len());
        
        if !analysis.orphaned_requirements.is_empty() {
            println!("\n{}", "Orphaned Requirements:".red());
            for &req_id in &analysis.orphaned_requirements {
                if let Some(req) = repository.get_requirements().iter().find(|r| r.id == req_id) {
                    println!("  ❌ {} - {}", req_id, req.name);
                }
            }
        }
        
        if !analysis.unverified_outputs.is_empty() {
            println!("\n{}", "Unverified Outputs:".yellow());
            for &output_id in &analysis.unverified_outputs {
                if let Some(output) = repository.get_outputs().iter().find(|o| o.id == output_id) {
                    println!("  ⚠️  {} - {}", output_id, output.name);
                }
            }
        }

        if !analysis.uncontrolled_risks.is_empty() {
            println!("\n{}", "Uncontrolled Risks:".red());
            for &risk_id in &analysis.uncontrolled_risks {
                if let Some(risk) = repository.get_risks().iter().find(|r| r.id == risk_id) {
                    println!("  🔥 {} - {} (Score: {:.2})", risk_id, risk.name, risk.risk_score);
                }
            }
        }

        // Show recommendations
        if !analysis.gaps.is_empty() {
            println!("\n{}", "Recommendations:".bold().green());
            for (i, gap) in analysis.gaps.iter().take(5).enumerate() {
                println!("  {}. {} - {}", i + 1, gap.entity_name, gap.recommendation);
            }
            
            if analysis.gaps.len() > 5 {
                println!("  ... and {} more recommendations", analysis.gaps.len() - 5);
            }
        }

        Ok(())
    }

    /// Add a new traceability link
    fn add_link(&mut self, _repository: &QualityRepository) -> Result<()> {
        println!("\n{}", "Adding Traceability Link".bold().cyan());
        println!("{}", "This feature is temporarily disabled due to borrowing issues".yellow());
        Ok(())
    }

    /// Remove a traceability link
    fn remove_link(&mut self, _repository: &QualityRepository) -> Result<()> {
        println!("{}", "Removing links is temporarily disabled due to borrowing issues".yellow());
        Ok(())
    }

    /// Suggest missing links
    fn suggest_links(&mut self, repository: &QualityRepository) -> Result<()> {
        let matrix = self.ensure_matrix_loaded()?;

        println!("\n{}", "Analyzing for Missing Links...".bold().yellow());
        
        let suggestions = matrix.suggest_missing_links(repository);
        
        if suggestions.is_empty() {
            println!("{}", "No missing links detected. Traceability appears complete!".green());
            return Ok(());
        }

        println!("\n{}", "=== Suggested Links ===".bold().cyan());
        
        for (i, suggestion) in suggestions.iter().enumerate() {
            let source_info = matrix.get_entity_info(suggestion.source_id, repository);
            let target_info = matrix.get_entity_info(suggestion.target_id, repository);
            
            println!("{}. {} → {} ({:?})", 
                     i + 1, source_info.0, target_info.0, suggestion.suggested_relation);
            println!("   Confidence: {:.1}%", suggestion.confidence * 100.0);
            println!("   Reason: {}", suggestion.reason);
            println!();
        }

        if let Some(true) = self.prompt_confirm("Would you like to add any of these suggestions?", false)? {
            self.add_suggested_links(suggestions, repository)?;
        }

        Ok(())
    }

    /// Generate detailed report
    fn generate_report(&self, repository: &QualityRepository) -> Result<()> {
        let matrix = self.ensure_matrix_loaded()?;

        println!("\n{}", "Generating Detailed Report...".bold().yellow());
        
        let report = matrix.generate_detailed_report(repository)?;
        println!("{}", report);

        if let Some(true) = self.prompt_confirm("Save report to file?", true)? {
            let filename = format!("traceability_report_{}.txt", 
                                 chrono::Utc::now().format("%Y%m%d_%H%M%S"));
            
            fs::write(&filename, &report)
                .map_err(|e| DesignTrackError::Module(format!("Failed to save report: {}", e)))?;
            
            println!("{}", format!("✓ Report saved to {}", filename).green());
        }

        Ok(())
    }

    /// Export matrix data
    fn export_data(&self, repository: &QualityRepository) -> Result<()> {
        let matrix = self.ensure_matrix_loaded()?;

        println!("\n{}", "Exporting Matrix Data".bold().cyan());
        
        let format_choices = vec![
            "CSV",
            "JSON", 
            "ASCII Text",
        ];

        let format = match self.show_submenu("Select export format:", format_choices)? {
            MenuResult::Selection(format) => format,
            MenuResult::GoBack => return Ok(()),
            MenuResult::Exit => return Ok(()),
        };

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        
        match format {
            "CSV" => {
                let csv_data = matrix.export_csv(repository)?;
                let filename = format!("traceability_matrix_{}.csv", timestamp);
                fs::write(&filename, csv_data)
                    .map_err(|e| DesignTrackError::Module(format!("Failed to export CSV: {}", e)))?;
                println!("{}", format!("✓ CSV exported to {}", filename).green());
            }
            "ASCII Text" => {
                let ascii_data = matrix.generate_ascii_matrix(repository);
                let filename = format!("traceability_matrix_{}.txt", timestamp);
                fs::write(&filename, ascii_data)
                    .map_err(|e| DesignTrackError::Module(format!("Failed to export ASCII: {}", e)))?;
                println!("{}", format!("✓ ASCII matrix exported to {}", filename).green());
            }
            "JSON" => {
                println!("{}", "JSON export not yet implemented".yellow());
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Helper: Select an entity from repository
    fn select_entity(&self, prompt: &str, repository: &QualityRepository) -> Result<Option<Id>> {
        let mut entity_choices = Vec::new();

        // Add requirements
        for req in repository.get_requirements() {
            entity_choices.push(format!("REQ: {} - {}", req.id, req.name));
        }

        // Add inputs
        for input in repository.get_inputs() {
            entity_choices.push(format!("INP: {} - {}", input.id, input.name));
        }

        // Add outputs
        for output in repository.get_outputs() {
            entity_choices.push(format!("OUT: {} - {}", output.id, output.name));
        }

        // Add controls
        for control in repository.get_controls() {
            entity_choices.push(format!("CTL: {} - {}", control.id, control.name));
        }

        // Add risks
        for risk in repository.get_risks() {
            entity_choices.push(format!("RSK: {} - {}", risk.id, risk.name));
        }

        if entity_choices.is_empty() {
            println!("{}", "No entities available".yellow());
            return Ok(None);
        }

        let selected = match self.prompt_select(prompt, entity_choices)? {
            Some(selected) => selected,
            None => return Ok(None),
        };

        // Extract ID from selection
        let id_str = selected.split(" - ").next()
            .and_then(|part| part.split(": ").nth(1))
            .ok_or_else(|| DesignTrackError::Validation("Invalid entity selection".to_string()))?;

        let id: Id = id_str.parse()
            .map_err(|_| DesignTrackError::Validation("Invalid entity ID".to_string()))?;

        Ok(Some(id))
    }

    /// Add suggested links interactively
    fn add_suggested_links(&mut self, suggestions: Vec<SuggestedLink>, repository: &QualityRepository) -> Result<()> {
        println!("{}", "Adding suggested links is not yet implemented".yellow());
        println!("This feature would allow interactive selection and addition of suggested links");
        Ok(())
    }

    /// Helper: Ensure matrix is loaded
    fn ensure_matrix_loaded(&self) -> Result<&TraceabilityMatrix> {
        self.matrix.as_ref()
            .ok_or_else(|| DesignTrackError::Validation("No matrix loaded. Please generate matrix first.".to_string()))
    }

    /// Helper: Ensure matrix is loaded (mutable)
    fn ensure_matrix_loaded_mut(&mut self) -> Result<&mut TraceabilityMatrix> {
        self.matrix.as_mut()
            .ok_or_else(|| DesignTrackError::Validation("No matrix loaded. Please generate matrix first.".to_string()))
    }

    /// Helper: Get confidence value with validation
    fn prompt_confidence(&self, prompt: &str, default: f32) -> Result<Option<f32>> {
        match CustomType::<f32>::new(prompt)
            .with_default(default)
            .with_validator(|input: &f32| {
                if *input >= 0.0 && *input <= 1.0 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Value must be between 0.0 and 1.0".into()))
                }
            })
            .prompt()
        {
            Ok(value) => Ok(Some(value)),
            Err(InquireError::OperationInterrupted) => Ok(None),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(e) => Err(DesignTrackError::Ui(e.to_string())),
        }
    }

    /// Helper functions following the blueprint pattern for ESC navigation
    fn show_submenu<T: Clone>(&self, prompt: &str, choices: Vec<T>) -> Result<MenuResult<T>> 
    where 
        T: std::fmt::Display,
    {
        let select = Select::new(prompt, choices.clone());
        
        match select.prompt() {
            Ok(choice) => Ok(MenuResult::Selection(choice)),
            Err(InquireError::OperationInterrupted) => Ok(MenuResult::GoBack),
            Err(InquireError::OperationCanceled) => Ok(MenuResult::GoBack),
            Err(e) => Err(DesignTrackError::Ui(e.to_string())),
        }
    }

    fn prompt_select(&self, prompt: &str, choices: Vec<String>) -> Result<Option<String>> {
        match Select::new(prompt, choices).prompt() {
            Ok(choice) => Ok(Some(choice)),
            Err(InquireError::OperationInterrupted) => Ok(None),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(e) => Err(DesignTrackError::Ui(e.to_string())),
        }
    }

    fn prompt_text(&self, prompt: &str) -> Result<Option<String>> {
        match Text::new(prompt).prompt() {
            Ok(text) => Ok(Some(text)),
            Err(InquireError::OperationInterrupted) => Ok(None),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(e) => Err(DesignTrackError::Ui(e.to_string())),
        }
    }

    fn prompt_confirm(&self, prompt: &str, default: bool) -> Result<Option<bool>> {
        match Confirm::new(prompt).with_default(default).prompt() {
            Ok(confirmed) => Ok(Some(confirmed)),
            Err(InquireError::OperationInterrupted) => Ok(None),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(e) => Err(DesignTrackError::Ui(e.to_string())),
        }
    }
}

impl Default for TraceabilityMenuInterface {
    fn default() -> Self {
        Self::new()
    }
}