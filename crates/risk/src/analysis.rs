//! Risk analysis utilities
//!
//! This module provides advanced risk analysis capabilities.

use crate::data::*;
use crate::repository::RiskRepository;
use tessera_core::Result;

/// Risk analysis engine
pub struct RiskAnalyzer {
    repository: RiskRepository,
}

impl RiskAnalyzer {
    /// Create a new risk analyzer
    pub fn new(repository: RiskRepository) -> Self {
        Self { repository }
    }

    /// Analyze risk trends
    pub fn analyze_risk_trends(&self) -> RiskTrendAnalysis {
        // Placeholder implementation
        RiskTrendAnalysis {
            trend: "Stable".to_string(),
            change_percentage: 0.0,
        }
    }

    /// Analyze risk distribution
    pub fn analyze_risk_distribution(&self) -> RiskDistributionAnalysis {
        let risks = self.repository.get_risks();
        let total = risks.len() as f64;
        
        if total == 0.0 {
            return RiskDistributionAnalysis {
                low_percentage: 0.0,
                medium_percentage: 0.0,
                high_percentage: 0.0,
                critical_percentage: 0.0,
            };
        }
        
        let low_count = risks.iter().filter(|r| r.risk_level == RiskLevel::Low).count() as f64;
        let medium_count = risks.iter().filter(|r| r.risk_level == RiskLevel::Medium).count() as f64;
        let high_count = risks.iter().filter(|r| r.risk_level == RiskLevel::High).count() as f64;
        let critical_count = risks.iter().filter(|r| r.risk_level == RiskLevel::Critical).count() as f64;
        
        RiskDistributionAnalysis {
            low_percentage: (low_count / total) * 100.0,
            medium_percentage: (medium_count / total) * 100.0,
            high_percentage: (high_count / total) * 100.0,
            critical_percentage: (critical_count / total) * 100.0,
        }
    }
}

/// Risk trend analysis results
#[derive(Debug, Clone)]
pub struct RiskTrendAnalysis {
    pub trend: String,
    pub change_percentage: f64,
}

/// Risk distribution analysis results
#[derive(Debug, Clone)]
pub struct RiskDistributionAnalysis {
    pub low_percentage: f64,
    pub medium_percentage: f64,
    pub high_percentage: f64,
    pub critical_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_distribution_analysis() {
        let mut repo = RiskRepository::new();
        
        let mut risk1 = Risk::new(
            "Low Risk".to_string(),
            "A low risk".to_string(),
            RiskCategory::Technical,
        );
        risk1.update_scores(1, 1, None);
        repo.add_risk(risk1).unwrap();
        
        let mut risk2 = Risk::new(
            "High Risk".to_string(),
            "A high risk".to_string(),
            RiskCategory::Safety,
        );
        risk2.update_scores(4, 4, None);
        repo.add_risk(risk2).unwrap();
        
        let analyzer = RiskAnalyzer::new(repo);
        let distribution = analyzer.analyze_risk_distribution();
        
        assert!(distribution.low_percentage > 0.0);
        assert!(distribution.high_percentage > 0.0);
        assert_eq!(distribution.low_percentage + distribution.medium_percentage + distribution.high_percentage + distribution.critical_percentage, 100.0);
    }
}