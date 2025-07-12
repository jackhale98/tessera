use crate::{
    EntityReference, ChangeImpact, ImpactAnalysis, ImpactSeverity, ImpactType, ModuleType
};
use tessera_core::{Id, Result, ProjectContext, Entity};
use std::collections::{HashMap, HashSet, VecDeque};
use petgraph::{Graph, Directed};
use petgraph::visit::EdgeRef;
use chrono::Utc;

/// Core impact analysis engine that traverses cross-module links
pub struct ImpactAnalyzer {
    /// Graph representing entity relationships across modules
    entity_graph: Graph<EntityReference, ImpactType, Directed>,
    /// Cache of entity states for quick lookup
    entity_cache: HashMap<Id, EntityReference>,
    /// Configuration for impact severity calculations
    severity_rules: SeverityRules,
}

/// Rules for calculating impact severity
#[derive(Debug, Clone)]
pub struct SeverityRules {
    /// Base severity for different impact types
    pub impact_type_severity: HashMap<ImpactType, ImpactSeverity>,
    /// Severity multipliers based on entity types
    pub entity_type_multipliers: HashMap<String, f32>,
    /// Depth-based severity degradation
    pub depth_degradation: f32,
}

impl Default for SeverityRules {
    fn default() -> Self {
        let mut impact_type_severity = HashMap::new();
        impact_type_severity.insert(ImpactType::DirectLink, ImpactSeverity::Medium);
        impact_type_severity.insert(ImpactType::IndirectLink, ImpactSeverity::Low);
        impact_type_severity.insert(ImpactType::StateChange, ImpactSeverity::High);
        impact_type_severity.insert(ImpactType::RequirementChange, ImpactSeverity::High);
        impact_type_severity.insert(ImpactType::RiskChange, ImpactSeverity::Medium);
        impact_type_severity.insert(ImpactType::VerificationChange, ImpactSeverity::Medium);

        let mut entity_type_multipliers = HashMap::new();
        entity_type_multipliers.insert("Requirement".to_string(), 1.5);
        entity_type_multipliers.insert("Risk".to_string(), 1.3);
        entity_type_multipliers.insert("DesignInput".to_string(), 1.4);
        entity_type_multipliers.insert("DesignOutput".to_string(), 1.2);
        entity_type_multipliers.insert("Verification".to_string(), 1.1);

        Self {
            impact_type_severity,
            entity_type_multipliers,
            depth_degradation: 0.8, // 20% reduction per depth level
        }
    }
}

impl ImpactAnalyzer {
    pub fn new() -> Self {
        Self {
            entity_graph: Graph::new(),
            entity_cache: HashMap::new(),
            severity_rules: SeverityRules::default(),
        }
    }

    /// Load entity relationships from all modules into the analysis graph
    pub async fn load_cross_module_relationships(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        // Clear existing data
        self.entity_graph.clear();
        self.entity_cache.clear();

        // Load entities from each module
        self.load_requirements_entities(project_ctx).await?;
        self.load_risk_entities(project_ctx).await?;
        self.load_verification_entities(project_ctx).await?;
        self.load_pm_entities(project_ctx).await?;
        self.load_tol_entities(project_ctx).await?;

        // Build cross-module links
        self.build_cross_module_links(project_ctx).await?;

        Ok(())
    }

    /// Perform comprehensive impact analysis for a changed entity
    pub async fn analyze_impact(
        &self,
        source_entity: EntityReference,
        change_description: String,
        project_ctx: &ProjectContext
    ) -> Result<ImpactAnalysis> {
        let mut analysis = ImpactAnalysis::new(source_entity.clone(), change_description);

        // Find all entities impacted by this change
        let impacted_entities = self.find_impacted_entities(&source_entity)?;

        // Calculate specific impacts for each affected entity
        for (entity_ref, impact_info) in impacted_entities {
            let impact = self.calculate_entity_impact(
                &source_entity,
                &entity_ref,
                &impact_info
            ).await?;
            
            analysis.add_impact(impact);
        }

        Ok(analysis)
    }

    /// Find all entities that would be impacted by changes to the source entity
    fn find_impacted_entities(
        &self,
        source_entity: &EntityReference
    ) -> Result<Vec<(EntityReference, ImpactInfo)>> {
        let mut impacted = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Find source node in graph
        let source_node = self.find_node_by_entity(source_entity)?;
        
        // BFS to find all connected entities
        queue.push_back((source_node, 0, ImpactType::DirectLink));
        visited.insert(source_node);

        while let Some((current_node, depth, impact_type)) = queue.pop_front() {
            // Skip if we've gone too deep
            if depth > 5 {
                continue;
            }

            let current_entity = &self.entity_graph[current_node];

            // Add to impacted list (skip source entity itself)
            if depth > 0 {
                impacted.push((
                    current_entity.clone(),
                    ImpactInfo {
                        depth,
                        impact_type: impact_type.clone(),
                        connection_strength: self.calculate_connection_strength(depth),
                    }
                ));
            }

            // Find connected entities
            for edge in self.entity_graph.edges(current_node) {
                let target_node = edge.target();
                if !visited.contains(&target_node) {
                    visited.insert(target_node);
                    let next_impact_type = if depth == 0 {
                        edge.weight().clone()
                    } else {
                        ImpactType::IndirectLink
                    };
                    queue.push_back((target_node, depth + 1, next_impact_type));
                }
            }
        }

        Ok(impacted)
    }

    /// Calculate specific impact details for an entity
    async fn calculate_entity_impact(
        &self,
        source: &EntityReference,
        target: &EntityReference,
        impact_info: &ImpactInfo
    ) -> Result<ChangeImpact> {
        // Calculate severity based on rules
        let base_severity = self.severity_rules.impact_type_severity
            .get(&impact_info.impact_type)
            .copied()
            .unwrap_or(ImpactSeverity::Low);

        let entity_multiplier = self.severity_rules.entity_type_multipliers
            .get(&target.entity_type)
            .copied()
            .unwrap_or(1.0);

        let depth_factor = self.severity_rules.depth_degradation.powi(impact_info.depth as i32);
        
        let final_severity = self.adjust_severity(
            base_severity,
            entity_multiplier * depth_factor * impact_info.connection_strength
        );

        // Estimate effort based on severity and entity type
        let estimated_effort = self.estimate_effort_hours(&final_severity, &target.entity_type);

        Ok(ChangeImpact {
            id: Id::new(),
            target_entity: target.clone(),
            impact_type: impact_info.impact_type.clone(),
            severity: final_severity,
            description: self.generate_impact_description(source, target, &impact_info.impact_type),
            affected_attributes: self.identify_affected_attributes(target, &impact_info.impact_type),
            propagation_depth: impact_info.depth,
            estimated_effort_hours: Some(estimated_effort),
            created: Utc::now(),
        })
    }

    // Helper methods for loading entities from each module
    async fn load_requirements_entities(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        // TODO: Load from tessera-requirements
        // For now, placeholder implementation
        Ok(())
    }

    async fn load_risk_entities(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        // TODO: Load from tessera-risk
        Ok(())
    }

    async fn load_verification_entities(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        // TODO: Load from tessera-verification
        Ok(())
    }

    async fn load_pm_entities(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        // TODO: Load from tessera-pm
        Ok(())
    }

    async fn load_tol_entities(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        // TODO: Load from tessera-tol
        Ok(())
    }

    async fn build_cross_module_links(&mut self, project_ctx: &ProjectContext) -> Result<()> {
        // TODO: Build links using tessera-core linking system
        Ok(())
    }

    // Helper methods
    fn find_node_by_entity(&self, entity: &EntityReference) -> Result<petgraph::graph::NodeIndex> {
        // TODO: Implement node lookup
        Err(tessera_core::DesignTrackError::Validation("Node not found".to_string()))
    }

    fn calculate_connection_strength(&self, depth: u32) -> f32 {
        // Closer connections have higher strength
        1.0 / (depth as f32 + 1.0)
    }

    fn adjust_severity(&self, base: ImpactSeverity, multiplier: f32) -> ImpactSeverity {
        let severity_value = match base {
            ImpactSeverity::Low => 1.0,
            ImpactSeverity::Medium => 2.0,
            ImpactSeverity::High => 3.0,
            ImpactSeverity::Critical => 4.0,
        };

        let adjusted = severity_value * multiplier;
        
        match adjusted {
            x if x >= 3.5 => ImpactSeverity::Critical,
            x if x >= 2.5 => ImpactSeverity::High,
            x if x >= 1.5 => ImpactSeverity::Medium,
            _ => ImpactSeverity::Low,
        }
    }

    fn estimate_effort_hours(&self, severity: &ImpactSeverity, entity_type: &str) -> f64 {
        let base_hours = match severity {
            ImpactSeverity::Low => 0.5,
            ImpactSeverity::Medium => 2.0,
            ImpactSeverity::High => 8.0,
            ImpactSeverity::Critical => 24.0,
        };

        // Adjust based on entity type complexity
        let complexity_multiplier = match entity_type {
            "Requirement" => 1.5,
            "Risk" => 1.3,
            "DesignInput" => 1.4,
            "Verification" => 1.2,
            _ => 1.0,
        };

        base_hours * complexity_multiplier
    }

    fn generate_impact_description(
        &self,
        source: &EntityReference,
        target: &EntityReference,
        impact_type: &ImpactType
    ) -> String {
        match impact_type {
            ImpactType::DirectLink => {
                format!("Changes to {} {} directly affect {} {}", 
                    source.module, source.name, target.module, target.name)
            },
            ImpactType::IndirectLink => {
                format!("Changes to {} {} indirectly impact {} {} through relationship chain", 
                    source.module, source.name, target.module, target.name)
            },
            ImpactType::RequirementChange => {
                format!("Requirement changes in {} {} require updates to {} {}", 
                    source.module, source.name, target.module, target.name)
            },
            _ => {
                format!("Changes to {} {} may affect {} {}", 
                    source.module, source.name, target.module, target.name)
            }
        }
    }

    fn identify_affected_attributes(&self, target: &EntityReference, impact_type: &ImpactType) -> Vec<String> {
        // TODO: More sophisticated attribute analysis based on entity type and impact type
        match impact_type {
            ImpactType::RequirementChange => vec!["requirements".to_string(), "acceptance_criteria".to_string()],
            ImpactType::RiskChange => vec!["risk_level".to_string(), "mitigation_actions".to_string()],
            ImpactType::VerificationChange => vec!["test_methods".to_string(), "acceptance_criteria".to_string()],
            _ => vec!["metadata".to_string()],
        }
    }
}

/// Information about how an entity is impacted
#[derive(Debug, Clone)]
struct ImpactInfo {
    depth: u32,
    impact_type: ImpactType,
    connection_strength: f32,
}

impl Default for ImpactAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}