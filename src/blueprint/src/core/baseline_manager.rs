use crate::core::{Project, baseline::*};
use crate::scheduling::Schedule;
use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};
use indexmap::IndexMap;
use chrono::{NaiveDateTime, Utc};

/// Manages baseline storage and operations following PM best practices
/// - Stores baselines in separate TOML files for git-friendly tracking
/// - Supports multiple baseline types (Initial, Approved, Working, Archived)
/// - Provides comprehensive baseline comparison and variance analysis
/// - Follows project management standards for baselining
pub struct BaselineManager {
    project_path: PathBuf,
    baselines_dir: PathBuf,
}

impl BaselineManager {
    pub fn new(project_path: PathBuf) -> Self {
        let baselines_dir = project_path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join("baselines");
        
        Self {
            project_path,
            baselines_dir,
        }
    }

    /// Create baselines directory if it doesn't exist
    pub fn ensure_baselines_directory(&self) -> Result<()> {
        if !self.baselines_dir.exists() {
            fs::create_dir_all(&self.baselines_dir)
                .context("Failed to create baselines directory")?;
        }
        Ok(())
    }

    /// Create a new baseline from current project and schedule
    pub fn create_baseline(
        &self,
        project: &Project,
        schedule: &Schedule,
        baseline_type: BaselineType,
        name: String,
        description: Option<String>,
        created_by: String,
    ) -> Result<ProjectBaseline> {
        self.ensure_baselines_directory()?;

        // Generate baseline ID with timestamp for uniqueness
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let baseline_id = uuid::Uuid::new_v4();

        let mut baseline = ProjectBaseline::new(
            baseline_id,
            name,
            created_by,
            baseline_type,
            project,
            schedule,
        );

        if let Some(desc) = description {
            baseline = baseline.with_description(desc);
        }

        // Handle baseline type logic
        match baseline_type {
            BaselineType::Initial => {
                // Archive any existing working baselines
                self.archive_working_baselines()?;
                baseline.set_as_current();
            }
            BaselineType::Approved => {
                // Archive any existing working baselines
                self.archive_working_baselines()?;
                baseline.set_as_current();
            }
            BaselineType::Working => {
                // Archive any existing working baselines first
                self.archive_working_baselines()?;
                baseline.set_as_current();
            }
            BaselineType::Archived => {
                // Already archived by default
            }
        }

        // Save baseline to file
        self.save_baseline(&baseline)?;

        // Update project with baseline reference
        self.update_project_baseline_references(project, &baseline)?;

        Ok(baseline)
    }

    /// Save baseline to a separate RON file
    pub fn save_baseline(&self, baseline: &ProjectBaseline) -> Result<()> {
        let file_path = self.baselines_dir.join(format!("{}.ron", baseline.baseline_id));
        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let content = ron::ser::to_string_pretty(baseline, pretty)
            .context("Failed to serialize baseline to RON")?;
        
        fs::write(&file_path, content)
            .context(format!("Failed to write baseline to {}", file_path.display()))?;
        
        Ok(())
    }

    /// Load a specific baseline by ID
    pub fn load_baseline(&self, baseline_id: &str) -> Result<ProjectBaseline> {
        let file_path = self.baselines_dir.join(format!("{}.ron", baseline_id));
        let content = fs::read_to_string(&file_path)
            .context(format!("Failed to read baseline file {}", file_path.display()))?;
        
        let baseline: ProjectBaseline = ron::from_str(&content)
            .context("Failed to parse baseline RON")?;
        
        Ok(baseline)
    }

    /// List all available baselines
    pub fn list_baselines(&self) -> Result<Vec<BaselineInfo>> {
        if !self.baselines_dir.exists() {
            return Ok(Vec::new());
        }

        let mut baselines = Vec::new();
        
        for entry in fs::read_dir(&self.baselines_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Some(baseline_id) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_baseline(baseline_id) {
                        Ok(baseline) => {
                            baselines.push(BaselineInfo {
                                baseline_id: baseline.baseline_id.to_string(),
                                name: baseline.name,
                                baseline_type: baseline.baseline_type,
                                created_date: baseline.created_date,
                                created_by: baseline.created_by,
                                is_current: baseline.is_current,
                                project_end_date: baseline.project_snapshot.end_date,
                                total_cost: baseline.project_snapshot.total_cost,
                            });
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load baseline {}: {}", baseline_id, e);
                        }
                    }
                }
            }
        }

        // Sort by creation date, most recent first
        baselines.sort_by(|a, b| b.created_date.cmp(&a.created_date));
        
        Ok(baselines)
    }

    /// Get the current active baseline
    pub fn get_current_baseline(&self) -> Result<Option<ProjectBaseline>> {
        let baselines = self.list_baselines()?;
        
        for baseline_info in baselines {
            if baseline_info.is_current {
                return Ok(Some(self.load_baseline(&baseline_info.baseline_id)?));
            }
        }
        
        Ok(None)
    }

    /// Compare two baselines and generate variance analysis
    pub fn compare_baselines(&self, current_id: &str, baseline_id: &str) -> Result<BaselineComparison> {
        let current = self.load_baseline(current_id)?;
        let baseline = self.load_baseline(baseline_id)?;
        
        Ok(current.compare_to(&baseline))
    }

    /// Set a baseline as the current active baseline
    pub fn set_current_baseline(&self, baseline_id: &str) -> Result<()> {
        // Archive all existing current baselines
        self.archive_working_baselines()?;
        
        // Load and update the specified baseline
        let mut baseline = self.load_baseline(baseline_id)?;
        baseline.set_as_current();
        
        self.save_baseline(&baseline)?;
        
        Ok(())
    }

    /// Archive a specific baseline
    pub fn archive_baseline(&self, baseline_id: &str) -> Result<()> {
        let mut baseline = self.load_baseline(baseline_id)?;
        baseline.archive();
        self.save_baseline(&baseline)?;
        Ok(())
    }

    /// Archive all working baselines (used when creating new baseline)
    fn archive_working_baselines(&self) -> Result<()> {
        let baselines = self.list_baselines()?;
        
        for baseline_info in baselines {
            if baseline_info.is_current {
                let mut baseline = self.load_baseline(&baseline_info.baseline_id)?;
                baseline.archive();
                self.save_baseline(&baseline)?;
            }
        }
        
        Ok(())
    }

    /// Delete a baseline (with safety checks)
    pub fn delete_baseline(&self, baseline_id: &str) -> Result<()> {
        let baseline = self.load_baseline(baseline_id)?;
        
        // Safety check: don't delete the current baseline
        if baseline.is_current {
            return Err(anyhow::anyhow!("Cannot delete the current active baseline. Set another baseline as current first."));
        }

        // Safety check: don't delete Initial baseline
        if baseline.baseline_type == BaselineType::Initial {
            return Err(anyhow::anyhow!("Cannot delete Initial baseline. Archive it instead."));
        }

        let file_path = self.baselines_dir.join(format!("{}.ron", baseline_id));
        fs::remove_file(&file_path)
            .context(format!("Failed to delete baseline file {}", file_path.display()))?;
        
        Ok(())
    }

    /// Update project file with baseline references
    fn update_project_baseline_references(&self, project: &Project, baseline: &ProjectBaseline) -> Result<()> {
        // This would update the main project.ron file to reference the new baseline
        // For now, we'll just track it in the project's baselines map
        // In a full implementation, this would save the updated project file
        
        // The project structure already has baselines and current_baseline_id fields
        // This function would be used to persist those changes to the project.ron file
        
        Ok(())
    }

    /// Generate baseline metrics for reporting
    pub fn generate_baseline_metrics(&self, baseline_id: &str) -> Result<BaselineMetrics> {
        let baseline = self.load_baseline(baseline_id)?;
        
        let total_tasks = baseline.project_snapshot.tasks.len();
        let total_milestones = baseline.project_snapshot.milestones.len();
        let total_resources = baseline.project_snapshot.resources.len();
        
        let duration_days = (baseline.project_snapshot.end_date - baseline.project_snapshot.start_date).num_days();
        
        // Calculate resource utilization
        let avg_resource_utilization = if total_resources > 0 {
            baseline.project_snapshot.resources.values()
                .map(|r| r.total_allocated_hours)
                .sum::<f32>() / total_resources as f32
        } else {
            0.0
        };

        Ok(BaselineMetrics {
            baseline_id: baseline.baseline_id.to_string(),
            name: baseline.name,
            total_tasks,
            total_milestones,
            total_resources,
            duration_days: duration_days as u32,
            total_cost: baseline.project_snapshot.total_cost,
            total_effort_hours: baseline.project_snapshot.total_effort_hours,
            avg_resource_utilization,
            created_date: baseline.created_date,
            baseline_type: baseline.baseline_type,
        })
    }
}

/// Summary information about a baseline for listing
#[derive(Debug, Clone)]
pub struct BaselineInfo {
    pub baseline_id: String,
    pub name: String,
    pub baseline_type: BaselineType,
    pub created_date: NaiveDateTime,
    pub created_by: String,
    pub is_current: bool,
    pub project_end_date: chrono::NaiveDate,
    pub total_cost: f32,
}

/// Metrics for baseline analysis and reporting
#[derive(Debug, Clone)]
pub struct BaselineMetrics {
    pub baseline_id: String,
    pub name: String,
    pub total_tasks: usize,
    pub total_milestones: usize,
    pub total_resources: usize,
    pub duration_days: u32,
    pub total_cost: f32,
    pub total_effort_hours: f32,
    pub avg_resource_utilization: f32,
    pub created_date: NaiveDateTime,
    pub baseline_type: BaselineType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::NaiveDate;
    use indexmap::IndexMap;
    use std::collections::HashMap;

    fn create_test_project() -> Project {
        Project::new("Test Project".to_string(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
    }

    fn create_test_schedule() -> Schedule {
        Schedule {
            project_name: "Test Project".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            tasks: IndexMap::new(),
            milestones: IndexMap::new(),
            critical_path: Vec::new(),
            total_cost: 1000.0,
            resource_utilization: HashMap::new(),
        }
    }

    #[test]
    fn test_baseline_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("project.ron");
        
        let manager = BaselineManager::new(project_path);
        assert!(manager.baselines_dir.ends_with("baselines"));
    }

    #[test]
    fn test_create_and_load_baseline() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("project.ron");
        
        let manager = BaselineManager::new(project_path);
        let project = create_test_project();
        let schedule = create_test_schedule();

        let baseline = manager.create_baseline(
            &project,
            &schedule,
            BaselineType::Initial,
            "Test Baseline".to_string(),
            Some("Initial project baseline".to_string()),
            "test_user".to_string(),
        ).unwrap();

        assert_eq!(baseline.name, "Test Baseline");
        assert_eq!(baseline.baseline_type, BaselineType::Initial);
        assert!(baseline.is_current);

        // Test loading the baseline
        let loaded = manager.load_baseline(&baseline.baseline_id.to_string()).unwrap();
        assert_eq!(loaded.name, baseline.name);
        assert_eq!(loaded.baseline_id, baseline.baseline_id);
    }

    #[test]
    fn test_list_baselines() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("project.ron");
        
        let manager = BaselineManager::new(project_path);
        let project = create_test_project();
        let schedule = create_test_schedule();

        // Create multiple baselines
        manager.create_baseline(
            &project,
            &schedule,
            BaselineType::Initial,
            "Initial".to_string(),
            None,
            "user1".to_string(),
        ).unwrap();

        manager.create_baseline(
            &project,
            &schedule,
            BaselineType::Working,
            "Working".to_string(),
            None,
            "user2".to_string(),
        ).unwrap();

        let baselines = manager.list_baselines().unwrap();
        assert_eq!(baselines.len(), 2);
        
        // Most recent should be first
        assert_eq!(baselines[0].name, "Working");
        assert_eq!(baselines[1].name, "Initial");
    }

    #[test]
    fn test_current_baseline_management() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("project.ron");
        
        let manager = BaselineManager::new(project_path);
        let project = create_test_project();
        let schedule = create_test_schedule();

        let baseline1 = manager.create_baseline(
            &project,
            &schedule,
            BaselineType::Initial,
            "Initial".to_string(),
            None,
            "user1".to_string(),
        ).unwrap();

        let baseline2 = manager.create_baseline(
            &project,
            &schedule,
            BaselineType::Working,
            "Working".to_string(),
            None,
            "user2".to_string(),
        ).unwrap();

        // Working should be current
        let current = manager.get_current_baseline().unwrap().unwrap();
        assert_eq!(current.baseline_id, baseline2.baseline_id);

        // Set initial as current
        manager.set_current_baseline(&baseline1.baseline_id.to_string()).unwrap();
        let current = manager.get_current_baseline().unwrap().unwrap();
        assert_eq!(current.baseline_id, baseline1.baseline_id);
    }
}