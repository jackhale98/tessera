use crate::{Risk, RiskLevel};
use tessera_core::Result;
use serde::{Deserialize, Serialize};

/// Simple risk assessment calculator
pub struct QualityRiskScorer {
    pub risk_thresholds: RiskThresholds,
}

/// Risk level thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    pub low_threshold: f64,
    pub medium_threshold: f64,
    pub high_threshold: f64,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            low_threshold: 0.25,
            medium_threshold: 0.5,
            high_threshold: 0.75,
        }
    }
}

/// Simple calculated risk score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedRiskScore {
    pub risk_id: tessera_core::Id,
    pub risk_name: String,
    pub probability: f64,
    pub impact: f64,
    pub total_score: f64,
    pub risk_level: RiskLevel,
    pub confidence: f64,
}

impl QualityRiskScorer {
    pub fn new() -> Self {
        Self {
            risk_thresholds: RiskThresholds::default(),
        }
    }

    /// Calculate risk score for a single risk
    pub fn calculate_risk_score(&self, risk: &Risk) -> CalculatedRiskScore {
        let total_score = risk.probability * risk.impact;
        let risk_level = self.determine_risk_level(total_score);
        
        CalculatedRiskScore {
            risk_id: risk.id,
            risk_name: risk.name.clone(),
            probability: risk.probability,
            impact: risk.impact,
            total_score,
            risk_level,
            confidence: 1.0, // Simple model has high confidence
        }
    }

    /// Calculate risk scores for multiple risks
    pub fn calculate_all_risk_scores(&self, risks: &[Risk]) -> Vec<CalculatedRiskScore> {
        risks.iter().map(|risk| self.calculate_risk_score(risk)).collect()
    }

    /// Determine risk level based on score
    fn determine_risk_level(&self, score: f64) -> RiskLevel {
        if score < self.risk_thresholds.low_threshold {
            RiskLevel::Low
        } else if score < self.risk_thresholds.medium_threshold {
            RiskLevel::Medium
        } else if score < self.risk_thresholds.high_threshold {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }

    /// Get overall project risk assessment
    pub fn assess_project_risk(&self, risks: &[Risk]) -> Result<ProjectRiskAssessment> {
        let scores = self.calculate_all_risk_scores(risks);
        
        let mut low_count = 0;
        let mut medium_count = 0;
        let mut high_count = 0;
        let mut critical_count = 0;
        
        let mut total_score = 0.0;
        
        for score in &scores {
            total_score += score.total_score;
            match score.risk_level {
                RiskLevel::Low => low_count += 1,
                RiskLevel::Medium => medium_count += 1,
                RiskLevel::High => high_count += 1,
                RiskLevel::Critical => critical_count += 1,
            }
        }
        
        let average_score = if !scores.is_empty() {
            total_score / scores.len() as f64
        } else {
            0.0
        };
        
        let overall_risk_level = self.determine_risk_level(average_score);
        
        Ok(ProjectRiskAssessment {
            total_risks: risks.len(),
            low_risks: low_count,
            medium_risks: medium_count,
            high_risks: high_count,
            critical_risks: critical_count,
            average_score,
            overall_risk_level,
            individual_scores: scores,
        })
    }
}

impl Default for QualityRiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Overall project risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRiskAssessment {
    pub total_risks: usize,
    pub low_risks: usize,
    pub medium_risks: usize,
    pub high_risks: usize,
    pub critical_risks: usize,
    pub average_score: f64,
    pub overall_risk_level: RiskLevel,
    pub individual_scores: Vec<CalculatedRiskScore>,
}