use crate::{QualityRepository, QualityRiskScorer, RiskThresholds};
use tessera_core::Result;
use inquire::{
    Confirm, CustomType, Select,
    validator::Validation,
};
use colored::Colorize;

/// Menu result type for navigation
#[derive(Debug)]
pub enum ScoringMenuResult {
    Continue,
    Exit,
    GoBack,
}

/// Simplified scoring menu for risk assessment
pub struct ScoringMenu {
    scorer: QualityRiskScorer,
}

impl ScoringMenu {
    pub fn new() -> Self {
        Self {
            scorer: QualityRiskScorer::new(),
        }
    }

    pub fn run(&mut self, repository: &QualityRepository) -> Result<ScoringMenuResult> {
        loop {
            match self.show_menu(repository)? {
                ScoringMenuResult::Continue => continue,
                ScoringMenuResult::Exit => return Ok(ScoringMenuResult::Exit),
                ScoringMenuResult::GoBack => return Ok(ScoringMenuResult::GoBack),
            }
        }
    }

    fn show_menu(&mut self, repository: &QualityRepository) -> Result<ScoringMenuResult> {
        println!("\n{}", "=== Risk Scoring Menu ===".bold().cyan());
        
        let options = vec![
            "Calculate Risk Scores",
            "Show Risk Assessment",
            "Configure Risk Thresholds",
            "Go Back",
        ];
        
        let choice = Select::new("Choose an option:", options)
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Validation(e.to_string()))?;

        match choice {
            "Calculate Risk Scores" => {
                self.calculate_risk_scores(repository)?;
                Ok(ScoringMenuResult::Continue)
            }
            "Show Risk Assessment" => {
                self.show_risk_assessment(repository)?;
                Ok(ScoringMenuResult::Continue)
            }
            "Configure Risk Thresholds" => {
                self.configure_thresholds()?;
                Ok(ScoringMenuResult::Continue)
            }
            "Go Back" => Ok(ScoringMenuResult::GoBack),
            _ => Ok(ScoringMenuResult::Continue),
        }
    }

    fn calculate_risk_scores(&mut self, repository: &QualityRepository) -> Result<()> {
        println!("\n{}", "Calculating Risk Scores...".bold().yellow());
        
        let risks = repository.get_risks();
        
        if risks.is_empty() {
            println!("{}", "No risks found to score.".yellow());
            return Ok(());
        }

        let assessment = self.scorer.assess_project_risk(risks)?;
        
        println!("{}", format!("✓ Calculated risk scores for {} risks", risks.len()).green());
        
        // Show summary by risk level
        self.display_project_risk_assessment(&assessment);
        
        Ok(())
    }

    fn show_risk_assessment(&mut self, repository: &QualityRepository) -> Result<()> {
        println!("\n{}", "Risk Assessment Report".bold().cyan());
        
        let risks = repository.get_risks();
        
        if risks.is_empty() {
            println!("{}", "No risks found.".yellow());
            return Ok(());
        }

        let assessment = self.scorer.assess_project_risk(risks)?;
        
        println!("\\n{}", "Overall Risk Assessment:".bold());
        println!("  Total Risks: {}", assessment.total_risks);
        println!("  {} Low Risk: {}", "🟢".green(), assessment.low_risks);
        println!("  {} Medium Risk: {}", "🟡".yellow(), assessment.medium_risks);
        println!("  {} High Risk: {}", "🟠".red(), assessment.high_risks);
        println!("  {} Critical Risk: {}", "🔴".bright_red(), assessment.critical_risks);
        println!("  Average Score: {:.2}", assessment.average_score);
        println!("  Overall Level: {:?}", assessment.overall_risk_level);
        
        println!("\\n{}", "Individual Risk Scores:".bold());
        for score in &assessment.individual_scores {
            let risk_color = match score.risk_level {
                crate::RiskLevel::Low => "🟢",
                crate::RiskLevel::Medium => "🟡",
                crate::RiskLevel::High => "🟠",
                crate::RiskLevel::Critical => "🔴",
            };
            println!("  {} {} - {:?} (P: {:.2}, I: {:.2}, Score: {:.2})", 
                     risk_color, score.risk_name, score.risk_level, 
                     score.probability, score.impact, score.total_score);
        }
        
        Ok(())
    }

    fn configure_thresholds(&mut self) -> Result<()> {
        println!("\\n{}", "Configure Risk Thresholds".bold().cyan());
        
        let current = &self.scorer.risk_thresholds;
        println!("Current thresholds:");
        println!("  Low: {}", current.low_threshold);
        println!("  Medium: {}", current.medium_threshold);
        println!("  High: {}", current.high_threshold);
        
        let update = Confirm::new("Update thresholds?")
            .with_default(false)
            .prompt()
            .map_err(|e| tessera_core::DesignTrackError::Validation(e.to_string()))?;
        
        if update {
            let low = CustomType::<f64>::new("Low threshold (0.0-1.0):")
                .with_default(current.low_threshold)
                .with_validator(|val: &f64| {
                    if *val >= 0.0 && *val <= 1.0 {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid("Must be between 0.0 and 1.0".into()))
                    }
                })
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Validation(e.to_string()))?;
            
            let medium = CustomType::<f64>::new("Medium threshold (0.0-1.0):")
                .with_default(current.medium_threshold)
                .with_validator(|val: &f64| {
                    if *val >= 0.0 && *val <= 1.0 {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid("Must be between 0.0 and 1.0".into()))
                    }
                })
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Validation(e.to_string()))?;
            
            let high = CustomType::<f64>::new("High threshold (0.0-1.0):")
                .with_default(current.high_threshold)
                .with_validator(|val: &f64| {
                    if *val >= 0.0 && *val <= 1.0 {
                        Ok(Validation::Valid)
                    } else {
                        Ok(Validation::Invalid("Must be between 0.0 and 1.0".into()))
                    }
                })
                .prompt()
                .map_err(|e| tessera_core::DesignTrackError::Validation(e.to_string()))?;
            
            self.scorer.risk_thresholds = RiskThresholds {
                low_threshold: low,
                medium_threshold: medium,
                high_threshold: high,
            };
            
            println!("{}", "✓ Risk thresholds updated successfully!".green());
        }
        
        Ok(())
    }

    fn display_project_risk_assessment(&self, assessment: &crate::ProjectRiskAssessment) {
        println!("\\n{}", "Risk Assessment Summary:".bold().green());
        println!("  Total Risks: {}", assessment.total_risks);
        println!("  {} Low: {}", "🟢".green(), assessment.low_risks);
        println!("  {} Medium: {}", "🟡".yellow(), assessment.medium_risks);
        println!("  {} High: {}", "🟠".red(), assessment.high_risks);
        println!("  {} Critical: {}", "🔴".bright_red(), assessment.critical_risks);
        println!("  Average Score: {:.2}", assessment.average_score);
        println!("  Overall Level: {:?}", assessment.overall_risk_level);
    }
}

impl Default for ScoringMenu {
    fn default() -> Self {
        Self::new()
    }
}