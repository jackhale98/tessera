use crate::QualityCommands;
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use tessera_quality::*;

pub async fn execute_quality_command(command: QualityCommands, project_ctx: ProjectContext) -> Result<()> {
    match command {
        QualityCommands::Dashboard => show_quality_dashboard(project_ctx).await,
    }
}

async fn show_quality_dashboard(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Quality Dashboard".bold().blue());
    
    let quality_dir = project_ctx.module_path("quality");
    let repo = QualityRepository::load_from_directory(&quality_dir)?;
    
    let requirements = repo.get_requirements();
    let inputs = repo.get_inputs();
    let outputs = repo.get_outputs();
    let verifications = repo.get_verifications();
    let risks = repo.get_risks();
    
    println!("\n{}", "Summary".bold());
    println!("Requirements: {}", requirements.len());
    println!("Design Inputs: {}", inputs.len());
    println!("Design Outputs: {}", outputs.len());
    println!("Verifications: {}", verifications.len());
    println!("Risks: {}", risks.len());
    
    if !requirements.is_empty() {
        println!("\n{}", "Requirements by Status:".bold());
        let mut status_counts = std::collections::HashMap::new();
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
        println!("\n{}", "Risk Summary:".bold());
        let high_risk_count = risks.iter().filter(|r| r.risk_score >= 0.7).count();
        let medium_risk_count = risks.iter().filter(|r| r.risk_score >= 0.3 && r.risk_score < 0.7).count();
        let low_risk_count = risks.iter().filter(|r| r.risk_score < 0.3).count();
        
        println!("  High Risk (≥0.7): {}", high_risk_count);
        println!("  Medium Risk (0.3-0.7): {}", medium_risk_count);
        println!("  Low Risk (<0.3): {}", low_risk_count);
    }
    
    Ok(())
}