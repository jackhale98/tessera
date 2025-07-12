use async_trait::async_trait;
use std::sync::Arc;
use tessera_core::{Id, Repository, RepositoryError};

use crate::{
    entities::{TeamMember, Role, Team, EntityState},
    repositories::{
        TeamMemberRepository, TeamMemberRepositoryExt, RoleRepository, RoleRepositoryExt,
        TeamRepository, TeamRepositoryExt, ApprovalRuleRepository
    },
    services::GitApprovalManager,
    integration::traits::{TeamValidator, TeamResolver, GitTeamManager, RoleManager},
};

pub struct TeamValidatorImpl {
    member_repo: Arc<TeamMemberRepository>,
    role_repo: Arc<RoleRepository>,
    team_repo: Arc<TeamRepository>,
}

impl TeamValidatorImpl {
    pub fn new(
        member_repo: Arc<TeamMemberRepository>,
        role_repo: Arc<RoleRepository>,
        team_repo: Arc<TeamRepository>,
    ) -> Self {
        Self {
            member_repo,
            role_repo,
            team_repo,
        }
    }
}

#[async_trait]
impl TeamValidator for TeamValidatorImpl {
    async fn validate_team_member(&self, member_id: &Id<TeamMember>) -> Result<bool, RepositoryError> {
        Ok(self.member_repo.get(&member_id.to_string()).await?.is_some())
    }

    async fn validate_team_member_active(&self, member_id: &Id<TeamMember>) -> Result<bool, RepositoryError> {
        if let Some(member) = self.member_repo.get(&member_id.to_string()).await? {
            Ok(member.active)
        } else {
            Ok(false)
        }
    }

    async fn validate_team_reference(&self, team_id: &Id<Team>) -> Result<bool, RepositoryError> {
        Ok(self.team_repo.get(&team_id.to_string()).await?.is_some())
    }

    async fn get_active_team_members(&self) -> Result<Vec<TeamMember>, RepositoryError> {
        self.member_repo.get_active_members().await
    }

    async fn resolve_git_username(&self, member_id: &Id<TeamMember>) -> Result<Option<String>, RepositoryError> {
        if let Some(member) = self.member_repo.get(&member_id.to_string()).await? {
            Ok(member.git_username)
        } else {
            Ok(None)
        }
    }

    async fn validate_role_assignment(
        &self, 
        member_id: &Id<TeamMember>, 
        role_id: &Id<Role>
    ) -> Result<bool, RepositoryError> {
        let member_exists = self.member_repo.get(&member_id.to_string()).await?.is_some();
        let role_exists = self.role_repo.get(&role_id.to_string()).await?.is_some();
        Ok(member_exists && role_exists)
    }
}

pub struct TeamResolverImpl {
    member_repo: Arc<TeamMemberRepository>,
    team_repo: Arc<TeamRepository>,
    approval_manager: Arc<GitApprovalManager>,
}

impl TeamResolverImpl {
    pub fn new(
        member_repo: Arc<TeamMemberRepository>,
        team_repo: Arc<TeamRepository>,
        approval_manager: Arc<GitApprovalManager>,
    ) -> Self {
        Self {
            member_repo,
            team_repo,
            approval_manager,
        }
    }
}

#[async_trait]
impl TeamResolver for TeamResolverImpl {
    async fn resolve_team_member(&self, identifier: &str) -> Result<Option<TeamMember>, RepositoryError> {
        // Try to resolve by ID first
        if let Ok(uuid) = uuid::Uuid::parse_str(identifier) {
            if let Some(member) = self.member_repo.get(identifier).await? {
                return Ok(Some(member));
            }
        }

        // Try to resolve by email
        if let Some(member) = self.member_repo.find_by_email(identifier).await? {
            return Ok(Some(member));
        }

        // Try to resolve by git username
        if let Some(member) = self.member_repo.find_by_git_username(identifier).await? {
            return Ok(Some(member));
        }

        // Try to resolve by name search
        let members = self.member_repo.find_by_name(identifier).await?;
        if members.len() == 1 {
            return Ok(Some(members[0].clone()));
        }

        Ok(None)
    }

    async fn resolve_team_member_by_email(&self, email: &str) -> Result<Option<TeamMember>, RepositoryError> {
        self.member_repo.find_by_email(email).await
    }

    async fn resolve_team_member_by_git_username(&self, username: &str) -> Result<Option<TeamMember>, RepositoryError> {
        self.member_repo.find_by_git_username(username).await
    }

    async fn get_team_members_by_role(&self, role_id: &Id<Role>) -> Result<Vec<TeamMember>, RepositoryError> {
        self.member_repo.get_members_by_role(role_id).await
    }

    async fn get_team_leads(&self) -> Result<Vec<TeamMember>, RepositoryError> {
        let teams = self.team_repo.list().await?;
        let mut leads = Vec::new();

        for team in teams {
            if let Some(lead_id) = team.lead_id {
                if let Some(lead) = self.member_repo.get(&lead_id.to_string()).await? {
                    if lead.active && !leads.iter().any(|l: &TeamMember| l.id == lead.id) {
                        leads.push(lead);
                    }
                }
            }
        }

        Ok(leads)
    }

    async fn get_approval_chain(
        &self, 
        module: &str, 
        state: EntityState,
        impact: Option<f64>
    ) -> Result<Vec<TeamMember>, RepositoryError> {
        let requirement = self.approval_manager
            .get_required_approvers(&format!("{}/", module), state, impact.unwrap_or(0.0))
            .await?;

        let mut approvers = Vec::new();
        for group in requirement.approval_groups {
            for git_username in group.eligible_approvers {
                // Remove @ prefix and find member
                let username = git_username.trim_start_matches('@');
                if let Some(member) = self.member_repo.find_by_git_username(username).await? {
                    if !approvers.iter().any(|a: &TeamMember| a.id == member.id) {
                        approvers.push(member);
                    }
                }
            }
        }

        Ok(approvers)
    }
}

pub struct GitTeamManagerImpl {
    approval_manager: Arc<GitApprovalManager>,
}

impl GitTeamManagerImpl {
    pub fn new(approval_manager: Arc<GitApprovalManager>) -> Self {
        Self { approval_manager }
    }
}

#[async_trait]
impl GitTeamManager for GitTeamManagerImpl {
    async fn sync_git_teams(&self) -> Result<Vec<String>, RepositoryError> {
        self.approval_manager.sync_git_teams().await
    }

    async fn get_git_approvers(&self, module_path: &str, state: EntityState) -> Result<Vec<String>, RepositoryError> {
        self.approval_manager.get_approvers_for_module(module_path, state).await
    }

    async fn generate_codeowners(&self, state: EntityState) -> Result<String, RepositoryError> {
        self.approval_manager.generate_codeowners_for_state(state).await
    }

    async fn validate_git_usernames(&self) -> Result<Vec<String>, RepositoryError> {
        self.approval_manager.validate_git_usernames().await
    }

    async fn get_required_approvers(
        &self,
        module_path: &str,
        entity_state: EntityState,
        change_impact: f64,
    ) -> Result<crate::entities::ApprovalRequirement, RepositoryError> {
        self.approval_manager.get_required_approvers(module_path, entity_state, change_impact).await
    }
}

pub struct RoleManagerImpl {
    role_repo: Arc<RoleRepository>,
}

impl RoleManagerImpl {
    pub fn new(role_repo: Arc<RoleRepository>) -> Self {
        Self { role_repo }
    }
}

#[async_trait]
impl RoleManager for RoleManagerImpl {
    async fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, RepositoryError> {
        self.role_repo.find_by_exact_name(name).await
    }

    async fn get_roles_for_path(&self, path: &str) -> Result<Vec<Role>, RepositoryError> {
        self.role_repo.get_roles_for_path(path).await
    }

    async fn get_roles_for_state(&self, state: EntityState) -> Result<Vec<Role>, RepositoryError> {
        self.role_repo.get_roles_for_state(state).await
    }

    async fn check_role_permission(
        &self,
        role_id: &Id<Role>,
        path: &str,
        module: &str,
        state: EntityState,
        impact: Option<f64>,
    ) -> Result<bool, RepositoryError> {
        if let Some(role) = self.role_repo.get(&role_id.to_string()).await? {
            Ok(role.can_approve_path(path) && role.can_approve_in_context(module, state, impact))
        } else {
            Ok(false)
        }
    }
}