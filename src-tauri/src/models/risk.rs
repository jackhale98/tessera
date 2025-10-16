use serde::{Deserialize, Serialize};
use crate::models::EntityMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hazard {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub causes: Vec<String>,
    pub harms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub risk_type: String, // From config

    // Initial risk assessment
    pub probability: u32,
    pub severity: u32,
    pub risk_score: u32, // Calculated from matrix

    // Residual risk (after controls)
    pub residual_probability: Option<u32>,
    pub residual_severity: Option<u32>,
    pub residual_risk_score: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskControl {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub control_type: String, // From config
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityType;

    #[test]
    fn test_hazard_creation() {
        let metadata = EntityMetadata::new(EntityType::Hazard);

        let hazard = Hazard {
            metadata: metadata.clone(),
            name: "HAZ-001: Electrical shock".to_string(),
            description: "User may contact live electrical components".to_string(),
            notes: Some("High priority hazard".to_string()),
            causes: vec![
                "Improper assembly".to_string(),
                "Missing insulation".to_string(),
            ],
            harms: vec![
                "Injury to user".to_string(),
                "Equipment damage".to_string(),
            ],
        };

        assert_eq!(hazard.name, "HAZ-001: Electrical shock");
        assert_eq!(hazard.causes.len(), 2);
        assert_eq!(hazard.harms.len(), 2);
    }

    #[test]
    fn test_risk_creation() {
        let metadata = EntityMetadata::new(EntityType::Risk);

        let risk = Risk {
            metadata: metadata.clone(),
            name: "RISK-001: Electrical shock during operation".to_string(),
            description: "Risk of electrical shock if user touches exposed terminals".to_string(),
            notes: None,
            risk_type: "Safety Risk".to_string(),
            probability: 2,
            severity: 5,
            risk_score: 10, // 2 * 5 from matrix
            residual_probability: Some(1),
            residual_severity: Some(5),
            residual_risk_score: Some(5),
        };

        assert_eq!(risk.probability, 2);
        assert_eq!(risk.severity, 5);
        assert_eq!(risk.risk_score, 10);
        assert_eq!(risk.residual_risk_score, Some(5));
    }

    #[test]
    fn test_risk_control_creation() {
        let metadata = EntityMetadata::new(EntityType::RiskControl);

        let control = RiskControl {
            metadata: metadata.clone(),
            name: "CTRL-001: Insulation covers".to_string(),
            description: "Add insulation covers to all exposed electrical terminals".to_string(),
            notes: Some("To be implemented in Rev B".to_string()),
            control_type: "Design Control".to_string(),
        };

        assert_eq!(control.name, "CTRL-001: Insulation covers");
        assert_eq!(control.control_type, "Design Control");
    }

    #[test]
    fn test_hazard_serialization() {
        let metadata = EntityMetadata::new(EntityType::Hazard);

        let hazard = Hazard {
            metadata,
            name: "HAZ-TEST".to_string(),
            description: "Test hazard".to_string(),
            notes: None,
            causes: vec!["Cause 1".to_string()],
            harms: vec!["Harm 1".to_string()],
        };

        // Serialize to RON
        let serialized = ron::to_string(&hazard).expect("Failed to serialize");
        assert!(serialized.contains("HAZ-TEST"));

        // Deserialize back
        let deserialized: Hazard = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, hazard.name);
        assert_eq!(deserialized.causes.len(), 1);
    }

    #[test]
    fn test_risk_serialization() {
        let metadata = EntityMetadata::new(EntityType::Risk);

        let risk = Risk {
            metadata,
            name: "RISK-TEST".to_string(),
            description: "Test risk".to_string(),
            notes: None,
            risk_type: "Test Type".to_string(),
            probability: 3,
            severity: 4,
            risk_score: 12,
            residual_probability: None,
            residual_severity: None,
            residual_risk_score: None,
        };

        // Serialize to RON
        let serialized = ron::to_string(&risk).expect("Failed to serialize");
        assert!(serialized.contains("RISK-TEST"));

        // Deserialize back
        let deserialized: Risk = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, risk.name);
        assert_eq!(deserialized.risk_score, 12);
    }
}
