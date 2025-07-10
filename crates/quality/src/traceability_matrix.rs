use crate::{Requirement, DesignInput, DesignOutput, DesignControl, Risk, QualityRepository};
use tessera_core::{Id, Result, DesignTrackError};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use colored::Colorize;

/// Traceability matrix for quality management
pub struct TraceabilityMatrix {
    requirements: Vec<Id>,
    inputs: Vec<Id>,
    outputs: Vec<Id>,
    controls: Vec<Id>,
    risks: Vec<Id>,
    links: HashMap<(Id, Id), TraceabilityLink>,
}

/// Relationship types in the traceability matrix
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TraceabilityRelation {
    Implements,    // Input/Output implements Requirement
    Controls,      // Control validates Output/Input  
    Mitigates,     // Control mitigates Risk
    References,    // General reference relationship
    Verifies,      // Output verifies Input
    Satisfies,     // Output satisfies Requirement
    Traces,        // General traceability link
}

/// Link in the traceability matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityLink {
    pub source_id: Id,
    pub target_id: Id,
    pub relation: TraceabilityRelation,
    pub confidence: f32, // 0.0 - 1.0
    pub notes: String,
}

/// Matrix analysis results
#[derive(Debug)]
pub struct TraceabilityAnalysis {
    pub total_requirements: usize,
    pub traced_requirements: usize,
    pub orphaned_requirements: Vec<Id>,
    pub unverified_outputs: Vec<Id>,
    pub uncontrolled_risks: Vec<Id>,
    pub coverage_percentage: f32,
    pub completeness_score: f32,
    pub gaps: Vec<TraceabilityGap>,
}

/// Identified gap in traceability
#[derive(Debug)]
pub struct TraceabilityGap {
    pub gap_type: GapType,
    pub entity_id: Id,
    pub entity_name: String,
    pub severity: GapSeverity,
    pub recommendation: String,
}

#[derive(Debug)]
pub enum GapType {
    UntraceableRequirement,
    UnverifiedOutput,
    UncontrolledRisk,
    MissingImplementation,
    WeakTraceability,
}

#[derive(Debug)]
pub enum GapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl TraceabilityMatrix {
    /// Create new traceability matrix from repository
    pub fn from_repository(repository: &QualityRepository) -> Self {
        let requirements: Vec<Id> = repository.get_requirements().iter().map(|r| r.id).collect();
        let inputs: Vec<Id> = repository.get_inputs().iter().map(|i| i.id).collect();
        let outputs: Vec<Id> = repository.get_outputs().iter().map(|o| o.id).collect();
        let controls: Vec<Id> = repository.get_controls().iter().map(|c| c.id).collect();
        let risks: Vec<Id> = repository.get_risks().iter().map(|r| r.id).collect();

        let mut matrix = Self {
            requirements,
            inputs,
            outputs,
            controls,
            risks,
            links: HashMap::new(),
        };

        // Auto-discover existing links from the repository
        matrix.discover_existing_links(repository);
        matrix
    }

    /// Discover existing links from repository data
    fn discover_existing_links(&mut self, repository: &QualityRepository) {
        // Links from requirements to other entities via the links field
        for requirement in repository.get_requirements() {
            for link in &requirement.links {
                self.add_link(TraceabilityLink {
                    source_id: requirement.id,
                    target_id: link.target_id,
                    relation: TraceabilityRelation::References,
                    confidence: 0.8,
                    notes: link.relation_type.clone(),
                });
            }
        }

        // Links from inputs to requirements
        for input in repository.get_inputs() {
            for &req_id in &input.requirements {
                self.add_link(TraceabilityLink {
                    source_id: req_id,
                    target_id: input.id,
                    relation: TraceabilityRelation::Implements,
                    confidence: 0.9,
                    notes: "Auto-discovered from input requirements".to_string(),
                });
            }
        }

        // Links from outputs to requirements and inputs
        for output in repository.get_outputs() {
            for &req_id in &output.requirements {
                self.add_link(TraceabilityLink {
                    source_id: req_id,
                    target_id: output.id,
                    relation: TraceabilityRelation::Satisfies,
                    confidence: 0.9,
                    notes: "Auto-discovered from output requirements".to_string(),
                });
            }

            for &input_id in &output.inputs {
                self.add_link(TraceabilityLink {
                    source_id: input_id,
                    target_id: output.id,
                    relation: TraceabilityRelation::Verifies,
                    confidence: 0.8,
                    notes: "Auto-discovered from output inputs".to_string(),
                });
            }
        }

        // Links from controls to outputs
        for control in repository.get_controls() {
            for &output_id in &control.outputs {
                self.add_link(TraceabilityLink {
                    source_id: output_id,
                    target_id: control.id,
                    relation: TraceabilityRelation::Controls,
                    confidence: 0.9,
                    notes: "Auto-discovered from control outputs".to_string(),
                });
            }
        }
    }

    /// Add a traceability link
    pub fn add_link(&mut self, link: TraceabilityLink) {
        let key = (link.source_id, link.target_id);
        self.links.insert(key, link);
    }

    /// Remove a traceability link
    pub fn remove_link(&mut self, source_id: Id, target_id: Id) {
        let key = (source_id, target_id);
        self.links.remove(&key);
    }

    /// Get all links for a specific entity
    pub fn get_entity_links(&self, entity_id: Id) -> Vec<&TraceabilityLink> {
        self.links.values()
            .filter(|link| link.source_id == entity_id || link.target_id == entity_id)
            .collect()
    }

    /// Analyze traceability completeness
    pub fn analyze_traceability(&self, repository: &QualityRepository) -> Result<TraceabilityAnalysis> {
        let mut orphaned_requirements = Vec::new();
        let mut unverified_outputs = Vec::new();
        let mut uncontrolled_risks = Vec::new();
        let mut gaps = Vec::new();

        // Check requirement traceability
        let mut traced_requirements = 0;
        for requirement in repository.get_requirements() {
            let has_forward_trace = self.links.iter()
                .any(|((source, _), _)| *source == requirement.id);
            
            if has_forward_trace {
                traced_requirements += 1;
            } else {
                orphaned_requirements.push(requirement.id);
                gaps.push(TraceabilityGap {
                    gap_type: GapType::UntraceableRequirement,
                    entity_id: requirement.id,
                    entity_name: requirement.name.clone(),
                    severity: GapSeverity::High,
                    recommendation: "Add implementation or verification links".to_string(),
                });
            }
        }

        // Check output verification
        for output in repository.get_outputs() {
            let has_verification = self.links.iter()
                .any(|((_, target), link)| *target == output.id && 
                     matches!(link.relation, TraceabilityRelation::Controls | TraceabilityRelation::Verifies));
            
            if !has_verification {
                unverified_outputs.push(output.id);
                gaps.push(TraceabilityGap {
                    gap_type: GapType::UnverifiedOutput,
                    entity_id: output.id,
                    entity_name: output.name.clone(),
                    severity: GapSeverity::Medium,
                    recommendation: "Add verification controls or tests".to_string(),
                });
            }
        }

        // Check risk controls
        for risk in repository.get_risks() {
            let has_control = self.links.iter()
                .any(|((_, target), link)| *target == risk.id && 
                     matches!(link.relation, TraceabilityRelation::Mitigates));
            
            if !has_control {
                uncontrolled_risks.push(risk.id);
                gaps.push(TraceabilityGap {
                    gap_type: GapType::UncontrolledRisk,
                    entity_id: risk.id,
                    entity_name: risk.name.clone(),
                    severity: if risk.risk_score > 0.7 { GapSeverity::Critical } else { GapSeverity::Medium },
                    recommendation: "Add mitigation controls".to_string(),
                });
            }
        }

        let total_requirements = self.requirements.len();
        let coverage_percentage = if total_requirements > 0 {
            (traced_requirements as f32 / total_requirements as f32) * 100.0
        } else {
            0.0
        };

        // Calculate overall completeness score
        let total_entities = total_requirements + self.outputs.len() + self.risks.len();
        let total_gaps = gaps.len();
        let completeness_score = if total_entities > 0 {
            ((total_entities - total_gaps) as f32 / total_entities as f32) * 100.0
        } else {
            100.0
        };

        Ok(TraceabilityAnalysis {
            total_requirements,
            traced_requirements,
            orphaned_requirements,
            unverified_outputs,
            uncontrolled_risks,
            coverage_percentage,
            completeness_score,
            gaps,
        })
    }

    /// Generate ASCII traceability matrix
    pub fn generate_ascii_matrix(&self, repository: &QualityRepository) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", "=== Quality Traceability Matrix ===".bold().cyan()));
        output.push_str(&format!("Links: {}\n\n", self.links.len()));

        // Requirements section
        if !self.requirements.is_empty() {
            output.push_str(&format!("{}\n", "Requirements Traceability:".bold().yellow()));
            for &req_id in &self.requirements {
                if let Some(req) = repository.get_requirements().iter().find(|r| r.id == req_id) {
                    let links = self.get_entity_links(req_id);
                    let status = if links.is_empty() { "❌ Orphaned" } else { "✅ Traced" };
                    output.push_str(&format!("  {} - {} ({})\n", req_id, req.name, status));
                    
                    for link in links {
                        if link.source_id == req_id {
                            output.push_str(&format!("    → {} {:?}\n", link.target_id, link.relation));
                        }
                    }
                }
            }
            output.push_str("\n");
        }

        // Matrix summary
        output.push_str(&format!("{}\n", "Matrix Summary:".bold().green()));
        output.push_str(&format!("  Requirements: {}\n", self.requirements.len()));
        output.push_str(&format!("  Inputs: {}\n", self.inputs.len()));
        output.push_str(&format!("  Outputs: {}\n", self.outputs.len()));
        output.push_str(&format!("  Controls: {}\n", self.controls.len()));
        output.push_str(&format!("  Risks: {}\n", self.risks.len()));
        output.push_str(&format!("  Total Links: {}\n", self.links.len()));

        output
    }

    /// Generate detailed traceability report
    pub fn generate_detailed_report(&self, repository: &QualityRepository) -> Result<String> {
        let analysis = self.analyze_traceability(repository)?;
        let mut output = String::new();

        output.push_str(&format!("{}\n", "=== Detailed Traceability Report ===".bold().cyan()));
        output.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")));

        // Executive Summary
        output.push_str(&format!("{}\n", "Executive Summary:".bold().green()));
        output.push_str(&format!("  Overall Completeness: {:.1}%\n", analysis.completeness_score));
        output.push_str(&format!("  Requirement Coverage: {:.1}%\n", analysis.coverage_percentage));
        output.push_str(&format!("  Total Gaps Identified: {}\n", analysis.gaps.len()));
        output.push_str("\n");

        // Gap Analysis
        if !analysis.gaps.is_empty() {
            output.push_str(&format!("{}\n", "Gap Analysis:".bold().red()));
            
            let mut critical_gaps = 0;
            let mut high_gaps = 0;
            let mut medium_gaps = 0;
            let mut low_gaps = 0;

            for gap in &analysis.gaps {
                match gap.severity {
                    GapSeverity::Critical => critical_gaps += 1,
                    GapSeverity::High => high_gaps += 1,
                    GapSeverity::Medium => medium_gaps += 1,
                    GapSeverity::Low => low_gaps += 1,
                }
            }

            output.push_str(&format!("  🔴 Critical: {}\n", critical_gaps));
            output.push_str(&format!("  🟠 High: {}\n", high_gaps));
            output.push_str(&format!("  🟡 Medium: {}\n", medium_gaps));
            output.push_str(&format!("  🟢 Low: {}\n", low_gaps));
            output.push_str("\n");

            // Detailed gap listing
            output.push_str(&format!("{}\n", "Detailed Gaps:".bold()));
            for gap in &analysis.gaps {
                let severity_icon = match gap.severity {
                    GapSeverity::Critical => "🔴",
                    GapSeverity::High => "🟠",
                    GapSeverity::Medium => "🟡", 
                    GapSeverity::Low => "🟢",
                };
                output.push_str(&format!("  {} {} - {} ({:?})\n", 
                    severity_icon, gap.entity_id, gap.entity_name, gap.gap_type));
                output.push_str(&format!("    💡 {}\n", gap.recommendation));
            }
            output.push_str("\n");
        }

        // Relationship Summary
        output.push_str(&format!("{}\n", "Relationship Summary:".bold().blue()));
        let mut relation_counts: HashMap<TraceabilityRelation, usize> = HashMap::new();
        for link in self.links.values() {
            *relation_counts.entry(link.relation.clone()).or_insert(0) += 1;
        }

        for (relation, count) in relation_counts {
            output.push_str(&format!("  {:?}: {}\n", relation, count));
        }

        Ok(output)
    }

    /// Export matrix data for external tools
    pub fn export_csv(&self, repository: &QualityRepository) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("Source ID,Source Name,Source Type,Target ID,Target Name,Target Type,Relation,Confidence,Notes\n");

        for link in self.links.values() {
            let source_info = self.get_entity_info(link.source_id, repository);
            let target_info = self.get_entity_info(link.target_id, repository);

            csv.push_str(&format!("{},{},{},{},{},{},{:?},{:.2},\"{}\"\n",
                link.source_id,
                source_info.0,
                source_info.1,
                link.target_id,
                target_info.0,
                target_info.1,
                link.relation,
                link.confidence,
                link.notes.replace("\"", "\"\"") // Escape quotes for CSV
            ));
        }

        Ok(csv)
    }

    /// Get entity information for display
    fn get_entity_info(&self, entity_id: Id, repository: &QualityRepository) -> (String, String) {
        // Check requirements
        if let Some(req) = repository.get_requirements().iter().find(|r| r.id == entity_id) {
            return (req.name.clone(), "Requirement".to_string());
        }

        // Check inputs
        if let Some(input) = repository.get_inputs().iter().find(|i| i.id == entity_id) {
            return (input.name.clone(), "Input".to_string());
        }

        // Check outputs
        if let Some(output) = repository.get_outputs().iter().find(|o| o.id == entity_id) {
            return (output.name.clone(), "Output".to_string());
        }

        // Check controls
        if let Some(control) = repository.get_controls().iter().find(|c| c.id == entity_id) {
            return (control.name.clone(), "Control".to_string());
        }

        // Check risks
        if let Some(risk) = repository.get_risks().iter().find(|r| r.id == entity_id) {
            return (risk.name.clone(), "Risk".to_string());
        }

        (format!("Unknown({})", entity_id), "Unknown".to_string())
    }

    /// Find potential links that could be created
    pub fn suggest_missing_links(&self, repository: &QualityRepository) -> Vec<SuggestedLink> {
        let mut suggestions = Vec::new();

        // Suggest links between requirements and unlinked outputs
        for requirement in repository.get_requirements() {
            for output in repository.get_outputs() {
                // Check if already linked
                let already_linked = self.links.iter()
                    .any(|((source, target), _)| 
                         (*source == requirement.id && *target == output.id) ||
                         (*source == output.id && *target == requirement.id));

                if !already_linked {
                    // Simple text similarity check
                    let similarity = calculate_text_similarity(&requirement.name, &output.name);
                    if similarity > 0.3 {
                        suggestions.push(SuggestedLink {
                            source_id: requirement.id,
                            target_id: output.id,
                            suggested_relation: TraceabilityRelation::Satisfies,
                            confidence: similarity,
                            reason: format!("Name similarity: {:.1}%", similarity * 100.0),
                        });
                    }
                }
            }
        }

        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(10); // Limit to top 10 suggestions

        suggestions
    }
}

/// Suggested link for improving traceability
#[derive(Debug)]
pub struct SuggestedLink {
    pub source_id: Id,
    pub target_id: Id,
    pub suggested_relation: TraceabilityRelation,
    pub confidence: f32,
    pub reason: String,
}

/// Simple text similarity calculation
fn calculate_text_similarity(text1: &str, text2: &str) -> f32 {
    let words1: HashSet<&str> = text1.to_lowercase().split_whitespace().collect();
    let words2: HashSet<&str> = text2.to_lowercase().split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}