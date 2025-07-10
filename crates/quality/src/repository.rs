use crate::data::*;
use tessera_core::{Entity, Id, Repository, Result};
use std::path::Path;

pub struct QualityRepository {
    requirements: Vec<Requirement>,
    inputs: Vec<DesignInput>,
    outputs: Vec<DesignOutput>,
    controls: Vec<DesignControl>,
    risks: Vec<Risk>,
}

impl QualityRepository {
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            controls: Vec::new(),
            risks: Vec::new(),
        }
    }
    
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let mut repo = Self::new();
        
        let requirements_file = dir.join("requirements.ron");
        if requirements_file.exists() {
            repo.requirements = Vec::<Requirement>::load_from_file(&requirements_file)?;
        }
        
        let inputs_file = dir.join("inputs.ron");
        if inputs_file.exists() {
            repo.inputs = Vec::<DesignInput>::load_from_file(&inputs_file)?;
        }
        
        let outputs_file = dir.join("outputs.ron");
        if outputs_file.exists() {
            repo.outputs = Vec::<DesignOutput>::load_from_file(&outputs_file)?;
        }
        
        let controls_file = dir.join("controls.ron");
        if controls_file.exists() {
            repo.controls = Vec::<DesignControl>::load_from_file(&controls_file)?;
        }
        
        let risks_file = dir.join("risks.ron");
        if risks_file.exists() {
            repo.risks = Vec::<Risk>::load_from_file(&risks_file)?;
        }
        
        Ok(repo)
    }
    
    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        tessera_core::ensure_directory(dir)?;
        
        Vec::<Requirement>::save_to_file(&self.requirements, dir.join("requirements.ron"))?;
        Vec::<DesignInput>::save_to_file(&self.inputs, dir.join("inputs.ron"))?;
        Vec::<DesignOutput>::save_to_file(&self.outputs, dir.join("outputs.ron"))?;
        Vec::<DesignControl>::save_to_file(&self.controls, dir.join("controls.ron"))?;
        Vec::<Risk>::save_to_file(&self.risks, dir.join("risks.ron"))?;
        
        Ok(())
    }
    
    pub fn add_requirement(&mut self, requirement: Requirement) -> Result<()> {
        requirement.validate()?;
        self.requirements.push(requirement);
        Ok(())
    }
    
    pub fn add_input(&mut self, input: DesignInput) -> Result<()> {
        input.validate()?;
        self.inputs.push(input);
        Ok(())
    }
    
    pub fn add_output(&mut self, output: DesignOutput) -> Result<()> {
        output.validate()?;
        self.outputs.push(output);
        Ok(())
    }
    
    pub fn add_control(&mut self, control: DesignControl) -> Result<()> {
        control.validate()?;
        self.controls.push(control);
        Ok(())
    }
    
    pub fn add_risk(&mut self, risk: Risk) -> Result<()> {
        risk.validate()?;
        self.risks.push(risk);
        Ok(())
    }
    
    pub fn get_requirements(&self) -> &[Requirement] {
        &self.requirements
    }
    
    pub fn get_inputs(&self) -> &[DesignInput] {
        &self.inputs
    }
    
    pub fn get_outputs(&self) -> &[DesignOutput] {
        &self.outputs
    }
    
    pub fn get_controls(&self) -> &[DesignControl] {
        &self.controls
    }
    
    pub fn get_risks(&self) -> &[Risk] {
        &self.risks
    }
    
    pub fn find_requirement_by_id(&self, id: Id) -> Option<&Requirement> {
        self.requirements.iter().find(|r| r.id == id)
    }
    
    pub fn find_input_by_id(&self, id: Id) -> Option<&DesignInput> {
        self.inputs.iter().find(|i| i.id == id)
    }
    
    pub fn find_output_by_id(&self, id: Id) -> Option<&DesignOutput> {
        self.outputs.iter().find(|o| o.id == id)
    }
    
    pub fn find_control_by_id(&self, id: Id) -> Option<&DesignControl> {
        self.controls.iter().find(|c| c.id == id)
    }
    
    pub fn find_risk_by_id(&self, id: Id) -> Option<&Risk> {
        self.risks.iter().find(|r| r.id == id)
    }
    
    pub fn update_requirement(&mut self, updated: Requirement) -> Result<()> {
        updated.validate()?;
        if let Some(pos) = self.requirements.iter().position(|r| r.id == updated.id) {
            self.requirements[pos] = updated;
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Requirement with id {} not found", updated.id)
            ))
        }
    }
    
    pub fn link_input_to_requirement(&mut self, input_id: Id, requirement_id: Id) -> Result<()> {
        if let Some(input) = self.inputs.iter_mut().find(|i| i.id == input_id) {
            if !input.requirements.contains(&requirement_id) {
                input.requirements.push(requirement_id);
                input.updated = chrono::Utc::now();
            }
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Input with id {} not found", input_id)
            ))
        }
    }
    
    pub fn link_output_to_requirement(&mut self, output_id: Id, requirement_id: Id) -> Result<()> {
        if let Some(output) = self.outputs.iter_mut().find(|o| o.id == output_id) {
            if !output.requirements.contains(&requirement_id) {
                output.requirements.push(requirement_id);
                output.updated = chrono::Utc::now();
            }
            Ok(())
        } else {
            Err(tessera_core::DesignTrackError::NotFound(
                format!("Output with id {} not found", output_id)
            ))
        }
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

impl Repository<Requirement> for Vec<Requirement> {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Requirement>> {
        load_items_from_file(path)
    }
    
    fn save_to_file<P: AsRef<Path>>(items: &[Requirement], path: P) -> Result<()> {
        save_items_to_file(items, path)
    }
    
    fn find_by_id(&self, id: Id) -> Option<&Requirement> {
        self.iter().find(|item| item.id() == id)
    }
    
    fn find_by_name(&self, name: &str) -> Option<&Requirement> {
        self.iter().find(|item| item.name() == name)
    }
    
    fn add(&mut self, item: Requirement) -> Result<()> {
        item.validate()?;
        self.push(item);
        Ok(())
    }
    
    fn update(&mut self, item: Requirement) -> Result<()> {
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
    
    fn list(&self) -> &[Requirement] {
        self
    }
}

impl Repository<DesignInput> for Vec<DesignInput> {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<DesignInput>> {
        load_items_from_file(path)
    }
    
    fn save_to_file<P: AsRef<Path>>(items: &[DesignInput], path: P) -> Result<()> {
        save_items_to_file(items, path)
    }
    
    fn find_by_id(&self, id: Id) -> Option<&DesignInput> {
        self.iter().find(|item| item.id() == id)
    }
    
    fn find_by_name(&self, name: &str) -> Option<&DesignInput> {
        self.iter().find(|item| item.name() == name)
    }
    
    fn add(&mut self, item: DesignInput) -> Result<()> {
        item.validate()?;
        self.push(item);
        Ok(())
    }
    
    fn update(&mut self, item: DesignInput) -> Result<()> {
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
    
    fn list(&self) -> &[DesignInput] {
        self
    }
}

impl Repository<DesignOutput> for Vec<DesignOutput> {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<DesignOutput>> {
        load_items_from_file(path)
    }
    
    fn save_to_file<P: AsRef<Path>>(items: &[DesignOutput], path: P) -> Result<()> {
        save_items_to_file(items, path)
    }
    
    fn find_by_id(&self, id: Id) -> Option<&DesignOutput> {
        self.iter().find(|item| item.id() == id)
    }
    
    fn find_by_name(&self, name: &str) -> Option<&DesignOutput> {
        self.iter().find(|item| item.name() == name)
    }
    
    fn add(&mut self, item: DesignOutput) -> Result<()> {
        item.validate()?;
        self.push(item);
        Ok(())
    }
    
    fn update(&mut self, item: DesignOutput) -> Result<()> {
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
    
    fn list(&self) -> &[DesignOutput] {
        self
    }
}

impl Repository<DesignControl> for Vec<DesignControl> {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<DesignControl>> {
        load_items_from_file(path)
    }
    
    fn save_to_file<P: AsRef<Path>>(items: &[DesignControl], path: P) -> Result<()> {
        save_items_to_file(items, path)
    }
    
    fn find_by_id(&self, id: Id) -> Option<&DesignControl> {
        self.iter().find(|item| item.id() == id)
    }
    
    fn find_by_name(&self, name: &str) -> Option<&DesignControl> {
        self.iter().find(|item| item.name() == name)
    }
    
    fn add(&mut self, item: DesignControl) -> Result<()> {
        item.validate()?;
        self.push(item);
        Ok(())
    }
    
    fn update(&mut self, item: DesignControl) -> Result<()> {
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
    
    fn list(&self) -> &[DesignControl] {
        self
    }
}

impl Repository<Risk> for Vec<Risk> {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Risk>> {
        load_items_from_file(path)
    }
    
    fn save_to_file<P: AsRef<Path>>(items: &[Risk], path: P) -> Result<()> {
        save_items_to_file(items, path)
    }
    
    fn find_by_id(&self, id: Id) -> Option<&Risk> {
        self.iter().find(|item| item.id() == id)
    }
    
    fn find_by_name(&self, name: &str) -> Option<&Risk> {
        self.iter().find(|item| item.name() == name)
    }
    
    fn add(&mut self, item: Risk) -> Result<()> {
        item.validate()?;
        self.push(item);
        Ok(())
    }
    
    fn update(&mut self, item: Risk) -> Result<()> {
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
    
    fn list(&self) -> &[Risk] {
        self
    }
}