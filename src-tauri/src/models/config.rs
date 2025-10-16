use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Project-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub schema_version: String,

    // Critical path
    pub critical_path_milestone_id: Option<Uuid>,

    // Requirements
    pub requirement_types: Vec<String>,

    // Risk management
    pub risk_types: Vec<String>,
    pub risk_control_types: Vec<String>,
    pub severity_levels: Vec<u32>,
    pub probability_levels: Vec<u32>,
    pub risk_matrix: HashMap<String, u32>, // Key format: "probability,severity"
    pub acceptable_risk_threshold: u32,
    pub warn_hazard_without_risk: bool,
    pub warn_risk_without_control: bool,
    pub warn_risk_without_verification: bool,

    // General entities
    pub general_entity_types: Vec<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let mut risk_matrix = HashMap::new();

        // Default 5x5 risk matrix
        for prob in 1..=5 {
            for sev in 1..=5 {
                let key = format!("{},{}", prob, sev);
                let score = prob * sev;
                risk_matrix.insert(key, score);
            }
        }

        Self {
            schema_version: "1.0.0".to_string(),
            critical_path_milestone_id: None,
            requirement_types: vec![
                "User Requirement".to_string(),
                "System Requirement".to_string(),
                "Design Requirement".to_string(),
                "Software Requirement".to_string(),
            ],
            risk_types: vec![
                "Safety Risk".to_string(),
                "Performance Risk".to_string(),
                "Quality Risk".to_string(),
            ],
            risk_control_types: vec![
                "Design Control".to_string(),
                "Process Control".to_string(),
                "Procedural Control".to_string(),
            ],
            severity_levels: vec![1, 2, 3, 4, 5],
            probability_levels: vec![1, 2, 3, 4, 5],
            risk_matrix,
            acceptable_risk_threshold: 10,
            warn_hazard_without_risk: true,
            warn_risk_without_control: true,
            warn_risk_without_verification: true,
            general_entity_types: vec![
                "Test Equipment".to_string(),
                "Software Module".to_string(),
                "Standard Operating Procedure".to_string(),
            ],
        }
    }
}

impl ProjectConfig {
    /// Get risk score from matrix
    pub fn get_risk_score(&self, probability: u32, severity: u32) -> Option<u32> {
        let key = format!("{},{}", probability, severity);
        self.risk_matrix.get(&key).copied()
    }

    /// Check if risk score is acceptable
    pub fn is_risk_acceptable(&self, risk_score: u32) -> bool {
        risk_score <= self.acceptable_risk_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();

        assert_eq!(config.schema_version, "1.0.0");
        assert_eq!(config.requirement_types.len(), 4);
        assert!(config.requirement_types.contains(&"User Requirement".to_string()));
        assert!(config.requirement_types.contains(&"System Requirement".to_string()));
        assert_eq!(config.acceptable_risk_threshold, 10);
        assert!(config.warn_hazard_without_risk);
    }

    #[test]
    fn test_risk_matrix() {
        let config = ProjectConfig::default();

        // Test various risk score calculations
        assert_eq!(config.get_risk_score(1, 1), Some(1));
        assert_eq!(config.get_risk_score(2, 3), Some(6));
        assert_eq!(config.get_risk_score(5, 5), Some(25));
        assert_eq!(config.get_risk_score(10, 10), None); // Out of range
    }

    #[test]
    fn test_is_risk_acceptable() {
        let config = ProjectConfig::default();

        assert!(config.is_risk_acceptable(1));
        assert!(config.is_risk_acceptable(5));
        assert!(config.is_risk_acceptable(10));
        assert!(!config.is_risk_acceptable(11));
        assert!(!config.is_risk_acceptable(25));
    }

    #[test]
    fn test_config_serialization_toml() {
        let config = ProjectConfig::default();

        // Serialize to TOML
        let serialized = toml::to_string(&config).expect("Failed to serialize");
        assert!(serialized.contains("schema_version"));
        assert!(serialized.contains("User Requirement"));
        assert!(serialized.contains("acceptable_risk_threshold"));

        // Deserialize back
        let deserialized: ProjectConfig = toml::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.schema_version, config.schema_version);
        assert_eq!(deserialized.acceptable_risk_threshold, config.acceptable_risk_threshold);
    }

    #[test]
    fn test_custom_config() {
        let mut config = ProjectConfig::default();

        // Customize config
        config.acceptable_risk_threshold = 15;
        config.requirement_types.push("Safety Requirement".to_string());
        config.general_entity_types.push("Custom Type".to_string());

        assert_eq!(config.acceptable_risk_threshold, 15);
        assert_eq!(config.requirement_types.len(), 5);
        assert!(config.requirement_types.contains(&"Safety Requirement".to_string()));
    }

    #[test]
    fn test_critical_path_milestone() {
        let mut config = ProjectConfig::default();
        assert!(config.critical_path_milestone_id.is_none());

        let milestone_id = Uuid::new_v4();
        config.critical_path_milestone_id = Some(milestone_id);

        assert_eq!(config.critical_path_milestone_id, Some(milestone_id));
    }

    #[test]
    fn test_warning_flags() {
        let mut config = ProjectConfig::default();

        assert!(config.warn_hazard_without_risk);
        assert!(config.warn_risk_without_control);
        assert!(config.warn_risk_without_verification);

        config.warn_hazard_without_risk = false;
        assert!(!config.warn_hazard_without_risk);
    }

    #[test]
    fn test_risk_levels() {
        let config = ProjectConfig::default();

        assert_eq!(config.severity_levels, vec![1, 2, 3, 4, 5]);
        assert_eq!(config.probability_levels, vec![1, 2, 3, 4, 5]);
    }
}
