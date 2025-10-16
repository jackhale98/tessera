use std::sync::Arc;
use uuid::Uuid;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{
    EntityType, EntityMetadata, Task, Requirement, Risk, Hazard, RiskControl,
    Milestone, Resource, TaskType, ResourceType, Calendar, Baseline,
    Assembly, Component, Feature, FeatureType, DistributionType,
    Mate, MateType, Stackup, AnalysisType, Supplier, Quote, CostDistribution,
    Verification, Validation, TestStatus, TestPriority, TestStep,
    Manufacturing, ProcessStatus, WorkInstructionStep,
};
use chrono::{Utc, NaiveDate};

// Import specialized managers
use crate::core::managers::{
    TaskManager, RequirementManager, RiskManager, DesignManager,
    TestingManager, ManufacturingManager,
};

/// Facade for entity lifecycle (CRUD operations)
/// Delegates to specialized managers for better modularity
pub struct EntityManager {
    task_manager: TaskManager,
    requirement_manager: RequirementManager,
    risk_manager: RiskManager,
    design_manager: DesignManager,
    testing_manager: TestingManager,
    manufacturing_manager: ManufacturingManager,
}

impl EntityManager {
    pub fn new(storage: Arc<RonStorage>) -> Self {
        Self {
            task_manager: TaskManager::new(Arc::clone(&storage)),
            requirement_manager: RequirementManager::new(Arc::clone(&storage)),
            risk_manager: RiskManager::new(Arc::clone(&storage)),
            design_manager: DesignManager::new(Arc::clone(&storage)),
            testing_manager: TestingManager::new(Arc::clone(&storage)),
            manufacturing_manager: ManufacturingManager::new(storage),
        }
    }

    // ============================================================================
    // Task Methods (delegate to TaskManager)
    // ============================================================================

    pub fn create_task(
        &self,
        name: String,
        description: String,
        scheduled_start: chrono::DateTime<Utc>,
        deadline: chrono::DateTime<Utc>,
        task_type: TaskType,
    ) -> EdtResult<Task> {
        self.task_manager.create_task(name, description, scheduled_start, deadline, task_type)
    }

    pub fn get_task(&self, id: &Uuid) -> EdtResult<Task> {
        self.task_manager.get_task(id)
    }

    pub fn update_task(&self, task: Task) -> EdtResult<Task> {
        self.task_manager.update_task(task)
    }

    pub fn delete_task(&self, id: &Uuid) -> EdtResult<()> {
        self.task_manager.delete_task(id)
    }

    pub fn list_task_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.task_manager.list_task_ids()
    }

    // ============================================================================
    // Milestone Methods (delegate to TaskManager)
    // ============================================================================

    pub fn create_milestone(
        &self,
        name: String,
        description: String,
        date: chrono::DateTime<Utc>,
    ) -> EdtResult<Milestone> {
        self.task_manager.create_milestone(name, description, date)
    }

    pub fn get_milestone(&self, id: &Uuid) -> EdtResult<Milestone> {
        self.task_manager.get_milestone(id)
    }

    pub fn update_milestone(&self, milestone: Milestone) -> EdtResult<Milestone> {
        self.task_manager.update_milestone(milestone)
    }

    pub fn delete_milestone(&self, id: &Uuid) -> EdtResult<()> {
        self.task_manager.delete_milestone(id)
    }

    // ============================================================================
    // Resource Methods (delegate to TaskManager)
    // ============================================================================

    pub fn create_resource(
        &self,
        name: String,
        description: String,
        resource_type: ResourceType,
    ) -> EdtResult<Resource> {
        self.task_manager.create_resource(name, description, resource_type)
    }

    pub fn get_resource(&self, id: &Uuid) -> EdtResult<Resource> {
        self.task_manager.get_resource(id)
    }

    pub fn update_resource(&self, resource: Resource) -> EdtResult<Resource> {
        self.task_manager.update_resource(resource)
    }

    pub fn delete_resource(&self, id: &Uuid) -> EdtResult<()> {
        self.task_manager.delete_resource(id)
    }

    // ============================================================================
    // Calendar Methods (delegate to TaskManager)
    // ============================================================================

    pub fn create_calendar(
        &self,
        name: String,
        work_hours_per_day: f64,
        work_days: Vec<chrono::Weekday>,
    ) -> EdtResult<Calendar> {
        self.task_manager.create_calendar(name, work_hours_per_day, work_days)
    }

    pub fn get_calendar(&self, id: &Uuid) -> EdtResult<Calendar> {
        self.task_manager.get_calendar(id)
    }

    pub fn update_calendar(&self, calendar: Calendar) -> EdtResult<Calendar> {
        self.task_manager.update_calendar(calendar)
    }

    pub fn delete_calendar(&self, id: &Uuid) -> EdtResult<()> {
        self.task_manager.delete_calendar(id)
    }

    // ============================================================================
    // Baseline Methods (delegate to TaskManager)
    // ============================================================================

    pub fn create_baseline(
        &self,
        name: String,
        description: String,
        task_ids: Vec<Uuid>,
    ) -> EdtResult<Baseline> {
        self.task_manager.create_baseline(name, description, task_ids)
    }

    pub fn get_baseline(&self, id: &Uuid) -> EdtResult<Baseline> {
        self.task_manager.get_baseline(id)
    }

    pub fn update_baseline(&self, baseline: Baseline) -> EdtResult<Baseline> {
        self.task_manager.update_baseline(baseline)
    }

    pub fn delete_baseline(&self, id: &Uuid) -> EdtResult<()> {
        self.task_manager.delete_baseline(id)
    }

    // ============================================================================
    // Requirement Methods (delegate to RequirementManager)
    // ============================================================================

    pub fn create_requirement(
        &self,
        name: String,
        description: String,
        requirement_type: String,
    ) -> EdtResult<Requirement> {
        self.requirement_manager.create_requirement(name, description, requirement_type)
    }

    pub fn get_requirement(&self, id: &Uuid) -> EdtResult<Requirement> {
        self.requirement_manager.get_requirement(id)
    }

    pub fn update_requirement(&self, requirement: Requirement) -> EdtResult<Requirement> {
        self.requirement_manager.update_requirement(requirement)
    }

    pub fn delete_requirement(&self, id: &Uuid) -> EdtResult<()> {
        self.requirement_manager.delete_requirement(id)
    }

    pub fn list_requirement_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.requirement_manager.list_requirement_ids()
    }

    // ============================================================================
    // Risk Methods (delegate to RiskManager)
    // ============================================================================

    pub fn create_risk(
        &self,
        name: String,
        description: String,
        risk_type: String,
        probability: u32,
        severity: u32,
    ) -> EdtResult<Risk> {
        self.risk_manager.create_risk(name, description, risk_type, probability, severity)
    }

    pub fn get_risk(&self, id: &Uuid) -> EdtResult<Risk> {
        self.risk_manager.get_risk(id)
    }

    pub fn update_risk(&self, risk: Risk) -> EdtResult<Risk> {
        self.risk_manager.update_risk(risk)
    }

    pub fn delete_risk(&self, id: &Uuid) -> EdtResult<()> {
        self.risk_manager.delete_risk(id)
    }

    pub fn list_risk_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.risk_manager.list_risk_ids()
    }

    // ============================================================================
    // Hazard Methods (delegate to RiskManager)
    // ============================================================================

    pub fn create_hazard(
        &self,
        name: String,
        description: String,
        causes: Vec<String>,
        harms: Vec<String>,
    ) -> EdtResult<Hazard> {
        self.risk_manager.create_hazard(name, description, causes, harms)
    }

    pub fn get_hazard(&self, id: &Uuid) -> EdtResult<Hazard> {
        self.risk_manager.get_hazard(id)
    }

    pub fn update_hazard(&self, hazard: Hazard) -> EdtResult<Hazard> {
        self.risk_manager.update_hazard(hazard)
    }

    pub fn delete_hazard(&self, id: &Uuid) -> EdtResult<()> {
        self.risk_manager.delete_hazard(id)
    }

    pub fn list_hazard_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.risk_manager.list_hazard_ids()
    }

    // ============================================================================
    // RiskControl Methods (delegate to RiskManager)
    // ============================================================================

    pub fn create_risk_control(
        &self,
        name: String,
        description: String,
        control_type: String,
    ) -> EdtResult<RiskControl> {
        self.risk_manager.create_risk_control(name, description, control_type)
    }

    pub fn get_risk_control(&self, id: &Uuid) -> EdtResult<RiskControl> {
        self.risk_manager.get_risk_control(id)
    }

    pub fn update_risk_control(&self, control: RiskControl) -> EdtResult<RiskControl> {
        self.risk_manager.update_risk_control(control)
    }

    pub fn delete_risk_control(&self, id: &Uuid) -> EdtResult<()> {
        self.risk_manager.delete_risk_control(id)
    }

    pub fn list_risk_control_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.risk_manager.list_risk_control_ids()
    }

    // ============================================================================
    // Design Entity Methods (delegate to DesignManager)
    // ============================================================================

    // Assembly
    pub fn create_assembly(
        &self,
        name: String,
        description: String,
        revision: String,
    ) -> EdtResult<Assembly> {
        self.design_manager.create_assembly(name, description, revision)
    }

    pub fn get_assembly(&self, id: &Uuid) -> EdtResult<Assembly> {
        self.design_manager.get_assembly(id)
    }

    pub fn update_assembly(&self, assembly: Assembly) -> EdtResult<Assembly> {
        self.design_manager.update_assembly(assembly)
    }

    pub fn delete_assembly(&self, id: &Uuid) -> EdtResult<()> {
        self.design_manager.delete_assembly(id)
    }

    pub fn list_assembly_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.design_manager.list_assembly_ids()
    }

    // Component
    pub fn create_component(
        &self,
        name: String,
        description: String,
        revision: String,
    ) -> EdtResult<Component> {
        self.design_manager.create_component(name, description, revision)
    }

    pub fn get_component(&self, id: &Uuid) -> EdtResult<Component> {
        self.design_manager.get_component(id)
    }

    pub fn update_component(&self, component: Component) -> EdtResult<Component> {
        self.design_manager.update_component(component)
    }

    pub fn delete_component(&self, id: &Uuid) -> EdtResult<()> {
        self.design_manager.delete_component(id)
    }

    pub fn list_component_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.design_manager.list_component_ids()
    }

    // Feature
    pub fn create_feature(
        &self,
        name: String,
        description: String,
        feature_type: FeatureType,
        nominal: f64,
        upper_tolerance: f64,
        lower_tolerance: f64,
        distribution_type: DistributionType,
    ) -> EdtResult<Feature> {
        self.design_manager.create_feature(
            name,
            description,
            feature_type,
            nominal,
            upper_tolerance,
            lower_tolerance,
            distribution_type,
        )
    }

    pub fn get_feature(&self, id: &Uuid) -> EdtResult<Feature> {
        self.design_manager.get_feature(id)
    }

    pub fn update_feature(&self, feature: Feature) -> EdtResult<Feature> {
        self.design_manager.update_feature(feature)
    }

    pub fn delete_feature(&self, id: &Uuid) -> EdtResult<()> {
        self.design_manager.delete_feature(id)
    }

    pub fn list_feature_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.design_manager.list_feature_ids()
    }

    // Mate
    pub fn create_mate(
        &self,
        name: String,
        description: String,
        mate_type: MateType,
    ) -> EdtResult<Mate> {
        self.design_manager.create_mate(name, description, mate_type)
    }

    pub fn get_mate(&self, id: &Uuid) -> EdtResult<Mate> {
        self.design_manager.get_mate(id)
    }

    pub fn update_mate(&self, mate: Mate) -> EdtResult<Mate> {
        self.design_manager.update_mate(mate)
    }

    pub fn delete_mate(&self, id: &Uuid) -> EdtResult<()> {
        self.design_manager.delete_mate(id)
    }

    pub fn list_mate_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.design_manager.list_mate_ids()
    }

    // Stackup
    pub fn create_stackup(
        &self,
        name: String,
        description: String,
        analysis_types: Vec<AnalysisType>,
    ) -> EdtResult<Stackup> {
        self.design_manager.create_stackup(name, description, analysis_types)
    }

    pub fn get_stackup(&self, id: &Uuid) -> EdtResult<Stackup> {
        self.design_manager.get_stackup(id)
    }

    pub fn update_stackup(&self, stackup: Stackup) -> EdtResult<Stackup> {
        self.design_manager.update_stackup(stackup)
    }

    pub fn delete_stackup(&self, id: &Uuid) -> EdtResult<()> {
        self.design_manager.delete_stackup(id)
    }

    pub fn list_stackup_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.design_manager.list_stackup_ids()
    }

    // Supplier
    pub fn create_supplier(
        &self,
        name: String,
        description: String,
    ) -> EdtResult<Supplier> {
        self.design_manager.create_supplier(name, description)
    }

    pub fn get_supplier(&self, id: &Uuid) -> EdtResult<Supplier> {
        self.design_manager.get_supplier(id)
    }

    pub fn update_supplier(&self, supplier: Supplier) -> EdtResult<Supplier> {
        self.design_manager.update_supplier(supplier)
    }

    pub fn delete_supplier(&self, id: &Uuid) -> EdtResult<()> {
        self.design_manager.delete_supplier(id)
    }

    pub fn list_supplier_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.design_manager.list_supplier_ids()
    }

    // Quote
    pub fn create_quote(
        &self,
        quote_number: String,
        quote_date: NaiveDate,
        quantity_price_pairs: Vec<(u32, f64)>,
        distribution_type: CostDistribution,
    ) -> EdtResult<Quote> {
        self.design_manager.create_quote(quote_number, quote_date, quantity_price_pairs, distribution_type)
    }

    pub fn get_quote(&self, id: &Uuid) -> EdtResult<Quote> {
        self.design_manager.get_quote(id)
    }

    pub fn update_quote(&self, quote: Quote) -> EdtResult<Quote> {
        self.design_manager.update_quote(quote)
    }

    pub fn delete_quote(&self, id: &Uuid) -> EdtResult<()> {
        self.design_manager.delete_quote(id)
    }

    pub fn list_quote_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.design_manager.list_quote_ids()
    }

    // ============================================================================
    // Verification Methods (delegate to TestingManager)
    // ============================================================================

    pub fn create_verification(
        &self,
        name: String,
        description: String,
        test_type: String,
        test_steps: Vec<TestStep>,
        acceptance_criteria: Vec<String>,
        priority: TestPriority,
    ) -> EdtResult<Verification> {
        self.testing_manager.create_verification(
            name,
            description,
            test_type,
            test_steps,
            acceptance_criteria,
            priority,
        )
    }

    pub fn get_verification(&self, id: &Uuid) -> EdtResult<Verification> {
        self.testing_manager.get_verification(id)
    }

    pub fn update_verification(&self, verification: Verification) -> EdtResult<Verification> {
        self.testing_manager.update_verification(verification)
    }

    pub fn delete_verification(&self, id: &Uuid) -> EdtResult<()> {
        self.testing_manager.delete_verification(id)
    }

    pub fn list_verification_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.testing_manager.list_verification_ids()
    }

    // ============================================================================
    // Validation Methods (delegate to TestingManager)
    // ============================================================================

    pub fn create_validation(
        &self,
        name: String,
        description: String,
        validation_type: String,
        participants: Vec<String>,
        success_criteria: Vec<String>,
        priority: TestPriority,
    ) -> EdtResult<Validation> {
        self.testing_manager.create_validation(
            name,
            description,
            validation_type,
            participants,
            success_criteria,
            priority,
        )
    }

    pub fn get_validation(&self, id: &Uuid) -> EdtResult<Validation> {
        self.testing_manager.get_validation(id)
    }

    pub fn update_validation(&self, validation: Validation) -> EdtResult<Validation> {
        self.testing_manager.update_validation(validation)
    }

    pub fn delete_validation(&self, id: &Uuid) -> EdtResult<()> {
        self.testing_manager.delete_validation(id)
    }

    pub fn list_validation_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.testing_manager.list_validation_ids()
    }

    // ============================================================================
    // Manufacturing Methods (delegate to ManufacturingManager)
    // ============================================================================

    pub fn create_manufacturing(
        &self,
        name: String,
        description: String,
        process_type: String,
        work_instructions: Vec<WorkInstructionStep>,
        priority: u32,
    ) -> EdtResult<Manufacturing> {
        self.manufacturing_manager.create_manufacturing(
            name,
            description,
            process_type,
            work_instructions,
            priority,
        )
    }

    pub fn get_manufacturing(&self, id: &Uuid) -> EdtResult<Manufacturing> {
        self.manufacturing_manager.get_manufacturing(id)
    }

    pub fn update_manufacturing(&self, manufacturing: Manufacturing) -> EdtResult<Manufacturing> {
        self.manufacturing_manager.update_manufacturing(manufacturing)
    }

    pub fn delete_manufacturing(&self, id: &Uuid) -> EdtResult<()> {
        self.manufacturing_manager.delete_manufacturing(id)
    }

    pub fn list_manufacturing_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.manufacturing_manager.list_manufacturing_ids()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::Duration;

    fn create_test_manager() -> (TempDir, EntityManager) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(RonStorage::new(temp_dir.path()).unwrap());
        let manager = EntityManager::new(storage);
        (temp_dir, manager)
    }

    #[test]
    fn test_create_task() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();
        let end = start + Duration::days(5);

        let task = manager
            .create_task(
                "Test Task".to_string(),
                "A test task".to_string(),
                start,
                end,
                TaskType::EffortDriven,
            )
            .unwrap();

        assert_eq!(task.name, "Test Task");
        assert_eq!(task.task_type, TaskType::EffortDriven);
        assert_eq!(task.percent_complete, 0.0);
    }

    #[test]
    fn test_create_task_validation_empty_name() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();
        let end = start + Duration::days(5);

        let result = manager.create_task(
            "  ".to_string(),
            "Description".to_string(),
            start,
            end,
            TaskType::EffortDriven,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_create_task_validation_invalid_dates() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();
        let end = start - Duration::days(1); // End before start

        let result = manager.create_task(
            "Test".to_string(),
            "Description".to_string(),
            start,
            end,
            TaskType::EffortDriven,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_task() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();
        let end = start + Duration::days(5);

        let task = manager
            .create_task(
                "Test Task".to_string(),
                "Description".to_string(),
                start,
                end,
                TaskType::EffortDriven,
            )
            .unwrap();

        let retrieved = manager.get_task(&task.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Test Task");
        assert_eq!(retrieved.metadata.id, task.metadata.id);
    }

    #[test]
    fn test_get_task_not_found() {
        let (_temp, manager) = create_test_manager();

        let fake_id = Uuid::new_v4();
        let result = manager.get_task(&fake_id);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::EntityNotFound(_)));
    }

    #[test]
    fn test_update_task() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();
        let end = start + Duration::days(5);

        let mut task = manager
            .create_task(
                "Original Name".to_string(),
                "Description".to_string(),
                start,
                end,
                TaskType::EffortDriven,
            )
            .unwrap();

        task.name = "Updated Name".to_string();
        task.percent_complete = 0.5;

        let updated = manager.update_task(task).unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.percent_complete, 0.5);

        // Verify it was persisted
        let retrieved = manager.get_task(&updated.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Updated Name");
    }

    #[test]
    fn test_delete_task() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();
        let end = start + Duration::days(5);

        let task = manager
            .create_task(
                "To Delete".to_string(),
                "Description".to_string(),
                start,
                end,
                TaskType::EffortDriven,
            )
            .unwrap();

        let task_id = task.metadata.id;

        manager.delete_task(&task_id).unwrap();

        let result = manager.get_task(&task_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_task_ids() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();
        let end = start + Duration::days(5);

        for i in 0..3 {
            manager
                .create_task(
                    format!("Task {}", i),
                    "Description".to_string(),
                    start,
                    end,
                    TaskType::EffortDriven,
                )
                .unwrap();
        }

        let ids = manager.list_task_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_create_requirement() {
        let (_temp, manager) = create_test_manager();

        let req = manager
            .create_requirement(
                "REQ-001".to_string(),
                "Test requirement".to_string(),
                "System Requirement".to_string(),
            )
            .unwrap();

        assert_eq!(req.name, "REQ-001");
        assert_eq!(req.requirement_type, "System Requirement");
    }

    #[test]
    fn test_get_and_update_requirement() {
        let (_temp, manager) = create_test_manager();

        let req = manager
            .create_requirement(
                "REQ-001".to_string(),
                "Test requirement".to_string(),
                "System Requirement".to_string(),
            )
            .unwrap();

        let retrieved = manager.get_requirement(&req.metadata.id).unwrap();
        assert_eq!(retrieved.name, "REQ-001");

        let mut updated = retrieved;
        updated.notes = Some("Added notes".to_string());

        let saved = manager.update_requirement(updated).unwrap();
        assert_eq!(saved.notes, Some("Added notes".to_string()));
    }

    #[test]
    fn test_create_risk() {
        let (_temp, manager) = create_test_manager();

        let risk = manager
            .create_risk(
                "RISK-001".to_string(),
                "Test risk".to_string(),
                "Safety Risk".to_string(),
                3,
                4,
            )
            .unwrap();

        assert_eq!(risk.name, "RISK-001");
        assert_eq!(risk.probability, 3);
        assert_eq!(risk.severity, 4);
        assert_eq!(risk.risk_score, 12); // 3 * 4
    }

    #[test]
    fn test_create_milestone() {
        let (_temp, manager) = create_test_manager();

        let date = Utc::now();
        let milestone = manager
            .create_milestone(
                "M1: Kickoff".to_string(),
                "Project start".to_string(),
                date,
            )
            .unwrap();

        assert_eq!(milestone.name, "M1: Kickoff");
    }

    #[test]
    fn test_create_resource() {
        let (_temp, manager) = create_test_manager();

        let resource = manager
            .create_resource(
                "John Doe".to_string(),
                "Engineer".to_string(),
                ResourceType::Labor,
            )
            .unwrap();

        assert_eq!(resource.name, "John Doe");
        assert_eq!(resource.resource_type, ResourceType::Labor);
    }

    #[test]
    fn test_create_calendar() {
        use chrono::Weekday;

        let (_temp, manager) = create_test_manager();

        let calendar = manager
            .create_calendar(
                "Standard Week".to_string(),
                8.0,
                vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri],
            )
            .unwrap();

        assert_eq!(calendar.name, "Standard Week");
        assert_eq!(calendar.work_hours_per_day, 8.0);
        assert_eq!(calendar.work_days.len(), 5);
    }

    #[test]
    fn test_create_calendar_validation() {
        use chrono::Weekday;

        let (_temp, manager) = create_test_manager();

        // Empty name
        let result1 = manager.create_calendar(
            "  ".to_string(),
            8.0,
            vec![Weekday::Mon],
        );
        assert!(result1.is_err());

        // Invalid hours (too high)
        let result2 = manager.create_calendar(
            "Test".to_string(),
            25.0,
            vec![Weekday::Mon],
        );
        assert!(result2.is_err());

        // No work days
        let result3 = manager.create_calendar(
            "Test".to_string(),
            8.0,
            vec![],
        );
        assert!(result3.is_err());
    }

    #[test]
    fn test_get_and_update_calendar() {
        use chrono::Weekday;

        let (_temp, manager) = create_test_manager();

        let calendar = manager
            .create_calendar(
                "Standard Week".to_string(),
                8.0,
                vec![Weekday::Mon, Weekday::Tue],
            )
            .unwrap();

        let retrieved = manager.get_calendar(&calendar.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Standard Week");

        let mut updated = retrieved;
        updated.work_hours_per_day = 7.5;

        let saved = manager.update_calendar(updated).unwrap();
        assert_eq!(saved.work_hours_per_day, 7.5);
    }

    #[test]
    fn test_delete_calendar() {
        use chrono::Weekday;

        let (_temp, manager) = create_test_manager();

        let calendar = manager
            .create_calendar(
                "To Delete".to_string(),
                8.0,
                vec![Weekday::Mon],
            )
            .unwrap();

        let id = calendar.metadata.id;
        manager.delete_calendar(&id).unwrap();

        let result = manager.get_calendar(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_baseline() {
        let (_temp, manager) = create_test_manager();

        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();

        let baseline = manager
            .create_baseline(
                "Q1 Baseline".to_string(),
                "First quarter baseline".to_string(),
                vec![task1_id, task2_id],
            )
            .unwrap();

        assert_eq!(baseline.name, "Q1 Baseline");
        assert_eq!(baseline.task_ids.len(), 2);
        assert!(baseline.task_ids.contains(&task1_id));
    }

    #[test]
    fn test_create_baseline_validation() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_baseline(
            "  ".to_string(),
            "Description".to_string(),
            vec![],
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_baseline() {
        let (_temp, manager) = create_test_manager();

        let task_id = Uuid::new_v4();

        let baseline = manager
            .create_baseline(
                "Q1 Baseline".to_string(),
                "Description".to_string(),
                vec![task_id],
            )
            .unwrap();

        let retrieved = manager.get_baseline(&baseline.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Q1 Baseline");

        let mut updated = retrieved;
        updated.description = "Updated description".to_string();

        let saved = manager.update_baseline(updated).unwrap();
        assert_eq!(saved.description, "Updated description");
    }

    #[test]
    fn test_delete_baseline() {
        let (_temp, manager) = create_test_manager();

        let baseline = manager
            .create_baseline(
                "To Delete".to_string(),
                "Description".to_string(),
                vec![],
            )
            .unwrap();

        let id = baseline.metadata.id;
        manager.delete_baseline(&id).unwrap();

        let result = manager.get_baseline(&id);
        assert!(result.is_err());
    }

    // Integration Tests

    #[test]
    fn test_integration_resource_with_calendar() {
        use chrono::Weekday;

        let (_temp, manager) = create_test_manager();

        // Create a calendar
        let calendar = manager
            .create_calendar(
                "Engineering Team Calendar".to_string(),
                8.0,
                vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri],
            )
            .unwrap();

        // Create a resource and assign the calendar
        let mut resource = manager
            .create_resource(
                "Senior Engineer".to_string(),
                "Full-time engineer".to_string(),
                ResourceType::Labor,
            )
            .unwrap();

        resource.calendar_id = Some(calendar.metadata.id);
        resource.bill_rate = Some(150.0);

        let updated_resource = manager.update_resource(resource).unwrap();

        // Verify the resource is linked to the calendar
        assert_eq!(updated_resource.calendar_id, Some(calendar.metadata.id));
        assert_eq!(updated_resource.bill_rate, Some(150.0));

        // Verify we can retrieve both
        let retrieved_calendar = manager.get_calendar(&calendar.metadata.id).unwrap();
        assert_eq!(retrieved_calendar.work_hours_per_day, 8.0);
    }

    #[test]
    fn test_integration_baseline_workflow() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();

        // Create multiple tasks
        let task1 = manager
            .create_task(
                "Design Phase".to_string(),
                "Initial design".to_string(),
                start,
                start + Duration::days(10),
                TaskType::EffortDriven,
            )
            .unwrap();

        let task2 = manager
            .create_task(
                "Implementation".to_string(),
                "Code implementation".to_string(),
                start + Duration::days(10),
                start + Duration::days(25),
                TaskType::EffortDriven,
            )
            .unwrap();

        let task3 = manager
            .create_task(
                "Testing".to_string(),
                "QA testing".to_string(),
                start + Duration::days(25),
                start + Duration::days(30),
                TaskType::EffortDriven,
            )
            .unwrap();

        // Create a baseline capturing all tasks
        let baseline = manager
            .create_baseline(
                "Q1 2025 Baseline".to_string(),
                "Initial project plan baseline".to_string(),
                vec![task1.metadata.id, task2.metadata.id, task3.metadata.id],
            )
            .unwrap();

        // Verify baseline contains all task IDs
        assert_eq!(baseline.task_ids.len(), 3);
        assert!(baseline.task_ids.contains(&task1.metadata.id));
        assert!(baseline.task_ids.contains(&task2.metadata.id));
        assert!(baseline.task_ids.contains(&task3.metadata.id));

        // Simulate updating task progress
        let mut updated_task1 = manager.get_task(&task1.metadata.id).unwrap();
        updated_task1.percent_complete = 0.75;
        manager.update_task(updated_task1).unwrap();

        // Baseline remains unchanged
        let retrieved_baseline = manager.get_baseline(&baseline.metadata.id).unwrap();
        assert_eq!(retrieved_baseline.task_ids.len(), 3);
    }

    #[test]
    fn test_integration_milestone_with_dependencies() {
        let (_temp, manager) = create_test_manager();

        let start = Utc::now();

        // Create tasks
        let task1 = manager
            .create_task(
                "Preparation".to_string(),
                "Prep work".to_string(),
                start,
                start + Duration::days(5),
                TaskType::EffortDriven,
            )
            .unwrap();

        // Create milestone that depends on task completion
        let mut milestone = manager
            .create_milestone(
                "Phase 1 Complete".to_string(),
                "End of first phase".to_string(),
                start + Duration::days(5),
            )
            .unwrap();

        // Add dependency
        milestone.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task1.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });

        let updated_milestone = manager.update_milestone(milestone).unwrap();

        // Verify dependency
        assert_eq!(updated_milestone.dependencies.len(), 1);
        assert_eq!(
            updated_milestone.dependencies[0].predecessor_id,
            task1.metadata.id
        );
    }

    #[test]
    fn test_integration_complete_project_setup() {
        use chrono::Weekday;

        let (_temp, manager) = create_test_manager();

        // 1. Create calendar
        let calendar = manager
            .create_calendar(
                "Standard Week".to_string(),
                8.0,
                vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri],
            )
            .unwrap();

        // 2. Create resources
        let mut engineer = manager
            .create_resource(
                "Alice Engineer".to_string(),
                "Senior Software Engineer".to_string(),
                ResourceType::Labor,
            )
            .unwrap();
        engineer.calendar_id = Some(calendar.metadata.id);
        engineer.bill_rate = Some(150.0);
        engineer.email = Some("alice@example.com".to_string());
        let engineer = manager.update_resource(engineer).unwrap();

        let mut contractor = manager
            .create_resource(
                "Bob Contractor".to_string(),
                "External consultant".to_string(),
                ResourceType::FlatCost,
            )
            .unwrap();
        contractor.bill_rate = Some(200.0);
        let contractor = manager.update_resource(contractor).unwrap();

        // 3. Create tasks
        let start = Utc::now();

        let mut task1 = manager
            .create_task(
                "Architecture Design".to_string(),
                "Design system architecture".to_string(),
                start,
                start + Duration::days(7),
                TaskType::EffortDriven,
            )
            .unwrap();
        task1.assigned_resources.push(crate::models::ResourceAssignment {
            resource_id: engineer.metadata.id,
            allocated_hours: 40.0,
        });
        task1.estimated_effort = Some(40.0);
        let task1 = manager.update_task(task1).unwrap();

        let mut task2 = manager
            .create_task(
                "Implementation".to_string(),
                "Implement the design".to_string(),
                start + Duration::days(7),
                start + Duration::days(21),
                TaskType::EffortDriven,
            )
            .unwrap();
        task2.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task1.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });
        task2.assigned_resources.push(crate::models::ResourceAssignment {
            resource_id: engineer.metadata.id,
            allocated_hours: 80.0,
        });
        task2.assigned_resources.push(crate::models::ResourceAssignment {
            resource_id: contractor.metadata.id,
            allocated_hours: 40.0,
        });
        task2.estimated_effort = Some(120.0);
        let task2 = manager.update_task(task2).unwrap();

        // 4. Create milestone
        let milestone = manager
            .create_milestone(
                "Development Complete".to_string(),
                "All development work finished".to_string(),
                start + Duration::days(21),
            )
            .unwrap();

        // 5. Create baseline
        let baseline = manager
            .create_baseline(
                "Project Plan v1.0".to_string(),
                "Initial approved project plan".to_string(),
                vec![task1.metadata.id, task2.metadata.id],
            )
            .unwrap();

        // Verify complete setup
        assert_eq!(calendar.work_hours_per_day, 8.0);
        assert_eq!(engineer.bill_rate, Some(150.0));
        assert_eq!(contractor.bill_rate, Some(200.0));
        assert_eq!(task1.assigned_resources.len(), 1);
        assert_eq!(task2.assigned_resources.len(), 2);
        assert_eq!(task2.dependencies.len(), 1);
        assert_eq!(milestone.name, "Development Complete");
        assert_eq!(baseline.task_ids.len(), 2);

        // Verify all entities are persisted and retrievable
        let _ = manager.get_calendar(&calendar.metadata.id).unwrap();
        let _ = manager.get_resource(&engineer.metadata.id).unwrap();
        let _ = manager.get_resource(&contractor.metadata.id).unwrap();
        let _ = manager.get_task(&task1.metadata.id).unwrap();
        let _ = manager.get_task(&task2.metadata.id).unwrap();
        let _ = manager.get_milestone(&milestone.metadata.id).unwrap();
        let _ = manager.get_baseline(&baseline.metadata.id).unwrap();
    }

    // ============================================================================
    // Design Entity Tests
    // ============================================================================

    #[test]
    fn test_create_assembly() {
        let (_temp, manager) = create_test_manager();

        let assembly = manager
            .create_assembly(
                "Main Assembly".to_string(),
                "Top-level product assembly".to_string(),
                "Rev A".to_string(),
            )
            .unwrap();

        assert_eq!(assembly.name, "Main Assembly");
        assert_eq!(assembly.revision, "Rev A");
        assert_eq!(assembly.metadata.entity_type, EntityType::Assembly);
    }

    #[test]
    fn test_create_assembly_validation() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_assembly(
            "  ".to_string(),
            "Description".to_string(),
            "Rev A".to_string(),
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_assembly() {
        let (_temp, manager) = create_test_manager();

        let assembly = manager
            .create_assembly(
                "Test Assembly".to_string(),
                "Description".to_string(),
                "Rev A".to_string(),
            )
            .unwrap();

        let retrieved = manager.get_assembly(&assembly.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Test Assembly");

        let mut updated = retrieved;
        updated.revision = "Rev B".to_string();
        updated.notes = Some("Updated to revision B".to_string());

        let saved = manager.update_assembly(updated).unwrap();
        assert_eq!(saved.revision, "Rev B");
        assert_eq!(saved.notes, Some("Updated to revision B".to_string()));
    }

    #[test]
    fn test_delete_assembly() {
        let (_temp, manager) = create_test_manager();

        let assembly = manager
            .create_assembly(
                "To Delete".to_string(),
                "Description".to_string(),
                "Rev A".to_string(),
            )
            .unwrap();

        let id = assembly.metadata.id;
        manager.delete_assembly(&id).unwrap();

        let result = manager.get_assembly(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_assembly_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..3 {
            manager
                .create_assembly(
                    format!("Assembly {}", i),
                    "Description".to_string(),
                    "Rev A".to_string(),
                )
                .unwrap();
        }

        let ids = manager.list_assembly_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_create_component() {
        let (_temp, manager) = create_test_manager();

        let component = manager
            .create_component(
                "Bracket".to_string(),
                "Mounting bracket".to_string(),
                "Rev 1".to_string(),
            )
            .unwrap();

        assert_eq!(component.name, "Bracket");
        assert_eq!(component.metadata.entity_type, EntityType::Component);
    }

    #[test]
    fn test_create_component_validation() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_component(
            "".to_string(),
            "Description".to_string(),
            "Rev 1".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_and_update_component() {
        let (_temp, manager) = create_test_manager();

        let component = manager
            .create_component(
                "Bracket".to_string(),
                "Mounting bracket".to_string(),
                "Rev 1".to_string(),
            )
            .unwrap();

        let retrieved = manager.get_component(&component.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Bracket");

        let mut updated = retrieved;
        updated.part_number = Some("BRK-001".to_string());
        updated.material = Some("Aluminum 6061-T6".to_string());
        updated.mass = Some(0.125);

        let saved = manager.update_component(updated).unwrap();
        assert_eq!(saved.part_number, Some("BRK-001".to_string()));
        assert_eq!(saved.material, Some("Aluminum 6061-T6".to_string()));
        assert_eq!(saved.mass, Some(0.125));
    }

    #[test]
    fn test_delete_component() {
        let (_temp, manager) = create_test_manager();

        let component = manager
            .create_component(
                "To Delete".to_string(),
                "Description".to_string(),
                "Rev 1".to_string(),
            )
            .unwrap();

        let id = component.metadata.id;
        manager.delete_component(&id).unwrap();

        let result = manager.get_component(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_component_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..4 {
            manager
                .create_component(
                    format!("Component {}", i),
                    "Description".to_string(),
                    "Rev 1".to_string(),
                )
                .unwrap();
        }

        let ids = manager.list_component_ids().unwrap();
        assert_eq!(ids.len(), 4);
    }

    #[test]
    fn test_create_feature() {
        let (_temp, manager) = create_test_manager();

        let feature = manager
            .create_feature(
                "Hole Diameter".to_string(),
                "Main mounting hole".to_string(),
                FeatureType::Internal,
                10.0,
                10.05,
                9.95,
                DistributionType::Normal,
            )
            .unwrap();

        assert_eq!(feature.name, "Hole Diameter");
        assert_eq!(feature.nominal, 10.0);
        assert_eq!(feature.upper_tolerance, 10.05);
        assert_eq!(feature.lower_tolerance, 9.95);
        assert_eq!(feature.metadata.entity_type, EntityType::Feature);
    }

    #[test]
    fn test_create_feature_validation_empty_name() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_feature(
            "  ".to_string(),
            "Description".to_string(),
            FeatureType::Internal,
            10.0,
            10.05,
            9.95,
            DistributionType::Normal,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_create_feature_validation_invalid_tolerances() {
        let (_temp, manager) = create_test_manager();

        // Upper tolerance less than lower tolerance
        let result = manager.create_feature(
            "Bad Tolerance".to_string(),
            "Description".to_string(),
            FeatureType::Internal,
            10.0,
            9.95, // Upper < Lower - INVALID
            10.05,
            DistributionType::Normal,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_feature() {
        let (_temp, manager) = create_test_manager();

        let feature = manager
            .create_feature(
                "Shaft Diameter".to_string(),
                "Drive shaft".to_string(),
                FeatureType::External,
                25.0,
                25.02,
                24.98,
                DistributionType::Normal,
            )
            .unwrap();

        let retrieved = manager.get_feature(&feature.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Shaft Diameter");

        let mut updated = retrieved;
        updated.drawing_location = Some("Detail A, Sheet 3".to_string());
        updated.custom_mean = Some(25.0);
        updated.custom_std_dev = Some(0.01);

        let saved = manager.update_feature(updated).unwrap();
        assert_eq!(saved.drawing_location, Some("Detail A, Sheet 3".to_string()));
        assert_eq!(saved.custom_mean, Some(25.0));
    }

    #[test]
    fn test_update_feature_validation() {
        let (_temp, manager) = create_test_manager();

        let mut feature = manager
            .create_feature(
                "Test Feature".to_string(),
                "Description".to_string(),
                FeatureType::Internal,
                10.0,
                10.05,
                9.95,
                DistributionType::Normal,
            )
            .unwrap();

        // Try to update with invalid tolerances
        feature.upper_tolerance = 9.90; // Now upper < lower

        let result = manager.update_feature(feature);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_feature() {
        let (_temp, manager) = create_test_manager();

        let feature = manager
            .create_feature(
                "To Delete".to_string(),
                "Description".to_string(),
                FeatureType::External,
                5.0,
                5.1,
                4.9,
                DistributionType::Uniform,
            )
            .unwrap();

        let id = feature.metadata.id;
        manager.delete_feature(&id).unwrap();

        let result = manager.get_feature(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_feature_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..5 {
            manager
                .create_feature(
                    format!("Feature {}", i),
                    "Description".to_string(),
                    FeatureType::Internal,
                    10.0 + i as f64,
                    10.05 + i as f64,
                    9.95 + i as f64,
                    DistributionType::Normal,
                )
                .unwrap();
        }

        let ids = manager.list_feature_ids().unwrap();
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn test_create_mate() {
        let (_temp, manager) = create_test_manager();

        let mate = manager
            .create_mate(
                "Pin-Hole Clearance".to_string(),
                "Clearance fit between pin and hole".to_string(),
                MateType::Clearance,
            )
            .unwrap();

        assert_eq!(mate.name, "Pin-Hole Clearance");
        assert_eq!(mate.mate_type, MateType::Clearance);
        assert_eq!(mate.metadata.entity_type, EntityType::Mate);
    }

    #[test]
    fn test_create_mate_validation() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_mate(
            "".to_string(),
            "Description".to_string(),
            MateType::Clearance,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_and_update_mate() {
        let (_temp, manager) = create_test_manager();

        let mate = manager
            .create_mate(
                "Bearing Fit".to_string(),
                "Press fit bearing".to_string(),
                MateType::InterferenceFit,
            )
            .unwrap();

        let retrieved = manager.get_mate(&mate.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Bearing Fit");

        let mut updated = retrieved;
        updated.mmc = Some(25.015);
        updated.lmc = Some(24.985);

        let saved = manager.update_mate(updated).unwrap();
        assert_eq!(saved.mmc, Some(25.015));
        assert_eq!(saved.lmc, Some(24.985));
    }

    #[test]
    fn test_delete_mate() {
        let (_temp, manager) = create_test_manager();

        let mate = manager
            .create_mate(
                "To Delete".to_string(),
                "Description".to_string(),
                MateType::Clearance,
            )
            .unwrap();

        let id = mate.metadata.id;
        manager.delete_mate(&id).unwrap();

        let result = manager.get_mate(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_mate_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..3 {
            manager
                .create_mate(
                    format!("Mate {}", i),
                    "Description".to_string(),
                    MateType::Clearance,
                )
                .unwrap();
        }

        let ids = manager.list_mate_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_create_stackup() {
        let (_temp, manager) = create_test_manager();

        let stackup = manager
            .create_stackup(
                "Gap Analysis".to_string(),
                "Critical gap between components".to_string(),
                vec![AnalysisType::WorstCase, AnalysisType::RSS],
            )
            .unwrap();

        assert_eq!(stackup.name, "Gap Analysis");
        assert_eq!(stackup.analysis_types.len(), 2);
        assert_eq!(stackup.metadata.entity_type, EntityType::Stackup);
    }

    #[test]
    fn test_create_stackup_validation_empty_name() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_stackup(
            "  ".to_string(),
            "Description".to_string(),
            vec![AnalysisType::WorstCase],
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_create_stackup_validation_no_analysis_types() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_stackup(
            "Test Stackup".to_string(),
            "Description".to_string(),
            vec![], // Empty - INVALID
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_stackup() {
        let (_temp, manager) = create_test_manager();

        let stackup = manager
            .create_stackup(
                "Assembly Tolerance".to_string(),
                "Total stack tolerance".to_string(),
                vec![AnalysisType::MonteCarlo],
            )
            .unwrap();

        let retrieved = manager.get_stackup(&stackup.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Assembly Tolerance");

        let mut updated = retrieved;
        updated.upper_spec_limit = Some(0.5);
        updated.lower_spec_limit = Some(-0.5);
        updated.analysis_types.push(AnalysisType::RSS);

        let saved = manager.update_stackup(updated).unwrap();
        assert_eq!(saved.upper_spec_limit, Some(0.5));
        assert_eq!(saved.analysis_types.len(), 2);
    }

    #[test]
    fn test_update_stackup_validation() {
        let (_temp, manager) = create_test_manager();

        let mut stackup = manager
            .create_stackup(
                "Test Stackup".to_string(),
                "Description".to_string(),
                vec![AnalysisType::WorstCase],
            )
            .unwrap();

        // Try to update with empty analysis_types
        stackup.analysis_types.clear();

        let result = manager.update_stackup(stackup);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_stackup() {
        let (_temp, manager) = create_test_manager();

        let stackup = manager
            .create_stackup(
                "To Delete".to_string(),
                "Description".to_string(),
                vec![AnalysisType::WorstCase],
            )
            .unwrap();

        let id = stackup.metadata.id;
        manager.delete_stackup(&id).unwrap();

        let result = manager.get_stackup(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_stackup_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..4 {
            manager
                .create_stackup(
                    format!("Stackup {}", i),
                    "Description".to_string(),
                    vec![AnalysisType::RSS],
                )
                .unwrap();
        }

        let ids = manager.list_stackup_ids().unwrap();
        assert_eq!(ids.len(), 4);
    }

    #[test]
    fn test_create_supplier() {
        let (_temp, manager) = create_test_manager();

        let supplier = manager
            .create_supplier(
                "Acme Manufacturing".to_string(),
                "CNC machining and fabrication".to_string(),
            )
            .unwrap();

        assert_eq!(supplier.name, "Acme Manufacturing");
        assert_eq!(supplier.metadata.entity_type, EntityType::Supplier);
    }

    #[test]
    fn test_create_supplier_validation() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_supplier(
            "  ".to_string(),
            "Description".to_string(),
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_supplier() {
        let (_temp, manager) = create_test_manager();

        let supplier = manager
            .create_supplier(
                "Beta Industries".to_string(),
                "Injection molding".to_string(),
            )
            .unwrap();

        let retrieved = manager.get_supplier(&supplier.metadata.id).unwrap();
        assert_eq!(retrieved.name, "Beta Industries");

        let mut updated = retrieved;
        updated.contact_name = Some("John Smith".to_string());
        updated.email = Some("john@betaindustries.com".to_string());
        updated.phone = Some("+1-555-0123".to_string());
        updated.address = Some("123 Industrial Way, Springfield".to_string());

        let saved = manager.update_supplier(updated).unwrap();
        assert_eq!(saved.contact_name, Some("John Smith".to_string()));
        assert_eq!(saved.email, Some("john@betaindustries.com".to_string()));
    }

    #[test]
    fn test_delete_supplier() {
        let (_temp, manager) = create_test_manager();

        let supplier = manager
            .create_supplier(
                "To Delete".to_string(),
                "Description".to_string(),
            )
            .unwrap();

        let id = supplier.metadata.id;
        manager.delete_supplier(&id).unwrap();

        let result = manager.get_supplier(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_supplier_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..3 {
            manager
                .create_supplier(
                    format!("Supplier {}", i),
                    "Description".to_string(),
                )
                .unwrap();
        }

        let ids = manager.list_supplier_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_create_quote() {
        let (_temp, manager) = create_test_manager();

        let quote_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let quote = manager
            .create_quote(
                "Q-2025-001".to_string(),
                quote_date,
                vec![(100, 5.50), (500, 4.75), (1000, 4.25)],
                CostDistribution::Linear,
            )
            .unwrap();

        assert_eq!(quote.quote_number, "Q-2025-001");
        assert_eq!(quote.quantity_price_pairs.len(), 3);
        assert_eq!(quote.distribution_type, CostDistribution::Linear);
        assert_eq!(quote.metadata.entity_type, EntityType::Quote);
    }

    #[test]
    fn test_create_quote_validation_empty_number() {
        let (_temp, manager) = create_test_manager();

        let quote_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let result = manager.create_quote(
            "  ".to_string(),
            quote_date,
            vec![(100, 5.50)],
            CostDistribution::Linear,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_create_quote_validation_no_price_pairs() {
        let (_temp, manager) = create_test_manager();

        let quote_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let result = manager.create_quote(
            "Q-2025-001".to_string(),
            quote_date,
            vec![], // Empty - INVALID
            CostDistribution::Linear,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_quote() {
        let (_temp, manager) = create_test_manager();

        let quote_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let quote = manager
            .create_quote(
                "Q-2025-002".to_string(),
                quote_date,
                vec![(50, 12.00), (100, 10.50)],
                CostDistribution::Power,
            )
            .unwrap();

        let retrieved = manager.get_quote(&quote.metadata.id).unwrap();
        assert_eq!(retrieved.quote_number, "Q-2025-002");

        let mut updated = retrieved;
        let expiration = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        updated.expiration_date = Some(expiration);
        updated.notes = Some("Net 30 terms".to_string());

        let saved = manager.update_quote(updated).unwrap();
        assert_eq!(saved.expiration_date, Some(expiration));
        assert_eq!(saved.notes, Some("Net 30 terms".to_string()));
    }

    #[test]
    fn test_update_quote_validation() {
        let (_temp, manager) = create_test_manager();

        let quote_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let mut quote = manager
            .create_quote(
                "Q-2025-003".to_string(),
                quote_date,
                vec![(100, 5.00)],
                CostDistribution::Exponential,
            )
            .unwrap();

        // Try to update with empty quantity_price_pairs
        quote.quantity_price_pairs.clear();

        let result = manager.update_quote(quote);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_quote() {
        let (_temp, manager) = create_test_manager();

        let quote_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let quote = manager
            .create_quote(
                "To Delete".to_string(),
                quote_date,
                vec![(100, 1.00)],
                CostDistribution::Logarithmic,
            )
            .unwrap();

        let id = quote.metadata.id;
        manager.delete_quote(&id).unwrap();

        let result = manager.get_quote(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_quote_ids() {
        let (_temp, manager) = create_test_manager();

        let quote_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        for i in 0..3 {
            manager
                .create_quote(
                    format!("Q-2025-{:03}", i),
                    quote_date,
                    vec![(100, 5.00 + i as f64)],
                    CostDistribution::Linear,
                )
                .unwrap();
        }

        let ids = manager.list_quote_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    // ============================================================================
    // Verification Tests
    // ============================================================================

    #[test]
    fn test_create_verification() {
        let (_temp, manager) = create_test_manager();

        let test_steps = vec![
            TestStep {
                step_number: 1,
                description: "Power on device".to_string(),
                expected_result: "LED turns green".to_string(),
                actual_result: None,
                passed: None,
            },
        ];

        let verification = manager
            .create_verification(
                "VER-001".to_string(),
                "Power-on test".to_string(),
                "Functional Test".to_string(),
                test_steps,
                vec!["Device powers on successfully".to_string()],
                TestPriority::High,
            )
            .unwrap();

        assert_eq!(verification.name, "VER-001");
        assert_eq!(verification.test_type, "Functional Test");
        assert_eq!(verification.priority, TestPriority::High);
        assert_eq!(verification.metadata.entity_type, EntityType::Verification);
    }

    #[test]
    fn test_create_verification_validation_empty_name() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_verification(
            "  ".to_string(),
            "Description".to_string(),
            "Test Type".to_string(),
            vec![],
            vec!["Criteria".to_string()],
            TestPriority::Medium,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_verification() {
        let (_temp, manager) = create_test_manager();

        let test_steps = vec![
            TestStep {
                step_number: 1,
                description: "Test step".to_string(),
                expected_result: "Pass".to_string(),
                actual_result: None,
                passed: None,
            },
        ];

        let verification = manager
            .create_verification(
                "VER-002".to_string(),
                "Test procedure".to_string(),
                "Integration Test".to_string(),
                test_steps,
                vec!["All steps pass".to_string()],
                TestPriority::Medium,
            )
            .unwrap();

        let retrieved = manager.get_verification(&verification.metadata.id).unwrap();
        assert_eq!(retrieved.name, "VER-002");

        let mut updated = retrieved;
        updated.executed_by = Some("Test Engineer".to_string());
        updated.pass_fail = Some(true);

        let saved = manager.update_verification(updated).unwrap();
        assert_eq!(saved.executed_by, Some("Test Engineer".to_string()));
        assert_eq!(saved.pass_fail, Some(true));
    }

    #[test]
    fn test_delete_verification() {
        let (_temp, manager) = create_test_manager();

        let verification = manager
            .create_verification(
                "To Delete".to_string(),
                "Description".to_string(),
                "Test".to_string(),
                vec![],
                vec!["Criteria".to_string()],
                TestPriority::Low,
            )
            .unwrap();

        let id = verification.metadata.id;
        manager.delete_verification(&id).unwrap();

        let result = manager.get_verification(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_verification_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..3 {
            manager
                .create_verification(
                    format!("VER-{:03}", i),
                    "Description".to_string(),
                    "Test".to_string(),
                    vec![],
                    vec!["Criteria".to_string()],
                    TestPriority::Medium,
                )
                .unwrap();
        }

        let ids = manager.list_verification_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    // ============================================================================
    // Validation Tests
    // ============================================================================

    #[test]
    fn test_create_validation() {
        let (_temp, manager) = create_test_manager();

        let validation = manager
            .create_validation(
                "VAL-001".to_string(),
                "User acceptance testing".to_string(),
                "UAT".to_string(),
                vec!["User 1".to_string(), "User 2".to_string()],
                vec!["Users can complete workflow".to_string()],
                TestPriority::High,
            )
            .unwrap();

        assert_eq!(validation.name, "VAL-001");
        assert_eq!(validation.validation_type, "UAT");
        assert_eq!(validation.participants.len(), 2);
        assert_eq!(validation.priority, TestPriority::High);
        assert_eq!(validation.metadata.entity_type, EntityType::Validation);
    }

    #[test]
    fn test_create_validation_validation_empty_name() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_validation(
            "".to_string(),
            "Description".to_string(),
            "Type".to_string(),
            vec!["Participant".to_string()],
            vec!["Criteria".to_string()],
            TestPriority::Medium,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_validation() {
        let (_temp, manager) = create_test_manager();

        let validation = manager
            .create_validation(
                "VAL-002".to_string(),
                "Field testing".to_string(),
                "Field Test".to_string(),
                vec!["Tester 1".to_string()],
                vec!["Device works in field".to_string()],
                TestPriority::High,
            )
            .unwrap();

        let retrieved = manager.get_validation(&validation.metadata.id).unwrap();
        assert_eq!(retrieved.name, "VAL-002");

        let mut updated = retrieved;
        updated.approved = Some(true);
        updated.approved_by = Some("Manager".to_string());
        updated.user_feedback.push("Great experience".to_string());

        let saved = manager.update_validation(updated).unwrap();
        assert_eq!(saved.approved, Some(true));
        assert_eq!(saved.approved_by, Some("Manager".to_string()));
        assert_eq!(saved.user_feedback.len(), 1);
    }

    #[test]
    fn test_delete_validation() {
        let (_temp, manager) = create_test_manager();

        let validation = manager
            .create_validation(
                "To Delete".to_string(),
                "Description".to_string(),
                "Type".to_string(),
                vec!["Person".to_string()],
                vec!["Criteria".to_string()],
                TestPriority::Low,
            )
            .unwrap();

        let id = validation.metadata.id;
        manager.delete_validation(&id).unwrap();

        let result = manager.get_validation(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_validation_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..4 {
            manager
                .create_validation(
                    format!("VAL-{:03}", i),
                    "Description".to_string(),
                    "Type".to_string(),
                    vec!["Participant".to_string()],
                    vec!["Criteria".to_string()],
                    TestPriority::Medium,
                )
                .unwrap();
        }

        let ids = manager.list_validation_ids().unwrap();
        assert_eq!(ids.len(), 4);
    }

    // ============================================================================
    // Manufacturing Tests
    // ============================================================================

    #[test]
    fn test_create_manufacturing() {
        let (_temp, manager) = create_test_manager();

        let work_instructions = vec![
            WorkInstructionStep {
                step_number: 1,
                operation: "Cut material to length".to_string(),
                description: "Use band saw to cut aluminum extrusion to specified length".to_string(),
                tools_required: vec!["Band saw".to_string()],
                estimated_time_minutes: Some(5.0),
                safety_notes: vec!["Wear safety glasses".to_string()],
                quality_checks: vec!["Verify length with caliper".to_string()],
            },
        ];

        let manufacturing = manager
            .create_manufacturing(
                "MFG-001".to_string(),
                "Assembly process".to_string(),
                "Assembly".to_string(),
                work_instructions,
                1,
            )
            .unwrap();

        assert_eq!(manufacturing.name, "MFG-001");
        assert_eq!(manufacturing.process_type, "Assembly");
        assert_eq!(manufacturing.priority, 1);
        assert_eq!(manufacturing.work_instructions.len(), 1);
        assert_eq!(manufacturing.metadata.entity_type, EntityType::Manufacturing);
    }

    #[test]
    fn test_create_manufacturing_validation_empty_name() {
        let (_temp, manager) = create_test_manager();

        let result = manager.create_manufacturing(
            "  ".to_string(),
            "Description".to_string(),
            "Type".to_string(),
            vec![],
            1,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_and_update_manufacturing() {
        let (_temp, manager) = create_test_manager();

        let manufacturing = manager
            .create_manufacturing(
                "MFG-002".to_string(),
                "Machining process".to_string(),
                "CNC Machining".to_string(),
                vec![],
                2,
            )
            .unwrap();

        let retrieved = manager.get_manufacturing(&manufacturing.metadata.id).unwrap();
        assert_eq!(retrieved.name, "MFG-002");

        let mut updated = retrieved;
        updated.work_center = Some("Machine Shop A".to_string());
        updated.operators.push("Operator 1".to_string());
        updated.setup_time_minutes = Some(30.0);
        updated.cycle_time_minutes = Some(15.0);

        let saved = manager.update_manufacturing(updated).unwrap();
        assert_eq!(saved.work_center, Some("Machine Shop A".to_string()));
        assert_eq!(saved.operators.len(), 1);
        assert_eq!(saved.setup_time_minutes, Some(30.0));
    }

    #[test]
    fn test_delete_manufacturing() {
        let (_temp, manager) = create_test_manager();

        let manufacturing = manager
            .create_manufacturing(
                "To Delete".to_string(),
                "Description".to_string(),
                "Type".to_string(),
                vec![],
                1,
            )
            .unwrap();

        let id = manufacturing.metadata.id;
        manager.delete_manufacturing(&id).unwrap();

        let result = manager.get_manufacturing(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_manufacturing_ids() {
        let (_temp, manager) = create_test_manager();

        for i in 0..3 {
            manager
                .create_manufacturing(
                    format!("MFG-{:03}", i),
                    "Description".to_string(),
                    "Type".to_string(),
                    vec![],
                    1,
                )
                .unwrap();
        }

        let ids = manager.list_manufacturing_ids().unwrap();
        assert_eq!(ids.len(), 3);
    }

    // ============================================================================
    // Integration Tests for New Entities
    // ============================================================================

    #[test]
    fn test_integration_verification_workflow() {
        let (_temp, manager) = create_test_manager();

        // Create test steps
        let test_steps = vec![
            TestStep {
                step_number: 1,
                description: "Apply power".to_string(),
                expected_result: "Device turns on".to_string(),
                actual_result: None,
                passed: None,
            },
            TestStep {
                step_number: 2,
                description: "Check voltage".to_string(),
                expected_result: "5V ± 0.1V".to_string(),
                actual_result: None,
                passed: None,
            },
        ];

        // Create verification
        let mut verification = manager
            .create_verification(
                "VER-INT-001".to_string(),
                "Power supply verification".to_string(),
                "Functional Test".to_string(),
                test_steps,
                vec![
                    "Device powers on".to_string(),
                    "Voltage within spec".to_string(),
                ],
                TestPriority::Critical,
            )
            .unwrap();

        // Execute test
        verification.test_steps[0].actual_result = Some("Device turned on".to_string());
        verification.test_steps[0].passed = Some(true);
        verification.test_steps[1].actual_result = Some("5.02V".to_string());
        verification.test_steps[1].passed = Some(true);
        verification.executed_by = Some("Test Engineer".to_string());
        verification.pass_fail = Some(true);

        let updated = manager.update_verification(verification).unwrap();

        // Verify the results
        assert_eq!(updated.pass_fail, Some(true));
        assert_eq!(updated.test_steps.len(), 2);
        assert_eq!(updated.test_steps[0].passed, Some(true));
        assert_eq!(updated.test_steps[1].passed, Some(true));
    }

    #[test]
    fn test_integration_validation_approval_workflow() {
        let (_temp, manager) = create_test_manager();

        // Create validation
        let mut validation = manager
            .create_validation(
                "VAL-INT-001".to_string(),
                "Customer acceptance testing".to_string(),
                "Customer UAT".to_string(),
                vec![
                    "Customer A".to_string(),
                    "Customer B".to_string(),
                    "Internal QA".to_string(),
                ],
                vec![
                    "All critical features work".to_string(),
                    "Performance meets requirements".to_string(),
                ],
                TestPriority::Critical,
            )
            .unwrap();

        // Add user feedback
        validation.user_feedback.push("Easy to use".to_string());
        validation.user_feedback.push("Meets all our needs".to_string());
        validation.results_summary = Some("All participants satisfied".to_string());

        // Approve
        validation.approved = Some(true);
        validation.approved_by = Some("Product Manager".to_string());

        let final_validation = manager.update_validation(validation).unwrap();

        // Verify approval workflow
        assert_eq!(final_validation.approved, Some(true));
        assert_eq!(final_validation.user_feedback.len(), 2);
        assert!(final_validation.results_summary.is_some());
    }

    #[test]
    fn test_integration_manufacturing_production_workflow() {
        let (_temp, manager) = create_test_manager();

        // Create manufacturing process with detailed work instructions
        let work_instructions = vec![
            WorkInstructionStep {
                step_number: 1,
                operation: "Load material into fixture".to_string(),
                description: "Place workpiece into fixture and secure with clamps".to_string(),
                tools_required: vec!["Fixture A".to_string()],
                estimated_time_minutes: Some(2.0),
                safety_notes: vec!["Ensure fixture is locked".to_string()],
                quality_checks: vec!["Visual inspection of alignment".to_string()],
            },
            WorkInstructionStep {
                step_number: 2,
                operation: "Machine part to drawing spec".to_string(),
                description: "Run CNC program to machine all features per drawing specifications".to_string(),
                tools_required: vec!["CNC Mill".to_string(), "Cutting tools".to_string()],
                estimated_time_minutes: Some(15.0),
                safety_notes: vec!["Do not open door while running".to_string()],
                quality_checks: vec![
                    "Measure critical dimensions".to_string(),
                    "Check surface finish".to_string(),
                ],
            },
        ];

        let mut manufacturing = manager
            .create_manufacturing(
                "MFG-INT-001".to_string(),
                "Part production process".to_string(),
                "CNC Machining".to_string(),
                work_instructions,
                1,
            )
            .unwrap();

        // Set up production details
        manufacturing.work_center = Some("CNC Area 1".to_string());
        manufacturing.equipment_required = vec!["CNC Mill #3".to_string()];
        manufacturing.operators = vec!["Operator Smith".to_string()];
        manufacturing.setup_time_minutes = Some(30.0);
        manufacturing.cycle_time_minutes = Some(17.0);
        manufacturing.materials_required = vec!["Aluminum 6061 - 2x4x6".to_string()];

        let updated = manager.update_manufacturing(manufacturing).unwrap();

        // Verify the complete setup
        assert_eq!(updated.work_instructions.len(), 2);
        assert_eq!(updated.operators.len(), 1);
        assert_eq!(updated.equipment_required.len(), 1);
        assert_eq!(updated.setup_time_minutes, Some(30.0));
        assert_eq!(updated.cycle_time_minutes, Some(17.0));
    }
}
