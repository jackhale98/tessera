use crate::data::{Risk, RiskCategory};
use tessera_core::{Id, Result};
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAnalysisConfig {
    pub simulations: usize,
    pub confidence_level: f64,
    pub risk_threshold: f64,
}

impl Default for RiskAnalysisConfig {
    fn default() -> Self {
        Self {
            simulations: 10000,
            confidence_level: 0.95,
            risk_threshold: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAnalysisResult {
    pub risk_id: Id,
    pub risk_name: String,
    pub monte_carlo_score: f64,
    pub confidence_interval: (f64, f64),
    pub percentile_95: f64,
    pub percentile_99: f64,
    pub recommendation: RiskRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskRecommendation {
    Accept,
    Monitor,
    Mitigate,
    Avoid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRiskAnalysis {
    pub config: RiskAnalysisConfig,
    pub individual_risks: Vec<RiskAnalysisResult>,
    pub overall_risk_score: f64,
    pub risk_categories: HashMap<String, f64>,
    pub high_risk_items: Vec<Id>,
    pub recommendations: Vec<String>,
}

pub struct RiskAnalyzer {
    config: RiskAnalysisConfig,
}

impl RiskAnalyzer {
    pub fn new(config: RiskAnalysisConfig) -> Self {
        Self { config }
    }
    
    pub fn analyze_risk(&self, risk: &Risk) -> Result<RiskAnalysisResult> {
        let mut rng = rand::thread_rng();
        let mut scores = Vec::new();
        
        let prob_std = 0.1;
        let impact_std = 0.1;
        
        let prob_dist = Normal::new(risk.probability, prob_std).map_err(|e| {
            tessera_core::DesignTrackError::Module(format!("Failed to create probability distribution: {}", e))
        })?;
        
        let impact_dist = Normal::new(risk.impact, impact_std).map_err(|e| {
            tessera_core::DesignTrackError::Module(format!("Failed to create impact distribution: {}", e))
        })?;
        
        for _ in 0..self.config.simulations {
            let prob = prob_dist.sample(&mut rng).clamp(0.0, 1.0);
            let impact = impact_dist.sample(&mut rng).clamp(0.0, 1.0);
            let score = prob * impact;
            scores.push(score);
        }
        
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean_score = scores.iter().sum::<f64>() / scores.len() as f64;
        let percentile_95 = scores[(0.95 * scores.len() as f64) as usize];
        let percentile_99 = scores[(0.99 * scores.len() as f64) as usize];
        
        let confidence_lower = scores[((1.0 - self.config.confidence_level) / 2.0 * scores.len() as f64) as usize];
        let confidence_upper = scores[((1.0 + self.config.confidence_level) / 2.0 * scores.len() as f64) as usize];
        
        let recommendation = if mean_score >= 0.8 {
            RiskRecommendation::Avoid
        } else if mean_score >= 0.6 {
            RiskRecommendation::Mitigate
        } else if mean_score >= 0.3 {
            RiskRecommendation::Monitor
        } else {
            RiskRecommendation::Accept
        };
        
        Ok(RiskAnalysisResult {
            risk_id: risk.id,
            risk_name: risk.name.clone(),
            monte_carlo_score: mean_score,
            confidence_interval: (confidence_lower, confidence_upper),
            percentile_95,
            percentile_99,
            recommendation,
        })
    }
    
    pub fn analyze_project_risks(&self, risks: &[Risk]) -> Result<ProjectRiskAnalysis> {
        let mut individual_risks = Vec::new();
        let mut category_scores: HashMap<String, Vec<f64>> = HashMap::new();
        
        for risk in risks {
            let analysis = self.analyze_risk(risk)?;
            
            let category = match &risk.category {
                RiskCategory::Technical => "Technical",
                RiskCategory::Schedule => "Schedule",
                RiskCategory::Cost => "Cost",
                RiskCategory::Quality => "Quality",
                RiskCategory::Safety => "Safety",
                RiskCategory::Regulatory => "Regulatory",
                RiskCategory::Market => "Market",
                RiskCategory::Resource => "Resource",
                RiskCategory::Other(name) => name,
            };
            
            category_scores.entry(category.to_string())
                .or_insert_with(Vec::new)
                .push(analysis.monte_carlo_score);
            
            individual_risks.push(analysis);
        }
        
        let overall_risk_score = individual_risks.iter()
            .map(|r| r.monte_carlo_score)
            .fold(0.0, |acc, score| acc + score - (acc * score));
        
        let risk_categories: HashMap<String, f64> = category_scores
            .into_iter()
            .map(|(category, scores)| {
                let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
                (category, avg_score)
            })
            .collect();
        
        let high_risk_items: Vec<Id> = individual_risks
            .iter()
            .filter(|r| r.monte_carlo_score >= self.config.risk_threshold)
            .map(|r| r.risk_id)
            .collect();
        
        let recommendations = self.generate_recommendations(&individual_risks, &risk_categories);
        
        Ok(ProjectRiskAnalysis {
            config: self.config.clone(),
            individual_risks,
            overall_risk_score,
            risk_categories,
            high_risk_items,
            recommendations,
        })
    }
    
    fn generate_recommendations(&self, risks: &[RiskAnalysisResult], categories: &HashMap<String, f64>) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if risks.iter().any(|r| r.monte_carlo_score >= 0.8) {
            recommendations.push("Critical risks identified - immediate action required".to_string());
        }
        
        let high_risk_count = risks.iter().filter(|r| r.monte_carlo_score >= 0.6).count();
        if high_risk_count > 0 {
            recommendations.push(format!("{} high-risk items require mitigation strategies", high_risk_count));
        }
        
        for (category, score) in categories {
            if *score >= 0.7 {
                recommendations.push(format!("High risk in {} category - review and mitigate", category));
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("Risk levels are within acceptable limits".to_string());
        }
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Risk, RiskCategory};
    
    #[test]
    fn test_risk_analysis() {
        let config = RiskAnalysisConfig::default();
        let analyzer = RiskAnalyzer::new(config);
        
        let risk = Risk::new(
            "Test Risk".to_string(),
            "A test risk for analysis".to_string(),
            RiskCategory::Technical,
        );
        
        let result = analyzer.analyze_risk(&risk).unwrap();
        
        assert_eq!(result.risk_id, risk.id);
        assert_eq!(result.risk_name, risk.name);
        assert!(result.monte_carlo_score >= 0.0);
        assert!(result.monte_carlo_score <= 1.0);
        assert!(result.confidence_interval.0 <= result.confidence_interval.1);
    }
}