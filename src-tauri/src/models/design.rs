use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;
use crate::models::EntityMetadata;

/// Assembly entity - represents a collection of components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assembly {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub revision: String,
    pub notes: Option<String>,
}

/// Component entity - represents a part in an assembly
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub revision: String,
    pub part_number: Option<String>,
    pub material: Option<String>,
    pub mass: Option<f64>,
    pub notes: Option<String>,
}

/// Feature type - External or Internal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureType {
    External,
    Internal,
}

/// Distribution type for tolerance analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistributionType {
    Normal,
    Uniform,
    Triangular,
}

/// Feature entity - represents a dimension with tolerances
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feature {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub feature_type: FeatureType,
    pub nominal: f64,
    pub upper_tolerance: f64,
    pub lower_tolerance: f64,
    pub distribution_type: DistributionType,
    pub custom_mean: Option<f64>,
    pub custom_std_dev: Option<f64>,
    pub drawing_location: Option<String>,
}

/// Mate type - clearance, transition, or interference fit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MateType {
    Clearance,
    Transition,
    InterferenceFit,
}

/// Mate analysis result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MateAnalysisResult {
    Pass,
    Fail,
}

/// Mate entity - represents a connection between two features
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mate {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub mate_type: MateType,

    // Calculated values
    pub mmc: Option<f64>,  // Maximum Material Condition
    pub lmc: Option<f64>,  // Least Material Condition
    pub analysis_result: Option<MateAnalysisResult>,
}

/// Analysis type for tolerance stackup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalysisType {
    WorstCase,
    RSS,
    MonteCarlo,
}

/// Stackup result for worst-case and RSS analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StackupResult {
    pub mean: f64,
    pub upper: f64,
    pub lower: f64,
}

/// Monte Carlo simulation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonteCarloResult {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub upper: f64,
    pub lower: f64,
    pub cp: Option<f64>,   // Process capability
    pub cpk: Option<f64>,  // Process capability index
    pub ppm_failures: Option<f64>,
}

/// Contribution sign for stackup features
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContributionSign {
    Positive,
    Negative,
}

/// Stackup feature contribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StackupFeatureContribution {
    pub feature_id: Uuid,
    pub sign: ContributionSign,
    pub contribution: f64,  // 0.0 to 1.0
}

/// Stackup entity - represents a tolerance stack analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stackup {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub analysis_types: Vec<AnalysisType>,
    pub upper_spec_limit: Option<f64>,
    pub lower_spec_limit: Option<f64>,
    pub feature_contributions: Vec<StackupFeatureContribution>,

    // Analysis results
    pub worst_case_result: Option<StackupResult>,
    pub rss_result: Option<StackupResult>,
    pub monte_carlo_result: Option<MonteCarloResult>,
}

/// Supplier entity - represents a component supplier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Supplier {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub contact_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

/// Cost distribution type for quote interpolation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CostDistribution {
    Linear,
    Power,
    Exponential,
    Logarithmic,
}

/// Quote entity - represents pricing from a supplier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub metadata: EntityMetadata,
    pub quote_number: String,
    pub quote_date: NaiveDate,
    pub expiration_date: Option<NaiveDate>,
    pub quantity_price_pairs: Vec<(u32, f64)>,
    pub distribution_type: CostDistribution,
    pub notes: Option<String>,
}

/// BOM item for generation results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BomItem {
    pub component_id: Uuid,
    pub part_number: Option<String>,
    pub description: String,
    pub revision: String,
    pub quantity: u32,
    pub cost_per_unit: f64,
    pub line_total: f64,
}

/// BOM generation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BomResult {
    pub assembly_id: Uuid,
    pub volume: u32,
    pub items: Vec<BomItem>,
    pub total_cost: f64,
    pub has_interpolated_costs: bool,
}

/// Cost estimate from interpolation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostEstimate {
    pub cost_per_unit: f64,
    pub is_interpolated: bool,
    pub r_squared: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityType;

    #[test]
    fn test_assembly_creation() {
        let assembly = Assembly {
            metadata: EntityMetadata::new(EntityType::Assembly),
            name: "Main Assembly".to_string(),
            description: "Primary product assembly".to_string(),
            revision: "A".to_string(),
            notes: Some("Initial design".to_string()),
        };

        assert_eq!(assembly.name, "Main Assembly");
        assert_eq!(assembly.revision, "A");
        assert_eq!(assembly.metadata.entity_type, EntityType::Assembly);
    }

    #[test]
    fn test_assembly_serialization() {
        let assembly = Assembly {
            metadata: EntityMetadata::new(EntityType::Assembly),
            name: "Test Assembly".to_string(),
            description: "Test".to_string(),
            revision: "1".to_string(),
            notes: None,
        };

        let serialized = ron::to_string(&assembly).expect("Failed to serialize");
        assert!(serialized.contains("Assembly"));
        assert!(serialized.contains("Test Assembly"));

        let deserialized: Assembly = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, assembly.name);
        assert_eq!(deserialized.metadata.id, assembly.metadata.id);
    }

    #[test]
    fn test_component_creation() {
        let component = Component {
            metadata: EntityMetadata::new(EntityType::Component),
            name: "Bracket".to_string(),
            description: "Mounting bracket".to_string(),
            revision: "B".to_string(),
            part_number: Some("BKT-001".to_string()),
            material: Some("Aluminum 6061".to_string()),
            mass: Some(0.125),
            notes: None,
        };

        assert_eq!(component.name, "Bracket");
        assert_eq!(component.part_number, Some("BKT-001".to_string()));
        assert_eq!(component.mass, Some(0.125));
    }

    #[test]
    fn test_component_serialization() {
        let component = Component {
            metadata: EntityMetadata::new(EntityType::Component),
            name: "Shaft".to_string(),
            description: "Drive shaft".to_string(),
            revision: "1".to_string(),
            part_number: Some("SFT-123".to_string()),
            material: None,
            mass: None,
            notes: None,
        };

        let serialized = ron::to_string(&component).expect("Failed to serialize");
        let deserialized: Component = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized.name, component.name);
        assert_eq!(deserialized.part_number, component.part_number);
    }

    #[test]
    fn test_feature_creation() {
        let feature = Feature {
            metadata: EntityMetadata::new(EntityType::Feature),
            name: "Hole Diameter".to_string(),
            description: "Main mounting hole".to_string(),
            notes: None,
            feature_type: FeatureType::Internal,
            nominal: 10.0,
            upper_tolerance: 0.1,
            lower_tolerance: -0.1,
            distribution_type: DistributionType::Normal,
            custom_mean: None,
            custom_std_dev: None,
            drawing_location: Some("Detail A".to_string()),
        };

        assert_eq!(feature.nominal, 10.0);
        assert_eq!(feature.upper_tolerance, 0.1);
        assert_eq!(feature.feature_type, FeatureType::Internal);
    }

    #[test]
    fn test_feature_serialization() {
        let feature = Feature {
            metadata: EntityMetadata::new(EntityType::Feature),
            name: "Width".to_string(),
            description: "Part width".to_string(),
            notes: None,
            feature_type: FeatureType::External,
            nominal: 50.0,
            upper_tolerance: 0.5,
            lower_tolerance: -0.5,
            distribution_type: DistributionType::Uniform,
            custom_mean: Some(50.0),
            custom_std_dev: Some(0.167),
            drawing_location: None,
        };

        let serialized = ron::to_string(&feature).expect("Failed to serialize");
        let deserialized: Feature = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized.nominal, feature.nominal);
        assert_eq!(deserialized.distribution_type, feature.distribution_type);
    }

    #[test]
    fn test_mate_creation() {
        let mate = Mate {
            metadata: EntityMetadata::new(EntityType::Mate),
            name: "Shaft-Bearing Fit".to_string(),
            description: "Interference fit between shaft and bearing".to_string(),
            notes: None,
            mate_type: MateType::InterferenceFit,
            mmc: Some(10.05),
            lmc: Some(9.95),
            analysis_result: Some(MateAnalysisResult::Pass),
        };

        assert_eq!(mate.mate_type, MateType::InterferenceFit);
        assert_eq!(mate.mmc, Some(10.05));
    }

    #[test]
    fn test_mate_serialization() {
        let mate = Mate {
            metadata: EntityMetadata::new(EntityType::Mate),
            name: "Clearance Fit".to_string(),
            description: "Loose fit".to_string(),
            notes: None,
            mate_type: MateType::Clearance,
            mmc: None,
            lmc: None,
            analysis_result: None,
        };

        let serialized = ron::to_string(&mate).expect("Failed to serialize");
        let deserialized: Mate = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized.mate_type, mate.mate_type);
    }

    #[test]
    fn test_stackup_creation() {
        let stackup = Stackup {
            metadata: EntityMetadata::new(EntityType::Stackup),
            name: "Gap Analysis".to_string(),
            description: "Critical gap dimension".to_string(),
            notes: None,
            analysis_types: vec![AnalysisType::WorstCase, AnalysisType::RSS],
            upper_spec_limit: Some(2.0),
            lower_spec_limit: Some(0.5),
            feature_contributions: vec![
                StackupFeatureContribution {
                    feature_id: Uuid::new_v4(),
                    sign: ContributionSign::Positive,
                    contribution: 1.0,
                },
            ],
            worst_case_result: None,
            rss_result: None,
            monte_carlo_result: None,
        };

        assert_eq!(stackup.analysis_types.len(), 2);
        assert_eq!(stackup.upper_spec_limit, Some(2.0));
    }

    #[test]
    fn test_stackup_serialization() {
        let stackup = Stackup {
            metadata: EntityMetadata::new(EntityType::Stackup),
            name: "Test Stackup".to_string(),
            description: "Test".to_string(),
            notes: None,
            analysis_types: vec![AnalysisType::MonteCarlo],
            upper_spec_limit: None,
            lower_spec_limit: None,
            feature_contributions: vec![],
            worst_case_result: None,
            rss_result: None,
            monte_carlo_result: None,
        };

        let serialized = ron::to_string(&stackup).expect("Failed to serialize");
        let deserialized: Stackup = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized.name, stackup.name);
        assert_eq!(deserialized.analysis_types.len(), 1);
    }

    #[test]
    fn test_supplier_creation() {
        let supplier = Supplier {
            metadata: EntityMetadata::new(EntityType::Supplier),
            name: "Acme Manufacturing".to_string(),
            description: "CNC machining services".to_string(),
            contact_name: Some("John Doe".to_string()),
            address: Some("123 Industrial Pkwy".to_string()),
            phone: Some("555-1234".to_string()),
            email: Some("john@acme.com".to_string()),
            notes: None,
        };

        assert_eq!(supplier.name, "Acme Manufacturing");
        assert_eq!(supplier.email, Some("john@acme.com".to_string()));
    }

    #[test]
    fn test_supplier_serialization() {
        let supplier = Supplier {
            metadata: EntityMetadata::new(EntityType::Supplier),
            name: "Test Supplier".to_string(),
            description: "Test".to_string(),
            contact_name: None,
            address: None,
            phone: None,
            email: None,
            notes: None,
        };

        let serialized = ron::to_string(&supplier).expect("Failed to serialize");
        let deserialized: Supplier = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized.name, supplier.name);
    }

    #[test]
    fn test_quote_creation() {
        let quote = Quote {
            metadata: EntityMetadata::new(EntityType::Quote),
            quote_number: "Q-2025-001".to_string(),
            quote_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            expiration_date: Some(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap()),
            quantity_price_pairs: vec![
                (100, 10.0),
                (500, 8.0),
                (1000, 6.0),
            ],
            distribution_type: CostDistribution::Power,
            notes: Some("Volume discounts available".to_string()),
        };

        assert_eq!(quote.quantity_price_pairs.len(), 3);
        assert_eq!(quote.quantity_price_pairs[0].1, 10.0);
    }

    #[test]
    fn test_quote_serialization() {
        let quote = Quote {
            metadata: EntityMetadata::new(EntityType::Quote),
            quote_number: "Q-001".to_string(),
            quote_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            expiration_date: None,
            quantity_price_pairs: vec![(10, 5.0)],
            distribution_type: CostDistribution::Linear,
            notes: None,
        };

        let serialized = ron::to_string(&quote).expect("Failed to serialize");
        let deserialized: Quote = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized.quote_number, quote.quote_number);
        assert_eq!(deserialized.quantity_price_pairs.len(), 1);
    }

    #[test]
    fn test_stackup_result() {
        let result = StackupResult {
            mean: 1.25,
            upper: 1.75,
            lower: 0.75,
        };

        assert_eq!(result.mean, 1.25);
        assert!(result.upper > result.mean);
        assert!(result.lower < result.mean);
    }

    #[test]
    fn test_monte_carlo_result() {
        let result = MonteCarloResult {
            mean: 10.0,
            median: 10.05,
            std_dev: 0.5,
            upper: 11.5,
            lower: 8.5,
            cp: Some(2.0),
            cpk: Some(1.8),
            ppm_failures: Some(10.0),
        };

        assert_eq!(result.cp, Some(2.0));
        assert!(result.cpk.unwrap() < result.cp.unwrap());
    }

    #[test]
    fn test_bom_item() {
        let item = BomItem {
            component_id: Uuid::new_v4(),
            part_number: Some("BKT-001".to_string()),
            description: "Mounting bracket".to_string(),
            revision: "A".to_string(),
            quantity: 4,
            cost_per_unit: 5.25,
            line_total: 21.00,
        };

        assert_eq!(item.quantity, 4);
        assert_eq!(item.line_total, 21.00);
    }

    #[test]
    fn test_distribution_type_variants() {
        let normal = DistributionType::Normal;
        let uniform = DistributionType::Uniform;
        let triangular = DistributionType::Triangular;

        assert_eq!(normal, DistributionType::Normal);
        assert_ne!(normal, uniform);
        assert_ne!(uniform, triangular);
    }

    #[test]
    fn test_analysis_type_variants() {
        let worst = AnalysisType::WorstCase;
        let rss = AnalysisType::RSS;
        let monte = AnalysisType::MonteCarlo;

        assert_eq!(worst, AnalysisType::WorstCase);
        assert_ne!(worst, rss);
        assert_ne!(rss, monte);
    }
}
