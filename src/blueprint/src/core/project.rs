use crate::core::{Task, Resource, Milestone, Calendar, ResourceCalendar, ProjectBaseline, IssueRegistry, RiskRegistry, ProgressSnapshot};
use anyhow::Result;
use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub start_date: NaiveDate,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub resources: IndexMap<String, Resource>,
    #[serde(default)]
    pub tasks: IndexMap<String, Task>,
    #[serde(default)]
    pub milestones: IndexMap<String, Milestone>,
    #[serde(default)]
    pub calendars: IndexMap<String, Calendar>,
    #[serde(default)]
    pub resource_calendars: Vec<ResourceCalendar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_calendar_id: Option<String>,
    #[serde(default)]
    pub baselines: IndexMap<String, ProjectBaseline>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_baseline_id: Option<String>,
    #[serde(default)]
    pub progress_snapshots: IndexMap<String, ProgressSnapshot>,
    #[serde(default)]
    pub issue_registry: IssueRegistry,
    #[serde(default)]
    pub risk_registry: RiskRegistry,
}

fn default_currency() -> String {
    "USD".to_string()
}

impl Project {
    pub fn new(name: String, start_date: NaiveDate) -> Self {
        let mut project = Self {
            name,
            start_date,
            currency: default_currency(),
            description: None,
            resources: IndexMap::new(),
            tasks: IndexMap::new(),
            milestones: IndexMap::new(),
            calendars: IndexMap::new(),
            resource_calendars: Vec::new(),
            default_calendar_id: None,
            baselines: IndexMap::new(),
            current_baseline_id: None,
            progress_snapshots: IndexMap::new(),
            issue_registry: IssueRegistry::new(),
            risk_registry: RiskRegistry::new(),
        };
        
        // Add default calendar
        let default_calendar = Calendar::default();
        project.calendars.insert("default".to_string(), default_calendar);
        project.default_calendar_id = Some("default".to_string());
        
        project
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let project: Project = ron::from_str(&content)?;
        project.validate()?;
        Ok(project)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let content = ron::ser::to_string_pretty(self, pretty)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        // Validate task dependencies exist
        for (task_id, task) in &self.tasks {
            for dep in &task.dependencies {
                if !self.tasks.contains_key(&dep.task_id) {
                    anyhow::bail!("Task '{}' has unknown dependency '{}'", task_id, dep.task_id);
                }
            }

            // Validate resource assignments exist
            for (resource_id, _) in &task.resource_assignments {
                if !self.resources.contains_key(resource_id) {
                    anyhow::bail!("Task '{}' assigned to unknown resource '{}'", task_id, resource_id);
                }
            }
        }

        // Validate milestone dependencies
        for (milestone_id, milestone) in &self.milestones {
            for dep in &milestone.dependencies {
                if !self.tasks.contains_key(dep) && !self.milestones.contains_key(dep) {
                    anyhow::bail!("Milestone '{}' has unknown dependency '{}'", milestone_id, dep);
                }
            }
        }

        // Validate calendar references
        for resource_calendar in &self.resource_calendars {
            if !self.resources.contains_key(&resource_calendar.resource_id) {
                anyhow::bail!("Resource calendar references unknown resource '{}'", resource_calendar.resource_id);
            }
            if !self.calendars.contains_key(&resource_calendar.calendar_id) {
                anyhow::bail!("Resource calendar references unknown calendar '{}'", resource_calendar.calendar_id);
            }
        }

        // Validate default calendar exists
        if let Some(default_calendar_id) = &self.default_calendar_id {
            if !self.calendars.contains_key(default_calendar_id) {
                anyhow::bail!("Default calendar '{}' does not exist", default_calendar_id);
            }
        }

        Ok(())
    }

    pub fn add_task(&mut self, id: String, task: Task) {
        self.tasks.insert(id, task);
    }

    pub fn add_resource(&mut self, id: String, resource: Resource) {
        self.resources.insert(id, resource);
    }

    pub fn add_milestone(&mut self, id: String, milestone: Milestone) {
        self.milestones.insert(id, milestone);
    }

    pub fn add_calendar(&mut self, id: String, calendar: Calendar) {
        self.calendars.insert(id, calendar);
    }

    pub fn add_resource_calendar(&mut self, resource_calendar: ResourceCalendar) {
        self.resource_calendars.push(resource_calendar);
    }

    pub fn get_calendar_for_resource(&self, resource_id: &str) -> Option<&Calendar> {
        // Check if resource has a specific calendar assigned
        for resource_calendar in &self.resource_calendars {
            if resource_calendar.resource_id == resource_id {
                return self.calendars.get(&resource_calendar.calendar_id);
            }
        }
        
        // Fall back to default calendar
        if let Some(default_calendar_id) = &self.default_calendar_id {
            return self.calendars.get(default_calendar_id);
        }
        
        None
    }

    pub fn get_default_calendar(&self) -> Option<&Calendar> {
        if let Some(default_calendar_id) = &self.default_calendar_id {
            self.calendars.get(default_calendar_id)
        } else {
            None
        }
    }

    pub fn set_default_calendar(&mut self, calendar_id: String) {
        self.default_calendar_id = Some(calendar_id);
    }

    // Baseline management
    pub fn create_baseline(
        &mut self,
        baseline_id: String,
        name: String,
        created_by: String,
        baseline_type: crate::core::BaselineType,
        schedule: &crate::scheduling::Schedule,
    ) -> Result<()> {
        let baseline = ProjectBaseline::new(
            uuid::Uuid::parse_str(&baseline_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            name,
            created_by,
            baseline_type,
            self,
            schedule,
        );

        // If this is set as current, unset any existing current baseline
        if baseline.is_current {
            for existing_baseline in self.baselines.values_mut() {
                existing_baseline.is_current = false;
            }
            self.current_baseline_id = Some(baseline_id.clone());
        }

        self.baselines.insert(baseline_id, baseline);
        Ok(())
    }

    pub fn get_current_baseline(&self) -> Option<&ProjectBaseline> {
        if let Some(baseline_id) = &self.current_baseline_id {
            self.baselines.get(baseline_id)
        } else {
            None
        }
    }

    pub fn set_current_baseline(&mut self, baseline_id: String) -> Result<()> {
        if !self.baselines.contains_key(&baseline_id) {
            anyhow::bail!("Baseline '{}' does not exist", baseline_id);
        }

        // Unset current flag on existing baselines
        for baseline in self.baselines.values_mut() {
            baseline.is_current = false;
        }

        // Set new current baseline
        if let Some(baseline) = self.baselines.get_mut(&baseline_id) {
            baseline.set_as_current();
            self.current_baseline_id = Some(baseline_id);
        }

        Ok(())
    }

    // Progress tracking
    pub fn record_progress_snapshot(
        &mut self,
        snapshot_id: String,
        snapshot: ProgressSnapshot,
    ) {
        self.progress_snapshots.insert(snapshot_id, snapshot);
    }

    pub fn get_latest_progress_snapshot(&self) -> Option<&ProgressSnapshot> {
        self.progress_snapshots.values()
            .max_by_key(|snapshot| snapshot.status_date)
    }

    pub fn calculate_earned_value(
        &self,
        status_date: chrono::NaiveDate,
    ) -> Option<crate::core::EarnedValueMetrics> {
        if let (Some(baseline), Some(progress)) = (self.get_current_baseline(), self.get_latest_progress_snapshot()) {
            Some(crate::core::EarnedValueMetrics::calculate(
                status_date,
                self.name.clone(),
                baseline,
                progress,
            ))
        } else {
            None
        }
    }

    // Issue management
    pub fn create_issue(&mut self, title: String, description: String, reported_by: String) -> String {
        self.issue_registry.create_issue(title, description, reported_by).to_string()
    }

    pub fn get_critical_issues(&self) -> Vec<&crate::core::Issue> {
        self.issue_registry.get_critical_issues()
    }

    pub fn get_overdue_issues(&self) -> Vec<&crate::core::Issue> {
        self.issue_registry.get_overdue_issues()
    }

    // Risk management
    pub fn create_risk(&mut self, title: String, description: String, identified_by: String) -> String {
        self.risk_registry.create_risk(title, description, identified_by).to_string()
    }

    pub fn get_high_priority_risks(&self) -> Vec<&crate::core::Risk> {
        self.risk_registry.get_high_priority_risks()
    }

    pub fn get_risks_requiring_review(&self) -> Vec<&crate::core::Risk> {
        self.risk_registry.get_risks_requiring_review()
    }

    // Project health assessment
    pub fn assess_project_health(&self) -> crate::core::ProjectStatus {
        let critical_issues = self.get_critical_issues().len();
        let high_priority_risks = self.get_high_priority_risks().len();
        let overdue_issues = self.get_overdue_issues().len();

        // Get earned value health if available
        let ev_health = if let Some(ev_metrics) = self.calculate_earned_value(chrono::Utc::now().naive_utc().date()) {
            match (ev_metrics.schedule_health(), ev_metrics.cost_health()) {
                (crate::core::ProjectStatus::Red, _) | (_, crate::core::ProjectStatus::Red) => crate::core::ProjectStatus::Red,
                (crate::core::ProjectStatus::Yellow, _) | (_, crate::core::ProjectStatus::Yellow) => crate::core::ProjectStatus::Yellow,
                _ => crate::core::ProjectStatus::Green,
            }
        } else {
            crate::core::ProjectStatus::Green
        };

        // Combine factors for overall health
        if critical_issues > 0 || high_priority_risks > 2 || overdue_issues > 5 || ev_health == crate::core::ProjectStatus::Red {
            crate::core::ProjectStatus::Red
        } else if high_priority_risks > 0 || overdue_issues > 2 || ev_health == crate::core::ProjectStatus::Yellow {
            crate::core::ProjectStatus::Yellow
        } else {
            crate::core::ProjectStatus::Green
        }
    }
}

