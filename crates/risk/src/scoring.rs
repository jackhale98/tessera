//! Risk scoring and calculation utilities
//!
//! This module provides risk scoring algorithms and configuration management.

use crate::data::*;
use tessera_core::Result;

/// Risk scoring calculator
pub struct RiskScorer {
    config: RiskAssessmentConfig,
}

impl RiskScorer {
    /// Create a new risk scorer with default configuration
    pub fn new() -> Self {
        Self {
            config: RiskAssessmentConfig::default(),
        }
    }

    /// Create a new risk scorer with custom configuration
    pub fn with_config(config: RiskAssessmentConfig) -> Self {
        Self { config }
    }

    /// Calculate risk score for a risk
    pub fn calculate_risk_score(&self, risk: &Risk) -> f64 {
        let normalized_probability = self.config.normalize_score(risk.probability, self.config.probability_scale);
        let normalized_impact = self.config.normalize_score(risk.impact, self.config.impact_scale);
        
        let base_score = normalized_probability * normalized_impact;
        
        // Apply detectability if available
        if let Some(detectability) = risk.detectability {
            if let Some(det_scale) = self.config.detectability_scale {
                let normalized_detectability = self.config.normalize_score(detectability, det_scale);
                // Lower detectability increases risk
                base_score * (1.0 + (1.0 - normalized_detectability))
            } else {
                base_score
            }
        } else {
            base_score
        }
    }

    /// Determine risk level from score
    pub fn determine_risk_level(&self, score: f64) -> RiskLevel {
        self.config.thresholds.risk_level_from_score(score)
    }

    /// Validate risk scores
    pub fn validate_risk_scores(&self, risk: &Risk) -> Result<()> {
        if !self.config.validate_score(risk.probability, self.config.probability_scale) {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Invalid probability score: {}", risk.probability)
            ));
        }

        if !self.config.validate_score(risk.impact, self.config.impact_scale) {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Invalid impact score: {}", risk.impact)
            ));
        }

        if let Some(detectability) = risk.detectability {
            if let Some(det_scale) = self.config.detectability_scale {
                if !self.config.validate_score(detectability, det_scale) {
                    return Err(tessera_core::DesignTrackError::Validation(
                        format!("Invalid detectability score: {}", detectability)
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_scoring() {
        let scorer = RiskScorer::new();
        let mut risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk".to_string(),
            RiskCategory::Technical,
        );
        
        risk.update_scores(3, 4, None);
        let score = scorer.calculate_risk_score(&risk);
        
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_risk_level_determination() {
        let scorer = RiskScorer::new();
        
        assert_eq!(scorer.determine_risk_level(2.0), RiskLevel::Low);
        assert_eq!(scorer.determine_risk_level(6.0), RiskLevel::Medium);
        assert_eq!(scorer.determine_risk_level(12.0), RiskLevel::High);
        assert_eq!(scorer.determine_risk_level(20.0), RiskLevel::Critical);
    }
}