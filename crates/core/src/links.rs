use crate::{Id, Result, DesignTrackError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModuleLink {
    pub id: Id,
    pub source_module: String,
    pub source_entity_id: Id,
    pub target_module: String,
    pub target_entity_id: Id,
    pub link_type: LinkType,
    pub description: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkType {
    // Quality to Tolerance links
    RequirementToComponent,
    RequirementToFeature,
    RequirementToStackup,
    OutputToComponent,
    OutputToFeature,
    RiskToStackup,
    
    // Quality to Project Management links
    RequirementToTask,
    OutputToTask,
    RiskToTask,
    
    // Project Management to Tolerance links
    TaskToComponent,
    TaskToFeature,
    TaskToStackup,
    
    // Generic links
    Reference,
    Verification,
    Compliance,
    Mitigation,
    Other(String),
}

impl CrossModuleLink {
    pub fn new(
        source_module: String,
        source_entity_id: Id,
        target_module: String,
        target_entity_id: Id,
        link_type: LinkType,
    ) -> Self {
        Self {
            id: Id::new(),
            source_module,
            source_entity_id,
            target_module,
            target_entity_id,
            link_type,
            description: None,
            created: chrono::Utc::now(),
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone)]
pub struct LinkRegistry {
    links: Vec<CrossModuleLink>,
    source_index: HashMap<(String, Id), Vec<usize>>, // (module, entity_id) -> link indices
    target_index: HashMap<(String, Id), Vec<usize>>, // (module, entity_id) -> link indices
}

impl LinkRegistry {
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            source_index: HashMap::new(),
            target_index: HashMap::new(),
        }
    }
    
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        let links: Vec<CrossModuleLink> = ron::from_str(&content)?;
        Ok(Self::from_links(links))
    }
    
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = crate::format_ron_pretty(&self.links)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn from_links(links: Vec<CrossModuleLink>) -> Self {
        let mut registry = Self::new();
        for link in links {
            registry.add_link(link);
        }
        registry
    }
    
    pub fn add_link(&mut self, link: CrossModuleLink) {
        let link_index = self.links.len();
        
        // Update source index
        let source_key = (link.source_module.clone(), link.source_entity_id);
        self.source_index.entry(source_key).or_insert_with(Vec::new).push(link_index);
        
        // Update target index
        let target_key = (link.target_module.clone(), link.target_entity_id);
        self.target_index.entry(target_key).or_insert_with(Vec::new).push(link_index);
        
        self.links.push(link);
    }
    
    pub fn remove_link(&mut self, link_id: Id) -> Result<()> {
        if let Some(pos) = self.links.iter().position(|link| link.id == link_id) {
            let _removed_link = self.links.remove(pos);
            
            // Rebuild indices (simple approach for now)
            self.rebuild_indices();
            
            Ok(())
        } else {
            Err(DesignTrackError::NotFound(format!("Link with id {} not found", link_id)))
        }
    }
    
    fn rebuild_indices(&mut self) {
        self.source_index.clear();
        self.target_index.clear();
        
        for (index, link) in self.links.iter().enumerate() {
            let source_key = (link.source_module.clone(), link.source_entity_id);
            self.source_index.entry(source_key).or_insert_with(Vec::new).push(index);
            
            let target_key = (link.target_module.clone(), link.target_entity_id);
            self.target_index.entry(target_key).or_insert_with(Vec::new).push(index);
        }
    }
    
    pub fn get_links_from(&self, module: &str, entity_id: Id) -> Vec<&CrossModuleLink> {
        let key = (module.to_string(), entity_id);
        self.source_index.get(&key)
            .map(|indices| indices.iter().map(|&i| &self.links[i]).collect())
            .unwrap_or_default()
    }
    
    pub fn get_links_to(&self, module: &str, entity_id: Id) -> Vec<&CrossModuleLink> {
        let key = (module.to_string(), entity_id);
        self.target_index.get(&key)
            .map(|indices| indices.iter().map(|&i| &self.links[i]).collect())
            .unwrap_or_default()
    }
    
    pub fn get_all_links(&self) -> &[CrossModuleLink] {
        &self.links
    }
    
    pub fn find_links_by_type(&self, link_type: &LinkType) -> Vec<&CrossModuleLink> {
        self.links.iter()
            .filter(|link| std::mem::discriminant(&link.link_type) == std::mem::discriminant(link_type))
            .collect()
    }
    
    pub fn validate_links(&self, validator: &dyn LinkValidator) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        
        for link in &self.links {
            if let Err(error) = validator.validate_link(link) {
                errors.push(format!("Link {}: {}", link.id, error));
            }
        }
        
        Ok(errors)
    }
}

pub trait LinkValidator {
    fn validate_link(&self, link: &CrossModuleLink) -> Result<()>;
}

pub struct BasicLinkValidator {
    valid_modules: std::collections::HashSet<String>,
}

impl BasicLinkValidator {
    pub fn new(valid_modules: Vec<String>) -> Self {
        Self {
            valid_modules: valid_modules.into_iter().collect(),
        }
    }
}

impl LinkValidator for BasicLinkValidator {
    fn validate_link(&self, link: &CrossModuleLink) -> Result<()> {
        if !self.valid_modules.contains(&link.source_module) {
            return Err(DesignTrackError::Validation(
                format!("Invalid source module: {}", link.source_module)
            ));
        }
        
        if !self.valid_modules.contains(&link.target_module) {
            return Err(DesignTrackError::Validation(
                format!("Invalid target module: {}", link.target_module)
            ));
        }
        
        if link.source_module == link.target_module && link.source_entity_id == link.target_entity_id {
            return Err(DesignTrackError::Validation(
                "Cannot link entity to itself".to_string()
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_link_registry() {
        let mut registry = LinkRegistry::new();
        
        let link = CrossModuleLink::new(
            "quality".to_string(),
            Id::new(),
            "tol".to_string(),
            Id::new(),
            LinkType::RequirementToComponent,
        );
        
        let source_id = link.source_entity_id;
        let target_id = link.target_entity_id;
        
        registry.add_link(link);
        
        let links_from = registry.get_links_from("quality", source_id);
        assert_eq!(links_from.len(), 1);
        
        let links_to = registry.get_links_to("tol", target_id);
        assert_eq!(links_to.len(), 1);
    }
}