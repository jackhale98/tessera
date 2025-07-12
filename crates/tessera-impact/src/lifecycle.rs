use crate::{EntityState, EntityReference, ApprovalLevel, ImpactAnalysis, convert_ron_error, convert_ron_spanned_error};
use tessera_core::{Id, Result, ProjectContext};
use tessera_team::{TeamRepository, Role, TeamMember};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use indexmap::IndexMap;

/// Manages entity lifecycle states and state transitions across modules
pub struct LifecycleManager {
    /// Current states of all tracked entities
    entity_states: IndexMap<Id, EntityLifecycleState>,
    /// Active approval workflows
    approval_workflows: IndexMap<Id, ApprovalWorkflow>,
    /// Lifecycle configuration rules
    config: LifecycleConfig,
}

/// Complete lifecycle state information for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityLifecycleState {
    pub entity_id: Id,
    pub entity_ref: EntityReference,
    pub current_state: EntityState,
    pub previous_state: Option<EntityState>,
    pub state_history: Vec<StateTransition>,
    pub locked: bool,
    pub locked_by: Option<Id>, // User/Team member ID
    pub locked_reason: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Record of a state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: EntityState,
    pub to_state: EntityState,
    pub timestamp: DateTime<Utc>,
    pub approver_id: Option<Id>,
    pub approval_workflow_id: Option<Id>,
    pub reason: String,
    pub impact_analysis_id: Option<Id>,
}

/// Approval workflow for entity state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalWorkflow {
    pub id: Id,
    pub entity_id: Id,
    pub requested_transition: StateTransition,
    pub required_approval_level: ApprovalLevel,
    pub status: ApprovalStatus,
    pub approvals: Vec<ApprovalRecord>,
    pub impact_analysis: Option<ImpactAnalysis>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
}

/// Individual approval record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub approver_id: Id,
    pub approval_level: ApprovalLevel,
    pub decision: ApprovalDecision,
    pub timestamp: DateTime<Utc>,
    pub comments: Option<String>,
}

/// Approval workflow status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    InProgress,
    Approved,
    Rejected,
    Expired,
    Cancelled,
}

/// Individual approval decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    RequestChanges,
}

/// Configuration for lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConfig {
    /// Whether to require approval for all state transitions
    pub require_approval_for_all_transitions: bool,
    /// Automatic state transitions based on conditions
    pub auto_transition_rules: Vec<AutoTransitionRule>,
    /// Default approval levels for different entity types
    pub default_approval_levels: HashMap<String, ApprovalLevel>,
    /// Escalation rules for overdue approvals
    pub escalation_rules: Vec<EscalationRule>,
}

/// Rule for automatic state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTransitionRule {
    pub from_state: EntityState,
    pub to_state: EntityState,
    pub conditions: Vec<String>, // Conditions that must be met
    pub delay_hours: Option<u32>, // Delay before auto-transition
}

/// Rule for escalating overdue approvals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub approval_level: ApprovalLevel,
    pub escalate_after_hours: u32,
    pub escalate_to_level: ApprovalLevel,
    pub notification_method: String,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            entity_states: IndexMap::new(),
            approval_workflows: IndexMap::new(),
            config: LifecycleConfig::default(),
        }
    }

    /// Load lifecycle data from project
    pub fn load_from_project(project_ctx: &ProjectContext) -> Result<Self> {
        let lifecycle_dir = project_ctx.root_path.join("impact");
        if !lifecycle_dir.exists() {
            std::fs::create_dir_all(&lifecycle_dir)?;
        }

        let mut manager = Self::new();

        // Load entity states
        let states_file = lifecycle_dir.join("entity_states.ron");
        if states_file.exists() {
            let content = std::fs::read_to_string(&states_file)?;
            manager.entity_states = ron::from_str(&content)?;
        }

        // Load approval workflows
        let workflows_file = lifecycle_dir.join("approval_workflows.ron");
        if workflows_file.exists() {
            let content = std::fs::read_to_string(&workflows_file)?;
            manager.approval_workflows = ron::from_str(&content)?;
        }

        // Load configuration
        let config_file = lifecycle_dir.join("lifecycle_config.ron");
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            manager.config = ron::from_str(&content)?;
        }

        Ok(manager)
    }

    /// Save lifecycle data to project
    pub fn save_to_project(&self, project_ctx: &ProjectContext) -> Result<()> {
        let lifecycle_dir = project_ctx.root_path.join("impact");
        std::fs::create_dir_all(&lifecycle_dir)?;

        // Save entity states
        let states_content = ron::ser::to_string_pretty(&self.entity_states, ron::ser::PrettyConfig::default())?;
        std::fs::write(lifecycle_dir.join("entity_states.ron"), states_content)?;

        // Save approval workflows
        let workflows_content = ron::ser::to_string_pretty(&self.approval_workflows, ron::ser::PrettyConfig::default())?;
        std::fs::write(lifecycle_dir.join("approval_workflows.ron"), workflows_content)?;

        // Save configuration
        let config_content = ron::ser::to_string_pretty(&self.config, ron::ser::PrettyConfig::default())?;
        std::fs::write(lifecycle_dir.join("lifecycle_config.ron"), config_content)?;

        Ok(())
    }

    /// Register an entity for lifecycle tracking
    pub fn register_entity(&mut self, entity_ref: EntityReference, initial_state: EntityState) -> Result<()> {
        let lifecycle_state = EntityLifecycleState {
            entity_id: entity_ref.id,
            entity_ref: entity_ref.clone(),
            current_state: initial_state,
            previous_state: None,
            state_history: Vec::new(),
            locked: false,
            locked_by: None,
            locked_reason: None,
            metadata: HashMap::new(),
            created: Utc::now(),
            updated: Utc::now(),
        };

        self.entity_states.insert(entity_ref.id, lifecycle_state);
        Ok(())
    }

    /// Request a state transition for an entity
    pub async fn request_state_transition(
        &mut self,
        entity_id: Id,
        target_state: EntityState,
        reason: String,
        requester_id: Id,
        impact_analysis: Option<ImpactAnalysis>,
        project_ctx: &ProjectContext
    ) -> Result<Option<Id>> {
        let entity_state = self.entity_states.get(&entity_id)
            .ok_or_else(|| tessera_core::DesignTrackError::Validation(
                "Entity not found in lifecycle tracking".to_string()
            ))?;

        let current_state = entity_state.current_state;

        // Check if transition is valid
        if !current_state.can_transition_to(target_state) {
            return Err(tessera_core::DesignTrackError::Validation(
                format!("Invalid transition from {:?} to {:?}", current_state, target_state)
            ));
        }

        // Check if entity is locked
        if entity_state.locked {
            return Err(tessera_core::DesignTrackError::Validation(
                "Entity is locked and cannot be modified".to_string()
            ));
        }

        // Determine if approval is required
        let required_approval_level = current_state.required_approval_level(target_state);
        
        if let Some(approval_level) = required_approval_level {
            // Create approval workflow
            let workflow_id = self.create_approval_workflow(
                entity_id,
                current_state,
                target_state,
                reason.clone(),
                approval_level,
                impact_analysis,
                project_ctx
            ).await?;

            Ok(Some(workflow_id))
        } else {
            // Direct transition without approval
            self.execute_state_transition(entity_id, target_state, reason, Some(requester_id), None, None)?;
            Ok(None)
        }
    }

    /// Create an approval workflow
    async fn create_approval_workflow(
        &mut self,
        entity_id: Id,
        from_state: EntityState,
        to_state: EntityState,
        reason: String,
        required_level: ApprovalLevel,
        impact_analysis: Option<ImpactAnalysis>,
        project_ctx: &ProjectContext
    ) -> Result<Id> {
        let transition = StateTransition {
            from_state,
            to_state,
            timestamp: Utc::now(),
            approver_id: None,
            approval_workflow_id: None,
            reason,
            impact_analysis_id: impact_analysis.as_ref().map(|a| a.id),
        };

        let workflow = ApprovalWorkflow {
            id: Id::new(),
            entity_id,
            requested_transition: transition,
            required_approval_level: required_level,
            status: ApprovalStatus::Pending,
            approvals: Vec::new(),
            impact_analysis,
            created: Utc::now(),
            updated: Utc::now(),
            deadline: Some(Utc::now() + chrono::Duration::days(7)), // Default 7-day deadline
        };

        let workflow_id = workflow.id;
        self.approval_workflows.insert(workflow_id, workflow);

        Ok(workflow_id)
    }

    /// Execute a state transition
    fn execute_state_transition(
        &mut self,
        entity_id: Id,
        target_state: EntityState,
        reason: String,
        approver_id: Option<Id>,
        workflow_id: Option<Id>,
        impact_analysis_id: Option<Id>
    ) -> Result<()> {
        let entity_state = self.entity_states.get_mut(&entity_id)
            .ok_or_else(|| tessera_core::DesignTrackError::Validation(
                "Entity not found".to_string()
            ))?;

        let transition = StateTransition {
            from_state: entity_state.current_state,
            to_state: target_state,
            timestamp: Utc::now(),
            approver_id,
            approval_workflow_id: workflow_id,
            reason,
            impact_analysis_id,
        };

        entity_state.previous_state = Some(entity_state.current_state);
        entity_state.current_state = target_state;
        entity_state.state_history.push(transition);
        entity_state.updated = Utc::now();

        Ok(())
    }

    /// Get current state of an entity
    pub fn get_entity_state(&self, entity_id: &Id) -> Option<EntityState> {
        self.entity_states.get(entity_id).map(|state| state.current_state)
    }

    /// Get entities in a specific state
    pub fn get_entities_in_state(&self, state: EntityState) -> Vec<&EntityLifecycleState> {
        self.entity_states.values()
            .filter(|entity_state| entity_state.current_state == state)
            .collect()
    }

    /// Get pending approval workflows
    pub fn get_pending_approvals(&self) -> Vec<&ApprovalWorkflow> {
        self.approval_workflows.values()
            .filter(|workflow| workflow.status == ApprovalStatus::Pending || workflow.status == ApprovalStatus::InProgress)
            .collect()
    }

    /// Process an approval decision
    pub async fn process_approval(
        &mut self,
        workflow_id: Id,
        approver_id: Id,
        decision: ApprovalDecision,
        comments: Option<String>,
        project_ctx: &ProjectContext
    ) -> Result<()> {
        // Load team repository to verify approver authority
        let team_repo = TeamRepository::load_from_project(project_ctx)?;
        let approver_level = self.get_approver_level(&team_repo, approver_id)?;

        let workflow = self.approval_workflows.get_mut(&workflow_id)
            .ok_or_else(|| tessera_core::DesignTrackError::Validation(
                "Approval workflow not found".to_string()
            ))?;

        // Verify approver has sufficient authority
        if approver_level < workflow.required_approval_level {
            return Err(tessera_core::DesignTrackError::Validation(
                "Insufficient approval authority".to_string()
            ));
        }

        // Record the approval
        let approval_record = ApprovalRecord {
            approver_id,
            approval_level: approver_level,
            decision: decision.clone(),
            timestamp: Utc::now(),
            comments,
        };

        workflow.approvals.push(approval_record);
        workflow.updated = Utc::now();

        // Extract needed data before borrowing conflicts
        let entity_id = workflow.entity_id;
        let to_state = workflow.requested_transition.to_state;
        let reason = workflow.requested_transition.reason.clone();
        let impact_analysis_id = workflow.requested_transition.impact_analysis_id;

        // Update workflow status based on decision
        match decision {
            ApprovalDecision::Approved => {
                workflow.status = ApprovalStatus::Approved;
                // Execute the state transition
                self.execute_state_transition(
                    entity_id,
                    to_state,
                    reason,
                    Some(approver_id),
                    Some(workflow_id),
                    impact_analysis_id
                )?;
            },
            ApprovalDecision::Rejected => {
                workflow.status = ApprovalStatus::Rejected;
            },
            ApprovalDecision::RequestChanges => {
                workflow.status = ApprovalStatus::InProgress;
            },
        }

        Ok(())
    }

    /// Get approver's authority level from team repository
    fn get_approver_level(&self, team_repo: &TeamRepository, approver_id: Id) -> Result<ApprovalLevel> {
        // TODO: Implement based on team member's role and authorities
        // For now, return a default level
        Ok(ApprovalLevel::TeamLead)
    }
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        let mut default_approval_levels = HashMap::new();
        default_approval_levels.insert("Requirement".to_string(), ApprovalLevel::Manager);
        default_approval_levels.insert("Risk".to_string(), ApprovalLevel::TeamLead);
        default_approval_levels.insert("DesignInput".to_string(), ApprovalLevel::Manager);
        default_approval_levels.insert("DesignOutput".to_string(), ApprovalLevel::TeamLead);
        default_approval_levels.insert("Verification".to_string(), ApprovalLevel::TeamLead);

        Self {
            require_approval_for_all_transitions: false,
            auto_transition_rules: Vec::new(),
            default_approval_levels,
            escalation_rules: Vec::new(),
        }
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}