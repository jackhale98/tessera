use async_trait::async_trait;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tessera_core::{Repository, RepositoryError};

use crate::entities::{Role, EntityState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleStorage {
    pub roles: IndexMap<String, Role>,
}

impl Default for RoleStorage {
    fn default() -> Self {
        Self {
            roles: IndexMap::new(),
        }
    }
}

pub struct RoleRepository {
    path: PathBuf,
}

impl RoleRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            path: base_path.join("team").join("roles.ron"),
        }
    }

    async fn load_storage(&self) -> Result<RoleStorage, RepositoryError> {
        if !self.path.exists() {
            return Ok(RoleStorage::default());
        }

        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| RepositoryError::LoadError(e.to_string()))?;

        ron::from_str(&content)
            .map_err(|e| RepositoryError::ParseError(e.to_string()))
    }

    async fn save_storage(&self, storage: &RoleStorage) -> Result<(), RepositoryError> {
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
impl Repository<Role> for RoleRepository {
    async fn create(&self, entity: Role) -> Result<Role, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        // Check for duplicate role name
        if storage.roles.values().any(|r| r.name == entity.name && r.id != entity.id) {
            return Err(RepositoryError::ValidationError(
                format!("Role '{}' already exists", entity.name)
            ));
        }
        
        storage.roles.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn get(&self, id: &str) -> Result<Option<Role>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.roles.get(id).cloned())
    }

    async fn update(&self, entity: Role) -> Result<Role, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if !storage.roles.contains_key(&entity.id.to_string()) {
            return Err(RepositoryError::NotFoundError(
                format!("Role with id {} not found", entity.id)
            ));
        }
        
        // Check for duplicate role name
        if storage.roles.values().any(|r| r.name == entity.name && r.id != entity.id) {
            return Err(RepositoryError::ValidationError(
                format!("Role '{}' already exists", entity.name)
            ));
        }
        
        storage.roles.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if storage.roles.remove(id).is_none() {
            return Err(RepositoryError::NotFoundError(
                format!("Role with id {} not found", id)
            ));
        }
        
        self.save_storage(&storage).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Role>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.roles.values().cloned().collect())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Role>, RepositoryError> {
        let storage = self.load_storage().await?;
        let name_lower = name.to_lowercase();
        
        Ok(storage.roles
            .values()
            .filter(|r| r.name.to_lowercase().contains(&name_lower))
            .cloned()
            .collect())
    }
}

#[async_trait]
pub trait RoleRepositoryExt: Repository<Role> {
    async fn find_by_exact_name(&self, name: &str) -> Result<Option<Role>, RepositoryError>;
    async fn get_roles_for_path(&self, path: &str) -> Result<Vec<Role>, RepositoryError>;
    async fn get_roles_for_state(&self, state: EntityState) -> Result<Vec<Role>, RepositoryError>;
}

#[async_trait]
impl RoleRepositoryExt for RoleRepository {
    async fn find_by_exact_name(&self, name: &str) -> Result<Option<Role>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.roles.values().find(|r| r.name == name).cloned())
    }

    async fn get_roles_for_path(&self, path: &str) -> Result<Vec<Role>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.roles.values()
            .filter(|r| r.can_approve_path(path))
            .cloned()
            .collect())
    }

    async fn get_roles_for_state(&self, state: EntityState) -> Result<Vec<Role>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.roles.values()
            .filter(|r| r.git_approval_authority.approval_contexts.iter()
                .any(|ctx| ctx.entity_states.contains(&state)))
            .cloned()
            .collect())
    }
}