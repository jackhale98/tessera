use clap::{Parser, Subcommand};
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use std::path::PathBuf;

mod commands;
mod interactive;
mod utils;
mod impact_service;

use commands::{
    execute_requirements_command, execute_risk_command, execute_verification_command,
    execute_quality_command, execute_pm_command, execute_tol_command, execute_team_command, 
    execute_impact_command, execute_link_command
};
use interactive::*;

#[derive(Parser)]
#[command(name = "tessera")]
#[command(about = "A comprehensive CLI-based engineering toolkit")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[arg(short, long, global = true)]
    project: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        template: Option<String>,
    },
    
    /// Requirements management commands
    Requirements {
        #[command(subcommand)]
        command: RequirementsCommands,
    },
    
    /// Risk management commands
    Risk {
        #[command(subcommand)]
        command: RiskCommands,
    },
    
    /// Verification and testing commands
    Verification {
        #[command(subcommand)]
        command: VerificationCommands,
    },
    
    /// Legacy quality management commands (deprecated)
    Quality {
        #[command(subcommand)]
        command: QualityCommands,
    },
    
    /// Project management commands
    Pm {
        #[command(subcommand)]
        command: PmCommands,
    },
    
    /// Tolerance analysis commands
    Tol {
        #[command(subcommand)]
        command: TolCommands,
    },
    
    /// Team management commands
    Team {
        #[command(subcommand)]
        command: TeamCommands,
    },
    
    /// Impact analysis and lifecycle management
    Impact {
        #[command(subcommand)]
        command: ImpactCommands,
    },
    
    /// Interactive mode
    Interactive {
        #[arg(short, long)]
        module: Option<String>,
    },
    
    /// Project status and information
    Status,
    
    /// Validate project files and links
    Validate,
    
    /// Manage cross-module links
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },
}

#[derive(Subcommand, Clone)]
enum RequirementsCommands {
    /// Add a new requirement
    #[command(name = "req:add")]
    AddRequirement,
    
    /// List requirements
    #[command(name = "req:list")]
    ListRequirements,
    
    /// Edit a requirement
    #[command(name = "req:edit")]
    EditRequirement,
    
    /// Add a design input
    #[command(name = "input:add")]
    AddInput,
    
    /// List design inputs
    #[command(name = "input:list")]
    ListInputs,
    
    /// Edit a design input
    #[command(name = "input:edit")]
    EditInput,
    
    /// Add a design output
    #[command(name = "output:add")]
    AddOutput,
    
    /// List design outputs
    #[command(name = "output:list")]
    ListOutputs,
    
    /// Edit a design output
    #[command(name = "output:edit")]
    EditOutput,
    
    /// Add a verification
    #[command(name = "verification:add")]
    AddVerification,
    
    /// List verifications
    #[command(name = "verification:list")]
    ListVerifications,
    
    /// Edit a verification
    #[command(name = "verification:edit")]
    EditVerification,
    
    /// Show requirements dashboard
    #[command(name = "dashboard")]
    Dashboard,
    
    /// Show traceability matrix
    #[command(name = "trace:matrix")]
    TraceabilityMatrix,
}

#[derive(Subcommand, Clone)]
enum RiskCommands {
    /// Add a new risk
    #[command(name = "risk:add")]
    AddRisk,
    
    /// List risks
    #[command(name = "risk:list")]
    ListRisks,
    
    /// Edit a risk
    #[command(name = "risk:edit")]
    EditRisk,
    
    /// Assess risks
    #[command(name = "risk:assess")]
    AssessRisks,
    
    /// Add a design control
    #[command(name = "control:add")]
    AddControl,
    
    /// List design controls
    #[command(name = "control:list")]
    ListControls,
    
    /// Edit a design control
    #[command(name = "control:edit")]
    EditControl,
    
    /// Show risk dashboard
    #[command(name = "dashboard")]
    Dashboard,
    
    /// Risk scoring analysis
    #[command(name = "scoring")]
    RiskScoring,
}

#[derive(Subcommand, Clone)]
enum VerificationCommands {
    /// Add a test procedure
    #[command(name = "procedure:add")]
    AddProcedure,
    
    /// List test procedures
    #[command(name = "procedure:list")]
    ListProcedures,
    
    /// Edit a test procedure
    #[command(name = "procedure:edit")]
    EditProcedure,
    
    /// Add a test execution
    #[command(name = "execution:add")]
    AddExecution,
    
    /// List test executions
    #[command(name = "execution:list")]
    ListExecutions,
    
    /// Edit a test execution
    #[command(name = "execution:edit")]
    EditExecution,
    
    /// Show verification dashboard
    #[command(name = "dashboard")]
    Dashboard,
    
    /// Generate test report
    #[command(name = "report")]
    GenerateReport,
}

#[derive(Subcommand, Clone)]
enum QualityCommands {
    /// Legacy quality dashboard
    Dashboard,
}

#[derive(Subcommand, Clone)]
enum PmCommands {
    /// Add a new task
    #[command(name = "task:add")]
    AddTask,
    
    /// List tasks
    #[command(name = "task:list")]
    ListTasks,
    
    /// Edit a task
    #[command(name = "task:edit")]
    EditTask,
    
    /// Delete a task
    #[command(name = "task:delete")]
    DeleteTask,
    
    /// Add a resource
    #[command(name = "resource:add")]
    AddResource,
    
    /// List resources
    #[command(name = "resource:list")]
    ListResources,
    
    /// Edit a resource
    #[command(name = "resource:edit")]
    EditResource,
    
    /// Delete a resource
    #[command(name = "resource:delete")]
    DeleteResource,
    
    /// Add a milestone
    #[command(name = "milestone:add")]
    AddMilestone,
    
    /// List milestones
    #[command(name = "milestone:list")]
    ListMilestones,
    
    /// Edit a milestone
    #[command(name = "milestone:edit")]
    EditMilestone,
    
    /// Delete a milestone
    #[command(name = "milestone:delete")]
    DeleteMilestone,
    
    /// Compute project schedule
    Schedule,
    
    /// Show cost analysis
    #[command(name = "cost:analysis")]
    CostAnalysis,
    
    /// Project management dashboard
    Dashboard,
    
    /// Add a project risk
    #[command(name = "risk:add")]
    AddRisk,
    
    /// List project risks
    #[command(name = "risk:list")]
    ListRisks,
    
    /// Edit a project risk
    #[command(name = "risk:edit")]
    EditRisk,
    
    /// Add an issue
    #[command(name = "issue:add")]
    AddIssue,
    
    /// List issues
    #[command(name = "issue:list")]
    ListIssues,
    
    /// Edit an issue
    #[command(name = "issue:edit")]
    EditIssue,
    
    /// Create a baseline
    #[command(name = "baseline:create")]
    CreateBaseline,
    
    /// List baselines
    #[command(name = "baseline:list")]
    ListBaselines,
    
    /// Compare baselines
    #[command(name = "baseline:compare")]
    CompareBaselines,
    
    /// Add a calendar
    #[command(name = "calendar:add")]
    AddCalendar,
    
    /// List calendars
    #[command(name = "calendar:list")]
    ListCalendars,
    
    /// Edit a calendar
    #[command(name = "calendar:edit")]
    EditCalendar,
    
    /// Assign calendar to resource
    #[command(name = "calendar:assign")]
    AssignCalendar,
    
    /// List calendar assignments
    #[command(name = "calendar:assignments")]
    ListCalendarAssignments,
    
    /// Remove calendar assignment
    #[command(name = "calendar:unassign")]
    UnassignCalendar,
    
    /// Check milestone status
    #[command(name = "milestone:status")]
    CheckMilestoneStatus,
    
    /// Show milestone alerts
    #[command(name = "milestone:alerts")]
    ShowMilestoneAlerts,

    /// Analyze critical path for a task
    #[command(name = "analysis:task-path")]
    AnalyzeTaskCriticalPath,

    /// Analyze critical path for a milestone
    #[command(name = "analysis:milestone-path")]
    AnalyzeMilestoneCriticalPath,
}

#[derive(Subcommand, Clone)]
enum TolCommands {
    /// Add a new component
    #[command(name = "component:add")]
    AddComponent,
    
    /// Edit a component
    #[command(name = "component:edit")]
    EditComponent,
    
    /// List components
    #[command(name = "component:list")]
    ListComponents,
    
    /// Add a feature
    #[command(name = "feature:add")]
    AddFeature,
    
    /// Edit a feature
    #[command(name = "feature:edit")]
    EditFeature,
    
    /// List features
    #[command(name = "feature:list")]
    ListFeatures,
    
    /// Add a mate
    #[command(name = "mate:add")]
    AddMate,
    
    /// Edit a mate
    #[command(name = "mate:edit")]
    EditMate,
    
    /// List mates
    #[command(name = "mate:list")]
    ListMates,
    
    /// Add a stackup
    #[command(name = "stackup:add")]
    AddStackup,
    
    /// Edit a stackup
    #[command(name = "stackup:edit")]
    EditStackup,
    
    /// Delete a stackup
    #[command(name = "stackup:delete")]
    DeleteStackup,
    
    /// Run tolerance analysis
    #[command(name = "analysis:run")]
    RunAnalysis,
    
    /// List previous analysis results
    #[command(name = "analysis:list")]
    ListAnalysis,
    
    /// Delete analysis results
    #[command(name = "analysis:delete")]
    DeleteAnalysis,
    
    /// Tolerance analysis dashboard
    Dashboard,
}

#[derive(Subcommand, Clone)]
enum TeamCommands {
    /// Add a new team member
    #[command(name = "member:add")]
    AddMember,
    
    /// List team members
    #[command(name = "member:list")]
    ListMembers,
    
    /// Edit a team member
    #[command(name = "member:edit")]
    EditMember,
    
    /// Deactivate a team member
    #[command(name = "member:deactivate")]
    DeactivateMember,
    
    /// Add a new role
    #[command(name = "role:add")]
    AddRole,
    
    /// List roles
    #[command(name = "role:list")]
    ListRoles,
    
    /// Edit a role
    #[command(name = "role:edit")]
    EditRole,
    
    /// Assign role to member
    #[command(name = "role:assign")]
    AssignRole,
    
    /// Remove role from member
    #[command(name = "role:remove")]
    RemoveRole,
    
    /// Add a new team
    #[command(name = "team:add")]
    AddTeam,
    
    /// List teams
    #[command(name = "team:list")]
    ListTeams,
    
    /// Edit a team
    #[command(name = "team:edit")]
    EditTeam,
    
    /// Add member to team
    #[command(name = "team:add-member")]
    AddMemberToTeam,
    
    /// Remove member from team
    #[command(name = "team:remove-member")]
    RemoveMemberFromTeam,
    
    /// Set team lead
    #[command(name = "team:set-lead")]
    SetTeamLead,
    
    /// Validate team configuration
    #[command(name = "validate")]
    Validate,
    
    /// Sync with git teams
    #[command(name = "git:sync")]
    GitSync,
    
    /// Generate CODEOWNERS file
    #[command(name = "git:codeowners")]
    GenerateCodeowners,
    
    /// Show team dashboard
    Dashboard,
}

#[derive(Subcommand, Clone)]
enum ImpactCommands {
    /// List automatically generated impact analyses
    #[command(name = "list")]
    ListAnalyses,
    
    /// Show details of a specific impact analysis
    #[command(name = "show")]
    ShowAnalysis {
        /// Analysis ID
        analysis_id: String,
    },
    
    /// Show recent change events
    #[command(name = "events")]
    ShowEvents,
    
    /// Show impact analysis for a specific entity
    #[command(name = "entity")]
    ShowEntityImpacts {
        /// Entity ID
        entity_id: String,
    },
    
    /// List pending approvals requiring action
    #[command(name = "approvals")]
    ListApprovals,
    
    /// Process an approval workflow
    #[command(name = "approve")]
    ProcessApproval {
        /// Workflow ID
        workflow_id: String,
        /// Decision (approved, rejected, changes)
        #[arg(short, long)]
        decision: String,
        /// Optional comments
        #[arg(short, long)]
        comments: Option<String>,
    },
    
    /// List active Git workflows
    #[command(name = "workflows")]
    ListWorkflows,
    
    /// Show workflow details
    #[command(name = "workflow")]
    ShowWorkflow {
        /// Workflow ID
        workflow_id: String,
    },
    
    /// Configure automatic impact analysis settings
    #[command(name = "config")]
    Configure,
    
    /// Show impact analysis dashboard and statistics
    Dashboard,
}

#[derive(Subcommand)]
enum LinkCommands {
    /// Add a cross-module link
    Add,
    
    /// List all links
    List,
    
    /// Show links for a specific entity
    Show,
    
    /// Remove a link
    Remove,
    
    /// Validate all links
    Validate,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Init { name, description, template }) => {
            let project_ctx = init_project(name, description, template)?;
            println!("{}", "Project initialized successfully!".green());
            println!("Project location: {}", project_ctx.root_path.display());
        },
        
        Some(Commands::Requirements { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_requirements_command(command, project_ctx).await?;
        },
        
        Some(Commands::Risk { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_risk_command(command, project_ctx).await?;
        },
        
        Some(Commands::Verification { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_verification_command(command, project_ctx).await?;
        },
        
        Some(Commands::Quality { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_quality_command(command, project_ctx).await?;
        },
        
        Some(Commands::Pm { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_pm_command(command, project_ctx).await?;
        },
        
        Some(Commands::Tol { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_tol_command(command, project_ctx).await?;
        },
        
        Some(Commands::Team { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_team_command(command, project_ctx).await?;
        },
        
        Some(Commands::Impact { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_impact_command(command, project_ctx).await?;
        },
        
        Some(Commands::Interactive { module }) => {
            let project_ctx = load_project_context(cli.project)?;
            run_interactive_mode(project_ctx, module).await?;
        },
        
        Some(Commands::Status) => {
            let project_ctx = load_project_context(cli.project)?;
            show_project_status(project_ctx)?;
        },
        
        Some(Commands::Validate) => {
            let project_ctx = load_project_context(cli.project)?;
            validate_project(project_ctx)?;
        },
        
        Some(Commands::Link { command }) => {
            let project_ctx = load_project_context(cli.project)?;
            execute_link_command(command, project_ctx).await?;
        },
        
        None => {
            let project_ctx = load_project_context(cli.project)?;
            run_interactive_mode(project_ctx, None).await?;
        },
    }
    
    Ok(())
}

fn load_project_context(project_path: Option<PathBuf>) -> Result<ProjectContext> {
    let workspace_path = match project_path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };
    
    ProjectContext::load_from_workspace(workspace_path)
}

fn init_project(name: String, description: Option<String>, _template: Option<String>) -> Result<ProjectContext> {
    let current_dir = std::env::current_dir()?;
    let description = description.unwrap_or_else(|| format!("Engineering project: {}", name));
    
    let metadata = tessera_core::ProjectMetadata::new(name, description);
    let project_ctx = ProjectContext::new(metadata, current_dir.clone());
    
    project_ctx.ensure_module_dirs()?;
    project_ctx.metadata.save_to_file(current_dir.join("project.ron"))?;
    
    Ok(project_ctx)
}

fn show_project_status(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Project Status".bold().blue());
    println!("Name: {}", project_ctx.metadata.name);
    println!("Description: {}", project_ctx.metadata.description);
    println!("Version: {}", project_ctx.metadata.version);
    println!("Created: {}", project_ctx.metadata.created.format("%Y-%m-%d %H:%M:%S"));
    println!("Modules: {}", project_ctx.metadata.modules.join(", "));
    
    Ok(())
}

fn validate_project(project_ctx: ProjectContext) -> Result<()> {
    println!("{}", "Validating project...".blue());
    
    for module in &project_ctx.metadata.modules {
        let module_path = project_ctx.module_path(module);
        if !module_path.exists() {
            println!("  {} Module directory missing: {}", "⚠".yellow(), module);
        } else {
            println!("  {} Module directory exists: {}", "✓".green(), module);
        }
    }
    
    println!("{}", "Validation complete!".green());
    Ok(())
}