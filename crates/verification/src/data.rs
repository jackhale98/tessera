//! Data structures for verification and testing
//!
//! This module defines the core data types used throughout the verification system,
//! including test procedures, executions, results, and evidence.

use serde::{Deserialize, Serialize};
use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Types of test procedures
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcedureType {
    Unit,
    Integration,
    System,
    Acceptance,
    Performance,
    Security,
    Usability,
    Regression,
    Smoke,
    Custom(String),
}

impl std::fmt::Display for ProcedureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcedureType::Unit => write!(f, "Unit"),
            ProcedureType::Integration => write!(f, "Integration"),
            ProcedureType::System => write!(f, "System"),
            ProcedureType::Acceptance => write!(f, "Acceptance"),
            ProcedureType::Performance => write!(f, "Performance"),
            ProcedureType::Security => write!(f, "Security"),
            ProcedureType::Usability => write!(f, "Usability"),
            ProcedureType::Regression => write!(f, "Regression"),
            ProcedureType::Smoke => write!(f, "Smoke"),
            ProcedureType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Execution methods for test procedures
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMethod {
    Manual,
    Automated,
    SemiAutomated,
    Scripted,
    External,
}

impl std::fmt::Display for ExecutionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMethod::Manual => write!(f, "Manual"),
            ExecutionMethod::Automated => write!(f, "Automated"),
            ExecutionMethod::SemiAutomated => write!(f, "Semi-Automated"),
            ExecutionMethod::Scripted => write!(f, "Scripted"),
            ExecutionMethod::External => write!(f, "External"),
        }
    }
}

/// Status of a test procedure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcedureStatus {
    Draft,
    UnderReview,
    Approved,
    Active,
    Deprecated,
    Retired,
}

impl std::fmt::Display for ProcedureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcedureStatus::Draft => write!(f, "Draft"),
            ProcedureStatus::UnderReview => write!(f, "Under Review"),
            ProcedureStatus::Approved => write!(f, "Approved"),
            ProcedureStatus::Active => write!(f, "Active"),
            ProcedureStatus::Deprecated => write!(f, "Deprecated"),
            ProcedureStatus::Retired => write!(f, "Retired"),
        }
    }
}

/// A single step in a test procedure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStep {
    pub step_number: usize,
    pub description: String,
    pub expected_result: String,
    pub notes: Option<String>,
    pub automation_script: Option<String>,
    pub timeout_seconds: Option<u32>,
}

impl TestStep {
    /// Create a new test step
    pub fn new(step_number: usize, description: String, expected_result: String) -> Self {
        Self {
            step_number,
            description,
            expected_result,
            notes: None,
            automation_script: None,
            timeout_seconds: None,
        }
    }

    /// Set automation script for this step
    pub fn set_automation_script(&mut self, script: String) {
        self.automation_script = Some(script);
    }

    /// Set timeout for this step
    pub fn set_timeout(&mut self, timeout_seconds: u32) {
        self.timeout_seconds = Some(timeout_seconds);
    }

    /// Add notes to this step
    pub fn set_notes(&mut self, notes: String) {
        self.notes = Some(notes);
    }
}

/// Test procedure definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProcedure {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub procedure_type: ProcedureType,
    pub execution_method: ExecutionMethod,
    pub status: ProcedureStatus,
    
    // Procedure definition
    pub preconditions: Vec<String>,
    pub steps: Vec<TestStep>,
    pub postconditions: Vec<String>,
    pub cleanup_steps: Vec<String>,
    
    // Requirements integration
    pub requirement_ids: Vec<tessera_core::Id>,
    pub design_output_ids: Vec<tessera_core::Id>,
    
    // Execution configuration
    pub estimated_duration_minutes: Option<u32>,
    pub required_environment: Option<String>,
    pub required_tools: Vec<String>,
    pub required_data: Vec<String>,
    
    // Management
    pub owner: Option<String>,
    pub reviewer: Option<String>,
    pub approval_date: Option<DateTime<Utc>>,
    pub version: String,
    
    // Metadata
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl TestProcedure {
    /// Create a new test procedure
    pub fn new(name: String, description: String, procedure_type: ProcedureType) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            name,
            description,
            procedure_type,
            execution_method: ExecutionMethod::Manual,
            status: ProcedureStatus::Draft,
            preconditions: Vec::new(),
            steps: Vec::new(),
            postconditions: Vec::new(),
            cleanup_steps: Vec::new(),
            requirement_ids: Vec::new(),
            design_output_ids: Vec::new(),
            estimated_duration_minutes: None,
            required_environment: None,
            required_tools: Vec::new(),
            required_data: Vec::new(),
            owner: None,
            reviewer: None,
            approval_date: None,
            version: "1.0".to_string(),
            tags: Vec::new(),
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    /// Add a test step
    pub fn add_step(&mut self, step: TestStep) {
        self.steps.push(step);
        self.steps.sort_by_key(|s| s.step_number);
        self.updated = Utc::now();
    }

    /// Add a precondition
    pub fn add_precondition(&mut self, condition: String) {
        self.preconditions.push(condition);
        self.updated = Utc::now();
    }

    /// Add a postcondition
    pub fn add_postcondition(&mut self, condition: String) {
        self.postconditions.push(condition);
        self.updated = Utc::now();
    }

    /// Link to requirement
    pub fn link_to_requirement(&mut self, requirement_id: tessera_core::Id) {
        if !self.requirement_ids.contains(&requirement_id) {
            self.requirement_ids.push(requirement_id);
            self.updated = Utc::now();
        }
    }

    /// Link to design output
    pub fn link_to_design_output(&mut self, design_output_id: tessera_core::Id) {
        if !self.design_output_ids.contains(&design_output_id) {
            self.design_output_ids.push(design_output_id);
            self.updated = Utc::now();
        }
    }

    /// Update procedure status
    pub fn update_status(&mut self, status: ProcedureStatus) {
        self.status = status;
        if status == ProcedureStatus::Approved {
            self.approval_date = Some(Utc::now());
        }
        self.updated = Utc::now();
    }

    /// Set execution method
    pub fn set_execution_method(&mut self, method: ExecutionMethod) {
        self.execution_method = method;
        self.updated = Utc::now();
    }

    /// Set estimated duration
    pub fn set_estimated_duration(&mut self, minutes: u32) {
        self.estimated_duration_minutes = Some(minutes);
        self.updated = Utc::now();
    }

    /// Set required environment
    pub fn set_required_environment(&mut self, environment: String) {
        self.required_environment = Some(environment);
        self.updated = Utc::now();
    }

    /// Add required tool
    pub fn add_required_tool(&mut self, tool: String) {
        if !self.required_tools.contains(&tool) {
            self.required_tools.push(tool);
            self.updated = Utc::now();
        }
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated = Utc::now();
        }
    }

    /// Check if procedure is executable
    pub fn is_executable(&self) -> bool {
        self.status == ProcedureStatus::Approved || self.status == ProcedureStatus::Active
    }

    /// Check if procedure is automated
    pub fn is_automated(&self) -> bool {
        matches!(self.execution_method, ExecutionMethod::Automated | ExecutionMethod::SemiAutomated)
    }

    /// Get total step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get automated step count
    pub fn automated_step_count(&self) -> usize {
        self.steps.iter().filter(|s| s.automation_script.is_some()).count()
    }

    /// Get automation coverage percentage
    pub fn automation_coverage(&self) -> f64 {
        if self.steps.is_empty() {
            0.0
        } else {
            (self.automated_step_count() as f64 / self.steps.len() as f64) * 100.0
        }
    }
}

impl Entity for TestProcedure {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Test procedure name cannot be empty".to_string(),
            ));
        }
        
        if self.description.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Test procedure description cannot be empty".to_string(),
            ));
        }

        if self.steps.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Test procedure must have at least one step".to_string(),
            ));
        }

        // Validate step numbers are sequential
        for (i, step) in self.steps.iter().enumerate() {
            if step.step_number != i + 1 {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Test procedure steps must be numbered sequentially".to_string(),
                ));
            }
        }

        // Validate each step
        for step in &self.steps {
            if step.description.trim().is_empty() {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Step description cannot be empty".to_string(),
                ));
            }
            
            if step.expected_result.trim().is_empty() {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Step expected result cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Test execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Blocked,
    Skipped,
    Cancelled,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Pending => write!(f, "Pending"),
            ExecutionStatus::Running => write!(f, "Running"),
            ExecutionStatus::Passed => write!(f, "Passed"),
            ExecutionStatus::Failed => write!(f, "Failed"),
            ExecutionStatus::Blocked => write!(f, "Blocked"),
            ExecutionStatus::Skipped => write!(f, "Skipped"),
            ExecutionStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Result of a single test step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_number: usize,
    pub status: ExecutionStatus,
    pub actual_result: Option<String>,
    pub duration_seconds: Option<u32>,
    pub error_message: Option<String>,
    pub evidence_files: Vec<String>,
    pub notes: Option<String>,
    pub executed_at: DateTime<Utc>,
}

impl StepResult {
    /// Create a new step result
    pub fn new(step_number: usize, status: ExecutionStatus) -> Self {
        Self {
            step_number,
            status,
            actual_result: None,
            duration_seconds: None,
            error_message: None,
            evidence_files: Vec::new(),
            notes: None,
            executed_at: Utc::now(),
        }
    }

    /// Set actual result
    pub fn set_actual_result(&mut self, result: String) {
        self.actual_result = Some(result);
    }

    /// Set duration
    pub fn set_duration(&mut self, seconds: u32) {
        self.duration_seconds = Some(seconds);
    }

    /// Set error message
    pub fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
    }

    /// Add evidence file
    pub fn add_evidence_file(&mut self, file_path: String) {
        self.evidence_files.push(file_path);
    }

    /// Set notes
    pub fn set_notes(&mut self, notes: String) {
        self.notes = Some(notes);
    }
}

/// Test execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution {
    pub id: Id,
    pub procedure_id: Id,
    pub execution_name: String,
    pub status: ExecutionStatus,
    
    // Execution details
    pub executor: Option<String>,
    pub environment: Option<String>,
    pub test_data_version: Option<String>,
    pub build_version: Option<String>,
    
    // Results
    pub step_results: Vec<StepResult>,
    pub overall_result: Option<String>,
    pub defects_found: Vec<String>,
    pub evidence_summary: Vec<String>,
    
    // Timing
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_duration_seconds: Option<u32>,
    
    // Integration
    pub verification_updated: bool,
    pub requirements_updated: bool,
    
    // Metadata
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl TestExecution {
    /// Create a new test execution
    pub fn new(procedure_id: Id, execution_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            procedure_id,
            execution_name,
            status: ExecutionStatus::Pending,
            executor: None,
            environment: None,
            test_data_version: None,
            build_version: None,
            step_results: Vec::new(),
            overall_result: None,
            defects_found: Vec::new(),
            evidence_summary: Vec::new(),
            started_at: None,
            completed_at: None,
            total_duration_seconds: None,
            verification_updated: false,
            requirements_updated: false,
            metadata: HashMap::new(),
            created: now,
            updated: now,
        }
    }

    /// Start execution
    pub fn start(&mut self, executor: Option<String>) {
        self.status = ExecutionStatus::Running;
        self.executor = executor;
        self.started_at = Some(Utc::now());
        self.updated = Utc::now();
    }

    /// Complete execution
    pub fn complete(&mut self, final_status: ExecutionStatus) {
        self.status = final_status;
        self.completed_at = Some(Utc::now());
        
        // Calculate total duration
        if let (Some(start), Some(end)) = (self.started_at, self.completed_at) {
            let duration = end - start;
            self.total_duration_seconds = Some(duration.num_seconds() as u32);
        }
        
        self.updated = Utc::now();
    }

    /// Add step result
    pub fn add_step_result(&mut self, result: StepResult) {
        self.step_results.push(result);
        self.step_results.sort_by_key(|r| r.step_number);
        self.updated = Utc::now();
    }

    /// Set overall result
    pub fn set_overall_result(&mut self, result: String) {
        self.overall_result = Some(result);
        self.updated = Utc::now();
    }

    /// Add defect found
    pub fn add_defect(&mut self, defect: String) {
        self.defects_found.push(defect);
        self.updated = Utc::now();
    }

    /// Mark verification as updated
    pub fn mark_verification_updated(&mut self) {
        self.verification_updated = true;
        self.updated = Utc::now();
    }

    /// Mark requirements as updated
    pub fn mark_requirements_updated(&mut self) {
        self.requirements_updated = true;
        self.updated = Utc::now();
    }

    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, ExecutionStatus::Passed | ExecutionStatus::Failed | ExecutionStatus::Cancelled)
    }

    /// Check if execution passed
    pub fn passed(&self) -> bool {
        self.status == ExecutionStatus::Passed
    }

    /// Get pass rate for steps
    pub fn step_pass_rate(&self) -> f64 {
        if self.step_results.is_empty() {
            0.0
        } else {
            let passed = self.step_results.iter().filter(|r| r.status == ExecutionStatus::Passed).count();
            (passed as f64 / self.step_results.len() as f64) * 100.0
        }
    }

    /// Get execution summary
    pub fn get_summary(&self) -> ExecutionSummary {
        let total_steps = self.step_results.len();
        let passed_steps = self.step_results.iter().filter(|r| r.status == ExecutionStatus::Passed).count();
        let failed_steps = self.step_results.iter().filter(|r| r.status == ExecutionStatus::Failed).count();
        let skipped_steps = self.step_results.iter().filter(|r| r.status == ExecutionStatus::Skipped).count();
        
        ExecutionSummary {
            execution_id: self.id,
            execution_name: self.execution_name.clone(),
            status: self.status,
            total_steps,
            passed_steps,
            failed_steps,
            skipped_steps,
            pass_rate: self.step_pass_rate(),
            duration_seconds: self.total_duration_seconds,
            defects_count: self.defects_found.len(),
            evidence_count: self.evidence_summary.len(),
        }
    }
}

impl Entity for TestExecution {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.execution_name
    }

    fn validate(&self) -> Result<()> {
        if self.execution_name.trim().is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Execution name cannot be empty".to_string(),
            ));
        }

        // Validate step results are sequential
        for (i, result) in self.step_results.iter().enumerate() {
            if result.step_number != i + 1 {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Step results must be numbered sequentially".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Summary of test execution
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub execution_id: Id,
    pub execution_name: String,
    pub status: ExecutionStatus,
    pub total_steps: usize,
    pub passed_steps: usize,
    pub failed_steps: usize,
    pub skipped_steps: usize,
    pub pass_rate: f64,
    pub duration_seconds: Option<u32>,
    pub defects_count: usize,
    pub evidence_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_procedure_creation() {
        let procedure = TestProcedure::new(
            "Test Procedure".to_string(),
            "A test procedure".to_string(),
            ProcedureType::Integration,
        );

        assert_eq!(procedure.name, "Test Procedure");
        assert_eq!(procedure.description, "A test procedure");
        assert_eq!(procedure.procedure_type, ProcedureType::Integration);
        assert_eq!(procedure.status, ProcedureStatus::Draft);
        assert_eq!(procedure.execution_method, ExecutionMethod::Manual);
    }

    #[test]
    fn test_test_step_creation() {
        let step = TestStep::new(
            1,
            "Click the button".to_string(),
            "Button should be clicked".to_string(),
        );

        assert_eq!(step.step_number, 1);
        assert_eq!(step.description, "Click the button");
        assert_eq!(step.expected_result, "Button should be clicked");
        assert!(step.automation_script.is_none());
    }

    #[test]
    fn test_test_execution_creation() {
        let procedure_id = Id::new();
        let execution = TestExecution::new(
            procedure_id,
            "Test Run 1".to_string(),
        );

        assert_eq!(execution.procedure_id, procedure_id);
        assert_eq!(execution.execution_name, "Test Run 1");
        assert_eq!(execution.status, ExecutionStatus::Pending);
        assert!(!execution.is_complete());
    }

    #[test]
    fn test_procedure_validation() {
        let mut procedure = TestProcedure::new(
            "Test Procedure".to_string(),
            "A test procedure".to_string(),
            ProcedureType::Integration,
        );

        // Should fail without steps
        assert!(procedure.validate().is_err());

        // Add a step
        let step = TestStep::new(1, "Test step".to_string(), "Expected result".to_string());
        procedure.add_step(step);

        // Should now pass
        assert!(procedure.validate().is_ok());
    }

    #[test]
    fn test_execution_summary() {
        let mut execution = TestExecution::new(
            Id::new(),
            "Test Execution".to_string(),
        );

        execution.add_step_result(StepResult::new(1, ExecutionStatus::Passed));
        execution.add_step_result(StepResult::new(2, ExecutionStatus::Failed));
        execution.add_step_result(StepResult::new(3, ExecutionStatus::Passed));

        let summary = execution.get_summary();
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.passed_steps, 2);
        assert_eq!(summary.failed_steps, 1);
        assert_eq!(summary.skipped_steps, 0);
        assert!((summary.pass_rate - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_automation_coverage() {
        let mut procedure = TestProcedure::new(
            "Test Procedure".to_string(),
            "A test procedure".to_string(),
            ProcedureType::Unit,
        );

        let mut step1 = TestStep::new(1, "Manual step".to_string(), "Result".to_string());
        let mut step2 = TestStep::new(2, "Automated step".to_string(), "Result".to_string());
        step2.set_automation_script("run_test.sh".to_string());

        procedure.add_step(step1);
        procedure.add_step(step2);

        assert_eq!(procedure.step_count(), 2);
        assert_eq!(procedure.automated_step_count(), 1);
        assert_eq!(procedure.automation_coverage(), 50.0);
    }
}