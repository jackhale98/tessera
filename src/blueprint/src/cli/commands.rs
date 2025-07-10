use crate::{Project, SchedulingEngine};
use crate::reporting::{GanttGenerator, CostReporter, ResourceReporter};
use crate::templates;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub fn init_project(name: &str, template: Option<&str>, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));
    let project_file = output_dir.join("project.ron");

    if project_file.exists() {
        anyhow::bail!("Project file already exists: {}", project_file.display());
    }

    let content = if let Some(template_name) = template {
        templates::get_template(template_name)?
    } else {
        templates::create_basic_project(name)
    };

    fs::write(&project_file, content)?;

    // Initialize git repository if not already in one
    if !output_dir.join(".git").exists() {
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&output_dir)
            .output()?;

        // Add project file to git
        std::process::Command::new("git")
            .arg("add")
            .arg("project.ron")
            .current_dir(&output_dir)
            .output()?;
    }

    super::print_success(&format!("Created project '{}' at {}", name, project_file.display()));
    super::print_info("Edit project.ron to define your tasks and resources");

    Ok(())
}

pub fn compute_schedule(project_path: &Path, output_dir: Option<PathBuf>) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message("Loading project...");

    let project = Project::load_from_file(project_path)?;

    pb.set_message("Computing schedule...");
    let engine = SchedulingEngine::new();
    let schedule = engine.compute_schedule(&project)?;

    pb.finish_with_message("Schedule computed!");

    // Display summary
    println!("\n{}", "Schedule Summary".bold());
    println!("{}", "─".repeat(50));
    println!("Project: {}", schedule.project_name.cyan());
    println!("Duration: {} → {} ({} days)",
        schedule.start_date.format("%Y-%m-%d"),
        schedule.end_date.format("%Y-%m-%d"),
        (schedule.end_date - schedule.start_date).num_days()
    );
    println!("Total Cost: {} {:.2}", project.currency, schedule.total_cost);
    println!("Tasks: {}", schedule.tasks.len());
    println!();

    // Save reports if output directory specified
    if let Some(dir) = output_dir {
        fs::create_dir_all(&dir)?;

        // Save schedule summary
        let summary_path = dir.join("schedule.json");
        let summary = serde_json::to_string_pretty(&schedule)?;
        fs::write(&summary_path, summary)?;

        // Generate reports
        let gantt_path = dir.join("gantt.md");
        let gantt = GanttGenerator::generate_markdown(&schedule)?;
        fs::write(&gantt_path, gantt)?;

        let cost_path = dir.join("costs.md");
        let costs = CostReporter::generate_markdown(&schedule, &project.currency)?;
        fs::write(&cost_path, costs)?;

        let resource_path = dir.join("resources.md");
        let resources = ResourceReporter::generate_markdown(&schedule)?;
        fs::write(&resource_path, resources)?;

        super::print_success(&format!("Reports saved to {}", dir.display()));
    }

    Ok(())
}

pub fn generate_gantt(project_path: &Path, format: &str, output_path: Option<&str>) -> Result<()> {
    let project = Project::load_from_file(project_path)?;
    let engine = SchedulingEngine::new();
    let schedule = engine.compute_schedule(&project)?;

    match format {
        "mermaid" | "resource" => {
            let gantt = GanttGenerator::generate_mermaid(&schedule)?;
            if let Some(path) = output_path {
                std::fs::write(path, gantt)?;
                super::print_success(&format!("Resource-grouped Gantt chart saved to {}", path));
            } else {
                println!("{}", gantt);
            }
        }
        "wbs" | "timeline" => {
            let gantt = GanttGenerator::generate_wbs_mermaid(&schedule)?;
            if let Some(path) = output_path {
                std::fs::write(path, gantt)?;
                super::print_success(&format!("WBS timeline Gantt chart saved to {}", path));
            } else {
                println!("{}", gantt);
            }
        }
        "utilization" | "heatmap" => {
            let gantt = GanttGenerator::generate_utilization_mermaid(&schedule)?;
            if let Some(path) = output_path {
                std::fs::write(path, gantt)?;
                super::print_success(&format!("Resource utilization chart saved to {}", path));
            } else {
                println!("{}", gantt);
            }
        }
        "markdown" | "md" => {
            let gantt = GanttGenerator::generate_markdown(&schedule)?;
            if let Some(path) = output_path {
                std::fs::write(path, gantt)?;
                super::print_success(&format!("Gantt chart saved to {}", path));
            } else {
                println!("{}", gantt);
            }
        }
        "png" | "svg" | "pdf" => {
            let default_filename = format!("gantt.{}", format);
            let output_file = output_path.unwrap_or(&default_filename);
            GanttGenerator::generate_image(&schedule, output_file, format)?;
        }
        _ => anyhow::bail!("Unsupported format: {}. Supported: mermaid/resource, wbs/timeline, utilization/heatmap, markdown, png, svg, pdf", format),
    }

    Ok(())
}

pub fn generate_cost_report(project_path: &Path, format: &str) -> Result<()> {
    let project = Project::load_from_file(project_path)?;
    let engine = SchedulingEngine::new();
    let schedule = engine.compute_schedule(&project)?;

    match format {
        "markdown" | "md" => {
            let report = CostReporter::generate_markdown(&schedule, &project.currency)?;
            println!("{}", report);
        }
        "csv" => {
            let report = CostReporter::generate_csv(&schedule)?;
            println!("{}", report);
        }
        _ => anyhow::bail!("Unsupported format: {}", format),
    }

    Ok(())
}

pub fn generate_resource_report(project_path: &Path, format: &str) -> Result<()> {
    let project = Project::load_from_file(project_path)?;
    let engine = SchedulingEngine::new();
    let schedule = engine.compute_schedule(&project)?;

    match format {
        "table" => {
            let report = ResourceReporter::generate_table(&schedule)?;
            println!("{}", report);
        }
        "markdown" | "md" => {
            let report = ResourceReporter::generate_markdown(&schedule)?;
            println!("{}", report);
        }
        _ => anyhow::bail!("Unsupported format: {}", format),
    }

    Ok(())
}

pub fn analyze_scenario(_project_path: &Path, scenario: Option<&str>) -> Result<()> {
    let scenario_name = scenario.unwrap_or("analysis");

    // Create a git branch for the scenario
    std::process::Command::new("git")
        .args(&["checkout", "-b", &format!("scenario/{}", scenario_name)])
        .output()?;

    super::print_success(&format!("Created scenario branch 'scenario/{}'", scenario_name));
    super::print_info("Modify project.ron to test your scenario");
    super::print_info(&format!("Run 'git checkout main' to return to the main timeline"));

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "project")]
pub struct ProjectFile {
    #[serde(flatten)]
    pub project: Project,
}
