use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::EntityMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub notes: Option<String>,

    // Scheduling
    pub scheduled_start: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,

    // Task type
    pub task_type: TaskType,
    pub scheduling_mode: SchedulingMode,

    // Progress
    pub percent_complete: f64, // 0.0 to 1.0
    pub percent_complete_history: Vec<(DateTime<Utc>, f64)>,

    // Resources and cost
    pub assigned_resources: Vec<ResourceAssignment>,
    pub estimated_effort: Option<f64>, // hours
    pub actual_cost: Option<f64>,
    pub calculated_cost: Option<f64>,

    // Dependencies
    pub dependencies: Vec<TaskDependency>,

    // Critical path analysis results
    pub is_critical_path: bool,
    pub slack: Option<f64>, // days

    // Baseline data (added when baseline created)
    pub baseline_data: Option<TaskBaseline>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    EffortDriven,
    DurationDriven,
    WorkDriven,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SchedulingMode {
    Automatic,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAssignment {
    pub resource_id: Uuid,
    pub allocated_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub predecessor_id: Uuid,
    pub dependency_type: DependencyType,
    pub lag_days: f64, // can be negative for lead time
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    FinishToStart,  // FS
    StartToStart,   // SS
    FinishToFinish, // FF
    StartToFinish,  // SF
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBaseline {
    pub baseline_id: Uuid,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub effort: f64,
    pub cost: f64,
    pub percent_complete: f64,
    pub dependencies: Vec<TaskDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub date: DateTime<Utc>,
    pub dependencies: Vec<TaskDependency>,
    pub is_critical_path: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub email: Option<String>,
    pub resource_type: ResourceType,
    pub bill_rate: Option<f64>,
    pub calendar_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Labor,
    FlatCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub metadata: EntityMetadata,
    pub name: String,
    pub work_hours_per_day: f64,
    pub work_days: Vec<chrono::Weekday>,
    pub holidays: Vec<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub metadata: EntityMetadata,
    pub name: String,
    pub description: String,
    pub created_date: DateTime<Utc>,
    pub task_ids: Vec<Uuid>,  // Tasks included in baseline
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EntityType;

    #[test]
    fn test_task_creation() {
        let metadata = EntityMetadata::new(EntityType::Task);
        let start = Utc::now();
        let end = start + chrono::Duration::days(5);

        let task = Task {
            metadata: metadata.clone(),
            name: "Test Task".to_string(),
            description: "A test task".to_string(),
            notes: Some("Test notes".to_string()),
            scheduled_start: start,
            deadline: end,
            actual_start: None,
            actual_end: None,
            task_type: TaskType::EffortDriven,
            scheduling_mode: SchedulingMode::Automatic,
            percent_complete: 0.0,
            percent_complete_history: vec![],
            assigned_resources: vec![],
            estimated_effort: Some(40.0),
            actual_cost: None,
            calculated_cost: None,
            dependencies: vec![],
            is_critical_path: false,
            slack: None,
            baseline_data: None,
        };

        assert_eq!(task.name, "Test Task");
        assert_eq!(task.task_type, TaskType::EffortDriven);
        assert_eq!(task.percent_complete, 0.0);
        assert_eq!(task.metadata.entity_type, EntityType::Task);
    }

    #[test]
    fn test_task_with_dependencies() {
        let predecessor_id = Uuid::new_v4();
        let dependency = TaskDependency {
            predecessor_id,
            dependency_type: DependencyType::FinishToStart,
            lag_days: 0.0,
        };

        assert_eq!(dependency.dependency_type, DependencyType::FinishToStart);
        assert_eq!(dependency.lag_days, 0.0);
    }

    #[test]
    fn test_resource_assignment() {
        let resource_id = Uuid::new_v4();
        let assignment = ResourceAssignment {
            resource_id,
            allocated_hours: 8.0,
        };

        assert_eq!(assignment.resource_id, resource_id);
        assert_eq!(assignment.allocated_hours, 8.0);
    }

    #[test]
    fn test_task_type_variants() {
        assert_eq!(TaskType::EffortDriven, TaskType::EffortDriven);
        assert_ne!(TaskType::EffortDriven, TaskType::DurationDriven);
        assert_ne!(TaskType::DurationDriven, TaskType::WorkDriven);
    }

    #[test]
    fn test_dependency_type_variants() {
        assert_eq!(DependencyType::FinishToStart, DependencyType::FinishToStart);
        assert_ne!(DependencyType::FinishToStart, DependencyType::StartToStart);
    }

    #[test]
    fn test_milestone_creation() {
        let metadata = EntityMetadata::new(EntityType::Milestone);
        let date = Utc::now();

        let milestone = Milestone {
            metadata: metadata.clone(),
            name: "Project Kickoff".to_string(),
            description: "Start of project".to_string(),
            date,
            dependencies: vec![],
            is_critical_path: true,
        };

        assert_eq!(milestone.name, "Project Kickoff");
        assert_eq!(milestone.is_critical_path, true);
    }

    #[test]
    fn test_resource_creation() {
        let metadata = EntityMetadata::new(EntityType::Resource);

        let resource = Resource {
            metadata: metadata.clone(),
            name: "John Doe".to_string(),
            description: "Senior Engineer".to_string(),
            email: Some("john@example.com".to_string()),
            resource_type: ResourceType::Labor,
            bill_rate: Some(150.0),
            calendar_id: None,
        };

        assert_eq!(resource.name, "John Doe");
        assert_eq!(resource.resource_type, ResourceType::Labor);
        assert_eq!(resource.bill_rate, Some(150.0));
    }

    #[test]
    fn test_task_serialization() {
        let metadata = EntityMetadata::new(EntityType::Task);
        let start = Utc::now();
        let end = start + chrono::Duration::days(5);

        let task = Task {
            metadata,
            name: "Test Task".to_string(),
            description: "Description".to_string(),
            notes: None,
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

        // Serialize to RON
        let serialized = ron::to_string(&task).expect("Failed to serialize");
        assert!(serialized.contains("Test Task"));
        assert!(serialized.contains("EffortDriven"));

        // Deserialize back
        let deserialized: Task = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, task.name);
        assert_eq!(deserialized.task_type, task.task_type);
        assert_eq!(deserialized.percent_complete, task.percent_complete);
    }

    #[test]
    fn test_calendar_creation() {
        use chrono::Weekday;

        let metadata = EntityMetadata::new(EntityType::Calendar);

        let calendar = Calendar {
            metadata: metadata.clone(),
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

        assert_eq!(calendar.name, "Standard Work Week");
        assert_eq!(calendar.work_hours_per_day, 8.0);
        assert_eq!(calendar.work_days.len(), 5);
        assert!(calendar.work_days.contains(&Weekday::Mon));
        assert!(!calendar.work_days.contains(&Weekday::Sat));
    }

    #[test]
    fn test_calendar_with_holidays() {
        use chrono::{Weekday, NaiveDate};

        let metadata = EntityMetadata::new(EntityType::Calendar);

        let calendar = Calendar {
            metadata,
            name: "US Calendar".to_string(),
            work_hours_per_day: 8.0,
            work_days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            holidays: vec![
                NaiveDate::from_ymd_opt(2025, 12, 25).unwrap(), // Christmas
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),    // New Year
            ],
        };

        assert_eq!(calendar.holidays.len(), 2);
        assert!(calendar.holidays.contains(&NaiveDate::from_ymd_opt(2025, 12, 25).unwrap()));
    }

    #[test]
    fn test_calendar_serialization() {
        use chrono::Weekday;

        let metadata = EntityMetadata::new(EntityType::Calendar);

        let calendar = Calendar {
            metadata,
            name: "Test Calendar".to_string(),
            work_hours_per_day: 8.0,
            work_days: vec![Weekday::Mon, Weekday::Tue],
            holidays: vec![],
        };

        // Serialize to RON
        let serialized = ron::to_string(&calendar).expect("Failed to serialize");
        assert!(serialized.contains("Test Calendar"));

        // Deserialize back
        let deserialized: Calendar = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, calendar.name);
        assert_eq!(deserialized.work_hours_per_day, calendar.work_hours_per_day);
        assert_eq!(deserialized.work_days.len(), 2);
    }

    #[test]
    fn test_baseline_creation() {
        let metadata = EntityMetadata::new(EntityType::Baseline);
        let now = Utc::now();

        let task1_id = Uuid::new_v4();
        let task2_id = Uuid::new_v4();

        let baseline = Baseline {
            metadata: metadata.clone(),
            name: "Q1 2025 Baseline".to_string(),
            description: "Baseline for Q1 planning".to_string(),
            created_date: now,
            task_ids: vec![task1_id, task2_id],
        };

        assert_eq!(baseline.name, "Q1 2025 Baseline");
        assert_eq!(baseline.task_ids.len(), 2);
        assert!(baseline.task_ids.contains(&task1_id));
        assert!(baseline.task_ids.contains(&task2_id));
    }

    #[test]
    fn test_baseline_serialization() {
        let metadata = EntityMetadata::new(EntityType::Baseline);
        let now = Utc::now();

        let baseline = Baseline {
            metadata,
            name: "Test Baseline".to_string(),
            description: "Testing serialization".to_string(),
            created_date: now,
            task_ids: vec![Uuid::new_v4()],
        };

        // Serialize to RON
        let serialized = ron::to_string(&baseline).expect("Failed to serialize");
        assert!(serialized.contains("Test Baseline"));

        // Deserialize back
        let deserialized: Baseline = ron::from_str(&serialized)
            .expect("Failed to deserialize");
        assert_eq!(deserialized.name, baseline.name);
        assert_eq!(deserialized.description, baseline.description);
        assert_eq!(deserialized.task_ids.len(), 1);
    }
}
