use std::sync::Arc;
use uuid::Uuid;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{
    EntityType, EntityMetadata,
    Task, TaskType, SchedulingMode,
    Milestone, Resource, ResourceType, Calendar, Baseline,
};
use chrono::Utc;

/// Manages Task, Milestone, Resource, Calendar, and Baseline entities
pub struct TaskManager {
    storage: Arc<RonStorage>,
}

impl TaskManager {
    pub fn new(storage: Arc<RonStorage>) -> Self {
        Self { storage }
    }

    // ============================================================================
    // Task Methods
    // ============================================================================

    /// Create a new Task entity
    pub fn create_task(
        &self,
        name: String,
        description: String,
        scheduled_start: chrono::DateTime<Utc>,
        deadline: chrono::DateTime<Utc>,
        task_type: TaskType,
    ) -> EdtResult<Task> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Task name cannot be empty".to_string()));
        }

        if scheduled_start >= deadline {
            return Err(EdtError::ValidationError(
                "Scheduled start must be before deadline".to_string()
            ));
        }

        let metadata = EntityMetadata::new(EntityType::Task);

        let task = Task {
            metadata,
            name,
            description,
            notes: None,
            scheduled_start,
            deadline,
            actual_start: None,
            actual_end: None,
            task_type,
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

        // Write to storage
        self.storage.write_task(&task)?;

        Ok(task)
    }

    /// Get a Task by ID
    pub fn get_task(&self, id: &Uuid) -> EdtResult<Task> {
        if !self.storage.exists(&EntityType::Task, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_task(id)
    }

    /// Update a Task
    pub fn update_task(&self, task: Task) -> EdtResult<Task> {
        // Validate
        if task.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Task name cannot be empty".to_string()));
        }

        if task.scheduled_start >= task.deadline {
            return Err(EdtError::ValidationError(
                "Scheduled start must be before deadline".to_string()
            ));
        }

        // Update timestamp
        let mut updated_task = task;
        updated_task.metadata.updated_at = Utc::now();

        // Write to storage
        self.storage.write_task(&updated_task)?;

        Ok(updated_task)
    }

    /// Delete a Task
    pub fn delete_task(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Task, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Task, id)
    }

    /// List all Task IDs
    pub fn list_task_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Task)
    }

    // ============================================================================
    // Milestone Methods
    // ============================================================================

    /// Create a Milestone
    pub fn create_milestone(
        &self,
        name: String,
        description: String,
        date: chrono::DateTime<Utc>,
    ) -> EdtResult<Milestone> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Milestone name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Milestone);

        let milestone = Milestone {
            metadata,
            name,
            description,
            date,
            dependencies: vec![],
            is_critical_path: false,
        };

        self.storage.write_milestone(&milestone)?;

        Ok(milestone)
    }

    /// Get a Milestone by ID
    pub fn get_milestone(&self, id: &Uuid) -> EdtResult<Milestone> {
        if !self.storage.exists(&EntityType::Milestone, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_milestone(id)
    }

    /// Update a Milestone
    pub fn update_milestone(&self, milestone: Milestone) -> EdtResult<Milestone> {
        if milestone.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Milestone name cannot be empty".to_string()));
        }

        let mut updated = milestone;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_milestone(&updated)?;

        Ok(updated)
    }

    /// Delete a Milestone
    pub fn delete_milestone(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Milestone, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Milestone, id)
    }

    // ============================================================================
    // Resource Methods
    // ============================================================================

    /// Create a Resource
    pub fn create_resource(
        &self,
        name: String,
        description: String,
        resource_type: ResourceType,
    ) -> EdtResult<Resource> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Resource name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Resource);

        let resource = Resource {
            metadata,
            name,
            description,
            email: None,
            resource_type,
            bill_rate: None,
            calendar_id: None,
        };

        self.storage.write_resource(&resource)?;

        Ok(resource)
    }

    /// Get a Resource by ID
    pub fn get_resource(&self, id: &Uuid) -> EdtResult<Resource> {
        if !self.storage.exists(&EntityType::Resource, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_resource(id)
    }

    /// Update a Resource
    pub fn update_resource(&self, resource: Resource) -> EdtResult<Resource> {
        if resource.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Resource name cannot be empty".to_string()));
        }

        let mut updated = resource;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_resource(&updated)?;

        Ok(updated)
    }

    /// Delete a Resource
    pub fn delete_resource(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Resource, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Resource, id)
    }

    // ============================================================================
    // Calendar Methods
    // ============================================================================

    /// Create a Calendar
    pub fn create_calendar(
        &self,
        name: String,
        work_hours_per_day: f64,
        work_days: Vec<chrono::Weekday>,
    ) -> EdtResult<Calendar> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Calendar name cannot be empty".to_string()));
        }

        if work_hours_per_day <= 0.0 || work_hours_per_day > 24.0 {
            return Err(EdtError::ValidationError(
                "Work hours per day must be between 0 and 24".to_string()
            ));
        }

        if work_days.is_empty() {
            return Err(EdtError::ValidationError(
                "At least one work day must be specified".to_string()
            ));
        }

        let metadata = EntityMetadata::new(EntityType::Calendar);

        let calendar = Calendar {
            metadata,
            name,
            work_hours_per_day,
            work_days,
            holidays: vec![],
        };

        self.storage.write_calendar(&calendar)?;

        Ok(calendar)
    }

    /// Get a Calendar by ID
    pub fn get_calendar(&self, id: &Uuid) -> EdtResult<Calendar> {
        if !self.storage.exists(&EntityType::Calendar, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_calendar(id)
    }

    /// Update a Calendar
    pub fn update_calendar(&self, calendar: Calendar) -> EdtResult<Calendar> {
        if calendar.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Calendar name cannot be empty".to_string()));
        }

        if calendar.work_hours_per_day <= 0.0 || calendar.work_hours_per_day > 24.0 {
            return Err(EdtError::ValidationError(
                "Work hours per day must be between 0 and 24".to_string()
            ));
        }

        let mut updated = calendar;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_calendar(&updated)?;

        Ok(updated)
    }

    /// Delete a Calendar
    pub fn delete_calendar(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Calendar, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Calendar, id)
    }

    // ============================================================================
    // Baseline Methods
    // ============================================================================

    /// Create a Baseline
    pub fn create_baseline(
        &self,
        name: String,
        description: String,
        task_ids: Vec<Uuid>,
    ) -> EdtResult<Baseline> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Baseline name cannot be empty".to_string()));
        }

        let metadata = EntityMetadata::new(EntityType::Baseline);

        let baseline = Baseline {
            metadata,
            name,
            description,
            created_date: Utc::now(),
            task_ids,
        };

        self.storage.write_baseline(&baseline)?;

        Ok(baseline)
    }

    /// Get a Baseline by ID
    pub fn get_baseline(&self, id: &Uuid) -> EdtResult<Baseline> {
        if !self.storage.exists(&EntityType::Baseline, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_baseline(id)
    }

    /// Update a Baseline
    pub fn update_baseline(&self, baseline: Baseline) -> EdtResult<Baseline> {
        if baseline.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Baseline name cannot be empty".to_string()));
        }

        let mut updated = baseline;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_baseline(&updated)?;

        Ok(updated)
    }

    /// Delete a Baseline
    pub fn delete_baseline(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Baseline, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Baseline, id)
    }
}
