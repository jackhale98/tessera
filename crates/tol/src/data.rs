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
    pub nominal: f64,
    pub tolerance: Tolerance,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    Length,
    Diameter,
    Radius,
    Angle,
    Position,
    Surface,
    Other(String),
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
    Beta { alpha: f64, beta: f64 },
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
    pub fn new(name: String, description: String, component_id: Id, feature_type: FeatureType, nominal: f64) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            component_id,
            feature_type,
            nominal,
            tolerance: Tolerance {
                plus: 0.1,
                minus: 0.1,
                distribution: ToleranceDistribution::Normal,
            },
            created: now,
            updated: now,
            metadata: IndexMap::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MateType {
    Clearance,
    Interference,
    Coincident,
    Parallel,
    Perpendicular,
    Concentric,
    Other(String),
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
    pub results: AnalysisResults,
    pub created: DateTime<Utc>,
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