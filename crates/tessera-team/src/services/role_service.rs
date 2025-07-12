use std::sync::Arc;
use tessera_core::{Id, Repository, RepositoryError};

use crate::{
    entities::{Role, GitApprovalAuthority, ApprovalContext, EntityState},
    repositories::RoleRepository,
};

pub struct RoleService {
    role_repo: Arc<RoleRepository>,
}

impl RoleService {
    pub fn new(role_repo: Arc<RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn create_role(
        &self,
        name: String,
        description: String,
    ) -> Result<Role, RepositoryError> {
        let role = Role::new(name, description);
        self.role_repo.create(role).await
    }

    pub async fn create_role_with_authority(
        &self,
        name: String,
        description: String,
        approval_authority: GitApprovalAuthority,
    ) -> Result<Role, RepositoryError> {
        let role = Role::new(name, description).with_approval_authority(approval_authority);
        self.role_repo.create(role).await
    }

    pub async fn update_role_approval_authority(
        &self,
        role_id: &Id<Role>,
        approval_authority: GitApprovalAuthority,
    ) -> Result<Role, RepositoryError> {
        let mut role = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Role {} not found", role_id)
            ))?;

        role.git_approval_authority = approval_authority;
        self.role_repo.update(role).await
    }

    pub async fn add_approval_path(
        &self,
        role_id: &Id<Role>,
        path: String,
    ) -> Result<Role, RepositoryError> {
        let mut role = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Role {} not found", role_id)
            ))?;

        if !role.git_approval_authority.can_approve_paths.contains(&path) {
            role.git_approval_authority.can_approve_paths.push(path);
        }

        self.role_repo.update(role).await
    }

    pub async fn remove_approval_path(
        &self,
        role_id: &Id<Role>,
        path: &str,
    ) -> Result<Role, RepositoryError> {
        let mut role = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Role {} not found", role_id)
            ))?;

        role.git_approval_authority.can_approve_paths.retain(|p| p != path);
        self.role_repo.update(role).await
    }

    pub async fn add_approval_context(
        &self,
        role_id: &Id<Role>,
        context: ApprovalContext,
    ) -> Result<Role, RepositoryError> {
        let mut role = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Role {} not found", role_id)
            ))?;

        role.git_approval_authority.approval_contexts.push(context);
        self.role_repo.update(role).await
    }

    pub async fn set_cost_approval_limit(
        &self,
        role_id: &Id<Role>,
        limit: Option<f64>,
    ) -> Result<Role, RepositoryError> {
        let mut role = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Role {} not found", role_id)
            ))?;

        role.git_approval_authority.max_cost_approval = limit;
        self.role_repo.update(role).await
    }

    pub async fn set_schedule_impact_limit(
        &self,
        role_id: &Id<Role>,
        limit: Option<u32>,
    ) -> Result<Role, RepositoryError> {
        let mut role = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Role {} not found", role_id)
            ))?;

        role.git_approval_authority.max_schedule_impact_days = limit;
        self.role_repo.update(role).await
    }

    pub async fn check_approval_authority(
        &self,
        role_id: &Id<Role>,
        path: &str,
        module: &str,
        state: EntityState,
        impact: Option<f64>,
    ) -> Result<bool, RepositoryError> {
        let role = self.role_repo.get(&role_id.to_string()).await?
            .ok_or_else(|| RepositoryError::NotFoundError(
                format!("Role {} not found", role_id)
            ))?;

        Ok(role.can_approve_path(path) && role.can_approve_in_context(module, state, impact))
    }

    pub async fn create_default_roles(&self) -> Result<Vec<Role>, RepositoryError> {
        let roles = vec![
            self.create_quality_engineer_role().await?,
            self.create_safety_engineer_role().await?,
            self.create_test_engineer_role().await?,
            self.create_lead_engineer_role().await?,
            self.create_manager_role().await?,
        ];

        Ok(roles)
    }

    async fn create_quality_engineer_role(&self) -> Result<Role, RepositoryError> {
        let authority = GitApprovalAuthority {
            can_approve_paths: vec![
                "requirements/".to_string(),
                "risk/".to_string(),
                "verification/".to_string(),
            ],
            approval_contexts: vec![
                ApprovalContext {
                    entity_states: vec![EntityState::InReview, EntityState::Approved],
                    modules: vec!["requirements".to_string(), "risk".to_string(), "verification".to_string()],
                    impact_threshold: None,
                },
            ],
            max_cost_approval: Some(25000.0),
            max_schedule_impact_days: Some(30),
        };

        self.create_role_with_authority(
            "Quality Engineer".to_string(),
            "Responsible for quality assurance and compliance activities".to_string(),
            authority,
        ).await
    }

    async fn create_safety_engineer_role(&self) -> Result<Role, RepositoryError> {
        let authority = GitApprovalAuthority {
            can_approve_paths: vec![
                "risk/".to_string(),
                "verification/".to_string(),
            ],
            approval_contexts: vec![
                ApprovalContext {
                    entity_states: vec![EntityState::InReview, EntityState::Approved, EntityState::Released],
                    modules: vec!["risk".to_string(), "verification".to_string()],
                    impact_threshold: None,
                },
            ],
            max_cost_approval: Some(50000.0),
            max_schedule_impact_days: Some(60),
        };

        self.create_role_with_authority(
            "Safety Engineer".to_string(),
            "Responsible for safety analysis and risk management".to_string(),
            authority,
        ).await
    }

    async fn create_test_engineer_role(&self) -> Result<Role, RepositoryError> {
        let authority = GitApprovalAuthority {
            can_approve_paths: vec![
                "verification/".to_string(),
                "tol/".to_string(),
            ],
            approval_contexts: vec![
                ApprovalContext {
                    entity_states: vec![EntityState::InReview],
                    modules: vec!["verification".to_string(), "tol".to_string()],
                    impact_threshold: None,
                },
            ],
            max_cost_approval: Some(10000.0),
            max_schedule_impact_days: Some(14),
        };

        self.create_role_with_authority(
            "Test Engineer".to_string(),
            "Responsible for test design and verification activities".to_string(),
            authority,
        ).await
    }

    async fn create_lead_engineer_role(&self) -> Result<Role, RepositoryError> {
        let authority = GitApprovalAuthority {
            can_approve_paths: vec![
                "*".to_string(), // Can approve all paths
            ],
            approval_contexts: vec![
                ApprovalContext {
                    entity_states: vec![EntityState::InReview, EntityState::Approved],
                    modules: vec!["*".to_string()],
                    impact_threshold: None,
                },
                ApprovalContext {
                    entity_states: vec![EntityState::Released],
                    modules: vec!["*".to_string()],
                    impact_threshold: Some(10.0),
                },
            ],
            max_cost_approval: Some(100000.0),
            max_schedule_impact_days: Some(90),
        };

        self.create_role_with_authority(
            "Lead Engineer".to_string(),
            "Responsible for technical leadership and major approvals".to_string(),
            authority,
        ).await
    }

    async fn create_manager_role(&self) -> Result<Role, RepositoryError> {
        let authority = GitApprovalAuthority {
            can_approve_paths: vec![
                "*".to_string(), // Can approve all paths
            ],
            approval_contexts: vec![
                ApprovalContext {
                    entity_states: vec![EntityState::Released],
                    modules: vec!["*".to_string()],
                    impact_threshold: None, // No threshold - can approve any impact
                },
            ],
            max_cost_approval: None, // No limit
            max_schedule_impact_days: None, // No limit
        };

        self.create_role_with_authority(
            "Manager".to_string(),
            "Responsible for management oversight and final approvals".to_string(),
            authority,
        ).await
    }
}