// Core entity types matching Rust backend

export type EntityType =
  | 'Task'
  | 'Milestone'
  | 'Resource'
  | 'Calendar'
  | 'Baseline'
  | 'Requirement'
  | 'Hazard'
  | 'Risk'
  | 'RiskControl'
  | 'Assembly'
  | 'Component'
  | 'Feature'
  | 'Mate'
  | 'Stackup'
  | 'Supplier'
  | 'Quote'
  | 'Verification'
  | 'Validation'
  | 'Manufacturing'
  | 'Comment'
  | 'General';

export type EntityStatus = 'Draft' | 'PendingApproval' | 'Approved' | 'Released';

export interface EntityMetadata {
  id: string;
  entity_type: EntityType;
  schema_version: string;
  created_at: string;
  updated_at: string;
  status: EntityStatus;
}

// Project Management Types
export type TaskType = 'EffortDriven' | 'DurationDriven' | 'WorkDriven';
export type SchedulingMode = 'Automatic' | 'Manual';
export type DependencyType = 'FinishToStart' | 'StartToStart' | 'FinishToFinish' | 'StartToFinish';
export type ResourceType = 'Labor' | 'FlatCost';

export interface ResourceAssignment {
  resource_id: string;
  allocated_hours: number;
}

export interface TaskDependency {
  predecessor_id: string;
  dependency_type: DependencyType;
  lag_days: number;
}

export interface TaskBaseline {
  baseline_id: string;
  start: string;
  end: string;
  effort: number;
  cost: number;
  percent_complete: number;
  dependencies: TaskDependency[];
}

export interface Task {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  scheduled_start: string;
  deadline: string;
  actual_start?: string;
  actual_end?: string;
  task_type: TaskType;
  scheduling_mode: SchedulingMode;
  percent_complete: number;
  percent_complete_history: Array<[string, number]>;
  assigned_resources: ResourceAssignment[];
  estimated_effort?: number;
  actual_cost?: number;
  calculated_cost?: number;
  dependencies: TaskDependency[];
  is_critical_path: boolean;
  slack?: number;
  baseline_data?: TaskBaseline;
}

export interface Milestone {
  metadata: EntityMetadata;
  name: string;
  description: string;
  date: string;
  dependencies: TaskDependency[];
  is_critical_path: boolean;
}

export interface Resource {
  metadata: EntityMetadata;
  name: string;
  description: string;
  email?: string;
  resource_type: ResourceType;
  bill_rate?: number;
  calendar_id?: string;
}

export interface Calendar {
  metadata: EntityMetadata;
  name: string;
  work_hours_per_day: number;
  work_days: string[];
  holidays: string[];
}

export interface Baseline {
  metadata: EntityMetadata;
  name: string;
  description: string;
  created_date: string;
  task_ids: string[];
}

// Requirements Types
export interface Requirement {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  requirement_type: string;
  rationale?: string;
  source?: string;
  verification_method?: string;
}

// Risk Management Types
export interface Hazard {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  causes: string[];
  harms: string[];
}

export interface Risk {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  risk_type: string;
  probability: number;
  severity: number;
  risk_score: number;
  residual_probability?: number;
  residual_severity?: number;
  residual_risk_score?: number;
}

export interface RiskControl {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  control_type: string;
}

// Design Types
export type FeatureType = 'External' | 'Internal';
export type DistributionType = 'Normal' | 'Uniform' | 'Triangular';
export type MateType = 'Clearance' | 'Transition' | 'InterferenceFit';
export type MateAnalysisResult = 'Pass' | 'Fail';
export type AnalysisType = 'WorstCase' | 'RSS' | 'MonteCarlo';
export type ContributionSign = 'Positive' | 'Negative';
export type CostDistribution = 'Linear' | 'Power' | 'Exponential' | 'Logarithmic';

export interface Assembly {
  metadata: EntityMetadata;
  name: string;
  description: string;
  revision: string;
  notes?: string;
}

export interface Component {
  metadata: EntityMetadata;
  name: string;
  description: string;
  revision: string;
  part_number?: string;
  material?: string;
  mass?: number;
  notes?: string;
}

export interface Feature {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  feature_type: FeatureType;
  nominal: number;
  upper_tolerance: number;
  lower_tolerance: number;
  distribution_type: DistributionType;
  custom_mean?: number;
  custom_std_dev?: number;
  drawing_location?: string;
}

export interface Mate {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  mate_type: MateType;
  mmc?: number;
  lmc?: number;
  analysis_result?: MateAnalysisResult;
}

export interface StackupResult {
  mean: number;
  upper: number;
  lower: number;
}

export interface MonteCarloResult {
  mean: number;
  median: number;
  std_dev: number;
  upper: number;
  lower: number;
  cp?: number;
  cpk?: number;
  ppm_failures?: number;
}

export interface StackupFeatureContribution {
  feature_id: string;
  sign: ContributionSign;
  contribution: number;
}

export interface Stackup {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  analysis_type: AnalysisType[];
  upper_spec_limit?: number;
  lower_spec_limit?: number;
  worst_case_result?: StackupResult;
  rss_result?: StackupResult;
  monte_carlo_result?: MonteCarloResult;
}

export interface Supplier {
  metadata: EntityMetadata;
  name: string;
  description: string;
  contact_name?: string;
  address?: string;
  phone?: string;
  email?: string;
  notes?: string;
}

export interface Quote {
  metadata: EntityMetadata;
  quote_number: string;
  quote_date: string;
  expiration_date?: string;
  quantity_price_pairs: Array<[number, number]>;
  distribution_type: CostDistribution;
}

// V&V Types
export interface Verification {
  metadata: EntityMetadata;
  name: string;
  description: string;
  revision: string;
  notes?: string;
}

export interface Validation {
  metadata: EntityMetadata;
  name: string;
  description: string;
  revision: string;
  notes?: string;
}

// Manufacturing Types
export interface Manufacturing {
  metadata: EntityMetadata;
  name: string;
  description: string;
  revision: string;
  notes?: string;
}

// Comment Types
export interface Comment {
  metadata: EntityMetadata;
  author_resource_id: string;
  content: string;
  parent_comment_id?: string;
  tagged_resource_ids: string[];
}

// General Types
export interface General {
  metadata: EntityMetadata;
  name: string;
  description: string;
  notes?: string;
  custom_type: string;
}

// Link Types
export type LinkType =
  | 'Related'
  | 'Parent'
  | 'Child'
  | 'Contains'
  | 'PartOf'
  | 'HasFeature'
  | 'Mates'
  | 'UsedInStackup'
  | 'Derives'
  | 'Satisfies'
  | 'Verifies'
  | 'Mitigates'
  | 'Hazardous'
  | 'Supplies'
  | 'Quotes'
  | 'Comments'
  | 'Replies'
  | 'Manufactures';

export interface LinkMetadata {
  quantity?: number;
  notes?: string;
}

export interface Link {
  id: string;
  from_entity_id: string;
  from_entity_type: EntityType;
  to_entity_id: string;
  to_entity_type: EntityType;
  link_type: LinkType;
  metadata?: LinkMetadata;
  created_at: string;
}

// Calculation Results
export interface CriticalPathResult {
  project_duration: number;
  critical_path: string[];
  task_slacks: Record<string, number>;
}

export interface EvmMetrics {
  planned_value: number;
  earned_value: number;
  actual_cost: number;
  cost_variance: number;
  schedule_variance: number;
  cost_performance_index: number;
  schedule_performance_index: number;
  estimate_at_completion: number;
  estimate_to_complete: number;
  variance_at_completion: number;
}

export interface BomItem {
  component_id: string;
  part_number?: string;
  description: string;
  revision: string;
  quantity: number;
  cost_per_unit: number;
  line_total: number;
}

export interface BomResult {
  assembly_id: string;
  volume: number;
  items: BomItem[];
  total_cost: number;
  has_interpolated_costs: boolean;
}

// Union type for all entities
export type Entity =
  | Task
  | Milestone
  | Resource
  | Calendar
  | Baseline
  | Requirement
  | Hazard
  | Risk
  | RiskControl
  | Assembly
  | Component
  | Feature
  | Mate
  | Stackup
  | Supplier
  | Quote
  | Verification
  | Validation
  | Manufacturing
  | Comment
  | General;

// Filter and Query Types
export interface EntityFilter {
  entity_type?: EntityType;
  status?: EntityStatus;
  search?: string;
  limit?: number;
  offset?: number;
}

// Dashboard Types
export interface DashboardMetrics {
  project_completion_percentage: number;
  estimated_end_date?: string;
  critical_path_most_behind_task?: string;
  risks_without_mitigations: number;
  requirements_without_verification: number;
  failed_tolerance_analyses: number;
  unread_comments: number;
}

export interface WarningItem {
  severity: 'high' | 'medium' | 'low';
  text: string;
  module: string;
  entity_id?: string;
}

export interface ActivityItem {
  user: string;
  action: string;
  entity: string;
  entity_id: string;
  time: string;
}
