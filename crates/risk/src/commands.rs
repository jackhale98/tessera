//! Commands for risk management
//!
//! This module provides the command interface for interacting with risks and design controls.

use crate::data::*;
use crate::repository::RiskRepository;
use tessera_core::{ProjectContext, Result};

/// Command handler for risk operations
pub struct RiskCommands {
    repository: RiskRepository,
    project_context: ProjectContext,
}

impl RiskCommands {
    /// Create a new risk command handler
    pub fn new(project_context: ProjectContext) -> Result<Self> {
        let repository = RiskRepository::load_from_project(&project_context)?;
        Ok(Self {
            repository,
            project_context,
        })
    }

    /// Save changes to persistent storage
    pub fn save(&self) -> Result<()> {
        self.repository.save_to_project(&self.project_context)
    }

    /// Simple risk assessment - displays basic risk information
    pub fn assess_risks(&self) -> Result<()> {
        let risks = self.repository.get_risks();
        
        if risks.is_empty() {
            println!("No risks found. Add risks first.");
            return Ok(());
        }
        
        println!("Risk Assessment Results:");
        println!("=======================");
        
        let mut risk_categories = std::collections::HashMap::new();
        let mut high_risk_count = 0;
        let mut critical_risk_count = 0;
        
        for risk in risks {
            let risk_level = match risk.risk_score {
                score if score >= 16.0 => {
                    critical_risk_count += 1;
                    "Critical"
                },
                score if score >= 9.0 => {
                    high_risk_count += 1;
                    "High"
                },
                score if score >= 4.0 => "Medium",
                _ => "Low",
            };
            
            *risk_categories.entry(risk.category.to_string()).or_insert(0) += 1;
            
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

    /// Show risk dashboard
    pub fn show_dashboard(&self) -> Result<()> {
        let stats = self.repository.get_statistics();
        
        println!("Risk Management Dashboard");
        println!("========================");
        println!("Total Risks: {}", stats.total_risks);
        println!("High Priority Risks: {}", stats.high_priority_risks);
        println!("Overdue Risks: {}", stats.overdue_risks);
        println!("Risks Needing Attention: {}", stats.risks_needing_attention);
        println!("Total Design Controls: {}", stats.total_controls);
        println!("Effective Controls: {}", stats.effective_controls);
        println!("Control Effectiveness: {:.1}%", stats.control_effectiveness_rate);

        if !stats.risks_by_level.is_empty() {
            println!("\nRisks by Level:");
            for (level, count) in &stats.risks_by_level {
                println!("  {}: {}", level, count);
            }
        }

        if !stats.risks_by_status.is_empty() {
            println!("\nRisks by Status:");
            for (status, count) in &stats.risks_by_status {
                println!("  {}: {}", status, count);
            }
        }

        Ok(())
    }

    // Placeholder methods for future implementation
    pub fn add_risk_interactive(&mut self) -> Result<()> {
        println!("Interactive risk creation not yet implemented");
        Ok(())
    }

    pub fn list_risks(&self) -> Result<()> {
        println!("Risk listing not yet implemented");
        Ok(())
    }

    pub fn add_design_control_interactive(&mut self) -> Result<()> {
        println!("Interactive design control creation not yet implemented");
        Ok(())
    }

    pub fn list_design_controls(&self) -> Result<()> {
        println!("Design control listing not yet implemented");
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

        let commands = RiskCommands::new(project_ctx);
        assert!(commands.is_ok());
    }
}