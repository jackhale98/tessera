use std::sync::Arc;
use tessera_core::{Id, Repository, RepositoryError};

use crate::{
    entities::{
        TeamMember, Role, Team, ApprovalRule, ApprovalRequirement, 
        ResolvedApprovalGroup, EntityState
    },
    repositories::{
        TeamMemberRepository, TeamMemberRepositoryExt, RoleRepository, 
        TeamRepository, TeamRepositoryExt, ApprovalRuleRepository, ApprovalRuleRepositoryExt
    },
};

pub struct GitApprovalManager {
    member_repo: Arc<TeamMemberRepository>,
    role_repo: Arc<RoleRepository>,
    team_repo: Arc<TeamRepository>,
    approval_rules_repo: Arc<ApprovalRuleRepository>,
}

impl GitApprovalManager {
    pub fn new(
        member_repo: Arc<TeamMemberRepository>,
        role_repo: Arc<RoleRepository>,
        team_repo: Arc<TeamRepository>,
        approval_rules_repo: Arc<ApprovalRuleRepository>,
    ) -> Self {
        Self {
            member_repo,
            role_repo,
            team_repo,
            approval_rules_repo,
        }
    }

    pub async fn get_required_approvers(
        &self,
        module_path: &str,
        entity_state: EntityState,
        change_impact: f64,
    ) -> Result<ApprovalRequirement, RepositoryError> {
        let rules = self.approval_rules_repo
            .get_rules_for_path_and_state(module_path, entity_state)
            .await?;

        for rule in rules {
            if rule.applies_to_impact(change_impact) {
                return self.resolve_approval_requirement(&rule).await;
            }
        }

        // Default fallback - require one approval from any active team member with git username
        let active_members = self.member_repo.get_active_members().await?;
        let git_usernames: Vec<String> = active_members
            .iter()
            .filter_map(|m| m.git_username.as_ref().map(|u| format!("@{}", u)))
            .collect();

        let default_group = ResolvedApprovalGroup {
            group_name: "Default".to_string(),
            eligible_approvers: git_usernames,
            approvals_needed_from_group: 1,
        };

        let mut requirement = ApprovalRequirement::new(1, false);
        requirement.add_group(default_group);
        Ok(requirement)
    }

    pub async fn generate_codeowners_for_state(
        &self,
        entity_state: EntityState,
    ) -> Result<String, RepositoryError> {
        let mut codeowners_lines = Vec::new();
        let rules = self.approval_rules_repo.get_rules_for_state(entity_state).await?;

        for rule in rules {
            let requirement = self.resolve_approval_requirement(&rule).await?;
            let approvers: Vec<String> = requirement.approval_groups
                .iter()
                .flat_map(|g| g.eligible_approvers.iter())
                .cloned()
                .collect();

            if !approvers.is_empty() {
                let line = format!("{} {}", rule.path_pattern, approvers.join(" "));
                codeowners_lines.push(line);
            }
        }

        Ok(codeowners_lines.join("\n"))
    }

    pub async fn validate_git_usernames(&self) -> Result<Vec<String>, RepositoryError> {
        let mut errors = Vec::new();
        let members = self.member_repo.get_active_members().await?;

        for member in members {
            if let Some(git_username) = &member.git_username {
                // Basic validation - ensure username is not empty and follows GitHub rules
                if git_username.is_empty() {
                    errors.push(format!(
                        "Member '{}' has empty git username", 
                        member.full_name()
                    ));
                } else if !self.is_valid_git_username(git_username) {
                    errors.push(format!(
                        "Member '{}' has invalid git username '{}' - must contain only alphanumeric characters, dashes, and underscores",
                        member.full_name(),
                        git_username
                    ));
                }
            } else if member.active {
                errors.push(format!(
                    "Active member '{}' has no git username configured",
                    member.full_name()
                ));
            }
        }

        Ok(errors)
    }

    pub async fn get_approvers_for_module(
        &self,
        module: &str,
        state: EntityState,
    ) -> Result<Vec<String>, RepositoryError> {
        let rules = self.approval_rules_repo
            .get_rules_for_path_and_state(&format!("{}/", module), state)
            .await?;

        let mut all_approvers = Vec::new();
        for rule in rules {
            let requirement = self.resolve_approval_requirement(&rule).await?;
            for group in requirement.approval_groups {
                all_approvers.extend(group.eligible_approvers);
            }
        }

        // Remove duplicates and sort
        all_approvers.sort();
        all_approvers.dedup();
        Ok(all_approvers)
    }

    async fn resolve_approval_requirement(
        &self,
        rule: &ApprovalRule,
    ) -> Result<ApprovalRequirement, RepositoryError> {
        let mut requirement = ApprovalRequirement::new(
            rule.min_approvals_required,
            rule.all_groups_required,
        );

        for approval_group in &rule.required_approver_groups {
            let mut eligible_approvers = Vec::new();

            // Add specific approvers
            for approver_id in &approval_group.required_approvers {
                if let Some(member) = self.member_repo.get(&approver_id.to_string()).await? {
                    if member.active {
                        if let Some(git_username) = &member.git_username {
                            eligible_approvers.push(format!("@{}", git_username));
                        }
                    }
                }
            }

            // Add approvers by role
            for role_id in &approval_group.required_roles {
                let members = self.member_repo.get_members_by_role(role_id).await?;
                for member in members {
                    if member.active {
                        if let Some(git_username) = &member.git_username {
                            eligible_approvers.push(format!("@{}", git_username));
                        }
                    }
                }
            }

            // Add approvers by team
            for team_id in &approval_group.required_teams {
                if let Some(team) = self.team_repo.get(&team_id.to_string()).await? {
                    for member_id in &team.members {
                        if let Some(member) = self.member_repo.get(&member_id.to_string()).await? {
                            if member.active {
                                if let Some(git_username) = &member.git_username {
                                    eligible_approvers.push(format!("@{}", git_username));
                                }
                            }
                        }
                    }
                }
            }

            // Remove duplicates
            eligible_approvers.sort();
            eligible_approvers.dedup();

            let resolved_group = ResolvedApprovalGroup {
                group_name: approval_group.name.clone(),
                eligible_approvers,
                approvals_needed_from_group: approval_group.min_approvals_from_group,
            };

            requirement.add_group(resolved_group);
        }

        Ok(requirement)
    }

    fn is_valid_git_username(&self, username: &str) -> bool {
        // GitHub username rules: alphanumeric + dashes/underscores, no consecutive dashes,
        // no leading/trailing dashes
        let re = regex::Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-_]{0,37}[a-zA-Z0-9])?$").unwrap();
        re.is_match(username) && !username.contains("--")
    }

    pub async fn sync_git_teams(&self) -> Result<Vec<String>, RepositoryError> {
        let mut sync_results = Vec::new();
        let teams = self.team_repo.list().await?;

        for team in teams {
            if let Some(git_team_name) = &team.git_team_name {
                let members_with_git: Vec<String> = self.get_team_members_with_git(&team).await?;
                
                if members_with_git.is_empty() {
                    sync_results.push(format!(
                        "Warning: Team '{}' (git: {}) has no members with git usernames",
                        team.name, git_team_name
                    ));
                } else {
                    sync_results.push(format!(
                        "Team '{}' (git: {}) has {} members: {}",
                        team.name,
                        git_team_name,
                        members_with_git.len(),
                        members_with_git.join(", ")
                    ));
                }
            }
        }

        Ok(sync_results)
    }

    async fn get_team_members_with_git(&self, team: &Team) -> Result<Vec<String>, RepositoryError> {
        let mut git_usernames = Vec::new();
        
        for member_id in &team.members {
            if let Some(member) = self.member_repo.get(&member_id.to_string()).await? {
                if member.active {
                    if let Some(git_username) = &member.git_username {
                        git_usernames.push(git_username.clone());
                    }
                }
            }
        }

        git_usernames.sort();
        Ok(git_usernames)
    }
}