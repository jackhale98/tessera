use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::EntityMetadata;

/// Test status for verification and validation activities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestStatus {
    NotStarted,
    InProgress,
    Passed,
    Failed,
    Blocked,
    Skipped,
}

/// Test severity/priority level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Verification entity - confirms product meets specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,

    // Test details
    pub test_type: String, // Unit, Integration, System, etc.
    pub test_procedure: Option<String>,
    pub test_steps: Vec<TestStep>,
    pub acceptance_criteria: Vec<String>,

    // Execution details
    pub status: TestStatus,
    pub priority: TestPriority,
    pub executed_by: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub execution_time_seconds: Option<f64>,

    // Results
    pub actual_result: Option<String>,
    pub pass_fail: Option<bool>,
    pub defects_found: Vec<String>,
}

/// Individual test step within a verification test
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestStep {
    pub step_number: u32,
    pub description: String,
    pub expected_result: String,
    pub actual_result: Option<String>,
    pub passed: Option<bool>,
}

/// Validation entity - confirms product meets user needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,

    // Validation details
    pub validation_type: String, // User Acceptance, Clinical, Field Test, etc.
    pub protocol: Option<String>,
    pub participants: Vec<String>,
    pub environment: Option<String>,

    // Execution details
    pub status: TestStatus,
    pub priority: TestPriority,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,

    // Results
    pub success_criteria: Vec<String>,
    pub results_summary: Option<String>,
    pub user_feedback: Vec<String>,
    pub issues_identified: Vec<String>,
    pub approved: Option<bool>,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityType;

    #[test]
    fn test_verification_creation() {
        let metadata = EntityMetadata::new(EntityType::Verification);

        let verification = Verification {
            metadata: metadata.clone(),
            name: "VER-001: Power supply voltage test".to_string(),
            description: "Verify power supply outputs correct voltage under load".to_string(),
            notes: Some("Critical safety test".to_string()),
            test_type: "System Test".to_string(),
            test_procedure: Some("TP-PSU-001".to_string()),
            test_steps: vec![
                TestStep {
                    step_number: 1,
                    description: "Connect power supply to load".to_string(),
                    expected_result: "5.0V ±0.1V".to_string(),
                    actual_result: None,
                    passed: None,
                },
                TestStep {
                    step_number: 2,
                    description: "Apply full load".to_string(),
                    expected_result: "Voltage remains within spec".to_string(),
                    actual_result: None,
                    passed: None,
                },
            ],
            acceptance_criteria: vec![
                "Voltage within 5.0V ±0.1V".to_string(),
                "No thermal shutdown".to_string(),
            ],
            status: TestStatus::NotStarted,
            priority: TestPriority::Critical,
            executed_by: None,
            executed_at: None,
            execution_time_seconds: None,
            actual_result: None,
            pass_fail: None,
            defects_found: vec![],
        };

        assert_eq!(verification.test_type, "System Test");
        assert_eq!(verification.test_steps.len(), 2);
        assert_eq!(verification.acceptance_criteria.len(), 2);
        assert_eq!(verification.status, TestStatus::NotStarted);
        assert_eq!(verification.priority, TestPriority::Critical);
    }

    #[test]
    fn test_verification_with_results() {
        let metadata = EntityMetadata::new(EntityType::Verification);

        let verification = Verification {
            metadata,
            name: "VER-002: Button response test".to_string(),
            description: "Verify button responds within 100ms".to_string(),
            notes: None,
            test_type: "Unit Test".to_string(),
            test_procedure: None,
            test_steps: vec![
                TestStep {
                    step_number: 1,
                    description: "Press button".to_string(),
                    expected_result: "Response < 100ms".to_string(),
                    actual_result: Some("85ms".to_string()),
                    passed: Some(true),
                },
            ],
            acceptance_criteria: vec!["Response time < 100ms".to_string()],
            status: TestStatus::Passed,
            priority: TestPriority::High,
            executed_by: Some("test_engineer@company.com".to_string()),
            executed_at: Some(Utc::now()),
            execution_time_seconds: Some(15.5),
            actual_result: Some("Button responded in 85ms".to_string()),
            pass_fail: Some(true),
            defects_found: vec![],
        };

        assert_eq!(verification.status, TestStatus::Passed);
        assert_eq!(verification.pass_fail, Some(true));
        assert!(verification.executed_by.is_some());
        assert!(verification.test_steps[0].passed.unwrap());
    }

    #[test]
    fn test_validation_creation() {
        let metadata = EntityMetadata::new(EntityType::Validation);

        let validation = Validation {
            metadata: metadata.clone(),
            name: "VAL-001: User acceptance testing".to_string(),
            description: "Validate system meets user workflow requirements".to_string(),
            notes: Some("5 users from target demographic".to_string()),
            validation_type: "User Acceptance Test".to_string(),
            protocol: Some("VAL-PROTOCOL-001".to_string()),
            participants: vec![
                "User A".to_string(),
                "User B".to_string(),
                "User C".to_string(),
            ],
            environment: Some("Simulated clinical environment".to_string()),
            status: TestStatus::InProgress,
            priority: TestPriority::Critical,
            start_date: Some(Utc::now()),
            end_date: None,
            success_criteria: vec![
                "80% task completion rate".to_string(),
                "Average SUS score > 70".to_string(),
            ],
            results_summary: None,
            user_feedback: vec![],
            issues_identified: vec![],
            approved: None,
            approved_by: None,
            approved_at: None,
        };

        assert_eq!(validation.validation_type, "User Acceptance Test");
        assert_eq!(validation.participants.len(), 3);
        assert_eq!(validation.success_criteria.len(), 2);
        assert_eq!(validation.status, TestStatus::InProgress);
    }

    #[test]
    fn test_validation_with_approval() {
        let metadata = EntityMetadata::new(EntityType::Validation);

        let validation = Validation {
            metadata,
            name: "VAL-002: Field validation".to_string(),
            description: "Field trial with beta users".to_string(),
            notes: None,
            validation_type: "Field Test".to_string(),
            protocol: None,
            participants: vec!["Beta User Group".to_string()],
            environment: Some("Real-world usage".to_string()),
            status: TestStatus::Passed,
            priority: TestPriority::High,
            start_date: Some(Utc::now()),
            end_date: Some(Utc::now()),
            success_criteria: vec!["No critical defects".to_string()],
            results_summary: Some("All success criteria met".to_string()),
            user_feedback: vec![
                "Easy to use".to_string(),
                "Fast response".to_string(),
            ],
            issues_identified: vec!["Minor UI inconsistency".to_string()],
            approved: Some(true),
            approved_by: Some("John Doe".to_string()),
            approved_at: Some(Utc::now()),
        };

        assert_eq!(validation.status, TestStatus::Passed);
        assert_eq!(validation.approved, Some(true));
        assert!(validation.approved_by.is_some());
        assert_eq!(validation.user_feedback.len(), 2);
        assert_eq!(validation.issues_identified.len(), 1);
    }

    #[test]
    fn test_test_status_variants() {
        assert_eq!(TestStatus::NotStarted, TestStatus::NotStarted);
        assert_ne!(TestStatus::NotStarted, TestStatus::Passed);
        assert_ne!(TestStatus::Passed, TestStatus::Failed);
    }

    #[test]
    fn test_test_priority_variants() {
        assert_eq!(TestPriority::Critical, TestPriority::Critical);
        assert_ne!(TestPriority::Critical, TestPriority::Low);
    }

    #[test]
    fn test_verification_serialization() {
        let metadata = EntityMetadata::new(EntityType::Verification);

        let verification = Verification {
            metadata,
            name: "VER-TEST".to_string(),
            description: "Test verification".to_string(),
            notes: None,
            test_type: "Unit Test".to_string(),
            test_procedure: None,
            test_steps: vec![],
            acceptance_criteria: vec!["Pass".to_string()],
            status: TestStatus::Passed,
            priority: TestPriority::Medium,
            executed_by: None,
            executed_at: None,
            execution_time_seconds: None,
            actual_result: None,
            pass_fail: Some(true),
            defects_found: vec![],
        };

        // Serialize to RON
        let serialized = ron::to_string(&verification).expect("Failed to serialize");
        assert!(serialized.contains("VER-TEST"));
        assert!(serialized.contains("Passed"));

        // Deserialize back
        let deserialized: Verification = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, verification.name);
        assert_eq!(deserialized.status, verification.status);
    }

    #[test]
    fn test_validation_serialization() {
        let metadata = EntityMetadata::new(EntityType::Validation);

        let validation = Validation {
            metadata,
            name: "VAL-TEST".to_string(),
            description: "Test validation".to_string(),
            notes: None,
            validation_type: "UAT".to_string(),
            protocol: None,
            participants: vec!["User1".to_string()],
            environment: None,
            status: TestStatus::Passed,
            priority: TestPriority::High,
            start_date: None,
            end_date: None,
            success_criteria: vec!["Pass".to_string()],
            results_summary: None,
            user_feedback: vec![],
            issues_identified: vec![],
            approved: Some(true),
            approved_by: None,
            approved_at: None,
        };

        // Serialize to RON
        let serialized = ron::to_string(&validation).expect("Failed to serialize");
        assert!(serialized.contains("VAL-TEST"));
        assert!(serialized.contains("UAT"));

        // Deserialize back
        let deserialized: Validation = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, validation.name);
        assert_eq!(deserialized.validation_type, validation.validation_type);
    }

    #[test]
    fn test_test_step_equality() {
        let step1 = TestStep {
            step_number: 1,
            description: "Step 1".to_string(),
            expected_result: "Result 1".to_string(),
            actual_result: None,
            passed: None,
        };

        let step2 = TestStep {
            step_number: 1,
            description: "Step 1".to_string(),
            expected_result: "Result 1".to_string(),
            actual_result: None,
            passed: None,
        };

        assert_eq!(step1, step2);
    }
}
