use std::sync::Arc;
use uuid::Uuid;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{
    EntityType, Manufacturing, ProcessStatus, WorkInstructionStep,
    QualityCheckpoint, ProductionBatch, EntityMetadata,
};
use chrono::Utc;

/// Manages Manufacturing entities
pub struct ManufacturingManager {
    storage: Arc<RonStorage>,
}

impl ManufacturingManager {
    pub fn new(storage: Arc<RonStorage>) -> Self {
        Self { storage }
    }

    /// Create a new Manufacturing entity
    pub fn create_manufacturing(
        &self,
        name: String,
        description: String,
        process_type: String,
        work_instructions: Vec<WorkInstructionStep>,
        priority: u32,
    ) -> EdtResult<Manufacturing> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Manufacturing name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Manufacturing);

        let manufacturing = Manufacturing {
            metadata,
            name,
            description,
            notes: None,
            process_type,
            work_center: None,
            equipment_required: vec![],
            work_instructions,
            status: ProcessStatus::Planned,
            priority,
            planned_start: None,
            planned_end: None,
            actual_start: None,
            actual_end: None,
            operators: vec![],
            setup_time_minutes: None,
            cycle_time_minutes: None,
            batches: vec![],
            quality_checkpoints: vec![],
            materials_required: vec![],
            material_lot_numbers: vec![],
            drawings: vec![],
            specifications: vec![],
            deviations: vec![],
            nonconformances: vec![],
        };

        self.storage.write_manufacturing(&manufacturing)?;
        Ok(manufacturing)
    }

    /// Get a Manufacturing entity by ID
    pub fn get_manufacturing(&self, id: &Uuid) -> EdtResult<Manufacturing> {
        if !self.storage.exists(&EntityType::Manufacturing, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_manufacturing(id)
    }

    /// Update a Manufacturing entity
    pub fn update_manufacturing(&self, manufacturing: Manufacturing) -> EdtResult<Manufacturing> {
        if manufacturing.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Manufacturing name cannot be empty".to_string()));
        }

        let mut updated = manufacturing;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_manufacturing(&updated)?;
        Ok(updated)
    }

    /// Delete a Manufacturing entity
    pub fn delete_manufacturing(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Manufacturing, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Manufacturing, id)
    }

    /// List all Manufacturing IDs
    pub fn list_manufacturing_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Manufacturing)
    }
}
