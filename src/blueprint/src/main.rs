use anyhow::Result;
use clap::{Parser, Subcommand};
use blueprint::cli::{self, interactive};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blueprint")]
#[command(version, about = "Text-based project management with modern tooling", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Project name
        name: String,
        /// Use a template
        #[arg(short, long)]
        template: Option<String>,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compute and display project schedule
    Schedule {
        /// Project file
        #[arg(default_value = "project.ron")]
        project: PathBuf,
        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate specific reports
    Report {
        /// Report type
        #[command(subcommand)]
        report_type: ReportType,
        /// Project file
        #[arg(default_value = "project.ron")]
        project: PathBuf,
    },
    /// Perform what-if analysis
    Analyze {
        /// Project file
        #[arg(default_value = "project.ron")]
        project: PathBuf,
        /// Scenario name
        #[arg(short, long)]
        scenario: Option<String>,
    },
    /// Interactive mode
    Interactive {
        /// Project file
        #[arg(default_value = "project.ron")]
        project: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ReportType {
    /// Generate Gantt chart
    Gantt {
        /// Output format: mermaid/resource (grouped by resources), wbs/timeline (project timeline), utilization/heatmap (resource workload), markdown, png, svg, pdf
        #[arg(short, long, default_value = "mermaid")]
        format: String,
        /// Output file path (for image formats)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate cost report
    Costs {
        /// Output format
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
    /// Generate resource utilization report
    Resources {
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // If no command specified, run interactive mode
            interactive::run(None)?;
        }
        Some(Commands::Init { name, template, output }) => {
            cli::commands::init_project(&name, template.as_deref(), output)?;
        }
        Some(Commands::Schedule { project, output }) => {
            cli::commands::compute_schedule(&project, output)?;
        }
        Some(Commands::Report { report_type, project }) => {
            match report_type {
                ReportType::Gantt { format, output } => {
                    cli::commands::generate_gantt(&project, &format, output.as_deref())?;
                }
                ReportType::Costs { format } => {
                    cli::commands::generate_cost_report(&project, &format)?;
                }
                ReportType::Resources { format } => {
                    cli::commands::generate_resource_report(&project, &format)?;
                }
            }
        }
        Some(Commands::Analyze { project, scenario }) => {
            cli::commands::analyze_scenario(&project, scenario.as_deref())?;
        }
        Some(Commands::Interactive { project }) => {
            interactive::run(project)?;
        }
    }

    Ok(())
}
