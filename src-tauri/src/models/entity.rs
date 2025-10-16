use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Entity metadata shared by all entity types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityMetadata {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: EntityStatus,
}

impl EntityMetadata {
    pub fn new(entity_type: EntityType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entity_type,
            schema_version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
            status: EntityStatus::Draft,
        }
    }
}

/// Entity status workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityStatus {
    Draft,
    PendingApproval,
    Approved,
    Released,
}

/// All supported entity types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    // Project Management
    Task,
    Milestone,
    Resource,
    Calendar,
    Baseline,

    // Requirements
    Requirement,

    // Risk Management
    Hazard,
    Risk,
    RiskControl,

    // Design
    Assembly,
    Component,
    Feature,
    Mate,
    Stackup,
    Supplier,
    Quote,

    // Verification & Validation
    Verification,
    Validation,

    // Manufacturing
    Manufacturing,

    // Collaboration
    Comment,

    // General
    General,
}

impl EntityType {
    /// Returns the folder name for this entity type
    pub fn folder_name(&self) -> &'static str {
        match self {
            EntityType::Task => "tasks",
            EntityType::Milestone => "milestones",
            EntityType::Resource => "resources",
            EntityType::Calendar => "calendars",
            EntityType::Baseline => "baselines",
            EntityType::Requirement => "requirements",
            EntityType::Hazard => "hazards",
            EntityType::Risk => "risks",
            EntityType::RiskControl => "risk_controls",
            EntityType::Assembly => "assemblies",
            EntityType::Component => "components",
            EntityType::Feature => "features",
            EntityType::Mate => "mates",
            EntityType::Stackup => "stackups",
            EntityType::Supplier => "suppliers",
            EntityType::Quote => "quotes",
            EntityType::Verification => "verification",
            EntityType::Validation => "validation",
            EntityType::Manufacturing => "manufacturing",
            EntityType::Comment => "comments",
            EntityType::General => "general",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_metadata_new() {
        let metadata = EntityMetadata::new(EntityType::Task);

        assert_eq!(metadata.entity_type, EntityType::Task);
        assert_eq!(metadata.schema_version, "1.0.0");
        assert_eq!(metadata.status, EntityStatus::Draft);
        assert!(metadata.id != Uuid::nil());
        assert!(metadata.created_at <= Utc::now());
        assert_eq!(metadata.created_at, metadata.updated_at);
    }

    #[test]
    fn test_entity_status_variants() {
        let draft = EntityStatus::Draft;
        let pending = EntityStatus::PendingApproval;
        let approved = EntityStatus::Approved;
        let released = EntityStatus::Released;

        assert_eq!(draft, EntityStatus::Draft);
        assert_ne!(draft, pending);
        assert_ne!(pending, approved);
        assert_ne!(approved, released);
    }

    #[test]
    fn test_entity_type_folder_names() {
        assert_eq!(EntityType::Task.folder_name(), "tasks");
        assert_eq!(EntityType::Milestone.folder_name(), "milestones");
        assert_eq!(EntityType::Resource.folder_name(), "resources");
        assert_eq!(EntityType::Requirement.folder_name(), "requirements");
        assert_eq!(EntityType::Risk.folder_name(), "risks");
        assert_eq!(EntityType::Component.folder_name(), "components");
        assert_eq!(EntityType::Verification.folder_name(), "verification");
        assert_eq!(EntityType::Comment.folder_name(), "comments");
    }

    #[test]
    fn test_entity_type_equality() {
        assert_eq!(EntityType::Task, EntityType::Task);
        assert_ne!(EntityType::Task, EntityType::Milestone);
    }

    #[test]
    fn test_entity_metadata_serialization() {
        let metadata = EntityMetadata::new(EntityType::Task);

        // Serialize to RON
        let serialized = ron::to_string(&metadata).expect("Failed to serialize");
        assert!(serialized.contains("Task"));
        assert!(serialized.contains("Draft"));

        // Deserialize back
        let deserialized: EntityMetadata = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.id, metadata.id);
        assert_eq!(deserialized.entity_type, metadata.entity_type);
    }

    #[test]
    fn test_entity_status_serialization() {
        let status = EntityStatus::PendingApproval;

        let serialized = ron::to_string(&status).expect("Failed to serialize");
        let deserialized: EntityStatus = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized, status);
    }

    #[test]
    fn test_entity_type_serialization() {
        let entity_type = EntityType::Requirement;

        let serialized = ron::to_string(&entity_type).expect("Failed to serialize");
        let deserialized: EntityType = ron::from_str(&serialized)
            .expect("Failed to deserialize");

        assert_eq!(deserialized, entity_type);
    }
}
