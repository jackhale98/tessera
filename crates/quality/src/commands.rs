use crate::data::*;
use crate::repository::QualityRepository;
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
        
        let source = Text::new("Source:")
            .with_help_message("Source of requirement (customer, regulation, standard, etc.)")
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
        
        let category = Select::new("Category:", categories).prompt()?.to_string();
        
        let requirement = Requirement::new(name, description, source, category);
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
            println!("   Category: {}", req.category);
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
        
        let category = Select::new("Risk category:", risk_categories).prompt()?.to_string();
        
        // Get risk scoring configuration from project
        let prob_config = &self.project_context.metadata.quality_settings.risk_probability_range;
        let impact_config = &self.project_context.metadata.quality_settings.risk_impact_range;
        
        let prob_values = prob_config.values();
        let impact_values = impact_config.values();
        
        let probability_str = Text::new(&format!("Probability [values available: {:?}]:", prob_values))
            .with_default(&prob_values[0].to_string())
            .prompt()?;
        let probability: i32 = probability_str.parse().unwrap_or(prob_values[0]);
        
        let impact_str = Text::new(&format!("Impact [values available: {:?}]:", impact_values))
            .with_default(&impact_values[0].to_string())
            .prompt()?;
        let impact: i32 = impact_str.parse().unwrap_or(impact_values[0]);
        
        let mut risk = Risk::new(name, description, category);
        risk.probability = probability;
        risk.impact = impact;
        risk.update_risk_score(prob_config, impact_config);
        
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
        
        println!("Risk Assessment Results:");
        println!("=======================");
        
        let mut risk_categories = HashMap::new();
        let mut high_risk_count = 0;
        let mut critical_risk_count = 0;
        
        for risk in risks {
            let risk_level = match risk.risk_score {
                score if score >= 0.75 => {
                    critical_risk_count += 1;
                    "Critical"
                },
                score if score >= 0.5 => {
                    high_risk_count += 1;
                    "High"
                },
                score if score >= 0.25 => "Medium",
                _ => "Low",
            };
            
            *risk_categories.entry(risk.category.clone()).or_insert(0) += 1;
            
            println!("• {} - Score: {:.2} ({}) - {}", 
                     risk.name, risk.risk_score, risk_level, risk.category);
        }
        
        println!("\nSummary:");
        println!("Critical Risks: {}", critical_risk_count);
        println!("High Risks: {}", high_risk_count);
        
        println!("\nRisk Categories:");
        for (category, count) in risk_categories {
            println!("  {}: {} risks", category, count);
        }
        
        if critical_risk_count > 0 {
            println!("\n⚠️  Immediate action required for critical risks!");
        } else if high_risk_count > 0 {
            println!("\n⚠️  Mitigation strategies needed for high risks.");
        } else {
            println!("\n✅ Risk levels are within acceptable limits.");
        }
        
        Ok(())
    }
    
    pub fn show_dashboard(&self) -> Result<()> {
        let requirements = self.repository.get_requirements();
        let inputs = self.repository.get_inputs();
        let outputs = self.repository.get_outputs();
        let verifications = self.repository.get_verifications();
        let risks = self.repository.get_risks();
        
        println!("Quality Dashboard");
        println!("================");
        println!("Requirements: {}", requirements.len());
        println!("Design Inputs: {}", inputs.len());
        println!("Design Outputs: {}", outputs.len());
        println!("Verifications: {}", verifications.len());
        println!("Risks: {}", risks.len());
        
        if !requirements.is_empty() {
            println!("\nRequirements by Status:");
            let mut status_counts = HashMap::new();
            for req in requirements {
                let status = match req.status {
                    RequirementStatus::Draft => "Draft",
                    RequirementStatus::Approved => "Approved",
                    RequirementStatus::Verified => "Verified",
                    RequirementStatus::Closed => "Closed",
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