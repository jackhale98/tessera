use crate::{DesignTrackError, Id, Result, LinkRegistry, EntityInfo, EntityBrowser};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    pub risk_probability_range: RiskScoringConfig,
    pub risk_impact_range: RiskScoringConfig,
    pub risk_tolerance_thresholds: RiskToleranceThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskToleranceThresholds {
    pub bar_threshold: f64,    // Broadly Acceptable Risk - below this is acceptable
    pub afap_threshold: f64,   // As Far As Practicable - between BAR and this needs reduction
    pub int_threshold: f64,    // Intolerable - above this is unacceptable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScoringConfig {
    pub range: [i32; 3], // [start, end, step] e.g., [1, 5, 1] or [2, 10, 2]
}

impl Default for RiskScoringConfig {
    fn default() -> Self {
        Self {
            range: [1, 5, 1], // Default 1-5 range with step 1
        }
    }
}

impl RiskScoringConfig {
    pub fn new(start: i32, end: i32, step: i32) -> Self {
        Self {
            range: [start, end, step],
        }
    }
    
    pub fn from_vec(range_vec: Vec<i32>) -> Result<Self> {
        if range_vec.len() != 3 {
            return Err(crate::DesignTrackError::Validation(
                "Range must have exactly 3 values: [start, end, step]".to_string()
            ));
        }
        if range_vec[0] >= range_vec[1] {
            return Err(crate::DesignTrackError::Validation(
                "Start value must be less than end value".to_string()
            ));
        }
        if range_vec[2] <= 0 {
            return Err(crate::DesignTrackError::Validation(
                "Step must be greater than 0".to_string()
            ));
        }
        Ok(Self {
            range: [range_vec[0], range_vec[1], range_vec[2]],
        })
    }
    
    pub fn values(&self) -> Vec<i32> {
        (self.range[0]..=self.range[1])
            .step_by(self.range[2] as usize)
            .collect()
    }
    
    pub fn normalize_to_0_1(&self, value: i32) -> f64 {
        (value - self.range[0]) as f64 / (self.range[1] - self.range[0]) as f64
    }
}

impl Default for RiskToleranceThresholds {
    fn default() -> Self {
        Self {
            bar_threshold: 0.25,   // 25% of max possible score
            afap_threshold: 0.50,  // 50% of max possible score
            int_threshold: 0.75,   // 75% of max possible score
        }
    }
}

impl RiskToleranceThresholds {
    pub fn new(bar: f64, afap: f64, int: f64) -> Result<Self> {
        if bar >= afap || afap >= int || int > 1.0 || bar < 0.0 {
            return Err(crate::DesignTrackError::Validation(
                "Risk thresholds must be: 0.0 ≤ BAR < AFAP < INT ≤ 1.0".to_string()
            ));
        }
        Ok(Self {
            bar_threshold: bar,
            afap_threshold: afap,
            int_threshold: int,
        })
    }
    
    pub fn categorize_risk(&self, normalized_score: f64) -> RiskCategory {
        if normalized_score < self.bar_threshold {
            RiskCategory::BroadlyAcceptable
        } else if normalized_score < self.afap_threshold {
            RiskCategory::TolerableWithReduction
        } else if normalized_score < self.int_threshold {
            RiskCategory::AsFarAsPracticable
        } else {
            RiskCategory::Intolerable
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskCategory {
    BroadlyAcceptable,      // BAR - Below BAR threshold
    TolerableWithReduction, // Between BAR and AFAP - should be reduced where reasonably practicable
    AsFarAsPracticable,     // Between AFAP and INT - must be reduced as far as practicable
    Intolerable,            // Above INT - not acceptable
}

impl std::fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskCategory::BroadlyAcceptable => write!(f, "BAR (Broadly Acceptable)"),
            RiskCategory::TolerableWithReduction => write!(f, "Tolerable (with reduction)"),
            RiskCategory::AsFarAsPracticable => write!(f, "AFAP (As Far As Practicable)"),
            RiskCategory::Intolerable => write!(f, "INT (Intolerable)"),
        }
    }
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            risk_probability_range: RiskScoringConfig::default(),
            risk_impact_range: RiskScoringConfig::default(),
            risk_tolerance_thresholds: RiskToleranceThresholds::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub created: DateTime<Utc>,
    pub modules: Vec<String>,
    pub git_repo: Option<String>,
    pub metadata: IndexMap<String, String>,
    pub quality_settings: QualitySettings,
}

impl ProjectMetadata {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: format!("project-{}", Id::new()),
            name,
            version: "0.1.0".to_string(),
            description,
            created: Utc::now(),
            modules: vec!["pm".to_string(), "quality".to_string(), "tol".to_string()],
            git_repo: None,
            metadata: IndexMap::new(),
            quality_settings: QualitySettings::default(),
        }
    }
    
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        
        // Try to load with current structure first
        match ron::from_str::<ProjectMetadata>(&content) {
            Ok(metadata) => Ok(metadata),
            Err(_) => {
                // If loading fails, try loading with partial structure and add defaults
                // This handles migration from older project files
                #[derive(Deserialize)]
                struct PartialProjectMetadata {
                    id: String,
                    name: String,
                    version: String,
                    description: String,
                    created: DateTime<Utc>,
                    modules: Vec<String>,
                    git_repo: Option<String>,
                    metadata: IndexMap<String, String>,
                }
                
                let partial: PartialProjectMetadata = ron::from_str(&content)?;
                Ok(ProjectMetadata {
                    id: partial.id,
                    name: partial.name,
                    version: partial.version,
                    description: partial.description,
                    created: partial.created,
                    modules: partial.modules,
                    git_repo: partial.git_repo,
                    metadata: partial.metadata,
                    quality_settings: QualitySettings::default(), // Add default quality settings
                })
            }
        }
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub metadata: ProjectMetadata,
    pub root_path: PathBuf,
    pub current_module: Option<String>,
    pub link_registry: LinkRegistry,
}

impl ProjectContext {
    pub fn new(metadata: ProjectMetadata, root_path: PathBuf) -> Self {
        Self {
            metadata,
            root_path,
            current_module: None,
            link_registry: LinkRegistry::new(),
        }
    }
    
    pub fn load_from_workspace<P: AsRef<Path>>(workspace_path: P) -> Result<Self> {
        let root_path = workspace_path.as_ref().to_path_buf();
        let project_file = root_path.join("project.ron");
        
        if !project_file.exists() {
            return Err(DesignTrackError::NotFound(
                "project.ron not found in workspace".to_string()
            ));
        }
        
        let metadata = ProjectMetadata::load_from_file(project_file)?;
        let mut context = Self::new(metadata, root_path.clone());
        
        // Load link registry
        let links_file = root_path.join("links.ron");
        context.link_registry = LinkRegistry::load_from_file(links_file)?;
        
        Ok(context)
    }
    
    pub fn module_path(&self, module: &str) -> PathBuf {
        self.root_path.join(module)
    }
    
    pub fn ensure_module_dirs(&self) -> Result<()> {
        for module in &self.metadata.modules {
            let module_path = self.module_path(module);
            std::fs::create_dir_all(&module_path)?;
        }
        Ok(())
    }
    
    pub fn set_current_module(&mut self, module: String) {
        self.current_module = Some(module);
    }
    
    pub fn save_links(&self) -> Result<()> {
        let links_file = self.root_path.join("links.ron");
        self.link_registry.save_to_file(links_file)?;
        Ok(())
    }
    
    pub fn add_link(&mut self, link: crate::CrossModuleLink) -> Result<()> {
        self.link_registry.add_link(link);
        self.save_links()?;
        Ok(())
    }
    
    pub fn get_links_from(&self, module: &str, entity_id: Id) -> Vec<&crate::CrossModuleLink> {
        self.link_registry.get_links_from(module, entity_id)
    }
    
    pub fn get_links_to(&self, module: &str, entity_id: Id) -> Vec<&crate::CrossModuleLink> {
        self.link_registry.get_links_to(module, entity_id)
    }
}

impl EntityBrowser for ProjectContext {
    fn get_all_entities(&self) -> Vec<EntityInfo> {
        let mut entities = Vec::new();
        
        // For now, we'll implement a basic version that works with the existing system
        // This can be enhanced once we have access to the module repositories
        
        // Add placeholder entities for testing
        entities.extend(self.get_entities_by_module("quality"));
        entities.extend(self.get_entities_by_module("pm"));
        entities.extend(self.get_entities_by_module("tol"));
        
        entities
    }
    
    fn get_entities_by_module(&self, _module: &str) -> Vec<EntityInfo> {
        // For now, return empty vec. This will be implemented when we have access to module repositories
        // The actual implementation will need to load each module's repository and extract entity info
        Vec::new()
    }
    
    fn get_entities_by_type(&self, entity_type: &str) -> Vec<EntityInfo> {
        self.get_all_entities()
            .into_iter()
            .filter(|e| e.entity_type == entity_type)
            .collect()
    }
    
    fn find_entity(&self, id: Id) -> Option<EntityInfo> {
        self.get_all_entities()
            .into_iter()
            .find(|e| e.id == id)
    }
}