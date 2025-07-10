use crate::data::*;
use crate::distributions::{DistributionEngine, ToleranceFeature};
use tessera_core::{Result, DesignTrackError};
use serde::{Deserialize, Serialize};

/// Sensitivity analysis results for stackup contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityAnalysis {
    pub stackup_id: String,
    pub stackup_name: String,
    pub total_variance: f64,
    pub total_std_dev: f64,
    pub contributions: Vec<SensitivityContribution>,
    pub created: chrono::DateTime<chrono::Utc>,
}

/// Individual feature contribution to stackup variance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityContribution {
    pub component_id: String,
    pub feature_id: String,
    pub feature_name: String,
    pub variance_contribution: f64,
    pub std_dev_contribution: f64,
    pub percentage: f64,
    pub direction: f64,
    pub half_count: bool,
    pub rank: usize,
}

/// Sensitivity analysis engine
pub struct SensitivityAnalyzer {
    distribution_engine: DistributionEngine,
}

impl SensitivityAnalyzer {
    /// Create new sensitivity analyzer
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            distribution_engine: DistributionEngine::new(seed),
        }
    }

    /// Calculate sensitivity analysis for a stackup
    pub fn analyze_stackup(
        &self,
        stackup: &Stackup,
        features: &[Feature],
        contributions: &[StackupContribution],
    ) -> Result<SensitivityAnalysis> {
        if contributions.is_empty() {
            return Err(DesignTrackError::Validation(
                "No contributions provided for sensitivity analysis".to_string(),
            ));
        }

        let mut total_variance = 0.0;
        let mut sensitivity_contributions = Vec::new();

        // Calculate variance contribution for each feature
        for contribution in contributions {
            let feature = features
                .iter()
                .find(|f| f.id == contribution.feature_id)
                .ok_or_else(|| {
                    DesignTrackError::Validation(format!(
                        "Feature {} not found in stackup",
                        contribution.feature_id
                    ))
                })?;

            let tolerance_feature = self.convert_to_tolerance_feature(feature, contribution)?;
            let feature_variance = self.calculate_feature_variance(&tolerance_feature)?;
            
            // Apply direction and half-count factors
            let direction_factor = contribution.direction.abs();
            let half_count_factor = if contribution.half_count { 0.5 } else { 1.0 };
            let multiplier = direction_factor * half_count_factor;
            
            // Variance contribution (multiplier squared because variance)
            let variance_contribution = feature_variance * multiplier.powi(2);
            total_variance += variance_contribution;

            sensitivity_contributions.push(SensitivityContribution {
                component_id: contribution.component_id.clone(),
                feature_id: contribution.feature_id.to_string(),
                feature_name: feature.name.clone(),
                variance_contribution,
                std_dev_contribution: variance_contribution.sqrt(),
                percentage: 0.0, // Will be calculated after total variance is known
                direction: contribution.direction,
                half_count: contribution.half_count,
                rank: 0, // Will be set after sorting
            });
        }

        let total_std_dev = total_variance.sqrt();

        // Calculate percentages and rank by contribution
        for contribution in &mut sensitivity_contributions {
            contribution.percentage = if total_variance > 0.0 {
                (contribution.variance_contribution / total_variance) * 100.0
            } else {
                0.0
            };
        }

        // Sort by percentage contribution (descending)
        sensitivity_contributions.sort_by(|a, b| {
            b.percentage.partial_cmp(&a.percentage).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (i, contribution) in sensitivity_contributions.iter_mut().enumerate() {
            contribution.rank = i + 1;
        }

        Ok(SensitivityAnalysis {
            stackup_id: stackup.id.to_string(),
            stackup_name: stackup.name.clone(),
            total_variance,
            total_std_dev,
            contributions: sensitivity_contributions,
            created: chrono::Utc::now(),
        })
    }

    /// Convert Feature to ToleranceFeature for variance calculation
    fn convert_to_tolerance_feature(
        &self,
        feature: &Feature,
        contribution: &StackupContribution,
    ) -> Result<ToleranceFeature> {
        let distribution_type = match feature.tolerance.distribution {
            ToleranceDistribution::Normal => crate::distributions::DistributionType::Normal,
            ToleranceDistribution::Uniform => crate::distributions::DistributionType::Uniform,
            ToleranceDistribution::Triangular => crate::distributions::DistributionType::Triangular,
            ToleranceDistribution::LogNormal => crate::distributions::DistributionType::LogNormal,
            ToleranceDistribution::Beta { .. } => crate::distributions::DistributionType::Normal, // Fallback
        };

        Ok(ToleranceFeature {
            id: feature.id.to_string(),
            name: feature.name.clone(),
            nominal: feature.nominal,
            plus_tolerance: feature.tolerance.plus,
            minus_tolerance: feature.tolerance.minus,
            distribution: distribution_type,
            direction: contribution.direction,
            half_count: contribution.half_count,
        })
    }

    /// Calculate variance for a tolerance feature
    fn calculate_feature_variance(&self, feature: &ToleranceFeature) -> Result<f64> {
        self.distribution_engine.calculate_feature_variance(feature)
    }

    /// Generate sensitivity report
    pub fn generate_report(&self, analysis: &SensitivityAnalysis) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Sensitivity Analysis Report: {}\n", analysis.stackup_name));
        report.push_str(&format!("Generated: {}\n", analysis.created.format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&"=".repeat(60));
        report.push_str("\n\n");

        report.push_str(&format!("Total Stackup Variance: {:.6}\n", analysis.total_variance));
        report.push_str(&format!("Total Standard Deviation: {:.6}\n", analysis.total_std_dev));
        report.push_str("\n");

        report.push_str("Feature Contributions (Ranked by Impact):\n");
        report.push_str(&"-".repeat(60));
        report.push_str("\n");

        for contribution in &analysis.contributions {
            let impact_level = self.classify_impact(contribution.percentage);
            report.push_str(&format!(
                "{}. {} ({})\n",
                contribution.rank,
                contribution.feature_name,
                contribution.component_id
            ));
            report.push_str(&format!("   Variance Contribution: {:.6}\n", contribution.variance_contribution));
            report.push_str(&format!("   Standard Deviation: {:.6}\n", contribution.std_dev_contribution));
            report.push_str(&format!("   Percentage: {:.2}%\n", contribution.percentage));
            report.push_str(&format!("   Direction: {:.1}\n", contribution.direction));
            report.push_str(&format!("   Half Count: {}\n", contribution.half_count));
            report.push_str(&format!("   Impact Level: {}\n", impact_level));
            report.push_str("\n");
        }

        report.push_str("\nInterpretation Guidelines:\n");
        report.push_str("- High Impact (>50%): Primary focus for tolerance tightening\n");
        report.push_str("- Medium Impact (25-50%): Secondary optimization targets\n");
        report.push_str("- Low Impact (<25%): Minimal effect on variation\n");

        report
    }

    /// Classify impact level based on percentage
    fn classify_impact(&self, percentage: f64) -> &'static str {
        if percentage >= 50.0 {
            "High"
        } else if percentage >= 25.0 {
            "Medium"
        } else {
            "Low"
        }
    }

    /// Generate ASCII bar chart of contributions
    pub fn generate_ascii_chart(&self, analysis: &SensitivityAnalysis) -> String {
        let mut chart = String::new();
        
        chart.push_str("Feature Contribution Chart\n");
        chart.push_str(&"=".repeat(50));
        chart.push_str("\n");

        let max_percentage = analysis.contributions
            .iter()
            .map(|c| c.percentage)
            .fold(0.0f64, |a, b| a.max(b));

        for contribution in &analysis.contributions {
            let bar_length = if max_percentage > 0.0 {
                ((contribution.percentage / max_percentage) * 40.0) as usize
            } else {
                0
            };

            let bar = "█".repeat(bar_length);
            let impact_symbol = match self.classify_impact(contribution.percentage) {
                "High" => "🔴",
                "Medium" => "🟡",
                "Low" => "🟢",
                _ => "⚪",
            };

            chart.push_str(&format!(
                "{} {:20} |{:40}| {:6.2}%\n",
                impact_symbol,
                contribution.feature_name.chars().take(20).collect::<String>(),
                bar,
                contribution.percentage
            ));
        }

        chart
    }
}

/// Stackup contribution data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackupContribution {
    pub component_id: String,
    pub feature_id: tessera_core::Id,
    pub direction: f64,
    pub half_count: bool,
}

impl StackupContribution {
    pub fn new(component_id: String, feature_id: tessera_core::Id) -> Self {
        Self {
            component_id,
            feature_id,
            direction: 1.0,
            half_count: false,
        }
    }
}

/// Sensitivity analysis utility functions
pub mod utils {
    use super::*;

    /// Calculate critical features based on sensitivity analysis
    pub fn identify_critical_features(
        analysis: &SensitivityAnalysis,
        threshold: f64,
    ) -> Vec<&SensitivityContribution> {
        analysis
            .contributions
            .iter()
            .filter(|c| c.percentage >= threshold)
            .collect()
    }

    /// Calculate cumulative contribution percentage
    pub fn calculate_cumulative_percentage(
        contributions: &[SensitivityContribution],
        up_to_rank: usize,
    ) -> f64 {
        contributions
            .iter()
            .take(up_to_rank)
            .map(|c| c.percentage)
            .sum()
    }

    /// Suggest tolerance tightening based on sensitivity analysis
    pub fn suggest_tolerance_improvements(
        analysis: &SensitivityAnalysis,
        target_reduction: f64,
    ) -> Vec<ToleranceImprovement> {
        let mut improvements = Vec::new();

        for contribution in &analysis.contributions {
            if contribution.percentage >= 10.0 {
                // Only suggest improvements for features with >10% contribution
                let current_impact = contribution.variance_contribution;
                let suggested_reduction = (target_reduction / 100.0) * current_impact;
                let new_variance = current_impact - suggested_reduction;
                let improvement_factor = (new_variance / current_impact).sqrt();

                improvements.push(ToleranceImprovement {
                    feature_name: contribution.feature_name.clone(),
                    current_contribution: contribution.percentage,
                    suggested_tolerance_factor: improvement_factor,
                    expected_variance_reduction: suggested_reduction,
                });
            }
        }

        improvements
    }

    #[derive(Debug, Clone)]
    pub struct ToleranceImprovement {
        pub feature_name: String,
        pub current_contribution: f64,
        pub suggested_tolerance_factor: f64,
        pub expected_variance_reduction: f64,
    }
}

impl Default for SensitivityAnalyzer {
    fn default() -> Self {
        Self::new(None)
    }
}