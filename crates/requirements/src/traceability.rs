//! Traceability analysis for requirements management
//!
//! This module provides traceability analysis capabilities to track
//! relationships between requirements, design inputs, outputs, and verifications.

// Data types will be imported in tests where needed
use crate::repository::RequirementsRepository;
use tessera_core::Id;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Traceability matrix for requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityMatrix {
    pub requirements_to_inputs: HashMap<Id, Vec<Id>>,
    pub inputs_to_outputs: HashMap<Id, Vec<Id>>,
    pub inputs_to_verifications: HashMap<Id, Vec<Id>>,
    pub forward_links: HashMap<Id, HashSet<Id>>,
    pub backward_links: HashMap<Id, HashSet<Id>>,
}

impl TraceabilityMatrix {
    /// Create a new empty traceability matrix
    pub fn new() -> Self {
        Self {
            requirements_to_inputs: HashMap::new(),
            inputs_to_outputs: HashMap::new(),
            inputs_to_verifications: HashMap::new(),
            forward_links: HashMap::new(),
            backward_links: HashMap::new(),
        }
    }

    /// Build traceability matrix from repository
    pub fn build_from_repository(repository: &RequirementsRepository) -> Self {
        let mut matrix = Self::new();

        // Build requirements to inputs mapping
        for input in repository.get_design_inputs() {
            matrix.requirements_to_inputs
                .entry(input.requirement_id)
                .or_insert_with(Vec::new)
                .push(input.id);
                
            // Add to forward/backward links
            matrix.forward_links
                .entry(input.requirement_id)
                .or_insert_with(HashSet::new)
                .insert(input.id);
            matrix.backward_links
                .entry(input.id)
                .or_insert_with(HashSet::new)
                .insert(input.requirement_id);
        }

        // Build inputs to outputs mapping
        for output in repository.get_design_outputs() {
            for input_id in &output.input_ids {
                matrix.inputs_to_outputs
                    .entry(*input_id)
                    .or_insert_with(Vec::new)
                    .push(output.id);
                    
                // Add to forward/backward links
                matrix.forward_links
                    .entry(*input_id)
                    .or_insert_with(HashSet::new)
                    .insert(output.id);
                matrix.backward_links
                    .entry(output.id)
                    .or_insert_with(HashSet::new)
                    .insert(*input_id);
            }
        }

        // Build inputs to verifications mapping
        for verification in repository.get_verifications() {
            for input_id in &verification.input_ids {
                matrix.inputs_to_verifications
                    .entry(*input_id)
                    .or_insert_with(Vec::new)
                    .push(verification.id);
                    
                // Add to forward/backward links
                matrix.forward_links
                    .entry(*input_id)
                    .or_insert_with(HashSet::new)
                    .insert(verification.id);
                matrix.backward_links
                    .entry(verification.id)
                    .or_insert_with(HashSet::new)
                    .insert(*input_id);
            }
        }

        matrix
    }

    /// Get all entities linked forward from a given entity
    pub fn get_forward_links(&self, entity_id: &Id) -> Vec<Id> {
        self.forward_links
            .get(entity_id)
            .map(|links| links.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all entities linked backward from a given entity
    pub fn get_backward_links(&self, entity_id: &Id) -> Vec<Id> {
        self.backward_links
            .get(entity_id)
            .map(|links| links.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the complete traceability path from a requirement to its verifications
    pub fn get_requirement_path(&self, requirement_id: &Id) -> Vec<TraceabilityPath> {
        let mut paths = Vec::new();
        
        if let Some(inputs) = self.requirements_to_inputs.get(requirement_id) {
            for input_id in inputs {
                if let Some(outputs) = self.inputs_to_outputs.get(input_id) {
                    for output_id in outputs {
                        if let Some(verifications) = self.inputs_to_verifications.get(input_id) {
                            for verification_id in verifications {
                                paths.push(TraceabilityPath {
                                    requirement_id: *requirement_id,
                                    input_id: *input_id,
                                    output_id: *output_id,
                                    verification_id: *verification_id,
                                });
                            }
                        } else {
                            // Output without verification
                            paths.push(TraceabilityPath {
                                requirement_id: *requirement_id,
                                input_id: *input_id,
                                output_id: *output_id,
                                verification_id: Id::new(), // Placeholder for missing verification
                            });
                        }
                    }
                } else {
                    // Input without output
                    paths.push(TraceabilityPath {
                        requirement_id: *requirement_id,
                        input_id: *input_id,
                        output_id: Id::new(), // Placeholder for missing output
                        verification_id: Id::new(), // Placeholder for missing verification
                    });
                }
            }
        }
        
        paths
    }

    /// Find gaps in traceability (missing links)
    pub fn find_gaps(&self, repository: &RequirementsRepository) -> TraceabilityGaps {
        let mut gaps = TraceabilityGaps::new();

        // Find requirements without inputs
        for requirement in repository.get_requirements() {
            if !self.requirements_to_inputs.contains_key(&requirement.id) {
                gaps.requirements_without_inputs.push(requirement.id);
            }
        }

        // Find inputs without outputs
        for input in repository.get_design_inputs() {
            if !self.inputs_to_outputs.contains_key(&input.id) {
                gaps.inputs_without_outputs.push(input.id);
            }
        }

        // Find outputs without verifications (check if any of their linked inputs have verifications)
        for output in repository.get_design_outputs() {
            let has_verification = output.input_ids.iter()
                .any(|input_id| self.inputs_to_verifications.contains_key(input_id));
            if !has_verification {
                gaps.outputs_without_verifications.push(output.id);
            }
        }

        gaps
    }

    /// Get traceability statistics
    pub fn get_statistics(&self, repository: &RequirementsRepository) -> TraceabilityStatistics {
        let total_requirements = repository.get_requirements().len();
        let total_inputs = repository.get_design_inputs().len();
        let total_outputs = repository.get_design_outputs().len();
        let total_verifications = repository.get_verifications().len();

        let requirements_with_inputs = self.requirements_to_inputs.len();
        let inputs_with_outputs = self.inputs_to_outputs.len();
        let inputs_with_verifications = self.inputs_to_verifications.len();

        let input_coverage = if total_requirements > 0 {
            (requirements_with_inputs as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        let output_coverage = if total_inputs > 0 {
            (inputs_with_outputs as f64 / total_inputs as f64) * 100.0
        } else {
            0.0
        };

        let verification_coverage = if total_inputs > 0 {
            (inputs_with_verifications as f64 / total_inputs as f64) * 100.0
        } else {
            0.0
        };

        TraceabilityStatistics {
            total_requirements,
            total_inputs,
            total_outputs,
            total_verifications,
            requirements_with_inputs,
            inputs_with_outputs,
            inputs_with_verifications,
            input_coverage,
            output_coverage,
            verification_coverage,
        }
    }

    /// Generate traceability report
    pub fn generate_report(&self, repository: &RequirementsRepository) -> TraceabilityReport {
        let statistics = self.get_statistics(repository);
        let gaps = self.find_gaps(repository);
        let paths = self.get_complete_paths(repository);

        TraceabilityReport {
            statistics,
            gaps,
            paths,
        }
    }

    /// Get all complete traceability paths
    fn get_complete_paths(&self, repository: &RequirementsRepository) -> Vec<CompletePath> {
        let mut paths = Vec::new();

        for requirement in repository.get_requirements() {
            let req_paths = self.get_requirement_path(&requirement.id);
            for path in req_paths {
                // Only include complete paths (all IDs are valid)
                if let (Some(input), Some(output), Some(verification)) = (
                    repository.get_design_input(&path.input_id),
                    repository.get_design_output(&path.output_id),
                    repository.get_verification(&path.verification_id),
                ) {
                    paths.push(CompletePath {
                        requirement_name: requirement.name.clone(),
                        input_name: input.name.clone(),
                        output_name: output.name.clone(),
                        verification_name: verification.name.clone(),
                        path,
                    });
                }
            }
        }

        paths
    }
}

impl Default for TraceabilityMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// A single traceability path from requirement to verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityPath {
    pub requirement_id: Id,
    pub input_id: Id,
    pub output_id: Id,
    pub verification_id: Id,
}

/// Gaps in traceability coverage
#[derive(Debug, Clone)]
pub struct TraceabilityGaps {
    pub requirements_without_inputs: Vec<Id>,
    pub inputs_without_outputs: Vec<Id>,
    pub outputs_without_verifications: Vec<Id>,
}

impl TraceabilityGaps {
    /// Create a new empty gaps report
    pub fn new() -> Self {
        Self {
            requirements_without_inputs: Vec::new(),
            inputs_without_outputs: Vec::new(),
            outputs_without_verifications: Vec::new(),
        }
    }

    /// Check if there are any gaps
    pub fn has_gaps(&self) -> bool {
        !self.requirements_without_inputs.is_empty()
            || !self.inputs_without_outputs.is_empty()
            || !self.outputs_without_verifications.is_empty()
    }

    /// Get total number of gaps
    pub fn total_gaps(&self) -> usize {
        self.requirements_without_inputs.len()
            + self.inputs_without_outputs.len()
            + self.outputs_without_verifications.len()
    }
}

impl Default for TraceabilityGaps {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about traceability coverage
#[derive(Debug, Clone)]
pub struct TraceabilityStatistics {
    pub total_requirements: usize,
    pub total_inputs: usize,
    pub total_outputs: usize,
    pub total_verifications: usize,
    pub requirements_with_inputs: usize,
    pub inputs_with_outputs: usize,
    pub inputs_with_verifications: usize,
    pub input_coverage: f64,
    pub output_coverage: f64,
    pub verification_coverage: f64,
}

/// Complete traceability report
#[derive(Debug, Clone)]
pub struct TraceabilityReport {
    pub statistics: TraceabilityStatistics,
    pub gaps: TraceabilityGaps,
    pub paths: Vec<CompletePath>,
}

/// A complete traceability path with entity names
#[derive(Debug, Clone)]
pub struct CompletePath {
    pub requirement_name: String,
    pub input_name: String,
    pub output_name: String,
    pub verification_name: String,
    pub path: TraceabilityPath,
}

/// Traceability analyzer for advanced analysis
pub struct TraceabilityAnalyzer {
    matrix: TraceabilityMatrix,
}

impl TraceabilityAnalyzer {
    /// Create a new analyzer from repository
    pub fn new(repository: &RequirementsRepository) -> Self {
        let matrix = TraceabilityMatrix::build_from_repository(repository);
        Self { matrix }
    }

    /// Find orphaned entities (entities with no links)
    pub fn find_orphaned_entities(&self, repository: &RequirementsRepository) -> OrphanedEntities {
        let mut orphaned = OrphanedEntities::new();

        // Find orphaned requirements (no inputs)
        for requirement in repository.get_requirements() {
            if self.matrix.get_forward_links(&requirement.id).is_empty() {
                orphaned.requirements.push(requirement.id);
            }
        }

        // Find orphaned inputs (no backward or forward links)
        for input in repository.get_design_inputs() {
            if self.matrix.get_forward_links(&input.id).is_empty() {
                orphaned.inputs.push(input.id);
            }
        }

        // Find orphaned outputs (no forward links)
        for output in repository.get_design_outputs() {
            if self.matrix.get_forward_links(&output.id).is_empty() {
                orphaned.outputs.push(output.id);
            }
        }

        // Find orphaned verifications (no backward links - should not happen)
        for verification in repository.get_verifications() {
            if self.matrix.get_backward_links(&verification.id).is_empty() {
                orphaned.verifications.push(verification.id);
            }
        }

        orphaned
    }

    /// Find circular dependencies (should not exist in linear model)
    pub fn find_circular_dependencies(&self) -> Vec<Vec<Id>> {
        let mut circular_deps = Vec::new();
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for (entity_id, _) in &self.matrix.forward_links {
            if !visited.contains(entity_id) {
                if let Some(cycle) = self.detect_cycle(*entity_id, &mut visited, &mut recursion_stack) {
                    circular_deps.push(cycle);
                }
            }
        }

        circular_deps
    }

    /// Detect cycle starting from a given entity
    fn detect_cycle(
        &self,
        entity_id: Id,
        visited: &mut HashSet<Id>,
        recursion_stack: &mut HashSet<Id>,
    ) -> Option<Vec<Id>> {
        visited.insert(entity_id);
        recursion_stack.insert(entity_id);

        if let Some(forward_links) = self.matrix.forward_links.get(&entity_id) {
            for &linked_id in forward_links {
                if !visited.contains(&linked_id) {
                    if let Some(cycle) = self.detect_cycle(linked_id, visited, recursion_stack) {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(&linked_id) {
                    // Found a cycle
                    return Some(vec![entity_id, linked_id]);
                }
            }
        }

        recursion_stack.remove(&entity_id);
        None
    }

    /// Get impact analysis for a given entity
    pub fn get_impact_analysis(&self, entity_id: &Id) -> ImpactAnalysis {
        let mut impacted_entities = HashSet::new();
        let mut to_process = vec![*entity_id];

        while let Some(current_id) = to_process.pop() {
            if let Some(forward_links) = self.matrix.forward_links.get(&current_id) {
                for &linked_id in forward_links {
                    if !impacted_entities.contains(&linked_id) {
                        impacted_entities.insert(linked_id);
                        to_process.push(linked_id);
                    }
                }
            }
        }

        ImpactAnalysis {
            source_entity: *entity_id,
            impacted_entities: impacted_entities.into_iter().collect(),
        }
    }

    /// Get dependency analysis for a given entity
    pub fn get_dependency_analysis(&self, entity_id: &Id) -> DependencyAnalysis {
        let mut dependencies = HashSet::new();
        let mut to_process = vec![*entity_id];

        while let Some(current_id) = to_process.pop() {
            if let Some(backward_links) = self.matrix.backward_links.get(&current_id) {
                for &linked_id in backward_links {
                    if !dependencies.contains(&linked_id) {
                        dependencies.insert(linked_id);
                        to_process.push(linked_id);
                    }
                }
            }
        }

        DependencyAnalysis {
            target_entity: *entity_id,
            dependencies: dependencies.into_iter().collect(),
        }
    }
}

/// Orphaned entities (entities with no links)
#[derive(Debug, Clone)]
pub struct OrphanedEntities {
    pub requirements: Vec<Id>,
    pub inputs: Vec<Id>,
    pub outputs: Vec<Id>,
    pub verifications: Vec<Id>,
}

impl OrphanedEntities {
    /// Create a new empty orphaned entities report
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            verifications: Vec::new(),
        }
    }

    /// Check if there are any orphaned entities
    pub fn has_orphans(&self) -> bool {
        !self.requirements.is_empty()
            || !self.inputs.is_empty()
            || !self.outputs.is_empty()
            || !self.verifications.is_empty()
    }

    /// Get total number of orphaned entities
    pub fn total_orphans(&self) -> usize {
        self.requirements.len() + self.inputs.len() + self.outputs.len() + self.verifications.len()
    }
}

impl Default for OrphanedEntities {
    fn default() -> Self {
        Self::new()
    }
}

/// Impact analysis for entity changes
#[derive(Debug, Clone)]
pub struct ImpactAnalysis {
    pub source_entity: Id,
    pub impacted_entities: Vec<Id>,
}

/// Dependency analysis for entity changes
#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    pub target_entity: Id,
    pub dependencies: Vec<Id>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::RequirementsRepository;
    use crate::data::{Requirement, RequirementCategory, Priority, DesignInput, DesignOutput, Verification};

    #[test]
    fn test_traceability_matrix_building() {
        let mut repo = RequirementsRepository::new();
        
        // Create a complete chain
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req_id = req.id;
        repo.add_requirement(req).unwrap();

        let input = DesignInput::new(
            "Test Input".to_string(),
            "A test input".to_string(),
            req_id,
            "Test specification".to_string(),
        );
        let input_id = input.id;
        repo.add_design_input(input).unwrap();

        let output = DesignOutput::new(
            "Test Output".to_string(),
            "A test output".to_string(),
            input_id,
            "Document".to_string(),
            "Test deliverable".to_string(),
        );
        let output_id = output.id;
        repo.add_design_output(output).unwrap();

        let verification = Verification::new(
            "Test Verification".to_string(),
            "A test verification".to_string(),
            output_id,
            "Test".to_string(),
            "Automated Test".to_string(),
        );
        repo.add_verification(verification).unwrap();

        // Build matrix
        let matrix = TraceabilityMatrix::build_from_repository(&repo);

        // Test forward links
        assert!(matrix.requirements_to_inputs.contains_key(&req_id));
        assert!(matrix.inputs_to_outputs.contains_key(&input_id));
        assert!(matrix.outputs_to_verifications.contains_key(&output_id));

        // Test path tracing
        let paths = matrix.get_requirement_path(&req_id);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].requirement_id, req_id);
        assert_eq!(paths[0].input_id, input_id);
        assert_eq!(paths[0].output_id, output_id);
    }

    #[test]
    fn test_gap_detection() {
        let mut repo = RequirementsRepository::new();
        
        // Add requirement without inputs
        let req = Requirement::new(
            "Orphaned Requirement".to_string(),
            "An orphaned requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        repo.add_requirement(req).unwrap();

        let matrix = TraceabilityMatrix::build_from_repository(&repo);
        let gaps = matrix.find_gaps(&repo);

        assert_eq!(gaps.requirements_without_inputs.len(), 1);
        assert!(gaps.has_gaps());
        assert_eq!(gaps.total_gaps(), 1);
    }

    #[test]
    fn test_traceability_statistics() {
        let mut repo = RequirementsRepository::new();
        
        // Add complete chain
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req_id = req.id;
        repo.add_requirement(req).unwrap();

        let input = DesignInput::new(
            "Test Input".to_string(),
            "A test input".to_string(),
            req_id,
            "Test specification".to_string(),
        );
        let input_id = input.id;
        repo.add_design_input(input).unwrap();

        let output = DesignOutput::new(
            "Test Output".to_string(),
            "A test output".to_string(),
            input_id,
            "Document".to_string(),
            "Test deliverable".to_string(),
        );
        let output_id = output.id;
        repo.add_design_output(output).unwrap();

        let verification = Verification::new(
            "Test Verification".to_string(),
            "A test verification".to_string(),
            output_id,
            "Test".to_string(),
            "Automated Test".to_string(),
        );
        repo.add_verification(verification).unwrap();

        let matrix = TraceabilityMatrix::build_from_repository(&repo);
        let stats = matrix.get_statistics(&repo);

        assert_eq!(stats.total_requirements, 1);
        assert_eq!(stats.total_inputs, 1);
        assert_eq!(stats.total_outputs, 1);
        assert_eq!(stats.total_verifications, 1);
        assert_eq!(stats.input_coverage, 100.0);
        assert_eq!(stats.output_coverage, 100.0);
        assert_eq!(stats.verification_coverage, 100.0);
    }

    #[test]
    fn test_impact_analysis() {
        let mut repo = RequirementsRepository::new();
        
        // Create a chain with multiple paths
        let req = Requirement::new(
            "Test Requirement".to_string(),
            "A test requirement".to_string(),
            RequirementCategory::Functional,
            Priority::High,
        );
        let req_id = req.id;
        repo.add_requirement(req).unwrap();

        let input = DesignInput::new(
            "Test Input".to_string(),
            "A test input".to_string(),
            req_id,
            "Test specification".to_string(),
        );
        let input_id = input.id;
        repo.add_design_input(input).unwrap();

        let output = DesignOutput::new(
            "Test Output".to_string(),
            "A test output".to_string(),
            input_id,
            "Document".to_string(),
            "Test deliverable".to_string(),
        );
        let output_id = output.id;
        repo.add_design_output(output).unwrap();

        let verification = Verification::new(
            "Test Verification".to_string(),
            "A test verification".to_string(),
            output_id,
            "Test".to_string(),
            "Automated Test".to_string(),
        );
        repo.add_verification(verification).unwrap();

        let analyzer = TraceabilityAnalyzer::new(&repo);
        let impact = analyzer.get_impact_analysis(&req_id);

        // Requirement should impact input, output, and verification
        assert_eq!(impact.impacted_entities.len(), 3);
        assert!(impact.impacted_entities.contains(&input_id));
        assert!(impact.impacted_entities.contains(&output_id));
    }
}