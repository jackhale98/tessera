use async_trait::async_trait;
use tessera_core::{Id, RepositoryError};

use crate::entities::{TeamMember, Role, Team, EntityState};

#[async_trait]
pub trait TeamValidator: Send + Sync {
    async fn validate_team_member(&self, member_id: &Id<TeamMember>) -> Result<bool, RepositoryError>;
    async fn validate_team_member_active(&self, member_id: &Id<TeamMember>) -> Result<bool, RepositoryError>;
    async fn validate_team_reference(&self, team_id: &Id<Team>) -> Result<bool, RepositoryError>;
    async fn get_active_team_members(&self) -> Result<Vec<TeamMember>, RepositoryError>;
    async fn resolve_git_username(&self, member_id: &Id<TeamMember>) -> Result<Option<String>, RepositoryError>;
    async fn validate_role_assignment(
        &self, 
        member_id: &Id<TeamMember>, 
        role_id: &Id<Role>
    ) -> Result<bool, RepositoryError>;
}

#[async_trait]
pub trait TeamResolver: Send + Sync {
    async fn resolve_team_member(&self, identifier: &str) -> Result<Option<TeamMember>, RepositoryError>;
    async fn resolve_team_member_by_email(&self, email: &str) -> Result<Option<TeamMember>, RepositoryError>;
    async fn resolve_team_member_by_git_username(&self, username: &str) -> Result<Option<TeamMember>, RepositoryError>;
    async fn get_team_members_by_role(&self, role_id: &Id<Role>) -> Result<Vec<TeamMember>, RepositoryError>;
    async fn get_team_leads(&self) -> Result<Vec<TeamMember>, RepositoryError>;
    async fn get_approval_chain(
        &self, 
        module: &str, 
        state: EntityState,
        impact: Option<f64>
    ) -> Result<Vec<TeamMember>, RepositoryError>;
}

#[async_trait]
pub trait GitTeamManager: Send + Sync {
    async fn sync_git_teams(&self) -> Result<Vec<String>, RepositoryError>;
    async fn get_git_approvers(&self, module_path: &str, state: EntityState) -> Result<Vec<String>, RepositoryError>;
    async fn generate_codeowners(&self, state: EntityState) -> Result<String, RepositoryError>;
    async fn validate_git_usernames(&self) -> Result<Vec<String>, RepositoryError>;
    async fn get_required_approvers(
        &self,
        module_path: &str,
        entity_state: EntityState,
        change_impact: f64,
    ) -> Result<crate::entities::ApprovalRequirement, RepositoryError>;
}

#[async_trait]
pub trait RoleManager: Send + Sync {
    async fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, RepositoryError>;
    async fn get_roles_for_path(&self, path: &str) -> Result<Vec<Role>, RepositoryError>;
    async fn get_roles_for_state(&self, state: EntityState) -> Result<Vec<Role>, RepositoryError>;
    async fn check_role_permission(
        &self,
        role_id: &Id<Role>,
        path: &str,
        module: &str,
        state: EntityState,
        impact: Option<f64>,
    ) -> Result<bool, RepositoryError>;
}