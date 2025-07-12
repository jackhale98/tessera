use crate::entities::*;
use tessera_core::{Id, Result, ProjectContext, Entity};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use ron::ser::{to_string_pretty, PrettyConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRepository {
    pub team_members: IndexMap<Id, TeamMember>,
    pub roles: IndexMap<Id, Role>,
    pub teams: IndexMap<Id, Team>,
    pub approval_rules: IndexMap<Id, ApprovalRule>,
}

impl TeamRepository {
    pub fn new() -> Self {
        Self {
            team_members: IndexMap::new(),
            roles: IndexMap::new(),
            teams: IndexMap::new(),
            approval_rules: IndexMap::new(),
        }
    }

    pub fn load_from_project(project_ctx: &ProjectContext) -> Result<Self> {
        let team_dir = project_ctx.module_path("team");
        Self::load_from_directory(&team_dir)
    }

    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let mut repo = Self::new();

        // Load team members
        let members_file = dir.join("members.ron");
        if members_file.exists() {
            let content = std::fs::read_to_string(&members_file)?;
            let members: IndexMap<Id, TeamMember> = ron::from_str(&content)?;
            repo.team_members = members;
        }

        // Load roles
        let roles_file = dir.join("roles.ron");
        if roles_file.exists() {
            let content = std::fs::read_to_string(&roles_file)?;
            let roles: IndexMap<Id, Role> = ron::from_str(&content)?;
            repo.roles = roles;
        }

        // Load teams
        let teams_file = dir.join("teams.ron");
        if teams_file.exists() {
            let content = std::fs::read_to_string(&teams_file)?;
            let teams: IndexMap<Id, Team> = ron::from_str(&content)?;
            repo.teams = teams;
        }

        // Load approval rules
        let approval_rules_file = dir.join("approval_rules.ron");
        if approval_rules_file.exists() {
            let content = std::fs::read_to_string(&approval_rules_file)?;
            let approval_rules: IndexMap<Id, ApprovalRule> = ron::from_str(&content)?;
            repo.approval_rules = approval_rules;
        }

        Ok(repo)
    }

    pub fn save_to_project(&self, project_ctx: &ProjectContext) -> Result<()> {
        let team_dir = project_ctx.module_path("team");
        self.save_to_directory(&team_dir)
    }

    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let config = PrettyConfig::default();

        // Save team members
        let members_content = to_string_pretty(&self.team_members, config.clone())?;
        std::fs::write(dir.join("members.ron"), members_content)?;

        // Save roles
        let roles_content = to_string_pretty(&self.roles, config.clone())?;
        std::fs::write(dir.join("roles.ron"), roles_content)?;

        // Save teams
        let teams_content = to_string_pretty(&self.teams, config.clone())?;
        std::fs::write(dir.join("teams.ron"), teams_content)?;

        // Save approval rules
        let approval_rules_content = to_string_pretty(&self.approval_rules, config)?;
        std::fs::write(dir.join("approval_rules.ron"), approval_rules_content)?;

        Ok(())
    }

    // Team Member operations
    pub fn add_team_member(&mut self, member: TeamMember) -> Result<()> {
        member.validate()?;
        self.team_members.insert(member.id(), member);
        Ok(())
    }

    pub fn get_team_member(&self, id: Id) -> Option<&TeamMember> {
        self.team_members.get(&id)
    }

    pub fn get_team_members(&self) -> &IndexMap<Id, TeamMember> {
        &self.team_members
    }

    pub fn get_active_team_members(&self) -> Vec<&TeamMember> {
        self.team_members.values().filter(|m| m.active).collect()
    }

    pub fn find_team_member_by_email(&self, email: &str) -> Option<&TeamMember> {
        self.team_members.values().find(|m| m.email == email)
    }

    pub fn find_team_member_by_git_username(&self, username: &str) -> Option<&TeamMember> {
        self.team_members.values()
            .find(|m| m.git_username.as_ref().map(|s| s.as_str()) == Some(username))
    }

    pub fn update_team_member(&mut self, member: TeamMember) -> Result<()> {
        member.validate()?;
        if !self.team_members.contains_key(&member.id()) {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Team member {} not found", member.id())
            ));
        }
        self.team_members.insert(member.id(), member);
        Ok(())
    }

    pub fn remove_team_member(&mut self, id: Id) -> Result<()> {
        if self.team_members.shift_remove(&id).is_none() {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Team member {} not found", id)
            ));
        }
        Ok(())
    }

    // Role operations
    pub fn add_role(&mut self, role: Role) -> Result<()> {
        role.validate()?;
        self.roles.insert(role.id(), role);
        Ok(())
    }

    pub fn get_role(&self, id: Id) -> Option<&Role> {
        self.roles.get(&id)
    }

    pub fn get_roles(&self) -> &IndexMap<Id, Role> {
        &self.roles
    }

    pub fn find_role_by_name(&self, name: &str) -> Option<&Role> {
        self.roles.values().find(|r| r.name == name)
    }

    pub fn update_role(&mut self, role: Role) -> Result<()> {
        role.validate()?;
        if !self.roles.contains_key(&role.id()) {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Role {} not found", role.id())
            ));
        }
        self.roles.insert(role.id(), role);
        Ok(())
    }

    pub fn remove_role(&mut self, id: Id) -> Result<()> {
        if self.roles.shift_remove(&id).is_none() {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Role {} not found", id)
            ));
        }
        Ok(())
    }

    // Team operations
    pub fn add_team(&mut self, team: Team) -> Result<()> {
        team.validate()?;
        self.teams.insert(team.id(), team);
        Ok(())
    }

    pub fn get_team(&self, id: Id) -> Option<&Team> {
        self.teams.get(&id)
    }

    pub fn get_teams(&self) -> &IndexMap<Id, Team> {
        &self.teams
    }

    pub fn find_team_by_name(&self, name: &str) -> Option<&Team> {
        self.teams.values().find(|t| t.name == name)
    }

    pub fn get_teams_by_member(&self, member_id: Id) -> Vec<&Team> {
        self.teams.values()
            .filter(|t| t.is_member(&member_id))
            .collect()
    }

    pub fn get_teams_by_lead(&self, lead_id: Id) -> Vec<&Team> {
        self.teams.values()
            .filter(|t| t.is_lead(&lead_id))
            .collect()
    }

    pub fn update_team(&mut self, team: Team) -> Result<()> {
        team.validate()?;
        if !self.teams.contains_key(&team.id()) {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Team {} not found", team.id())
            ));
        }
        self.teams.insert(team.id(), team);
        Ok(())
    }

    pub fn remove_team(&mut self, id: Id) -> Result<()> {
        if self.teams.shift_remove(&id).is_none() {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Team {} not found", id)
            ));
        }
        Ok(())
    }

    // Approval rule operations
    pub fn add_approval_rule(&mut self, rule: ApprovalRule) -> Result<()> {
        rule.validate()?;
        self.approval_rules.insert(rule.id(), rule);
        Ok(())
    }

    pub fn get_approval_rule(&self, id: Id) -> Option<&ApprovalRule> {
        self.approval_rules.get(&id)
    }

    pub fn get_approval_rules(&self) -> &IndexMap<Id, ApprovalRule> {
        &self.approval_rules
    }

    pub fn get_approval_rules_for_path(&self, path: &str) -> Vec<&ApprovalRule> {
        self.approval_rules.values()
            .filter(|r| r.applies_to_path(path))
            .collect()
    }

    pub fn update_approval_rule(&mut self, rule: ApprovalRule) -> Result<()> {
        rule.validate()?;
        if !self.approval_rules.contains_key(&rule.id()) {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Approval rule {} not found", rule.id())
            ));
        }
        self.approval_rules.insert(rule.id(), rule);
        Ok(())
    }

    pub fn remove_approval_rule(&mut self, id: Id) -> Result<()> {
        if self.approval_rules.shift_remove(&id).is_none() {
            return Err(tessera_core::DesignTrackError::NotFound(
                format!("Approval rule {} not found", id)
            ));
        }
        Ok(())
    }

    // Validation and utility methods
    pub fn validate_all(&self) -> Result<()> {
        for member in self.team_members.values() {
            member.validate()?;
        }
        for role in self.roles.values() {
            role.validate()?;
        }
        for team in self.teams.values() {
            team.validate()?;
        }
        for rule in self.approval_rules.values() {
            rule.validate()?;
        }
        Ok(())
    }
}