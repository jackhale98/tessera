use crate::data::*;
use crate::distributions::*;
use crate::sensitivity::*;
use tessera_core::{Result, DesignTrackError};
use serde::{Deserialize, Serialize};

/// Process capability analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCapabilityAnalysis {
    pub stackup_id: String,
    pub stackup_name: String,
    pub specification_limits: SpecificationLimits,
    pub process_statistics: ProcessStatistics,
    pub capability_indices: CapabilityIndices,
    pub performance_metrics: PerformanceMetrics,
    pub quality_assessment: QualityAssessment,
    pub samples: Vec<f64>,
    pub created: chrono::DateTime<chrono::Utc>,
}

/// Specification limits for process capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationLimits {
    pub lower_spec_limit: Option<f64>,
    pub upper_spec_limit: Option<f64>,
    pub target: Option<f64>,
    pub specification_width: Option<f64>,
}

/// Process statistics from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStatistics {
    pub mean: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub sample_size: usize,
}

/// Process capability indices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityIndices {
    pub cp: Option<f64>,    // Process capability (potential)
    pub cpk: Option<f64>,   // Process capability index (actual)
    pub pp: Option<f64>,    // Process performance (potential)
    pub ppk: Option<f64>,   // Process performance index (actual)
    pub cpm: Option<f64>,   // Taguchi capability index
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub yield_percentage: f64,
    pub defect_rate: f64,
    pub sigma_level: f64,
    pub ppm_defects: f64,
    pub ppm_above_usl: f64,
    pub ppm_below_lsl: f64,
}

/// Quality assessment based on capability indices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub overall_rating: QualityRating,
    pub cp_rating: QualityRating,
    pub cpk_rating: QualityRating,
    pub recommendations: Vec<String>,
}

/// Quality rating enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum QualityRating {
    Excellent,    // ≥ 1.67
    Good,         // ≥ 1.33
    Adequate,     // ≥ 1.0
    Marginal,     // ≥ 0.67
    Poor,         // < 0.67
}

/// Process capability analyzer
pub struct ProcessCapabilityAnalyzer {
    distribution_engine: DistributionEngine,
}

impl ProcessCapabilityAnalyzer {
    /// Create new process capability analyzer
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            distribution_engine: DistributionEngine::new(seed),
        }
    }

    /// Analyze process capability for a stackup
    pub fn analyze_stackup(
        &mut self,
        stackup: &Stackup,
        features: &[Feature],
        contributions: &[StackupContribution],
        spec_limits: SpecificationLimits,
        sample_size: usize,
    ) -> Result<ProcessCapabilityAnalysis> {
        // Generate samples using Monte Carlo simulation
        let samples = self.generate_stackup_samples(features, contributions, sample_size)?;
        
        // Calculate process statistics
        let process_stats = self.calculate_process_statistics(&samples);
        
        // Calculate capability indices
        let capability_indices = self.calculate_capability_indices(&process_stats, &spec_limits);
        
        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(&samples, &spec_limits);
        
        // Generate quality assessment
        let quality_assessment = self.assess_quality(&capability_indices, &performance_metrics);

        Ok(ProcessCapabilityAnalysis {
            stackup_id: stackup.id.to_string(),
            stackup_name: stackup.name.clone(),
            specification_limits: spec_limits,
            process_statistics: process_stats,
            capability_indices,
            performance_metrics,
            quality_assessment,
            samples,
            created: chrono::Utc::now(),
        })
    }

    /// Generate samples for the stackup using Monte Carlo
    fn generate_stackup_samples(
        &mut self,
        features: &[Feature],
        contributions: &[StackupContribution],
        sample_size: usize,
    ) -> Result<Vec<f64>> {
        let mut samples = Vec::with_capacity(sample_size);

        for _ in 0..sample_size {
            let mut stackup_value = 0.0;

            for contribution in contributions {
                let feature = features
                    .iter()
                    .find(|f| f.id == contribution.feature_id)
                    .ok_or_else(|| {
                        DesignTrackError::Validation(format!(
                            "Feature {} not found",
                            contribution.feature_id
                        ))
                    })?;

                let tolerance_feature = self.convert_to_tolerance_feature(feature, contribution)?;
                let sample_value = self.distribution_engine.sample_feature_value(&tolerance_feature)?;
                
                // Apply direction and half-count factors
                let direction_factor = contribution.direction;
                let half_count_factor = if contribution.half_count { 0.5 } else { 1.0 };
                
                stackup_value += sample_value * direction_factor * half_count_factor;
            }

            samples.push(stackup_value);
        }

        Ok(samples)
    }

    /// Convert Feature to ToleranceFeature
    fn convert_to_tolerance_feature(
        &self,
        feature: &Feature,
        contribution: &StackupContribution,
    ) -> Result<ToleranceFeature> {
        let distribution_type = match feature.tolerance.distribution {
            ToleranceDistribution::Normal => DistributionType::Normal,
            ToleranceDistribution::Uniform => DistributionType::Uniform,
            ToleranceDistribution::Triangular => DistributionType::Triangular,
            ToleranceDistribution::Beta { .. } => DistributionType::Normal, // Fallback
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

    /// Calculate process statistics from samples
    fn calculate_process_statistics(&self, samples: &[f64]) -> ProcessStatistics {
        let stats = self.distribution_engine.calculate_statistics(samples);
        
        ProcessStatistics {
            mean: stats.mean,
            std_dev: stats.std_dev,
            variance: stats.variance,
            min: stats.min,
            max: stats.max,
            range: stats.max - stats.min,
            sample_size: stats.count,
        }
    }

    /// Calculate capability indices
    fn calculate_capability_indices(
        &self,
        stats: &ProcessStatistics,
        spec_limits: &SpecificationLimits,
    ) -> CapabilityIndices {
        let usl = spec_limits.upper_spec_limit;
        let lsl = spec_limits.lower_spec_limit;
        let target = spec_limits.target;
        let mean = stats.mean;
        let std_dev = stats.std_dev;

        // Calculate Cp (Process Capability)
        let cp = if let (Some(usl), Some(lsl)) = (usl, lsl) {
            Some((usl - lsl) / (6.0 * std_dev))
        } else {
            None
        };

        // Calculate Cpk (Process Capability Index)
        let cpk = match (usl, lsl) {
            (Some(usl), Some(lsl)) => {
                let cpu = (usl - mean) / (3.0 * std_dev);
                let cpl = (mean - lsl) / (3.0 * std_dev);
                Some(cpu.min(cpl))
            }
            (Some(usl), None) => {
                Some((usl - mean) / (3.0 * std_dev))
            }
            (None, Some(lsl)) => {
                Some((mean - lsl) / (3.0 * std_dev))
            }
            (None, None) => None,
        };

        // Pp and Ppk are the same as Cp and Cpk for short-term studies
        let pp = cp;
        let ppk = cpk;

        // Calculate Cpm (Taguchi Capability Index)
        let cpm = if let (Some(usl), Some(lsl), Some(target)) = (usl, lsl, target) {
            let spec_width = usl - lsl;
            let target_variance = (mean - target).powi(2);
            let total_variance = stats.variance + target_variance;
            Some(spec_width / (6.0 * total_variance.sqrt()))
        } else {
            None
        };

        CapabilityIndices {
            cp,
            cpk,
            pp,
            ppk,
            cpm,
        }
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(
        &self,
        samples: &[f64],
        spec_limits: &SpecificationLimits,
    ) -> PerformanceMetrics {
        let total_samples = samples.len() as f64;
        let mut in_spec_count = 0;
        let mut above_usl_count = 0;
        let mut below_lsl_count = 0;

        for &sample in samples {
            let mut in_spec = true;

            if let Some(usl) = spec_limits.upper_spec_limit {
                if sample > usl {
                    above_usl_count += 1;
                    in_spec = false;
                }
            }

            if let Some(lsl) = spec_limits.lower_spec_limit {
                if sample < lsl {
                    below_lsl_count += 1;
                    in_spec = false;
                }
            }

            if in_spec {
                in_spec_count += 1;
            }
        }

        let yield_percentage = (in_spec_count as f64 / total_samples) * 100.0;
        let defect_rate = ((total_samples - in_spec_count as f64) / total_samples) * 100.0;
        let ppm_defects = defect_rate * 10000.0; // Convert to parts per million
        let ppm_above_usl = (above_usl_count as f64 / total_samples) * 1_000_000.0;
        let ppm_below_lsl = (below_lsl_count as f64 / total_samples) * 1_000_000.0;

        // Calculate sigma level based on yield
        let sigma_level = if yield_percentage >= 99.99966 {
            6.0
        } else if yield_percentage >= 99.9937 {
            5.0
        } else if yield_percentage >= 99.87 {
            4.0
        } else if yield_percentage >= 99.73 {
            3.0
        } else if yield_percentage >= 95.45 {
            2.0
        } else {
            1.0
        };

        PerformanceMetrics {
            yield_percentage,
            defect_rate,
            sigma_level,
            ppm_defects,
            ppm_above_usl,
            ppm_below_lsl,
        }
    }

    /// Assess quality based on capability indices
    fn assess_quality(
        &self,
        capability_indices: &CapabilityIndices,
        performance_metrics: &PerformanceMetrics,
    ) -> QualityAssessment {
        let cp_rating = self.rate_capability_index(capability_indices.cp);
        let cpk_rating = self.rate_capability_index(capability_indices.cpk);
        
        // Overall rating is based on the more conservative (worse) of Cp and Cpk
        let overall_rating = match (cp_rating, cpk_rating) {
            (a, b) if a == b => a,
            (a, b) => if self.rating_value(&a) < self.rating_value(&b) { a } else { b },
        };

        let recommendations = self.generate_recommendations(
            capability_indices,
            performance_metrics,
            &overall_rating,
        );

        QualityAssessment {
            overall_rating,
            cp_rating,
            cpk_rating,
            recommendations,
        }
    }

    /// Rate capability index
    fn rate_capability_index(&self, index: Option<f64>) -> QualityRating {
        match index {
            Some(value) if value >= 1.67 => QualityRating::Excellent,
            Some(value) if value >= 1.33 => QualityRating::Good,
            Some(value) if value >= 1.0 => QualityRating::Adequate,
            Some(value) if value >= 0.67 => QualityRating::Marginal,
            Some(_) => QualityRating::Poor,
            None => QualityRating::Poor,
        }
    }

    /// Get numeric value for rating comparison
    fn rating_value(&self, rating: &QualityRating) -> u8 {
        match rating {
            QualityRating::Excellent => 5,
            QualityRating::Good => 4,
            QualityRating::Adequate => 3,
            QualityRating::Marginal => 2,
            QualityRating::Poor => 1,
        }
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(
        &self,
        capability_indices: &CapabilityIndices,
        performance_metrics: &PerformanceMetrics,
        overall_rating: &QualityRating,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        match overall_rating {
            QualityRating::Excellent => {
                recommendations.push("Process is excellent. Consider cost optimization.".to_string());
            }
            QualityRating::Good => {
                recommendations.push("Process is good. Monitor for consistency.".to_string());
            }
            QualityRating::Adequate => {
                recommendations.push("Process is adequate. Look for improvement opportunities.".to_string());
            }
            QualityRating::Marginal => {
                recommendations.push("Process needs improvement. Focus on variance reduction.".to_string());
            }
            QualityRating::Poor => {
                recommendations.push("Process requires immediate attention. Major improvements needed.".to_string());
            }
        }

        // Specific recommendations based on indices
        if let (Some(cp), Some(cpk)) = (capability_indices.cp, capability_indices.cpk) {
            if cp > cpk + 0.2 {
                recommendations.push("Process is not well-centered. Adjust process mean.".to_string());
            }
            if cp < 1.33 {
                recommendations.push("Process spread is too wide. Reduce process variation.".to_string());
            }
        }

        if performance_metrics.sigma_level < 3.0 {
            recommendations.push("Sigma level is below 3. Implement process controls.".to_string());
        }

        if performance_metrics.ppm_defects > 1000.0 {
            recommendations.push("Defect rate is high. Investigate root causes.".to_string());
        }

        recommendations
    }

    /// Generate process capability report
    pub fn generate_report(&self, analysis: &ProcessCapabilityAnalysis) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Process Capability Analysis Report: {}\n", analysis.stackup_name));
        report.push_str(&format!("Generated: {}\n", analysis.created.format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&"=".repeat(80));
        report.push_str("\n\n");

        // Specification limits
        report.push_str("Specification Limits:\n");
        report.push_str(&"-".repeat(30));
        report.push_str("\n");
        if let Some(lsl) = analysis.specification_limits.lower_spec_limit {
            report.push_str(&format!("Lower Specification Limit: {:.6}\n", lsl));
        }
        if let Some(usl) = analysis.specification_limits.upper_spec_limit {
            report.push_str(&format!("Upper Specification Limit: {:.6}\n", usl));
        }
        if let Some(target) = analysis.specification_limits.target {
            report.push_str(&format!("Target Value: {:.6}\n", target));
        }
        report.push_str("\n");

        // Process statistics
        report.push_str("Process Statistics:\n");
        report.push_str(&"-".repeat(30));
        report.push_str("\n");
        report.push_str(&format!("Mean: {:.6}\n", analysis.process_statistics.mean));
        report.push_str(&format!("Standard Deviation: {:.6}\n", analysis.process_statistics.std_dev));
        report.push_str(&format!("Variance: {:.6}\n", analysis.process_statistics.variance));
        report.push_str(&format!("Range: {:.6}\n", analysis.process_statistics.range));
        report.push_str(&format!("Sample Size: {}\n", analysis.process_statistics.sample_size));
        report.push_str("\n");

        // Capability indices
        report.push_str("Capability Indices:\n");
        report.push_str(&"-".repeat(30));
        report.push_str("\n");
        if let Some(cp) = analysis.capability_indices.cp {
            report.push_str(&format!("Cp (Process Capability): {:.4}\n", cp));
        }
        if let Some(cpk) = analysis.capability_indices.cpk {
            report.push_str(&format!("Cpk (Process Capability Index): {:.4}\n", cpk));
        }
        if let Some(pp) = analysis.capability_indices.pp {
            report.push_str(&format!("Pp (Process Performance): {:.4}\n", pp));
        }
        if let Some(ppk) = analysis.capability_indices.ppk {
            report.push_str(&format!("Ppk (Process Performance Index): {:.4}\n", ppk));
        }
        if let Some(cpm) = analysis.capability_indices.cpm {
            report.push_str(&format!("Cpm (Taguchi Index): {:.4}\n", cpm));
        }
        report.push_str("\n");

        // Performance metrics
        report.push_str("Performance Metrics:\n");
        report.push_str(&"-".repeat(30));
        report.push_str("\n");
        report.push_str(&format!("Yield: {:.2}%\n", analysis.performance_metrics.yield_percentage));
        report.push_str(&format!("Defect Rate: {:.2}%\n", analysis.performance_metrics.defect_rate));
        report.push_str(&format!("Sigma Level: {:.1}\n", analysis.performance_metrics.sigma_level));
        report.push_str(&format!("PPM Defects: {:.0}\n", analysis.performance_metrics.ppm_defects));
        report.push_str("\n");

        // Quality assessment
        report.push_str("Quality Assessment:\n");
        report.push_str(&"-".repeat(30));
        report.push_str("\n");
        report.push_str(&format!("Overall Rating: {:?}\n", analysis.quality_assessment.overall_rating));
        report.push_str(&format!("Cp Rating: {:?}\n", analysis.quality_assessment.cp_rating));
        report.push_str(&format!("Cpk Rating: {:?}\n", analysis.quality_assessment.cpk_rating));
        report.push_str("\n");

        // Recommendations
        report.push_str("Recommendations:\n");
        report.push_str(&"-".repeat(30));
        report.push_str("\n");
        for (i, rec) in analysis.quality_assessment.recommendations.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, rec));
        }

        report
    }
}

impl SpecificationLimits {
    /// Create new specification limits
    pub fn new() -> Self {
        Self {
            lower_spec_limit: None,
            upper_spec_limit: None,
            target: None,
            specification_width: None,
        }
    }

    /// Set lower specification limit
    pub fn with_lower_limit(mut self, limit: f64) -> Self {
        self.lower_spec_limit = Some(limit);
        self.update_width();
        self
    }

    /// Set upper specification limit
    pub fn with_upper_limit(mut self, limit: f64) -> Self {
        self.upper_spec_limit = Some(limit);
        self.update_width();
        self
    }

    /// Set target value
    pub fn with_target(mut self, target: f64) -> Self {
        self.target = Some(target);
        self
    }

    /// Update specification width
    fn update_width(&mut self) {
        if let (Some(usl), Some(lsl)) = (self.upper_spec_limit, self.lower_spec_limit) {
            self.specification_width = Some(usl - lsl);
        }
    }

    /// Check if limits are valid
    pub fn is_valid(&self) -> bool {
        match (self.lower_spec_limit, self.upper_spec_limit) {
            (Some(lsl), Some(usl)) => lsl < usl,
            (Some(_), None) | (None, Some(_)) => true,
            (None, None) => false,
        }
    }
}

impl Default for ProcessCapabilityAnalyzer {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Default for SpecificationLimits {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for QualityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityRating::Excellent => write!(f, "Excellent"),
            QualityRating::Good => write!(f, "Good"),
            QualityRating::Adequate => write!(f, "Adequate"),
            QualityRating::Marginal => write!(f, "Marginal"),
            QualityRating::Poor => write!(f, "Poor"),
        }
    }
}