use clap::{Parser, Subcommand};
use colored::Colorize;
use tessera_core::{ProjectContext, Result};
use std::path::PathBuf;

mod commands;
mod interactive;
mod utils;

use commands::*;
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
    
    /// Quality management commands
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
enum QualityCommands {
    /// Add a new requirement
    #[command(name = "req:add")]
    AddRequirement,
    
    /// List requirements
    #[command(name = "req:list")]
    ListRequirements,
    
    /// Add a design input
    #[command(name = "input:add")]
    AddInput,
    
    /// List design inputs
    #[command(name = "input:list")]
    ListInputs,
    
    /// Link input to requirement
    #[command(name = "input:link-req")]
    LinkInputToRequirement,
    
    /// Add a design output
    #[command(name = "output:add")]
    AddOutput,
    
    /// List design outputs
    #[command(name = "output:list")]
    ListOutputs,
    
    
    /// Link output to input
    #[command(name = "output:link-input")]
    LinkOutputToInput,
    
    /// Add a verification
    #[command(name = "verification:add")]
    AddVerification,
    
    /// List verifications
    #[command(name = "verification:list")]
    ListVerifications,
    
    /// Link verification to output
    #[command(name = "verification:link-output")]
    LinkVerificationToOutput,
    
    /// Add a risk
    #[command(name = "risk:add")]
    AddRisk,
    
    /// List risks
    #[command(name = "risk:list")]
    ListRisks,
    
    /// Assess project risks
    #[command(name = "risk:assess")]
    AssessRisks,
    
    /// Traceability matrix
    #[command(name = "trace:matrix")]
    TraceabilityMatrix,
    
    /// Risk scoring tools
    #[command(name = "risk:score")]
    RiskScoring,
    
    /// Quality dashboard
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
    
    /// Add a resource
    #[command(name = "resource:add")]
    AddResource,
    
    /// Add a milestone
    #[command(name = "milestone:add")]
    AddMilestone,
    
    /// Compute project schedule
    Schedule,
    
    /// Project management dashboard
    Dashboard,
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
    
    /// Run tolerance analysis
    #[command(name = "analysis:run")]
    RunAnalysis,
    
    /// Configure analysis settings
    #[command(name = "analysis:config")]
    ConfigureAnalysis,
    
    /// Tolerance analysis dashboard
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