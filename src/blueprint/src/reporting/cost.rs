use crate::scheduling::Schedule;
use anyhow::Result;
use std::fmt::Write;

pub struct CostReporter;

impl CostReporter {
    pub fn generate_markdown(schedule: &Schedule, currency: &str) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "# Cost Report: {}", schedule.project_name)?;
        writeln!(&mut output)?;
        writeln!(&mut output, "## Summary")?;
        writeln!(&mut output, "- **Total Cost**: {} {:.2}", currency, schedule.total_cost)?;
        writeln!(&mut output, "- **Average Daily Cost**: {} {:.2}",
            currency,
            schedule.total_cost / (schedule.end_date - schedule.start_date).num_days() as f32
        )?;
        writeln!(&mut output)?;

        writeln!(&mut output, "## Cost by Resource")?;
        writeln!(&mut output)?;
        writeln!(&mut output, "| Resource | Hours | Rate | Cost |")?;
        writeln!(&mut output, "|----------|-------|------|------|")?;

        let mut resource_costs: std::collections::HashMap<String, (f32, f32, f32)> = std::collections::HashMap::new();

        for task in schedule.tasks.values() {
            let entry = resource_costs.entry(task.assigned_to.clone()).or_insert((0.0, 0.0, 0.0));
            entry.0 += task.effort;
            entry.2 += task.cost;
            if entry.1 == 0.0 && task.effort > 0.0 {
                entry.1 = task.cost / task.effort; // Calculate hourly rate
            }
        }

        for (resource_id, (hours, rate, cost)) in &resource_costs {
            writeln!(
                &mut output,
                "| {} | {:.1} | {} {:.2} | {} {:.2} |",
                resource_id, hours, currency, rate, currency, cost
            )?;
        }

        writeln!(&mut output)?;
        writeln!(&mut output, "## Cost by Task")?;
        writeln!(&mut output)?;
        writeln!(&mut output, "| Task | Hours | Cost | % of Total |")?;
        writeln!(&mut output, "|------|-------|------|------------|")?;

        let mut tasks: Vec<_> = schedule.tasks.values().collect();
        tasks.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap());

        for task in tasks {
            let percentage = (task.cost / schedule.total_cost) * 100.0;
            writeln!(
                &mut output,
                "| {} | {:.1} | {} {:.2} | {:.1}% |",
                task.name, task.effort, currency, task.cost, percentage
            )?;
        }

        Ok(output)
    }

    pub fn generate_csv(schedule: &Schedule) -> Result<String> {
        let mut output = String::new();

        writeln!(&mut output, "Task,Resource,Hours,Cost")?;

        for task in schedule.tasks.values() {
            writeln!(
                &mut output,
                "{},{},{:.1},{:.2}",
                task.name, task.assigned_to, task.effort, task.cost
            )?;
        }

        Ok(output)
    }
}
