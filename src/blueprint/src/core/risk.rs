use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub risk_id: Uuid,
    pub title: String,
    pub description: String,
    pub category: RiskCategory,
    pub probability: RiskProbability,
    pub impact: RiskImpact,
    pub risk_score: f32, // Calculated: probability * impact
    pub status: RiskStatus,
    pub owner: Option<String>,
    pub identified_by: String,
    pub identified_date: NaiveDateTime,
    pub last_reviewed: Option<NaiveDateTime>,
    pub target_resolution_date: Option<NaiveDate>,
    pub actual_resolution_date: Option<NaiveDate>,
    pub related_tasks: Vec<String>,
    pub related_milestones: Vec<String>,
    pub related_issues: Vec<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationAction {
    pub action_id: Uuid,
    pub description: String,
    pub assigned_to: String,
    pub due_date: NaiveDate,
    pub status: ActionStatus,
    pub estimated_cost: Option<f32>,
    pub actual_cost: Option<f32>,
    pub completion_date: Option<NaiveDate>,
    pub effectiveness_rating: Option<EffectivenessRating>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResponse {
    pub response_id: Uuid,
    pub response_type: RiskResponseType,
    pub description: String,
    pub assigned_to: String,
    pub target_date: NaiveDate,
    pub status: ActionStatus,
    pub cost: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComment {
    pub comment_id: Uuid,
    pub author: String,
    pub content: String,
    pub created_date: NaiveDateTime,
    pub comment_type: CommentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Technical,      // Technical risks (e.g., technology failure, complexity)
    Schedule,       // Schedule-related risks
    Cost,           // Budget and cost risks
    Resource,       // Human resource and skill risks
    External,       // External dependencies, vendors, regulations
    Organizational, // Organizational and process risks
    Quality,        // Quality and performance risks
    Security,       // Security and compliance risks
    Environmental,  // Environmental and infrastructure risks
    Market,         // Market and business risks
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskProbability {
    VeryLow,    // 0-10% chance
    Low,        // 11-30% chance
    Medium,     // 31-50% chance
    High,       // 51-70% chance
    VeryHigh,   // 71-90% chance
    Certain,    // 91-100% chance
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskImpact {
    Negligible, // Minimal impact
    Minor,      // Small impact
    Moderate,   // Moderate impact
    Major,      // Significant impact
    Severe,     // Severe impact
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskStatus {
    Identified,     // Risk identified and recorded
    Analyzing,      // Risk being analyzed
    Planning,       // Mitigation plan being developed
    Mitigating,     // Mitigation actions in progress
    Monitoring,     // Risk being monitored
    Realized,       // Risk has occurred (became an issue)
    Closed,         // Risk no longer applicable
    Transferred,    // Risk transferred to another party
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskResponseType {
    Avoid,      // Eliminate the risk
    Mitigate,   // Reduce probability or impact
    Transfer,   // Transfer risk to another party
    Accept,     // Accept the risk
    Escalate,   // Escalate to higher authority
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    NotStarted,
    InProgress,
    Completed,
    OnHold,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectivenessRating {
    VeryEffective,
    Effective,
    SomewhatEffective,
    NotEffective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentType {
    General,
    StatusUpdate,
    MitigationUpdate,
    Escalation,
    Resolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessImpact {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRegistry {
    pub risks: HashMap<Uuid, Risk>,
    pub risk_matrix: RiskMatrix,
    pub risk_appetite: RiskAppetite,
    pub review_cycle_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMatrix {
    pub probability_thresholds: HashMap<RiskProbability, f32>,
    pub impact_thresholds: HashMap<RiskImpact, f32>,
    pub risk_levels: HashMap<String, RiskLevel>, // Risk score ranges to risk levels
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
#[serde(rename_all = "snake_case")]
pub enum RiskTolerance {
    Averse,     // Very low risk tolerance
    Cautious,   // Low risk tolerance
    Balanced,   // Moderate risk tolerance
    Seeking,    // High risk tolerance
    Aggressive, // Very high risk tolerance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub total_risks: u32,
    pub active_risks: u32,
    pub risks_by_status: HashMap<RiskStatus, u32>,
    pub risks_by_category: HashMap<RiskCategory, u32>,
    pub risks_by_level: HashMap<String, u32>, // High, Medium, Low risks
    pub average_risk_score: f32,
    pub highest_risk_score: f32,
    pub risks_requiring_action: u32,
    pub overdue_mitigation_actions: u32,
    pub risks_realized_this_period: u32,
    pub mitigation_effectiveness: f32, // Average effectiveness of completed actions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub assessment_id: Uuid,
    pub project_name: String,
    pub assessment_date: NaiveDate,
    pub assessor: String,
    pub risk_summary: RiskSummary,
    pub top_risks: Vec<Uuid>, // Risk IDs of top risks
    pub recommended_actions: Vec<String>,
    pub overall_risk_rating: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    pub total_identified: u32,
    pub high_probability_high_impact: u32,
    pub requiring_immediate_action: u32,
    pub effectively_mitigated: u32,
    pub newly_identified: u32,
    pub realized_risks: u32,
}

impl Risk {
    pub fn new(
        title: String,
        description: String,
        identified_by: String,
    ) -> Self {
        Self {
            risk_id: Uuid::new_v4(),
            title,
            description,
            category: RiskCategory::Technical,
            probability: RiskProbability::Medium,
            impact: RiskImpact::Moderate,
            risk_score: 0.0,
            status: RiskStatus::Identified,
            owner: None,
            identified_by,
            identified_date: chrono::Utc::now().naive_utc(),
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
        }
    }

    pub fn with_category(mut self, category: RiskCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_probability_and_impact(mut self, probability: RiskProbability, impact: RiskImpact) -> Self {
        self.probability = probability;
        self.impact = impact;
        self.calculate_risk_score();
        self
    }

    pub fn assign_owner(&mut self, owner: String) {
        self.owner = Some(owner);
        if self.status == RiskStatus::Identified {
            self.status = RiskStatus::Analyzing;
        }
    }

    pub fn calculate_risk_score(&mut self) {
        let prob_value = self.probability.numeric_value();
        let impact_value = self.impact.numeric_value();
        self.risk_score = prob_value * impact_value;
    }

    pub fn add_mitigation_action(&mut self, description: String, assigned_to: String, due_date: NaiveDate) -> Uuid {
        let action_id = Uuid::new_v4();
        let action = MitigationAction {
            action_id,
            description,
            assigned_to,
            due_date,
            status: ActionStatus::NotStarted,
            estimated_cost: None,
            actual_cost: None,
            completion_date: None,
            effectiveness_rating: None,
        };
        self.mitigation_actions.push(action);
        
        if matches!(self.status, RiskStatus::Identified | RiskStatus::Analyzing) {
            self.status = RiskStatus::Planning;
        }
        
        action_id
    }

    pub fn add_response(&mut self, response_type: RiskResponseType, description: String, assigned_to: String, target_date: NaiveDate) -> Uuid {
        let response_id = Uuid::new_v4();
        let response = RiskResponse {
            response_id,
            response_type,
            description,
            assigned_to,
            target_date,
            status: ActionStatus::NotStarted,
            cost: None,
        };
        self.risk_responses.push(response);
        response_id
    }

    pub fn add_comment(&mut self, author: String, content: String, comment_type: CommentType) -> Uuid {
        let comment_id = Uuid::new_v4();
        let comment = RiskComment {
            comment_id,
            author,
            content,
            created_date: chrono::Utc::now().naive_utc(),
            comment_type,
        };
        self.comments.push(comment);
        comment_id
    }

    pub fn realize_risk(&mut self, issue_id: Uuid) {
        self.status = RiskStatus::Realized;
        self.actual_resolution_date = Some(chrono::Utc::now().naive_utc().date());
        self.related_issues.push(issue_id.to_string());
    }

    pub fn close_risk(&mut self, reason: String) {
        self.status = RiskStatus::Closed;
        self.actual_resolution_date = Some(chrono::Utc::now().naive_utc().date());
        self.add_comment(
            "System".to_string(),
            format!("Risk closed: {}", reason),
            CommentType::Resolution,
        );
    }

    pub fn update_review(&mut self, reviewer: String) {
        self.last_reviewed = Some(chrono::Utc::now().naive_utc());
        self.add_comment(
            reviewer,
            "Risk reviewed".to_string(),
            CommentType::StatusUpdate,
        );
    }

    pub fn is_high_priority(&self) -> bool {
        self.risk_score >= 15.0 || // High score threshold
        matches!(self.probability, RiskProbability::High | RiskProbability::VeryHigh | RiskProbability::Certain) &&
        matches!(self.impact, RiskImpact::Major | RiskImpact::Severe)
    }

    pub fn is_overdue_for_review(&self, review_cycle_days: u32) -> bool {
        if let Some(last_reviewed) = self.last_reviewed {
            let days_since_review = (chrono::Utc::now().naive_utc() - last_reviewed).num_days();
            days_since_review > review_cycle_days as i64
        } else {
            let days_since_identified = (chrono::Utc::now().naive_utc() - self.identified_date).num_days();
            days_since_identified > review_cycle_days as i64
        }
    }

    pub fn get_overdue_actions(&self) -> Vec<&MitigationAction> {
        let current_date = chrono::Utc::now().naive_utc().date();
        self.mitigation_actions.iter()
            .filter(|action| {
                action.due_date < current_date && 
                !matches!(action.status, ActionStatus::Completed | ActionStatus::Cancelled)
            })
            .collect()
    }

    pub fn calculate_residual_risk(&mut self) {
        // Calculate residual risk based on mitigation effectiveness
        let completed_actions: Vec<_> = self.mitigation_actions.iter()
            .filter(|action| action.status == ActionStatus::Completed)
            .collect();

        if completed_actions.is_empty() {
            self.residual_probability = Some(self.probability);
            self.residual_impact = Some(self.impact);
            return;
        }

        // Simple reduction based on effectiveness ratings
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

        // Cap reductions at reasonable levels
        probability_reduction = probability_reduction.min(0.7);
        impact_reduction = impact_reduction.min(0.5);

        // Calculate residual values
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

impl RiskRegistry {
    pub fn new() -> Self {
        Self {
            risks: HashMap::new(),
            risk_matrix: RiskMatrix::default(),
            risk_appetite: RiskAppetite::default(),
            review_cycle_days: 30,
        }
    }

    pub fn create_risk(&mut self, title: String, description: String, identified_by: String) -> Uuid {
        let mut risk = Risk::new(title, description, identified_by);
        let risk_id = risk.risk_id;
        risk.calculate_risk_score();
        
        self.risks.insert(risk_id, risk);
        risk_id
    }

    pub fn get_risk(&self, risk_id: &Uuid) -> Option<&Risk> {
        self.risks.get(risk_id)
    }

    pub fn get_risk_mut(&mut self, risk_id: &Uuid) -> Option<&mut Risk> {
        self.risks.get_mut(risk_id)
    }

    pub fn get_active_risks(&self) -> Vec<&Risk> {
        self.risks.values()
            .filter(|risk| !matches!(risk.status, RiskStatus::Closed | RiskStatus::Realized))
            .collect()
    }

    pub fn get_high_priority_risks(&self) -> Vec<&Risk> {
        self.risks.values()
            .filter(|risk| risk.is_high_priority())
            .collect()
    }

    pub fn get_risks_requiring_review(&self) -> Vec<&Risk> {
        self.risks.values()
            .filter(|risk| risk.is_overdue_for_review(self.review_cycle_days))
            .collect()
    }

    pub fn get_risks_by_category(&self, category: RiskCategory) -> Vec<&Risk> {
        self.risks.values()
            .filter(|risk| risk.category == category)
            .collect()
    }

    pub fn get_risks_by_owner(&self, owner: &str) -> Vec<&Risk> {
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

    pub fn calculate_metrics(&self) -> RiskMetrics {
        let total_risks = self.risks.len() as u32;
        let active_risks = self.get_active_risks().len() as u32;

        let mut risks_by_status = HashMap::new();
        let mut risks_by_category = HashMap::new();
        let mut risks_by_level = HashMap::new();

        let mut risk_scores: Vec<f32> = Vec::new();
        let mut risks_requiring_action = 0;
        let mut overdue_actions = 0;
        let mut realized_risks = 0;

        for risk in self.risks.values() {
            *risks_by_status.entry(risk.status).or_insert(0) += 1;
            *risks_by_category.entry(risk.category).or_insert(0) += 1;

            risk_scores.push(risk.risk_score);

            if risk.is_high_priority() {
                risks_requiring_action += 1;
            }

            overdue_actions += risk.get_overdue_actions().len() as u32;

            if risk.status == RiskStatus::Realized {
                realized_risks += 1;
            }

            // Categorize by risk level
            let level = self.get_risk_level(risk.risk_score);
            *risks_by_level.entry(level).or_insert(0) += 1;
        }

        let average_risk_score = if !risk_scores.is_empty() {
            risk_scores.iter().sum::<f32>() / risk_scores.len() as f32
        } else {
            0.0
        };

        let highest_risk_score = risk_scores.iter().fold(0.0f32, |acc, &x| acc.max(x));

        // Calculate mitigation effectiveness
        let completed_actions: Vec<_> = self.risks.values()
            .flat_map(|risk| &risk.mitigation_actions)
            .filter(|action| action.status == ActionStatus::Completed)
            .collect();

        let mitigation_effectiveness = if !completed_actions.is_empty() {
            let effectiveness_scores: Vec<f32> = completed_actions.iter()
                .filter_map(|action| action.effectiveness_rating)
                .map(|rating| match rating {
                    EffectivenessRating::VeryEffective => 4.0,
                    EffectivenessRating::Effective => 3.0,
                    EffectivenessRating::SomewhatEffective => 2.0,
                    EffectivenessRating::NotEffective => 1.0,
                })
                .collect();

            if !effectiveness_scores.is_empty() {
                (effectiveness_scores.iter().sum::<f32>() / effectiveness_scores.len() as f32) * 25.0 // Convert to percentage
            } else {
                0.0
            }
        } else {
            0.0
        };

        RiskMetrics {
            total_risks,
            active_risks,
            risks_by_status,
            risks_by_category,
            risks_by_level,
            average_risk_score,
            highest_risk_score,
            risks_requiring_action,
            overdue_mitigation_actions: overdue_actions,
            risks_realized_this_period: realized_risks,
            mitigation_effectiveness,
        }
    }

    fn get_risk_level(&self, risk_score: f32) -> String {
        if risk_score >= 20.0 {
            "Critical".to_string()
        } else if risk_score >= 15.0 {
            "High".to_string()
        } else if risk_score >= 8.0 {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }
}

impl Default for RiskRegistry {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_risk_creation() {
        let risk = Risk::new(
            "Test Risk".to_string(),
            "Test description".to_string(),
            "risk_manager@example.com".to_string(),
        );

        assert!(!risk.risk_id.to_string().is_empty());
        assert_eq!(risk.title, "Test Risk");
        assert_eq!(risk.status, RiskStatus::Identified);
        assert_eq!(risk.probability, RiskProbability::Medium);
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut risk = Risk::new(
            "Test Risk".to_string(),
            "Test description".to_string(),
            "risk_manager@example.com".to_string(),
        );

        risk = risk.with_probability_and_impact(RiskProbability::High, RiskImpact::Major);
        
        assert_eq!(risk.probability, RiskProbability::High);
        assert_eq!(risk.impact, RiskImpact::Major);
        assert_eq!(risk.risk_score, 2.4); // 0.6 * 4.0
    }

    #[test]
    fn test_risk_mitigation_action() {
        let mut risk = Risk::new(
            "Test Risk".to_string(),
            "Test description".to_string(),
            "risk_manager@example.com".to_string(),
        );

        let action_id = risk.add_mitigation_action(
            "Implement backup system".to_string(),
            "developer@example.com".to_string(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        assert!(!action_id.to_string().is_empty());
        assert_eq!(risk.mitigation_actions.len(), 1);
        assert_eq!(risk.status, RiskStatus::Planning);
    }

    #[test]
    fn test_risk_registry() {
        let mut registry = RiskRegistry::new();
        
        let risk_id = registry.create_risk(
            "Test Risk".to_string(),
            "Test description".to_string(),
            "risk_manager@example.com".to_string(),
        );

        assert!(!risk_id.to_string().is_empty());
        assert!(registry.get_risk(&risk_id).is_some());
        assert_eq!(registry.get_active_risks().len(), 1);

        let metrics = registry.calculate_metrics();
        assert_eq!(metrics.total_risks, 1);
        assert_eq!(metrics.active_risks, 1);
    }

    #[test]
    fn test_high_priority_risk_identification() {
        let mut risk = Risk::new(
            "Critical Risk".to_string(),
            "Test description".to_string(),
            "risk_manager@example.com".to_string(),
        );

        risk = risk.with_probability_and_impact(RiskProbability::High, RiskImpact::Severe);
        
        assert!(risk.is_high_priority());
        assert_eq!(risk.risk_score, 3.0); // 0.6 * 5.0
    }
}