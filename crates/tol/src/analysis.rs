use crate::data::*;
use tessera_core::{Result, Id};
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform, Triangular, LogNormal};
use statrs::distribution::Beta;
use std::fs;
use std::io::Write;

pub struct ToleranceAnalyzer {
    config: AnalysisConfig,
}

impl ToleranceAnalyzer {
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }
    
    pub fn analyze_stackup(&self, stackup: &Stackup, features: &[Feature]) -> Result<StackupAnalysis> {
        let stackup_features: Vec<&Feature> = stackup.dimension_chain
            .iter()
            .filter_map(|&id| features.iter().find(|f| f.id == id))
            .collect();
        
        if stackup_features.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "No valid features found in stackup dimension chain".to_string()
            ));
        }
        
        let results = match self.config.method {
            AnalysisMethod::WorstCase => self.worst_case_analysis(&stackup_features),
            AnalysisMethod::RootSumSquare => self.root_sum_square_analysis(&stackup_features),
            AnalysisMethod::MonteCarlo => self.monte_carlo_analysis(&stackup_features)?,
        };
        
        Ok(StackupAnalysis {
            stackup_id: stackup.id,
            stackup_name: stackup.name.clone(),
            target_dimension: stackup.target_dimension,
            config: self.config.clone(),
            feature_contributions: Vec::new(), // Will be populated by caller
            results,
            created: chrono::Utc::now(),
        })
    }
    
    pub fn analyze_stackup_with_contributions(&self, stackup: &Stackup, features: &[Feature], contributions: &[FeatureContribution]) -> Result<StackupAnalysis> {
        if contributions.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "No feature contributions specified".to_string()
            ));
        }
        
        let stackup_features: Vec<&Feature> = contributions.iter()
            .filter_map(|contrib| features.iter().find(|f| f.id == contrib.feature_id))
            .collect();
        
        if stackup_features.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "No valid features found for contributions".to_string()
            ));
        }
        
        let mut results = match self.config.method {
            AnalysisMethod::WorstCase => self.worst_case_analysis_with_contributions(&stackup_features, contributions),
            AnalysisMethod::RootSumSquare => self.root_sum_square_analysis_with_contributions(&stackup_features, contributions),
            AnalysisMethod::MonteCarlo => self.monte_carlo_analysis_with_contributions(&stackup_features, contributions)?,
        };
        
        // For Monte Carlo analysis, save simulation data to CSV and update results
        if matches!(self.config.method, AnalysisMethod::MonteCarlo) {
            // Generate CSV file only for Monte Carlo analyses
            // We need to re-run a limited simulation to get the sample data for CSV export
            let csv_path = self.save_monte_carlo_simulation_data(&stackup_features, contributions, &stackup.id)?;
            results.distribution_data_file = Some(csv_path);
        }
        
        Ok(StackupAnalysis {
            stackup_id: stackup.id,
            stackup_name: stackup.name.clone(),
            target_dimension: stackup.target_dimension,
            config: self.config.clone(),
            feature_contributions: contributions.to_vec(),
            results,
            created: chrono::Utc::now(),
        })
    }
    
    fn worst_case_analysis(&self, features: &[&Feature]) -> AnalysisResults {
        let nominal_dimension: f64 = features.iter().map(|f| f.nominal).sum();
        let plus_tolerance: f64 = features.iter().map(|f| f.tolerance.plus).sum();
        let minus_tolerance: f64 = features.iter().map(|f| f.tolerance.minus).sum();
        
        let predicted_tolerance = Tolerance {
            plus: plus_tolerance,
            minus: minus_tolerance,
            distribution: ToleranceDistribution::Uniform,
        };
        
        // Conservative estimates for worst case
        let cp = 1.0; // Worst case assumes minimal capability
        let cpk = 0.8;
        let sigma_level = 3.0;
        let yield_percentage = 99.73; // 3-sigma
        
        AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            three_sigma_tolerance: None,
            user_specified_tolerance: None,
            cp,
            cpk,
            sigma_level,
            yield_percentage,
            distribution_data_file: None,
            quartile_data: None,
        }
    }
    
    fn root_sum_square_analysis(&self, features: &[&Feature]) -> AnalysisResults {
        let nominal_dimension: f64 = features.iter().map(|f| f.nominal).sum();
        
        // RSS calculation for normal distributions
        // Calculate total variance from individual feature variances
        let variance: f64 = features.iter()
            .map(|f| {
                // Standard deviation for a tolerance band is (plus + minus) / 6 for 3-sigma
                let std_dev = (f.tolerance.plus + f.tolerance.minus) / 6.0;
                std_dev.powi(2) // variance
            })
            .sum();
        
        let total_std_dev = variance.sqrt();
        
        // For symmetric normal distribution, 3-sigma limits
        let predicted_tolerance = Tolerance {
            plus: 3.0 * total_std_dev,
            minus: 3.0 * total_std_dev,
            distribution: ToleranceDistribution::Normal,
        };
        
        // Calculate process capability metrics
        let usl = nominal_dimension + predicted_tolerance.plus;
        let lsl = nominal_dimension - predicted_tolerance.minus;
        let cp = (usl - lsl) / (6.0 * total_std_dev);
        let cpk = ((usl - nominal_dimension) / (3.0 * total_std_dev)).min((nominal_dimension - lsl) / (3.0 * total_std_dev));
        let sigma_level = cpk * 3.0;
        
        AnalysisResults {
            nominal_dimension,
            predicted_tolerance: predicted_tolerance.clone(),
            three_sigma_tolerance: Some(predicted_tolerance), // RSS always uses 3-sigma
            user_specified_tolerance: None, // RSS doesn't use confidence intervals
            cp,
            cpk,
            sigma_level,
            yield_percentage: 99.9937, // 4-sigma
            distribution_data_file: None,
            quartile_data: None,
        }
    }
    
    fn monte_carlo_analysis(&self, features: &[&Feature]) -> Result<AnalysisResults> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(self.config.simulations);
        
        for _ in 0..self.config.simulations {
            let mut total_dimension = 0.0;
            
            for feature in features {
                let sample = self.sample_feature_distribution(feature, &mut rng)?;
                total_dimension += sample;
            }
            
            samples.push(total_dimension);
        }
        
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let nominal_dimension = samples.iter().sum::<f64>() / samples.len() as f64;
        let std_dev = self.calculate_std_dev(&samples, nominal_dimension);
        
        // Calculate quartile data
        let quartile_data = self.calculate_quartile_data(&samples);
        
        // Calculate 3-sigma tolerance (engineering standard) - 99.73% of data
        // For normal distribution: 99.73% means 0.135% in each tail
        let three_sigma_lower_idx = ((0.00135 * samples.len() as f64) as usize).max(0);
        let three_sigma_upper_idx = ((0.99865 * samples.len() as f64) as usize).min(samples.len() - 1);
        
        let three_sigma_tolerance = Tolerance {
            plus: samples[three_sigma_upper_idx] - nominal_dimension,
            minus: nominal_dimension - samples[three_sigma_lower_idx],
            distribution: ToleranceDistribution::Normal,
        };
        
        // Calculate user-specified confidence level tolerance
        // For 95% CI: alpha = 0.05, so 2.5% in each tail
        let alpha = 1.0 - self.config.confidence_level;
        let lower_tail = alpha / 2.0;
        let upper_tail = 1.0 - (alpha / 2.0);
        
        let user_lower_idx = ((lower_tail * samples.len() as f64) as usize).max(0);
        let user_upper_idx = ((upper_tail * samples.len() as f64) as usize).min(samples.len() - 1);
        
        let user_specified_tolerance = Tolerance {
            plus: samples[user_upper_idx] - nominal_dimension,
            minus: nominal_dimension - samples[user_lower_idx],
            distribution: ToleranceDistribution::Normal,
        };
        
        // Determine which tolerance to use as primary based on config
        let predicted_tolerance = if self.config.use_three_sigma {
            three_sigma_tolerance.clone()
        } else {
            user_specified_tolerance.clone()
        };
        
        // Calculate process capability metrics based on primary tolerance
        let usl = nominal_dimension + predicted_tolerance.plus;
        let lsl = nominal_dimension - predicted_tolerance.minus;
        let cp = (usl - lsl) / (6.0 * std_dev);
        let cpk = ((usl - nominal_dimension) / (3.0 * std_dev)).min((nominal_dimension - lsl) / (3.0 * std_dev));
        let sigma_level = cpk * 3.0;
        let yield_percentage = self.calculate_yield(&samples, lsl, usl);
        
        Ok(AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            three_sigma_tolerance: Some(three_sigma_tolerance),
            user_specified_tolerance: Some(user_specified_tolerance),
            cp,
            cpk,
            sigma_level,
            yield_percentage,
            distribution_data_file: None, // Will be set by wrapper if needed
            quartile_data: Some(quartile_data),
        })
    }
    
    fn sample_feature_distribution(&self, feature: &Feature, rng: &mut impl Rng) -> Result<f64> {
        let low = feature.nominal - feature.tolerance.minus;
        let high = feature.nominal + feature.tolerance.plus;
        
        let sample = match &feature.tolerance.distribution {
            ToleranceDistribution::Normal => {
                // INDUSTRY BEST PRACTICE: For normal distributions, tolerance limits typically represent:
                // - Design tolerances: often ±3σ (99.73% of parts within spec)
                // - Process capability: Cp = (USL - LSL) / 6σ
                // 
                // However, tolerance interpretation varies by industry:
                // - Automotive: Often ±3σ for critical dimensions
                // - Aerospace: May use ±4σ or ±6σ for safety-critical parts
                // - Medical devices: Typically ±3σ to ±6σ depending on risk
                //
                // For manufacturing tolerance analysis, we assume the tolerance represents
                // the process capability window. A common conservative approach is:
                // σ = (tolerance_plus + tolerance_minus) / 6 (assuming ±3σ limits)
                //
                // This can be made configurable in future versions for different industries
                let total_tolerance_band = feature.tolerance.plus + feature.tolerance.minus;
                let std_dev = total_tolerance_band / 6.0; // Conservative 3-sigma assumption
                
                let normal = Normal::new(feature.nominal, std_dev).map_err(|e| {
                    tessera_core::DesignTrackError::Module(format!("Failed to create normal distribution: {}", e))
                })?;
                normal.sample(rng)
            },
            ToleranceDistribution::Uniform => {
                let uniform = Uniform::new(low, high);
                uniform.sample(rng)
            },
            ToleranceDistribution::Triangular => {
                let triangular = Triangular::new(low, high, feature.nominal).map_err(|e| {
                    tessera_core::DesignTrackError::Module(format!("Failed to create triangular distribution: {}", e))
                })?;
                triangular.sample(rng)
            },
            ToleranceDistribution::LogNormal => {
                if low <= 0.0 {
                    return Err(tessera_core::DesignTrackError::Validation(
                        "LogNormal distribution requires positive values".to_string()
                    ));
                }
                let log_mean = feature.nominal.ln();
                let log_std = (feature.tolerance.plus + feature.tolerance.minus) / (6.0 * feature.nominal);
                let log_normal = LogNormal::new(log_mean, log_std).map_err(|e| {
                    tessera_core::DesignTrackError::Module(format!("Failed to create log-normal distribution: {}", e))
                })?;
                log_normal.sample(rng)
            },
            ToleranceDistribution::Beta { alpha, beta } => {
                let beta_dist = Beta::new(*alpha, *beta).map_err(|e| {
                    tessera_core::DesignTrackError::Module(format!("Failed to create beta distribution: {}", e))
                })?;
                let beta_sample = beta_dist.sample(rng);
                // Scale from [0,1] to [low, high]
                low + beta_sample * (high - low)
            },
        };
        
        Ok(sample)
    }
    
    fn worst_case_analysis_with_contributions(&self, features: &[&Feature], contributions: &[FeatureContribution]) -> AnalysisResults {
        let mut nominal_dimension = 0.0;
        let mut plus_tolerance = 0.0;
        let mut minus_tolerance = 0.0;
        
        for (feature, contribution) in features.iter().zip(contributions.iter()) {
            let multiplier = if contribution.half_count { 0.5 } else { 1.0 };
            let direction_multiplier = contribution.direction * multiplier;
            
            nominal_dimension += feature.nominal * direction_multiplier;
            plus_tolerance += feature.tolerance.plus * direction_multiplier.abs();
            minus_tolerance += feature.tolerance.minus * direction_multiplier.abs();
        }
        
        let predicted_tolerance = Tolerance {
            plus: plus_tolerance,
            minus: minus_tolerance,
            distribution: ToleranceDistribution::Uniform,
        };
        
        AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            three_sigma_tolerance: None,
            user_specified_tolerance: None,
            cp: 1.0,
            cpk: 0.8,
            sigma_level: 3.0,
            yield_percentage: 99.73,
            distribution_data_file: None,
            quartile_data: None,
        }
    }
    
    fn root_sum_square_analysis_with_contributions(&self, features: &[&Feature], contributions: &[FeatureContribution]) -> AnalysisResults {
        let mut nominal_dimension = 0.0;
        let mut variance = 0.0;
        
        for (feature, contribution) in features.iter().zip(contributions.iter()) {
            let multiplier = if contribution.half_count { 0.5 } else { 1.0 };
            let direction_multiplier = contribution.direction * multiplier;
            
            nominal_dimension += feature.nominal * direction_multiplier;
            
            // For RSS with normal distributions, we need to calculate variance correctly
            // Standard deviation for a tolerance band is (plus + minus) / 6 for 3-sigma
            let std_dev = (feature.tolerance.plus + feature.tolerance.minus) / 6.0;
            // Variance contribution includes the direction multiplier squared
            variance += (std_dev * direction_multiplier.abs()).powi(2);
        }
        
        // Total standard deviation
        let total_std_dev = variance.sqrt();
        
        // For symmetric normal distribution, 3-sigma limits
        let predicted_tolerance = Tolerance {
            plus: 3.0 * total_std_dev,
            minus: 3.0 * total_std_dev,
            distribution: ToleranceDistribution::Normal,
        };
        
        // Calculate process capability metrics
        let usl = nominal_dimension + predicted_tolerance.plus;
        let lsl = nominal_dimension - predicted_tolerance.minus;
        let cp = (usl - lsl) / (6.0 * total_std_dev);
        let cpk = ((usl - nominal_dimension) / (3.0 * total_std_dev)).min((nominal_dimension - lsl) / (3.0 * total_std_dev));
        let sigma_level = cpk * 3.0;
        
        AnalysisResults {
            nominal_dimension,
            predicted_tolerance: predicted_tolerance.clone(),
            three_sigma_tolerance: Some(predicted_tolerance), // RSS always uses 3-sigma
            user_specified_tolerance: None, // RSS doesn't use confidence intervals
            cp,
            cpk,
            sigma_level,
            yield_percentage: 99.9937, // 4-sigma for RSS
            distribution_data_file: None,
            quartile_data: None,
        }
    }
    
    fn monte_carlo_analysis_with_contributions(&self, features: &[&Feature], contributions: &[FeatureContribution]) -> Result<AnalysisResults> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(self.config.simulations);
        
        for _ in 0..self.config.simulations {
            let mut total_dimension = 0.0;
            
            for (feature, contribution) in features.iter().zip(contributions.iter()) {
                let multiplier = if contribution.half_count { 0.5 } else { 1.0 };
                let direction_multiplier = contribution.direction * multiplier;
                
                let sample = self.sample_feature_distribution(feature, &mut rng)?;
                total_dimension += sample * direction_multiplier;
            }
            
            samples.push(total_dimension);
        }
        
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let nominal_dimension = samples.iter().sum::<f64>() / samples.len() as f64;
        let std_dev = self.calculate_std_dev(&samples, nominal_dimension);
        
        // Calculate quartile data for visualization
        let quartile_data = self.calculate_quartile_data(&samples);
        
        // Calculate 3-sigma tolerance (engineering standard) - 99.73% of data
        let three_sigma_lower_idx = ((0.00135 * samples.len() as f64) as usize).max(0);
        let three_sigma_upper_idx = ((0.99865 * samples.len() as f64) as usize).min(samples.len() - 1);
        
        let three_sigma_tolerance = Tolerance {
            plus: samples[three_sigma_upper_idx] - nominal_dimension,
            minus: nominal_dimension - samples[three_sigma_lower_idx],
            distribution: ToleranceDistribution::Normal,
        };
        
        // Calculate user-specified confidence level tolerance
        let alpha = 1.0 - self.config.confidence_level;
        let lower_tail = alpha / 2.0;
        let upper_tail = 1.0 - (alpha / 2.0);
        
        let user_lower_idx = ((lower_tail * samples.len() as f64) as usize).max(0);
        let user_upper_idx = ((upper_tail * samples.len() as f64) as usize).min(samples.len() - 1);
        
        let user_specified_tolerance = Tolerance {
            plus: samples[user_upper_idx] - nominal_dimension,
            minus: nominal_dimension - samples[user_lower_idx],
            distribution: ToleranceDistribution::Normal,
        };
        
        // Use 3-sigma tolerance as primary for consistency with RSS method
        let predicted_tolerance = three_sigma_tolerance.clone();
        
        let usl = nominal_dimension + predicted_tolerance.plus;
        let lsl = nominal_dimension - predicted_tolerance.minus;
        let cp = (usl - lsl) / (6.0 * std_dev);
        let cpk = ((usl - nominal_dimension) / (3.0 * std_dev)).min((nominal_dimension - lsl) / (3.0 * std_dev));
        let sigma_level = cpk * 3.0;
        let yield_percentage = self.calculate_yield(&samples, lsl, usl);
        
        Ok(AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            three_sigma_tolerance: Some(three_sigma_tolerance),
            user_specified_tolerance: Some(user_specified_tolerance),
            cp,
            cpk,
            sigma_level,
            yield_percentage,
            distribution_data_file: None, // Will be set by wrapper if needed
            quartile_data: Some(quartile_data),
        })
    }
    
    fn calculate_std_dev(&self, samples: &[f64], mean: f64) -> f64 {
        let variance = samples.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (samples.len() - 1) as f64;
        variance.sqrt()
    }
    
    fn calculate_yield(&self, samples: &[f64], lsl: f64, usl: f64) -> f64 {
        let in_spec_count = samples.iter()
            .filter(|&&x| x >= lsl && x <= usl)
            .count();
        (in_spec_count as f64 / samples.len() as f64) * 100.0
    }
    
    fn save_monte_carlo_simulation_data(&self, features: &[&Feature], contributions: &[FeatureContribution], stackup_id: &Id) -> Result<String> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(self.config.simulations);
        
        // Generate simulation samples using the same logic as the analysis
        for _ in 0..self.config.simulations {
            let mut total_dimension = 0.0;
            
            for (feature, contribution) in features.iter().zip(contributions.iter()) {
                let multiplier = if contribution.half_count { 0.5 } else { 1.0 };
                let direction_multiplier = contribution.direction * multiplier;
                
                let sample = self.sample_feature_distribution(feature, &mut rng)?;
                total_dimension += sample * direction_multiplier;
            }
            
            samples.push(total_dimension);
        }
        
        // Generate timestamp for filename
        let analysis_timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        
        // Save to CSV using existing method
        self.save_simulation_data_to_csv(&samples, stackup_id, &analysis_timestamp)
    }

    fn save_simulation_data_to_csv(&self, samples: &[f64], stackup_id: &Id, analysis_timestamp: &str) -> Result<String> {
        // Create nested directory structure: data/simulations/
        let data_dir = std::path::Path::new("data");
        let simulations_dir = data_dir.join("simulations");
        
        // Create directories if they don't exist
        fs::create_dir_all(&simulations_dir).map_err(|e| {
            tessera_core::DesignTrackError::Module(format!("Failed to create simulations directory: {}", e))
        })?;
        
        // Generate CSV filename with stackup ID and timestamp
        let filename = format!("stackup_{}_{}.csv", stackup_id, analysis_timestamp);
        let csv_path = simulations_dir.join(&filename);
        
        // Write CSV file
        let mut file = fs::File::create(&csv_path).map_err(|e| {
            tessera_core::DesignTrackError::Module(format!("Failed to create CSV file: {}", e))
        })?;
        
        // Write CSV header
        writeln!(file, "sample_number,dimension_value").map_err(|e| {
            tessera_core::DesignTrackError::Module(format!("Failed to write CSV header: {}", e))
        })?;
        
        // Write data
        for (i, &value) in samples.iter().enumerate() {
            writeln!(file, "{},{:.12}", i + 1, value).map_err(|e| {
                tessera_core::DesignTrackError::Module(format!("Failed to write CSV data: {}", e))
            })?;
        }
        
        // Return relative path for storage in metadata
        Ok(format!("data/simulations/{}", filename))
    }
    
    fn calculate_quartile_data(&self, sorted_samples: &[f64]) -> QuartileData {
        let n = sorted_samples.len();
        
        // Calculate percentile indices
        let min_idx = 0;
        let max_idx = n - 1;
        let p5_idx = ((n as f64) * 0.05) as usize;
        let q1_idx = ((n as f64) * 0.25) as usize;
        let median_idx = ((n as f64) * 0.50) as usize;
        let q3_idx = ((n as f64) * 0.75) as usize;
        let p95_idx = ((n as f64) * 0.95) as usize;
        
        // Get values at calculated indices
        let minimum = sorted_samples[min_idx];
        let maximum = sorted_samples[max_idx];
        let p5 = sorted_samples[p5_idx];
        let q1 = sorted_samples[q1_idx];
        let median = sorted_samples[median_idx];
        let q3 = sorted_samples[q3_idx];
        let p95 = sorted_samples[p95_idx];
        
        // Calculate interquartile range
        let iqr = q3 - q1;
        
        QuartileData {
            minimum,
            q1,
            median,
            q3,
            maximum,
            iqr,
            p5,
            p95,
        }
    }
}

impl Default for ToleranceAnalyzer {
    fn default() -> Self {
        Self::new(AnalysisConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tessera_core::Id;
    
    #[test]
    fn test_worst_case_analysis() {
        let analyzer = ToleranceAnalyzer::default();
        let features = vec![
            Feature::new("Length1".to_string(), "First length".to_string(), Id::new(), FeatureType::Length, FeatureCategory::External, 10.0),
            Feature::new("Length2".to_string(), "Second length".to_string(), Id::new(), FeatureType::Length, FeatureCategory::External, 20.0),
        ];
        let feature_refs: Vec<&Feature> = features.iter().collect();
        
        let results = analyzer.worst_case_analysis(&feature_refs);
        
        assert_eq!(results.nominal_dimension, 30.0);
        assert_eq!(results.predicted_tolerance.plus, 0.2);
        assert_eq!(results.predicted_tolerance.minus, 0.2);
    }
}