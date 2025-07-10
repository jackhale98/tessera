use tessera_core::{Id, Entity, Repository, Result, DesignTrackError, format_ron_pretty};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRisk {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub category: RiskCategory,
    pub probability: RiskProbability,
    pub impact: RiskImpact,
    pub risk_score: f32,
    pub status: RiskStatus,
    pub owner: Option<String>,
    pub identified_by: String,
    pub identified_date: DateTime<Utc>,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub target_resolution_date: Option<NaiveDate>,
    pub actual_resolution_date: Option<NaiveDate>,
    pub related_tasks: Vec<Id>,
    pub related_milestones: Vec<Id>,
    pub related_issues: Vec<Id>,
    pub mitigation_strategy: Option<String>,
    pub contingency_plan: Option<String>,
    pub mitigation_actions: Vec<MitigationAction>,
    pub risk_responses: Vec<RiskResponse>,
    pub comments: Vec<RiskComment>,
    pub cost_impact: Option<f32>,
    pub schedule_impact_days: Option<i32>,
    pub business_impact: BusinessImpact,
    pub residual_probability: Option<RiskProbability>,
    pub residual_impact: Option<RiskImpact>,
    pub tags: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationAction {
    pub id: Id,
    pub description: String,
    pub assigned_to: String,
    pub due_date: NaiveDate,
    pub status: ActionStatus,
    pub estimated_cost: Option<f32>,
    pub actual_cost: Option<f32>,
    pub completion_date: Option<NaiveDate>,
    pub effectiveness_rating: Option<EffectivenessRating>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResponse {
    pub id: Id,
    pub response_type: RiskResponseType,
    pub description: String,
    pub assigned_to: String,
    pub target_date: NaiveDate,
    pub status: ActionStatus,
    pub cost: Option<f32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComment {
    pub id: Id,
    pub author: String,
    pub content: String,
    pub created_date: DateTime<Utc>,
    pub comment_type: CommentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskCategory {
    Technical,
    Schedule,
    Cost,
    Resource,
    External,
    Organizational,
    Quality,
    Security,
    Environmental,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskProbability {
    VeryLow,    // 0-10%
    Low,        // 11-30%
    Medium,     // 31-50%
    High,       // 51-70%
    VeryHigh,   // 71-90%
    Certain,    // 91-100%
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskImpact {
    Negligible,
    Minor,
    Moderate,
    Major,
    Severe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskStatus {
    Identified,
    Analyzing,
    Planning,
    Mitigating,
    Monitoring,
    Realized,
    Closed,
    Transferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskResponseType {
    Avoid,
    Mitigate,
    Transfer,
    Accept,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionStatus {
    NotStarted,
    InProgress,
    Completed,
    OnHold,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectivenessRating {
    VeryEffective,
    Effective,
    SomewhatEffective,
    NotEffective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommentType {
    General,
    StatusUpdate,
    MitigationUpdate,
    Escalation,
    Resolution,
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
pub struct RiskRegistry {
    pub risks: HashMap<Id, ProjectRisk>,
    pub risk_matrix: RiskMatrix,
    pub risk_appetite: RiskAppetite,
    pub review_cycle_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMatrix {
    pub probability_thresholds: HashMap<RiskProbability, f32>,
    pub impact_thresholds: HashMap<RiskImpact, f32>,
    pub risk_levels: HashMap<String, RiskLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLevel {
    pub name: String,
    pub color: String,
    pub action_required: String,
    pub escalation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAppetite {
    pub overall_tolerance: RiskTolerance,
    pub category_tolerances: HashMap<RiskCategory, RiskTolerance>,
    pub maximum_acceptable_score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskTolerance {
    Averse,
    Cautious,
    Balanced,
    Seeking,
    Aggressive,
}

impl Entity for ProjectRisk {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &str {
        &self.title
    }

    fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(DesignTrackError::Validation("Risk title cannot be empty".to_string()));
        }
        if self.identified_by.trim().is_empty() {
            return Err(DesignTrackError::Validation("Risk must have an identified_by field".to_string()));
        }
        Ok(())
    }
}

impl ProjectRisk {
    pub fn new(title: String, description: String, identified_by: String) -> Self {
        let now = Utc::now();
        let mut risk = Self {
            id: Id::new(),
            title,
            description,
            category: RiskCategory::Technical,
            probability: RiskProbability::Medium,
            impact: RiskImpact::Moderate,
            risk_score: 0.0,
            status: RiskStatus::Identified,
            owner: None,
            identified_by,
            identified_date: now,
            last_reviewed: None,
            target_resolution_date: None,
            actual_resolution_date: None,
            related_tasks: Vec::new(),
            related_milestones: Vec::new(),
            related_issues: Vec::new(),
            mitigation_strategy: None,
            contingency_plan: None,
            mitigation_actions: Vec::new(),
            risk_responses: Vec::new(),
            comments: Vec::new(),
            cost_impact: None,
            schedule_impact_days: None,
            business_impact: BusinessImpact::Low,
            residual_probability: None,
            residual_impact: None,
            tags: Vec::new(),
            created: now,
            updated: now,
            metadata: HashMap::new(),
        };
        risk.calculate_risk_score();
        risk
    }

    pub fn with_category(mut self, category: RiskCategory) -> Self {
        self.category = category;
        self.updated = Utc::now();
        self
    }

    pub fn with_probability_and_impact(mut self, probability: RiskProbability, impact: RiskImpact) -> Self {
        self.probability = probability;
        self.impact = impact;
        self.calculate_risk_score();
        self.updated = Utc::now();
        self
    }

    pub fn assign_owner(&mut self, owner: String) {
        self.owner = Some(owner);
        if self.status == RiskStatus::Identified {
            self.status = RiskStatus::Analyzing;
        }
        self.updated = Utc::now();
    }

    pub fn calculate_risk_score(&mut self) {
        let prob_value = self.probability.numeric_value();
        let impact_value = self.impact.numeric_value();
        self.risk_score = prob_value * impact_value;
    }

    pub fn add_mitigation_action(&mut self, description: String, assigned_to: String, due_date: NaiveDate) -> Id {
        let action = MitigationAction {
            id: Id::new(),
            description,
            assigned_to,
            due_date,
            status: ActionStatus::NotStarted,
            estimated_cost: None,
            actual_cost: None,
            completion_date: None,
            effectiveness_rating: None,
            notes: None,
        };
        let action_id = action.id;
        self.mitigation_actions.push(action);
        
        if matches!(self.status, RiskStatus::Identified | RiskStatus::Analyzing) {
            self.status = RiskStatus::Planning;
        }
        
        self.updated = Utc::now();
        action_id
    }

    pub fn add_response(&mut self, response_type: RiskResponseType, description: String, assigned_to: String, target_date: NaiveDate) -> Id {
        let response = RiskResponse {
            id: Id::new(),
            response_type,
            description,
            assigned_to,
            target_date,
            status: ActionStatus::NotStarted,
            cost: None,
            notes: None,
        };
        let response_id = response.id;
        self.risk_responses.push(response);
        self.updated = Utc::now();
        response_id
    }

    pub fn add_comment(&mut self, author: String, content: String, comment_type: CommentType) -> Id {
        let comment = RiskComment {
            id: Id::new(),
            author,
            content,
            created_date: Utc::now(),
            comment_type,
        };
        let comment_id = comment.id;
        self.comments.push(comment);
        self.updated = Utc::now();
        comment_id
    }

    pub fn realize_risk(&mut self, issue_id: Id) {
        self.status = RiskStatus::Realized;
        self.actual_resolution_date = Some(Utc::now().date_naive());
        self.related_issues.push(issue_id);
        self.updated = Utc::now();
    }

    pub fn close_risk(&mut self, reason: String) {
        self.status = RiskStatus::Closed;
        self.actual_resolution_date = Some(Utc::now().date_naive());
        self.add_comment(
            "System".to_string(),
            format!("Risk closed: {}", reason),
            CommentType::Resolution,
        );
    }

    pub fn update_review(&mut self, reviewer: String) {
        self.last_reviewed = Some(Utc::now());
        self.add_comment(
            reviewer,
            "Risk reviewed".to_string(),
            CommentType::StatusUpdate,
        );
    }

    pub fn is_high_priority(&self) -> bool {
        self.risk_score >= 15.0 ||
        matches!(self.probability, RiskProbability::High | RiskProbability::VeryHigh | RiskProbability::Certain) &&
        matches!(self.impact, RiskImpact::Major | RiskImpact::Severe)
    }

    pub fn is_overdue_for_review(&self, review_cycle_days: u32) -> bool {
        if let Some(last_reviewed) = self.last_reviewed {
            let days_since_review = (Utc::now() - last_reviewed).num_days();
            days_since_review > review_cycle_days as i64
        } else {
            let days_since_identified = (Utc::now() - self.identified_date).num_days();
            days_since_identified > review_cycle_days as i64
        }
    }

    pub fn get_overdue_actions(&self) -> Vec<&MitigationAction> {
        let current_date = Utc::now().date_naive();
        self.mitigation_actions.iter()
            .filter(|action| {
                action.due_date < current_date && 
                !matches!(action.status, ActionStatus::Completed | ActionStatus::Cancelled)
            })
            .collect()
    }

    pub fn calculate_residual_risk(&mut self) {
        let completed_actions: Vec<_> = self.mitigation_actions.iter()
            .filter(|action| action.status == ActionStatus::Completed)
            .collect();

        if completed_actions.is_empty() {
            self.residual_probability = Some(self.probability);
            self.residual_impact = Some(self.impact);
            return;
        }

        let mut probability_reduction: f32 = 0.0;
        let mut impact_reduction: f32 = 0.0;
        
        for action in completed_actions {
            if let Some(effectiveness) = action.effectiveness_rating {
                let reduction_factor = match effectiveness {
                    EffectivenessRating::VeryEffective => 0.4,
                    EffectivenessRating::Effective => 0.3,
                    EffectivenessRating::SomewhatEffective => 0.2,
                    EffectivenessRating::NotEffective => 0.0,
                };
                probability_reduction += reduction_factor;
                impact_reduction += reduction_factor;
            }
        }

        probability_reduction = probability_reduction.min(0.7);
        impact_reduction = impact_reduction.min(0.5);

        let current_prob_value = self.probability.numeric_value();
        let current_impact_value = self.impact.numeric_value();
        
        let residual_prob_value = current_prob_value * (1.0 - probability_reduction);
        let residual_impact_value = current_impact_value * (1.0 - impact_reduction);

        self.residual_probability = Some(RiskProbability::from_numeric(residual_prob_value));
        self.residual_impact = Some(RiskImpact::from_numeric(residual_impact_value));
    }
}

impl RiskProbability {
    pub fn numeric_value(&self) -> f32 {
        match self {
            RiskProbability::VeryLow => 0.05,
            RiskProbability::Low => 0.20,
            RiskProbability::Medium => 0.40,
            RiskProbability::High => 0.60,
            RiskProbability::VeryHigh => 0.80,
            RiskProbability::Certain => 0.95,
        }
    }

    pub fn from_numeric(value: f32) -> Self {
        match value {
            v if v <= 0.10 => RiskProbability::VeryLow,
            v if v <= 0.30 => RiskProbability::Low,
            v if v <= 0.50 => RiskProbability::Medium,
            v if v <= 0.70 => RiskProbability::High,
            v if v <= 0.90 => RiskProbability::VeryHigh,
            _ => RiskProbability::Certain,
        }
    }
}

impl RiskImpact {
    pub fn numeric_value(&self) -> f32 {
        match self {
            RiskImpact::Negligible => 1.0,
            RiskImpact::Minor => 2.0,
            RiskImpact::Moderate => 3.0,
            RiskImpact::Major => 4.0,
            RiskImpact::Severe => 5.0,
        }
    }

    pub fn from_numeric(value: f32) -> Self {
        match value {
            v if v <= 1.5 => RiskImpact::Negligible,
            v if v <= 2.5 => RiskImpact::Minor,
            v if v <= 3.5 => RiskImpact::Moderate,
            v if v <= 4.5 => RiskImpact::Major,
            _ => RiskImpact::Severe,
        }
    }
}

impl Default for RiskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskRegistry {
    pub fn new() -> Self {
        Self {
            risks: HashMap::new(),
            risk_matrix: RiskMatrix::default(),
            risk_appetite: RiskAppetite::default(),
            review_cycle_days: 30,
        }
    }

    pub fn create_risk(&mut self, title: String, description: String, identified_by: String) -> Id {
        let mut risk = ProjectRisk::new(title, description, identified_by);
        let risk_id = risk.id;
        risk.calculate_risk_score();
        
        self.risks.insert(risk_id, risk);
        risk_id
    }

    pub fn get_risk(&self, risk_id: &Id) -> Option<&ProjectRisk> {
        self.risks.get(risk_id)
    }

    pub fn get_risk_mut(&mut self, risk_id: &Id) -> Option<&mut ProjectRisk> {
        self.risks.get_mut(risk_id)
    }

    pub fn get_all_risks(&self) -> Vec<&ProjectRisk> {
        self.risks.values().collect()
    }

    pub fn get_active_risks(&self) -> Vec<&ProjectRisk> {
        self.risks.values()
            .filter(|risk| !matches!(risk.status, RiskStatus::Closed | RiskStatus::Realized))
            .collect()
    }

    pub fn get_high_priority_risks(&self) -> Vec<&ProjectRisk> {
        self.risks.values()
            .filter(|risk| risk.is_high_priority())
            .collect()
    }

    pub fn get_risks_requiring_review(&self) -> Vec<&ProjectRisk> {
        self.risks.values()
            .filter(|risk| risk.is_overdue_for_review(self.review_cycle_days))
            .collect()
    }

    pub fn get_risks_by_category(&self, category: RiskCategory) -> Vec<&ProjectRisk> {
        self.risks.values()
            .filter(|risk| risk.category == category)
            .collect()
    }

    pub fn get_risks_by_owner(&self, owner: &str) -> Vec<&ProjectRisk> {
        self.risks.values()
            .filter(|risk| {
                if let Some(ref risk_owner) = risk.owner {
                    risk_owner == owner
                } else {
                    false
                }
            })
            .collect()
    }
}

impl Default for RiskMatrix {
    fn default() -> Self {
        let mut probability_thresholds = HashMap::new();
        probability_thresholds.insert(RiskProbability::VeryLow, 0.05);
        probability_thresholds.insert(RiskProbability::Low, 0.20);
        probability_thresholds.insert(RiskProbability::Medium, 0.40);
        probability_thresholds.insert(RiskProbability::High, 0.60);
        probability_thresholds.insert(RiskProbability::VeryHigh, 0.80);
        probability_thresholds.insert(RiskProbability::Certain, 0.95);

        let mut impact_thresholds = HashMap::new();
        impact_thresholds.insert(RiskImpact::Negligible, 1.0);
        impact_thresholds.insert(RiskImpact::Minor, 2.0);
        impact_thresholds.insert(RiskImpact::Moderate, 3.0);
        impact_thresholds.insert(RiskImpact::Major, 4.0);
        impact_thresholds.insert(RiskImpact::Severe, 5.0);

        let mut risk_levels = HashMap::new();
        risk_levels.insert("Low".to_string(), RiskLevel {
            name: "Low".to_string(),
            color: "Green".to_string(),
            action_required: "Monitor".to_string(),
            escalation_required: false,
        });
        risk_levels.insert("Medium".to_string(), RiskLevel {
            name: "Medium".to_string(),
            color: "Yellow".to_string(),
            action_required: "Mitigate".to_string(),
            escalation_required: false,
        });
        risk_levels.insert("High".to_string(), RiskLevel {
            name: "High".to_string(),
            color: "Orange".to_string(),
            action_required: "Immediate Action".to_string(),
            escalation_required: true,
        });
        risk_levels.insert("Critical".to_string(), RiskLevel {
            name: "Critical".to_string(),
            color: "Red".to_string(),
            action_required: "Emergency Response".to_string(),
            escalation_required: true,
        });

        Self {
            probability_thresholds,
            impact_thresholds,
            risk_levels,
        }
    }
}

impl Default for RiskAppetite {
    fn default() -> Self {
        Self {
            overall_tolerance: RiskTolerance::Balanced,
            category_tolerances: HashMap::new(),
            maximum_acceptable_score: 12.0,
        }
    }
}

pub struct ProjectRiskRepository {
    risks: Vec<ProjectRisk>,
}

impl ProjectRiskRepository {
    pub fn new() -> Self {
        Self { risks: Vec::new() }
    }

    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<ProjectRisk>> {
        if !path.as_ref().exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let risks: Vec<ProjectRisk> = ron::from_str(&content)?;
        Ok(risks)
    }

    pub fn save_to_file<P: AsRef<std::path::Path>>(risks: &[ProjectRisk], path: P) -> Result<()> {
        let content = format_ron_pretty(risks)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Repository<ProjectRisk> for ProjectRiskRepository {
    fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<ProjectRisk>> {
        Self::load_from_file(path)
    }

    fn save_to_file<P: AsRef<std::path::Path>>(items: &[ProjectRisk], path: P) -> Result<()> {
        Self::save_to_file(items, path)
    }

    fn find_by_id(&self, id: Id) -> Option<&ProjectRisk> {
        self.risks.iter().find(|r| r.id == id)
    }

    fn find_by_name(&self, name: &str) -> Option<&ProjectRisk> {
        self.risks.iter().find(|r| r.title == name)
    }

    fn add(&mut self, item: ProjectRisk) -> Result<()> {
        item.validate()?;
        self.risks.push(item);
        Ok(())
    }

    fn update(&mut self, item: ProjectRisk) -> Result<()> {
        item.validate()?;
        if let Some(index) = self.risks.iter().position(|r| r.id == item.id) {
            self.risks[index] = item;
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Risk not found".to_string()))
        }
    }

    fn remove(&mut self, id: Id) -> Result<()> {
        if let Some(index) = self.risks.iter().position(|r| r.id == id) {
            self.risks.remove(index);
            Ok(())
        } else {
            Err(DesignTrackError::NotFound("Risk not found".to_string()))
        }
    }

    fn list(&self) -> &[ProjectRisk] {
        &self.risks
    }
}

// Display implementations
impl std::fmt::Display for RiskProbability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskProbability::VeryLow => write!(f, "Very Low"),
            RiskProbability::Low => write!(f, "Low"),
            RiskProbability::Medium => write!(f, "Medium"),
            RiskProbability::High => write!(f, "High"),
            RiskProbability::VeryHigh => write!(f, "Very High"),
            RiskProbability::Certain => write!(f, "Certain"),
        }
    }
}

impl std::fmt::Display for RiskImpact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskImpact::Negligible => write!(f, "Negligible"),
            RiskImpact::Minor => write!(f, "Minor"),
            RiskImpact::Moderate => write!(f, "Moderate"),
            RiskImpact::Major => write!(f, "Major"),
            RiskImpact::Severe => write!(f, "Severe"),
        }
    }
}

impl std::fmt::Display for RiskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskStatus::Identified => write!(f, "Identified"),
            RiskStatus::Analyzing => write!(f, "Analyzing"),
            RiskStatus::Planning => write!(f, "Planning"),
            RiskStatus::Mitigating => write!(f, "Mitigating"),
            RiskStatus::Monitoring => write!(f, "Monitoring"),
            RiskStatus::Realized => write!(f, "Realized"),
            RiskStatus::Closed => write!(f, "Closed"),
            RiskStatus::Transferred => write!(f, "Transferred"),
        }
    }
}

impl std::fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskCategory::Technical => write!(f, "Technical"),
            RiskCategory::Schedule => write!(f, "Schedule"),
            RiskCategory::Cost => write!(f, "Cost"),
            RiskCategory::Resource => write!(f, "Resource"),
            RiskCategory::External => write!(f, "External"),
            RiskCategory::Organizational => write!(f, "Organizational"),
            RiskCategory::Quality => write!(f, "Quality"),
            RiskCategory::Security => write!(f, "Security"),
            RiskCategory::Environmental => write!(f, "Environmental"),
            RiskCategory::Market => write!(f, "Market"),
        }
    }
}

impl std::fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionStatus::NotStarted => write!(f, "Not Started"),
            ActionStatus::InProgress => write!(f, "In Progress"),
            ActionStatus::Completed => write!(f, "Completed"),
            ActionStatus::OnHold => write!(f, "On Hold"),
            ActionStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}