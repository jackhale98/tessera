use tessera_core::{Result, DesignTrackError};
use serde::{Deserialize, Serialize};
use rand::{Rng, SeedableRng};
use rand::distributions::Distribution;
use rand_distr::{Normal, Uniform, Triangular, LogNormal};
use rand::rngs::StdRng;
use std::f64::consts::PI;

/// Enhanced distribution types from atlas implementation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DistributionType {
    Normal,
    Uniform,
    Triangular,
    LogNormal,
}

/// Distribution parameters for statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionParameters {
    pub distribution_type: DistributionType,
    pub mean: f64,
    pub std_dev: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub mode: Option<f64>, // For triangular distribution
}

/// Enhanced feature with distribution support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToleranceFeature {
    pub id: String,
    pub name: String,
    pub nominal: f64,
    pub plus_tolerance: f64,
    pub minus_tolerance: f64,
    pub distribution: DistributionType,
    pub direction: f64, // ±1.0 or custom multiplier
    pub half_count: bool, // For partial contributions
}

/// Statistical distribution engine
pub struct DistributionEngine {
    rng: StdRng,
}

impl DistributionEngine {
    /// Create new distribution engine with optional seed
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };
        
        Self { rng }
    }

    /// Calculate distribution parameters from tolerance
    pub fn calculate_parameters(&self, feature: &ToleranceFeature) -> Result<DistributionParameters> {
        let mean = feature.nominal;
        let upper_limit = mean + feature.plus_tolerance;
        let lower_limit = mean - feature.minus_tolerance;
        let total_tolerance = feature.plus_tolerance + feature.minus_tolerance;
        
        let (std_dev, mode) = match feature.distribution {
            DistributionType::Normal => {
                // 6-sigma approach (99.73% coverage)
                let std_dev = total_tolerance / 6.0;
                (std_dev, None)
            }
            DistributionType::Uniform => {
                // For uniform distribution, std_dev = range / sqrt(12)
                let std_dev = total_tolerance / (12.0_f64.sqrt());
                (std_dev, None)
            }
            DistributionType::Triangular => {
                // For triangular distribution, std_dev = range / sqrt(24)
                let std_dev = total_tolerance / (24.0_f64.sqrt());
                // Mode defaults to nominal unless otherwise specified
                (std_dev, Some(mean))
            }
            DistributionType::LogNormal => {
                // For log-normal, calculate parameters in log space
                if lower_limit <= 0.0 {
                    return Err(DesignTrackError::Validation(
                        "LogNormal distribution requires positive values".to_string()
                    ));
                }
                // Approximate std_dev for log-normal
                let std_dev = total_tolerance / 6.0;
                (std_dev, None)
            }
        };

        Ok(DistributionParameters {
            distribution_type: feature.distribution,
            mean,
            std_dev,
            min_value: lower_limit,
            max_value: upper_limit,
            mode,
        })
    }

    /// Sample value from feature distribution
    pub fn sample_feature_value(&mut self, feature: &ToleranceFeature) -> Result<f64> {
        let params = self.calculate_parameters(feature)?;
        
        match feature.distribution {
            DistributionType::Normal => {
                let normal = Normal::new(params.mean, params.std_dev)
                    .map_err(|e| DesignTrackError::Module(format!("Normal distribution error: {}", e)))?;
                Ok(normal.sample(&mut self.rng))
            }
            DistributionType::Uniform => {
                let uniform = Uniform::new(params.min_value, params.max_value);
                Ok(uniform.sample(&mut self.rng))
            }
            DistributionType::Triangular => {
                let mode = params.mode.unwrap_or(params.mean);
                let triangular = Triangular::new(params.min_value, params.max_value, mode)
                    .map_err(|e| DesignTrackError::Module(format!("Triangular distribution error: {}", e)))?;
                Ok(triangular.sample(&mut self.rng))
            }
            DistributionType::LogNormal => {
                // Convert to log space
                let log_mean = params.mean.ln();
                let log_std = (params.std_dev / params.mean).abs(); // Coefficient of variation approximation
                
                let log_normal = LogNormal::new(log_mean, log_std)
                    .map_err(|e| DesignTrackError::Module(format!("LogNormal distribution error: {}", e)))?;
                Ok(log_normal.sample(&mut self.rng))
            }
        }
    }

    /// Calculate theoretical variance for a distribution
    pub fn calculate_feature_variance(&self, feature: &ToleranceFeature) -> Result<f64> {
        let params = self.calculate_parameters(feature)?;
        let range = params.max_value - params.min_value;
        
        let variance = match feature.distribution {
            DistributionType::Normal => params.std_dev.powi(2),
            DistributionType::Uniform => range.powi(2) / 12.0,
            DistributionType::Triangular => range.powi(2) / 24.0,
            DistributionType::LogNormal => {
                // For log-normal: Var = (e^(σ²) - 1) * e^(2μ + σ²)
                let log_mean = params.mean.ln();
                let log_var = (params.std_dev / params.mean).powi(2);
                (log_var.exp() - 1.0) * (2.0 * log_mean + log_var).exp()
            }
        };
        
        Ok(variance)
    }

    /// Calculate probability density function value
    pub fn calculate_pdf(&self, feature: &ToleranceFeature, x: f64) -> Result<f64> {
        let params = self.calculate_parameters(feature)?;
        
        let pdf = match feature.distribution {
            DistributionType::Normal => {
                Self::normal_pdf(x, params.mean, params.std_dev)
            }
            DistributionType::Uniform => {
                if x >= params.min_value && x <= params.max_value {
                    1.0 / (params.max_value - params.min_value)
                } else {
                    0.0
                }
            }
            DistributionType::Triangular => {
                let a = params.min_value;
                let b = params.max_value;
                let c = params.mode.unwrap_or(params.mean);
                
                if x < a || x > b {
                    0.0
                } else if x <= c {
                    2.0 * (x - a) / ((b - a) * (c - a))
                } else {
                    2.0 * (b - x) / ((b - a) * (b - c))
                }
            }
            DistributionType::LogNormal => {
                if x <= 0.0 {
                    0.0
                } else {
                    let log_mean = params.mean.ln();
                    let log_std = params.std_dev / params.mean;
                    let coefficient = 1.0 / (x * log_std * (2.0 * PI).sqrt());
                    let exponent = -0.5 * ((x.ln() - log_mean) / log_std).powi(2);
                    coefficient * exponent.exp()
                }
            }
        };
        
        Ok(pdf)
    }

    /// Normal probability density function
    fn normal_pdf(x: f64, mean: f64, std_dev: f64) -> f64 {
        let coefficient = 1.0 / (std_dev * (2.0 * PI).sqrt());
        let exponent = -0.5 * ((x - mean) / std_dev).powi(2);
        coefficient * exponent.exp()
    }

    /// Generate samples for histogram analysis
    pub fn generate_samples(&mut self, feature: &ToleranceFeature, count: usize) -> Result<Vec<f64>> {
        let mut samples = Vec::with_capacity(count);
        
        for _ in 0..count {
            let sample = self.sample_feature_value(feature)?;
            samples.push(sample);
        }
        
        Ok(samples)
    }

    /// Calculate percentiles from samples
    pub fn calculate_percentiles(&self, mut samples: Vec<f64>) -> PercentileData {
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = samples.len();
        
        if len == 0 {
            return PercentileData::default();
        }
        
        let get_percentile = |p: f64| -> f64 {
            let index = (p / 100.0 * (len - 1) as f64).round() as usize;
            samples[index.min(len - 1)]
        };
        
        PercentileData {
            p0_1: get_percentile(0.1),
            p1: get_percentile(1.0),
            p5: get_percentile(5.0),
            p10: get_percentile(10.0),
            p25: get_percentile(25.0),
            p50: get_percentile(50.0),
            p75: get_percentile(75.0),
            p90: get_percentile(90.0),
            p95: get_percentile(95.0),
            p99: get_percentile(99.0),
            p99_9: get_percentile(99.9),
        }
    }

    /// Calculate basic statistics from samples
    pub fn calculate_statistics(&self, samples: &[f64]) -> BasicStatistics {
        if samples.is_empty() {
            return BasicStatistics::default();
        }
        
        let n = samples.len() as f64;
        let mean = samples.iter().sum::<f64>() / n;
        
        let variance = samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1.0);
            
        let std_dev = variance.sqrt();
        
        let min = samples.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = samples.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        BasicStatistics {
            mean,
            std_dev,
            variance,
            min,
            max,
            count: samples.len(),
        }
    }
}

/// Percentile data structure from atlas
#[derive(Debug, Clone, Default)]
pub struct PercentileData {
    pub p0_1: f64,
    pub p1: f64,
    pub p5: f64,
    pub p10: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub p99_9: f64,
}

/// Basic statistics data structure
#[derive(Debug, Clone, Default)]
pub struct BasicStatistics {
    pub mean: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl Default for DistributionType {
    fn default() -> Self {
        DistributionType::Normal
    }
}

impl std::fmt::Display for DistributionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionType::Normal => write!(f, "Normal"),
            DistributionType::Uniform => write!(f, "Uniform"),
            DistributionType::Triangular => write!(f, "Triangular"),
            DistributionType::LogNormal => write!(f, "LogNormal"),
        }
    }
}

impl ToleranceFeature {
    pub fn new(name: String, nominal: f64, plus_tol: f64, minus_tol: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            nominal,
            plus_tolerance: plus_tol,
            minus_tolerance: minus_tol,
            distribution: DistributionType::Normal,
            direction: 1.0,
            half_count: false,
        }
    }

    /// Get total tolerance range
    pub fn total_tolerance(&self) -> f64 {
        self.plus_tolerance + self.minus_tolerance
    }

    /// Get upper limit
    pub fn upper_limit(&self) -> f64 {
        self.nominal + self.plus_tolerance
    }

    /// Get lower limit  
    pub fn lower_limit(&self) -> f64 {
        self.nominal - self.minus_tolerance
    }
}