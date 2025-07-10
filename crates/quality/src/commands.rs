use crate::data::*;
use crate::repository::QualityRepository;
use crate::risk_analysis::{RiskAnalyzer, RiskAnalysisConfig};
use tessera_core::{ProjectContext, Result};
use inquire::{Select, Text};
use std::collections::HashMap;

pub struct QualityCommands {
    repository: QualityRepository,
    project_context: ProjectContext,
}

impl QualityCommands {
    pub fn new(project_context: ProjectContext) -> Result<Self> {
        let quality_dir = project_context.module_path("quality");
        let repository = QualityRepository::load_from_directory(&quality_dir)?;
        
        Ok(Self {
            repository,
            project_context,
        })
    }
    
    pub async fn add_requirement_interactive(&mut self) -> Result<()> {
        let name = Text::new("Requirement name:")
            .with_help_message("Enter a concise name for the requirement")
            .prompt()?;
        
        let description = Text::new("Description:")
            .with_help_message("Detailed description of the requirement")
            .prompt()?;
        
        let categories = vec![
            "Functional",
            "Performance", 
            "Safety",
            "Regulatory",
            "Usability",
            "Reliability",
            "Maintainability",
            "Environmental",
            "Other",
        ];
        
        let category_str = Select::new("Category:", categories).prompt()?;
        let category = match category_str {
            "Functional" => RequirementCategory::Functional,
            "Performance" => RequirementCategory::Performance,
            "Safety" => RequirementCategory::Safety,
            "Regulatory" => RequirementCategory::Regulatory,
            "Usability" => RequirementCategory::Usability,
            "Reliability" => RequirementCategory::Reliability,
            "Maintainability" => RequirementCategory::Maintainability,
            "Environmental" => RequirementCategory::Environmental,
            _ => {
                let other_name = Text::new("Other category name:").prompt()?;
                RequirementCategory::Other(other_name)
            }
        };
        
        let requirement = Requirement::new(name, description, category);
        self.repository.add_requirement(requirement.clone())?;
        
        let quality_dir = self.project_context.module_path("quality");
        self.repository.save_to_directory(&quality_dir)?;
        
        println!("✓ Requirement '{}' added successfully!", requirement.name);
        println!("ID: {}", requirement.id);
        
        Ok(())
    }
    
    pub fn list_requirements(&self) -> Result<()> {
        let requirements = self.repository.get_requirements();
        
        if requirements.is_empty() {
            println!("No requirements found");
            return Ok(());
        }
        
        println!("Requirements:");
        for (i, req) in requirements.iter().enumerate() {
            println!("{}. {} - {}", i + 1, req.name, req.description);
            println!("   ID: {}", req.id);
            println!("   Category: {:?}", req.category);
            println!("   Priority: {:?}", req.priority);
            println!("   Status: {:?}", req.status);
            println!();
        }
        
        Ok(())
    }
    
    pub async fn add_risk_interactive(&mut self) -> Result<()> {
        let name = Text::new("Risk name:")
            .prompt()?;
        
        let description = Text::new("Description:")
            .prompt()?;
        
        let risk_categories = vec![
            "Technical",
            "Schedule",
            "Cost",
            "Quality",
            "Safety",
            "Regulatory",
            "Market",
            "Resource",
            "Other",
        ];
        
        let category_str = Select::new("Risk category:", risk_categories).prompt()?;
        let category = match category_str {
            "Technical" => RiskCategory::Technical,
            "Schedule" => RiskCategory::Schedule,
            "Cost" => RiskCategory::Cost,
            "Quality" => RiskCategory::Quality,
            "Safety" => RiskCategory::Safety,
            "Regulatory" => RiskCategory::Regulatory,
            "Market" => RiskCategory::Market,
            "Resource" => RiskCategory::Resource,
            _ => {
                let other_name = Text::new("Other category name:").prompt()?;
                RiskCategory::Other(other_name)
            }
        };
        
        let probability_str = Text::new("Probability (0.0 - 1.0):")
            .with_default("0.5")
            .prompt()?;
        let probability: f64 = probability_str.parse().unwrap_or(0.5);
        
        let impact_str = Text::new("Impact (0.0 - 1.0):")
            .with_default("0.5")
            .prompt()?;
        let impact: f64 = impact_str.parse().unwrap_or(0.5);
        
        let mut risk = Risk::new(name, description, category);
        risk.probability = probability.clamp(0.0, 1.0);
        risk.impact = impact.clamp(0.0, 1.0);
        risk.update_risk_score();
        
        self.repository.add_risk(risk.clone())?;
        
        let quality_dir = self.project_context.module_path("quality");
        self.repository.save_to_directory(&quality_dir)?;
        
        println!("✓ Risk '{}' added successfully!", risk.name);
        println!("ID: {}", risk.id);
        println!("Risk Score: {:.2}", risk.risk_score);
        
        Ok(())
    }
    
    pub fn assess_risks(&self) -> Result<()> {
        let risks = self.repository.get_risks();
        
        if risks.is_empty() {
            println!("No risks found. Add risks first.");
            return Ok(());
        }
        
        let config = RiskAnalysisConfig::default();
        let analyzer = RiskAnalyzer::new(config);
        
        println!("Running Monte Carlo risk analysis...");
        let analysis = analyzer.analyze_project_risks(risks)?;
        
        println!("\nRisk Analysis Results:");
        println!("Overall Risk Score: {:.2}", analysis.overall_risk_score);
        println!("High Risk Items: {}", analysis.high_risk_items.len());
        
        println!("\nIndividual Risk Analysis:");
        for result in &analysis.individual_risks {
            println!("  {} - Score: {:.3}, 95th Percentile: {:.3}, Recommendation: {:?}",
                     result.risk_name, result.monte_carlo_score, result.percentile_95, result.recommendation);
        }
        
        println!("\nRecommendations:");
        for recommendation in &analysis.recommendations {
            println!("• {}", recommendation);
        }
        
        Ok(())
    }
    
    pub fn show_dashboard(&self) -> Result<()> {
        let requirements = self.repository.get_requirements();
        let inputs = self.repository.get_inputs();
        let outputs = self.repository.get_outputs();
        let controls = self.repository.get_controls();
        let risks = self.repository.get_risks();
        
        println!("Quality Dashboard");
        println!("================");
        println!("Requirements: {}", requirements.len());
        println!("Design Inputs: {}", inputs.len());
        println!("Design Outputs: {}", outputs.len());
        println!("Design Controls: {}", controls.len());
        println!("Risks: {}", risks.len());
        
        if !requirements.is_empty() {
            println!("\nRequirements by Status:");
            let mut status_counts = HashMap::new();
            for req in requirements {
                let status = match req.status {
                    RequirementStatus::Draft => "Draft",
                    RequirementStatus::Approved => "Approved",
                    RequirementStatus::Implemented => "Implemented",
                    RequirementStatus::Verified => "Verified",
                    RequirementStatus::Failed => "Failed",
                    RequirementStatus::Deprecated => "Deprecated",
                };
                *status_counts.entry(status).or_insert(0) += 1;
            }
            
            for (status, count) in status_counts {
                println!("  {}: {}", status, count);
            }
        }
        
        if !risks.is_empty() {
            println!("\nRisk Summary:");
            let high_risk_count = risks.iter().filter(|r| r.risk_score >= 0.7).count();
            let medium_risk_count = risks.iter().filter(|r| r.risk_score >= 0.3 && r.risk_score < 0.7).count();
            let low_risk_count = risks.iter().filter(|r| r.risk_score < 0.3).count();
            
            println!("  High Risk (≥0.7): {}", high_risk_count);
            println!("  Medium Risk (0.3-0.7): {}", medium_risk_count);
            println!("  Low Risk (<0.3): {}", low_risk_count);
        }
        
        Ok(())
    }
}