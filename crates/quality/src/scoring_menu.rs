use crate::{QualityRepository, QualityRiskScorer, ScoringRule, RiskThresholds, CalculatedRiskScore};
use tessera_core::{Id, Result, DesignTrackError};
use inquire::{
    Confirm, CustomType, Select, Text, InquireError,
    validator::Validation,
};
use colored::Colorize;

/// Menu result type for navigation
#[derive(Debug)]
enum MenuResult<T> {
    Selection(T),
    GoBack,
    Exit,
}

/// Main menu interface for quality risk scoring
pub struct ScoringMenuInterface {
    scorer: QualityRiskScorer,
}

impl ScoringMenuInterface {
    pub fn new() -> Self {
        Self {
            scorer: QualityRiskScorer::new(),
        }
    }

    /// Main scoring menu with ESC navigation
    pub fn show_scoring_menu(&mut self, repository: &mut QualityRepository) -> Result<()> {
        loop {
            println!("\n{}", "=== Quality Risk Scoring ===".bold().cyan());
            
            let choices = vec![
                "Auto-Calculate All Risk Scores",
                "Calculate Single Requirement Risk",
                "View Risk Summary Report",
                "Configure Scoring Rules",
                "Configure Risk Thresholds",
                "Export Risk Data",
                "Back to Main Menu",
            ];

            match self.show_submenu("Quality Risk Scoring Options:", choices)? {
                MenuResult::Selection(choice) => {
                    match choice {
                        "Auto-Calculate All Risk Scores" => self.auto_calculate_all(repository)?,
                        "Calculate Single Requirement Risk" => self.calculate_single_risk(repository)?,
                        "View Risk Summary Report" => self.show_risk_summary(repository)?,
                        "Configure Scoring Rules" => self.configure_scoring_rules()?,
                        "Configure Risk Thresholds" => self.configure_thresholds()?,
                        "Export Risk Data" => self.export_risk_data(repository)?,
                        "Back to Main Menu" => break,
                        _ => unreachable!(),
                    }
                }
                MenuResult::GoBack | MenuResult::Exit => break,
            }
        }
        Ok(())
    }

    /// Auto-calculate risk scores for all requirements
    fn auto_calculate_all(&mut self, repository: &mut QualityRepository) -> Result<()> {
        println!("\n{}", "Auto-Calculating Risk Scores...".bold().yellow());
        
        let results = self.scorer.auto_calculate_all_risks(repository)?;
        
        if results.is_empty() {
            println!("{}", "No requirements found to score.".yellow());
            return Ok(());
        }

        println!("{}", format!("✓ Calculated risk scores for {} requirements", results.len()).green());
        
        // Show summary by risk level
        let summary = self.scorer.generate_risk_summary(&results);
        self.display_risk_summary(&summary);
        
        // Ask if user wants to see details
        if let Some(true) = self.prompt_confirm("Show detailed results?", false)? {
            self.show_detailed_results(&results);
        }

        Ok(())
    }

    /// Calculate risk for a single requirement
    fn calculate_single_risk(&mut self, repository: &QualityRepository) -> Result<()> {
        let requirements = repository.get_all_requirements();
        
        if requirements.is_empty() {
            println!("{}", "No requirements available.".yellow());
            return Ok(());
        }

        // Select requirement
        let req_choices: Vec<String> = requirements.iter()
            .map(|req| format!("{} - {}", req.id, req.name))
            .collect();

        let selected = match self.prompt_select("Select requirement:", req_choices)? {
            Some(selected) => selected,
            None => return Ok(()),
        };

        let req_id_str = selected.split(" - ").next().unwrap();
        let req_id: Id = req_id_str.parse()
            .map_err(|_| DesignTrackError::Validation("Invalid requirement ID".to_string()))?;

        let requirement = requirements.iter()
            .find(|req| req.id == req_id)
            .ok_or_else(|| DesignTrackError::NotFound("Requirement not found".to_string()))?;

        // Calculate risk score
        let score = self.scorer.calculate_requirement_risk_score(requirement, repository)?;
        
        // Display results
        self.display_single_risk_result(&requirement.name, &score);

        Ok(())
    }

    /// Show risk summary report
    fn show_risk_summary(&mut self, repository: &QualityRepository) -> Result<()> {
        println!("\n{}", "Generating Risk Summary Report...".bold().yellow());
        
        let requirements = repository.get_all_requirements();
        let mut scores = Vec::new();
        
        for requirement in requirements {
            if let Ok(score) = self.scorer.calculate_requirement_risk_score(&requirement, repository) {
                scores.push((requirement.id, score));
            }
        }

        let summary = self.scorer.generate_risk_summary(&scores);
        self.display_risk_summary(&summary);

        // Show risk distribution chart
        self.show_risk_distribution(&scores);

        Ok(())
    }

    /// Configure scoring rules
    fn configure_scoring_rules(&mut self) -> Result<()> {
        loop {
            println!("\n{}", "=== Scoring Rules Configuration ===".bold().cyan());
            
            let current_rules = self.scorer.get_scoring_rules();
            println!("Current rules: {} active", current_rules.len());
            
            let choices = vec![
                "View Current Rules",
                "Add New Rule",
                "Enable/Disable Rule",
                "Reset to Defaults",
                "Back",
            ];

            match self.show_submenu("Scoring Rules Options:", choices)? {
                MenuResult::Selection(choice) => {
                    match choice {
                        "View Current Rules" => self.view_current_rules()?,
                        "Add New Rule" => self.add_scoring_rule()?,
                        "Enable/Disable Rule" => self.toggle_rule()?,
                        "Reset to Defaults" => self.reset_rules_to_defaults()?,
                        "Back" => break,
                        _ => unreachable!(),
                    }
                }
                MenuResult::GoBack | MenuResult::Exit => break,
            }
        }
        Ok(())
    }

    /// Configure risk thresholds
    fn configure_thresholds(&mut self) -> Result<()> {
        println!("\n{}", "=== Risk Thresholds Configuration ===".bold().cyan());
        
        let current = self.scorer.get_risk_thresholds();
        println!("Current thresholds:");
        println!("  Low:      0.0 - {:.2}", current.low_threshold);
        println!("  Medium:   {:.2} - {:.2}", current.low_threshold, current.medium_threshold);
        println!("  High:     {:.2} - {:.2}", current.medium_threshold, current.high_threshold);
        println!("  Critical: {:.2} - 1.0", current.high_threshold);

        if let Some(true) = self.prompt_confirm("Update thresholds?", false)? {
            let low = self.prompt_threshold("Low threshold (0.0-1.0):", current.low_threshold)?;
            let medium = self.prompt_threshold("Medium threshold (0.0-1.0):", current.medium_threshold)?;
            let high = self.prompt_threshold("High threshold (0.0-1.0):", current.high_threshold)?;

            if low < medium && medium < high && high <= 1.0 {
                let new_thresholds = RiskThresholds {
                    low_threshold: low,
                    medium_threshold: medium,
                    high_threshold: high,
                    critical_threshold: 1.0,
                };
                
                self.scorer.update_risk_thresholds(new_thresholds);
                println!("{}", "✓ Thresholds updated successfully".green());
            } else {
                println!("{}", "❌ Invalid thresholds - must be in ascending order".red());
            }
        }

        Ok(())
    }

    /// Export risk data
    fn export_risk_data(&mut self, repository: &QualityRepository) -> Result<()> {
        println!("\n{}", "Risk Data Export".bold().cyan());
        println!("{}", "Export functionality would be implemented here".yellow());
        println!("Available formats: CSV, JSON, PDF Report");
        Ok(())
    }

    /// Display detailed risk results
    fn display_single_risk_result(&self, title: &str, score: &CalculatedRiskScore) {
        println!("\n{}", format!("=== Risk Analysis: {} ===", title).bold());
        
        let risk_color = match score.risk_level {
            crate::RiskLevel::Low => "green",
            crate::RiskLevel::Medium => "yellow", 
            crate::RiskLevel::High => "red",
            crate::RiskLevel::Critical => "bright red",
        };
        
        println!("Risk Score: {:.2} ({:?})", score.total_score, score.risk_level);
        println!("Confidence: {:.1}%", score.confidence_level * 100.0);
        println!("Calculated: {}", score.calculated_at.format("%Y-%m-%d %H:%M"));
        
        if !score.contributing_factors.is_empty() {
            println!("\nContributing Factors:");
            for factor in &score.contributing_factors {
                println!("  • {}", factor);
            }
        }
        
        if !score.recommendations.is_empty() {
            println!("\nRecommendations:");
            for (i, rec) in score.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
        }
        
        if !score.score_breakdown.is_empty() {
            println!("\nScore Breakdown:");
            for (rule, value) in &score.score_breakdown {
                println!("  {}: {:.3}", rule, value);
            }
        }
    }

    /// Display risk summary
    fn display_risk_summary(&self, summary: &crate::RiskSummaryReport) {
        println!("\n{}", "=== Risk Summary Report ===".bold());
        println!("Total Requirements: {}", summary.total_requirements);
        println!("Average Risk Score: {:.2}", summary.average_risk_score);
        println!();
        println!("Risk Distribution:");
        println!("  🔴 Critical: {} ({:.1}%)", summary.critical_count, 
                 summary.critical_count as f32 / summary.total_requirements as f32 * 100.0);
        println!("  🟠 High:     {} ({:.1}%)", summary.high_count,
                 summary.high_count as f32 / summary.total_requirements as f32 * 100.0);
        println!("  🟡 Medium:   {} ({:.1}%)", summary.medium_count,
                 summary.medium_count as f32 / summary.total_requirements as f32 * 100.0);
        println!("  🟢 Low:      {} ({:.1}%)", summary.low_count,
                 summary.low_count as f32 / summary.total_requirements as f32 * 100.0);
        println!("Generated: {}", summary.generated_at.format("%Y-%m-%d %H:%M"));
    }

    /// Show detailed results for multiple calculations
    fn show_detailed_results(&self, results: &[(Id, CalculatedRiskScore)]) {
        println!("\n{}", "=== Detailed Risk Results ===".bold());
        
        for (id, score) in results {
            let color = match score.risk_level {
                crate::RiskLevel::Low => "green",
                crate::RiskLevel::Medium => "yellow",
                crate::RiskLevel::High => "red", 
                crate::RiskLevel::Critical => "bright red",
            };
            
            println!("{}: {:.2} ({:?}) - {} factors", 
                     id, score.total_score, score.risk_level, score.contributing_factors.len());
        }
    }

    /// Show risk distribution visualization
    fn show_risk_distribution(&self, scores: &[(Id, CalculatedRiskScore)]) {
        if scores.is_empty() {
            return;
        }

        println!("\n{}", "Risk Distribution Chart:".bold());
        
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        
        for (_, score) in scores {
            match score.risk_level {
                crate::RiskLevel::Critical => critical += 1,
                crate::RiskLevel::High => high += 1,
                crate::RiskLevel::Medium => medium += 1,
                crate::RiskLevel::Low => low += 1,
            }
        }
        
        let total = scores.len();
        let bar_width = 50;
        
        // Simple ASCII bar chart
        println!("Critical [{}] {}", 
                 "█".repeat(critical * bar_width / total.max(1)).red(),
                 critical);
        println!("High     [{}] {}", 
                 "█".repeat(high * bar_width / total.max(1)).bright_red(),
                 high);
        println!("Medium   [{}] {}", 
                 "█".repeat(medium * bar_width / total.max(1)).yellow(),
                 medium);
        println!("Low      [{}] {}", 
                 "█".repeat(low * bar_width / total.max(1)).green(),
                 low);
    }

    /// View current scoring rules
    fn view_current_rules(&self) -> Result<()> {
        let rules = self.scorer.get_scoring_rules();
        
        println!("\n{}", "Current Scoring Rules:".bold());
        for (i, rule) in rules.iter().enumerate() {
            let status = if rule.auto_trigger { "✓" } else { "✗" };
            println!("{}. {} {} (Impact: {:.2}, Weight: {:.2})", 
                     i + 1, status, rule.name, rule.impact_factor, rule.weight);
        }
        
        Ok(())
    }

    /// Add a new scoring rule
    fn add_scoring_rule(&mut self) -> Result<()> {
        println!("{}", "Adding new scoring rule is not yet implemented".yellow());
        println!("This feature would allow custom rule creation");
        Ok(())
    }

    /// Toggle rule on/off
    fn toggle_rule(&mut self) -> Result<()> {
        println!("{}", "Rule toggling is not yet implemented".yellow());
        println!("This feature would allow enabling/disabling specific rules");
        Ok(())
    }

    /// Reset rules to defaults
    fn reset_rules_to_defaults(&mut self) -> Result<()> {
        if let Some(true) = self.prompt_confirm("Reset all rules to defaults?", false)? {
            self.scorer = QualityRiskScorer::new();
            println!("{}", "✓ Rules reset to defaults".green());
        }
        Ok(())
    }

    /// Helper: Get threshold value with validation
    fn prompt_threshold(&self, prompt: &str, default: f32) -> Result<f32> {
        CustomType::<f32>::new(prompt)
            .with_default(default)
            .with_validator(|input: &f32| {
                if *input >= 0.0 && *input <= 1.0 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Value must be between 0.0 and 1.0".into()))
                }
            })
            .prompt()
            .map_err(|e| DesignTrackError::Ui(e.to_string()))
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

    fn prompt_confirm(&self, prompt: &str, default: bool) -> Result<Option<bool>> {
        match Confirm::new(prompt).with_default(default).prompt() {
            Ok(confirmed) => Ok(Some(confirmed)),
            Err(InquireError::OperationInterrupted) => Ok(None),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(e) => Err(DesignTrackError::Ui(e.to_string())),
        }
    }
}

impl Default for ScoringMenuInterface {
    fn default() -> Self {
        Self::new()
    }
}