use std::sync::Arc;
use tessera_core::{Id, Repository, RepositoryError};

use crate::{
    entities::{TeamMember, Role, Team},
    repositories::{TeamMemberRepository, TeamRepository},
};

pub struct TeamService {
    member_repo: Arc<TeamMemberRepository>,
    role_repo: Arc<dyn Repository<Role> + Send + Sync>,
    team_repo: Arc<TeamRepository>,
}

impl TeamService {
    pub fn new(
        member_repo: Arc<TeamMemberRepository>,
        role_repo: Arc<dyn Repository<Role> + Send + Sync>,
        team_repo: Arc<TeamRepository>,
    ) -> Self {
        Self {
            member_repo,
            role_repo,
            team_repo,
        }
    }

    pub async fn create_team_member(
        &self,
        first_name: String,
        last_name: String,
        email: String,
        job_title: String,
        department: String,
        primary_role_id: Id<Role>,
    ) -> Result<TeamMember, RepositoryError> {
        // Verify role exists
        let role = self.role_repo.get(&primary_role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::ValidationError(
                format!("Primary role {} not found", primary_role_id)
            ))?;

        let member = TeamMember::new(
            first_name,
            last_name,
            email,
            job_title,
            department,
            primary_role_id,
        );

        self.member_repo.create(member).await
    }

    pub async fn update_team_member_contact(
        &self,
        member_id: &Id<TeamMember>,
        phone: Option<String>,
        office_location: Option<String>,
        time_zone: Option<String>,
    ) -> Result<TeamMember, RepositoryError> {
        let mut member = self.member_repo.get(&member_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", member_id)
            ))?;

        member.update_contact_info(phone, office_location, time_zone);
        self.member_repo.update(member).await
    }

    pub async fn update_team_member_system_integration(
        &self,
        member_id: &Id<TeamMember>,
        git_username: Option<String>,
        slack_username: Option<String>,
    ) -> Result<TeamMember, RepositoryError> {
        let mut member = self.member_repo.get(&member_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", member_id)
            ))?;

        member.update_system_integration(git_username, slack_username);
        self.member_repo.update(member).await
    }

    pub async fn deactivate_team_member(
        &self,
        member_id: &Id<TeamMember>,
    ) -> Result<TeamMember, RepositoryError> {
        let mut member = self.member_repo.get(&member_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", member_id)
            ))?;

        member.deactivate();
        
        // Remove from all teams
        use crate::repositories::TeamRepositoryExt;
        let teams = self.team_repo.get_teams_by_member(member_id).await?;
        for mut team in teams {
            team.remove_member(member_id);
            self.team_repo.update(team).await?;
        }

        self.member_repo.update(member).await
    }

    pub async fn add_role_to_member(
        &self,
        member_id: &Id<TeamMember>,
        role_id: &Id<Role>,
    ) -> Result<TeamMember, RepositoryError> {
        // Verify role exists
        let _ = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::ValidationError(
                format!("Role {} not found", role_id)
            ))?;

        let mut member = self.member_repo.get(&member_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", member_id)
            ))?;

        member.add_role(role_id.clone());
        self.member_repo.update(member).await
    }

    pub async fn remove_role_from_member(
        &self,
        member_id: &Id<TeamMember>,
        role_id: &Id<Role>,
    ) -> Result<TeamMember, RepositoryError> {
        let mut member = self.member_repo.get(&member_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", member_id)
            ))?;

        if member.primary_role == *role_id {
            return Err(RepositoryError::ValidationError(
                "Cannot remove primary role".to_string()
            ));
        }

        member.remove_role(role_id);
        self.member_repo.update(member).await
    }

    pub async fn create_team(
        &self,
        name: String,
        description: String,
        team_type: crate::entities::TeamType,
    ) -> Result<Team, RepositoryError> {
        let team = Team::new(name, description, team_type);
        self.team_repo.create(team).await
    }

    pub async fn add_member_to_team(
        &self,
        team_id: &Id<Team>,
        member_id: &Id<TeamMember>,
    ) -> Result<(Team, TeamMember), RepositoryError> {
        // Verify member exists and is active
        let mut member = self.member_repo.get(&member_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", member_id)
            ))?;

        if !member.active {
            return Err(RepositoryError::ValidationError(
                "Cannot add inactive member to team".to_string()
            ));
        }

        let mut team = self.team_repo.get(&team_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team {} not found", team_id)
            ))?;

        team.add_member(member_id.clone());
        member.add_team_membership(team_id.clone());

        let updated_team = self.team_repo.update(team).await?;
        let updated_member = self.member_repo.update(member).await?;

        Ok((updated_team, updated_member))
    }

    pub async fn remove_member_from_team(
        &self,
        team_id: &Id<Team>,
        member_id: &Id<TeamMember>,
    ) -> Result<(Team, TeamMember), RepositoryError> {
        let mut team = self.team_repo.get(&team_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team {} not found", team_id)
            ))?;

        let mut member = self.member_repo.get(&member_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", member_id)
            ))?;

        team.remove_member(member_id);
        member.remove_team_membership(team_id);

        let updated_team = self.team_repo.update(team).await?;
        let updated_member = self.member_repo.update(member).await?;

        Ok((updated_team, updated_member))
    }

    pub async fn set_team_lead(
        &self,
        team_id: &Id<Team>,
        lead_id: &Id<TeamMember>,
    ) -> Result<Team, RepositoryError> {
        // Verify member exists and is active
        let member = self.member_repo.get(&lead_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team member {} not found", lead_id)
            ))?;

        if !member.active {
            return Err(RepositoryError::ValidationError(
                "Cannot set inactive member as team lead".to_string()
            ));
        }

        let mut team = self.team_repo.get(&team_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Team {} not found", team_id)
            ))?;

        team.set_lead(lead_id.clone());
        self.team_repo.update(team).await
    }
}