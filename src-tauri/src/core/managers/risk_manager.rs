use std::sync::Arc;
use uuid::Uuid;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{EntityType, EntityMetadata, Risk};
use chrono::Utc;

/// Manages Risk entities
pub struct RiskManager {
    storage: Arc<RonStorage>,
}

impl RiskManager {
    pub fn new(storage: Arc<RonStorage>) -> Self {
        Self { storage }
    }

    /// Create a new Risk entity
    pub fn create_risk(
        &self,
        name: String,
        description: String,
        risk_type: String,
        probability: u32,
        severity: u32,
    ) -> EdtResult<Risk> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Risk name cannot be empty".to_string()));
        }

        // Simple risk score calculation (probability * severity)
        let risk_score = probability * severity;

        let metadata = EntityMetadata::new(EntityType::Risk);

        let risk = Risk {
            metadata,
            name,
            description,
            notes: None,
            risk_type,
            probability,
            severity,
            risk_score,
            residual_probability: None,
            residual_severity: None,
            residual_risk_score: None,
        };

        self.storage.write_risk(&risk)?;

        Ok(risk)
    }

    /// Get a Risk by ID
    pub fn get_risk(&self, id: &Uuid) -> EdtResult<Risk> {
        if !self.storage.exists(&EntityType::Risk, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_risk(id)
    }

    /// Update a Risk
    pub fn update_risk(&self, risk: Risk) -> EdtResult<Risk> {
        if risk.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Risk name cannot be empty".to_string()));
        }

        let mut updated = risk;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_risk(&updated)?;

        Ok(updated)
    }

    /// Delete a Risk
    pub fn delete_risk(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Risk, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Risk, id)
    }

    /// List all Risk IDs
    pub fn list_risk_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Risk)
    }
}
