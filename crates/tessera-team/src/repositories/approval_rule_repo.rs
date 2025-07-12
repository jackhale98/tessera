use async_trait::async_trait;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tessera_core::{Repository, RepositoryError};

use crate::entities::{ApprovalRule, EntityState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRuleStorage {
    pub rules: IndexMap<String, ApprovalRule>,
}

impl Default for ApprovalRuleStorage {
    fn default() -> Self {
        Self {
            rules: IndexMap::new(),
        }
    }
}

pub struct ApprovalRuleRepository {
    path: PathBuf,
}

impl ApprovalRuleRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            path: base_path.join("team").join("approval_rules.ron"),
        }
    }

    async fn load_storage(&self) -> Result<ApprovalRuleStorage, RepositoryError> {
        if !self.path.exists() {
            return Ok(ApprovalRuleStorage::default());
        }

        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| RepositoryError::LoadError(e.to_string()))?;

        ron::from_str(&content)
            .map_err(|e| RepositoryError::ParseError(e.to_string()))
    }

    async fn save_storage(&self, storage: &ApprovalRuleStorage) -> Result<(), RepositoryError> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| RepositoryError::SaveError(e.to_string()))?;
        }

        let content = ron::ser::to_string_pretty(storage, ron::ser::PrettyConfig::default())
            .map_err(|e| RepositoryError::SaveError(e.to_string()))?;

        tokio::fs::write(&self.path, content)
            .await
            .map_err(|e| RepositoryError::SaveError(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl Repository<ApprovalRule> for ApprovalRuleRepository {
    async fn create(&self, entity: ApprovalRule) -> Result<ApprovalRule, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        // Validate the rule
        entity.validate()
            .map_err(|e| RepositoryError::ValidationError(e))?;
        
        storage.rules.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn get(&self, id: &str) -> Result<Option<ApprovalRule>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.rules.get(id).cloned())
    }

    async fn update(&self, entity: ApprovalRule) -> Result<ApprovalRule, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if !storage.rules.contains_key(&entity.id.to_string()) {
            return Err(RepositoryError::NotFoundError(
                format!("Approval rule with id {} not found", entity.id)
            ));
        }
        
        // Validate the rule
        entity.validate()
            .map_err(|e| RepositoryError::ValidationError(e))?;
        
        storage.rules.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if storage.rules.remove(id).is_none() {
            return Err(RepositoryError::NotFoundError(
                format!("Approval rule with id {} not found", id)
            ));
        }
        
        self.save_storage(&storage).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<ApprovalRule>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.rules.values().cloned().collect())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<ApprovalRule>, RepositoryError> {
        let storage = self.load_storage().await?;
        let name_lower = name.to_lowercase();
        
        Ok(storage.rules
            .values()
            .filter(|r| r.name.to_lowercase().contains(&name_lower))
            .cloned()
            .collect())
    }
}

#[async_trait]
pub trait ApprovalRuleRepositoryExt: Repository<ApprovalRule> {
    async fn get_rules_for_path(&self, path: &str) -> Result<Vec<ApprovalRule>, RepositoryError>;
    async fn get_rules_for_state(&self, state: EntityState) -> Result<Vec<ApprovalRule>, RepositoryError>;
    async fn get_rules_for_path_and_state(
        &self, 
        path: &str, 
        state: EntityState
    ) -> Result<Vec<ApprovalRule>, RepositoryError>;
}

#[async_trait]
impl ApprovalRuleRepositoryExt for ApprovalRuleRepository {
    async fn get_rules_for_path(&self, path: &str) -> Result<Vec<ApprovalRule>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.rules.values()
            .filter(|r| r.applies_to_path(path))
            .cloned()
            .collect())
    }

    async fn get_rules_for_state(&self, state: EntityState) -> Result<Vec<ApprovalRule>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.rules.values()
            .filter(|r| r.applies_to_state(state))
            .cloned()
            .collect())
    }

    async fn get_rules_for_path_and_state(
        &self, 
        path: &str, 
        state: EntityState
    ) -> Result<Vec<ApprovalRule>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.rules.values()
            .filter(|r| r.applies_to_path(path) && r.applies_to_state(state))
            .cloned()
            .collect())
    }
}