use crate::data::*;
use tessera_core::Result;
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform, Triangular, LogNormal};
use statrs::distribution::Beta;

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
        
        let results = match self.config.method {
            AnalysisMethod::WorstCase => self.worst_case_analysis_with_contributions(&stackup_features, contributions),
            AnalysisMethod::RootSumSquare => self.root_sum_square_analysis_with_contributions(&stackup_features, contributions),
            AnalysisMethod::MonteCarlo => self.monte_carlo_analysis_with_contributions(&stackup_features, contributions)?,
        };
        
        Ok(StackupAnalysis {
            stackup_id: stackup.id,
            stackup_name: stackup.name.clone(),
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
            cp,
            cpk,
            sigma_level,
            yield_percentage,
            distribution_data: None,
        }
    }
    
    fn root_sum_square_analysis(&self, features: &[&Feature]) -> AnalysisResults {
        let nominal_dimension: f64 = features.iter().map(|f| f.nominal).sum();
        
        // RSS calculation for normal distributions
        let plus_rss: f64 = features.iter()
            .map(|f| f.tolerance.plus.powi(2))
            .sum::<f64>()
            .sqrt();
        
        let minus_rss: f64 = features.iter()
            .map(|f| f.tolerance.minus.powi(2))
            .sum::<f64>()
            .sqrt();
        
        let predicted_tolerance = Tolerance {
            plus: plus_rss,
            minus: minus_rss,
            distribution: ToleranceDistribution::Normal,
        };
        
        // Better estimates for RSS
        let cp = 1.33;
        let cpk = 1.2;
        let sigma_level = 4.0;
        let yield_percentage = 99.9937; // 4-sigma
        
        AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            cp,
            cpk,
            sigma_level,
            yield_percentage,
            distribution_data: None,
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
        
        // Calculate tolerance based on confidence level
        let alpha = (1.0 - self.config.confidence_level) / 2.0;
        let lower_idx = (alpha * samples.len() as f64) as usize;
        let upper_idx = ((1.0 - alpha) * samples.len() as f64) as usize;
        
        let predicted_tolerance = Tolerance {
            plus: samples[upper_idx] - nominal_dimension,
            minus: nominal_dimension - samples[lower_idx],
            distribution: ToleranceDistribution::Normal,
        };
        
        // Calculate process capability metrics
        let usl = nominal_dimension + predicted_tolerance.plus;
        let lsl = nominal_dimension - predicted_tolerance.minus;
        let cp = (usl - lsl) / (6.0 * std_dev);
        let cpk = ((usl - nominal_dimension) / (3.0 * std_dev)).min((nominal_dimension - lsl) / (3.0 * std_dev));
        let sigma_level = cpk * 3.0;
        let yield_percentage = self.calculate_yield(&samples, lsl, usl);
        
        Ok(AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            cp,
            cpk,
            sigma_level,
            yield_percentage,
            distribution_data: Some(samples),
        })
    }
    
    fn sample_feature_distribution(&self, feature: &Feature, rng: &mut impl Rng) -> Result<f64> {
        let low = feature.nominal - feature.tolerance.minus;
        let high = feature.nominal + feature.tolerance.plus;
        
        let sample = match &feature.tolerance.distribution {
            ToleranceDistribution::Normal => {
                let std_dev = (feature.tolerance.plus + feature.tolerance.minus) / 6.0; // 3-sigma
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
            cp: 1.0,
            cpk: 0.8,
            sigma_level: 3.0,
            yield_percentage: 99.73,
            distribution_data: None,
        }
    }
    
    fn root_sum_square_analysis_with_contributions(&self, features: &[&Feature], contributions: &[FeatureContribution]) -> AnalysisResults {
        let mut nominal_dimension = 0.0;
        let mut plus_variance = 0.0;
        let mut minus_variance = 0.0;
        
        for (feature, contribution) in features.iter().zip(contributions.iter()) {
            let multiplier = if contribution.half_count { 0.5 } else { 1.0 };
            let direction_multiplier = contribution.direction * multiplier;
            
            nominal_dimension += feature.nominal * direction_multiplier;
            plus_variance += (feature.tolerance.plus * direction_multiplier).powi(2);
            minus_variance += (feature.tolerance.minus * direction_multiplier).powi(2);
        }
        
        let predicted_tolerance = Tolerance {
            plus: plus_variance.sqrt(),
            minus: minus_variance.sqrt(),
            distribution: ToleranceDistribution::Normal,
        };
        
        AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            cp: 1.33,
            cpk: 1.2,
            sigma_level: 4.0,
            yield_percentage: 99.9937,
            distribution_data: None,
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
        
        let alpha = (1.0 - self.config.confidence_level) / 2.0;
        let lower_idx = (alpha * samples.len() as f64) as usize;
        let upper_idx = ((1.0 - alpha) * samples.len() as f64) as usize;
        
        let predicted_tolerance = Tolerance {
            plus: samples[upper_idx] - nominal_dimension,
            minus: nominal_dimension - samples[lower_idx],
            distribution: ToleranceDistribution::Normal,
        };
        
        let usl = nominal_dimension + predicted_tolerance.plus;
        let lsl = nominal_dimension - predicted_tolerance.minus;
        let cp = (usl - lsl) / (6.0 * std_dev);
        let cpk = ((usl - nominal_dimension) / (3.0 * std_dev)).min((nominal_dimension - lsl) / (3.0 * std_dev));
        let sigma_level = cpk * 3.0;
        let yield_percentage = self.calculate_yield(&samples, lsl, usl);
        
        Ok(AnalysisResults {
            nominal_dimension,
            predicted_tolerance,
            cp,
            cpk,
            sigma_level,
            yield_percentage,
            distribution_data: Some(samples),
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
}

impl Default for ToleranceAnalyzer {
    fn default() -> Self {
        Self::new(AnalysisConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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