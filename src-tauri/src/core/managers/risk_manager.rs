use std::sync::Arc;
use uuid::Uuid;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{EntityType, EntityMetadata, Risk, Hazard, RiskControl};
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

    // ============================================================================
    // Hazard Methods
    // ============================================================================

    /// Create a new Hazard entity
    pub fn create_hazard(
        &self,
        name: String,
        description: String,
        causes: Vec<String>,
        harms: Vec<String>,
    ) -> EdtResult<Hazard> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Hazard name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Hazard);

        let hazard = Hazard {
            metadata,
            name,
            description,
            notes: None,
            causes,
            harms,
        };

        self.storage.write_hazard(&hazard)?;

        Ok(hazard)
    }

    /// Get a Hazard by ID
    pub fn get_hazard(&self, id: &Uuid) -> EdtResult<Hazard> {
        if !self.storage.exists(&EntityType::Hazard, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_hazard(id)
    }

    /// Update a Hazard
    pub fn update_hazard(&self, hazard: Hazard) -> EdtResult<Hazard> {
        if hazard.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Hazard name cannot be empty".to_string()));
        }

        let mut updated = hazard;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_hazard(&updated)?;

        Ok(updated)
    }

    /// Delete a Hazard
    pub fn delete_hazard(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Hazard, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Hazard, id)
    }

    /// List all Hazard IDs
    pub fn list_hazard_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Hazard)
    }

    // ============================================================================
    // RiskControl Methods
    // ============================================================================

    /// Create a new RiskControl entity
    pub fn create_risk_control(
        &self,
        name: String,
        description: String,
        control_type: String,
    ) -> EdtResult<RiskControl> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("RiskControl name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::RiskControl);

        let control = RiskControl {
            metadata,
            name,
            description,
            notes: None,
            control_type,
        };

        self.storage.write_risk_control(&control)?;

        Ok(control)
    }

    /// Get a RiskControl by ID
    pub fn get_risk_control(&self, id: &Uuid) -> EdtResult<RiskControl> {
        if !self.storage.exists(&EntityType::RiskControl, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_risk_control(id)
    }

    /// Update a RiskControl
    pub fn update_risk_control(&self, control: RiskControl) -> EdtResult<RiskControl> {
        if control.name.trim().is_empty() {
            return Err(EdtError::ValidationError("RiskControl name cannot be empty".to_string()));
        }

        let mut updated = control;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_risk_control(&updated)?;

        Ok(updated)
    }

    /// Delete a RiskControl
    pub fn delete_risk_control(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::RiskControl, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::RiskControl, id)
    }

    /// List all RiskControl IDs
    pub fn list_risk_control_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::RiskControl)
    }
}
