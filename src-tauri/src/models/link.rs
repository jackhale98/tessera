use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::EntityType;

/// Link between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub from_entity_id: Uuid,
    pub from_entity_type: EntityType,
    pub to_entity_id: Uuid,
    pub to_entity_type: EntityType,
    pub link_type: LinkType,
    pub metadata: Option<LinkMetadata>,
    pub created_at: DateTime<Utc>,
}

impl Link {
    pub fn new(
        from_entity_id: Uuid,
        from_entity_type: EntityType,
        to_entity_id: Uuid,
        to_entity_type: EntityType,
        link_type: LinkType,
        metadata: Option<LinkMetadata>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_entity_id,
            from_entity_type,
            to_entity_id,
            to_entity_type,
            link_type,
            metadata,
            created_at: Utc::now(),
        }
    }
}

/// Types of links between entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LinkType {
    // Generic
    Related,

    // Hierarchical
    Parent,
    Child,

    // Design specific
    Contains,       // Assembly → Component
    PartOf,         // Component → Assembly
    HasFeature,     // Component → Feature
    Mates,          // Feature → Feature (via Mate)
    UsedInStackup,  // Feature → Stackup

    // Requirements
    Derives,        // Parent req → child req
    Satisfies,      // Design → Requirement
    Verifies,       // Verification → Requirement/Risk

    // Risk
    Mitigates,      // RiskControl → Risk
    Hazardous,      // Component → Risk

    // BOM
    Supplies,       // Supplier → Component
    Quotes,         // Quote → Component

    // Comments
    Comments,       // Comment → Any entity
    Replies,        // Comment → Comment

    // Manufacturing
    Manufactures,   // Manufacturing → Component/Assembly
}

/// Optional metadata for links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMetadata {
    pub quantity: Option<u32>, // For component-assembly links
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let link = Link::new(
            from_id,
            EntityType::Component,
            to_id,
            EntityType::Requirement,
            LinkType::Satisfies,
            None,
        );

        assert_eq!(link.from_entity_id, from_id);
        assert_eq!(link.to_entity_id, to_id);
        assert_eq!(link.from_entity_type, EntityType::Component);
        assert_eq!(link.to_entity_type, EntityType::Requirement);
        assert_eq!(link.link_type, LinkType::Satisfies);
        assert!(link.metadata.is_none());
    }

    #[test]
    fn test_link_with_metadata() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let metadata = LinkMetadata {
            quantity: Some(5),
            notes: Some("5 units required per assembly".to_string()),
        };

        let link = Link::new(
            from_id,
            EntityType::Assembly,
            to_id,
            EntityType::Component,
            LinkType::Contains,
            Some(metadata.clone()),
        );

        assert_eq!(link.link_type, LinkType::Contains);
        assert!(link.metadata.is_some());

        let link_meta = link.metadata.unwrap();
        assert_eq!(link_meta.quantity, Some(5));
        assert!(link_meta.notes.is_some());
    }

    #[test]
    fn test_link_type_variants() {
        assert_eq!(LinkType::Related, LinkType::Related);
        assert_ne!(LinkType::Related, LinkType::Parent);
        assert_ne!(LinkType::Contains, LinkType::PartOf);
        assert_ne!(LinkType::Satisfies, LinkType::Verifies);
        assert_ne!(LinkType::Mitigates, LinkType::Hazardous);
    }

    #[test]
    fn test_link_serialization() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let link = Link::new(
            from_id,
            EntityType::Risk,
            to_id,
            EntityType::RiskControl,
            LinkType::Mitigates,
            None,
        );

        // Serialize to RON
        let serialized = ron::to_string(&link).expect("Failed to serialize");
        assert!(serialized.contains("Mitigates"));
        assert!(serialized.contains("Risk"));
        assert!(serialized.contains("RiskControl"));

        // Deserialize back
        let deserialized: Link = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.id, link.id);
        assert_eq!(deserialized.link_type, link.link_type);
        assert_eq!(deserialized.from_entity_id, link.from_entity_id);
    }

    #[test]
    fn test_link_metadata_serialization() {
        let metadata = LinkMetadata {
            quantity: Some(10),
            notes: Some("Test notes".to_string()),
        };

        // Serialize to RON
        let serialized = ron::to_string(&metadata).expect("Failed to serialize");
        assert!(serialized.contains("10"));

        // Deserialize back
        let deserialized: LinkMetadata = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.quantity, Some(10));
    }

    #[test]
    fn test_design_links() {
        let assembly_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();

        let contains_link = Link::new(
            assembly_id,
            EntityType::Assembly,
            component_id,
            EntityType::Component,
            LinkType::Contains,
            Some(LinkMetadata {
                quantity: Some(2),
                notes: None,
            }),
        );

        assert_eq!(contains_link.link_type, LinkType::Contains);
        assert_eq!(contains_link.metadata.as_ref().unwrap().quantity, Some(2));
    }

    #[test]
    fn test_requirement_links() {
        let req_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();

        let satisfies_link = Link::new(
            component_id,
            EntityType::Component,
            req_id,
            EntityType::Requirement,
            LinkType::Satisfies,
            None,
        );

        assert_eq!(satisfies_link.link_type, LinkType::Satisfies);
        assert_eq!(satisfies_link.from_entity_type, EntityType::Component);
        assert_eq!(satisfies_link.to_entity_type, EntityType::Requirement);
    }

    #[test]
    fn test_risk_links() {
        let control_id = Uuid::new_v4();
        let risk_id = Uuid::new_v4();

        let mitigates_link = Link::new(
            control_id,
            EntityType::RiskControl,
            risk_id,
            EntityType::Risk,
            LinkType::Mitigates,
            None,
        );

        assert_eq!(mitigates_link.link_type, LinkType::Mitigates);
    }
}
