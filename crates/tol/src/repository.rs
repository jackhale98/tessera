use crate::data::*;
use tessera_core::{Entity, Id, Repository, Result};
use std::path::Path;

pub struct ToleranceRepository {
    components: Vec<Component>,
    features: Vec<Feature>,
    mates: Vec<Mate>,
    stackups: Vec<Stackup>,
    analyses: Vec<StackupAnalysis>,
}

impl ToleranceRepository {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            features: Vec::new(),
            mates: Vec::new(),
            stackups: Vec::new(),
            analyses: Vec::new(),
        }
    }
    
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let mut repo = Self::new();
        
        let components_file = dir.join("components.ron");
        if components_file.exists() {
            repo.components = load_items_from_file(&components_file)?;
        }
        
        let features_file = dir.join("features.ron");
        if features_file.exists() {
            repo.features = load_items_from_file(&features_file)?;
        }
        
        let mates_file = dir.join("mates.ron");
        if mates_file.exists() {
            repo.mates = load_items_from_file(&mates_file)?;
        }
        
        let stackups_file = dir.join("stackups.ron");
        if stackups_file.exists() {
            repo.stackups = load_items_from_file(&stackups_file)?;
        }
        
        let analyses_file = dir.join("analyses.ron");
        if analyses_file.exists() {
            repo.analyses = load_items_from_file(&analyses_file)?;
        }
        
        Ok(repo)
    }
    
    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        tessera_core::ensure_directory(dir)?;
        
        save_items_to_file(&self.components, dir.join("components.ron"))?;
        save_items_to_file(&self.features, dir.join("features.ron"))?;
        save_items_to_file(&self.mates, dir.join("mates.ron"))?;
        save_items_to_file(&self.stackups, dir.join("stackups.ron"))?;
        save_items_to_file(&self.analyses, dir.join("analyses.ron"))?;
        
        Ok(())
    }
    
    // Component methods
    pub fn add_component(&mut self, component: Component) -> Result<()> {
        component.validate()?;
        self.components.push(component);
        Ok(())
    }
    
    pub fn get_components(&self) -> &[Component] {
        &self.components
    }
    
    pub fn find_component_by_id(&self, id: Id) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }
    
    // Feature methods
    pub fn add_feature(&mut self, feature: Feature) -> Result<()> {
        feature.validate()?;
        self.features.push(feature);
        Ok(())
    }
    
    pub fn get_features(&self) -> &[Feature] {
        &self.features
    }
    
    pub fn find_feature_by_id(&self, id: Id) -> Option<&Feature> {
        self.features.iter().find(|f| f.id == id)
    }
    
    pub fn get_features_for_component(&self, component_id: Id) -> Vec<&Feature> {
        self.features.iter().filter(|f| f.component_id == component_id).collect()
    }
    
    // Mate methods
    pub fn add_mate(&mut self, mate: Mate) -> Result<()> {
        mate.validate()?;
        self.mates.push(mate);
        Ok(())
    }
    
    pub fn get_mates(&self) -> &[Mate] {
        &self.mates
    }
    
    pub fn find_mate_by_id(&self, id: Id) -> Option<&Mate> {
        self.mates.iter().find(|m| m.id == id)
    }
    
    // Stackup methods
    pub fn add_stackup(&mut self, stackup: Stackup) -> Result<()> {
        stackup.validate()?;
        self.stackups.push(stackup);
        Ok(())
    }
    
    pub fn get_stackups(&self) -> &[Stackup] {
        &self.stackups
    }
    
    pub fn find_stackup_by_id(&self, id: Id) -> Option<&Stackup> {
        self.stackups.iter().find(|s| s.id == id)
    }
    
    pub fn update_stackup(&mut self, updated: Stackup) -> Result<()> {
        updated.validate()?;
        if let Some(pos) = self.stackups.iter().position(|s| s.id == updated.id) {
            self.stackups[pos] = updated;
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Stackup with id {} not found", updated.id)
            ))
        }
    }
    
    // Analysis methods
    pub fn add_analysis(&mut self, analysis: StackupAnalysis) -> Result<()> {
        self.analyses.push(analysis);
        Ok(())
    }
    
    pub fn get_analyses(&self) -> &[StackupAnalysis] {
        &self.analyses
    }
    
    pub fn get_analyses_for_stackup(&self, stackup_id: Id) -> Vec<&StackupAnalysis> {
        self.analyses.iter().filter(|a| a.stackup_id == stackup_id).collect()
    }
}

// Helper functions for loading/saving RON files
pub fn load_items_from_file<T, P>(path: P) -> Result<Vec<T>>
where
    T: for<'de> serde::Deserialize<'de>,
    P: AsRef<Path>,
{
    let content = std::fs::read_to_string(path)?;
    let items: Vec<T> = ron::from_str(&content)?;
    Ok(items)
}

pub fn save_items_to_file<T, P>(items: &[T], path: P) -> Result<()>
where
    T: serde::Serialize,
    P: AsRef<Path>,
{
    let content = tessera_core::format_ron_pretty(items)?;
    std::fs::write(path, content)?;
    Ok(())
}

// Repository trait implementations
impl Repository<Component> for Vec<Component> {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Component>> {
        load_items_from_file(path)
    }
    
    fn save_to_file<P: AsRef<Path>>(items: &[Component], path: P) -> Result<()> {
        save_items_to_file(items, path)
    }
    
    fn find_by_id(&self, id: Id) -> Option<&Component> {
        self.iter().find(|item| item.id() == id)
    }
    
    fn find_by_name(&self, name: &str) -> Option<&Component> {
        self.iter().find(|item| item.name() == name)
    }
    
    fn add(&mut self, item: Component) -> Result<()> {
        item.validate()?;
        self.push(item);
        Ok(())
    }
    
    fn update(&mut self, item: Component) -> Result<()> {
        item.validate()?;
        if let Some(pos) = self.iter().position(|existing| existing.id() == item.id()) {
            self[pos] = item;
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Item with id {} not found", item.id())
            ))
        }
    }
    
    fn remove(&mut self, id: Id) -> Result<()> {
        if let Some(pos) = self.iter().position(|item| item.id() == id) {
            self.remove(pos);
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Item with id {} not found", id)
            ))
        }
    }
    
    fn list(&self) -> &[Component] {
        self
    }
}