use serde::{Deserialize, Serialize};
use tessera_core::{Entity, Id, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::EntityState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRule {
    pub id: Id,
    pub name: String,
    pub path_pattern: String,
    pub required_approver_groups: Vec<ApprovalGroup>,
    pub min_approvals_required: u32,
    pub entity_states: Vec<EntityState>,
    pub impact_threshold: Option<f64>,
    pub all_groups_required: bool,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalGroup {
    pub name: String,
    pub required_approvers: Vec<Id>,
    pub required_roles: Vec<Id>,
    pub required_teams: Vec<Id>,
    pub min_approvals_from_group: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequirement {
    pub total_approvals_needed: u32,
    pub approval_groups: Vec<ResolvedApprovalGroup>,
    pub all_groups_must_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedApprovalGroup {
    pub group_name: String,
    pub eligible_approvers: Vec<String>,
    pub approvals_needed_from_group: u32,
}

impl ApprovalRule {
    pub fn new(name: String, path_pattern: String) -> Self {
        Self {
            id: Id::new(),
            name,
            path_pattern,
            required_approver_groups: Vec::new(),
            min_approvals_required: 1,
            entity_states: vec![EntityState::InReview],
            impact_threshold: None,
            all_groups_required: false,
            metadata: HashMap::new(),
            created: Utc::now(),
        }
    }

    pub fn applies_to_path(&self, path: &str) -> bool {
        if self.path_pattern.ends_with('/') {
            path.starts_with(&self.path_pattern)
        } else if self.path_pattern.contains('*') {
            // Simple glob pattern matching - just check if it starts with the prefix
            let prefix = self.path_pattern.trim_end_matches('*');
            path.starts_with(prefix)
        } else {
            path == self.path_pattern
        }
    }

    pub fn applies_to_state(&self, state: EntityState) -> bool {
        self.entity_states.contains(&state)
    }

    pub fn applies_to_impact(&self, impact: f64) -> bool {
        self.impact_threshold.map_or(true, |threshold| impact >= threshold)
    }

    pub fn add_approver_group(&mut self, group: ApprovalGroup) {
        self.required_approver_groups.push(group);
    }

    pub fn set_states(&mut self, states: Vec<EntityState>) {
        self.entity_states = states;
    }

    pub fn set_impact_threshold(&mut self, threshold: f64) {
        self.impact_threshold = Some(threshold);
    }

    pub fn set_all_groups_required(&mut self, required: bool) {
        self.all_groups_required = required;
    }

    pub fn validate_approvals(&self) -> Result<()> {
        if self.required_approver_groups.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "At least one approver group must be defined".to_string()
            ));
        }

        // Calculate total possible approvals
        let total_possible: u32 = self.required_approver_groups
            .iter()
            .map(|g| g.min_approvals_from_group)
            .sum();

        if total_possible < self.min_approvals_required {
            return Err(tessera_core::DesignTrackError::Validation(format!(
                "Total possible approvals ({}) is less than minimum required ({})",
                total_possible, self.min_approvals_required
            )));
        }

        Ok(())
    }
}

impl Entity for ApprovalRule {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Approval rule name cannot be empty".to_string()
            ));
        }
        if self.path_pattern.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Path pattern cannot be empty".to_string()
            ));
        }
        if self.entity_states.is_empty() {
            return Err(tessera_core::DesignTrackError::Validation(
                "At least one entity state must be specified".to_string()
            ));
        }
        if self.min_approvals_required == 0 {
            return Err(tessera_core::DesignTrackError::Validation(
                "Minimum approvals required must be at least 1".to_string()
            ));
        }

        Ok(())
    }
}

impl ApprovalGroup {
    pub fn new(name: String) -> Self {
        Self {
            name,
            required_approvers: Vec::new(),
            required_roles: Vec::new(),
            required_teams: Vec::new(),
            min_approvals_from_group: 1,
        }
    }

    pub fn with_approvers(mut self, approvers: Vec<Id>) -> Self {
        self.required_approvers = approvers;
        self
    }

    pub fn with_roles(mut self, roles: Vec<Id>) -> Self {
        self.required_roles = roles;
        self
    }

    pub fn with_teams(mut self, teams: Vec<Id>) -> Self {
        self.required_teams = teams;
        self
    }

    pub fn with_min_approvals(mut self, min: u32) -> Self {
        self.min_approvals_from_group = min;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.required_approvers.is_empty() && 
        self.required_roles.is_empty() && 
        self.required_teams.is_empty()
    }
}

impl ApprovalRequirement {
    pub fn new(total_approvals: u32, all_groups_required: bool) -> Self {
        Self {
            total_approvals_needed: total_approvals,
            approval_groups: Vec::new(),
            all_groups_must_approve: all_groups_required,
        }
    }

    pub fn add_group(&mut self, group: ResolvedApprovalGroup) {
        self.approval_groups.push(group);
    }

    pub fn is_satisfied_by(&self, approvers: &[String]) -> bool {
        if approvers.len() < self.total_approvals_needed as usize {
            return false;
        }

        if self.all_groups_must_approve {
            // Each group must have its minimum met
            self.approval_groups.iter().all(|group| {
                let group_approvals = approvers.iter()
                    .filter(|a| group.eligible_approvers.contains(a))
                    .count();
                group_approvals >= group.approvals_needed_from_group as usize
            })
        } else {
            // Just need total approvals from any eligible approvers
            let eligible_approvals = approvers.iter()
                .filter(|a| self.approval_groups.iter()
                    .any(|g| g.eligible_approvers.contains(a)))
                .count();
            eligible_approvals >= self.total_approvals_needed as usize
        }
    }
}