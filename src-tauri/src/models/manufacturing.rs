use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::EntityMetadata;

/// Manufacturing process status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessStatus {
    Planned,
    InProgress,
    Completed,
    OnHold,
    Cancelled,
}

/// Quality control result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityStatus {
    Pending,
    Passed,
    Failed,
    Rework,
}

/// Work instruction step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkInstructionStep {
    pub step_number: u32,
    pub operation: String,
    pub description: String,
    pub tools_required: Vec<String>,
    pub estimated_time_minutes: Option<f64>,
    pub safety_notes: Vec<String>,
    pub quality_checks: Vec<String>,
}

/// Quality control checkpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityCheckpoint {
    pub checkpoint_id: String,
    pub description: String,
    pub measurement_type: String,
    pub specification: String,
    pub measured_value: Option<String>,
    pub status: QualityStatus,
    pub inspector: Option<String>,
    pub inspected_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Production batch information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductionBatch {
    pub batch_number: String,
    pub quantity_planned: u32,
    pub quantity_completed: u32,
    pub quantity_passed: u32,
    pub quantity_failed: u32,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Manufacturing entity - process plans, work instructions, production data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manufacturing {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,

    // Process details
    pub process_type: String, // Machining, Assembly, Inspection, Packaging, etc.
    pub work_center: Option<String>,
    pub equipment_required: Vec<String>,
    pub work_instructions: Vec<WorkInstructionStep>,

    // Scheduling
    pub status: ProcessStatus,
    pub priority: u32, // 1 = highest priority
    pub planned_start: Option<DateTime<Utc>>,
    pub planned_end: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,

    // Resources
    pub operators: Vec<String>,
    pub setup_time_minutes: Option<f64>,
    pub cycle_time_minutes: Option<f64>,

    // Production tracking
    pub batches: Vec<ProductionBatch>,
    pub quality_checkpoints: Vec<QualityCheckpoint>,

    // Materials
    pub materials_required: Vec<String>,
    pub material_lot_numbers: Vec<String>,

    // Documentation
    pub drawings: Vec<String>,
    pub specifications: Vec<String>,
    pub deviations: Vec<String>,
    pub nonconformances: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityType;

    #[test]
    fn test_manufacturing_creation() {
        let metadata = EntityMetadata::new(EntityType::Manufacturing);

        let manufacturing = Manufacturing {
            metadata: metadata.clone(),
            name: "MFG-001: Bracket assembly".to_string(),
            description: "Assembly process for mounting bracket".to_string(),
            notes: Some("Use torque wrench for all fasteners".to_string()),
            process_type: "Assembly".to_string(),
            work_center: Some("Assembly Line 1".to_string()),
            equipment_required: vec![
                "Torque wrench".to_string(),
                "Fixture A-123".to_string(),
            ],
            work_instructions: vec![
                WorkInstructionStep {
                    step_number: 1,
                    operation: "Position base plate".to_string(),
                    description: "Place base plate in fixture".to_string(),
                    tools_required: vec!["Fixture A-123".to_string()],
                    estimated_time_minutes: Some(2.0),
                    safety_notes: vec!["Wear safety glasses".to_string()],
                    quality_checks: vec!["Verify orientation".to_string()],
                },
                WorkInstructionStep {
                    step_number: 2,
                    operation: "Install fasteners".to_string(),
                    description: "Install 4x M6 screws, torque to 5 Nm".to_string(),
                    tools_required: vec!["Torque wrench".to_string()],
                    estimated_time_minutes: Some(5.0),
                    safety_notes: vec![],
                    quality_checks: vec!["Verify torque".to_string()],
                },
            ],
            status: ProcessStatus::Planned,
            priority: 1,
            planned_start: None,
            planned_end: None,
            actual_start: None,
            actual_end: None,
            operators: vec![],
            setup_time_minutes: Some(30.0),
            cycle_time_minutes: Some(7.0),
            batches: vec![],
            quality_checkpoints: vec![],
            materials_required: vec!["Base plate".to_string(), "M6 screws".to_string()],
            material_lot_numbers: vec![],
            drawings: vec!["DWG-001".to_string()],
            specifications: vec!["SPEC-001".to_string()],
            deviations: vec![],
            nonconformances: vec![],
        };

        assert_eq!(manufacturing.process_type, "Assembly");
        assert_eq!(manufacturing.work_instructions.len(), 2);
        assert_eq!(manufacturing.equipment_required.len(), 2);
        assert_eq!(manufacturing.status, ProcessStatus::Planned);
    }

    #[test]
    fn test_manufacturing_with_production_data() {
        let metadata = EntityMetadata::new(EntityType::Manufacturing);

        let manufacturing = Manufacturing {
            metadata,
            name: "MFG-002: CNC machining".to_string(),
            description: "Machine housing component".to_string(),
            notes: None,
            process_type: "Machining".to_string(),
            work_center: Some("CNC Mill 3".to_string()),
            equipment_required: vec!["CNC Mill".to_string()],
            work_instructions: vec![],
            status: ProcessStatus::InProgress,
            priority: 2,
            planned_start: Some(Utc::now()),
            planned_end: None,
            actual_start: Some(Utc::now()),
            actual_end: None,
            operators: vec!["Operator A".to_string()],
            setup_time_minutes: Some(45.0),
            cycle_time_minutes: Some(12.5),
            batches: vec![
                ProductionBatch {
                    batch_number: "BATCH-001".to_string(),
                    quantity_planned: 100,
                    quantity_completed: 75,
                    quantity_passed: 73,
                    quantity_failed: 2,
                    start_date: Some(Utc::now()),
                    end_date: None,
                },
            ],
            quality_checkpoints: vec![
                QualityCheckpoint {
                    checkpoint_id: "QC-001".to_string(),
                    description: "Dimensional inspection".to_string(),
                    measurement_type: "CMM".to_string(),
                    specification: "50.0mm ±0.1mm".to_string(),
                    measured_value: Some("50.05mm".to_string()),
                    status: QualityStatus::Passed,
                    inspector: Some("Inspector B".to_string()),
                    inspected_at: Some(Utc::now()),
                    notes: None,
                },
            ],
            materials_required: vec!["Aluminum 6061".to_string()],
            material_lot_numbers: vec!["LOT-12345".to_string()],
            drawings: vec!["DWG-HOUSING-001".to_string()],
            specifications: vec![],
            deviations: vec![],
            nonconformances: vec![],
        };

        assert_eq!(manufacturing.status, ProcessStatus::InProgress);
        assert_eq!(manufacturing.batches.len(), 1);
        assert_eq!(manufacturing.batches[0].quantity_completed, 75);
        assert_eq!(manufacturing.quality_checkpoints.len(), 1);
        assert_eq!(manufacturing.quality_checkpoints[0].status, QualityStatus::Passed);
    }

    #[test]
    fn test_manufacturing_with_quality_failure() {
        let metadata = EntityMetadata::new(EntityType::Manufacturing);

        let manufacturing = Manufacturing {
            metadata,
            name: "MFG-003: Final inspection".to_string(),
            description: "Final quality inspection".to_string(),
            notes: None,
            process_type: "Inspection".to_string(),
            work_center: Some("QC Lab".to_string()),
            equipment_required: vec!["CMM".to_string()],
            work_instructions: vec![],
            status: ProcessStatus::OnHold,
            priority: 1,
            planned_start: None,
            planned_end: None,
            actual_start: None,
            actual_end: None,
            operators: vec![],
            setup_time_minutes: None,
            cycle_time_minutes: Some(15.0),
            batches: vec![],
            quality_checkpoints: vec![
                QualityCheckpoint {
                    checkpoint_id: "QC-FINAL".to_string(),
                    description: "Surface finish check".to_string(),
                    measurement_type: "Visual".to_string(),
                    specification: "Ra < 3.2 μm".to_string(),
                    measured_value: Some("Ra = 4.5 μm".to_string()),
                    status: QualityStatus::Failed,
                    inspector: Some("Inspector C".to_string()),
                    inspected_at: Some(Utc::now()),
                    notes: Some("Surface too rough, requires rework".to_string()),
                },
            ],
            materials_required: vec![],
            material_lot_numbers: vec![],
            drawings: vec![],
            specifications: vec!["SURF-SPEC-001".to_string()],
            deviations: vec![],
            nonconformances: vec!["NCR-001".to_string()],
        };

        assert_eq!(manufacturing.status, ProcessStatus::OnHold);
        assert_eq!(manufacturing.quality_checkpoints[0].status, QualityStatus::Failed);
        assert_eq!(manufacturing.nonconformances.len(), 1);
    }

    #[test]
    fn test_process_status_variants() {
        assert_eq!(ProcessStatus::Planned, ProcessStatus::Planned);
        assert_ne!(ProcessStatus::Planned, ProcessStatus::InProgress);
        assert_ne!(ProcessStatus::Completed, ProcessStatus::Cancelled);
    }

    #[test]
    fn test_quality_status_variants() {
        assert_eq!(QualityStatus::Passed, QualityStatus::Passed);
        assert_ne!(QualityStatus::Passed, QualityStatus::Failed);
    }

    #[test]
    fn test_work_instruction_step_equality() {
        let step1 = WorkInstructionStep {
            step_number: 1,
            operation: "Op 1".to_string(),
            description: "Desc 1".to_string(),
            tools_required: vec![],
            estimated_time_minutes: Some(5.0),
            safety_notes: vec![],
            quality_checks: vec![],
        };

        let step2 = WorkInstructionStep {
            step_number: 1,
            operation: "Op 1".to_string(),
            description: "Desc 1".to_string(),
            tools_required: vec![],
            estimated_time_minutes: Some(5.0),
            safety_notes: vec![],
            quality_checks: vec![],
        };

        assert_eq!(step1, step2);
    }

    #[test]
    fn test_manufacturing_serialization() {
        let metadata = EntityMetadata::new(EntityType::Manufacturing);

        let manufacturing = Manufacturing {
            metadata,
            name: "MFG-TEST".to_string(),
            description: "Test manufacturing".to_string(),
            notes: None,
            process_type: "Assembly".to_string(),
            work_center: None,
            equipment_required: vec![],
            work_instructions: vec![],
            status: ProcessStatus::Completed,
            priority: 1,
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

        // Serialize to RON
        let serialized = ron::to_string(&manufacturing).expect("Failed to serialize");
        assert!(serialized.contains("MFG-TEST"));
        assert!(serialized.contains("Completed"));

        // Deserialize back
        let deserialized: Manufacturing = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, manufacturing.name);
        assert_eq!(deserialized.status, manufacturing.status);
    }

    #[test]
    fn test_production_batch_tracking() {
        let batch = ProductionBatch {
            batch_number: "B-001".to_string(),
            quantity_planned: 100,
            quantity_completed: 95,
            quantity_passed: 90,
            quantity_failed: 5,
            start_date: Some(Utc::now()),
            end_date: None,
        };

        assert_eq!(batch.quantity_planned, 100);
        assert_eq!(batch.quantity_passed + batch.quantity_failed, 95);
    }
}
