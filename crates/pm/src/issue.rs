use tessera_core::{Id, Entity, Repository, Result, DesignTrackError, format_ron_pretty};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub priority: IssuePriority,
    pub severity: IssueSeverity,
    pub status: IssueStatus,
    pub category: IssueCategory,
    pub reported_by: String,
    pub reported_date: DateTime<Utc>,
    pub assigned_to: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub resolution_date: Option<DateTime<Utc>>,
    pub resolution_description: Option<String>,
    pub related_tasks: Vec<Id>,
    pub related_milestones: Vec<Id>,
    pub related_risks: Vec<Id>,
    pub attachments: Vec<IssueAttachment>,
    pub comments: Vec<IssueComment>,
    pub escalation_level: EscalationLevel,
    pub business_impact: BusinessImpact,
    pub estimated_effort_hours: Option<f32>,
    pub actual_effort_hours: Option<f32>,
    pub tags: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: Id,
    pub author: String,
    pub content: String,
    pub created_date: DateTime<Utc>,
    pub is_resolution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAttachment {
    pub id: Id,
    pub filename: String,
    pub file_path: String,
    pub uploaded_by: String,
    pub uploaded_date: DateTime<Utc>,
    pub file_size_bytes: u64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssuePriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueSeverity {
    Blocker,
    Major,
    Minor,
    Trivial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueStatus {
    Open,
    InProgress,
    PendingReview,
    Resolved,
    Closed,
    Deferred,
    Duplicate,
    CannotReproduce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
    Technical,
    Process,
    Resource,
    Scope,
    Quality,
    Communication,
    External,
    Requirements,
    Environment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EscalationLevel {
    None,
    TeamLead,
    ProjectManager,
    Stakeholder,
    Executive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BusinessImpact {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRegistry {
    pub issues: HashMap<Id, Issue>,
    pub escalation_rules: Vec<EscalationRule>,
    pub sla_definitions: Vec<IssueSlaDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub id: Id,
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
    pub id: Id,
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

impl Entity for Issue {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.title
    }

    fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(DesignTrackError::Validation("Issue title cannot be empty".to_string()));
        }
        if self.reported_by.trim().is_empty() {
            return Err(DesignTrackError::Validation("Issue must have a reported_by field".to_string()));
        }
        Ok(())
    }
}

impl Issue {
    pub fn new(title: String, description: String, reported_by: String) -> Self {
        let now = Utc::now();
        Self {
            id: Id::new(),
            title,
            description,
            priority: IssuePriority::Medium,
            severity: IssueSeverity::Minor,
            status: IssueStatus::Open,
            category: IssueCategory::Technical,
            reported_by,
            reported_date: now,
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
            created: now,
            updated: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: IssuePriority) -> Self {
        self.priority = priority;
        self.updated = Utc::now();
        self
    }

    pub fn with_severity(mut self, severity: IssueSeverity) -> Self {
        self.severity = severity;
        self.updated = Utc::now();
        self
    }

    pub fn with_category(mut self, category: IssueCategory) -> Self {
        self.category = category;
        self.updated = Utc::now();
        self
    }

    pub fn with_due_date(mut self, due_date: NaiveDate) -> Self {
        self.due_date = Some(due_date);
        self.updated = Utc::now();
        self
    }

    pub fn assign_to(&mut self, assignee: String) {
        self.assigned_to = Some(assignee);
        if self.status == IssueStatus::Open {
            self.status = IssueStatus::InProgress;
        }
        self.updated = Utc::now();
    }

    pub fn add_comment(&mut self, author: String, content: String) -> Id {
        let comment = IssueComment {
            id: Id::new(),
            author,
            content,
            created_date: Utc::now(),
            is_resolution: false,
        };
        let comment_id = comment.id;
        self.comments.push(comment);
        self.updated = Utc::now();
        comment_id
    }

    pub fn resolve(&mut self, resolver: String, resolution_description: String) {
        self.status = IssueStatus::Resolved;
        self.resolution_date = Some(Utc::now());
        self.resolution_description = Some(resolution_description.clone());
        
        let comment = IssueComment {
            id: Id::new(),
            author: resolver,
            content: resolution_description,
            created_date: Utc::now(),
            is_resolution: true,
        };
        self.comments.push(comment);
        self.updated = Utc::now();
    }

    pub fn close(&mut self) {
        self.status = IssueStatus::Closed;
        self.updated = Utc::now();
    }

    pub fn escalate(&mut self, level: EscalationLevel) {
        self.escalation_level = level;
        
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
        self.updated = Utc::now();
    }

    pub fn add_related_task(&mut self, task_id: Id) {
        if !self.related_tasks.contains(&task_id) {
            self.related_tasks.push(task_id);
            self.updated = Utc::now();
        }
    }

    pub fn add_related_risk(&mut self, risk_id: Id) {
        if !self.related_risks.contains(&risk_id) {
            self.related_risks.push(risk_id);
            self.updated = Utc::now();
        }
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated = Utc::now();
        }
    }

    pub fn days_open(&self) -> u32 {
        let now = Utc::now();
        (now - self.reported_date).num_days().max(0) as u32
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            let now = Utc::now().date_naive();
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

    pub fn create_issue(&mut self, title: String, description: String, reported_by: String) -> Id {
        let issue = Issue::new(title, description, reported_by);
        let issue_id = issue.id;
        self.issues.insert(issue_id, issue);
        
        self.check_escalation_rules(&issue_id);
        
        issue_id
    }

    pub fn get_issue(&self, issue_id: &Id) -> Option<&Issue> {
        self.issues.get(issue_id)
    }

    pub fn get_issue_mut(&mut self, issue_id: &Id) -> Option<&mut Issue> {
        self.issues.get_mut(issue_id)
    }

    pub fn get_all_issues(&self) -> Vec<&Issue> {
        self.issues.values().collect()
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

        let resolved_issues: Vec<_> = self.issues.values()
            .filter(|issue| matches!(issue.status, IssueStatus::Resolved | IssueStatus::Closed))
            .collect();

        let average_resolution_time_hours = if !resolved_issues.is_empty() {
            let total_resolution_time: f64 = resolved_issues.iter()
                .filter_map(|issue| issue.resolution_date)
                .zip(resolved_issues.iter())
                .map(|(resolution_date, issue)| {
                    (resolution_date - issue.reported_date).num_hours() as f64
                })
                .sum();
            
            (total_resolution_time / resolved_issues.len() as f64) as f32
        } else {
            0.0
        };

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
            sla_compliance_percentage: 95.0,
        }
    }

    fn check_escalation_rules(&mut self, issue_id: &Id) {
        if let Some(issue) = self.issues.get(issue_id) {
            for rule in &self.escalation_rules.clone() {
                if rule.enabled && self.matches_escalation_conditions(issue, &rule.conditions) {
                    if let Some(issue) = self.issues.get_mut(issue_id) {
                        issue.escalate(rule.action.escalation_level);
                        if let Some(ref assignee) = rule.action.auto_assign_to {
                            issue.assign_to(assignee.clone());
                        }
                    }
                    break;
                }
            }
        }
    }

    fn matches_escalation_conditions(&self, issue: &Issue, conditions: &EscalationConditions) -> bool {
        if let Some(priority_threshold) = conditions.priority_threshold {
            if issue.priority < priority_threshold {
                return false;
            }
        }

        if let Some(severity_threshold) = conditions.severity_threshold {
            if issue.severity < severity_threshold {
                return false;
            }
        }

        if let Some(days_threshold) = conditions.days_open_threshold {
            if issue.days_open() < days_threshold {
                return false;
            }
        }

        if let Some(impact_threshold) = conditions.business_impact_threshold {
            if issue.business_impact < impact_threshold {
                return false;
            }
        }

        if !conditions.categories.is_empty() && !conditions.categories.contains(&issue.category) {
            return false;
        }

        true
    }

    fn default_sla_definitions() -> Vec<IssueSlaDef> {
        vec![
            IssueSlaDef {
                id: Id::new(),
                name: "Critical Issues SLA".to_string(),
                priority: IssuePriority::Critical,
                response_time_hours: 2,
                resolution_time_hours: 24,
                business_hours_only: false,
            },
            IssueSlaDef {
                id: Id::new(),
                name: "High Priority Issues SLA".to_string(),
                priority: IssuePriority::High,
                response_time_hours: 8,
                resolution_time_hours: 72,
                business_hours_only: true,
            },
            IssueSlaDef {
                id: Id::new(),
                name: "Medium Priority Issues SLA".to_string(),
                priority: IssuePriority::Medium,
                response_time_hours: 24,
                resolution_time_hours: 168,
            business_hours_only: true,
            },
            IssueSlaDef {
                id: Id::new(),
                name: "Low Priority Issues SLA".to_string(),
                priority: IssuePriority::Low,
                response_time_hours: 72,
                resolution_time_hours: 336,
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

pub struct IssueRepository {
    issues: Vec<Issue>,
}

impl IssueRepository {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }
}

impl Repository<Issue> for IssueRepository {
    fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Issue>> {
        if !path.as_ref().exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let issues: Vec<Issue> = ron::from_str(&content)?;
        Ok(issues)
    }

    fn save_to_file<P: AsRef<std::path::Path>>(items: &[Issue], path: P) -> Result<()> {
        let content = format_ron_pretty(items)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn find_by_id(&self, id: Id) -> Option<&Issue> {
        self.issues.iter().find(|i| i.id == id)
    }

    fn find_by_name(&self, name: &str) -> Option<&Issue> {
        self.issues.iter().find(|i| i.title == name)
    }

    fn add(&mut self, item: Issue) -> Result<()> {
        item.validate()?;
        self.issues.push(item);
        Ok(())
    }

    fn update(&mut self, item: Issue) -> Result<()> {
        item.validate()?;
        if let Some(index) = self.issues.iter().position(|i| i.id == item.id) {
            self.issues[index] = item;
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Issue not found".to_string()))
        }
    }

    fn remove(&mut self, id: Id) -> Result<()> {
        if let Some(index) = self.issues.iter().position(|i| i.id == id) {
            self.issues.remove(index);
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Issue not found".to_string()))
        }
    }

    fn list(&self) -> &[Issue] {
        &self.issues
    }
}

// Display implementations
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