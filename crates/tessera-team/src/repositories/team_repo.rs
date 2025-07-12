use async_trait::async_trait;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tessera_core::{Id, Repository, RepositoryError};

use crate::entities::{Team, TeamMember, TeamType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStorage {
    pub teams: IndexMap<String, Team>,
}

impl Default for TeamStorage {
    fn default() -> Self {
        Self {
            teams: IndexMap::new(),
        }
    }
}

pub struct TeamRepository {
    path: PathBuf,
}

impl TeamRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            path: base_path.join("team").join("teams.ron"),
        }
    }

    async fn load_storage(&self) -> Result<TeamStorage, RepositoryError> {
        if !self.path.exists() {
            return Ok(TeamStorage::default());
        }

        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| RepositoryError::LoadError(e.to_string()))?;

        ron::from_str(&content)
            .map_err(|e| RepositoryError::ParseError(e.to_string()))
    }

    async fn save_storage(&self, storage: &TeamStorage) -> Result<(), RepositoryError> {
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
impl Repository<Team> for TeamRepository {
    async fn create(&self, entity: Team) -> Result<Team, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        // Check for duplicate team name
        if storage.teams.values().any(|t| t.name == entity.name && t.id != entity.id) {
            return Err(RepositoryError::ValidationError(
                format!("Team '{}' already exists", entity.name)
            ));
        }
        
        // Check for duplicate git team name if provided
        if let Some(git_name) = &entity.git_team_name {
            if storage.teams.values().any(|t| 
                t.git_team_name.as_ref() == Some(git_name) && t.id != entity.id
            ) {
                return Err(RepositoryError::ValidationError(
                    format!("Git team name '{}' is already in use", git_name)
                ));
            }
        }
        
        storage.teams.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn get(&self, id: &str) -> Result<Option<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.get(id).cloned())
    }

    async fn update(&self, entity: Team) -> Result<Team, RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if !storage.teams.contains_key(&entity.id.to_string()) {
            return Err(RepositoryError::NotFoundError(
                format!("Team with id {} not found", entity.id)
            ));
        }
        
        // Check for duplicate team name
        if storage.teams.values().any(|t| t.name == entity.name && t.id != entity.id) {
            return Err(RepositoryError::ValidationError(
                format!("Team '{}' already exists", entity.name)
            ));
        }
        
        // Check for duplicate git team name if provided
        if let Some(git_name) = &entity.git_team_name {
            if storage.teams.values().any(|t| 
                t.git_team_name.as_ref() == Some(git_name) && t.id != entity.id
            ) {
                return Err(RepositoryError::ValidationError(
                    format!("Git team name '{}' is already in use", git_name)
                ));
            }
        }
        
        storage.teams.insert(entity.id.to_string(), entity.clone());
        self.save_storage(&storage).await?;
        Ok(entity)
    }

    async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let mut storage = self.load_storage().await?;
        
        if storage.teams.remove(id).is_none() {
            return Err(RepositoryError::NotFoundError(
                format!("Team with id {} not found", id)
            ));
        }
        
        self.save_storage(&storage).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.values().cloned().collect())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        let name_lower = name.to_lowercase();
        
        Ok(storage.teams
            .values()
            .filter(|t| t.name.to_lowercase().contains(&name_lower))
            .cloned()
            .collect())
    }
}

#[async_trait]
pub trait TeamRepositoryExt: Repository<Team> {
    async fn find_by_exact_name(&self, name: &str) -> Result<Option<Team>, RepositoryError>;
    async fn find_by_git_team_name(&self, git_name: &str) -> Result<Option<Team>, RepositoryError>;
    async fn get_teams_by_type(&self, team_type: &TeamType) -> Result<Vec<Team>, RepositoryError>;
    async fn get_teams_by_member(&self, member_id: &Id<TeamMember>) -> Result<Vec<Team>, RepositoryError>;
    async fn get_teams_by_lead(&self, lead_id: &Id<TeamMember>) -> Result<Vec<Team>, RepositoryError>;
    async fn get_child_teams(&self, parent_id: &Id<Team>) -> Result<Vec<Team>, RepositoryError>;
}

#[async_trait]
impl TeamRepositoryExt for TeamRepository {
    async fn find_by_exact_name(&self, name: &str) -> Result<Option<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.values().find(|t| t.name == name).cloned())
    }

    async fn find_by_git_team_name(&self, git_name: &str) -> Result<Option<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.values()
            .find(|t| t.git_team_name.as_ref() == Some(&git_name.to_string()))
            .cloned())
    }

    async fn get_teams_by_type(&self, team_type: &TeamType) -> Result<Vec<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.values()
            .filter(|t| &t.team_type == team_type)
            .cloned()
            .collect())
    }

    async fn get_teams_by_member(&self, member_id: &Id<TeamMember>) -> Result<Vec<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.values()
            .filter(|t| t.is_member(member_id))
            .cloned()
            .collect())
    }

    async fn get_teams_by_lead(&self, lead_id: &Id<TeamMember>) -> Result<Vec<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.values()
            .filter(|t| t.is_lead(lead_id))
            .cloned()
            .collect())
    }

    async fn get_child_teams(&self, parent_id: &Id<Team>) -> Result<Vec<Team>, RepositoryError> {
        let storage = self.load_storage().await?;
        Ok(storage.teams.values()
            .filter(|t| t.parent_team_id.as_ref() == Some(parent_id))
            .cloned()
            .collect())
    }
}