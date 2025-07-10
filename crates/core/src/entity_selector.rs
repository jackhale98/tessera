use crate::{Id, Result, traits::{EntityInfo, EntityBrowser}};
use inquire::{Select, InquireError};

pub struct EntitySelector;

impl EntitySelector {
    pub fn select_entity(
        browser: &dyn EntityBrowser,
        prompt: &str,
        module_filter: Option<&str>,
        entity_type_filter: Option<&str>,
    ) -> Result<Option<EntityInfo>> {
        let entities = match (module_filter, entity_type_filter) {
            (Some(module), Some(entity_type)) => {
                browser.get_entities_by_module(module)
                    .into_iter()
                    .filter(|e| e.entity_type == entity_type)
                    .collect()
            }
            (Some(module), None) => browser.get_entities_by_module(module),
            (None, Some(entity_type)) => browser.get_entities_by_type(entity_type),
            (None, None) => browser.get_all_entities(),
        };

        if entities.is_empty() {
            return Ok(None);
        }

        let display_names: Vec<String> = entities.iter()
            .map(|e| e.display_name())
            .collect();

        let selection = Select::new(prompt, display_names)
            .with_help_message("Use arrow keys to navigate, type to filter, Enter to select, Esc to cancel")
            .prompt();

        match selection {
            Ok(selected_display) => {
                let selected_entity = entities.into_iter()
                    .find(|e| e.display_name() == selected_display)
                    .expect("Selected entity should exist");
                Ok(Some(selected_entity))
            }
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(InquireError::OperationInterrupted) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn select_multiple_entities(
        browser: &dyn EntityBrowser,
        prompt: &str,
        module_filter: Option<&str>,
        entity_type_filter: Option<&str>,
    ) -> Result<Vec<EntityInfo>> {
        let entities = match (module_filter, entity_type_filter) {
            (Some(module), Some(entity_type)) => {
                browser.get_entities_by_module(module)
                    .into_iter()
                    .filter(|e| e.entity_type == entity_type)
                    .collect()
            }
            (Some(module), None) => browser.get_entities_by_module(module),
            (None, Some(entity_type)) => browser.get_entities_by_type(entity_type),
            (None, None) => browser.get_all_entities(),
        };

        if entities.is_empty() {
            return Ok(Vec::new());
        }

        let display_names: Vec<String> = entities.iter()
            .map(|e| e.display_name())
            .collect();

        let selection = inquire::MultiSelect::new(prompt, display_names)
            .with_help_message("Use arrow keys to navigate, Space to select/deselect, Enter to confirm, Esc to cancel")
            .prompt();

        match selection {
            Ok(selected_displays) => {
                let selected_entities = entities.into_iter()
                    .filter(|e| selected_displays.contains(&e.display_name()))
                    .collect();
                Ok(selected_entities)
            }
            Err(InquireError::OperationCanceled) => Ok(Vec::new()),
            Err(InquireError::OperationInterrupted) => Ok(Vec::new()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn select_entity_by_id(browser: &dyn EntityBrowser, id: Id) -> Option<EntityInfo> {
        browser.find_entity(id)
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
                    EntityInfo::new(Id::new(), "Req1".to_string(), "quality".to_string(), "Requirement".to_string())
                        .with_description("First requirement".to_string()),
                    EntityInfo::new(Id::new(), "Task1".to_string(), "pm".to_string(), "Task".to_string())
                        .with_description("First task".to_string()),
                    EntityInfo::new(Id::new(), "Comp1".to_string(), "tol".to_string(), "Component".to_string()),
                ],
            }
        }
    }

    impl EntityBrowser for MockEntityBrowser {
        fn get_all_entities(&self) -> Vec<EntityInfo> {
            self.entities.clone()
        }

        fn get_entities_by_module(&self, module: &str) -> Vec<EntityInfo> {
            self.entities.iter()
                .filter(|e| e.module == module)
                .cloned()
                .collect()
        }

        fn get_entities_by_type(&self, entity_type: &str) -> Vec<EntityInfo> {
            self.entities.iter()
                .filter(|e| e.entity_type == entity_type)
                .cloned()
                .collect()
        }

        fn find_entity(&self, id: Id) -> Option<EntityInfo> {
            self.entities.iter()
                .find(|e| e.id == id)
                .cloned()
        }
    }

    #[test]
    fn test_entity_info_display_name() {
        let entity = EntityInfo::new(Id::new(), "Test".to_string(), "module".to_string(), "Type".to_string());
        assert_eq!(entity.display_name(), "Type: Test");

        let entity_with_desc = entity.with_description("Test description".to_string());
        assert_eq!(entity_with_desc.display_name(), "Type: Test - Test description");
    }

    #[test]
    fn test_mock_browser_filtering() {
        let browser = MockEntityBrowser::new();
        
        let quality_entities = browser.get_entities_by_module("quality");
        assert_eq!(quality_entities.len(), 1);
        assert_eq!(quality_entities[0].name, "Req1");

        let requirement_entities = browser.get_entities_by_type("Requirement");
        assert_eq!(requirement_entities.len(), 1);
        assert_eq!(requirement_entities[0].name, "Req1");

        let all_entities = browser.get_all_entities();
        assert_eq!(all_entities.len(), 3);
    }
}