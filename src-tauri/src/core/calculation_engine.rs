use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use crate::core::{EdtResult, EdtError, EntityManager, LinkManager};
use crate::models::{Task, EntityType, Stackup, StackupResult, MonteCarloResult, ContributionSign, DistributionType, LinkType, BomResult, BomItem, CostEstimate};
use serde::{Serialize, Deserialize};
use rand::Rng;
use rand::distributions::Distribution;
use statrs::distribution::{Normal, Uniform, Triangular};

/// Calculation engine for project management calculations
pub struct CalculationEngine {
    entity_manager: Arc<EntityManager>,
    link_manager: Arc<Mutex<LinkManager>>,
}

/// Result of Critical Path Method analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPathResult {
    pub project_duration: f64,  // Total project duration in days
    pub critical_path: Vec<Uuid>,  // Task IDs on the critical path
    pub task_slacks: HashMap<Uuid, f64>,  // Slack for each task (in days)
}

/// Earned Value Management metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmMetrics {
    pub planned_value: f64,  // PV (Budgeted Cost of Work Scheduled)
    pub earned_value: f64,  // EV (Budgeted Cost of Work Performed)
    pub actual_cost: f64,  // AC (Actual Cost of Work Performed)
    pub cost_variance: f64,  // CV = EV - AC
    pub schedule_variance: f64,  // SV = EV - PV
    pub cost_performance_index: f64,  // CPI = EV / AC
    pub schedule_performance_index: f64,  // SPI = EV / PV
    pub estimate_at_completion: f64,  // EAC = BAC / CPI
    pub estimate_to_complete: f64,  // ETC = EAC - AC
    pub variance_at_completion: f64,  // VAC = BAC - EAC
}

impl CalculationEngine {
    pub fn new(entity_manager: Arc<EntityManager>, link_manager: Arc<Mutex<LinkManager>>) -> Self {
        Self { entity_manager, link_manager }
    }

    /// Calculate critical path using CPM algorithm
    pub fn calculate_critical_path(&self) -> EdtResult<CriticalPathResult> {
        // 1. Get all tasks
        let task_ids = self.entity_manager.list_task_ids()?;
        let mut tasks = Vec::new();
        for id in &task_ids {
            tasks.push(self.entity_manager.get_task(id)?);
        }

        if tasks.is_empty() {
            return Ok(CriticalPathResult {
                project_duration: 0.0,
                critical_path: vec![],
                task_slacks: HashMap::new(),
            });
        }

        // 2. Build dependency graph
        let mut graph: DiGraph<Uuid, ()> = DiGraph::new();
        let mut node_map: HashMap<Uuid, NodeIndex> = HashMap::new();

        // Add all tasks as nodes
        for task in &tasks {
            let node = graph.add_node(task.metadata.id);
            node_map.insert(task.metadata.id, node);
        }

        // Add edges for dependencies
        for task in &tasks {
            let to_node = node_map[&task.metadata.id];
            for dep in &task.dependencies {
                if let Some(&from_node) = node_map.get(&dep.predecessor_id) {
                    graph.add_edge(from_node, to_node, ());
                }
            }
        }

        // 3. Topological sort
        let sorted = match petgraph::algo::toposort(&graph, None) {
            Ok(sorted) => sorted,
            Err(_) => {
                return Err(EdtError::CalculationError(
                    "Cycle detected in task dependencies".to_string()
                ));
            }
        };

        // 4. Calculate task durations
        let mut durations: HashMap<Uuid, f64> = HashMap::new();
        for task in &tasks {
            let duration = (task.deadline - task.scheduled_start).num_days() as f64;
            durations.insert(task.metadata.id, duration.max(0.0));
        }

        // 5. Forward pass - calculate earliest start and finish
        let mut earliest_start: HashMap<NodeIndex, f64> = HashMap::new();
        let mut earliest_finish: HashMap<NodeIndex, f64> = HashMap::new();

        for &node in &sorted {
            let task_id = graph[node];
            let duration = durations[&task_id];

            // Calculate earliest start: max of predecessors' earliest finish
            let es = graph
                .neighbors_directed(node, Direction::Incoming)
                .map(|pred| earliest_finish[&pred])
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            earliest_start.insert(node, es);
            earliest_finish.insert(node, es + duration);
        }

        // 6. Find project end time
        let project_end = earliest_finish
            .values()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0);

        // 7. Backward pass - calculate latest start and finish
        let mut latest_start: HashMap<NodeIndex, f64> = HashMap::new();
        let mut latest_finish: HashMap<NodeIndex, f64> = HashMap::new();

        for &node in sorted.iter().rev() {
            let task_id = graph[node];
            let duration = durations[&task_id];

            // Calculate latest finish: min of successors' latest start
            let lf = graph
                .neighbors_directed(node, Direction::Outgoing)
                .filter_map(|succ| latest_start.get(&succ))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .copied()
                .unwrap_or(project_end);

            latest_finish.insert(node, lf);
            latest_start.insert(node, lf - duration);
        }

        // 8. Calculate slack and identify critical path
        let mut critical_tasks = Vec::new();
        let mut task_slacks = HashMap::new();

        for &node in &sorted {
            let task_id = graph[node];
            let slack = latest_start[&node] - earliest_start[&node];
            task_slacks.insert(task_id, slack);

            // Critical path: tasks with near-zero slack (accounting for floating point precision)
            if slack.abs() < 0.001 {
                critical_tasks.push(task_id);
            }
        }

        Ok(CriticalPathResult {
            project_duration: project_end,
            critical_path: critical_tasks,
            task_slacks,
        })
    }

    /// Calculate Earned Value Management metrics
    pub fn calculate_evm(&self) -> EdtResult<EvmMetrics> {
        // Get all tasks
        let task_ids = self.entity_manager.list_task_ids()?;
        let mut tasks = Vec::new();
        for id in &task_ids {
            tasks.push(self.entity_manager.get_task(id)?);
        }

        // Calculate totals
        let mut planned_value = 0.0;
        let mut earned_value = 0.0;
        let mut actual_cost = 0.0;
        let mut budget_at_completion = 0.0;

        for task in &tasks {
            // Planned Value: estimated cost for work scheduled
            let task_budget = task.calculated_cost.unwrap_or(
                task.estimated_effort.unwrap_or(0.0) * 100.0  // Default rate if not set
            );
            budget_at_completion += task_budget;

            // Check if task should be started by now
            let now = chrono::Utc::now();
            if task.scheduled_start <= now {
                planned_value += task_budget;
            }

            // Earned Value: estimated cost * percent complete
            earned_value += task_budget * task.percent_complete;

            // Actual Cost: what has actually been spent
            actual_cost += task.actual_cost.unwrap_or(0.0);
        }

        // Calculate variances and indices
        let cost_variance = earned_value - actual_cost;
        let schedule_variance = earned_value - planned_value;

        let cost_performance_index = if actual_cost > 0.0 {
            earned_value / actual_cost
        } else {
            1.0
        };

        let schedule_performance_index = if planned_value > 0.0 {
            earned_value / planned_value
        } else {
            1.0
        };

        let estimate_at_completion = if cost_performance_index > 0.0 {
            budget_at_completion / cost_performance_index
        } else {
            budget_at_completion
        };

        let estimate_to_complete = estimate_at_completion - actual_cost;
        let variance_at_completion = budget_at_completion - estimate_at_completion;

        Ok(EvmMetrics {
            planned_value,
            earned_value,
            actual_cost,
            cost_variance,
            schedule_variance,
            cost_performance_index,
            schedule_performance_index,
            estimate_at_completion,
            estimate_to_complete,
            variance_at_completion,
        })
    }

    /// Calculate Worst Case tolerance stackup
    /// Sums all maximum tolerances to find the absolute worst-case scenario
    pub fn calculate_worst_case(&self, stackup_id: &Uuid) -> EdtResult<StackupResult> {
        let stackup = self.entity_manager.get_stackup(stackup_id)?;

        if stackup.feature_contributions.is_empty() {
            return Err(EdtError::CalculationError(
                "Stackup has no feature contributions".to_string()
            ));
        }

        let mut nominal_sum = 0.0;
        let mut upper_sum = 0.0;
        let mut lower_sum = 0.0;

        for contribution in &stackup.feature_contributions {
            let feature = self.entity_manager.get_feature(&contribution.feature_id)?;

            let nominal = feature.nominal * contribution.contribution;
            let upper_tol = feature.upper_tolerance * contribution.contribution;
            let lower_tol = feature.lower_tolerance * contribution.contribution;

            match contribution.sign {
                ContributionSign::Positive => {
                    nominal_sum += nominal;
                    upper_sum += upper_tol;
                    lower_sum += lower_tol;
                }
                ContributionSign::Negative => {
                    nominal_sum -= nominal;
                    // When negating, upper becomes lower and vice versa
                    upper_sum -= lower_tol;
                    lower_sum -= upper_tol;
                }
            }
        }

        Ok(StackupResult {
            mean: nominal_sum,
            upper: nominal_sum + upper_sum,
            lower: nominal_sum + lower_sum,
        })
    }

    /// Calculate RSS (Root Sum Square) tolerance stackup
    /// Statistical method assuming normal distributions and independence
    pub fn calculate_rss(&self, stackup_id: &Uuid) -> EdtResult<StackupResult> {
        let stackup = self.entity_manager.get_stackup(stackup_id)?;

        if stackup.feature_contributions.is_empty() {
            return Err(EdtError::CalculationError(
                "Stackup has no feature contributions".to_string()
            ));
        }

        let mut nominal_sum = 0.0;
        let mut variance_sum = 0.0;

        for contribution in &stackup.feature_contributions {
            let feature = self.entity_manager.get_feature(&contribution.feature_id)?;

            let nominal = feature.nominal * contribution.contribution;

            // Calculate standard deviation from tolerances
            // For normal distribution: tolerance range ≈ ±3σ (99.7% coverage)
            let tolerance_range = (feature.upper_tolerance - feature.lower_tolerance) / 2.0;
            let std_dev = tolerance_range / 3.0;
            let variance = std_dev * std_dev * contribution.contribution * contribution.contribution;

            match contribution.sign {
                ContributionSign::Positive => {
                    nominal_sum += nominal;
                }
                ContributionSign::Negative => {
                    nominal_sum -= nominal;
                }
            }

            // Variance is always positive (squared values)
            variance_sum += variance;
        }

        // RSS: ±3σ for 99.7% coverage
        let combined_std_dev = variance_sum.sqrt();
        let tolerance_spread = 3.0 * combined_std_dev;

        Ok(StackupResult {
            mean: nominal_sum,
            upper: nominal_sum + tolerance_spread,
            lower: nominal_sum - tolerance_spread,
        })
    }

    /// Calculate Monte Carlo tolerance stackup simulation
    /// Runs multiple iterations sampling from feature distributions
    pub fn calculate_monte_carlo(
        &self,
        stackup_id: &Uuid,
        iterations: usize
    ) -> EdtResult<MonteCarloResult> {
        let stackup = self.entity_manager.get_stackup(stackup_id)?;

        if stackup.feature_contributions.is_empty() {
            return Err(EdtError::CalculationError(
                "Stackup has no feature contributions".to_string()
            ));
        }

        if iterations < 1000 {
            return Err(EdtError::ValidationError(
                "Monte Carlo requires at least 1000 iterations".to_string()
            ));
        }

        let mut rng = rand::thread_rng();
        let mut results = Vec::with_capacity(iterations);

        // Get features and set up distributions
        let mut feature_data = Vec::new();
        for contribution in &stackup.feature_contributions {
            let feature = self.entity_manager.get_feature(&contribution.feature_id)?;
            feature_data.push((feature, contribution));
        }

        // Run Monte Carlo simulation
        for _ in 0..iterations {
            let mut sample_sum = 0.0;

            for (feature, contribution) in &feature_data {
                let sample_value = self.sample_feature_value(feature, &mut rng)?;
                let weighted_value = sample_value * contribution.contribution;

                match contribution.sign {
                    ContributionSign::Positive => sample_sum += weighted_value,
                    ContributionSign::Negative => sample_sum -= weighted_value,
                }
            }

            results.push(sample_sum);
        }

        // Calculate statistics
        results.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean = results.iter().sum::<f64>() / results.len() as f64;
        let median = results[results.len() / 2];

        let variance: f64 = results.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / results.len() as f64;
        let std_dev = variance.sqrt();

        // Percentiles for ±3σ equivalent (99.7% coverage)
        let lower_idx = (iterations as f64 * 0.00135).round() as usize;
        let upper_idx = (iterations as f64 * 0.99865).round() as usize;
        let lower = results[lower_idx];
        let upper = results[upper_idx];

        // Calculate process capability indices if spec limits are defined
        let (cp, cpk, ppm_failures) = if let (Some(usl), Some(lsl)) =
            (stackup.upper_spec_limit, stackup.lower_spec_limit) {

            let spec_range = usl - lsl;
            let process_spread = 6.0 * std_dev;
            let cp = spec_range / process_spread;

            let cpu = (usl - mean) / (3.0 * std_dev);
            let cpl = (mean - lsl) / (3.0 * std_dev);
            let cpk = cpu.min(cpl);

            // Count failures (out of spec)
            let failures = results.iter()
                .filter(|&&x| x < lsl || x > usl)
                .count();
            let ppm = (failures as f64 / iterations as f64) * 1_000_000.0;

            (Some(cp), Some(cpk), Some(ppm))
        } else {
            (None, None, None)
        };

        Ok(MonteCarloResult {
            mean,
            median,
            std_dev,
            upper,
            lower,
            cp,
            cpk,
            ppm_failures,
        })
    }

    /// Sample a value from a feature's distribution
    fn sample_feature_value<R: Rng>(&self, feature: &crate::models::Feature, rng: &mut R) -> EdtResult<f64> {
        let mean = feature.custom_mean.unwrap_or(feature.nominal);

        match feature.distribution_type {
            DistributionType::Normal => {
                // Calculate std dev from tolerance range or use custom
                let std_dev = if let Some(custom_std) = feature.custom_std_dev {
                    custom_std
                } else {
                    let tolerance_range = (feature.upper_tolerance - feature.lower_tolerance) / 2.0;
                    tolerance_range / 3.0  // ±3σ covers 99.7%
                };

                let normal = Normal::new(mean, std_dev)
                    .map_err(|e| EdtError::CalculationError(format!("Invalid normal distribution: {}", e)))?;

                Ok(normal.sample(rng))
            }
            DistributionType::Uniform => {
                let min = feature.nominal + feature.lower_tolerance;
                let max = feature.nominal + feature.upper_tolerance;

                let uniform = Uniform::new(min, max)
                    .map_err(|e| EdtError::CalculationError(format!("Invalid uniform distribution: {}", e)))?;

                Ok(uniform.sample(rng))
            }
            DistributionType::Triangular => {
                let min = feature.nominal + feature.lower_tolerance;
                let max = feature.nominal + feature.upper_tolerance;

                let triangular = Triangular::new(min, mean, max)
                    .map_err(|e| EdtError::CalculationError(format!("Invalid triangular distribution: {}", e)))?;

                Ok(triangular.sample(rng))
            }
        }
    }

    /// Calculate MMC/LMC for a mate between two features (shaft and hole)
    /// Returns (min_clearance, max_clearance) or (min_interference, max_interference)
    pub fn calculate_mate_analysis(
        &self,
        shaft_feature_id: &Uuid,
        hole_feature_id: &Uuid
    ) -> EdtResult<(f64, f64, f64, f64)> {
        let shaft = self.entity_manager.get_feature(shaft_feature_id)?;
        let hole = self.entity_manager.get_feature(hole_feature_id)?;

        // Validate feature types
        use crate::models::FeatureType;
        if shaft.feature_type != FeatureType::External {
            return Err(EdtError::ValidationError(
                "Shaft feature must be External type".to_string()
            ));
        }
        if hole.feature_type != FeatureType::Internal {
            return Err(EdtError::ValidationError(
                "Hole feature must be Internal type".to_string()
            ));
        }

        // For shaft (external feature):
        // MMC = largest size (nominal + upper tolerance)
        // LMC = smallest size (nominal + lower tolerance)
        let shaft_mmc = shaft.nominal + shaft.upper_tolerance;
        let shaft_lmc = shaft.nominal + shaft.lower_tolerance;

        // For hole (internal feature):
        // MMC = smallest size (nominal + lower tolerance) - most material removed
        // LMC = largest size (nominal + upper tolerance) - least material removed
        let hole_mmc = hole.nominal + hole.lower_tolerance;
        let hole_lmc = hole.nominal + hole.upper_tolerance;

        // Calculate clearance/interference:
        // Positive = clearance, Negative = interference
        // Min clearance: smallest hole - largest shaft (MMC condition)
        let min_clearance = hole_mmc - shaft_mmc;

        // Max clearance: largest hole - smallest shaft (LMC condition)
        let max_clearance = hole_lmc - shaft_lmc;

        // Return (shaft_mmc, shaft_lmc, hole_mmc, hole_lmc) for storage in Mate entity
        // Alternatively return (min_clearance, max_clearance)
        // Let's return both for flexibility: (min_clearance, max_clearance, shaft_mmc/hole_lmc, hole_mmc/shaft_lmc)
        Ok((min_clearance, max_clearance, shaft_mmc, hole_mmc))
    }

    /// Generate BOM (Bill of Materials) for an assembly at a specific volume
    /// Traverses assembly hierarchy and calculates costs from quotes
    pub fn generate_bom(&self, assembly_id: &Uuid, volume: u32) -> EdtResult<BomResult> {
        if volume == 0 {
            return Err(EdtError::ValidationError(
                "Volume must be greater than zero".to_string()
            ));
        }

        // Verify assembly exists
        let _assembly = self.entity_manager.get_assembly(assembly_id)?;

        let mut bom_items: Vec<BomItem> = Vec::new();
        let mut has_interpolated_costs = false;

        // Traverse assembly hierarchy
        self.traverse_assembly(
            assembly_id,
            1, // Initial multiplier
            volume,
            &mut bom_items,
            &mut has_interpolated_costs,
        )?;

        // Calculate total cost
        let total_cost: f64 = bom_items.iter().map(|item| item.line_total).sum();

        Ok(BomResult {
            assembly_id: *assembly_id,
            volume,
            items: bom_items,
            total_cost,
            has_interpolated_costs,
        })
    }

    /// Recursively traverse assembly hierarchy to build BOM
    fn traverse_assembly(
        &self,
        assembly_id: &Uuid,
        multiplier: u32,
        volume: u32,
        bom_items: &mut Vec<BomItem>,
        has_interpolated: &mut bool,
    ) -> EdtResult<()> {
        // Collect link data first, then release lock before processing
        let links_data = {
            let link_manager = self.link_manager.lock()
                .map_err(|e| EdtError::CalculationError(format!("Failed to lock link manager: {}", e)))?;

            // Get all components and sub-assemblies contained in this assembly
            let links = link_manager.get_links_from(assembly_id);

            // Clone the data we need
            links.iter().map(|link| {
                (
                    link.to_entity_id,
                    link.to_entity_type.clone(),
                    link.link_type.clone(),
                    link.metadata.as_ref().and_then(|m| m.quantity).unwrap_or(1),
                )
            }).collect::<Vec<_>>()
        }; // Lock is released here

        // Process links without holding the lock
        for (to_id, to_type, link_type, quantity) in links_data {
            // Check if this is a Contains link to a Component
            if link_type == LinkType::Contains && to_type == EntityType::Component {
                let component = self.entity_manager.get_component(&to_id)?;
                let total_quantity = quantity * multiplier;

                // Find quote for this component
                let cost_estimate = self.find_component_cost(&to_id, volume)?;
                let line_total = cost_estimate.cost_per_unit * (total_quantity as f64) * (volume as f64);

                if cost_estimate.is_interpolated {
                    *has_interpolated = true;
                }

                bom_items.push(BomItem {
                    component_id: to_id,
                    part_number: component.part_number.clone(),
                    description: component.description.clone(),
                    revision: component.revision.clone(),
                    quantity: total_quantity,
                    cost_per_unit: cost_estimate.cost_per_unit,
                    line_total,
                });
            }
            // Check if this is a Contains link to a sub-Assembly
            else if link_type == LinkType::Contains && to_type == EntityType::Assembly {
                let new_multiplier = quantity * multiplier;
                // Recursively traverse sub-assembly
                self.traverse_assembly(&to_id, new_multiplier, volume, bom_items, has_interpolated)?;
            }
        }

        Ok(())
    }

    /// Find cost for a component at given volume, with interpolation if needed
    fn find_component_cost(&self, component_id: &Uuid, volume: u32) -> EdtResult<CostEstimate> {
        let link_manager = self.link_manager.lock()
            .map_err(|e| EdtError::CalculationError(format!("Failed to lock link manager: {}", e)))?;

        // Find all quotes for this component
        let links = link_manager.get_links_to(component_id);

        let mut quotes = Vec::new();
        for link in links {
            if link.link_type == LinkType::Quotes && link.from_entity_type == EntityType::Quote {
                let quote = self.entity_manager.get_quote(&link.from_entity_id)?;
                quotes.push(quote);
            }
        }

        if quotes.is_empty() {
            return Ok(CostEstimate {
                cost_per_unit: 0.0,
                is_interpolated: false,
                r_squared: None,
            });
        }

        // Use the most recent quote (or could select based on other criteria)
        let quote = &quotes[0];

        // Check if we have exact volume match
        for (qty, price) in &quote.quantity_price_pairs {
            if *qty == volume {
                return Ok(CostEstimate {
                    cost_per_unit: *price,
                    is_interpolated: false,
                    r_squared: None,
                });
            }
        }

        // Need to interpolate
        drop(link_manager); // Release lock before interpolation
        self.interpolate_cost(quote, volume)
    }

    /// Interpolate cost for a volume using quote data
    pub fn interpolate_cost(&self, quote: &crate::models::Quote, volume: u32) -> EdtResult<CostEstimate> {
        if quote.quantity_price_pairs.is_empty() {
            return Err(EdtError::CalculationError(
                "Quote has no price data for interpolation".to_string()
            ));
        }

        // Sort pairs by quantity
        let mut pairs = quote.quantity_price_pairs.clone();
        pairs.sort_by_key(|p| p.0);

        // If volume is below minimum, use minimum price
        if volume < pairs[0].0 {
            return Ok(CostEstimate {
                cost_per_unit: pairs[0].1,
                is_interpolated: true,
                r_squared: None,
            });
        }

        // If volume is above maximum, use maximum price
        if volume > pairs[pairs.len() - 1].0 {
            return Ok(CostEstimate {
                cost_per_unit: pairs[pairs.len() - 1].1,
                is_interpolated: true,
                r_squared: None,
            });
        }

        // Find bracketing points
        let mut lower_idx = 0;
        for (i, (qty, _price)) in pairs.iter().enumerate() {
            if *qty <= volume {
                lower_idx = i;
            } else {
                break;
            }
        }

        let upper_idx = (lower_idx + 1).min(pairs.len() - 1);

        // Linear interpolation between two points
        let (q1, p1) = pairs[lower_idx];
        let (q2, p2) = pairs[upper_idx];

        // Check for exact match
        if q1 == volume {
            return Ok(CostEstimate {
                cost_per_unit: p1,
                is_interpolated: false,
                r_squared: None,
            });
        }

        if q2 == volume {
            return Ok(CostEstimate {
                cost_per_unit: p2,
                is_interpolated: false,
                r_squared: None,
            });
        }

        if q1 == q2 {
            return Ok(CostEstimate {
                cost_per_unit: p1,
                is_interpolated: true,
                r_squared: None,
            });
        }

        // Linear interpolation: p = p1 + (p2 - p1) * (v - q1) / (q2 - q1)
        let t = (volume as f64 - q1 as f64) / (q2 as f64 - q1 as f64);
        let interpolated_price = p1 + (p2 - p1) * t;

        // Calculate R² for linear fit (simplified - just for these two points)
        // For true R², we'd need to fit all points
        let r_squared = if pairs.len() > 2 {
            Some(self.calculate_r_squared(&pairs))
        } else {
            None
        };

        Ok(CostEstimate {
            cost_per_unit: interpolated_price,
            is_interpolated: true,
            r_squared,
        })
    }

    /// Calculate R² (coefficient of determination) for linear fit
    fn calculate_r_squared(&self, pairs: &[(u32, f64)]) -> f64 {
        if pairs.len() < 2 {
            return 0.0;
        }

        // Calculate means
        let n = pairs.len() as f64;
        let sum_x: f64 = pairs.iter().map(|(q, _)| *q as f64).sum();
        let sum_y: f64 = pairs.iter().map(|(_, p)| *p).sum();
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;

        // Calculate linear regression coefficients
        let sum_xy: f64 = pairs.iter().map(|(q, p)| (*q as f64) * p).sum();
        let sum_x2: f64 = pairs.iter().map(|(q, _)| (*q as f64).powi(2)).sum();

        let numerator = sum_xy - (sum_x * sum_y / n);
        let denominator = sum_x2 - (sum_x * sum_x / n);

        if denominator.abs() < 1e-10 {
            return 0.0;
        }

        let slope = numerator / denominator;
        let intercept = mean_y - slope * mean_x;

        // Calculate R²
        let ss_tot: f64 = pairs.iter()
            .map(|(_, p)| (p - mean_y).powi(2))
            .sum();

        let ss_res: f64 = pairs.iter()
            .map(|(q, p)| {
                let predicted = slope * (*q as f64) + intercept;
                (p - predicted).powi(2)
            })
            .sum();

        if ss_tot.abs() < 1e-10 {
            return 0.0;
        }

        (1.0 - (ss_res / ss_tot)).max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::RonStorage;
    use crate::models::TaskType;
    use tempfile::TempDir;
    use chrono::{Duration, Utc};

    fn create_test_engine() -> (TempDir, CalculationEngine) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(RonStorage::new(temp_dir.path()).unwrap());
        let entity_manager = Arc::new(EntityManager::new(storage));
        let link_manager = Arc::new(Mutex::new(LinkManager::new()));
        let engine = CalculationEngine::new(entity_manager, link_manager);
        (temp_dir, engine)
    }

    #[test]
    fn test_critical_path_empty() {
        let (_temp, engine) = create_test_engine();

        let result = engine.calculate_critical_path().unwrap();

        assert_eq!(result.project_duration, 0.0);
        assert_eq!(result.critical_path.len(), 0);
    }

    #[test]
    fn test_critical_path_single_task() {
        let (_temp, engine) = create_test_engine();

        let start = Utc::now();
        let end = start + Duration::days(5);

        let task = engine.entity_manager.create_task(
            "Task A".to_string(),
            "Single task".to_string(),
            start,
            end,
            TaskType::EffortDriven,
        ).unwrap();

        let result = engine.calculate_critical_path().unwrap();

        assert_eq!(result.project_duration, 5.0);
        assert_eq!(result.critical_path.len(), 1);
        assert!(result.critical_path.contains(&task.metadata.id));
        assert_eq!(result.task_slacks[&task.metadata.id], 0.0);
    }

    #[test]
    fn test_critical_path_linear_chain() {
        let (_temp, engine) = create_test_engine();

        let start = Utc::now();

        // Create Task A (5 days)
        let task_a = engine.entity_manager.create_task(
            "Task A".to_string(),
            "First task".to_string(),
            start,
            start + Duration::days(5),
            TaskType::EffortDriven,
        ).unwrap();

        // Create Task B (3 days), depends on A
        let mut task_b = engine.entity_manager.create_task(
            "Task B".to_string(),
            "Second task".to_string(),
            start + Duration::days(5),
            start + Duration::days(8),
            TaskType::EffortDriven,
        ).unwrap();
        task_b.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task_a.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });
        engine.entity_manager.update_task(task_b.clone()).unwrap();

        // Create Task C (4 days), depends on B
        let mut task_c = engine.entity_manager.create_task(
            "Task C".to_string(),
            "Third task".to_string(),
            start + Duration::days(8),
            start + Duration::days(12),
            TaskType::EffortDriven,
        ).unwrap();
        task_c.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task_b.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });
        engine.entity_manager.update_task(task_c.clone()).unwrap();

        let result = engine.calculate_critical_path().unwrap();

        // Total duration should be 5 + 3 + 4 = 12 days
        assert_eq!(result.project_duration, 12.0);

        // All tasks should be on critical path
        assert_eq!(result.critical_path.len(), 3);
        assert!(result.critical_path.contains(&task_a.metadata.id));
        assert!(result.critical_path.contains(&task_b.metadata.id));
        assert!(result.critical_path.contains(&task_c.metadata.id));

        // All tasks should have zero slack
        assert!(result.task_slacks[&task_a.metadata.id].abs() < 0.001);
        assert!(result.task_slacks[&task_b.metadata.id].abs() < 0.001);
        assert!(result.task_slacks[&task_c.metadata.id].abs() < 0.001);
    }

    #[test]
    fn test_critical_path_with_slack() {
        let (_temp, engine) = create_test_engine();

        let start = Utc::now();

        // Create diamond pattern:
        //     B(2 days)
        //   /           \
        // A(1)           D(1)
        //   \           /
        //     C(5 days)

        let task_a = engine.entity_manager.create_task(
            "Task A".to_string(),
            "Start".to_string(),
            start,
            start + Duration::days(1),
            TaskType::EffortDriven,
        ).unwrap();

        let mut task_b = engine.entity_manager.create_task(
            "Task B".to_string(),
            "Short path".to_string(),
            start + Duration::days(1),
            start + Duration::days(3),
            TaskType::EffortDriven,
        ).unwrap();
        task_b.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task_a.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });
        engine.entity_manager.update_task(task_b.clone()).unwrap();

        let mut task_c = engine.entity_manager.create_task(
            "Task C".to_string(),
            "Long path (critical)".to_string(),
            start + Duration::days(1),
            start + Duration::days(6),
            TaskType::EffortDriven,
        ).unwrap();
        task_c.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task_a.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });
        engine.entity_manager.update_task(task_c.clone()).unwrap();

        let mut task_d = engine.entity_manager.create_task(
            "Task D".to_string(),
            "End".to_string(),
            start + Duration::days(6),
            start + Duration::days(7),
            TaskType::EffortDriven,
        ).unwrap();
        task_d.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task_b.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });
        task_d.dependencies.push(crate::models::TaskDependency {
            predecessor_id: task_c.metadata.id,
            dependency_type: crate::models::DependencyType::FinishToStart,
            lag_days: 0.0,
        });
        engine.entity_manager.update_task(task_d.clone()).unwrap();

        let result = engine.calculate_critical_path().unwrap();

        // Critical path: A -> C -> D = 1 + 5 + 1 = 7 days
        assert_eq!(result.project_duration, 7.0);

        // Critical path should include A, C, D but not B
        assert!(result.critical_path.contains(&task_a.metadata.id));
        assert!(result.critical_path.contains(&task_c.metadata.id));
        assert!(result.critical_path.contains(&task_d.metadata.id));
        assert!(!result.critical_path.contains(&task_b.metadata.id));

        // Task B should have slack of 3 days (5 - 2)
        let slack_b = result.task_slacks[&task_b.metadata.id];
        assert!((slack_b - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_evm_no_tasks() {
        let (_temp, engine) = create_test_engine();

        let result = engine.calculate_evm().unwrap();

        assert_eq!(result.planned_value, 0.0);
        assert_eq!(result.earned_value, 0.0);
        assert_eq!(result.actual_cost, 0.0);
    }

    #[test]
    fn test_evm_single_task() {
        let (_temp, engine) = create_test_engine();

        let start = Utc::now() - Duration::days(5);  // Started in the past
        let end = start + Duration::days(10);

        let mut task = engine.entity_manager.create_task(
            "Task A".to_string(),
            "Test task".to_string(),
            start,
            end,
            TaskType::EffortDriven,
        ).unwrap();

        task.estimated_effort = Some(100.0);  // 100 hours
        task.calculated_cost = Some(10000.0);  // $10,000 budget
        task.percent_complete = 0.5;  // 50% complete
        task.actual_cost = Some(6000.0);  // Spent $6,000

        engine.entity_manager.update_task(task).unwrap();

        let result = engine.calculate_evm().unwrap();

        // PV: Task started, so PV = 10,000
        assert_eq!(result.planned_value, 10000.0);

        // EV: 50% complete, so EV = 5,000
        assert_eq!(result.earned_value, 5000.0);

        // AC: Actual cost = 6,000
        assert_eq!(result.actual_cost, 6000.0);

        // CV: EV - AC = 5,000 - 6,000 = -1,000 (over budget)
        assert_eq!(result.cost_variance, -1000.0);

        // SV: EV - PV = 5,000 - 10,000 = -5,000 (behind schedule)
        assert_eq!(result.schedule_variance, -5000.0);

        // CPI: EV / AC = 5,000 / 6,000 = 0.833 (over budget)
        assert!((result.cost_performance_index - 0.833).abs() < 0.01);

        // SPI: EV / PV = 5,000 / 10,000 = 0.5 (behind schedule)
        assert_eq!(result.schedule_performance_index, 0.5);
    }

    // ============================================================================
    // Tolerance Analysis Tests
    // ============================================================================

    #[test]
    fn test_worst_case_single_feature() {
        use crate::models::{FeatureType, DistributionType, StackupFeatureContribution, ContributionSign, AnalysisType};

        let (_temp, engine) = create_test_engine();

        // Create a feature: 10.0 +0.1/-0.2
        let feature = engine.entity_manager.create_feature(
            "Length".to_string(),
            "Part length".to_string(),
            FeatureType::External,
            10.0,
            0.1,
            -0.2,
            DistributionType::Normal,
        ).unwrap();

        // Create stackup with single feature
        let mut stackup = engine.entity_manager.create_stackup(
            "Simple Stackup".to_string(),
            "Single feature test".to_string(),
            vec![AnalysisType::WorstCase],
        ).unwrap();

        stackup.feature_contributions = vec![
            StackupFeatureContribution {
                feature_id: feature.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            }
        ];
        engine.entity_manager.update_stackup(stackup.clone()).unwrap();

        let result = engine.calculate_worst_case(&stackup.metadata.id).unwrap();

        // Mean should be nominal
        assert!((result.mean - 10.0).abs() < 0.001);
        // Upper: 10.0 + 0.1 = 10.1
        assert!((result.upper - 10.1).abs() < 0.001);
        // Lower: 10.0 + (-0.2) = 9.8
        assert!((result.lower - 9.8).abs() < 0.001);
    }

    #[test]
    fn test_worst_case_multiple_features() {
        use crate::models::{FeatureType, DistributionType, StackupFeatureContribution, ContributionSign, AnalysisType};

        let (_temp, engine) = create_test_engine();

        // Feature 1: 100.0 +0.5/-0.5
        let feature1 = engine.entity_manager.create_feature(
            "Length A".to_string(),
            "Part A length".to_string(),
            FeatureType::External,
            100.0,
            0.5,
            -0.5,
            DistributionType::Normal,
        ).unwrap();

        // Feature 2: 50.0 +0.3/-0.3
        let feature2 = engine.entity_manager.create_feature(
            "Length B".to_string(),
            "Part B length".to_string(),
            FeatureType::External,
            50.0,
            0.3,
            -0.3,
            DistributionType::Normal,
        ).unwrap();

        // Feature 3: 20.0 +0.2/-0.2 (negative contribution - subtracted)
        let feature3 = engine.entity_manager.create_feature(
            "Gap".to_string(),
            "Gap dimension".to_string(),
            FeatureType::External,
            20.0,
            0.2,
            -0.2,
            DistributionType::Normal,
        ).unwrap();

        // Create stackup: A + B - C
        let mut stackup = engine.entity_manager.create_stackup(
            "Complex Stackup".to_string(),
            "Multi-feature test".to_string(),
            vec![AnalysisType::WorstCase],
        ).unwrap();

        stackup.feature_contributions = vec![
            StackupFeatureContribution {
                feature_id: feature1.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            },
            StackupFeatureContribution {
                feature_id: feature2.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            },
            StackupFeatureContribution {
                feature_id: feature3.metadata.id,
                sign: ContributionSign::Negative,
                contribution: 1.0,
            },
        ];
        engine.entity_manager.update_stackup(stackup.clone()).unwrap();

        let result = engine.calculate_worst_case(&stackup.metadata.id).unwrap();

        // Mean: 100 + 50 - 20 = 130
        assert!((result.mean - 130.0).abs() < 0.001);

        // Upper worst case: 100.5 + 50.3 - 19.8 = 131.0
        assert!((result.upper - 131.0).abs() < 0.001);

        // Lower worst case: 99.5 + 49.7 - 20.2 = 129.0
        assert!((result.lower - 129.0).abs() < 0.001);
    }

    #[test]
    fn test_rss_calculation() {
        use crate::models::{FeatureType, DistributionType, StackupFeatureContribution, ContributionSign, AnalysisType};

        let (_temp, engine) = create_test_engine();

        // Create two features with same tolerance
        // Feature 1: 100.0 ±0.3
        let feature1 = engine.entity_manager.create_feature(
            "Dim 1".to_string(),
            "First dimension".to_string(),
            FeatureType::External,
            100.0,
            0.3,
            -0.3,
            DistributionType::Normal,
        ).unwrap();

        // Feature 2: 50.0 ±0.4
        let feature2 = engine.entity_manager.create_feature(
            "Dim 2".to_string(),
            "Second dimension".to_string(),
            FeatureType::External,
            50.0,
            0.4,
            -0.4,
            DistributionType::Normal,
        ).unwrap();

        let mut stackup = engine.entity_manager.create_stackup(
            "RSS Test".to_string(),
            "Test RSS calculation".to_string(),
            vec![AnalysisType::RSS],
        ).unwrap();

        stackup.feature_contributions = vec![
            StackupFeatureContribution {
                feature_id: feature1.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            },
            StackupFeatureContribution {
                feature_id: feature2.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            },
        ];
        engine.entity_manager.update_stackup(stackup.clone()).unwrap();

        let result = engine.calculate_rss(&stackup.metadata.id).unwrap();

        // Mean: 100 + 50 = 150
        assert!((result.mean - 150.0).abs() < 0.001);

        // RSS calculation:
        // σ1 = 0.3/3 = 0.1, σ2 = 0.4/3 = 0.133...
        // Combined σ = sqrt(0.1² + 0.133²) = sqrt(0.01 + 0.0178) = sqrt(0.0278) ≈ 0.1667
        // ±3σ = ±0.5 (approximately)
        let expected_spread = ((0.1_f64.powi(2) + (0.4 / 3.0_f64).powi(2)).sqrt()) * 3.0;

        assert!((result.upper - (150.0 + expected_spread)).abs() < 0.01);
        assert!((result.lower - (150.0 - expected_spread)).abs() < 0.01);

        // RSS should always be smaller than worst case for multiple features
        // Worst case total tolerance: 0.3 + 0.4 = 0.7
        // RSS tolerance should be less than 0.7
        let rss_range = result.upper - result.lower;
        assert!(rss_range < 1.4); // Worst case would be 1.4 total range
    }

    #[test]
    fn test_monte_carlo_basic() {
        use crate::models::{FeatureType, DistributionType, StackupFeatureContribution, ContributionSign, AnalysisType};

        let (_temp, engine) = create_test_engine();

        // Create feature: 10.0 ±0.5
        let feature = engine.entity_manager.create_feature(
            "Test Dim".to_string(),
            "Test dimension".to_string(),
            FeatureType::External,
            10.0,
            0.5,
            -0.5,
            DistributionType::Normal,
        ).unwrap();

        let mut stackup = engine.entity_manager.create_stackup(
            "MC Test".to_string(),
            "Monte Carlo test".to_string(),
            vec![AnalysisType::MonteCarlo],
        ).unwrap();

        stackup.feature_contributions = vec![
            StackupFeatureContribution {
                feature_id: feature.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            }
        ];
        engine.entity_manager.update_stackup(stackup.clone()).unwrap();

        let result = engine.calculate_monte_carlo(&stackup.metadata.id, 10000).unwrap();

        // Mean should be close to nominal (within 1% due to sampling)
        assert!((result.mean - 10.0).abs() < 0.1);

        // Median should be close to mean for normal distribution
        assert!((result.median - result.mean).abs() < 0.1);

        // Upper and lower should bracket the mean
        assert!(result.upper > result.mean);
        assert!(result.lower < result.mean);

        // Standard deviation should be reasonable (tolerance/3 ≈ 0.167)
        assert!(result.std_dev > 0.1 && result.std_dev < 0.25);
    }

    #[test]
    fn test_monte_carlo_with_spec_limits() {
        use crate::models::{FeatureType, DistributionType, StackupFeatureContribution, ContributionSign, AnalysisType};

        let (_temp, engine) = create_test_engine();

        // Create feature: 10.0 ±0.1
        let feature = engine.entity_manager.create_feature(
            "Critical Dim".to_string(),
            "Critical dimension".to_string(),
            FeatureType::External,
            10.0,
            0.1,
            -0.1,
            DistributionType::Normal,
        ).unwrap();

        let mut stackup = engine.entity_manager.create_stackup(
            "MC Spec Test".to_string(),
            "Monte Carlo with spec limits".to_string(),
            vec![AnalysisType::MonteCarlo],
        ).unwrap();

        // Set spec limits: 9.5 to 10.5 (generous limits)
        stackup.upper_spec_limit = Some(10.5);
        stackup.lower_spec_limit = Some(9.5);

        stackup.feature_contributions = vec![
            StackupFeatureContribution {
                feature_id: feature.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            }
        ];
        engine.entity_manager.update_stackup(stackup.clone()).unwrap();

        let result = engine.calculate_monte_carlo(&stackup.metadata.id, 10000).unwrap();

        // Cp and Cpk should be calculated
        assert!(result.cp.is_some());
        assert!(result.cpk.is_some());
        assert!(result.ppm_failures.is_some());

        // With generous limits, Cp should be good (> 1.0)
        assert!(result.cp.unwrap() > 1.0);

        // Cpk should be close to Cp for centered process
        assert!((result.cp.unwrap() - result.cpk.unwrap()).abs() < 0.5);

        // PPM failures should be very low
        assert!(result.ppm_failures.unwrap() < 1000.0); // Less than 0.1%
    }

    #[test]
    fn test_monte_carlo_minimum_iterations() {
        use crate::models::{FeatureType, DistributionType, StackupFeatureContribution, ContributionSign, AnalysisType};

        let (_temp, engine) = create_test_engine();

        let feature = engine.entity_manager.create_feature(
            "Test".to_string(),
            "Test".to_string(),
            FeatureType::External,
            10.0,
            0.1,
            -0.1,
            DistributionType::Normal,
        ).unwrap();

        let mut stackup = engine.entity_manager.create_stackup(
            "MC Min Test".to_string(),
            "Test minimum iterations".to_string(),
            vec![AnalysisType::MonteCarlo],
        ).unwrap();

        stackup.feature_contributions = vec![
            StackupFeatureContribution {
                feature_id: feature.metadata.id,
                sign: ContributionSign::Positive,
                contribution: 1.0,
            }
        ];
        engine.entity_manager.update_stackup(stackup.clone()).unwrap();

        // Should fail with too few iterations
        let result = engine.calculate_monte_carlo(&stackup.metadata.id, 500);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 1000"));
    }

    #[test]
    fn test_mate_analysis_clearance_fit() {
        use crate::models::{FeatureType, DistributionType};

        let (_temp, engine) = create_test_engine();

        // Shaft: 10.0 +0.0/-0.1 (9.9 to 10.0)
        let shaft = engine.entity_manager.create_feature(
            "Shaft".to_string(),
            "Shaft diameter".to_string(),
            FeatureType::External,
            10.0,
            0.0,
            -0.1,
            DistributionType::Normal,
        ).unwrap();

        // Hole: 10.2 +0.1/-0.0 (10.2 to 10.3)
        let hole = engine.entity_manager.create_feature(
            "Hole".to_string(),
            "Hole diameter".to_string(),
            FeatureType::Internal,
            10.2,
            0.1,
            0.0,
            DistributionType::Normal,
        ).unwrap();

        let (min_clearance, max_clearance, shaft_mmc, hole_mmc) =
            engine.calculate_mate_analysis(&shaft.metadata.id, &hole.metadata.id).unwrap();

        // Shaft MMC (largest): 10.0
        assert!((shaft_mmc - 10.0).abs() < 0.001);

        // Hole MMC (smallest): 10.2
        assert!((hole_mmc - 10.2).abs() < 0.001);

        // Min clearance: 10.2 - 10.0 = 0.2 (always fits)
        assert!((min_clearance - 0.2).abs() < 0.001);

        // Max clearance: 10.3 - 9.9 = 0.4
        assert!((max_clearance - 0.4).abs() < 0.001);

        // Positive clearances mean clearance fit
        assert!(min_clearance > 0.0);
        assert!(max_clearance > 0.0);
    }

    #[test]
    fn test_mate_analysis_interference_fit() {
        use crate::models::{FeatureType, DistributionType};

        let (_temp, engine) = create_test_engine();

        // Shaft: 10.1 +0.05/-0.0 (10.1 to 10.15) - oversized
        let shaft = engine.entity_manager.create_feature(
            "Shaft".to_string(),
            "Interference shaft".to_string(),
            FeatureType::External,
            10.1,
            0.05,
            0.0,
            DistributionType::Normal,
        ).unwrap();

        // Hole: 10.0 +0.0/-0.05 (9.95 to 10.0) - undersized
        let hole = engine.entity_manager.create_feature(
            "Hole".to_string(),
            "Interference hole".to_string(),
            FeatureType::Internal,
            10.0,
            0.0,
            -0.05,
            DistributionType::Normal,
        ).unwrap();

        let (min_clearance, max_clearance, _shaft_mmc, _hole_mmc) =
            engine.calculate_mate_analysis(&shaft.metadata.id, &hole.metadata.id).unwrap();

        // Min clearance: 9.95 - 10.15 = -0.2 (interference)
        assert!(min_clearance < 0.0);
        assert!((min_clearance + 0.2).abs() < 0.001);

        // Max clearance: 10.0 - 10.1 = -0.1 (always interference)
        assert!(max_clearance < 0.0);
        assert!((max_clearance + 0.1).abs() < 0.001);
    }

    #[test]
    fn test_mate_analysis_transition_fit() {
        use crate::models::{FeatureType, DistributionType};

        let (_temp, engine) = create_test_engine();

        // Shaft: 10.0 +0.05/-0.05
        let shaft = engine.entity_manager.create_feature(
            "Shaft".to_string(),
            "Transition shaft".to_string(),
            FeatureType::External,
            10.0,
            0.05,
            -0.05,
            DistributionType::Normal,
        ).unwrap();

        // Hole: 10.0 +0.05/-0.05 (same tolerance)
        let hole = engine.entity_manager.create_feature(
            "Hole".to_string(),
            "Transition hole".to_string(),
            FeatureType::Internal,
            10.0,
            0.05,
            -0.05,
            DistributionType::Normal,
        ).unwrap();

        let (min_clearance, max_clearance, _shaft_mmc, _hole_mmc) =
            engine.calculate_mate_analysis(&shaft.metadata.id, &hole.metadata.id).unwrap();

        // Min clearance: 9.95 - 10.05 = -0.1 (interference at MMC)
        assert!(min_clearance < 0.0);

        // Max clearance: 10.05 - 9.95 = 0.1 (clearance at LMC)
        assert!(max_clearance > 0.0);

        // Transition fit: sometimes clears, sometimes interferes
        assert!(min_clearance < 0.0 && max_clearance > 0.0);
    }

    #[test]
    fn test_mate_analysis_invalid_feature_types() {
        use crate::models::{FeatureType, DistributionType};

        let (_temp, engine) = create_test_engine();

        // Create two external features (both shafts)
        let shaft1 = engine.entity_manager.create_feature(
            "Shaft 1".to_string(),
            "First shaft".to_string(),
            FeatureType::External,
            10.0,
            0.1,
            -0.1,
            DistributionType::Normal,
        ).unwrap();

        let shaft2 = engine.entity_manager.create_feature(
            "Shaft 2".to_string(),
            "Second shaft".to_string(),
            FeatureType::External,
            10.0,
            0.1,
            -0.1,
            DistributionType::Normal,
        ).unwrap();

        // Should fail - both external
        let result = engine.calculate_mate_analysis(&shaft1.metadata.id, &shaft2.metadata.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Internal"));
    }

    #[test]
    fn test_worst_case_empty_contributions() {
        use crate::models::AnalysisType;

        let (_temp, engine) = create_test_engine();

        // Create stackup with no contributions
        let stackup = engine.entity_manager.create_stackup(
            "Empty Stackup".to_string(),
            "No contributions".to_string(),
            vec![AnalysisType::WorstCase],
        ).unwrap();

        let result = engine.calculate_worst_case(&stackup.metadata.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no feature contributions"));
    }

    // ============================================================================
    // BOM Generation and Cost Interpolation Tests
    // ============================================================================

    #[test]
    fn test_cost_interpolation_exact_match() {
        use crate::models::{Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, engine) = create_test_engine();

        // Create quote with specific volumes
        let quote = Quote {
            metadata: crate::models::EntityMetadata::new(EntityType::Quote),
            quote_number: "Q-001".to_string(),
            quote_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            expiration_date: None,
            quantity_price_pairs: vec![
                (100, 10.0),
                (500, 8.0),
                (1000, 6.0),
            ],
            distribution_type: CostDistribution::Linear,
            notes: None,
        };

        // Test exact match
        let result = engine.interpolate_cost(&quote, 500).unwrap();
        assert_eq!(result.cost_per_unit, 8.0);
        assert!(!result.is_interpolated);
    }

    #[test]
    fn test_cost_interpolation_between_points() {
        use crate::models::{Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, engine) = create_test_engine();

        let quote = Quote {
            metadata: crate::models::EntityMetadata::new(EntityType::Quote),
            quote_number: "Q-001".to_string(),
            quote_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            expiration_date: None,
            quantity_price_pairs: vec![
                (100, 10.0),
                (200, 8.0),
            ],
            distribution_type: CostDistribution::Linear,
            notes: None,
        };

        // Test interpolation at midpoint
        let result = engine.interpolate_cost(&quote, 150).unwrap();
        assert!(result.is_interpolated);
        // Linear interpolation: 10.0 + (8.0 - 10.0) * 0.5 = 9.0
        assert!((result.cost_per_unit - 9.0).abs() < 0.001);
    }

    #[test]
    fn test_cost_interpolation_below_minimum() {
        use crate::models::{Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, engine) = create_test_engine();

        let quote = Quote {
            metadata: crate::models::EntityMetadata::new(EntityType::Quote),
            quote_number: "Q-001".to_string(),
            quote_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            expiration_date: None,
            quantity_price_pairs: vec![(100, 10.0), (500, 8.0)],
            distribution_type: CostDistribution::Linear,
            notes: None,
        };

        // Test volume below minimum
        let result = engine.interpolate_cost(&quote, 50).unwrap();
        assert!(result.is_interpolated);
        assert_eq!(result.cost_per_unit, 10.0); // Should use minimum price
    }

    #[test]
    fn test_cost_interpolation_above_maximum() {
        use crate::models::{Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, engine) = create_test_engine();

        let quote = Quote {
            metadata: crate::models::EntityMetadata::new(EntityType::Quote),
            quote_number: "Q-001".to_string(),
            quote_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            expiration_date: None,
            quantity_price_pairs: vec![(100, 10.0), (500, 8.0)],
            distribution_type: CostDistribution::Linear,
            notes: None,
        };

        // Test volume above maximum
        let result = engine.interpolate_cost(&quote, 1000).unwrap();
        assert!(result.is_interpolated);
        assert_eq!(result.cost_per_unit, 8.0); // Should use maximum price
    }

    #[test]
    fn test_bom_generation_simple_assembly() {
        use crate::models::{LinkMetadata, Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, engine) = create_test_engine();

        // Create assembly
        let assembly = engine.entity_manager.create_assembly(
            "Main Assembly".to_string(),
            "Test assembly".to_string(),
            "A".to_string(),
        ).unwrap();

        // Create component
        let mut component = engine.entity_manager.create_component(
            "Bracket".to_string(),
            "Mounting bracket".to_string(),
            "1".to_string(),
        ).unwrap();
        component.part_number = Some("BKT-001".to_string());
        let component = engine.entity_manager.update_component(component).unwrap();

        // Create quote for component
        let quote = engine.entity_manager.create_quote(
            "Q-001".to_string(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            vec![(100, 5.0), (500, 4.0)],
            CostDistribution::Linear,
        ).unwrap();

        // Link quote to component
        {
            let mut link_mgr = engine.link_manager.lock().unwrap();
            link_mgr.create_link(
                quote.metadata.id,
                EntityType::Quote,
                component.metadata.id,
                EntityType::Component,
                LinkType::Quotes,
                None,
            ).unwrap();

            // Link component to assembly (quantity: 2)
            link_mgr.create_link(
                assembly.metadata.id,
                EntityType::Assembly,
                component.metadata.id,
                EntityType::Component,
                LinkType::Contains,
                Some(LinkMetadata {
                    quantity: Some(2),
                    notes: None,
                }),
            ).unwrap();
        }

        // Generate BOM for 100 units
        let bom = engine.generate_bom(&assembly.metadata.id, 100).unwrap();

        assert_eq!(bom.volume, 100);
        assert_eq!(bom.items.len(), 1);
        assert_eq!(bom.items[0].quantity, 2); // 2 per assembly
        assert_eq!(bom.items[0].cost_per_unit, 5.0); // Exact match at 100
        assert_eq!(bom.items[0].line_total, 2.0 * 5.0 * 100.0); // qty * price * volume
        assert_eq!(bom.total_cost, 1000.0);
        assert!(!bom.has_interpolated_costs);
    }

    #[test]
    fn test_bom_generation_hierarchical() {
        use crate::models::{LinkMetadata, Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, engine) = create_test_engine();

        // Create top-level assembly
        let top_assembly = engine.entity_manager.create_assembly(
            "Product".to_string(),
            "Complete product".to_string(),
            "A".to_string(),
        ).unwrap();

        // Create sub-assembly
        let sub_assembly = engine.entity_manager.create_assembly(
            "Sub Assembly".to_string(),
            "Module".to_string(),
            "A".to_string(),
        ).unwrap();

        // Create components
        let mut comp1 = engine.entity_manager.create_component(
            "Part 1".to_string(),
            "Component 1".to_string(),
            "1".to_string(),
        ).unwrap();
        comp1.part_number = Some("P1-001".to_string());
        let comp1 = engine.entity_manager.update_component(comp1).unwrap();

        let mut comp2 = engine.entity_manager.create_component(
            "Part 2".to_string(),
            "Component 2".to_string(),
            "1".to_string(),
        ).unwrap();
        comp2.part_number = Some("P2-001".to_string());
        let comp2 = engine.entity_manager.update_component(comp2).unwrap();

        // Create quotes
        let quote1 = engine.entity_manager.create_quote(
            "Q-001".to_string(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            vec![(100, 10.0)],
            CostDistribution::Linear,
        ).unwrap();

        let quote2 = engine.entity_manager.create_quote(
            "Q-002".to_string(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            vec![(100, 20.0)],
            CostDistribution::Linear,
        ).unwrap();

        {
            let mut link_mgr = engine.link_manager.lock().unwrap();

            // Link quotes to components
            link_mgr.create_link(
                quote1.metadata.id,
                EntityType::Quote,
                comp1.metadata.id,
                EntityType::Component,
                LinkType::Quotes,
                None,
            ).unwrap();

            link_mgr.create_link(
                quote2.metadata.id,
                EntityType::Quote,
                comp2.metadata.id,
                EntityType::Component,
                LinkType::Quotes,
                None,
            ).unwrap();

            // Link comp1 to sub-assembly (qty 3)
            link_mgr.create_link(
                sub_assembly.metadata.id,
                EntityType::Assembly,
                comp1.metadata.id,
                EntityType::Component,
                LinkType::Contains,
                Some(LinkMetadata {
                    quantity: Some(3),
                    notes: None,
                }),
            ).unwrap();

            // Link comp2 directly to top assembly (qty 1)
            link_mgr.create_link(
                top_assembly.metadata.id,
                EntityType::Assembly,
                comp2.metadata.id,
                EntityType::Component,
                LinkType::Contains,
                Some(LinkMetadata {
                    quantity: Some(1),
                    notes: None,
                }),
            ).unwrap();

            // Link sub-assembly to top assembly (qty 2)
            link_mgr.create_link(
                top_assembly.metadata.id,
                EntityType::Assembly,
                sub_assembly.metadata.id,
                EntityType::Assembly,
                LinkType::Contains,
                Some(LinkMetadata {
                    quantity: Some(2),
                    notes: None,
                }),
            ).unwrap();
        }

        // Generate BOM for 100 units
        let bom = engine.generate_bom(&top_assembly.metadata.id, 100).unwrap();

        assert_eq!(bom.volume, 100);
        // Should have comp1 (from sub-assembly) and comp2 (direct)
        assert_eq!(bom.items.len(), 2);

        // Find comp1 in BOM (3 per sub-assembly * 2 sub-assemblies = 6)
        let comp1_item = bom.items.iter().find(|i| i.component_id == comp1.metadata.id).unwrap();
        assert_eq!(comp1_item.quantity, 6);
        assert_eq!(comp1_item.cost_per_unit, 10.0);
        assert_eq!(comp1_item.line_total, 6.0 * 10.0 * 100.0);

        // Find comp2 in BOM (1 per top assembly)
        let comp2_item = bom.items.iter().find(|i| i.component_id == comp2.metadata.id).unwrap();
        assert_eq!(comp2_item.quantity, 1);
        assert_eq!(comp2_item.cost_per_unit, 20.0);
        assert_eq!(comp2_item.line_total, 1.0 * 20.0 * 100.0);

        // Total: (6 * 10 + 1 * 20) * 100 = 8000
        assert_eq!(bom.total_cost, 8000.0);
    }

    #[test]
    fn test_bom_generation_with_interpolation() {
        use crate::models::{LinkMetadata, Quote, CostDistribution};
        use chrono::NaiveDate;

        let (_temp, engine) = create_test_engine();

        let assembly = engine.entity_manager.create_assembly(
            "Assembly".to_string(),
            "Test".to_string(),
            "A".to_string(),
        ).unwrap();

        let mut component = engine.entity_manager.create_component(
            "Part".to_string(),
            "Component".to_string(),
            "1".to_string(),
        ).unwrap();
        component.part_number = Some("P-001".to_string());
        let component = engine.entity_manager.update_component(component).unwrap();

        // Quote with volumes 100 and 500
        let quote = engine.entity_manager.create_quote(
            "Q-001".to_string(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            vec![(100, 10.0), (500, 8.0)],
            CostDistribution::Linear,
        ).unwrap();

        {
            let mut link_mgr = engine.link_manager.lock().unwrap();

            link_mgr.create_link(
                quote.metadata.id,
                EntityType::Quote,
                component.metadata.id,
                EntityType::Component,
                LinkType::Quotes,
                None,
            ).unwrap();

            link_mgr.create_link(
                assembly.metadata.id,
                EntityType::Assembly,
                component.metadata.id,
                EntityType::Component,
                LinkType::Contains,
                Some(LinkMetadata {
                    quantity: Some(1),
                    notes: None,
                }),
            ).unwrap();
        }

        // Generate BOM for 300 units (requires interpolation)
        let bom = engine.generate_bom(&assembly.metadata.id, 300).unwrap();

        assert_eq!(bom.volume, 300);
        assert_eq!(bom.items.len(), 1);
        assert!(bom.has_interpolated_costs); // Should flag interpolation

        // Interpolated price at 300: 10.0 + (8.0 - 10.0) * (300 - 100) / (500 - 100) = 9.0
        assert!((bom.items[0].cost_per_unit - 9.0).abs() < 0.001);
    }

    #[test]
    fn test_bom_generation_empty_assembly() {
        let (_temp, engine) = create_test_engine();

        let assembly = engine.entity_manager.create_assembly(
            "Empty Assembly".to_string(),
            "No components".to_string(),
            "A".to_string(),
        ).unwrap();

        let bom = engine.generate_bom(&assembly.metadata.id, 100).unwrap();

        assert_eq!(bom.items.len(), 0);
        assert_eq!(bom.total_cost, 0.0);
    }

    #[test]
    fn test_bom_generation_zero_volume() {
        let (_temp, engine) = create_test_engine();

        let assembly = engine.entity_manager.create_assembly(
            "Assembly".to_string(),
            "Test".to_string(),
            "A".to_string(),
        ).unwrap();

        let result = engine.generate_bom(&assembly.metadata.id, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("greater than zero"));
    }

    #[test]
    fn test_bom_generation_component_without_quote() {
        use crate::models::LinkMetadata;

        let (_temp, engine) = create_test_engine();

        let assembly = engine.entity_manager.create_assembly(
            "Assembly".to_string(),
            "Test".to_string(),
            "A".to_string(),
        ).unwrap();

        let mut component = engine.entity_manager.create_component(
            "Part".to_string(),
            "Component without quote".to_string(),
            "1".to_string(),
        ).unwrap();
        component.part_number = Some("P-001".to_string());
        let component = engine.entity_manager.update_component(component).unwrap();

        {
            let mut link_mgr = engine.link_manager.lock().unwrap();

            link_mgr.create_link(
                assembly.metadata.id,
                EntityType::Assembly,
                component.metadata.id,
                EntityType::Component,
                LinkType::Contains,
                Some(LinkMetadata {
                    quantity: Some(1),
                    notes: None,
                }),
            ).unwrap();
        }

        let bom = engine.generate_bom(&assembly.metadata.id, 100).unwrap();

        assert_eq!(bom.items.len(), 1);
        assert_eq!(bom.items[0].cost_per_unit, 0.0); // No quote, cost is 0
        assert_eq!(bom.total_cost, 0.0);
    }
}
