use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use crate::core::EdtResult;
use crate::models::{
    EntityType, Task, Requirement, Risk, Hazard, RiskControl, Milestone, Resource, Calendar, Baseline,
    Assembly, Component, Feature, Mate, Stackup, Supplier, Quote,
};

/// RON file storage for entities
pub struct RonStorage {
    project_root: PathBuf,
}

impl RonStorage {
    /// Create a new RON storage instance
    pub fn new<P: AsRef<Path>>(project_root: P) -> EdtResult<Self> {
        let project_root = project_root.as_ref().to_path_buf();

        // Ensure entities directory exists
        let entities_dir = project_root.join("entities");
        fs::create_dir_all(&entities_dir)?;

        Ok(Self { project_root })
    }

    /// Get the directory path for an entity type
    fn get_entity_dir(&self, entity_type: &EntityType) -> PathBuf {
        self.project_root
            .join("entities")
            .join(entity_type.folder_name())
    }

    /// Get the file path for an entity
    fn get_entity_path(&self, entity_type: &EntityType, entity_id: &Uuid) -> PathBuf {
        self.get_entity_dir(entity_type)
            .join(format!("{}.ron", entity_id))
    }

    /// Ensure directory exists for entity type
    fn ensure_entity_dir(&self, entity_type: &EntityType) -> EdtResult<()> {
        let dir = self.get_entity_dir(entity_type);
        fs::create_dir_all(dir)?;
        Ok(())
    }

    /// Write a Task entity to RON file
    pub fn write_task(&self, task: &Task) -> EdtResult<()> {
        self.ensure_entity_dir(&task.metadata.entity_type)?;
        let path = self.get_entity_path(&task.metadata.entity_type, &task.metadata.id);
        let serialized = ron::ser::to_string_pretty(task, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Task entity from RON file
    pub fn read_task(&self, entity_id: &Uuid) -> EdtResult<Task> {
        let path = self.get_entity_path(&EntityType::Task, entity_id);
        let content = fs::read_to_string(path)?;
        let task: Task = ron::from_str(&content)?;
        Ok(task)
    }

    /// Write a Requirement entity
    pub fn write_requirement(&self, requirement: &Requirement) -> EdtResult<()> {
        self.ensure_entity_dir(&requirement.metadata.entity_type)?;
        let path = self.get_entity_path(&requirement.metadata.entity_type, &requirement.metadata.id);
        let serialized = ron::ser::to_string_pretty(requirement, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Requirement entity
    pub fn read_requirement(&self, entity_id: &Uuid) -> EdtResult<Requirement> {
        let path = self.get_entity_path(&EntityType::Requirement, entity_id);
        let content = fs::read_to_string(path)?;
        let requirement: Requirement = ron::from_str(&content)?;
        Ok(requirement)
    }

    /// Write a Risk entity
    pub fn write_risk(&self, risk: &Risk) -> EdtResult<()> {
        self.ensure_entity_dir(&risk.metadata.entity_type)?;
        let path = self.get_entity_path(&risk.metadata.entity_type, &risk.metadata.id);
        let serialized = ron::ser::to_string_pretty(risk, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Risk entity
    pub fn read_risk(&self, entity_id: &Uuid) -> EdtResult<Risk> {
        let path = self.get_entity_path(&EntityType::Risk, entity_id);
        let content = fs::read_to_string(path)?;
        let risk: Risk = ron::from_str(&content)?;
        Ok(risk)
    }

    /// Write a Hazard entity
    pub fn write_hazard(&self, hazard: &Hazard) -> EdtResult<()> {
        self.ensure_entity_dir(&hazard.metadata.entity_type)?;
        let path = self.get_entity_path(&hazard.metadata.entity_type, &hazard.metadata.id);
        let serialized = ron::ser::to_string_pretty(hazard, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Hazard entity
    pub fn read_hazard(&self, entity_id: &Uuid) -> EdtResult<Hazard> {
        let path = self.get_entity_path(&EntityType::Hazard, entity_id);
        let content = fs::read_to_string(path)?;
        let hazard: Hazard = ron::from_str(&content)?;
        Ok(hazard)
    }

    /// Write a RiskControl entity
    pub fn write_risk_control(&self, control: &RiskControl) -> EdtResult<()> {
        self.ensure_entity_dir(&control.metadata.entity_type)?;
        let path = self.get_entity_path(&control.metadata.entity_type, &control.metadata.id);
        let serialized = ron::ser::to_string_pretty(control, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a RiskControl entity
    pub fn read_risk_control(&self, entity_id: &Uuid) -> EdtResult<RiskControl> {
        let path = self.get_entity_path(&EntityType::RiskControl, entity_id);
        let content = fs::read_to_string(path)?;
        let control: RiskControl = ron::from_str(&content)?;
        Ok(control)
    }

    /// Write a Milestone entity
    pub fn write_milestone(&self, milestone: &Milestone) -> EdtResult<()> {
        self.ensure_entity_dir(&milestone.metadata.entity_type)?;
        let path = self.get_entity_path(&milestone.metadata.entity_type, &milestone.metadata.id);
        let serialized = ron::ser::to_string_pretty(milestone, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Milestone entity
    pub fn read_milestone(&self, entity_id: &Uuid) -> EdtResult<Milestone> {
        let path = self.get_entity_path(&EntityType::Milestone, entity_id);
        let content = fs::read_to_string(path)?;
        let milestone: Milestone = ron::from_str(&content)?;
        Ok(milestone)
    }

    /// Write a Resource entity
    pub fn write_resource(&self, resource: &Resource) -> EdtResult<()> {
        self.ensure_entity_dir(&resource.metadata.entity_type)?;
        let path = self.get_entity_path(&resource.metadata.entity_type, &resource.metadata.id);
        let serialized = ron::ser::to_string_pretty(resource, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Resource entity
    pub fn read_resource(&self, entity_id: &Uuid) -> EdtResult<Resource> {
        let path = self.get_entity_path(&EntityType::Resource, entity_id);
        let content = fs::read_to_string(path)?;
        let resource: Resource = ron::from_str(&content)?;
        Ok(resource)
    }

    /// Write a Calendar entity
    pub fn write_calendar(&self, calendar: &Calendar) -> EdtResult<()> {
        self.ensure_entity_dir(&calendar.metadata.entity_type)?;
        let path = self.get_entity_path(&calendar.metadata.entity_type, &calendar.metadata.id);
        let serialized = ron::ser::to_string_pretty(calendar, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Calendar entity
    pub fn read_calendar(&self, entity_id: &Uuid) -> EdtResult<Calendar> {
        let path = self.get_entity_path(&EntityType::Calendar, entity_id);
        let content = fs::read_to_string(path)?;
        let calendar: Calendar = ron::from_str(&content)?;
        Ok(calendar)
    }

    /// Write a Baseline entity
    pub fn write_baseline(&self, baseline: &Baseline) -> EdtResult<()> {
        self.ensure_entity_dir(&baseline.metadata.entity_type)?;
        let path = self.get_entity_path(&baseline.metadata.entity_type, &baseline.metadata.id);
        let serialized = ron::ser::to_string_pretty(baseline, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Baseline entity
    pub fn read_baseline(&self, entity_id: &Uuid) -> EdtResult<Baseline> {
        let path = self.get_entity_path(&EntityType::Baseline, entity_id);
        let content = fs::read_to_string(path)?;
        let baseline: Baseline = ron::from_str(&content)?;
        Ok(baseline)
    }

    /// Write an Assembly entity
    pub fn write_assembly(&self, assembly: &Assembly) -> EdtResult<()> {
        self.ensure_entity_dir(&assembly.metadata.entity_type)?;
        let path = self.get_entity_path(&assembly.metadata.entity_type, &assembly.metadata.id);
        let serialized = ron::ser::to_string_pretty(assembly, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read an Assembly entity
    pub fn read_assembly(&self, entity_id: &Uuid) -> EdtResult<Assembly> {
        let path = self.get_entity_path(&EntityType::Assembly, entity_id);
        let content = fs::read_to_string(path)?;
        let assembly: Assembly = ron::from_str(&content)?;
        Ok(assembly)
    }

    /// Write a Component entity
    pub fn write_component(&self, component: &Component) -> EdtResult<()> {
        self.ensure_entity_dir(&component.metadata.entity_type)?;
        let path = self.get_entity_path(&component.metadata.entity_type, &component.metadata.id);
        let serialized = ron::ser::to_string_pretty(component, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Component entity
    pub fn read_component(&self, entity_id: &Uuid) -> EdtResult<Component> {
        let path = self.get_entity_path(&EntityType::Component, entity_id);
        let content = fs::read_to_string(path)?;
        let component: Component = ron::from_str(&content)?;
        Ok(component)
    }

    /// Write a Feature entity
    pub fn write_feature(&self, feature: &Feature) -> EdtResult<()> {
        self.ensure_entity_dir(&feature.metadata.entity_type)?;
        let path = self.get_entity_path(&feature.metadata.entity_type, &feature.metadata.id);
        let serialized = ron::ser::to_string_pretty(feature, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Feature entity
    pub fn read_feature(&self, entity_id: &Uuid) -> EdtResult<Feature> {
        let path = self.get_entity_path(&EntityType::Feature, entity_id);
        let content = fs::read_to_string(path)?;
        let feature: Feature = ron::from_str(&content)?;
        Ok(feature)
    }

    /// Write a Mate entity
    pub fn write_mate(&self, mate: &Mate) -> EdtResult<()> {
        self.ensure_entity_dir(&mate.metadata.entity_type)?;
        let path = self.get_entity_path(&mate.metadata.entity_type, &mate.metadata.id);
        let serialized = ron::ser::to_string_pretty(mate, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Mate entity
    pub fn read_mate(&self, entity_id: &Uuid) -> EdtResult<Mate> {
        let path = self.get_entity_path(&EntityType::Mate, entity_id);
        let content = fs::read_to_string(path)?;
        let mate: Mate = ron::from_str(&content)?;
        Ok(mate)
    }

    /// Write a Stackup entity
    pub fn write_stackup(&self, stackup: &Stackup) -> EdtResult<()> {
        self.ensure_entity_dir(&stackup.metadata.entity_type)?;
        let path = self.get_entity_path(&stackup.metadata.entity_type, &stackup.metadata.id);
        let serialized = ron::ser::to_string_pretty(stackup, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Stackup entity
    pub fn read_stackup(&self, entity_id: &Uuid) -> EdtResult<Stackup> {
        let path = self.get_entity_path(&EntityType::Stackup, entity_id);
        let content = fs::read_to_string(path)?;
        let stackup: Stackup = ron::from_str(&content)?;
        Ok(stackup)
    }

    /// Write a Supplier entity
    pub fn write_supplier(&self, supplier: &Supplier) -> EdtResult<()> {
        self.ensure_entity_dir(&supplier.metadata.entity_type)?;
        let path = self.get_entity_path(&supplier.metadata.entity_type, &supplier.metadata.id);
        let serialized = ron::ser::to_string_pretty(supplier, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Supplier entity
    pub fn read_supplier(&self, entity_id: &Uuid) -> EdtResult<Supplier> {
        let path = self.get_entity_path(&EntityType::Supplier, entity_id);
        let content = fs::read_to_string(path)?;
        let supplier: Supplier = ron::from_str(&content)?;
        Ok(supplier)
    }

    /// Write a Quote entity
    pub fn write_quote(&self, quote: &Quote) -> EdtResult<()> {
        self.ensure_entity_dir(&quote.metadata.entity_type)?;
        let path = self.get_entity_path(&quote.metadata.entity_type, &quote.metadata.id);
        let serialized = ron::ser::to_string_pretty(quote, ron::ser::PrettyConfig::default())?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Read a Quote entity
    pub fn read_quote(&self, entity_id: &Uuid) -> EdtResult<Quote> {
        let path = self.get_entity_path(&EntityType::Quote, entity_id);
        let content = fs::read_to_string(path)?;
        let quote: Quote = ron::from_str(&content)?;
        Ok(quote)
    }

    /// Delete an entity file
    pub fn delete(&self, entity_type: &EntityType, entity_id: &Uuid) -> EdtResult<()> {
        let path = self.get_entity_path(entity_type, entity_id);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Check if entity exists
    pub fn exists(&self, entity_type: &EntityType, entity_id: &Uuid) -> bool {
        self.get_entity_path(entity_type, entity_id).exists()
    }

    /// List all entity IDs of a given type
    pub fn list_ids(&self, entity_type: &EntityType) -> EdtResult<Vec<Uuid>> {
        let dir = self.get_entity_dir(entity_type);

        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut ids = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(uuid) = Uuid::parse_str(file_stem) {
                        ids.push(uuid);
                    }
                }
            }
        }

        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::models::{EntityMetadata, EntityType, TaskType, SchedulingMode, ResourceType};
    use chrono::Utc;

    fn create_test_storage() -> (TempDir, RonStorage) {
        let temp_dir = TempDir::new().unwrap();
        let storage = RonStorage::new(temp_dir.path()).unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_storage_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let storage = RonStorage::new(temp_dir.path()).unwrap();

        // Entities directory should be created
        assert!(temp_dir.path().join("entities").exists());
    }

    #[test]
    fn test_write_and_read_task() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Task);
        let task_id = metadata.id;
        let start = Utc::now();
        let end = start + chrono::Duration::days(5);

        let task = Task {
            metadata,
            name: "Test Task".to_string(),
            description: "A test task".to_string(),
            notes: Some("Test notes".to_string()),
            scheduled_start: start,
            deadline: end,
            actual_start: None,
            actual_end: None,
            task_type: TaskType::EffortDriven,
            scheduling_mode: SchedulingMode::Automatic,
            percent_complete: 0.5,
            percent_complete_history: vec![],
            assigned_resources: vec![],
            estimated_effort: Some(40.0),
            actual_cost: None,
            calculated_cost: None,
            dependencies: vec![],
            is_critical_path: false,
            slack: Some(2.5),
            baseline_data: None,
        };

        // Write task
        storage.write_task(&task).unwrap();

        // Verify file exists
        assert!(storage.exists(&EntityType::Task, &task_id));

        // Read task back
        let read_task = storage.read_task(&task_id).unwrap();
        assert_eq!(read_task.name, "Test Task");
        assert_eq!(read_task.percent_complete, 0.5);
        assert_eq!(read_task.task_type, TaskType::EffortDriven);
    }

    #[test]
    fn test_write_and_read_requirement() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Requirement);
        let req_id = metadata.id;

        let requirement = Requirement {
            metadata,
            name: "REQ-001".to_string(),
            description: "Test requirement".to_string(),
            notes: None,
            requirement_type: "System Requirement".to_string(),
            rationale: Some("Testing".to_string()),
            source: None,
            verification_method: None,
        };

        storage.write_requirement(&requirement).unwrap();
        assert!(storage.exists(&EntityType::Requirement, &req_id));

        let read_req = storage.read_requirement(&req_id).unwrap();
        assert_eq!(read_req.name, "REQ-001");
        assert_eq!(read_req.requirement_type, "System Requirement");
    }

    #[test]
    fn test_write_and_read_risk() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Risk);
        let risk_id = metadata.id;

        let risk = Risk {
            metadata,
            name: "RISK-001".to_string(),
            description: "Test risk".to_string(),
            notes: None,
            risk_type: "Safety Risk".to_string(),
            probability: 3,
            severity: 4,
            risk_score: 12,
            residual_probability: Some(1),
            residual_severity: Some(4),
            residual_risk_score: Some(4),
        };

        storage.write_risk(&risk).unwrap();
        assert!(storage.exists(&EntityType::Risk, &risk_id));

        let read_risk = storage.read_risk(&risk_id).unwrap();
        assert_eq!(read_risk.name, "RISK-001");
        assert_eq!(read_risk.risk_score, 12);
    }

    #[test]
    fn test_write_and_read_hazard() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Hazard);
        let hazard_id = metadata.id;

        let hazard = Hazard {
            metadata,
            name: "HAZ-001".to_string(),
            description: "Test hazard".to_string(),
            notes: None,
            causes: vec!["Cause 1".to_string()],
            harms: vec!["Harm 1".to_string()],
        };

        storage.write_hazard(&hazard).unwrap();
        assert!(storage.exists(&EntityType::Hazard, &hazard_id));

        let read_hazard = storage.read_hazard(&hazard_id).unwrap();
        assert_eq!(read_hazard.name, "HAZ-001");
        assert_eq!(read_hazard.causes.len(), 1);
    }

    #[test]
    fn test_write_and_read_milestone() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Milestone);
        let milestone_id = metadata.id;

        let milestone = Milestone {
            metadata,
            name: "M1: Project Start".to_string(),
            description: "Project kickoff".to_string(),
            date: Utc::now(),
            dependencies: vec![],
            is_critical_path: true,
        };

        storage.write_milestone(&milestone).unwrap();
        assert!(storage.exists(&EntityType::Milestone, &milestone_id));

        let read_milestone = storage.read_milestone(&milestone_id).unwrap();
        assert_eq!(read_milestone.name, "M1: Project Start");
        assert!(read_milestone.is_critical_path);
    }

    #[test]
    fn test_write_and_read_resource() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Resource);
        let resource_id = metadata.id;

        let resource = Resource {
            metadata,
            name: "John Doe".to_string(),
            description: "Senior Engineer".to_string(),
            email: Some("john@example.com".to_string()),
            resource_type: ResourceType::Labor,
            bill_rate: Some(150.0),
            calendar_id: None,
        };

        storage.write_resource(&resource).unwrap();
        assert!(storage.exists(&EntityType::Resource, &resource_id));

        let read_resource = storage.read_resource(&resource_id).unwrap();
        assert_eq!(read_resource.name, "John Doe");
        assert_eq!(read_resource.resource_type, ResourceType::Labor);
    }

    #[test]
    fn test_delete_entity() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Task);
        let task_id = metadata.id;

        let task = Task {
            metadata,
            name: "To Delete".to_string(),
            description: "Will be deleted".to_string(),
            notes: None,
            scheduled_start: Utc::now(),
            deadline: Utc::now(),
            actual_start: None,
            actual_end: None,
            task_type: TaskType::EffortDriven,
            scheduling_mode: SchedulingMode::Manual,
            percent_complete: 0.0,
            percent_complete_history: vec![],
            assigned_resources: vec![],
            estimated_effort: None,
            actual_cost: None,
            calculated_cost: None,
            dependencies: vec![],
            is_critical_path: false,
            slack: None,
            baseline_data: None,
        };

        storage.write_task(&task).unwrap();
        assert!(storage.exists(&EntityType::Task, &task_id));

        storage.delete(&EntityType::Task, &task_id).unwrap();
        assert!(!storage.exists(&EntityType::Task, &task_id));
    }

    #[test]
    fn test_list_entity_ids() {
        let (_temp, storage) = create_test_storage();

        // Create multiple tasks
        for i in 0..3 {
            let metadata = EntityMetadata::new(EntityType::Task);
            let task = Task {
                metadata,
                name: format!("Task {}", i),
                description: format!("Description {}", i),
                notes: None,
                scheduled_start: Utc::now(),
                deadline: Utc::now(),
                actual_start: None,
                actual_end: None,
                task_type: TaskType::EffortDriven,
                scheduling_mode: SchedulingMode::Automatic,
                percent_complete: 0.0,
                percent_complete_history: vec![],
                assigned_resources: vec![],
                estimated_effort: None,
                actual_cost: None,
                calculated_cost: None,
                dependencies: vec![],
                is_critical_path: false,
                slack: None,
                baseline_data: None,
            };
            storage.write_task(&task).unwrap();
        }

        let task_ids = storage.list_ids(&EntityType::Task).unwrap();
        assert_eq!(task_ids.len(), 3);
    }

    #[test]
    fn test_read_nonexistent_entity() {
        let (_temp, storage) = create_test_storage();

        let fake_id = Uuid::new_v4();
        let result = storage.read_task(&fake_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_and_read_calendar() {
        use chrono::Weekday;

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Calendar);
        let calendar_id = metadata.id;

        let calendar = Calendar {
            metadata,
            name: "Standard Work Week".to_string(),
            work_hours_per_day: 8.0,
            work_days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            holidays: vec![],
        };

        storage.write_calendar(&calendar).unwrap();
        assert!(storage.exists(&EntityType::Calendar, &calendar_id));

        let read_calendar = storage.read_calendar(&calendar_id).unwrap();
        assert_eq!(read_calendar.name, "Standard Work Week");
        assert_eq!(read_calendar.work_hours_per_day, 8.0);
        assert_eq!(read_calendar.work_days.len(), 5);
    }

    #[test]
    fn test_write_and_read_baseline() {
        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Baseline);
        let baseline_id = metadata.id;

        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();

        let baseline = Baseline {
            metadata,
            name: "Q1 2025 Baseline".to_string(),
            description: "Baseline for Q1 planning".to_string(),
            created_date: Utc::now(),
            task_ids: vec![task1_id, task2_id],
        };

        storage.write_baseline(&baseline).unwrap();
        assert!(storage.exists(&EntityType::Baseline, &baseline_id));

        let read_baseline = storage.read_baseline(&baseline_id).unwrap();
        assert_eq!(read_baseline.name, "Q1 2025 Baseline");
        assert_eq!(read_baseline.task_ids.len(), 2);
        assert!(read_baseline.task_ids.contains(&task1_id));
    }

    #[test]
    fn test_write_and_read_assembly() {
        use crate::models::Assembly;

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Assembly);
        let assembly_id = metadata.id;

        let assembly = Assembly {
            metadata,
            name: "Main Assembly".to_string(),
            description: "Primary product assembly".to_string(),
            revision: "A".to_string(),
            notes: Some("Initial design".to_string()),
        };

        storage.write_assembly(&assembly).unwrap();
        assert!(storage.exists(&EntityType::Assembly, &assembly_id));

        let read_assembly = storage.read_assembly(&assembly_id).unwrap();
        assert_eq!(read_assembly.name, "Main Assembly");
        assert_eq!(read_assembly.revision, "A");
    }

    #[test]
    fn test_write_and_read_component() {
        use crate::models::Component;

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Component);
        let component_id = metadata.id;

        let component = Component {
            metadata,
            name: "Bracket".to_string(),
            description: "Mounting bracket".to_string(),
            revision: "B".to_string(),
            part_number: Some("BKT-001".to_string()),
            material: Some("Aluminum 6061".to_string()),
            mass: Some(0.125),
            notes: None,
        };

        storage.write_component(&component).unwrap();
        assert!(storage.exists(&EntityType::Component, &component_id));

        let read_component = storage.read_component(&component_id).unwrap();
        assert_eq!(read_component.name, "Bracket");
        assert_eq!(read_component.part_number, Some("BKT-001".to_string()));
    }

    #[test]
    fn test_write_and_read_feature() {
        use crate::models::{Feature, FeatureType, DistributionType};

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Feature);
        let feature_id = metadata.id;

        let feature = Feature {
            metadata,
            name: "Hole Diameter".to_string(),
            description: "Main mounting hole".to_string(),
            notes: None,
            feature_type: FeatureType::Internal,
            nominal: 10.0,
            upper_tolerance: 0.1,
            lower_tolerance: -0.1,
            distribution_type: DistributionType::Normal,
            custom_mean: None,
            custom_std_dev: None,
            drawing_location: Some("Detail A".to_string()),
        };

        storage.write_feature(&feature).unwrap();
        assert!(storage.exists(&EntityType::Feature, &feature_id));

        let read_feature = storage.read_feature(&feature_id).unwrap();
        assert_eq!(read_feature.name, "Hole Diameter");
        assert_eq!(read_feature.nominal, 10.0);
    }

    #[test]
    fn test_write_and_read_mate() {
        use crate::models::{Mate, MateType, MateAnalysisResult};

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Mate);
        let mate_id = metadata.id;

        let mate = Mate {
            metadata,
            name: "Shaft-Bearing Fit".to_string(),
            description: "Interference fit".to_string(),
            notes: None,
            mate_type: MateType::InterferenceFit,
            mmc: Some(10.05),
            lmc: Some(9.95),
            analysis_result: Some(MateAnalysisResult::Pass),
        };

        storage.write_mate(&mate).unwrap();
        assert!(storage.exists(&EntityType::Mate, &mate_id));

        let read_mate = storage.read_mate(&mate_id).unwrap();
        assert_eq!(read_mate.name, "Shaft-Bearing Fit");
        assert_eq!(read_mate.mate_type, MateType::InterferenceFit);
    }

    #[test]
    fn test_write_and_read_stackup() {
        use crate::models::{Stackup, AnalysisType, StackupFeatureContribution, ContributionSign};

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Stackup);
        let stackup_id = metadata.id;

        let stackup = Stackup {
            metadata,
            name: "Gap Analysis".to_string(),
            description: "Critical gap dimension".to_string(),
            notes: None,
            analysis_types: vec![AnalysisType::WorstCase, AnalysisType::RSS],
            upper_spec_limit: Some(2.0),
            lower_spec_limit: Some(0.5),
            feature_contributions: vec![
                StackupFeatureContribution {
                    feature_id: Uuid::new_v4(),
                    sign: ContributionSign::Positive,
                    contribution: 1.0,
                },
            ],
            worst_case_result: None,
            rss_result: None,
            monte_carlo_result: None,
        };

        storage.write_stackup(&stackup).unwrap();
        assert!(storage.exists(&EntityType::Stackup, &stackup_id));

        let read_stackup = storage.read_stackup(&stackup_id).unwrap();
        assert_eq!(read_stackup.name, "Gap Analysis");
        assert_eq!(read_stackup.analysis_types.len(), 2);
    }

    #[test]
    fn test_write_and_read_supplier() {
        use crate::models::Supplier;

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Supplier);
        let supplier_id = metadata.id;

        let supplier = Supplier {
            metadata,
            name: "Acme Manufacturing".to_string(),
            description: "CNC machining services".to_string(),
            contact_name: Some("John Doe".to_string()),
            address: Some("123 Industrial Pkwy".to_string()),
            phone: Some("555-1234".to_string()),
            email: Some("john@acme.com".to_string()),
            notes: None,
        };

        storage.write_supplier(&supplier).unwrap();
        assert!(storage.exists(&EntityType::Supplier, &supplier_id));

        let read_supplier = storage.read_supplier(&supplier_id).unwrap();
        assert_eq!(read_supplier.name, "Acme Manufacturing");
        assert_eq!(read_supplier.email, Some("john@acme.com".to_string()));
    }

    #[test]
    fn test_write_and_read_quote() {
        use crate::models::{Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, storage) = create_test_storage();

        let metadata = EntityMetadata::new(EntityType::Quote);
        let quote_id = metadata.id;

        let quote = Quote {
            metadata,
            quote_number: "Q-2025-001".to_string(),
            quote_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            expiration_date: Some(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap()),
            quantity_price_pairs: vec![
                (100, 10.0),
                (500, 8.0),
                (1000, 6.0),
            ],
            distribution_type: CostDistribution::Power,
            notes: Some("Volume discounts available".to_string()),
        };

        storage.write_quote(&quote).unwrap();
        assert!(storage.exists(&EntityType::Quote, &quote_id));

        let read_quote = storage.read_quote(&quote_id).unwrap();
        assert_eq!(read_quote.quote_number, "Q-2025-001");
        assert_eq!(read_quote.quantity_price_pairs.len(), 3);
    }
}
