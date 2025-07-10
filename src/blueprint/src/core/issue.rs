use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub issue_id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: IssuePriority,
    pub severity: IssueSeverity,
    pub status: IssueStatus,
    pub category: IssueCategory,
    pub reported_by: String,
    pub reported_date: NaiveDateTime,
    pub assigned_to: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub resolution_date: Option<NaiveDateTime>,
    pub resolution_description: Option<String>,
    pub related_tasks: Vec<String>,
    pub related_milestones: Vec<String>,
    pub related_risks: Vec<String>,
    pub attachments: Vec<IssueAttachment>,
    pub comments: Vec<IssueComment>,
    pub escalation_level: EscalationLevel,
    pub business_impact: BusinessImpact,
    pub estimated_effort_hours: Option<f32>,
    pub actual_effort_hours: Option<f32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub comment_id: Uuid,
    pub author: String,
    pub content: String,
    pub created_date: NaiveDateTime,
    pub is_resolution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAttachment {
    pub attachment_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub uploaded_by: String,
    pub uploaded_date: NaiveDateTime,
    pub file_size_bytes: u64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuePriority {
    Critical,   // Must be resolved immediately
    High,       // Should be resolved quickly
    Medium,     // Normal priority
    Low,        // Can be deferred
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Blocker,    // Prevents all work from continuing
    Major,      // Significantly impacts project progress
    Minor,      // Limited impact on project
    Trivial,    // Minimal or no impact
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,           // Issue reported and acknowledged
    InProgress,     // Actively being worked on
    PendingReview,  // Solution implemented, awaiting review
    Resolved,       // Issue fixed and verified
    Closed,         // Issue closed (may be resolved or deferred)
    Deferred,       // Issue postponed to future release
    Duplicate,      // Duplicate of another issue
    CannotReproduce, // Unable to reproduce the issue
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    Technical,      // Technical/engineering issues
    Process,        // Process or methodology issues
    Resource,       // Resource availability or skills issues
    Scope,          // Scope creep or clarification needed
    Quality,        // Quality or defect issues
    Communication,  // Communication breakdown
    External,       // External dependencies or vendor issues
    Requirements,   // Requirements gaps or changes
    Environment,    // Infrastructure or environment issues
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationLevel {
    None,           // No escalation needed
    TeamLead,       // Escalated to team lead
    ProjectManager, // Escalated to project manager
    Stakeholder,    // Escalated to stakeholders
    Executive,      // Escalated to executive level
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessImpact {
    None,           // No business impact
    Low,            // Minor business impact
    Medium,         // Moderate business impact
    High,           // Significant business impact
    Critical,       // Severe business impact
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRegistry {
    pub issues: HashMap<Uuid, Issue>,
    pub escalation_rules: Vec<EscalationRule>,
    pub sla_definitions: Vec<IssueSlaDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub rule_id: Uuid,
    pub name: String,
    pub conditions: EscalationConditions,
    pub action: EscalationAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConditions {
    pub priority_threshold: Option<IssuePriority>,
    pub severity_threshold: Option<IssueSeverity>,
    pub days_open_threshold: Option<u32>,
    pub business_impact_threshold: Option<BusinessImpact>,
    pub categories: Vec<IssueCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationAction {
    pub escalation_level: EscalationLevel,
    pub notify_roles: Vec<String>,
    pub auto_assign_to: Option<String>,
    pub priority_increase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSlaDef {
    pub sla_id: Uuid,
    pub name: String,
    pub priority: IssuePriority,
    pub response_time_hours: u32,
    pub resolution_time_hours: u32,
    pub business_hours_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueMetrics {
    pub total_issues: u32,
    pub open_issues: u32,
    pub overdue_issues: u32,
    pub average_resolution_time_hours: f32,
    pub issues_by_priority: HashMap<IssuePriority, u32>,
    pub issues_by_severity: HashMap<IssueSeverity, u32>,
    pub issues_by_category: HashMap<IssueCategory, u32>,
    pub issues_by_status: HashMap<IssueStatus, u32>,
    pub escalated_issues: u32,
    pub sla_compliance_percentage: f32,
}

impl Issue {
    pub fn new(
        title: String,
        description: String,
        reported_by: String,
    ) -> Self {
        Self {
            issue_id: Uuid::new_v4(),
            title,
            description,
            priority: IssuePriority::Medium,
            severity: IssueSeverity::Minor,
            status: IssueStatus::Open,
            category: IssueCategory::Technical,
            reported_by,
            reported_date: chrono::Utc::now().naive_utc(),
            assigned_to: None,
            due_date: None,
            resolution_date: None,
            resolution_description: None,
            related_tasks: Vec::new(),
            related_milestones: Vec::new(),
            related_risks: Vec::new(),
            attachments: Vec::new(),
            comments: Vec::new(),
            escalation_level: EscalationLevel::None,
            business_impact: BusinessImpact::Low,
            estimated_effort_hours: None,
            actual_effort_hours: None,
            tags: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: IssuePriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_severity(mut self, severity: IssueSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_category(mut self, category: IssueCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_due_date(mut self, due_date: NaiveDate) -> Self {
        self.due_date = Some(due_date);
        self
    }

    pub fn assign_to(&mut self, assignee: String) {
        self.assigned_to = Some(assignee);
        if self.status == IssueStatus::Open {
            self.status = IssueStatus::InProgress;
        }
    }

    pub fn add_comment(&mut self, author: String, content: String) -> Uuid {
        let comment_id = Uuid::new_v4();
        let comment = IssueComment {
            comment_id,
            author,
            content,
            created_date: chrono::Utc::now().naive_utc(),
            is_resolution: false,
        };
        self.comments.push(comment);
        comment_id
    }

    pub fn resolve(&mut self, resolver: String, resolution_description: String) {
        self.status = IssueStatus::Resolved;
        self.resolution_date = Some(chrono::Utc::now().naive_utc());
        self.resolution_description = Some(resolution_description.clone());
        
        // Add resolution comment
        let comment = IssueComment {
            comment_id: Uuid::new_v4(),
            author: resolver,
            content: resolution_description,
            created_date: chrono::Utc::now().naive_utc(),
            is_resolution: true,
        };
        self.comments.push(comment);
    }

    pub fn close(&mut self) {
        self.status = IssueStatus::Closed;
    }

    pub fn escalate(&mut self, level: EscalationLevel) {
        self.escalation_level = level;
        
        // Auto-increase priority on escalation
        match level {
            EscalationLevel::TeamLead => {
                if self.priority == IssuePriority::Low {
                    self.priority = IssuePriority::Medium;
                }
            }
            EscalationLevel::ProjectManager => {
                if matches!(self.priority, IssuePriority::Low | IssuePriority::Medium) {
                    self.priority = IssuePriority::High;
                }
            }
            EscalationLevel::Stakeholder | EscalationLevel::Executive => {
                self.priority = IssuePriority::Critical;
            }
            EscalationLevel::None => {}
        }
    }

    pub fn add_related_task(&mut self, task_id: String) {
        if !self.related_tasks.contains(&task_id) {
            self.related_tasks.push(task_id);
        }
    }

    pub fn add_related_risk(&mut self, risk_id: String) {
        if !self.related_risks.contains(&risk_id) {
            self.related_risks.push(risk_id);
        }
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn days_open(&self) -> u32 {
        let now = chrono::Utc::now().naive_utc();
        (now - self.reported_date).num_days().max(0) as u32
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            let now = chrono::Utc::now().naive_utc().date();
            now > due_date && !matches!(self.status, IssueStatus::Resolved | IssueStatus::Closed)
        } else {
            false
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(self.priority, IssuePriority::Critical) ||
        matches!(self.severity, IssueSeverity::Blocker) ||
        matches!(self.business_impact, BusinessImpact::Critical)
    }
}

impl IssueRegistry {
    pub fn new() -> Self {
        Self {
            issues: HashMap::new(),
            escalation_rules: Vec::new(),
            sla_definitions: Self::default_sla_definitions(),
        }
    }

    pub fn create_issue(&mut self, title: String, description: String, reported_by: String) -> Uuid {
        let issue = Issue::new(title, description, reported_by);
        let issue_id = issue.issue_id;
        self.issues.insert(issue_id, issue);
        
        // Check for auto-escalation
        self.check_escalation_rules(&issue_id);
        
        issue_id
    }

    pub fn get_issue(&self, issue_id: &Uuid) -> Option<&Issue> {
        self.issues.get(issue_id)
    }

    pub fn get_issue_mut(&mut self, issue_id: &Uuid) -> Option<&mut Issue> {
        self.issues.get_mut(issue_id)
    }

    pub fn get_open_issues(&self) -> Vec<&Issue> {
        self.issues.values()
            .filter(|issue| !matches!(issue.status, IssueStatus::Resolved | IssueStatus::Closed))
            .collect()
    }

    pub fn get_critical_issues(&self) -> Vec<&Issue> {
        self.issues.values()
            .filter(|issue| issue.is_critical())
            .collect()
    }

    pub fn get_overdue_issues(&self) -> Vec<&Issue> {
        self.issues.values()
            .filter(|issue| issue.is_overdue())
            .collect()
    }

    pub fn get_issues_by_assignee(&self, assignee: &str) -> Vec<&Issue> {
        self.issues.values()
            .filter(|issue| {
                if let Some(ref assigned_to) = issue.assigned_to {
                    assigned_to == assignee
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_issues_by_category(&self, category: IssueCategory) -> Vec<&Issue> {
        self.issues.values()
            .filter(|issue| issue.category == category)
            .collect()
    }

    pub fn get_issues_by_priority(&self, priority: IssuePriority) -> Vec<&Issue> {
        self.issues.values()
            .filter(|issue| issue.priority == priority)
            .collect()
    }

    pub fn get_issues_by_status(&self, status: IssueStatus) -> Vec<&Issue> {
        self.issues.values()
            .filter(|issue| issue.status == status)
            .collect()
    }

    pub fn calculate_metrics(&self) -> IssueMetrics {
        let total_issues = self.issues.len() as u32;
        let open_issues = self.get_open_issues().len() as u32;
        let overdue_issues = self.get_overdue_issues().len() as u32;
        let escalated_issues = self.issues.values()
            .filter(|issue| issue.escalation_level != EscalationLevel::None)
            .count() as u32;

        // Calculate average resolution time
        let resolved_issues: Vec<_> = self.issues.values()
            .filter(|issue| matches!(issue.status, IssueStatus::Resolved | IssueStatus::Closed))
            .collect();

        let average_resolution_time_hours = if !resolved_issues.is_empty() {
            let total_resolution_time: f64 = resolved_issues.iter()
                .filter_map(|issue| issue.resolution_date)
                .map(|resolution_date| {
                    resolved_issues.iter()
                        .find(|i| i.resolution_date == Some(resolution_date))
                        .map(|i| (resolution_date - i.reported_date).num_hours() as f64)
                        .unwrap_or(0.0)
                })
                .sum();
            
            (total_resolution_time / resolved_issues.len() as f64) as f32
        } else {
            0.0
        };

        // Count by priority
        let mut issues_by_priority = HashMap::new();
        let mut issues_by_severity = HashMap::new();
        let mut issues_by_category = HashMap::new();
        let mut issues_by_status = HashMap::new();

        for issue in self.issues.values() {
            *issues_by_priority.entry(issue.priority).or_insert(0) += 1;
            *issues_by_severity.entry(issue.severity).or_insert(0) += 1;
            *issues_by_category.entry(issue.category).or_insert(0) += 1;
            *issues_by_status.entry(issue.status).or_insert(0) += 1;
        }

        IssueMetrics {
            total_issues,
            open_issues,
            overdue_issues,
            average_resolution_time_hours,
            issues_by_priority,
            issues_by_severity,
            issues_by_category,
            issues_by_status,
            escalated_issues,
            sla_compliance_percentage: 95.0, // Placeholder - would need SLA tracking
        }
    }

    fn check_escalation_rules(&mut self, issue_id: &Uuid) {
        if let Some(issue) = self.issues.get(issue_id) {
            for rule in &self.escalation_rules {
                if rule.enabled && self.matches_escalation_conditions(issue, &rule.conditions) {
                    // Apply escalation
                    if let Some(issue) = self.issues.get_mut(issue_id) {
                        issue.escalate(rule.action.escalation_level);
                        if let Some(ref assignee) = rule.action.auto_assign_to {
                            issue.assign_to(assignee.clone());
                        }
                    }
                    break; // Apply only the first matching rule
                }
            }
        }
    }

    fn matches_escalation_conditions(&self, issue: &Issue, conditions: &EscalationConditions) -> bool {
        // Check priority threshold
        if let Some(priority_threshold) = conditions.priority_threshold {
            if issue.priority < priority_threshold {
                return false;
            }
        }

        // Check severity threshold
        if let Some(severity_threshold) = conditions.severity_threshold {
            if issue.severity < severity_threshold {
                return false;
            }
        }

        // Check days open threshold
        if let Some(days_threshold) = conditions.days_open_threshold {
            if issue.days_open() < days_threshold {
                return false;
            }
        }

        // Check business impact threshold
        if let Some(impact_threshold) = conditions.business_impact_threshold {
            if issue.business_impact < impact_threshold {
                return false;
            }
        }

        // Check categories
        if !conditions.categories.is_empty() && !conditions.categories.contains(&issue.category) {
            return false;
        }

        true
    }

    fn default_sla_definitions() -> Vec<IssueSlaDef> {
        vec![
            IssueSlaDef {
                sla_id: Uuid::new_v4(),
                name: "Critical Issues SLA".to_string(),
                priority: IssuePriority::Critical,
                response_time_hours: 2,
                resolution_time_hours: 24,
                business_hours_only: false,
            },
            IssueSlaDef {
                sla_id: Uuid::new_v4(),
                name: "High Priority Issues SLA".to_string(),
                priority: IssuePriority::High,
                response_time_hours: 8,
                resolution_time_hours: 72,
                business_hours_only: true,
            },
            IssueSlaDef {
                sla_id: Uuid::new_v4(),
                name: "Medium Priority Issues SLA".to_string(),
                priority: IssuePriority::Medium,
                response_time_hours: 24,
                resolution_time_hours: 168, // 1 week
                business_hours_only: true,
            },
            IssueSlaDef {
                sla_id: Uuid::new_v4(),
                name: "Low Priority Issues SLA".to_string(),
                priority: IssuePriority::Low,
                response_time_hours: 72,
                resolution_time_hours: 336, // 2 weeks
                business_hours_only: true,
            },
        ]
    }
}

impl Default for IssueRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Implement ordering for priority and severity to enable threshold comparisons
impl PartialOrd for IssuePriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_value = match self {
            IssuePriority::Low => 1,
            IssuePriority::Medium => 2,
            IssuePriority::High => 3,
            IssuePriority::Critical => 4,
        };
        let other_value = match other {
            IssuePriority::Low => 1,
            IssuePriority::Medium => 2,
            IssuePriority::High => 3,
            IssuePriority::Critical => 4,
        };
        self_value.partial_cmp(&other_value)
    }
}

impl PartialOrd for IssueSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_value = match self {
            IssueSeverity::Trivial => 1,
            IssueSeverity::Minor => 2,
            IssueSeverity::Major => 3,
            IssueSeverity::Blocker => 4,
        };
        let other_value = match other {
            IssueSeverity::Trivial => 1,
            IssueSeverity::Minor => 2,
            IssueSeverity::Major => 3,
            IssueSeverity::Blocker => 4,
        };
        self_value.partial_cmp(&other_value)
    }
}

impl PartialOrd for BusinessImpact {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_value = match self {
            BusinessImpact::None => 0,
            BusinessImpact::Low => 1,
            BusinessImpact::Medium => 2,
            BusinessImpact::High => 3,
            BusinessImpact::Critical => 4,
        };
        let other_value = match other {
            BusinessImpact::None => 0,
            BusinessImpact::Low => 1,
            BusinessImpact::Medium => 2,
            BusinessImpact::High => 3,
            BusinessImpact::Critical => 4,
        };
        self_value.partial_cmp(&other_value)
    }
}

impl std::fmt::Display for IssuePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssuePriority::Critical => write!(f, "Critical"),
            IssuePriority::High => write!(f, "High"),
            IssuePriority::Medium => write!(f, "Medium"),
            IssuePriority::Low => write!(f, "Low"),
        }
    }
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueStatus::Open => write!(f, "Open"),
            IssueStatus::InProgress => write!(f, "In Progress"),
            IssueStatus::PendingReview => write!(f, "Pending Review"),
            IssueStatus::Resolved => write!(f, "Resolved"),
            IssueStatus::Closed => write!(f, "Closed"),
            IssueStatus::Deferred => write!(f, "Deferred"),
            IssueStatus::Duplicate => write!(f, "Duplicate"),
            IssueStatus::CannotReproduce => write!(f, "Cannot Reproduce"),
        }
    }
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Blocker => write!(f, "Blocker"),
            IssueSeverity::Major => write!(f, "Major"),
            IssueSeverity::Minor => write!(f, "Minor"),
            IssueSeverity::Trivial => write!(f, "Trivial"),
        }
    }
}

impl std::fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueCategory::Technical => write!(f, "Technical"),
            IssueCategory::Process => write!(f, "Process"),
            IssueCategory::Resource => write!(f, "Resource"),
            IssueCategory::Scope => write!(f, "Scope"),
            IssueCategory::Quality => write!(f, "Quality"),
            IssueCategory::Communication => write!(f, "Communication"),
            IssueCategory::External => write!(f, "External"),
            IssueCategory::Requirements => write!(f, "Requirements"),
            IssueCategory::Environment => write!(f, "Environment"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_issue_creation() {
        let issue = Issue::new(
            "Test Issue".to_string(),
            "Test description".to_string(),
            "reporter@example.com".to_string(),
        );

        assert!(!issue.issue_id.to_string().is_empty());
        assert_eq!(issue.title, "Test Issue");
        assert_eq!(issue.status, IssueStatus::Open);
        assert_eq!(issue.priority, IssuePriority::Medium);
    }

    #[test]
    fn test_issue_assignment() {
        let mut issue = Issue::new(
            "Test Issue".to_string(),
            "Test description".to_string(),
            "reporter@example.com".to_string(),
        );

        issue.assign_to("developer@example.com".to_string());
        
        assert_eq!(issue.assigned_to, Some("developer@example.com".to_string()));
        assert_eq!(issue.status, IssueStatus::InProgress);
    }

    #[test]
    fn test_issue_resolution() {
        let mut issue = Issue::new(
            "Test Issue".to_string(),
            "Test description".to_string(),
            "reporter@example.com".to_string(),
        );

        issue.resolve("developer@example.com".to_string(), "Fixed the bug".to_string());
        
        assert_eq!(issue.status, IssueStatus::Resolved);
        assert!(issue.resolution_date.is_some());
        assert_eq!(issue.resolution_description, Some("Fixed the bug".to_string()));
        assert!(!issue.comments.is_empty());
    }

    #[test]
    fn test_issue_registry() {
        let mut registry = IssueRegistry::new();
        
        let issue_id = registry.create_issue(
            "Test Issue".to_string(),
            "Test description".to_string(),
            "reporter@example.com".to_string(),
        );

        assert!(!issue_id.to_string().is_empty());
        assert!(registry.get_issue(&issue_id).is_some());
        assert_eq!(registry.get_open_issues().len(), 1);

        let metrics = registry.calculate_metrics();
        assert_eq!(metrics.total_issues, 1);
        assert_eq!(metrics.open_issues, 1);
    }

    #[test]
    fn test_issue_escalation() {
        let mut issue = Issue::new(
            "Test Issue".to_string(),
            "Test description".to_string(),
            "reporter@example.com".to_string(),
        );

        issue.escalate(EscalationLevel::ProjectManager);
        
        assert_eq!(issue.escalation_level, EscalationLevel::ProjectManager);
        assert_eq!(issue.priority, IssuePriority::High);
    }
}