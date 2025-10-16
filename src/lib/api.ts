/**
 * Tauri API client for Tessera
 * Provides type-safe wrappers around Tauri commands
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  Task, Milestone, Resource, Calendar, Baseline,
  Requirement, Hazard, Risk, RiskControl,
  Assembly, Component, Feature, Mate, Stackup, Supplier, Quote,
  Verification, Validation, Manufacturing,
  CriticalPathResult, EvmMetrics, BomResult,
  StackupResult, MonteCarloResult,
  AnalysisType
} from '@/types';

// Task API
export const taskApi = {
  create: (data: Partial<Task>) => invoke<Task>('create_task', { data }),
  get: (id: string) => invoke<Task>('get_task', { id }),
  update: (id: string, data: Partial<Task>) => invoke<Task>('update_task', { id, data }),
  delete: (id: string) => invoke<void>('delete_task', { id }),
  list: () => invoke<Task[]>('list_tasks'),
};

// Milestone API
export const milestoneApi = {
  create: (data: Partial<Milestone>) => invoke<Milestone>('create_milestone', { data }),
  get: (id: string) => invoke<Milestone>('get_milestone', { id }),
  update: (id: string, data: Partial<Milestone>) => invoke<Milestone>('update_milestone', { id, data }),
  delete: (id: string) => invoke<void>('delete_milestone', { id }),
  list: () => invoke<Milestone[]>('list_milestones'),
};

// Resource API
export const resourceApi = {
  create: (data: Partial<Resource>) => invoke<Resource>('create_resource', { data }),
  get: (id: string) => invoke<Resource>('get_resource', { id }),
  update: (id: string, data: Partial<Resource>) => invoke<Resource>('update_resource', { id, data }),
  delete: (id: string) => invoke<void>('delete_resource', { id }),
  list: () => invoke<Resource[]>('list_resources'),
};

// Calendar API
export const calendarApi = {
  create: (data: Partial<Calendar>) => invoke<Calendar>('create_calendar', { data }),
  get: (id: string) => invoke<Calendar>('get_calendar', { id }),
  update: (id: string, data: Partial<Calendar>) => invoke<Calendar>('update_calendar', { id, data }),
  delete: (id: string) => invoke<void>('delete_calendar', { id }),
  list: () => invoke<Calendar[]>('list_calendars'),
};

// Baseline API
export const baselineApi = {
  create: (data: Partial<Baseline>) => invoke<Baseline>('create_baseline', { data }),
  get: (id: string) => invoke<Baseline>('get_baseline', { id }),
  update: (id: string, data: Partial<Baseline>) => invoke<Baseline>('update_baseline', { id, data }),
  delete: (id: string) => invoke<void>('delete_baseline', { id }),
  list: () => invoke<Baseline[]>('list_baselines'),
};

// Requirement API
export const requirementApi = {
  create: (data: Partial<Requirement>) => invoke<Requirement>('create_requirement', { data }),
  get: (id: string) => invoke<Requirement>('get_requirement', { id }),
  update: (id: string, data: Partial<Requirement>) => invoke<Requirement>('update_requirement', { id, data }),
  delete: (id: string) => invoke<void>('delete_requirement', { id }),
  list: () => invoke<Requirement[]>('list_requirements'),
};

// Hazard API
export const hazardApi = {
  create: (data: Partial<Hazard>) => invoke<Hazard>('create_hazard', { data }),
  get: (id: string) => invoke<Hazard>('get_hazard', { id }),
  update: (id: string, data: Partial<Hazard>) => invoke<Hazard>('update_hazard', { id, data }),
  delete: (id: string) => invoke<void>('delete_hazard', { id }),
  list: () => invoke<Hazard[]>('list_hazards'),
};

// Risk API
export const riskApi = {
  create: (data: Partial<Risk>) => invoke<Risk>('create_risk', { data }),
  get: (id: string) => invoke<Risk>('get_risk', { id }),
  update: (id: string, data: Partial<Risk>) => invoke<Risk>('update_risk', { id, data }),
  delete: (id: string) => invoke<void>('delete_risk', { id }),
  list: () => invoke<Risk[]>('list_risks'),
};

// Risk Control API
export const riskControlApi = {
  create: (data: Partial<RiskControl>) => invoke<RiskControl>('create_risk_control', { data }),
  get: (id: string) => invoke<RiskControl>('get_risk_control', { id }),
  update: (id: string, data: Partial<RiskControl>) => invoke<RiskControl>('update_risk_control', { id, data }),
  delete: (id: string) => invoke<void>('delete_risk_control', { id }),
  list: () => invoke<RiskControl[]>('list_risk_controls'),
};

// Assembly API
export const assemblyApi = {
  create: (data: Partial<Assembly>) => invoke<Assembly>('create_assembly', { data }),
  get: (id: string) => invoke<Assembly>('get_assembly', { id }),
  update: (id: string, data: Partial<Assembly>) => invoke<Assembly>('update_assembly', { id, data }),
  delete: (id: string) => invoke<void>('delete_assembly', { id }),
  list: () => invoke<Assembly[]>('list_assemblies'),
};

// Component API
export const componentApi = {
  create: (data: Partial<Component>) => invoke<Component>('create_component', { data }),
  get: (id: string) => invoke<Component>('get_component', { id }),
  update: (id: string, data: Partial<Component>) => invoke<Component>('update_component', { id, data }),
  delete: (id: string) => invoke<void>('delete_component', { id }),
  list: () => invoke<Component[]>('list_components'),
};

// Feature API
export const featureApi = {
  create: (data: Partial<Feature>) => invoke<Feature>('create_feature', { data }),
  get: (id: string) => invoke<Feature>('get_feature', { id }),
  update: (id: string, data: Partial<Feature>) => invoke<Feature>('update_feature', { id, data }),
  delete: (id: string) => invoke<void>('delete_feature', { id }),
  list: () => invoke<Feature[]>('list_features'),
};

// Mate API
export const mateApi = {
  create: (data: Partial<Mate>) => invoke<Mate>('create_mate', { data }),
  get: (id: string) => invoke<Mate>('get_mate', { id }),
  update: (id: string, data: Partial<Mate>) => invoke<Mate>('update_mate', { id, data }),
  delete: (id: string) => invoke<void>('delete_mate', { id }),
  list: () => invoke<Mate[]>('list_mates'),
};

// Stackup API
export const stackupApi = {
  create: (data: Partial<Stackup>) => invoke<Stackup>('create_stackup', { data }),
  get: (id: string) => invoke<Stackup>('get_stackup', { id }),
  update: (id: string, data: Partial<Stackup>) => invoke<Stackup>('update_stackup', { id, data }),
  delete: (id: string) => invoke<void>('delete_stackup', { id }),
  list: () => invoke<Stackup[]>('list_stackups'),
};

// Supplier API
export const supplierApi = {
  create: (data: Partial<Supplier>) => invoke<Supplier>('create_supplier', { data }),
  get: (id: string) => invoke<Supplier>('get_supplier', { id }),
  update: (id: string, data: Partial<Supplier>) => invoke<Supplier>('update_supplier', { id, data }),
  delete: (id: string) => invoke<void>('delete_supplier', { id }),
  list: () => invoke<Supplier[]>('list_suppliers'),
};

// Quote API
export const quoteApi = {
  create: (data: Partial<Quote>) => invoke<Quote>('create_quote', { data }),
  get: (id: string) => invoke<Quote>('get_quote', { id }),
  update: (id: string, data: Partial<Quote>) => invoke<Quote>('update_quote', { id, data }),
  delete: (id: string) => invoke<void>('delete_quote', { id }),
  list: () => invoke<Quote[]>('list_quotes'),
};

// Verification API
export const verificationApi = {
  create: (data: Partial<Verification>) => invoke<Verification>('create_verification', { data }),
  get: (id: string) => invoke<Verification>('get_verification', { id }),
  update: (id: string, data: Partial<Verification>) => invoke<Verification>('update_verification', { id, data }),
  delete: (id: string) => invoke<void>('delete_verification', { id }),
  list: () => invoke<Verification[]>('list_verifications'),
};

// Validation API
export const validationApi = {
  create: (data: Partial<Validation>) => invoke<Validation>('create_validation', { data }),
  get: (id: string) => invoke<Validation>('get_validation', { id }),
  update: (id: string, data: Partial<Validation>) => invoke<Validation>('update_validation', { id, data }),
  delete: (id: string) => invoke<void>('delete_validation', { id }),
  list: () => invoke<Validation[]>('list_validations'),
};

// Manufacturing API
export const manufacturingApi = {
  create: (data: Partial<Manufacturing>) => invoke<Manufacturing>('create_manufacturing', { data }),
  get: (id: string) => invoke<Manufacturing>('get_manufacturing', { id }),
  update: (id: string, data: Partial<Manufacturing>) => invoke<Manufacturing>('update_manufacturing', { id, data }),
  delete: (id: string) => invoke<void>('delete_manufacturing', { id }),
  list: () => invoke<Manufacturing[]>('list_manufacturing'),
};

// Calculation API
export const calculationApi = {
  criticalPath: () => invoke<CriticalPathResult>('calculate_critical_path'),
  evm: () => invoke<EvmMetrics>('calculate_evm'),
  worstCase: (stackupId: string) => invoke<StackupResult>('calculate_worst_case', { stackupId }),
  rss: (stackupId: string) => invoke<StackupResult>('calculate_rss', { stackupId }),
  monteCarlo: (stackupId: string, numSamples: number) =>
    invoke<MonteCarloResult>('calculate_monte_carlo', { stackupId, numSamples }),
  generateBom: (assemblyId: string, volume: number) =>
    invoke<BomResult>('generate_bom', { assemblyId, volume }),
};

// Export all APIs
export const api = {
  task: taskApi,
  milestone: milestoneApi,
  resource: resourceApi,
  calendar: calendarApi,
  baseline: baselineApi,
  requirement: requirementApi,
  hazard: hazardApi,
  risk: riskApi,
  riskControl: riskControlApi,
  assembly: assemblyApi,
  component: componentApi,
  feature: featureApi,
  mate: mateApi,
  stackup: stackupApi,
  supplier: supplierApi,
  quote: quoteApi,
  verification: verificationApi,
  validation: validationApi,
  manufacturing: manufacturingApi,
  calculation: calculationApi,
};
