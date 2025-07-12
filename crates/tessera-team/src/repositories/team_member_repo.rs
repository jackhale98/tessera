use async_trait::async_trait;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tessera_core::{Id, Repository, RepositoryError};

use crate::entities::{TeamMember, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberStorage {
    pub members: IndexMap<String, TeamMember>,
}

impl Default for TeamMemberStorage {
    fn default() -> Self {
        Self {
            members: IndexMap::new(),
        }
    }
}

pub struct TeamMemberRepository {
    path: PathBuf,
}

impl TeamMemberRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            path: base_path.join("team").join("members.ron"),
        }
    }

    async fn load_storage(&self) -> Result<TeamMemberStorage, RepositoryError> {
        if !self.path.exists() {
            return Ok(TeamMemberStorage::default());
        }

        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| RepositoryError::LoadError(e.to_string()))?;

        ron::from_str(&content)
            .map_err(|e| RepositoryError::ParseError(e.to_string()))
    }

    async fn save_storage(&self, storage: &TeamMemberStorage) -> Result<(), RepositoryError> {
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
impl Repository<TeamMember> for TeamMemberRepository {
    async fn create(&self, entity: TeamMember) -> Result<TeamMember, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        // Check for duplicate email
        if storage.members.values().any(|m| m.email == entity.email && m.id != entity.id) {
            return Err(RepositoryError::ValidationError(
                format!("Email '{}' is already in use", entity.email)
            ));
        }
        
        // Check for duplicate git username if provided
        if let Some(git_username) = &entity.git_username {
            if storage.members.values().any(|m| 
                m.git_username.as_ref() == Some(git_username) && m.id != entity.id
            ) {
                return Err(RepositoryError::ValidationError(
                    format!("Git username '{}' is already in use", git_username)
                ));
            }
        }
        
        storage.members.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn get(&self, id: &str) -> Result<Option<TeamMember>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.members.get(id).cloned())
    }

    async fn update(&self, entity: TeamMember) -> Result<TeamMember, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if !storage.members.contains_key(&entity.id.to_string()) {
            return Err(RepositoryError::NotFoundError(
                format!("Team member with id {} not found", entity.id)
            ));
        }
        
        // Check for duplicate email
        if storage.members.values().any(|m| m.email == entity.email && m.id != entity.id) {
            return Err(RepositoryError::ValidationError(
                format!("Email '{}' is already in use", entity.email)
            ));
        }
        
        // Check for duplicate git username if provided
        if let Some(git_username) = &entity.git_username {
            if storage.members.values().any(|m| 
                m.git_username.as_ref() == Some(git_username) && m.id != entity.id
            ) {
                return Err(RepositoryError::ValidationError(
                    format!("Git username '{}' is already in use", git_username)
                ));
            }
        }
        
        storage.members.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if storage.members.remove(id).is_none() {
            return Err(RepositoryError::NotFoundError(
                format!("Team member with id {} not found", id)
            ));
        }
        
        self.save_storage(&storage).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<TeamMember>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.members.values().cloned().collect())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<TeamMember>, RepositoryError> {
        let storage = self.load_storage().await?;
        let name_lower = name.to_lowercase();
        
        Ok(storage.members
            .values()
            .filter(|m| m.full_name().to_lowercase().contains(&name_lower))
            .cloned()
            .collect())
    }
}

#[async_trait]
pub trait TeamMemberRepositoryExt: Repository<TeamMember> {
    async fn find_by_email(&self, email: &str) -> Result<Option<TeamMember>, RepositoryError>;
    async fn find_by_git_username(&self, username: &str) -> Result<Option<TeamMember>, RepositoryError>;
    async fn get_active_members(&self) -> Result<Vec<TeamMember>, RepositoryError>;
    async fn get_members_by_role(&self, role_id: &Id<Role>) -> Result<Vec<TeamMember>, RepositoryError>;
}

#[async_trait]
impl TeamMemberRepositoryExt for TeamMemberRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<TeamMember>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.members.values().find(|m| m.email == email).cloned())
    }

    async fn find_by_git_username(&self, username: &str) -> Result<Option<TeamMember>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.members.values()
            .find(|m| m.git_username.as_ref() == Some(&username.to_string()))
            .cloned())
    }

    async fn get_active_members(&self) -> Result<Vec<TeamMember>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.members.values()
            .filter(|m| m.active)
            .cloned()
            .collect())
    }

    async fn get_members_by_role(&self, role_id: &Id<Role>) -> Result<Vec<TeamMember>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.members.values()
            .filter(|m| m.primary_role == *role_id || m.additional_roles.contains(role_id))
            .cloned()
            .collect())
    }
}