use crate::{Task, Resource, Project, Calendar};
use tessera_core::{Id, Result, DesignTrackError};
use chrono::{DateTime, Utc, NaiveDate, Weekday, Duration};
use std::collections::HashMap;

/// Advanced duration management and calculation utilities
pub struct DurationManager;

#[derive(Debug, Clone)]
pub struct DurationCalculation {
    pub estimated_duration_days: f32,
    pub estimated_duration_hours: f32,
    pub working_days: f32,
    pub calendar_days: i64,
    pub resource_utilization: f32,
    pub critical_path_impact: bool,
    pub calculation_method: DurationMethod,
}

#[derive(Debug, Clone)]
pub enum DurationMethod {
    EffortDriven,      // Duration = Effort / Resource capacity
    FixedDuration,     // Duration is fixed, adjust resource allocation
    FixedWork,         // Work is fixed, adjust duration based on resources
    ResourceConstrained, // Account for resource availability
}

#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub resource_id: Id,
    pub allocation_percentage: f32,
    pub daily_hours: f32,
    pub available_from: Option<DateTime<Utc>>,
    pub available_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SchedulingConstraints {
    pub earliest_start: Option<DateTime<Utc>>,
    pub latest_finish: Option<DateTime<Utc>>,
    pub fixed_start: Option<DateTime<Utc>>,
    pub fixed_finish: Option<DateTime<Utc>>,
    pub must_start_on: Option<NaiveDate>,
    pub must_finish_on: Option<NaiveDate>,
}

impl DurationManager {
    /// Calculate task duration using effort-driven approach
    pub fn calculate_effort_driven_duration(
        task: &Task,
        resources: &[Resource],
        calendar: &Calendar,
    ) -> Result<DurationCalculation> {
        if task.estimated_hours == 0.0 {
            return Ok(DurationCalculation {
                estimated_duration_days: 1.0,
                estimated_duration_hours: 8.0,
                working_days: 1.0,
                calendar_days: 1,
                resource_utilization: 0.0,
                critical_path_impact: false,
                calculation_method: DurationMethod::EffortDriven,
            });
        }

        let assigned_resources: Vec<&Resource> = resources.iter()
            .filter(|r| task.assigned_resources.contains(&r.id))
            .collect();

        if assigned_resources.is_empty() {
            // No resources assigned - use default 8 hours/day
            let duration_days = task.estimated_hours / 8.0;
            return Ok(DurationCalculation {
                estimated_duration_days: duration_days,
                estimated_duration_hours: task.estimated_hours,
                working_days: duration_days,
                calendar_days: Self::calculate_calendar_days(duration_days, calendar)?,
                resource_utilization: 0.0,
                critical_path_impact: false,
                calculation_method: DurationMethod::EffortDriven,
            });
        }

        // Calculate total daily capacity
        let total_daily_hours: f32 = assigned_resources.iter()
            .map(|r| r.daily_hours)
            .sum();

        let duration_days = if total_daily_hours > 0.0 {
            task.estimated_hours / total_daily_hours
        } else {
            task.estimated_hours / 8.0 // Fallback
        };

        let utilization = if assigned_resources.len() > 0 {
            total_daily_hours / (assigned_resources.len() as f32 * 8.0) * 100.0
        } else {
            0.0
        };

        Ok(DurationCalculation {
            estimated_duration_days: duration_days,
            estimated_duration_hours: task.estimated_hours,
            working_days: duration_days,
            calendar_days: Self::calculate_calendar_days(duration_days, calendar)?,
            resource_utilization: utilization,
            critical_path_impact: false,
            calculation_method: DurationMethod::EffortDriven,
        })
    }

    /// Calculate duration for fixed duration tasks
    pub fn calculate_fixed_duration(
        task: &Task,
        fixed_duration_days: f32,
        resources: &[Resource],
        calendar: &Calendar,
    ) -> Result<DurationCalculation> {
        let assigned_resources: Vec<&Resource> = resources.iter()
            .filter(|r| task.assigned_resources.contains(&r.id))
            .collect();

        let total_daily_hours: f32 = assigned_resources.iter()
            .map(|r| r.daily_hours)
            .sum();

        let required_effort = total_daily_hours * fixed_duration_days;
        
        let utilization = if !assigned_resources.is_empty() {
            total_daily_hours / (assigned_resources.len() as f32 * 8.0) * 100.0
        } else {
            0.0
        };

        Ok(DurationCalculation {
            estimated_duration_days: fixed_duration_days,
            estimated_duration_hours: required_effort,
            working_days: fixed_duration_days,
            calendar_days: Self::calculate_calendar_days(fixed_duration_days, calendar)?,
            resource_utilization: utilization,
            critical_path_impact: false,
            calculation_method: DurationMethod::FixedDuration,
        })
    }

    /// Calculate duration for fixed work tasks
    pub fn calculate_fixed_work_duration(
        fixed_work_hours: f32,
        resource_allocations: &[ResourceAllocation],
        calendar: &Calendar,
    ) -> Result<DurationCalculation> {
        if resource_allocations.is_empty() {
            let duration_days = fixed_work_hours / 8.0;
            return Ok(DurationCalculation {
                estimated_duration_days: duration_days,
                estimated_duration_hours: fixed_work_hours,
                working_days: duration_days,
                calendar_days: Self::calculate_calendar_days(duration_days, calendar)?,
                resource_utilization: 0.0,
                critical_path_impact: false,
                calculation_method: DurationMethod::FixedWork,
            });
        }

        // Calculate effective daily capacity based on allocations
        let effective_daily_hours: f32 = resource_allocations.iter()
            .map(|alloc| alloc.daily_hours * (alloc.allocation_percentage / 100.0))
            .sum();

        let duration_days = if effective_daily_hours > 0.0 {
            fixed_work_hours / effective_daily_hours
        } else {
            fixed_work_hours / 8.0
        };

        let avg_utilization = resource_allocations.iter()
            .map(|alloc| alloc.allocation_percentage)
            .sum::<f32>() / resource_allocations.len() as f32;

        Ok(DurationCalculation {
            estimated_duration_days: duration_days,
            estimated_duration_hours: fixed_work_hours,
            working_days: duration_days,
            calendar_days: Self::calculate_calendar_days(duration_days, calendar)?,
            resource_utilization: avg_utilization,
            critical_path_impact: false,
            calculation_method: DurationMethod::FixedWork,
        })
    }

    /// Calculate resource-constrained duration considering availability
    pub fn calculate_resource_constrained_duration(
        task: &Task,
        resource_allocations: &[ResourceAllocation],
        calendar: &Calendar,
        start_date: DateTime<Utc>,
    ) -> Result<DurationCalculation> {
        if resource_allocations.is_empty() {
            return Self::calculate_effort_driven_duration(task, &[], calendar);
        }

        let mut total_work_remaining = task.estimated_hours;
        let mut current_date = start_date;
        let mut working_days = 0.0;

        // Simulate day-by-day resource allocation
        while total_work_remaining > 0.01 {
            let current_naive_date = current_date.date_naive();
            
            if calendar.is_working_day(current_naive_date) {
                let daily_capacity: f32 = resource_allocations.iter()
                    .filter(|alloc| {
                        // Check if resource is available on this date
                        let available = alloc.available_from
                            .map_or(true, |from| current_date >= from) &&
                            alloc.available_until
                            .map_or(true, |until| current_date <= until);
                        available
                    })
                    .map(|alloc| alloc.daily_hours * (alloc.allocation_percentage / 100.0))
                    .sum();

                if daily_capacity > 0.0 {
                    let work_done = daily_capacity.min(total_work_remaining);
                    total_work_remaining -= work_done;
                    working_days += 1.0;
                }
            }

            current_date += Duration::days(1);
            
            // Safety check to prevent infinite loops
            if working_days > 1000.0 {
                return Err(DesignTrackError::Module(
                    "Duration calculation exceeded maximum days".to_string()
                ));
            }
        }

        let calendar_days = (current_date - start_date).num_days();
        let avg_utilization = resource_allocations.iter()
            .map(|alloc| alloc.allocation_percentage)
            .sum::<f32>() / resource_allocations.len() as f32;

        Ok(DurationCalculation {
            estimated_duration_days: working_days,
            estimated_duration_hours: task.estimated_hours,
            working_days,
            calendar_days,
            resource_utilization: avg_utilization,
            critical_path_impact: false,
            calculation_method: DurationMethod::ResourceConstrained,
        })
    }

    /// Calculate the calendar days needed for a given number of working days
    fn calculate_calendar_days(working_days: f32, calendar: &Calendar) -> Result<i64> {
        if working_days <= 0.0 {
            return Ok(0);
        }

        let start_date = Utc::now().date_naive();
        let end_date = calendar.add_working_days(start_date, working_days);
        Ok((end_date - start_date).num_days())
    }

    /// Calculate project duration from task durations and dependencies
    pub fn calculate_project_duration(
        project: &Project,
        calendar: &Calendar,
    ) -> Result<DurationCalculation> {
        if project.tasks.is_empty() {
            return Ok(DurationCalculation {
                estimated_duration_days: 0.0,
                estimated_duration_hours: 0.0,
                working_days: 0.0,
                calendar_days: 0,
                resource_utilization: 0.0,
                critical_path_impact: false,
                calculation_method: DurationMethod::EffortDriven,
            });
        }

        // Find earliest start and latest finish dates
        let earliest_start = project.tasks.values()
            .filter_map(|task| task.start_date)
            .min()
            .unwrap_or_else(|| Utc::now());

        let latest_finish = project.tasks.values()
            .filter_map(|task| task.due_date)
            .max()
            .unwrap_or_else(|| Utc::now());

        let calendar_days = (latest_finish - earliest_start).num_days();
        let working_days = calendar.calculate_working_days_between(
            earliest_start.date_naive(),
            latest_finish.date_naive(),
        ) as f32;

        let total_effort: f32 = project.tasks.values()
            .map(|task| task.estimated_hours)
            .sum();

        // Calculate average resource utilization
        let total_resources = project.resources.len();
        let avg_utilization = if total_resources > 0 {
            let total_capacity = total_resources as f32 * working_days * 8.0;
            if total_capacity > 0.0 {
                (total_effort / total_capacity) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(DurationCalculation {
            estimated_duration_days: working_days,
            estimated_duration_hours: total_effort,
            working_days,
            calendar_days,
            resource_utilization: avg_utilization,
            critical_path_impact: true,
            calculation_method: DurationMethod::EffortDriven,
        })
    }

    /// Optimize task duration by adjusting resource allocation
    pub fn optimize_task_duration(
        task: &Task,
        available_resources: &[Resource],
        target_duration_days: f32,
        calendar: &Calendar,
    ) -> Result<Vec<ResourceAllocation>> {
        if target_duration_days <= 0.0 {
            return Err(DesignTrackError::Validation(
                "Target duration must be positive".to_string()
            ));
        }

        let required_daily_hours = task.estimated_hours / target_duration_days;
        let mut allocations = Vec::new();
        let mut remaining_hours = required_daily_hours;

        // Sort resources by efficiency (hours per cost unit, if available)
        let mut sorted_resources = available_resources.to_vec();
        sorted_resources.sort_by(|a, b| b.daily_hours.partial_cmp(&a.daily_hours).unwrap());

        for resource in sorted_resources {
            if remaining_hours <= 0.0 {
                break;
            }

            let allocation_hours = remaining_hours.min(resource.daily_hours);
            let allocation_percentage = (allocation_hours / resource.daily_hours) * 100.0;

            allocations.push(ResourceAllocation {
                resource_id: resource.id,
                allocation_percentage,
                daily_hours: allocation_hours,
                available_from: None,
                available_until: None,
            });

            remaining_hours -= allocation_hours;
        }

        if remaining_hours > 0.01 {
            return Err(DesignTrackError::Module(
                format!("Insufficient resource capacity. Need {:.1} more daily hours", remaining_hours)
            ));
        }

        Ok(allocations)
    }

    /// Calculate duration variance and performance metrics
    pub fn calculate_duration_variance(
        planned_duration: f32,
        actual_duration: f32,
        planned_effort: f32,
        actual_effort: f32,
    ) -> DurationVarianceMetrics {
        let duration_variance = actual_duration - planned_duration;
        let duration_variance_percent = if planned_duration > 0.0 {
            (duration_variance / planned_duration) * 100.0
        } else {
            0.0
        };

        let effort_variance = actual_effort - planned_effort;
        let effort_variance_percent = if planned_effort > 0.0 {
            (effort_variance / planned_effort) * 100.0
        } else {
            0.0
        };

        let productivity = if actual_duration > 0.0 {
            actual_effort / actual_duration
        } else {
            0.0
        };

        let efficiency = if planned_effort > 0.0 && actual_effort > 0.0 {
            planned_effort / actual_effort
        } else {
            1.0
        };

        DurationVarianceMetrics {
            duration_variance_days: duration_variance,
            duration_variance_percent,
            effort_variance_hours: effort_variance,
            effort_variance_percent,
            productivity_hours_per_day: productivity,
            efficiency_ratio: efficiency,
            performance_category: Self::categorize_performance(duration_variance_percent, effort_variance_percent),
        }
    }

    /// Categorize performance based on variance metrics
    fn categorize_performance(
        duration_variance_percent: f32,
        effort_variance_percent: f32,
    ) -> PerformanceCategory {
        match (duration_variance_percent, effort_variance_percent) {
            (d, e) if d <= -10.0 && e <= -10.0 => PerformanceCategory::Excellent,
            (d, e) if d <= 5.0 && e <= 5.0 => PerformanceCategory::Good,
            (d, e) if d <= 15.0 && e <= 15.0 => PerformanceCategory::Acceptable,
            (d, e) if d <= 25.0 && e <= 25.0 => PerformanceCategory::Poor,
            _ => PerformanceCategory::Critical,
        }
    }

    /// Calculate critical path impact of duration changes
    pub fn analyze_critical_path_impact(
        task_id: Id,
        duration_change_days: f32,
        project: &Project,
    ) -> Result<CriticalPathImpact> {
        // This is a simplified analysis - full critical path analysis would require
        // the scheduling engine
        let task = project.tasks.get(&task_id)
            .ok_or_else(|| DesignTrackError::NotFound(format!("Task {}", task_id)))?;

        // Find tasks that depend on this task
        let dependent_tasks: Vec<_> = project.tasks.values()
            .filter(|t| t.dependencies.contains(&task_id))
            .collect();

        // Calculate potential delays
        let mut max_delay = duration_change_days;
        let mut affected_tasks = vec![task_id];

        // Simple propagation analysis
        for dep_task in dependent_tasks {
            affected_tasks.push(dep_task.id);
            // In a full implementation, this would recursively calculate impacts
        }

        Ok(CriticalPathImpact {
            task_id,
            duration_change_days,
            project_delay_days: max_delay,
            affected_task_count: affected_tasks.len(),
            affected_task_ids: affected_tasks,
            is_critical_path: max_delay > 0.0,
            mitigation_options: Self::suggest_mitigation_options(duration_change_days),
        })
    }

    /// Suggest mitigation options for duration overruns
    fn suggest_mitigation_options(duration_change_days: f32) -> Vec<String> {
        let mut options = Vec::new();

        if duration_change_days > 0.0 {
            options.push("Add additional resources to reduce duration".to_string());
            options.push("Work overtime to catch up".to_string());
            options.push("Reduce scope to fit timeline".to_string());
            options.push("Parallelize tasks where possible".to_string());
            
            if duration_change_days > 5.0 {
                options.push("Consider fast-tracking critical activities".to_string());
                options.push("Evaluate crashing options (cost vs. time trade-off)".to_string());
            }
        }

        options
    }
}

#[derive(Debug, Clone)]
pub struct DurationVarianceMetrics {
    pub duration_variance_days: f32,
    pub duration_variance_percent: f32,
    pub effort_variance_hours: f32,
    pub effort_variance_percent: f32,
    pub productivity_hours_per_day: f32,
    pub efficiency_ratio: f32,
    pub performance_category: PerformanceCategory,
}

#[derive(Debug, Clone)]
pub enum PerformanceCategory {
    Excellent,  // Under budget and ahead of schedule
    Good,       // Minor variances within acceptable range
    Acceptable, // Moderate variances
    Poor,       // Significant overruns
    Critical,   // Major overruns requiring intervention
}

#[derive(Debug, Clone)]
pub struct CriticalPathImpact {
    pub task_id: Id,
    pub duration_change_days: f32,
    pub project_delay_days: f32,
    pub affected_task_count: usize,
    pub affected_task_ids: Vec<Id>,
    pub is_critical_path: bool,
    pub mitigation_options: Vec<String>,
}