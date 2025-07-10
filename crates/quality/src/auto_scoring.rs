use crate::{Requirement, RiskLevel, RequirementStatus, QualityRepository};
use tessera_core::{Id, Result, DesignTrackError};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Auto-calculation engine for quality risk scoring
pub struct QualityRiskScorer {
    scoring_rules: Vec<ScoringRule>,
    risk_thresholds: RiskThresholds,
}

/// Rule-based scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringRule {
    pub name: String,
    pub condition: ScoringCondition,
    pub impact_factor: f32,
    pub weight: f32,
    pub auto_trigger: bool,
}

/// Conditions that trigger risk score adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoringCondition {
    RequirementStatus(RequirementStatus),
    OverdueRequirement { days: i64 },
    MissingTraceability,
    LowTestCoverage { threshold: f32 },
    HighDefectRate { threshold: f32 },
    RequirementComplexity { threshold: i32 },
    ReviewFailure,
    DependencyRisk,
}

/// Risk score thresholds for automated categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    pub low_threshold: f32,      // 0.0 - 0.3
    pub medium_threshold: f32,   // 0.3 - 0.6
    pub high_threshold: f32,     // 0.6 - 0.8
    pub critical_threshold: f32, // 0.8 - 1.0
}

/// Calculated risk score with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedRiskScore {
    pub total_score: f32,
    pub risk_level: RiskLevel,
    pub score_breakdown: HashMap<String, f32>,
    pub contributing_factors: Vec<String>,
    pub recommendations: Vec<String>,
    pub confidence_level: f32,
    pub calculated_at: DateTime<Utc>,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            low_threshold: 0.3,
            medium_threshold: 0.6,
            high_threshold: 0.8,
            critical_threshold: 1.0,
        }
    }
}

impl QualityRiskScorer {
    pub fn new() -> Self {
        Self {
            scoring_rules: Self::create_default_rules(),
            risk_thresholds: RiskThresholds::default(),
        }
    }

    /// Create default scoring rules following best practices
    fn create_default_rules() -> Vec<ScoringRule> {
        vec![
            ScoringRule {
                name: "Incomplete Requirements".to_string(),
                condition: ScoringCondition::RequirementStatus(RequirementStatus::Draft),
                impact_factor: 0.6,
                weight: 1.0,
                auto_trigger: true,
            },
            ScoringRule {
                name: "Failed Requirements".to_string(),
                condition: ScoringCondition::RequirementStatus(RequirementStatus::Failed),
                impact_factor: 0.8,
                weight: 1.5,
                auto_trigger: true,
            },
            ScoringRule {
                name: "Overdue Requirements".to_string(),
                condition: ScoringCondition::OverdueRequirement { days: 7 },
                impact_factor: 0.7,
                weight: 1.2,
                auto_trigger: true,
            },
            ScoringRule {
                name: "Missing Traceability".to_string(),
                condition: ScoringCondition::MissingTraceability,
                impact_factor: 0.5,
                weight: 0.8,
                auto_trigger: true,
            },
            ScoringRule {
                name: "Low Test Coverage".to_string(),
                condition: ScoringCondition::LowTestCoverage { threshold: 70.0 },
                impact_factor: 0.6,
                weight: 1.0,
                auto_trigger: true,
            },
            ScoringRule {
                name: "High Defect Rate".to_string(),
                condition: ScoringCondition::HighDefectRate { threshold: 5.0 },
                impact_factor: 0.7,
                weight: 1.3,
                auto_trigger: true,
            },
            ScoringRule {
                name: "Complex Requirements".to_string(),
                condition: ScoringCondition::RequirementComplexity { threshold: 8 },
                impact_factor: 0.4,
                weight: 0.7,
                auto_trigger: false,
            },
            ScoringRule {
                name: "Review Failures".to_string(),
                condition: ScoringCondition::ReviewFailure,
                impact_factor: 0.5,
                weight: 0.9,
                auto_trigger: true,
            },
        ]
    }

    /// Calculate risk score for a quality requirement
    pub fn calculate_requirement_risk_score(
        &self,
        requirement: &Requirement,
        repository: &QualityRepository,
    ) -> Result<CalculatedRiskScore> {
        let mut total_score = 0.0;
        let mut score_breakdown = HashMap::new();
        let mut contributing_factors = Vec::new();
        let mut total_weight = 0.0;

        // Apply each scoring rule
        for rule in &self.scoring_rules {
            if rule.auto_trigger && self.evaluate_condition(&rule.condition, requirement, repository)? {
                let rule_score = rule.impact_factor * rule.weight;
                total_score += rule_score;
                total_weight += rule.weight;
                
                score_breakdown.insert(rule.name.clone(), rule_score);
                contributing_factors.push(rule.name.clone());
            }
        }

        // Normalize score (0.0 - 1.0)
        let normalized_score = if total_weight > 0.0 {
            (total_score / total_weight).min(1.0)
        } else {
            0.0
        };

        // Determine risk level
        let risk_level = self.determine_risk_level(normalized_score);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&contributing_factors, &risk_level);

        // Calculate confidence based on available data
        let confidence_level = self.calculate_confidence_level(requirement, &contributing_factors);

        Ok(CalculatedRiskScore {
            total_score: normalized_score,
            risk_level,
            score_breakdown,
            contributing_factors,
            recommendations,
            confidence_level,
            calculated_at: Utc::now(),
        })
    }

    /// Evaluate a scoring condition against a requirement
    fn evaluate_condition(
        &self,
        condition: &ScoringCondition,
        requirement: &Requirement,
        _repository: &QualityRepository,
    ) -> Result<bool> {
        match condition {
            ScoringCondition::RequirementStatus(status) => {
                Ok(requirement.status == *status)
            }
            ScoringCondition::OverdueRequirement { days } => {
                if let Some(due_date) = requirement.due_date {
                    let now = Utc::now();
                    let overdue_days = (now - due_date).num_days();
                    Ok(overdue_days >= *days)
                } else {
                    Ok(false)
                }
            }
            ScoringCondition::MissingTraceability => {
                Ok(requirement.traced_to.is_empty() && requirement.traced_from.is_empty())
            }
            ScoringCondition::LowTestCoverage { threshold } => {
                // For now, assume test coverage is in metadata
                if let Some(coverage_str) = requirement.metadata.get("test_coverage") {
                    if let Ok(coverage) = coverage_str.parse::<f32>() {
                        Ok(coverage < *threshold)
                    } else {
                        Ok(true) // Missing or invalid coverage data is risky
                    }
                } else {
                    Ok(true) // No coverage data
                }
            }
            ScoringCondition::HighDefectRate { threshold } => {
                if let Some(defect_str) = requirement.metadata.get("defect_rate") {
                    if let Ok(defect_rate) = defect_str.parse::<f32>() {
                        Ok(defect_rate > *threshold)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false) // No defect data
                }
            }
            ScoringCondition::RequirementComplexity { threshold } => {
                // Simple complexity calculation based on description length and metadata
                let description_complexity = requirement.description.len() / 100;
                let metadata_complexity = requirement.metadata.len();
                let total_complexity = description_complexity + metadata_complexity;
                Ok(total_complexity > *threshold as usize)
            }
            ScoringCondition::ReviewFailure => {
                Ok(requirement.metadata.get("review_status") == Some(&"failed".to_string()))
            }
            ScoringCondition::DependencyRisk => {
                // High dependency count indicates potential risk
                Ok(requirement.traced_to.len() + requirement.traced_from.len() > 10)
            }
        }
    }

    /// Determine risk level based on score
    fn determine_risk_level(&self, score: f32) -> RiskLevel {
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

    /// Generate actionable recommendations based on risk factors
    fn generate_recommendations(&self, factors: &[String], risk_level: &RiskLevel) -> Vec<String> {
        let mut recommendations = Vec::new();

        for factor in factors {
            match factor.as_str() {
                "Incomplete Requirements" => {
                    recommendations.push("Complete requirement definition and acceptance criteria".to_string());
                }
                "Failed Requirements" => {
                    recommendations.push("Review failure reasons and implement corrective actions".to_string());
                }
                "Overdue Requirements" => {
                    recommendations.push("Update schedule and resource allocation for overdue items".to_string());
                }
                "Missing Traceability" => {
                    recommendations.push("Establish traceability links to upstream and downstream items".to_string());
                }
                "Low Test Coverage" => {
                    recommendations.push("Increase test coverage to meet quality standards".to_string());
                }
                "High Defect Rate" => {
                    recommendations.push("Investigate root causes and improve quality processes".to_string());
                }
                "Complex Requirements" => {
                    recommendations.push("Consider breaking down complex requirements into simpler components".to_string());
                }
                "Review Failures" => {
                    recommendations.push("Address review feedback and re-submit for approval".to_string());
                }
                _ => {
                    recommendations.push(format!("Address issue: {}", factor));
                }
            }
        }

        // Add general recommendations based on risk level
        match risk_level {
            RiskLevel::Critical => {
                recommendations.push("⚠️ CRITICAL: Immediate attention required - escalate to management".to_string());
            }
            RiskLevel::High => {
                recommendations.push("🔴 HIGH: Schedule urgent review and mitigation actions".to_string());
            }
            RiskLevel::Medium => {
                recommendations.push("🟡 MEDIUM: Monitor closely and plan mitigation if needed".to_string());
            }
            RiskLevel::Low => {
                recommendations.push("🟢 LOW: Continue monitoring as part of regular reviews".to_string());
            }
        }

        recommendations
    }

    /// Calculate confidence level based on available data
    fn calculate_confidence_level(&self, requirement: &Requirement, factors: &[String]) -> f32 {
        let mut confidence_score = 0.0;
        let mut data_points = 0;

        // Check for key data availability
        if requirement.due_date.is_some() {
            confidence_score += 0.2;
            data_points += 1;
        }

        if !requirement.traced_to.is_empty() || !requirement.traced_from.is_empty() {
            confidence_score += 0.2;
            data_points += 1;
        }

        if requirement.metadata.contains_key("test_coverage") {
            confidence_score += 0.2;
            data_points += 1;
        }

        if requirement.metadata.contains_key("review_status") {
            confidence_score += 0.2;
            data_points += 1;
        }

        if !factors.is_empty() {
            confidence_score += 0.2;
            data_points += 1;
        }

        // Normalize to 0.0-1.0 range
        if data_points > 0 {
            confidence_score / data_points as f32
        } else {
            0.3 // Low confidence if no data
        }
    }

    /// Auto-calculate risk scores for all requirements in repository
    pub fn auto_calculate_all_risks(&self, repository: &mut QualityRepository) -> Result<Vec<(Id, CalculatedRiskScore)>> {
        let mut results = Vec::new();
        let requirements: Vec<_> = repository.get_all_requirements().to_vec();

        for requirement in requirements {
            match self.calculate_requirement_risk_score(&requirement, repository) {
                Ok(score) => {
                    // Update the requirement's risk score
                    let mut updated_req = requirement.clone();
                    updated_req.risk_score = Some(score.total_score as f64);
                    let _ = repository.update_requirement(updated_req); // Ignore errors for now

                    results.push((requirement.id, score));
                }
                Err(e) => {
                    eprintln!("Failed to calculate risk for requirement {}: {}", requirement.id, e);
                }
            }
        }

        Ok(results)
    }

    /// Update scoring rules configuration
    pub fn update_scoring_rules(&mut self, rules: Vec<ScoringRule>) {
        self.scoring_rules = rules;
    }

    /// Update risk thresholds
    pub fn update_risk_thresholds(&mut self, thresholds: RiskThresholds) {
        self.risk_thresholds = thresholds;
    }

    /// Get current scoring configuration
    pub fn get_scoring_rules(&self) -> &[ScoringRule] {
        &self.scoring_rules
    }

    /// Get current risk thresholds
    pub fn get_risk_thresholds(&self) -> &RiskThresholds {
        &self.risk_thresholds
    }

    /// Generate risk summary report
    pub fn generate_risk_summary(&self, scores: &[(Id, CalculatedRiskScore)]) -> RiskSummaryReport {
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;
        let mut total_score = 0.0;

        for (_, score) in scores {
            match score.risk_level {
                RiskLevel::Critical => critical_count += 1,
                RiskLevel::High => high_count += 1,
                RiskLevel::Medium => medium_count += 1,
                RiskLevel::Low => low_count += 1,
            }
            total_score += score.total_score;
        }

        let average_score = if !scores.is_empty() {
            total_score / scores.len() as f32
        } else {
            0.0
        };

        RiskSummaryReport {
            total_requirements: scores.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            average_risk_score: average_score,
            generated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummaryReport {
    pub total_requirements: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub average_risk_score: f32,
    pub generated_at: DateTime<Utc>,
}

impl Default for QualityRiskScorer {
    fn default() -> Self {
        Self::new()
    }
}