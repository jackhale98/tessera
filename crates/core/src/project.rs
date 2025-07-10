use crate::{DesignTrackError, Id, Result, LinkRegistry, EntityInfo, EntityBrowser};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
        }
    }
    
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let metadata: ProjectMetadata = ron::from_str(&content)?;
        Ok(metadata)
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