use crate::{ImpactAnalysis, ApprovalWorkflow, ApprovalStatus, ConfigurationManager, EntityReference, convert_git_error};
use tessera_core::{Id, Result, ProjectContext};
use tessera_team::TeamRepository;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use git2::{Repository, Branch, BranchType, Signature, ObjectType};
use std::collections::HashMap;
use std::path::Path;

/// Git workflow orchestrator for impact analysis and approval processes
pub struct GitWorkflowOrchestrator {
    /// Git repository handle
    repository: Repository,
    /// Configuration manager
    config: ConfigurationManager,
    /// Active Git workflows
    active_workflows: HashMap<Id, GitWorkflow>,
}

/// Represents a Git workflow for impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitWorkflow {
    pub id: Id,
    pub approval_workflow_id: Id,
    pub impact_analysis_id: Id,
    pub branch_name: String,
    pub pull_request_url: Option<String>,
    pub status: GitWorkflowStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub commits: Vec<GitCommitInfo>,
}

/// Git workflow status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitWorkflowStatus {
    BranchCreated,
    CommitsPushed,
    PullRequestCreated,
    ReviewInProgress,
    Approved,
    Merged,
    Rejected,
    Cancelled,
}

/// Information about a Git commit in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitInfo {
    pub commit_hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub files_changed: Vec<String>,
}

/// Pull request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub title: String,
    pub description: String,
    pub labels: Vec<String>,
    pub reviewers: Vec<String>,
    pub assignees: Vec<String>,
    pub milestone: Option<String>,
}

impl GitWorkflowOrchestrator {
    /// Create a new Git workflow orchestrator
    pub fn new(project_ctx: &ProjectContext, config: ConfigurationManager) -> Result<Self> {
        let repository = Repository::open(&project_ctx.root_path)?;
        
        Ok(Self {
            repository,
            config,
            active_workflows: HashMap::new(),
        })
    }

    /// Create a new Git workflow for an impact analysis
    pub async fn create_workflow(
        &mut self,
        impact_analysis: &ImpactAnalysis,
        approval_workflow: &ApprovalWorkflow,
        project_ctx: &ProjectContext
    ) -> Result<Id> {
        let workflow_id = Id::new();
        
        // Generate branch name
        let branch_name = self.config.generate_branch_name(
            &impact_analysis.source_entity.module,
            &impact_analysis.source_entity.entity_type,
            &impact_analysis.source_entity.id
        );

        // Create Git branch
        self.create_feature_branch(&branch_name)?;

        // Create workflow record
        let git_workflow = GitWorkflow {
            id: workflow_id,
            approval_workflow_id: approval_workflow.id,
            impact_analysis_id: impact_analysis.id,
            branch_name: branch_name.clone(),
            pull_request_url: None,
            status: GitWorkflowStatus::BranchCreated,
            created: Utc::now(),
            updated: Utc::now(),
            commits: Vec::new(),
        };

        self.active_workflows.insert(workflow_id, git_workflow);

        // Create initial commit with impact analysis
        self.commit_impact_analysis(&branch_name, impact_analysis, approval_workflow).await?;

        // Create pull request if configured
        if self.should_auto_create_pr(impact_analysis) {
            self.create_pull_request(&workflow_id, impact_analysis, project_ctx).await?;
        }

        Ok(workflow_id)
    }

    /// Create a feature branch for the impact analysis
    fn create_feature_branch(&self, branch_name: &str) -> Result<()> {
        // Get current HEAD commit
        let head = self.repository.head()?;
        let target_commit = head.peel_to_commit()?;

        // Create new branch
        self.repository.branch(branch_name, &target_commit, false)?;

        Ok(())
    }

    /// Commit impact analysis data to the branch
    async fn commit_impact_analysis(
        &mut self,
        branch_name: &str,
        impact_analysis: &ImpactAnalysis,
        approval_workflow: &ApprovalWorkflow
    ) -> Result<()> {
        // Switch to the branch
        let branch_ref = format!("refs/heads/{}", branch_name);
        self.repository.set_head(&branch_ref)?;

        // Create impact analysis files
        let impact_dir = self.repository.workdir()
            .ok_or_else(|| tessera_core::DesignTrackError::Validation("No working directory".to_string()))?
            .join("impact")
            .join("analyses");

        std::fs::create_dir_all(&impact_dir)?;

        // Write impact analysis data
        let analysis_file = impact_dir.join(format!("impact_analysis_{}.ron", impact_analysis.id));
        let analysis_content = ron::ser::to_string_pretty(impact_analysis, ron::ser::PrettyConfig::default())?;
        std::fs::write(&analysis_file, analysis_content)?;

        // Write approval workflow data
        let workflow_file = impact_dir.join(format!("approval_workflow_{}.ron", approval_workflow.id));
        let workflow_content = ron::ser::to_string_pretty(approval_workflow, ron::ser::PrettyConfig::default())?;
        std::fs::write(&workflow_file, workflow_content)?;

        // Stage files
        let mut index = self.repository.index()?;
        index.add_path(Path::new(&format!("impact/analyses/impact_analysis_{}.ron", impact_analysis.id)))?;
        index.add_path(Path::new(&format!("impact/analyses/approval_workflow_{}.ron", approval_workflow.id)))?;
        index.write()?;

        // Create commit
        let signature = self.get_git_signature()?;
        let author_name = signature.name().unwrap_or("Unknown").to_string();
        
        let tree_id = index.write_tree()?;
        let tree = self.repository.find_tree(tree_id)?;
        let parent_commit = self.repository.head()?.peel_to_commit()?;

        let commit_message = format!(
            "Add impact analysis for {} {}\n\nSeverity: {}\nAffected entities: {}\nEstimated effort: {:.1} hours",
            impact_analysis.source_entity.module,
            impact_analysis.source_entity.name,
            impact_analysis.max_severity,
            impact_analysis.total_affected_entities,
            impact_analysis.estimated_total_effort_hours
        );

        let commit_id = self.repository.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &commit_message,
            &tree,
            &[&parent_commit]
        )?;

        // Drop signature to release borrow
        drop(signature);

        // Record commit info
        let commit_info = GitCommitInfo {
            commit_hash: commit_id.to_string(),
            message: commit_message.clone(),
            author: author_name,
            timestamp: Utc::now(),
            files_changed: vec![
                format!("impact/analyses/impact_analysis_{}.ron", impact_analysis.id),
                format!("impact/analyses/approval_workflow_{}.ron", approval_workflow.id),
            ],
        };

        if let Some(workflow) = self.active_workflows.values_mut()
            .find(|w| w.branch_name == branch_name) {
            
            workflow.commits.push(commit_info);
            workflow.status = GitWorkflowStatus::CommitsPushed;
            workflow.updated = Utc::now();
        }

        Ok(())
    }

    /// Create a pull request for the impact analysis
    async fn create_pull_request(
        &mut self,
        workflow_id: &Id,
        impact_analysis: &ImpactAnalysis,
        project_ctx: &ProjectContext
    ) -> Result<()> {
        // Generate PR info first (before mutable borrow)
        let pr_info = self.generate_pr_info(impact_analysis, project_ctx).await?;

        // TODO: Create actual GitHub PR using GitHub API
        // For now, we'll simulate PR creation
        let pr_url = format!("https://github.com/org/repo/pull/{}", workflow_id.to_string());
        
        // Now update the workflow
        let workflow = self.active_workflows.get_mut(workflow_id)
            .ok_or_else(|| tessera_core::DesignTrackError::Validation("Workflow not found".to_string()))?;
        
        workflow.pull_request_url = Some(pr_url.clone());
        workflow.status = GitWorkflowStatus::PullRequestCreated;
        workflow.updated = Utc::now();

        println!("Created pull request: {}", pr_url);

        Ok(())
    }

    /// Generate pull request information from impact analysis
    async fn generate_pr_info(
        &self,
        impact_analysis: &ImpactAnalysis,
        project_ctx: &ProjectContext
    ) -> Result<PullRequestInfo> {
        let template = self.config.get_pr_template(impact_analysis.max_severity);
        
        // Replace template variables
        let title = format!("Impact Analysis: {} {}", 
            impact_analysis.source_entity.module,
            impact_analysis.source_entity.name
        );

        let description = template
            .replace("{source_entity}", &format!("{} {}", 
                impact_analysis.source_entity.module, impact_analysis.source_entity.name))
            .replace("{change_description}", &impact_analysis.change_description)
            .replace("{timestamp}", &impact_analysis.analysis_timestamp.to_string())
            .replace("{max_severity}", &impact_analysis.max_severity.to_string())
            .replace("{total_affected}", &impact_analysis.total_affected_entities.to_string())
            .replace("{estimated_effort}", &impact_analysis.estimated_total_effort_hours.to_string())
            .replace("{required_approval_level}", 
                &impact_analysis.required_approval_level.map(|l| l.to_string()).unwrap_or("None".to_string()));

        // Get labels based on severity
        let labels = self.config.workflow_config.pr_templates.impact_labels
            .get(&impact_analysis.max_severity)
            .cloned()
            .unwrap_or_default();

        // Determine reviewers based on impact analysis
        let reviewers = self.determine_reviewers(impact_analysis, project_ctx).await?;

        Ok(PullRequestInfo {
            title,
            description,
            labels,
            reviewers,
            assignees: Vec::new(),
            milestone: None,
        })
    }

    /// Determine who should review the PR based on impact analysis
    async fn determine_reviewers(
        &self,
        impact_analysis: &ImpactAnalysis,
        project_ctx: &ProjectContext
    ) -> Result<Vec<String>> {
        let mut reviewers: Vec<String> = Vec::new();

        // Load team repository to get reviewer information
        let team_repo = TeamRepository::load_from_project(project_ctx)?;

        // Get affected modules
        let impacts_by_module = impact_analysis.impacts_by_module();
        let affected_modules: Vec<_> = impacts_by_module.keys().collect();

        // Find team members responsible for affected modules
        for &module in &affected_modules {
            if let Some(team_ids) = self.config.team_config.module_ownership.get(module) {
                for &team_id in team_ids {
                    // Get team members and their Git usernames
                    if let Some(team) = team_repo.get_team(team_id) {
                        for &member_id in &team.members {
                            if let Some(member) = team_repo.get_team_member(member_id) {
                                if let Some(git_username) = &member.git_username {
                                    if !reviewers.contains(git_username) {
                                        reviewers.push(git_username.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Ensure we have at least one reviewer for high/critical impact
        if reviewers.is_empty() && impact_analysis.max_severity >= crate::ImpactSeverity::High {
            // Add default reviewers or escalate
            reviewers.push("default-reviewer".to_string());
        }

        Ok(reviewers)
    }

    /// Update workflow status based on external events (e.g., PR approval)
    pub fn update_workflow_status(&mut self, workflow_id: &Id, new_status: GitWorkflowStatus) -> Result<()> {
        if let Some(workflow) = self.active_workflows.get_mut(workflow_id) {
            workflow.status = new_status;
            workflow.updated = Utc::now();
        }
        Ok(())
    }

    /// Merge approved workflow
    pub async fn merge_workflow(&mut self, workflow_id: &Id) -> Result<()> {
        // Extract workflow data first to avoid borrowing conflicts
        let (branch_name, should_merge) = {
            let workflow = self.active_workflows.get(workflow_id)
                .ok_or_else(|| tessera_core::DesignTrackError::Validation("Workflow not found".to_string()))?;

            if workflow.status != GitWorkflowStatus::Approved {
                return Err(tessera_core::DesignTrackError::Validation(
                    "Workflow not approved for merge".to_string()
                ));
            }

            (workflow.branch_name.clone(), true)
        };

        if !should_merge {
            return Ok(());
        }

        // Switch to main branch
        self.repository.set_head("refs/heads/main")?;

        // Merge the feature branch
        let branch_ref = format!("refs/heads/{}", branch_name);
        let branch_commit = self.repository.reference_to_annotated_commit(
            &self.repository.find_reference(&branch_ref)?
        )?;

        // Perform merge
        let mut merge_options = git2::MergeOptions::new();
        let mut checkout_options = git2::build::CheckoutBuilder::new();
        
        self.repository.merge(&[&branch_commit], Some(&mut merge_options), Some(&mut checkout_options))?;

        // Create merge commit
        let signature = self.get_git_signature()?;
        let mut index = self.repository.index()?;
        
        if index.has_conflicts() {
            return Err(tessera_core::DesignTrackError::Validation(
                "Merge conflicts detected".to_string()
            ));
        }

        let tree_id = index.write_tree()?;
        let tree = self.repository.find_tree(tree_id)?;
        let head_commit = self.repository.head()?.peel_to_commit()?;
        let branch_commit_obj = self.repository.find_commit(branch_commit.id())?;

        let merge_message = format!("Merge impact analysis workflow {}", workflow_id);
        
        self.repository.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &merge_message,
            &tree,
            &[&head_commit, &branch_commit_obj]
        )?;

        // Drop signature to release borrow
        drop(signature);

        // Clean up: delete the feature branch
        let mut branch = self.repository.find_branch(&branch_name, BranchType::Local)?;
        branch.delete()?;

        // Update workflow status
        if let Some(workflow) = self.active_workflows.get_mut(workflow_id) {
            workflow.status = GitWorkflowStatus::Merged;
            workflow.updated = Utc::now();
        }

        Ok(())
    }

    /// Cancel a workflow
    pub fn cancel_workflow(&mut self, workflow_id: &Id) -> Result<()> {
        if let Some(workflow) = self.active_workflows.get_mut(workflow_id) {
            // Delete the feature branch if it exists
            if let Ok(mut branch) = self.repository.find_branch(&workflow.branch_name, BranchType::Local) {
                let _ = branch.delete();
            }

            workflow.status = GitWorkflowStatus::Cancelled;
            workflow.updated = Utc::now();
        }
        Ok(())
    }

    /// Get Git signature for commits
    fn get_git_signature(&self) -> Result<Signature> {
        // Try to get signature from Git config
        let config = self.repository.config()?;
        let name = config.get_string("user.name").unwrap_or_else(|_| "Tessera Impact Analysis".to_string());
        let email = config.get_string("user.email").unwrap_or_else(|_| "tessera@example.com".to_string());
        
        Ok(Signature::now(&name, &email)?)
    }

    /// Determine if PR should be auto-created
    fn should_auto_create_pr(&self, impact_analysis: &ImpactAnalysis) -> bool {
        // Auto-create for medium severity and above
        impact_analysis.max_severity >= crate::ImpactSeverity::Medium
    }

    /// Get workflow by ID
    pub fn get_workflow(&self, workflow_id: &Id) -> Option<&GitWorkflow> {
        self.active_workflows.get(workflow_id)
    }

    /// Get all active workflows
    pub fn get_active_workflows(&self) -> Vec<&GitWorkflow> {
        self.active_workflows.values().collect()
    }

    /// Save workflow state to project
    pub fn save_workflows(&self, project_ctx: &ProjectContext) -> Result<()> {
        let workflow_dir = project_ctx.root_path.join("impact").join("workflows");
        std::fs::create_dir_all(&workflow_dir)?;

        let workflows_content = ron::ser::to_string_pretty(&self.active_workflows, ron::ser::PrettyConfig::default())?;
        std::fs::write(workflow_dir.join("git_workflows.ron"), workflows_content)?;

        Ok(())
    }

    /// Load workflow state from project
    pub fn load_workflows(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        let workflows_file = project_ctx.root_path.join("impact").join("workflows").join("git_workflows.ron");
        
        if workflows_file.exists() {
            let content = std::fs::read_to_string(workflows_file)?;
            self.active_workflows = ron::from_str(&content)?;
        }

        Ok(())
    }
}