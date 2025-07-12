use crate::{ImpactSeverity, ApprovalLevel, ModuleType, ImpactType, convert_ron_error, convert_ron_spanned_error};
use tessera_core::{Id, Result, ProjectContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;

/// Configuration management for impact analysis rules and workflows
pub struct ConfigurationManager {
    pub impact_config: ImpactConfiguration,
    pub workflow_config: WorkflowConfiguration,
    pub team_config: TeamConfiguration,
}

/// Configuration for impact analysis rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactConfiguration {
    /// Rules for determining impact severity
    pub severity_rules: SeverityConfiguration,
    /// Cross-module impact multipliers
    pub module_multipliers: HashMap<ModuleType, f32>,
    /// Entity type impact weights
    pub entity_weights: HashMap<String, f32>,
    /// Maximum analysis depth for impact traversal
    pub max_analysis_depth: u32,
    /// Threshold for requiring impact analysis
    pub analysis_threshold: ImpactSeverity,
}

/// Rules for calculating impact severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityConfiguration {
    /// Base severity for different impact types
    pub impact_type_base_severity: HashMap<ImpactType, ImpactSeverity>,
    /// Multipliers for different entity relationships
    pub relationship_multipliers: HashMap<String, f32>,
    /// Depth degradation factor (0.0 - 1.0)
    pub depth_degradation_factor: f32,
    /// Module cross-impact factors
    pub cross_module_factors: HashMap<(ModuleType, ModuleType), f32>,
}

/// Configuration for approval workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfiguration {
    /// Git branch naming patterns
    pub branch_naming: BranchNamingConfig,
    /// Pull request templates
    pub pr_templates: PRTemplateConfig,
    /// Approval deadlines by level
    pub approval_deadlines: HashMap<ApprovalLevel, u32>, // hours
    /// Escalation rules
    pub escalation_config: EscalationConfig,
    /// Notification settings
    pub notification_config: NotificationConfig,
}

/// Git branch naming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchNamingConfig {
    /// Prefix for impact analysis branches
    pub impact_branch_prefix: String,
    /// Pattern for feature branches: {prefix}/{module}/{entity_type}/{entity_id}
    pub feature_branch_pattern: String,
    /// Pattern for hotfix branches
    pub hotfix_branch_pattern: String,
    /// Pattern for release branches
    pub release_branch_pattern: String,
}

/// Pull request template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRTemplateConfig {
    /// Template for impact analysis PRs
    pub impact_analysis_template: String,
    /// Template for entity updates
    pub entity_update_template: String,
    /// Template for cross-module changes
    pub cross_module_template: String,
    /// Custom labels for different impact types
    pub impact_labels: HashMap<ImpactSeverity, Vec<String>>,
}

/// Escalation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Enable automatic escalation
    pub auto_escalation_enabled: bool,
    /// Escalation rules by approval level
    pub escalation_rules: HashMap<ApprovalLevel, EscalationRule>,
    /// Maximum escalation levels
    pub max_escalation_levels: u32,
}

/// Individual escalation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    /// Hours before escalation triggers
    pub escalate_after_hours: u32,
    /// Next approval level to escalate to
    pub escalate_to_level: ApprovalLevel,
    /// Whether to notify the original approver
    pub notify_original_approver: bool,
    /// Custom escalation message
    pub escalation_message: Option<String>,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable email notifications
    pub email_enabled: bool,
    /// Enable Slack notifications
    pub slack_enabled: bool,
    /// Enable GitHub notifications
    pub github_enabled: bool,
    /// Custom notification templates
    pub templates: HashMap<String, String>,
    /// Notification frequency settings
    pub frequency_settings: FrequencySettings,
}

/// Notification frequency settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencySettings {
    /// Initial notification delay (minutes)
    pub initial_delay_minutes: u32,
    /// Reminder frequency (hours)
    pub reminder_frequency_hours: u32,
    /// Maximum number of reminders
    pub max_reminders: u32,
}

/// Team-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfiguration {
    /// Team-specific approval rules
    pub team_approval_rules: IndexMap<Id, TeamApprovalRule>,
    /// Module ownership assignments
    pub module_ownership: HashMap<ModuleType, Vec<Id>>, // Team IDs
    /// Entity type ownership
    pub entity_ownership: HashMap<String, Vec<Id>>, // Team member IDs
    /// Override rules for specific entities
    pub entity_overrides: HashMap<Id, ApprovalOverride>,
}

/// Team-specific approval rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamApprovalRule {
    pub team_id: Id,
    pub module_types: Vec<ModuleType>,
    pub entity_types: Vec<String>,
    pub required_approval_level: ApprovalLevel,
    pub parallel_approval_required: bool,
    pub custom_rules: Vec<String>,
}

/// Approval override for specific entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalOverride {
    pub entity_id: Id,
    pub required_level: ApprovalLevel,
    pub specific_approvers: Vec<Id>,
    pub bypass_normal_workflow: bool,
    pub custom_workflow_id: Option<Id>,
}

impl ConfigurationManager {
    pub fn new() -> Self {
        Self {
            impact_config: ImpactConfiguration::default(),
            workflow_config: WorkflowConfiguration::default(),
            team_config: TeamConfiguration::default(),
        }
    }

    /// Load configuration from project
    pub fn load_from_project(project_ctx: &ProjectContext) -> Result<Self> {
        let config_dir = project_ctx.root_path.join("impact").join("config");
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
            // Create default configuration files
            let manager = Self::new();
            manager.save_to_project(project_ctx)?;
            return Ok(manager);
        }

        let mut manager = Self::new();

        // Load impact configuration
        let impact_config_file = config_dir.join("impact_config.ron");
        if impact_config_file.exists() {
            let content = std::fs::read_to_string(&impact_config_file)?;
            manager.impact_config = ron::from_str(&content)?;
        }

        // Load workflow configuration
        let workflow_config_file = config_dir.join("workflow_config.ron");
        if workflow_config_file.exists() {
            let content = std::fs::read_to_string(&workflow_config_file)?;
            manager.workflow_config = ron::from_str(&content)?;
        }

        // Load team configuration
        let team_config_file = config_dir.join("team_config.ron");
        if team_config_file.exists() {
            let content = std::fs::read_to_string(&team_config_file)?;
            manager.team_config = ron::from_str(&content)?;
        }

        Ok(manager)
    }

    /// Save configuration to project
    pub fn save_to_project(&self, project_ctx: &ProjectContext) -> Result<()> {
        let config_dir = project_ctx.root_path.join("impact").join("config");
        std::fs::create_dir_all(&config_dir)?;

        // Save impact configuration
        let impact_content = ron::ser::to_string_pretty(&self.impact_config, ron::ser::PrettyConfig::default())?;
        std::fs::write(config_dir.join("impact_config.ron"), impact_content)?;

        // Save workflow configuration
        let workflow_content = ron::ser::to_string_pretty(&self.workflow_config, ron::ser::PrettyConfig::default())?;
        std::fs::write(config_dir.join("workflow_config.ron"), workflow_content)?;

        // Save team configuration
        let team_content = ron::ser::to_string_pretty(&self.team_config, ron::ser::PrettyConfig::default())?;
        std::fs::write(config_dir.join("team_config.ron"), team_content)?;

        Ok(())
    }

    /// Get severity configuration for impact analysis
    pub fn get_severity_rules(&self) -> &SeverityConfiguration {
        &self.impact_config.severity_rules
    }

    /// Get workflow configuration
    pub fn get_workflow_config(&self) -> &WorkflowConfiguration {
        &self.workflow_config
    }

    /// Get team configuration for approvals
    pub fn get_team_config(&self) -> &TeamConfiguration {
        &self.team_config
    }

    /// Update impact configuration
    pub fn update_impact_config(&mut self, config: ImpactConfiguration) {
        self.impact_config = config;
    }

    /// Add team approval rule
    pub fn add_team_approval_rule(&mut self, rule: TeamApprovalRule) {
        self.team_config.team_approval_rules.insert(rule.team_id, rule);
    }

    /// Get approval level required for entity
    pub fn get_required_approval_level(&self, entity_id: &Id, entity_type: &str) -> ApprovalLevel {
        // Check for entity-specific overrides first
        if let Some(override_rule) = self.team_config.entity_overrides.get(entity_id) {
            return override_rule.required_level;
        }

        // Check team-specific rules
        for rule in self.team_config.team_approval_rules.values() {
            if rule.entity_types.contains(&entity_type.to_string()) {
                return rule.required_approval_level;
            }
        }

        // Fall back to default
        ApprovalLevel::TeamLead
    }

    /// Generate branch name for impact analysis
    pub fn generate_branch_name(&self, module: &ModuleType, entity_type: &str, entity_id: &Id) -> String {
        format!("{}/{}/{}/{}",
            self.workflow_config.branch_naming.impact_branch_prefix,
            module.to_string().to_lowercase(),
            entity_type.to_lowercase(),
            entity_id.to_string()
        )
    }

    /// Get PR template for impact analysis
    pub fn get_pr_template(&self, severity: ImpactSeverity) -> &str {
        match severity {
            ImpactSeverity::Critical | ImpactSeverity::High => {
                &self.workflow_config.pr_templates.cross_module_template
            },
            _ => {
                &self.workflow_config.pr_templates.impact_analysis_template
            }
        }
    }
}

// Default implementations for all configuration types
impl Default for ImpactConfiguration {
    fn default() -> Self {
        Self {
            severity_rules: SeverityConfiguration::default(),
            module_multipliers: create_default_module_multipliers(),
            entity_weights: create_default_entity_weights(),
            max_analysis_depth: 5,
            analysis_threshold: ImpactSeverity::Medium,
        }
    }
}

impl Default for SeverityConfiguration {
    fn default() -> Self {
        let mut impact_type_severity = HashMap::new();
        impact_type_severity.insert(ImpactType::DirectLink, ImpactSeverity::Medium);
        impact_type_severity.insert(ImpactType::IndirectLink, ImpactSeverity::Low);
        impact_type_severity.insert(ImpactType::StateChange, ImpactSeverity::High);
        impact_type_severity.insert(ImpactType::RequirementChange, ImpactSeverity::High);
        impact_type_severity.insert(ImpactType::RiskChange, ImpactSeverity::Medium);
        impact_type_severity.insert(ImpactType::VerificationChange, ImpactSeverity::Medium);

        Self {
            impact_type_base_severity: impact_type_severity,
            relationship_multipliers: HashMap::new(),
            depth_degradation_factor: 0.8,
            cross_module_factors: create_default_cross_module_factors(),
        }
    }
}

impl Default for WorkflowConfiguration {
    fn default() -> Self {
        Self {
            branch_naming: BranchNamingConfig::default(),
            pr_templates: PRTemplateConfig::default(),
            approval_deadlines: create_default_approval_deadlines(),
            escalation_config: EscalationConfig::default(),
            notification_config: NotificationConfig::default(),
        }
    }
}

impl Default for BranchNamingConfig {
    fn default() -> Self {
        Self {
            impact_branch_prefix: "impact".to_string(),
            feature_branch_pattern: "feature/{module}/{entity_type}/{entity_id}".to_string(),
            hotfix_branch_pattern: "hotfix/{module}/{entity_type}/{entity_id}".to_string(),
            release_branch_pattern: "release/{version}".to_string(),
        }
    }
}

impl Default for PRTemplateConfig {
    fn default() -> Self {
        Self {
            impact_analysis_template: create_default_pr_template(),
            entity_update_template: "## Entity Update\n\n**Entity:** {entity_name}\n**Module:** {module}\n**Changes:** {changes}\n\n## Impact Analysis\n{impact_summary}".to_string(),
            cross_module_template: "## Cross-Module Impact Analysis\n\n**Source:** {source_entity}\n**Affected Modules:** {affected_modules}\n**Impact Summary:** {impact_summary}\n\n## Required Approvals\n{approval_requirements}".to_string(),
            impact_labels: create_default_impact_labels(),
        }
    }
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            auto_escalation_enabled: true,
            escalation_rules: create_default_escalation_rules(),
            max_escalation_levels: 3,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            email_enabled: true,
            slack_enabled: false,
            github_enabled: true,
            templates: HashMap::new(),
            frequency_settings: FrequencySettings::default(),
        }
    }
}

impl Default for FrequencySettings {
    fn default() -> Self {
        Self {
            initial_delay_minutes: 30,
            reminder_frequency_hours: 24,
            max_reminders: 3,
        }
    }
}

impl Default for TeamConfiguration {
    fn default() -> Self {
        Self {
            team_approval_rules: IndexMap::new(),
            module_ownership: HashMap::new(),
            entity_ownership: HashMap::new(),
            entity_overrides: HashMap::new(),
        }
    }
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for creating default configurations
fn create_default_module_multipliers() -> HashMap<ModuleType, f32> {
    let mut multipliers = HashMap::new();
    multipliers.insert(ModuleType::Requirements, 1.5);
    multipliers.insert(ModuleType::Risk, 1.3);
    multipliers.insert(ModuleType::Verification, 1.2);
    multipliers.insert(ModuleType::ProjectManagement, 1.1);
    multipliers.insert(ModuleType::ToleranceAnalysis, 1.0);
    multipliers.insert(ModuleType::Team, 0.8);
    multipliers
}

fn create_default_entity_weights() -> HashMap<String, f32> {
    let mut weights = HashMap::new();
    weights.insert("Requirement".to_string(), 1.5);
    weights.insert("DesignInput".to_string(), 1.4);
    weights.insert("Risk".to_string(), 1.3);
    weights.insert("DesignOutput".to_string(), 1.2);
    weights.insert("Verification".to_string(), 1.1);
    weights.insert("Task".to_string(), 1.0);
    weights.insert("Component".to_string(), 1.0);
    weights
}

fn create_default_cross_module_factors() -> HashMap<(ModuleType, ModuleType), f32> {
    let mut factors = HashMap::new();
    factors.insert((ModuleType::Requirements, ModuleType::Risk), 1.4);
    factors.insert((ModuleType::Requirements, ModuleType::Verification), 1.3);
    factors.insert((ModuleType::Risk, ModuleType::Verification), 1.2);
    factors.insert((ModuleType::Requirements, ModuleType::ProjectManagement), 1.1);
    factors
}

fn create_default_approval_deadlines() -> HashMap<ApprovalLevel, u32> {
    let mut deadlines = HashMap::new();
    deadlines.insert(ApprovalLevel::TeamMember, 24);
    deadlines.insert(ApprovalLevel::TeamLead, 48);
    deadlines.insert(ApprovalLevel::Manager, 72);
    deadlines.insert(ApprovalLevel::Director, 120);
    deadlines.insert(ApprovalLevel::Executive, 168);
    deadlines
}

fn create_default_escalation_rules() -> HashMap<ApprovalLevel, EscalationRule> {
    let mut rules = HashMap::new();
    rules.insert(ApprovalLevel::TeamMember, EscalationRule {
        escalate_after_hours: 24,
        escalate_to_level: ApprovalLevel::TeamLead,
        notify_original_approver: true,
        escalation_message: Some("Approval request escalated to team lead due to timeout".to_string()),
    });
    rules.insert(ApprovalLevel::TeamLead, EscalationRule {
        escalate_after_hours: 48,
        escalate_to_level: ApprovalLevel::Manager,
        notify_original_approver: true,
        escalation_message: Some("Approval request escalated to manager due to timeout".to_string()),
    });
    rules
}

fn create_default_impact_labels() -> HashMap<ImpactSeverity, Vec<String>> {
    let mut labels = HashMap::new();
    labels.insert(ImpactSeverity::Low, vec!["impact:low".to_string(), "review:routine".to_string()]);
    labels.insert(ImpactSeverity::Medium, vec!["impact:medium".to_string(), "review:standard".to_string()]);
    labels.insert(ImpactSeverity::High, vec!["impact:high".to_string(), "review:thorough".to_string()]);
    labels.insert(ImpactSeverity::Critical, vec!["impact:critical".to_string(), "review:comprehensive".to_string(), "priority:urgent".to_string()]);
    labels
}

fn create_default_pr_template() -> String {
    r#"## Impact Analysis Summary

**Source Entity:** {source_entity}
**Change Description:** {change_description}
**Analysis Timestamp:** {timestamp}

## Affected Entities

{affected_entities_table}

## Impact Severity

**Maximum Severity:** {max_severity}
**Total Affected:** {total_affected}
**Estimated Effort:** {estimated_effort} hours

## Approval Requirements

**Required Level:** {required_approval_level}
**Approval Workflow:** {workflow_status}

## Additional Notes

{additional_notes}

---
*This PR was automatically generated by Tessera Impact Analysis*"#.to_string()
}