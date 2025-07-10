use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub part_number: Option<String>,
    pub features: Vec<Id>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

impl Entity for Component {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Component name cannot be empty".to_string()
            ));
        }
        Ok(())
    }
}

impl Component {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            part_number: None,
            features: Vec::new(),
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub component_id: Id,
    pub feature_type: FeatureType,
    pub feature_category: FeatureCategory,
    pub nominal: f64,
    pub tolerance: Tolerance,
    pub distribution: Option<ToleranceDistribution>,
    pub distribution_params: Option<DistributionParams>,
    pub drawing_location: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FeatureType {
    Length,
    Diameter,
    Radius,
    Angle,
    Position,
    Surface,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FeatureCategory {
    External, // External features (shafts, pins, etc.)
    Internal, // Internal features (holes, slots, etc.)
}

impl std::fmt::Display for FeatureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureCategory::External => write!(f, "External"),
            FeatureCategory::Internal => write!(f, "Internal"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tolerance {
    pub plus: f64,
    pub minus: f64,
    pub distribution: ToleranceDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToleranceDistribution {
    Normal,
    Uniform,
    Triangular,
    LogNormal,
    Beta { alpha: f64, beta: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionParams {
    pub calculated: bool,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mode: Option<f64>,
    pub shape: Option<f64>,
    pub scale: Option<f64>,
}

impl DistributionParams {
    pub fn calculate_from_feature(feature: &Feature) -> Self {
        let total_tolerance = feature.tolerance.plus + feature.tolerance.minus;
        let mean = feature.nominal;
        let std_dev = total_tolerance / 6.0;

        Self {
            calculated: true,
            mean: Some(mean),
            std_dev: Some(std_dev),
            min: Some(mean - 3.0 * std_dev),
            max: Some(mean + 3.0 * std_dev),
            mode: Some(mean),
            shape: Some(2.0),
            scale: Some(std_dev),
        }
    }

    pub fn get_required_params(&self, dist_type: &ToleranceDistribution) -> Vec<(&'static str, f64)> {
        match dist_type {
            ToleranceDistribution::Normal => vec![
                ("Mean", self.mean.unwrap_or(0.0)),
                ("Std Dev", self.std_dev.unwrap_or(0.0)),
            ],
            ToleranceDistribution::Uniform => vec![
                ("Min", self.min.unwrap_or(0.0)),
                ("Max", self.max.unwrap_or(0.0)),
            ],
            ToleranceDistribution::Triangular => vec![
                ("Min", self.min.unwrap_or(0.0)),
                ("Max", self.max.unwrap_or(0.0)),
                ("Mode", self.mode.unwrap_or(0.0)),
            ],
            ToleranceDistribution::LogNormal => vec![
                ("Mean", self.mean.unwrap_or(0.0)),
                ("Std Dev", self.std_dev.unwrap_or(0.0)),
            ],
            ToleranceDistribution::Beta { alpha, beta } => vec![
                ("Alpha", *alpha),
                ("Beta", *beta),
            ],
        }
    }
}

impl Entity for Feature {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Feature name cannot be empty".to_string()
            ));
        }
        if self.tolerance.plus < 0.0 || self.tolerance.minus < 0.0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Tolerance values cannot be negative".to_string()
            ));
        }
        Ok(())
    }
}

impl Feature {
    pub fn new(name: String, description: String, component_id: Id, feature_type: FeatureType, feature_category: FeatureCategory, nominal: f64) -> Self {
        let now = Utc::now();
        let mut feature = Self {
            id: Id::new(),
            name,
            description,
            component_id,
            feature_type,
            feature_category,
            nominal,
            tolerance: Tolerance {
                plus: 0.1,
                minus: 0.1,
                distribution: ToleranceDistribution::Normal,
            },
            distribution: Some(ToleranceDistribution::Normal),
            distribution_params: None,
            drawing_location: None,
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        };
        
        feature.distribution_params = Some(DistributionParams::calculate_from_feature(&feature));
        feature
    }

    pub fn update_distribution(&mut self, dist_type: ToleranceDistribution) {
        self.distribution = Some(dist_type);
        if self.distribution_params.as_ref().map_or(true, |p| p.calculated) {
            self.distribution_params = Some(DistributionParams::calculate_from_feature(self));
        }
        self.updated = Utc::now();
    }

    /// Calculate Maximum Material Condition (MMC) value
    pub fn mmc(&self) -> f64 {
        match self.feature_category {
            FeatureCategory::External => {
                // For external features, MMC is at the high limit (largest size)
                self.nominal + self.tolerance.plus
            },
            FeatureCategory::Internal => {
                // For internal features, MMC is at the low limit (smallest size)
                self.nominal - self.tolerance.minus
            },
        }
    }

    /// Calculate Least Material Condition (LMC) value
    pub fn lmc(&self) -> f64 {
        match self.feature_category {
            FeatureCategory::External => {
                // For external features, LMC is at the low limit (smallest size)
                self.nominal - self.tolerance.minus
            },
            FeatureCategory::Internal => {
                // For internal features, LMC is at the high limit (largest size)
                self.nominal + self.tolerance.plus
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mate {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub mate_type: MateType,
    pub primary_feature: Id,
    pub secondary_feature: Id,
    pub offset: f64,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MateType {
    Clearance,
    Transition,
    Interference,
}

impl Default for MateType {
    fn default() -> Self {
        MateType::Clearance
    }
}

impl std::fmt::Display for MateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MateType::Clearance => write!(f, "Clearance"),
            MateType::Transition => write!(f, "Transition"),
            MateType::Interference => write!(f, "Interference"),
        }
    }
}

impl Entity for Mate {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Mate name cannot be empty".to_string()
            ));
        }
        if self.primary_feature == self.secondary_feature {
            return Err(tessera_core::DesignTrackError::Validation(
                "Primary and secondary features cannot be the same".to_string()
            ));
        }
        Ok(())
    }
}

impl Mate {
    pub fn new(name: String, description: String, mate_type: MateType, primary_feature: Id, secondary_feature: Id) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            mate_type,
            primary_feature,
            secondary_feature,
            offset: 0.0,
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }

    pub fn calculate_nominal_fit(&self, primary_feature: &Feature, secondary_feature: &Feature) -> f64 {
        match (primary_feature.feature_type, secondary_feature.feature_type) {
            (FeatureType::Diameter, FeatureType::Diameter) => {
                match self.mate_type {
                    MateType::Clearance | MateType::Transition => {
                        secondary_feature.nominal - primary_feature.nominal
                    },
                    MateType::Interference => {
                        primary_feature.nominal - secondary_feature.nominal
                    },
                }
            },
            _ => self.offset,
        }
    }

    /// Calculate fit using MMC conditions (tightest fit)
    pub fn calculate_mmc_fit(&self, primary_feature: &Feature, secondary_feature: &Feature) -> f64 {
        match (primary_feature.feature_type, secondary_feature.feature_type) {
            (FeatureType::Diameter, FeatureType::Diameter) => {
                match self.mate_type {
                    MateType::Clearance | MateType::Transition => {
                        // For clearance/transition fits, MMC condition is:
                        // Internal feature at MMC (smallest) - External feature at MMC (largest)
                        secondary_feature.mmc() - primary_feature.mmc()
                    },
                    MateType::Interference => {
                        // For interference fits, MMC condition is:
                        // External feature at MMC (largest) - Internal feature at MMC (smallest)
                        primary_feature.mmc() - secondary_feature.mmc()
                    },
                }
            },
            _ => self.offset,
        }
    }

    /// Calculate fit using LMC conditions (loosest fit)
    pub fn calculate_lmc_fit(&self, primary_feature: &Feature, secondary_feature: &Feature) -> f64 {
        match (primary_feature.feature_type, secondary_feature.feature_type) {
            (FeatureType::Diameter, FeatureType::Diameter) => {
                match self.mate_type {
                    MateType::Clearance | MateType::Transition => {
                        // For clearance/transition fits, LMC condition is:
                        // Internal feature at LMC (largest) - External feature at LMC (smallest)
                        secondary_feature.lmc() - primary_feature.lmc()
                    },
                    MateType::Interference => {
                        // For interference fits, LMC condition is:
                        // External feature at LMC (smallest) - Internal feature at LMC (largest)
                        primary_feature.lmc() - secondary_feature.lmc()
                    },
                }
            },
            _ => self.offset,
        }
    }

    pub fn calculate_min_fit(&self, primary_feature: &Feature, secondary_feature: &Feature) -> f64 {
        // Minimum fit is the tightest condition (MMC)
        self.calculate_mmc_fit(primary_feature, secondary_feature)
    }

    pub fn calculate_max_fit(&self, primary_feature: &Feature, secondary_feature: &Feature) -> f64 {
        // Maximum fit is the loosest condition (LMC)
        self.calculate_lmc_fit(primary_feature, secondary_feature)
    }

    pub fn validate_fit(&self, primary_feature: &Feature, secondary_feature: &Feature) -> FitValidation {
        let nominal_fit = self.calculate_nominal_fit(primary_feature, secondary_feature);
        let min_fit = self.calculate_min_fit(primary_feature, secondary_feature);
        let max_fit = self.calculate_max_fit(primary_feature, secondary_feature);

        match self.mate_type {
            MateType::Clearance => {
                if min_fit <= 0.0 {
                    FitValidation {
                        is_valid: false,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: Some("Clearance fit must have positive minimum clearance".to_string()),
                    }
                } else {
                    FitValidation {
                        is_valid: true,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: None,
                    }
                }
            },
            MateType::Interference => {
                if max_fit >= 0.0 {
                    FitValidation {
                        is_valid: false,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: Some("Interference fit must have negative maximum clearance".to_string()),
                    }
                } else {
                    FitValidation {
                        is_valid: true,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: None,
                    }
                }
            },
            MateType::Transition => {
                if min_fit >= 0.0 || max_fit <= 0.0 {
                    FitValidation {
                        is_valid: false,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: Some("Transition fit must have both positive and negative clearances".to_string()),
                    }
                } else {
                    FitValidation {
                        is_valid: true,
                        nominal_fit,
                        min_fit,
                        max_fit,
                        error_message: None,
                    }
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FitValidation {
    pub is_valid: bool,
    pub nominal_fit: f64,
    pub min_fit: f64,
    pub max_fit: f64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stackup {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub dimension_chain: Vec<Id>, // Feature IDs in the chain
    pub target_dimension: f64,
    pub tolerance_target: Tolerance,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

impl Entity for Stackup {
    fn id(&self) -> Id {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Stackup name cannot be empty".to_string()
            ));
        }
        if self.dimension_chain.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Stackup must have at least one dimension in the chain".to_string()
            ));
        }
        Ok(())
    }
}

impl Stackup {
    pub fn new(name: String, description: String, target_dimension: f64) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            dimension_chain: Vec::new(),
            target_dimension,
            tolerance_target: Tolerance {
                plus: 0.1,
                minus: 0.1,
                distribution: ToleranceDistribution::Normal,
            },
            created: now,
            updated: now,
            metadata: IndexMap::new(),
        }
    }
    
    pub fn add_dimension(&mut self, feature_id: Id) {
        self.dimension_chain.push(feature_id);
        self.updated = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackupAnalysis {
    pub stackup_id: Id,
    pub stackup_name: String,
    pub config: AnalysisConfig,
    pub feature_contributions: Vec<FeatureContribution>,
    pub results: AnalysisResults,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureContribution {
    pub feature_id: Id,
    pub feature_name: String,
    pub direction: f64, // +1.0, -1.0, or custom multiplier
    pub half_count: bool, // For partial contributions
    pub contribution_type: ContributionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContributionType {
    Additive,   // Adds to the stackup
    Subtractive, // Subtracts from the stackup
    Custom(f64), // Custom multiplier
}

impl Default for ContributionType {
    fn default() -> Self {
        ContributionType::Additive
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub method: AnalysisMethod,
    pub simulations: usize,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisMethod {
    WorstCase,
    RootSumSquare,
    MonteCarlo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    pub nominal_dimension: f64,
    pub predicted_tolerance: Tolerance,
    pub cp: f64, // Process capability
    pub cpk: f64, // Process capability index
    pub sigma_level: f64,
    pub yield_percentage: f64,
    pub distribution_data: Option<Vec<f64>>, // For Monte Carlo
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            method: AnalysisMethod::MonteCarlo,
            simulations: 10000,
            confidence_level: 0.95,
        }
    }
}