use crate::core::{Project, EarnedValueMetrics, ProjectStatus};
use crate::scheduling::Schedule;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDashboard {
    pub project_name: String,
    pub generated_date: NaiveDateTime,
    pub status_date: NaiveDate,
    pub overall_health: ProjectHealth,
    pub executive_summary: ExecutiveSummary,
    pub schedule_status: ScheduleStatus,
    pub cost_status: CostStatus,
    pub scope_status: ScopeStatus,
    pub quality_status: QualityStatus,
    pub risk_status: RiskStatus,
    pub issue_status: IssueStatus,
    pub resource_status: ResourceStatus,
    pub key_metrics: KeyMetrics,
    pub trending_indicators: TrendingIndicators,
    pub alerts_and_actions: AlertsAndActions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectHealth {
    Green,   // On track, no significant issues
    Yellow,  // At risk, requires attention
    Red,     // Critical issues, immediate action required
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub overall_status: ProjectStatus,
    pub completion_percentage: f32,
    pub days_remaining: i32,
    pub budget_consumed_percentage: f32,
    pub critical_issues_count: u32,
    pub high_risks_count: u32,
    pub key_accomplishments: Vec<String>,
    pub immediate_concerns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStatus {
    pub health: ProjectHealth,
    pub schedule_performance_index: f32, // SPI
    pub schedule_variance_days: i32,
    pub critical_path_status: CriticalPathStatus,
    pub milestones_status: MilestonesStatus,
    pub tasks_completed_on_time: u32,
    pub tasks_behind_schedule: u32,
    pub forecast_completion_date: NaiveDate,
    pub schedule_confidence: ConfidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStatus {
    pub health: ProjectHealth,
    pub cost_performance_index: f32, // CPI
    pub cost_variance: f32,
    pub budget_at_completion: f32,
    pub estimate_at_completion: f32,
    pub variance_at_completion: f32,
    pub burn_rate: f32, // Cost per day
    pub budget_utilization_rate: f32,
    pub cost_confidence: ConfidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeStatus {
    pub health: ProjectHealth,
    pub scope_completion_percentage: f32,
    pub deliverables_completed: u32,
    pub deliverables_remaining: u32,
    pub scope_changes_approved: u32,
    pub scope_changes_pending: u32,
    pub scope_creep_indicator: f32, // Percentage increase from baseline
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStatus {
    pub health: ProjectHealth,
    pub quality_metrics: QualityMetrics,
    pub defect_density: f32,
    pub customer_satisfaction_score: Option<f32>,
    pub review_pass_rate: f32,
    pub rework_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStatus {
    pub health: ProjectHealth,
    pub total_active_risks: u32,
    pub high_probability_high_impact: u32,
    pub risks_requiring_immediate_action: u32,
    pub risks_mitigated_this_period: u32,
    pub top_risks: Vec<TopRisk>,
    pub risk_trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatus {
    pub health: ProjectHealth,
    pub total_open_issues: u32,
    pub critical_issues: u32,
    pub overdue_issues: u32,
    pub average_resolution_time_days: f32,
    pub issue_trend: TrendDirection,
    pub sla_compliance_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    pub health: ProjectHealth,
    pub resource_utilization: HashMap<String, ResourceUtilization>,
    pub average_utilization_percentage: f32,
    pub overallocated_resources: u32,
    pub underutilized_resources: u32,
    pub resource_conflicts: u32,
    pub availability_concerns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub resource_name: String,
    pub current_allocation: f32,
    pub planned_allocation: f32,
    pub utilization_health: ProjectHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPathStatus {
    pub health: ProjectHealth,
    pub critical_path_length_days: i32,
    pub float_available_days: i32,
    pub tasks_on_critical_path: u32,
    pub critical_path_completion: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestonesStatus {
    pub health: ProjectHealth,
    pub total_milestones: u32,
    pub completed_milestones: u32,
    pub milestones_at_risk: u32,
    pub upcoming_milestone: Option<UpcomingMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingMilestone {
    pub name: String,
    pub target_date: NaiveDate,
    pub completion_percentage: f32,
    pub risk_level: ProjectHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub defects_found: u32,
    pub defects_resolved: u32,
    pub test_coverage_percentage: f32,
    pub code_review_coverage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopRisk {
    pub risk_id: String,
    pub title: String,
    pub probability: String,
    pub impact: String,
    pub risk_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetrics {
    pub earned_value: EarnedValueMetrics,
    pub productivity_metrics: ProductivityMetrics,
    pub performance_indicators: Vec<KPI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub velocity: f32, // Tasks or story points completed per time period
    pub throughput: f32, // Work items completed per time period
    pub cycle_time_days: f32, // Average time from start to completion
    pub lead_time_days: f32, // Average time from request to delivery
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPI {
    pub name: String,
    pub current_value: f32,
    pub target_value: f32,
    pub unit: String,
    pub trend: TrendDirection,
    pub health: ProjectHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingIndicators {
    pub schedule_trend: TrendData,
    pub cost_trend: TrendData,
    pub quality_trend: TrendData,
    pub risk_trend: TrendData,
    pub velocity_trend: TrendData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub direction: TrendDirection,
    pub rate_of_change: f32,
    pub confidence: ConfidenceLevel,
    pub historical_data: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub date: NaiveDate,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsAndActions {
    pub critical_alerts: Vec<Alert>,
    pub warnings: Vec<Alert>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub escalations_required: Vec<Escalation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub created_date: NaiveDateTime,
    pub category: AlertCategory,
    pub recommended_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_id: String,
    pub title: String,
    pub description: String,
    pub priority: ActionPriority,
    pub estimated_effort: Option<String>,
    pub responsible_party: Option<String>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub escalation_id: String,
    pub title: String,
    pub reason: String,
    pub escalation_level: EscalationLevel,
    pub stakeholders_to_notify: Vec<String>,
    pub urgency: EscalationUrgency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCategory {
    Schedule,
    Cost,
    Quality,
    Risk,
    Resource,
    Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationLevel {
    TeamLead,
    ProjectManager,
    Stakeholder,
    Executive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationUrgency {
    Immediate,
    Within24Hours,
    WithinWeek,
    NextReview,
}

pub struct DashboardGenerator;

impl DashboardGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_dashboard(
        &self,
        project: &Project,
        schedule: &Schedule,
        status_date: NaiveDate,
    ) -> ProjectDashboard {
        let overall_health = self.calculate_overall_health(project, schedule);
        let executive_summary = self.generate_executive_summary(project, schedule, status_date);
        let schedule_status = self.generate_schedule_status(project, schedule, status_date);
        let cost_status = self.generate_cost_status(project, schedule, status_date);
        let scope_status = self.generate_scope_status(project, schedule);
        let quality_status = self.generate_quality_status(project);
        let risk_status = self.generate_risk_status(project);
        let issue_status = self.generate_issue_status(project);
        let resource_status = self.generate_resource_status(project, schedule);
        let key_metrics = self.generate_key_metrics(project, schedule, status_date);
        let trending_indicators = self.generate_trending_indicators(project);
        let alerts_and_actions = self.generate_alerts_and_actions(project, schedule);

        ProjectDashboard {
            project_name: project.name.clone(),
            generated_date: chrono::Utc::now().naive_utc(),
            status_date,
            overall_health,
            executive_summary,
            schedule_status,
            cost_status,
            scope_status,
            quality_status,
            risk_status,
            issue_status,
            resource_status,
            key_metrics,
            trending_indicators,
            alerts_and_actions,
        }
    }

    fn calculate_overall_health(&self, project: &Project, _schedule: &Schedule) -> ProjectHealth {
        let project_health = project.assess_project_health();
        match project_health {
            ProjectStatus::Green => ProjectHealth::Green,
            ProjectStatus::Yellow => ProjectHealth::Yellow,
            ProjectStatus::Red => ProjectHealth::Red,
            _ => ProjectHealth::Yellow,
        }
    }

    fn generate_executive_summary(
        &self,
        project: &Project,
        schedule: &Schedule,
        status_date: NaiveDate,
    ) -> ExecutiveSummary {
        let overall_status = project.assess_project_health();
        
        // Calculate completion percentage
        let completion_percentage = if let Some(progress) = project.get_latest_progress_snapshot() {
            progress.calculate_overall_completion()
        } else {
            0.0
        };

        // Calculate days remaining
        let days_remaining = (schedule.end_date - status_date).num_days().max(0) as i32;

        // Calculate budget consumed
        let budget_consumed_percentage = if let Some(ev_metrics) = project.calculate_earned_value(status_date) {
            ev_metrics.percent_spent
        } else {
            0.0
        };

        let critical_issues_count = project.get_critical_issues().len() as u32;
        let high_risks_count = project.get_high_priority_risks().len() as u32;

        // Generate key accomplishments and concerns
        let key_accomplishments = vec![
            "Project planning and baseline established".to_string(),
            "Team resources allocated and onboarded".to_string(),
        ];

        let mut immediate_concerns = Vec::new();
        if critical_issues_count > 0 {
            immediate_concerns.push(format!("{} critical issues require immediate attention", critical_issues_count));
        }
        if high_risks_count > 2 {
            immediate_concerns.push(format!("{} high-priority risks need mitigation", high_risks_count));
        }

        ExecutiveSummary {
            overall_status,
            completion_percentage,
            days_remaining,
            budget_consumed_percentage,
            critical_issues_count,
            high_risks_count,
            key_accomplishments,
            immediate_concerns,
        }
    }

    fn generate_schedule_status(
        &self,
        project: &Project,
        schedule: &Schedule,
        status_date: NaiveDate,
    ) -> ScheduleStatus {
        let schedule_performance_index = if let Some(ev_metrics) = project.calculate_earned_value(status_date) {
            ev_metrics.schedule_performance_index
        } else {
            1.0
        };

        let health = if schedule_performance_index >= 0.95 {
            ProjectHealth::Green
        } else if schedule_performance_index >= 0.85 {
            ProjectHealth::Yellow
        } else {
            ProjectHealth::Red
        };

        let schedule_variance_days = if let Some(baseline) = project.get_current_baseline() {
            (schedule.end_date - baseline.project_snapshot.end_date).num_days() as i32
        } else {
            0
        };

        // Calculate critical path status
        let critical_path_status = CriticalPathStatus {
            health: ProjectHealth::Green, // Placeholder
            critical_path_length_days: (schedule.end_date - schedule.start_date).num_days() as i32,
            float_available_days: 0, // Would need float calculation
            tasks_on_critical_path: schedule.critical_path.len() as u32,
            critical_path_completion: 50.0, // Placeholder
        };

        // Calculate milestones status
        let milestones_status = MilestonesStatus {
            health: ProjectHealth::Green,
            total_milestones: schedule.milestones.len() as u32,
            completed_milestones: 0, // Would need progress data
            milestones_at_risk: 0,
            upcoming_milestone: None,
        };

        ScheduleStatus {
            health,
            schedule_performance_index,
            schedule_variance_days,
            critical_path_status,
            milestones_status,
            tasks_completed_on_time: 0, // Would need progress tracking
            tasks_behind_schedule: 0,
            forecast_completion_date: schedule.end_date,
            schedule_confidence: ConfidenceLevel::Medium,
        }
    }

    fn generate_cost_status(
        &self,
        project: &Project,
        _schedule: &Schedule,
        status_date: NaiveDate,
    ) -> CostStatus {
        if let Some(ev_metrics) = project.calculate_earned_value(status_date) {
            let health = if ev_metrics.cost_performance_index >= 0.95 {
                ProjectHealth::Green
            } else if ev_metrics.cost_performance_index >= 0.85 {
                ProjectHealth::Yellow
            } else {
                ProjectHealth::Red
            };

            CostStatus {
                health,
                cost_performance_index: ev_metrics.cost_performance_index,
                cost_variance: ev_metrics.cost_variance,
                budget_at_completion: ev_metrics.budget_at_completion,
                estimate_at_completion: ev_metrics.estimate_at_completion,
                variance_at_completion: ev_metrics.variance_at_completion,
                burn_rate: ev_metrics.actual_cost / status_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f32, // Simplified
                budget_utilization_rate: ev_metrics.percent_spent / 100.0,
                cost_confidence: ConfidenceLevel::Medium,
            }
        } else {
            // Default values when no earned value data available
            CostStatus {
                health: ProjectHealth::Green,
                cost_performance_index: 1.0,
                cost_variance: 0.0,
                budget_at_completion: 0.0,
                estimate_at_completion: 0.0,
                variance_at_completion: 0.0,
                burn_rate: 0.0,
                budget_utilization_rate: 0.0,
                cost_confidence: ConfidenceLevel::Low,
            }
        }
    }

    fn generate_scope_status(&self, _project: &Project, schedule: &Schedule) -> ScopeStatus {
        ScopeStatus {
            health: ProjectHealth::Green,
            scope_completion_percentage: 0.0, // Would need progress tracking
            deliverables_completed: 0,
            deliverables_remaining: schedule.tasks.len() as u32,
            scope_changes_approved: 0,
            scope_changes_pending: 0,
            scope_creep_indicator: 0.0,
        }
    }

    fn generate_quality_status(&self, _project: &Project) -> QualityStatus {
        QualityStatus {
            health: ProjectHealth::Green,
            quality_metrics: QualityMetrics {
                defects_found: 0,
                defects_resolved: 0,
                test_coverage_percentage: 0.0,
                code_review_coverage: 0.0,
            },
            defect_density: 0.0,
            customer_satisfaction_score: None,
            review_pass_rate: 0.0,
            rework_percentage: 0.0,
        }
    }

    fn generate_risk_status(&self, project: &Project) -> RiskStatus {
        let risk_metrics = project.risk_registry.calculate_metrics();
        let high_priority_risks = project.get_high_priority_risks();
        
        let health = if risk_metrics.risks_requiring_action > 2 {
            ProjectHealth::Red
        } else if risk_metrics.risks_requiring_action > 0 {
            ProjectHealth::Yellow
        } else {
            ProjectHealth::Green
        };

        let top_risks: Vec<TopRisk> = high_priority_risks.iter()
            .take(5)
            .map(|risk| TopRisk {
                risk_id: risk.risk_id.to_string(),
                title: risk.title.clone(),
                probability: format!("{}", risk.probability),
                impact: format!("{}", risk.impact),
                risk_score: risk.risk_score,
            })
            .collect();

        RiskStatus {
            health,
            total_active_risks: risk_metrics.active_risks,
            high_probability_high_impact: risk_metrics.risks_requiring_action,
            risks_requiring_immediate_action: risk_metrics.risks_requiring_action,
            risks_mitigated_this_period: 0, // Would need historical tracking
            top_risks,
            risk_trend: TrendDirection::Stable,
        }
    }

    fn generate_issue_status(&self, project: &Project) -> IssueStatus {
        let issue_metrics = project.issue_registry.calculate_metrics();
        
        let critical_count = issue_metrics.issues_by_priority.get(&crate::core::IssuePriority::Critical).unwrap_or(&0);
        
        let health = if *critical_count > 0 {
            ProjectHealth::Red
        } else if issue_metrics.overdue_issues > 0 {
            ProjectHealth::Yellow
        } else {
            ProjectHealth::Green
        };

        IssueStatus {
            health,
            total_open_issues: issue_metrics.open_issues,
            critical_issues: *critical_count,
            overdue_issues: issue_metrics.overdue_issues,
            average_resolution_time_days: issue_metrics.average_resolution_time_hours / 24.0,
            issue_trend: TrendDirection::Stable,
            sla_compliance_percentage: issue_metrics.sla_compliance_percentage,
        }
    }

    fn generate_resource_status(&self, _project: &Project, schedule: &Schedule) -> ResourceStatus {
        let mut resource_utilization = HashMap::new();
        let mut total_utilization = 0.0;
        let mut resource_count = 0;

        for (resource_id, utilization) in &schedule.resource_utilization {
            let health = if utilization.utilization_percentage > 100.0 {
                ProjectHealth::Red
            } else if utilization.utilization_percentage > 90.0 {
                ProjectHealth::Yellow
            } else {
                ProjectHealth::Green
            };

            resource_utilization.insert(resource_id.clone(), ResourceUtilization {
                resource_name: utilization.name.clone(),
                current_allocation: utilization.utilization_percentage / 100.0,
                planned_allocation: 1.0, // Placeholder
                utilization_health: health,
            });

            total_utilization += utilization.utilization_percentage;
            resource_count += 1;
        }

        let average_utilization = if resource_count > 0 {
            total_utilization / resource_count as f32
        } else {
            0.0
        };

        let overallocated_resources = resource_utilization.values()
            .filter(|r| r.current_allocation > 1.0)
            .count() as u32;

        let underutilized_resources = resource_utilization.values()
            .filter(|r| r.current_allocation < 0.5)
            .count() as u32;

        ResourceStatus {
            health: ProjectHealth::Green,
            resource_utilization,
            average_utilization_percentage: average_utilization,
            overallocated_resources,
            underutilized_resources,
            resource_conflicts: 0,
            availability_concerns: Vec::new(),
        }
    }

    fn generate_key_metrics(
        &self,
        project: &Project,
        _schedule: &Schedule,
        status_date: NaiveDate,
    ) -> KeyMetrics {
        let earned_value = project.calculate_earned_value(status_date)
            .unwrap_or_else(|| EarnedValueMetrics {
                status_date,
                project_name: project.name.clone(),
                planned_value: 0.0,
                earned_value: 0.0,
                actual_cost: 0.0,
                budget_at_completion: 0.0,
                schedule_variance: 0.0,
                cost_variance: 0.0,
                schedule_performance_index: 1.0,
                cost_performance_index: 1.0,
                estimate_at_completion: 0.0,
                estimate_to_complete: 0.0,
                variance_at_completion: 0.0,
                percent_complete: 0.0,
                percent_spent: 0.0,
            });

        let productivity_metrics = ProductivityMetrics {
            velocity: 0.0, // Would need historical data
            throughput: 0.0,
            cycle_time_days: 0.0,
            lead_time_days: 0.0,
        };

        let performance_indicators = vec![
            KPI {
                name: "Schedule Performance".to_string(),
                current_value: earned_value.schedule_performance_index,
                target_value: 1.0,
                unit: "ratio".to_string(),
                trend: TrendDirection::Stable,
                health: if earned_value.schedule_performance_index >= 0.95 { ProjectHealth::Green } else { ProjectHealth::Yellow },
            },
            KPI {
                name: "Cost Performance".to_string(),
                current_value: earned_value.cost_performance_index,
                target_value: 1.0,
                unit: "ratio".to_string(),
                trend: TrendDirection::Stable,
                health: if earned_value.cost_performance_index >= 0.95 { ProjectHealth::Green } else { ProjectHealth::Yellow },
            },
        ];

        KeyMetrics {
            earned_value,
            productivity_metrics,
            performance_indicators,
        }
    }

    fn generate_trending_indicators(&self, _project: &Project) -> TrendingIndicators {
        // Placeholder implementation - would need historical data
        let default_trend = TrendData {
            direction: TrendDirection::Stable,
            rate_of_change: 0.0,
            confidence: ConfidenceLevel::Low,
            historical_data: Vec::new(),
        };

        TrendingIndicators {
            schedule_trend: default_trend.clone(),
            cost_trend: default_trend.clone(),
            quality_trend: default_trend.clone(),
            risk_trend: default_trend.clone(),
            velocity_trend: default_trend,
        }
    }

    fn generate_alerts_and_actions(
        &self,
        project: &Project,
        _schedule: &Schedule,
    ) -> AlertsAndActions {
        let mut critical_alerts = Vec::new();
        let mut warnings = Vec::new();
        let mut recommended_actions = Vec::new();
        let escalations_required = Vec::new();

        // Check for critical issues
        let critical_issues = project.get_critical_issues();
        if !critical_issues.is_empty() {
            critical_alerts.push(Alert {
                alert_id: "ALERT-001".to_string(),
                severity: AlertSeverity::Critical,
                title: "Critical Issues Detected".to_string(),
                description: format!("{} critical issues require immediate attention", critical_issues.len()),
                created_date: chrono::Utc::now().naive_utc(),
                category: AlertCategory::Quality,
                recommended_action: Some("Review and prioritize critical issues for immediate resolution".to_string()),
            });
        }

        // Check for high-priority risks
        let high_risks = project.get_high_priority_risks();
        if high_risks.len() > 2 {
            warnings.push(Alert {
                alert_id: "WARN-001".to_string(),
                severity: AlertSeverity::High,
                title: "Multiple High-Priority Risks".to_string(),
                description: format!("{} high-priority risks identified", high_risks.len()),
                created_date: chrono::Utc::now().naive_utc(),
                category: AlertCategory::Risk,
                recommended_action: Some("Review risk mitigation strategies and escalate if necessary".to_string()),
            });
        }

        // Generate recommended actions
        if !critical_issues.is_empty() {
            recommended_actions.push(RecommendedAction {
                action_id: "ACTION-001".to_string(),
                title: "Address Critical Issues".to_string(),
                description: "Conduct immediate review of all critical issues and assign resolution owners".to_string(),
                priority: ActionPriority::Critical,
                estimated_effort: Some("4-8 hours".to_string()),
                responsible_party: Some("Project Manager".to_string()),
                due_date: Some(chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1)),
            });
        }

        AlertsAndActions {
            critical_alerts,
            warnings,
            recommended_actions,
            escalations_required,
        }
    }
}

impl Default for DashboardGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Project;
    use chrono::NaiveDate;

    #[test]
    fn test_dashboard_generation() {
        let project = Project::new("Test Project".to_string(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let schedule = crate::scheduling::Schedule {
            project_name: "Test Project".to_string(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            tasks: indexmap::IndexMap::new(),
            milestones: indexmap::IndexMap::new(),
            critical_path: Vec::new(),
            total_cost: 100000.0,
            resource_utilization: std::collections::HashMap::new(),
        };

        let dashboard_generator = DashboardGenerator::new();
        let dashboard = dashboard_generator.generate_dashboard(
            &project,
            &schedule,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        );

        assert_eq!(dashboard.project_name, "Test Project");
        assert_eq!(dashboard.status_date, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        assert_eq!(dashboard.overall_health, ProjectHealth::Green);
    }

    #[test]
    fn test_health_calculation() {
        let mut project = Project::new("Test Project".to_string(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        
        // Add a critical issue
        project.create_issue(
            "Critical Bug".to_string(),
            "System crash".to_string(),
            "tester@example.com".to_string(),
        );

        let issue_uuid = *project.issue_registry.issues.keys().next().unwrap();
        if let Some(issue) = project.issue_registry.get_issue_mut(&issue_uuid) {
            issue.priority = crate::core::IssuePriority::Critical;
        }

        let health = project.assess_project_health();
        assert_eq!(health, crate::core::ProjectStatus::Red);
    }
}