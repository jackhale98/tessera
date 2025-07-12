use serde::{Deserialize, Serialize};
use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub git_approval_authority: GitApprovalAuthority,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitApprovalAuthority {
    pub can_approve_paths: Vec<String>,
    pub approval_contexts: Vec<ApprovalContext>,
    pub max_cost_approval: Option<f64>,
    pub max_schedule_impact_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalContext {
    pub entity_states: Vec<EntityState>,
    pub modules: Vec<String>,
    pub impact_threshold: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityState {
    Draft,
    InReview,
    Approved,
    Released,
}

impl Role {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Id::new(),
            name,
            description,
            git_approval_authority: GitApprovalAuthority {
                can_approve_paths: Vec::new(),
                approval_contexts: Vec::new(),
                max_cost_approval: None,
                max_schedule_impact_days: None,
            },
            metadata: HashMap::new(),
            created: Utc::now(),
        }
    }

    pub fn with_approval_authority(mut self, authority: GitApprovalAuthority) -> Self {
        self.git_approval_authority = authority;
        self
    }

    pub fn can_approve_path(&self, path: &str) -> bool {
        self.git_approval_authority.can_approve_paths.iter()
            .any(|pattern| {
                if pattern.ends_with('/') {
                    path.starts_with(pattern)
                } else {
                    path == pattern || path.starts_with(&format!("{}/", pattern))
                }
            })
    }

    pub fn can_approve_in_context(
        &self, 
        module: &str, 
        state: EntityState, 
        impact: Option<f64>
    ) -> bool {
        self.git_approval_authority.approval_contexts.iter()
            .any(|ctx| {
                ctx.modules.contains(&module.to_string()) &&
                ctx.entity_states.contains(&state) &&
                (ctx.impact_threshold.is_none() || 
                 impact.map_or(true, |i| ctx.impact_threshold.map_or(true, |t| i >= t)))
            })
    }

    pub fn can_approve_cost(&self, cost: f64) -> bool {
        self.git_approval_authority.max_cost_approval
            .map_or(false, |max| cost <= max)
    }

    pub fn can_approve_schedule_impact(&self, days: u32) -> bool {
        self.git_approval_authority.max_schedule_impact_days
            .map_or(false, |max| days <= max)
    }
}

impl Entity for Role {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Role name cannot be empty".to_string()
            ));
        }
        if self.description.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Role description cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }
}

impl std::fmt::Display for EntityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityState::Draft => write!(f, "Draft"),
            EntityState::InReview => write!(f, "In Review"),
            EntityState::Approved => write!(f, "Approved"),
            EntityState::Released => write!(f, "Released"),
        }
    }
}