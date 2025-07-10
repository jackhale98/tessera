use crate::{Id, Result, EntityInfo, EntityBrowser, EntitySelector, CrossModuleLink, LinkType};
use inquire::{Select, Text, Confirm};

pub struct LinkingHelper;

impl LinkingHelper {
    pub fn create_cross_module_link(
        browser: &dyn EntityBrowser,
        source_module: &str,
        source_entity_id: Id,
    ) -> Result<Option<CrossModuleLink>> {
        // Find the source entity
        let source_entity = browser.find_entity(source_entity_id)
            .ok_or_else(|| crate::DesignTrackError::NotFound(format!("Source entity {} not found", source_entity_id)))?;
        
        println!("Creating link from: {}", source_entity.display_name());
        
        // Select target entity
        let target_entity = match EntitySelector::select_entity(
            browser,
            "Select target entity to link to:",
            None, // No module filter - allow cross-module linking
            None, // No entity type filter
        )? {
            Some(entity) => entity,
            None => return Ok(None), // User cancelled
        };
        
        // Select link type
        let link_type = Self::select_link_type(&source_entity, &target_entity)?;
        
        // Optional description
        let description = Self::get_link_description()?;
        
        // Create the link
        let mut link = CrossModuleLink::new(
            source_module.to_string(),
            source_entity_id,
            target_entity.module.clone(),
            target_entity.id,
            link_type,
        );
        
        if let Some(desc) = description {
            link = link.with_description(desc);
        }
        
        Ok(Some(link))
    }
    
    pub fn select_entity_to_link(
        browser: &dyn EntityBrowser,
        prompt: &str,
        module_filter: Option<&str>,
        entity_type_filter: Option<&str>,
    ) -> Result<Option<EntityInfo>> {
        EntitySelector::select_entity(browser, prompt, module_filter, entity_type_filter)
    }
    
    pub fn select_multiple_entities_to_link(
        browser: &dyn EntityBrowser,
        prompt: &str,
        module_filter: Option<&str>,
        entity_type_filter: Option<&str>,
    ) -> Result<Vec<EntityInfo>> {
        EntitySelector::select_multiple_entities(browser, prompt, module_filter, entity_type_filter)
    }
    
    fn select_link_type(source: &EntityInfo, target: &EntityInfo) -> Result<LinkType> {
        let suggested_types = Self::suggest_link_types(source, target);
        
        let mut options = Vec::new();
        for link_type in suggested_types {
            options.push(format!("{:?}", link_type));
        }
        
        // Add common generic types
        options.extend(vec![
            "Reference".to_string(),
            "Verification".to_string(),
            "Compliance".to_string(),
            "Mitigation".to_string(),
            "Other".to_string(),
        ]);
        
        let selection = Select::new("Select link type:", options).prompt()?;
        
        let link_type = match selection.as_str() {
            "RequirementToComponent" => LinkType::RequirementToComponent,
            "RequirementToFeature" => LinkType::RequirementToFeature,
            "RequirementToStackup" => LinkType::RequirementToStackup,
            "RequirementToTask" => LinkType::RequirementToTask,
            "OutputToComponent" => LinkType::OutputToComponent,
            "OutputToFeature" => LinkType::OutputToFeature,
            "OutputToTask" => LinkType::OutputToTask,
            "RiskToStackup" => LinkType::RiskToStackup,
            "RiskToTask" => LinkType::RiskToTask,
            "TaskToComponent" => LinkType::TaskToComponent,
            "TaskToFeature" => LinkType::TaskToFeature,
            "TaskToStackup" => LinkType::TaskToStackup,
            "Reference" => LinkType::Reference,
            "Verification" => LinkType::Verification,
            "Compliance" => LinkType::Compliance,
            "Mitigation" => LinkType::Mitigation,
            "Other" => {
                let custom_type = Text::new("Enter custom link type:").prompt()?;
                LinkType::Other(custom_type)
            }
            _ => LinkType::Reference, // Default fallback
        };
        
        Ok(link_type)
    }
    
    fn suggest_link_types(source: &EntityInfo, target: &EntityInfo) -> Vec<LinkType> {
        let mut suggestions = Vec::new();
        
        match (source.entity_type.as_str(), target.entity_type.as_str()) {
            ("Requirement", "Component") => suggestions.push(LinkType::RequirementToComponent),
            ("Requirement", "Feature") => suggestions.push(LinkType::RequirementToFeature),
            ("Requirement", "Stackup") => suggestions.push(LinkType::RequirementToStackup),
            ("Requirement", "Task") => suggestions.push(LinkType::RequirementToTask),
            ("Design Output", "Component") => suggestions.push(LinkType::OutputToComponent),
            ("Design Output", "Feature") => suggestions.push(LinkType::OutputToFeature),
            ("Design Output", "Task") => suggestions.push(LinkType::OutputToTask),
            ("Risk", "Stackup") => suggestions.push(LinkType::RiskToStackup),
            ("Risk", "Task") => suggestions.push(LinkType::RiskToTask),
            ("Task", "Component") => suggestions.push(LinkType::TaskToComponent),
            ("Task", "Feature") => suggestions.push(LinkType::TaskToFeature),
            ("Task", "Stackup") => suggestions.push(LinkType::TaskToStackup),
            _ => {}
        }
        
        suggestions
    }
    
    fn get_link_description() -> Result<Option<String>> {
        let add_description = Confirm::new("Add description to link?")
            .with_default(false)
            .prompt()?;
        
        if add_description {
            let description = Text::new("Enter link description:")
                .with_help_message("Optional description explaining the relationship")
                .prompt()?;
            Ok(Some(description))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::EntityBrowser;
    
    struct MockEntityBrowser {
        entities: Vec<EntityInfo>,
    }
    
    impl MockEntityBrowser {
        fn new() -> Self {
            Self {
                entities: vec![
                    EntityInfo::new(
                        Id::new(),
                        "Req1".to_string(),
                        "quality".to_string(),
                        "Requirement".to_string()
                    ),
                    EntityInfo::new(
                        Id::new(),
                        "Comp1".to_string(),
                        "tol".to_string(),
                        "Component".to_string()
                    ),
                    EntityInfo::new(
                        Id::new(),
                        "Task1".to_string(),
                        "pm".to_string(),
                        "Task".to_string()
                    ),
                ],
            }
        }
    }
    
    impl EntityBrowser for MockEntityBrowser {
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
    
    #[test]
    fn test_suggest_link_types() {
        let req = EntityInfo::new(
            Id::new(),
            "Test Req".to_string(),
            "quality".to_string(),
            "Requirement".to_string()
        );
        let comp = EntityInfo::new(
            Id::new(),
            "Test Comp".to_string(),
            "tol".to_string(),
            "Component".to_string()
        );
        
        let suggestions = LinkingHelper::suggest_link_types(&req, &comp);
        assert_eq!(suggestions.len(), 1);
        assert!(matches!(suggestions[0], LinkType::RequirementToComponent));
    }
    
    #[test]
    fn test_mock_browser() {
        let browser = MockEntityBrowser::new();
        let all_entities = browser.get_all_entities();
        assert_eq!(all_entities.len(), 3);
        
        let quality_entities = browser.get_entities_by_module("quality");
        assert_eq!(quality_entities.len(), 1);
        
        let requirements = browser.get_entities_by_type("Requirement");
        assert_eq!(requirements.len(), 1);
    }
}