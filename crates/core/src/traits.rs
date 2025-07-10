use crate::{Id, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub trait Entity {
    fn id(&self) -> Id;
    fn name(&self) -> &str;
    fn validate(&self) -> Result<()>;
}

pub trait Repository<T: Entity> {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<T>>;
    fn save_to_file<P: AsRef<Path>>(items: &[T], path: P) -> Result<()>;
    fn find_by_id(&self, id: Id) -> Option<&T>;
    fn find_by_name(&self, name: &str) -> Option<&T>;
    fn add(&mut self, item: T) -> Result<()>;
    fn update(&mut self, item: T) -> Result<()>;
    fn remove(&mut self, id: Id) -> Result<()>;
    fn list(&self) -> &[T];
}

pub trait Linkable {
    fn get_links(&self) -> Vec<Id>;
    fn add_link(&mut self, target_id: Id) -> Result<()>;
    fn remove_link(&mut self, target_id: Id) -> Result<()>;
    fn validate_links(&self, resolver: &dyn LinkResolver) -> Result<()>;
}

pub trait LinkResolver {
    fn resolve(&self, id: Id) -> Result<String>;
    fn exists(&self, id: Id) -> bool;
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub id: Id,
    pub name: String,
    pub module: String,
    pub entity_type: String,
    pub description: Option<String>,
}

impl EntityInfo {
    pub fn new(id: Id, name: String, module: String, entity_type: String) -> Self {
        Self {
            id,
            name,
            module,
            entity_type,
            description: None,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn display_name(&self) -> String {
        match &self.description {
            Some(desc) => format!("{}: {} - {}", self.entity_type, self.name, desc),
            None => format!("{}: {}", self.entity_type, self.name),
        }
    }
}

pub trait EntityBrowser {
    fn get_all_entities(&self) -> Vec<EntityInfo>;
    fn get_entities_by_module(&self, module: &str) -> Vec<EntityInfo>;
    fn get_entities_by_type(&self, entity_type: &str) -> Vec<EntityInfo>;
    fn find_entity(&self, id: Id) -> Option<EntityInfo>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: Id,
    pub target_id: Id,
    pub link_type: String,
    pub relation_type: Option<String>,
    pub description: Option<String>,
}

impl Link {
    pub fn new(target_id: Id, link_type: String) -> Self {
        Self {
            id: Id::new(),
            target_id,
            link_type,
            relation_type: None,
            description: None,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}