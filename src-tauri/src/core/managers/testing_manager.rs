use std::sync::Arc;
use uuid::Uuid;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{
    EntityType, Verification, Validation, TestStatus, TestPriority,
    TestStep, EntityMetadata,
};
use chrono::Utc;

/// Manages Verification and Validation entities
pub struct TestingManager {
    storage: Arc<RonStorage>,
}

impl TestingManager {
    pub fn new(storage: Arc<RonStorage>) -> Self {
        Self { storage }
    }

    // ============================================================================
    // Verification Methods
    // ============================================================================

    /// Create a new Verification entity
    pub fn create_verification(
        &self,
        name: String,
        description: String,
        test_type: String,
        test_steps: Vec<TestStep>,
        acceptance_criteria: Vec<String>,
        priority: TestPriority,
    ) -> EdtResult<Verification> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Verification name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Verification);

        let verification = Verification {
            metadata,
            name,
            description,
            notes: None,
            test_type,
            test_procedure: None,
            test_steps,
            acceptance_criteria,
            status: TestStatus::NotStarted,
            priority,
            executed_by: None,
            executed_at: None,
            execution_time_seconds: None,
            actual_result: None,
            pass_fail: None,
            defects_found: vec![],
        };

        self.storage.write_verification(&verification)?;
        Ok(verification)
    }

    /// Get a Verification by ID
    pub fn get_verification(&self, id: &Uuid) -> EdtResult<Verification> {
        if !self.storage.exists(&EntityType::Verification, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_verification(id)
    }

    /// Update a Verification
    pub fn update_verification(&self, verification: Verification) -> EdtResult<Verification> {
        if verification.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Verification name cannot be empty".to_string()));
        }

        let mut updated = verification;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_verification(&updated)?;
        Ok(updated)
    }

    /// Delete a Verification
    pub fn delete_verification(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Verification, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Verification, id)
    }

    /// List all Verification IDs
    pub fn list_verification_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Verification)
    }

    // ============================================================================
    // Validation Methods
    // ============================================================================

    /// Create a new Validation entity
    pub fn create_validation(
        &self,
        name: String,
        description: String,
        validation_type: String,
        participants: Vec<String>,
        success_criteria: Vec<String>,
        priority: TestPriority,
    ) -> EdtResult<Validation> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Validation name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Validation);

        let validation = Validation {
            metadata,
            name,
            description,
            notes: None,
            validation_type,
            protocol: None,
            participants,
            environment: None,
            status: TestStatus::NotStarted,
            priority,
            start_date: None,
            end_date: None,
            success_criteria,
            results_summary: None,
            user_feedback: vec![],
            issues_identified: vec![],
            approved: None,
            approved_by: None,
            approved_at: None,
        };

        self.storage.write_validation(&validation)?;
        Ok(validation)
    }

    /// Get a Validation by ID
    pub fn get_validation(&self, id: &Uuid) -> EdtResult<Validation> {
        if !self.storage.exists(&EntityType::Validation, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_validation(id)
    }

    /// Update a Validation
    pub fn update_validation(&self, validation: Validation) -> EdtResult<Validation> {
        if validation.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Validation name cannot be empty".to_string()));
        }

        let mut updated = validation;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_validation(&updated)?;
        Ok(updated)
    }

    /// Delete a Validation
    pub fn delete_validation(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Validation, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Validation, id)
    }

    /// List all Validation IDs
    pub fn list_validation_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Validation)
    }
}
