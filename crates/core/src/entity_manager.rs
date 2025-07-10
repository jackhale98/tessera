use crate::{Id, Result, EntityInfo, EntityBrowser};
use std::path::Path;

pub struct EntityManager {
    entities: Vec<EntityInfo>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
    
    pub fn load_from_project_path<P: AsRef<Path>>(project_path: P) -> Result<Self> {
        let mut manager = Self::new();
        
        // Load entities from each module
        manager.load_quality_entities(&project_path)?;
        manager.load_pm_entities(&project_path)?;
        manager.load_tol_entities(&project_path)?;
        
        Ok(manager)
    }
    
    fn load_quality_entities<P: AsRef<Path>>(&mut self, project_path: P) -> Result<()> {
        let quality_path = project_path.as_ref().join("quality");
        
        // Load requirements
        if let Ok(entities) = self.load_entities_from_file(
            &quality_path.join("requirements.ron"),
            "quality",
            "Requirement"
        ) {
            self.entities.extend(entities);
        }
        
        // Load design inputs
        if let Ok(entities) = self.load_entities_from_file(
            &quality_path.join("inputs.ron"),
            "quality",
            "Design Input"
        ) {
            self.entities.extend(entities);
        }
        
        // Load design outputs
        if let Ok(entities) = self.load_entities_from_file(
            &quality_path.join("outputs.ron"),
            "quality",
            "Design Output"
        ) {
            self.entities.extend(entities);
        }
        
        // Load design controls
        if let Ok(entities) = self.load_entities_from_file(
            &quality_path.join("controls.ron"),
            "quality",
            "Design Control"
        ) {
            self.entities.extend(entities);
        }
        
        // Load risks
        if let Ok(entities) = self.load_entities_from_file(
            &quality_path.join("risks.ron"),
            "quality",
            "Risk"
        ) {
            self.entities.extend(entities);
        }
        
        Ok(())
    }
    
    fn load_pm_entities<P: AsRef<Path>>(&mut self, project_path: P) -> Result<()> {
        let pm_path = project_path.as_ref().join("pm");
        
        // Load tasks
        if let Ok(entities) = self.load_entities_from_file(
            &pm_path.join("tasks.ron"),
            "pm",
            "Task"
        ) {
            self.entities.extend(entities);
        }
        
        // Load resources
        if let Ok(entities) = self.load_entities_from_file(
            &pm_path.join("resources.ron"),
            "pm",
            "Resource"
        ) {
            self.entities.extend(entities);
        }
        
        // Load milestones
        if let Ok(entities) = self.load_entities_from_file(
            &pm_path.join("milestones.ron"),
            "pm",
            "Milestone"
        ) {
            self.entities.extend(entities);
        }
        
        // Load PM risks
        if let Ok(entities) = self.load_entities_from_file(
            &pm_path.join("pm_risks.ron"),
            "pm",
            "PM Risk"
        ) {
            self.entities.extend(entities);
        }
        
        // Load issues
        if let Ok(entities) = self.load_entities_from_file(
            &pm_path.join("issues.ron"),
            "pm",
            "Issue"
        ) {
            self.entities.extend(entities);
        }
        
        Ok(())
    }
    
    fn load_tol_entities<P: AsRef<Path>>(&mut self, project_path: P) -> Result<()> {
        let tol_path = project_path.as_ref().join("tol");
        
        // Load components
        if let Ok(entities) = self.load_entities_from_file(
            &tol_path.join("components.ron"),
            "tol",
            "Component"
        ) {
            self.entities.extend(entities);
        }
        
        // Load features
        if let Ok(entities) = self.load_entities_from_file(
            &tol_path.join("features.ron"),
            "tol",
            "Feature"
        ) {
            self.entities.extend(entities);
        }
        
        // Load stackups
        if let Ok(entities) = self.load_entities_from_file(
            &tol_path.join("stackups.ron"),
            "tol",
            "Stackup"
        ) {
            self.entities.extend(entities);
        }
        
        Ok(())
    }
    
    fn load_entities_from_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        module: &str,
        entity_type: &str
    ) -> Result<Vec<EntityInfo>> {
        let path = file_path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        
        // Parse the RON file to extract basic entity info
        // This is a simplified approach - in reality, we'd need to parse the actual entity types
        let entities = self.parse_ron_entities(&content, module, entity_type)?;
        
        Ok(entities)
    }
    
    fn parse_ron_entities(
        &self,
        _content: &str,
        _module: &str,
        _entity_type: &str
    ) -> Result<Vec<EntityInfo>> {
        // This is a simplified parser that extracts basic entity info
        // In a real implementation, we'd parse the actual entity types
        
        // For now, just return empty vec and let the modules handle their own entities
        // This can be enhanced later with proper RON parsing
        Ok(Vec::new())
    }
    
    pub fn add_entity(&mut self, entity: EntityInfo) {
        self.entities.push(entity);
    }
    
    pub fn refresh_from_project_path<P: AsRef<Path>>(&mut self, project_path: P) -> Result<()> {
        self.entities.clear();
        
        self.load_quality_entities(&project_path)?;
        self.load_pm_entities(&project_path)?;
        self.load_tol_entities(&project_path)?;
        
        Ok(())
    }
}

impl EntityBrowser for EntityManager {
    fn get_all_entities(&self) -> Vec<EntityInfo> {
        self.entities.clone()
    }
    
    fn get_entities_by_module(&self, module: &str) -> Vec<EntityInfo> {
        self.entities
            .iter()
            .filter(|e| e.module == module)
            .cloned()
            .collect()
    }
    
    fn get_entities_by_type(&self, entity_type: &str) -> Vec<EntityInfo> {
        self.entities
            .iter()
            .filter(|e| e.entity_type == entity_type)
            .cloned()
            .collect()
    }
    
    fn find_entity(&self, id: Id) -> Option<EntityInfo> {
        self.entities
            .iter()
            .find(|e| e.id == id)
            .cloned()
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_manager_creation() {
        let manager = EntityManager::new();
        assert_eq!(manager.get_all_entities().len(), 0);
    }
    
    #[test]
    fn test_entity_manager_add_entity() {
        let mut manager = EntityManager::new();
        let entity = EntityInfo::new(
            Id::new(),
            "Test Entity".to_string(),
            "test".to_string(),
            "TestType".to_string()
        );
        
        manager.add_entity(entity.clone());
        
        let all_entities = manager.get_all_entities();
        assert_eq!(all_entities.len(), 1);
        assert_eq!(all_entities[0].name, "Test Entity");
        
        let found = manager.find_entity(entity.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Entity");
    }
    
    #[test]
    fn test_entity_manager_filtering() {
        let mut manager = EntityManager::new();
        
        let entity1 = EntityInfo::new(
            Id::new(),
            "Entity 1".to_string(),
            "module1".to_string(),
            "Type1".to_string()
        );
        let entity2 = EntityInfo::new(
            Id::new(),
            "Entity 2".to_string(),
            "module2".to_string(),
            "Type1".to_string()
        );
        let entity3 = EntityInfo::new(
            Id::new(),
            "Entity 3".to_string(),
            "module1".to_string(),
            "Type2".to_string()
        );
        
        manager.add_entity(entity1);
        manager.add_entity(entity2);
        manager.add_entity(entity3);
        
        let module1_entities = manager.get_entities_by_module("module1");
        assert_eq!(module1_entities.len(), 2);
        
        let type1_entities = manager.get_entities_by_type("Type1");
        assert_eq!(type1_entities.len(), 2);
        
        let type2_entities = manager.get_entities_by_type("Type2");
        assert_eq!(type2_entities.len(), 1);
    }
}