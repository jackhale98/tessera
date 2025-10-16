use std::sync::Arc;
use uuid::Uuid;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{EntityType, EntityMetadata, Requirement};
use chrono::Utc;

/// Manages Requirement entities
pub struct RequirementManager {
    storage: Arc<RonStorage>,
}

impl RequirementManager {
    pub fn new(storage: Arc<RonStorage>) -> Self {
        Self { storage }
    }

    /// Create a new Requirement entity
    pub fn create_requirement(
        &self,
        name: String,
        description: String,
        requirement_type: String,
    ) -> EdtResult<Requirement> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Requirement name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Requirement);

        let requirement = Requirement {
            metadata,
            name,
            description,
            notes: None,
            requirement_type,
            rationale: None,
            source: None,
            verification_method: None,
        };

        self.storage.write_requirement(&requirement)?;

        Ok(requirement)
    }

    /// Get a Requirement by ID
    pub fn get_requirement(&self, id: &Uuid) -> EdtResult<Requirement> {
        if !self.storage.exists(&EntityType::Requirement, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_requirement(id)
    }

    /// Update a Requirement
    pub fn update_requirement(&self, requirement: Requirement) -> EdtResult<Requirement> {
        if requirement.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Requirement name cannot be empty".to_string()));
        }

        let mut updated = requirement;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_requirement(&updated)?;

        Ok(updated)
    }

    /// Delete a Requirement
    pub fn delete_requirement(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Requirement, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Requirement, id)
    }

    /// List all Requirement IDs
    pub fn list_requirement_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Requirement)
    }
}
