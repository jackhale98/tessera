use serde::{Deserialize, Serialize};
use crate::models::EntityMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,
    pub requirement_type: String, // From config
    pub rationale: Option<String>,
    pub source: Option<String>,
    pub verification_method: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityType;

    #[test]
    fn test_requirement_creation() {
        let metadata = EntityMetadata::new(EntityType::Requirement);

        let requirement = Requirement {
            metadata: metadata.clone(),
            name: "REQ-001: System shall power on".to_string(),
            description: "The system shall power on within 5 seconds of pressing the power button".to_string(),
            notes: Some("Critical requirement".to_string()),
            requirement_type: "System Requirement".to_string(),
            rationale: Some("User expectation for quick startup".to_string()),
            source: Some("Customer feedback".to_string()),
            verification_method: Some("Test procedure TP-001".to_string()),
        };

        assert_eq!(requirement.name, "REQ-001: System shall power on");
        assert_eq!(requirement.requirement_type, "System Requirement");
        assert!(requirement.rationale.is_some());
    }

    #[test]
    fn test_requirement_serialization() {
        let metadata = EntityMetadata::new(EntityType::Requirement);

        let requirement = Requirement {
            metadata,
            name: "REQ-002".to_string(),
            description: "Test requirement".to_string(),
            notes: None,
            requirement_type: "User Requirement".to_string(),
            rationale: None,
            source: None,
            verification_method: None,
        };

        // Serialize to RON
        let serialized = ron::to_string(&requirement).expect("Failed to serialize");
        assert!(serialized.contains("REQ-002"));
        assert!(serialized.contains("User Requirement"));

        // Deserialize back
        let deserialized: Requirement = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, requirement.name);
    }
}
