use crate::scheduling::Schedule;
use anyhow::Result;
use comfy_table::{Table, presets};
use std::fmt::Write;

pub struct ResourceReporter;

impl ResourceReporter {
    pub fn generate_table(schedule: &Schedule) -> Result<String> {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL)
            .set_header(vec!["Resource", "Total Hours", "Utilization %", "Tasks"]);

        for (_, utilization) in &schedule.resource_utilization {
            table.add_row(vec![
                utilization.name.clone(),
                format!("{:.1}", utilization.total_hours),
                format!("{:.1}%", utilization.utilization_percentage),
                utilization.task_assignments.len().to_string(),
            ]);
        }

        Ok(table.to_string())
    }

    pub fn generate_markdown(schedule: &Schedule) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "# Resource Utilization Report: {}", schedule.project_name)?;
        writeln!(&mut output)?;

        for (_resource_id, utilization) in &schedule.resource_utilization {
            writeln!(&mut output, "## {}", utilization.name)?;
            writeln!(&mut output)?;
            writeln!(&mut output, "- **Total Hours**: {:.1}", utilization.total_hours)?;
            writeln!(&mut output, "- **Utilization**: {:.1}%", utilization.utilization_percentage)?;
            writeln!(&mut output)?;

            if !utilization.task_assignments.is_empty() {
                writeln!(&mut output, "### Assigned Tasks")?;
                writeln!(&mut output)?;
                writeln!(&mut output, "| Task | Start | End | Hours |")?;
                writeln!(&mut output, "|------|-------|-----|-------|")?;

                for assignment in &utilization.task_assignments {
                    writeln!(
                        &mut output,
                        "| {} | {} | {} | {:.1} |",
                        assignment.task_name,
                        assignment.start_date.format("%Y-%m-%d"),
                        assignment.end_date.format("%Y-%m-%d"),
                        assignment.hours
                    )?;
                }
                writeln!(&mut output)?;
            }
        }

        Ok(output)
    }
}
