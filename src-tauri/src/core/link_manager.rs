use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use crate::core::{EdtResult, EdtError};
use crate::models::{Link, LinkType, LinkMetadata, EntityType};

/// Manages entity relationships and links
pub struct LinkManager {
    /// In-memory graph for cycle detection and impact analysis
    graph: DiGraph<Uuid, LinkType>,
    /// Map from UUID to NodeIndex for quick lookups
    node_map: HashMap<Uuid, NodeIndex>,
    /// All links (in a real implementation, this would use storage)
    links: HashMap<Uuid, Link>,
}

impl LinkManager {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            links: HashMap::new(),
        }
    }

    /// Get or create a node in the graph for an entity
    fn get_or_create_node(&mut self, entity_id: Uuid) -> NodeIndex {
        if let Some(&node_idx) = self.node_map.get(&entity_id) {
            return node_idx;
        }

        let node_idx = self.graph.add_node(entity_id);
        self.node_map.insert(entity_id, node_idx);
        node_idx
    }

    /// Check if creating a link would create a cycle
    fn would_create_cycle(&self, from: Uuid, to: Uuid) -> bool {
        // Get node indices
        let from_idx = match self.node_map.get(&from) {
            Some(&idx) => idx,
            None => return false, // New node, can't create cycle
        };

        let to_idx = match self.node_map.get(&to) {
            Some(&idx) => idx,
            None => return false, // New node, can't create cycle
        };

        // Check if there's already a path from 'to' to 'from'
        // If so, adding from->to would create a cycle
        petgraph::algo::has_path_connecting(&self.graph, to_idx, from_idx, None)
    }

    /// Create a new link between entities
    pub fn create_link(
        &mut self,
        from_entity_id: Uuid,
        from_entity_type: EntityType,
        to_entity_id: Uuid,
        to_entity_type: EntityType,
        link_type: LinkType,
        metadata: Option<LinkMetadata>,
    ) -> EdtResult<Link> {
        // Validate: can't link entity to itself
        if from_entity_id == to_entity_id {
            return Err(EdtError::ValidationError(
                "Cannot link entity to itself".to_string()
            ));
        }

        // Check for cycles (for directional links)
        if self.would_create_cycle(from_entity_id, to_entity_id) {
            return Err(EdtError::ValidationError(
                "Link would create a cycle".to_string()
            ));
        }

        // Create the link
        let link = Link::new(
            from_entity_id,
            from_entity_type,
            to_entity_id,
            to_entity_type,
            link_type.clone(),
            metadata,
        );

        // Add to graph
        let from_node = self.get_or_create_node(from_entity_id);
        let to_node = self.get_or_create_node(to_entity_id);
        self.graph.add_edge(from_node, to_node, link_type);

        // Store link
        self.links.insert(link.id, link.clone());

        Ok(link)
    }

    /// Get a link by ID
    pub fn get_link(&self, link_id: &Uuid) -> Option<&Link> {
        self.links.get(link_id)
    }

    /// Get all links from an entity
    pub fn get_links_from(&self, entity_id: &Uuid) -> Vec<&Link> {
        self.links
            .values()
            .filter(|link| link.from_entity_id == *entity_id)
            .collect()
    }

    /// Get all links to an entity
    pub fn get_links_to(&self, entity_id: &Uuid) -> Vec<&Link> {
        self.links
            .values()
            .filter(|link| link.to_entity_id == *entity_id)
            .collect()
    }

    /// Get all links related to an entity (both from and to)
    pub fn get_all_links(&self, entity_id: &Uuid) -> Vec<&Link> {
        self.links
            .values()
            .filter(|link| {
                link.from_entity_id == *entity_id || link.to_entity_id == *entity_id
            })
            .collect()
    }

    /// Delete a link
    pub fn delete_link(&mut self, link_id: &Uuid) -> EdtResult<()> {
        if let Some(link) = self.links.remove(link_id) {
            // Remove from graph
            if let (Some(&from_node), Some(&to_node)) = (
                self.node_map.get(&link.from_entity_id),
                self.node_map.get(&link.to_entity_id),
            ) {
                // Find and remove the edge
                if let Some(edge) = self.graph.find_edge(from_node, to_node) {
                    self.graph.remove_edge(edge);
                }
            }
            Ok(())
        } else {
            Err(EdtError::EntityNotFound(format!("Link not found: {}", link_id)))
        }
    }

    /// Get impact analysis - all entities reachable from this entity
    pub fn get_impacted_entities(&self, entity_id: &Uuid) -> Vec<Uuid> {
        let node_idx = match self.node_map.get(entity_id) {
            Some(&idx) => idx,
            None => return vec![],
        };

        let mut visited = HashSet::new();
        let mut stack = vec![node_idx];
        let mut impacted = Vec::new();

        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                let entity_id = self.graph[current];
                impacted.push(entity_id);

                // Add all neighbors to stack
                for neighbor in self.graph.neighbors_directed(current, Direction::Outgoing) {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }

        impacted
    }

    /// Count total links
    pub fn link_count(&self) -> usize {
        self.links.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_link() {
        let mut manager = LinkManager::new();

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let link = manager
            .create_link(
                from_id,
                EntityType::Component,
                to_id,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        assert_eq!(link.from_entity_id, from_id);
        assert_eq!(link.to_entity_id, to_id);
        assert_eq!(link.link_type, LinkType::Satisfies);
        assert_eq!(manager.link_count(), 1);
    }

    #[test]
    fn test_create_link_self_reference() {
        let mut manager = LinkManager::new();

        let entity_id = Uuid::new_v4();

        let result = manager.create_link(
            entity_id,
            EntityType::Task,
            entity_id,
            EntityType::Task,
            LinkType::Related,
            None,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_create_link_with_cycle_detection() {
        let mut manager = LinkManager::new();

        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        // Create chain: A -> B -> C
        manager
            .create_link(a, EntityType::Task, b, EntityType::Task, LinkType::Parent, None)
            .unwrap();

        manager
            .create_link(b, EntityType::Task, c, EntityType::Task, LinkType::Parent, None)
            .unwrap();

        // Try to create C -> A, which would create a cycle
        let result = manager.create_link(
            c,
            EntityType::Task,
            a,
            EntityType::Task,
            LinkType::Parent,
            None,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EdtError::ValidationError(_)));
    }

    #[test]
    fn test_get_link() {
        let mut manager = LinkManager::new();

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let link = manager
            .create_link(
                from_id,
                EntityType::Component,
                to_id,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        let link_id = link.id;

        let retrieved = manager.get_link(&link_id).unwrap();
        assert_eq!(retrieved.id, link_id);
    }

    #[test]
    fn test_get_links_from() {
        let mut manager = LinkManager::new();

        let from_id = Uuid::new_v4();
        let to_id1 = Uuid::new_v4();
        let to_id2 = Uuid::new_v4();

        manager
            .create_link(
                from_id,
                EntityType::Component,
                to_id1,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        manager
            .create_link(
                from_id,
                EntityType::Component,
                to_id2,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        let links = manager.get_links_from(&from_id);
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_get_links_to() {
        let mut manager = LinkManager::new();

        let from_id1 = Uuid::new_v4();
        let from_id2 = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        manager
            .create_link(
                from_id1,
                EntityType::Component,
                to_id,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        manager
            .create_link(
                from_id2,
                EntityType::Component,
                to_id,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        let links = manager.get_links_to(&to_id);
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_get_all_links() {
        let mut manager = LinkManager::new();

        let entity_id = Uuid::new_v4();
        let other_id1 = Uuid::new_v4();
        let other_id2 = Uuid::new_v4();

        // Links from entity
        manager
            .create_link(
                entity_id,
                EntityType::Component,
                other_id1,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        // Links to entity
        manager
            .create_link(
                other_id2,
                EntityType::Component,
                entity_id,
                EntityType::Assembly,
                LinkType::PartOf,
                None,
            )
            .unwrap();

        let all_links = manager.get_all_links(&entity_id);
        assert_eq!(all_links.len(), 2);
    }

    #[test]
    fn test_delete_link() {
        let mut manager = LinkManager::new();

        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let link = manager
            .create_link(
                from_id,
                EntityType::Component,
                to_id,
                EntityType::Requirement,
                LinkType::Satisfies,
                None,
            )
            .unwrap();

        let link_id = link.id;
        assert_eq!(manager.link_count(), 1);

        manager.delete_link(&link_id).unwrap();
        assert_eq!(manager.link_count(), 0);
        assert!(manager.get_link(&link_id).is_none());
    }

    #[test]
    fn test_impact_analysis() {
        let mut manager = LinkManager::new();

        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        // Create tree: A -> B -> C
        //                    B -> D
        manager
            .create_link(a, EntityType::Task, b, EntityType::Task, LinkType::Parent, None)
            .unwrap();

        manager
            .create_link(b, EntityType::Task, c, EntityType::Task, LinkType::Parent, None)
            .unwrap();

        manager
            .create_link(b, EntityType::Task, d, EntityType::Task, LinkType::Parent, None)
            .unwrap();

        let impacted = manager.get_impacted_entities(&a);

        // Should include A, B, C, D
        assert_eq!(impacted.len(), 4);
        assert!(impacted.contains(&a));
        assert!(impacted.contains(&b));
        assert!(impacted.contains(&c));
        assert!(impacted.contains(&d));
    }

    #[test]
    fn test_impact_analysis_isolated_entity() {
        let manager = LinkManager::new();

        let entity_id = Uuid::new_v4();
        let impacted = manager.get_impacted_entities(&entity_id);

        // No links, should return empty
        assert_eq!(impacted.len(), 0);
    }

    #[test]
    fn test_link_with_metadata() {
        let mut manager = LinkManager::new();

        let assembly_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();

        let metadata = LinkMetadata {
            quantity: Some(5),
            notes: Some("5 units per assembly".to_string()),
        };

        let link = manager
            .create_link(
                assembly_id,
                EntityType::Assembly,
                component_id,
                EntityType::Component,
                LinkType::Contains,
                Some(metadata),
            )
            .unwrap();

        assert!(link.metadata.is_some());
        assert_eq!(link.metadata.as_ref().unwrap().quantity, Some(5));
    }
}
