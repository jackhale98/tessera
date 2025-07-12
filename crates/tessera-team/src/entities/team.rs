use serde::{Deserialize, Serialize};
use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub team_type: TeamType,
    pub lead_id: Option<Id>,
    pub members: Vec<Id>,
    pub parent_team_id: Option<Id>,
    pub git_team_name: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamType {
    Engineering,
    Quality,
    Manufacturing,
    ProjectManagement,
    Safety,
    Test,
    Management,
    Custom(String),
}

impl Team {
    pub fn new(name: String, description: String, team_type: TeamType) -> Self {
        Self {
            id: Id::new(),
            name,
            description,
            team_type,
            lead_id: None,
            members: Vec::new(),
            parent_team_id: None,
            git_team_name: None,
            metadata: HashMap::new(),
            created: Utc::now(),
        }
    }

    pub fn set_lead(&mut self, lead_id: Id) {
        self.lead_id = Some(lead_id);
        // Ensure lead is also a member
        if !self.members.contains(&lead_id) {
            self.members.push(lead_id);
        }
    }

    pub fn add_member(&mut self, member_id: Id) {
        if !self.members.contains(&member_id) {
            self.members.push(member_id);
        }
    }

    pub fn remove_member(&mut self, member_id: &Id) {
        self.members.retain(|m| m != member_id);
        // If removed member was lead, clear lead
        if self.lead_id.as_ref() == Some(member_id) {
            self.lead_id = None;
        }
    }

    pub fn set_parent_team(&mut self, parent_id: Id) {
        self.parent_team_id = Some(parent_id);
    }

    pub fn set_git_team_name(&mut self, git_name: String) {
        self.git_team_name = Some(git_name);
    }

    pub fn is_member(&self, member_id: &Id) -> bool {
        self.members.contains(member_id)
    }

    pub fn is_lead(&self, member_id: &Id) -> bool {
        self.lead_id.as_ref() == Some(member_id)
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

impl Entity for Team {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Team name cannot be empty".to_string()
            ));
        }
        if self.description.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Team description cannot be empty".to_string()
            ));
        }
        
        // Ensure lead is a member
        if let Some(lead_id) = &self.lead_id {
            if !self.members.contains(lead_id) {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Team lead must be a member of the team".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

impl std::fmt::Display for TeamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamType::Engineering => write!(f, "Engineering"),
            TeamType::Quality => write!(f, "Quality"),
            TeamType::Manufacturing => write!(f, "Manufacturing"),
            TeamType::ProjectManagement => write!(f, "Project Management"),
            TeamType::Safety => write!(f, "Safety"),
            TeamType::Test => write!(f, "Test"),
            TeamType::Management => write!(f, "Management"),
            TeamType::Custom(name) => write!(f, "{}", name),
        }
    }
}